//! The two layers of the server: the outer layer for each request, and the
//! `/api/` layer for the session (ADR 0055 items 10 to 13, 19 and 20).

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use studio_core::vault::VaultFile;
use tower::ServiceExt;

use crate::accounts::{self, Accounts};
use crate::config::{Origin, Signup};
use crate::users::Users;

/// The name of the session cookie (ADR 0055 item 11).
pub const COOKIE_NAME: &str = "__Host-studio-session";

/// The seconds in a day.
const DAY: i64 = 86_400;
/// A session ends after this time with no request.
pub const IDLE_LIMIT: i64 = 7 * DAY;
/// A session ends after this time in total.
pub const TOTAL_LIMIT: i64 = 30 * DAY;
/// The server writes `last_seen` at most one time each period.
const TOUCH_GAP: i64 = 60;
/// The largest body of an auth route.
const MAX_AUTH_BODY: usize = 64 * 1024;

/// The state that each handler and layer shares.
#[derive(Debug)]
pub struct ServerState {
    /// `server.sqlite`.
    pub accounts: Accounts,
    /// The open users.
    pub users: Users,
    /// The public origin.
    pub origin: Origin,
    /// The sign-up mode.
    pub signup: Signup,
    /// A vault that no person can open. A sign-in with an unknown email
    /// address derives a key from it, so that the time is the same
    /// (ADR 0056 item 8).
    pub dummy_wrapper: VaultFile,
}

/// The shared state.
pub type Shared = Arc<ServerState>;

/// A JSON error response, in the form of the API errors.
pub fn failure(status: StatusCode, code: &str, message: &str) -> Response {
    let body = serde_json::json!({ "error": { "code": code, "message": message } });
    (status, axum::Json(body)).into_response()
}

/// A 500 response. The message names no secret.
pub fn internal(message: &str) -> Response {
    failure(StatusCode::INTERNAL_SERVER_ERROR, "internal", message)
}

/// The cookie header value of a new session.
#[must_use]
pub fn session_cookie(token: &str) -> String {
    format!("{COOKIE_NAME}={token}; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=2592000")
}

/// The cookie header value that removes the cookie.
#[must_use]
pub fn cleared_cookie() -> String {
    format!("{COOKIE_NAME}=; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=0")
}

/// The CSRF token of a session token (ADR 0055 item 12).
#[must_use]
pub fn csrf_of(token: &str) -> String {
    accounts::sha256_hex(&format!("studio-csrf:{token}"))
}

/// Read a JSON body. A different content type gives 415.
///
/// # Errors
///
/// Gives the response to send when the body is not usable.
pub async fn read_json<T: DeserializeOwned>(request: Request) -> Result<T, Response> {
    let is_json = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.split(';')
                .next()
                .is_some_and(|t| t.trim().eq_ignore_ascii_case("application/json"))
        });
    if !is_json {
        return Err(failure(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "send the body as application/json",
        ));
    }
    let bytes = axum::body::to_bytes(request.into_body(), MAX_AUTH_BODY)
        .await
        .map_err(|_| {
            failure(
                StatusCode::BAD_REQUEST,
                "bad_request",
                "the body is too large",
            )
        })?;
    // The error text of serde can repeat a value, so the server does not send it.
    serde_json::from_slice(&bytes).map_err(|_| {
        failure(
            StatusCode::BAD_REQUEST,
            "bad_request",
            "the body is not the JSON that the route needs",
        )
    })
}

fn host_of(request: &Request) -> Option<String> {
    request
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(str::to_ascii_lowercase)
        .or_else(|| {
            request
                .uri()
                .authority()
                .map(|a| a.as_str().to_ascii_lowercase())
        })
}

/// The outer layer: the `Host` check and the `Origin` check (ADR 0055
/// item 19).
pub async fn outer(State(state): State<Shared>, request: Request, next: Next) -> Response {
    if host_of(&request).as_deref() != Some(state.origin.authority.as_str()) {
        return failure(
            StatusCode::FORBIDDEN,
            "host_not_allowed",
            "the host is not the host of the public origin",
        );
    }
    if request.method() != Method::GET && request.method() != Method::HEAD {
        let origin = request
            .headers()
            .get(header::ORIGIN)
            .and_then(|v| v.to_str().ok())
            .map(str::to_ascii_lowercase);
        if origin.as_deref() != Some(state.origin.text.as_str()) {
            return failure(
                StatusCode::FORBIDDEN,
                "origin_not_allowed",
                "the origin is not the public origin",
            );
        }
    }
    next.run(request).await
}

/// The token in the cookie header, when it has the form of a token.
fn token_from(headers: &HeaderMap) -> Option<String> {
    for value in headers.get_all(header::COOKIE) {
        let Ok(text) = value.to_str() else { continue };
        for part in text.split(';') {
            let Some((name, token)) = part.trim().split_once('=') else {
                continue;
            };
            if name == COOKIE_NAME
                && token.len() == 64
                && token.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Some(token.to_owned());
            }
        }
    }
    None
}

/// An active session.
#[derive(Debug)]
pub struct Session {
    /// The session token. Do not log it.
    pub token: String,
    /// The account of the session.
    pub account_id: i64,
}

/// The state of the session of a request.
#[derive(Debug)]
pub enum Resolved {
    /// The request has no session cookie, or the database has no such session.
    Missing,
    /// The session ended. The server has removed the row.
    Ended,
    /// The session is active.
    Active(Session),
}

/// Find the session of a request, and count the request.
///
/// # Errors
///
/// Gives a 500 response when the database fails.
pub fn resolve(state: &ServerState, headers: &HeaderMap) -> Result<Resolved, Response> {
    let Some(token) = token_from(headers) else {
        return Ok(Resolved::Missing);
    };
    let hash = accounts::sha256_hex(&token);
    let db_error = |_: String| internal("the account database failed");
    let Some(row) = state.accounts.session(&hash).map_err(db_error)? else {
        return Ok(Resolved::Missing);
    };
    let now = accounts::now();
    if now - row.last_seen > IDLE_LIMIT || now - row.created_at > TOTAL_LIMIT {
        state.accounts.delete_session(&hash).map_err(db_error)?;
        return Ok(Resolved::Ended);
    }
    state
        .accounts
        .touch_session(&hash, now, TOUCH_GAP)
        .map_err(db_error)?;
    Ok(Resolved::Active(Session {
        token,
        account_id: row.account_id,
    }))
}

fn same(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0, |d, (x, y)| d | (x ^ y)) == 0
}

/// The `/api/` layer: find the session, check the CSRF header, and send
/// the request to the router of the user (ADR 0055 item 5).
pub async fn api(State(state): State<Shared>, request: Request) -> Response {
    let session = match resolve(&state, request.headers()) {
        Ok(Resolved::Active(session)) => session,
        Ok(Resolved::Ended) => {
            return failure(
                StatusCode::UNAUTHORIZED,
                "session_ended",
                "the session ended: sign in again",
            );
        }
        Ok(Resolved::Missing) => {
            return failure(
                StatusCode::UNAUTHORIZED,
                "session_required",
                "sign in first",
            );
        }
        Err(response) => return response,
    };
    let sent = request
        .headers()
        .get("x-studio-csrf")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !same(sent, &csrf_of(&session.token)) {
        return failure(
            StatusCode::FORBIDDEN,
            "csrf_required",
            "send the X-Studio-CSRF header of the session",
        );
    }
    let folder = match state.accounts.folder_of(session.account_id) {
        Ok(Some(folder)) => folder,
        Ok(None) => {
            return failure(
                StatusCode::UNAUTHORIZED,
                "session_required",
                "the account does not exist",
            );
        }
        Err(_) => return internal("the account database failed"),
    };
    let user = match state.users.get(session.account_id, &folder).await {
        Ok(user) => user,
        Err(error) => return error.into_response(),
    };
    match user.router.oneshot(request).await {
        Ok(response) => response,
        Err(never) => match never {},
    }
}

/// The fallback of the router: `/api/` goes to the `/api/` layer, and each
/// other path gives 404. `/app/v1/` gives 404 on the server in this packet
/// (task 034c).
pub async fn dispatch(State(state): State<Shared>, request: Request) -> Response {
    if request.uri().path().starts_with("/api/") {
        api(State(state), request).await
    } else {
        failure(StatusCode::NOT_FOUND, "not_found", "no such route")
    }
}

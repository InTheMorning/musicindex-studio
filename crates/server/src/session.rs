//! The two layers of the server: the outer layer for each request, and the
//! `/api/` layer for the session (ADR 0055 items 10 to 13, 19 and 20).

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use studio_core::vault::VaultFile;
use tokio::sync::Semaphore;
use tower::ServiceExt;

use crate::accounts::{self, Accounts};
use crate::config::{Origin, Signup};
use crate::limits::{self, Attempts};
use crate::nip46_sessions::Nip46Sessions;
use crate::users::Users;

/// The name of the session cookie (ADR 0055 item 11).
pub const COOKIE_NAME: &str = "__Host-studio-session";

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
    /// A session ends after this many seconds with no request.
    pub session_idle: i64,
    /// A session ends after this many seconds in total.
    pub session_max: i64,
    /// The vault of a user locks after this time with no use of a secret.
    pub vault_idle: Duration,
    /// The permits of the Argon2id derivations (ADR 0055 item 22).
    pub argon2: Arc<Semaphore>,
    /// How long a request waits for a permit.
    pub argon2_wait: Duration,
    /// The failed sign-ins (ADR 0055 item 23).
    pub attempts: Attempts,
    /// The header that gives the client address, when the operator names one.
    pub client_header: Option<String>,
    /// The NIP-46 connections (ADR 0056 items 12 to 16).
    pub nip46: Nip46Sessions,
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
    /// True when the session can only add a new sign-in method.
    pub restricted: bool,
}

/// The routes that a restricted session reaches, as a method and a path
/// (ADR 0056 item 3). This is the one list. A route that needs a session
/// and is not here gives 403 `method_required`.
const RESTRICTED_ROUTES: &[(&str, &str)] = &[
    ("GET", "/api/v1/status"),
    ("POST", "/api/v1/vault/passphrase"),
    ("POST", "/api/v1/vault/signer-key"),
    ("POST", "/api/v1/vault/nostr-wrapper"),
    ("GET", "/auth/v1/session"),
    ("POST", "/auth/v1/signout"),
    ("POST", "/auth/v1/passkeys/options"),
    ("POST", "/auth/v1/passkeys"),
];

/// The routes whose success adds a sign-in method. A response 2xx of one of
/// them removes the limit of the session. `POST /auth/v1/passkeys` is not
/// here: its handler removes the limit itself, after the checks.
const METHOD_ROUTES: &[&str] = &["/api/v1/vault/passphrase", "/api/v1/vault/nostr-wrapper"];

/// Give 403 `method_required` when the session is restricted and the route
/// is not in the list of [`RESTRICTED_ROUTES`].
///
/// # Errors
///
/// Gives the 403 response.
pub fn check_restricted(session: &Session, method: &Method, path: &str) -> Result<(), Response> {
    if !session.restricted
        || RESTRICTED_ROUTES
            .iter()
            .any(|(m, p)| *m == method.as_str() && *p == path)
    {
        return Ok(());
    }
    Err(failure(
        StatusCode::FORBIDDEN,
        "method_required",
        "add a new sign-in method first",
    ))
}

/// True when a response 2xx for this request adds a sign-in method.
fn adds_method(method: &Method, path: &str) -> bool {
    method == Method::POST && METHOD_ROUTES.contains(&path)
}

/// A restricted session must add a method that signs in to its own
/// account: a passphrase for an account with an email address, or a Nostr
/// wrapper for the public key of a Nostr account (ADR 0056 item 29). The
/// function gives the request again, with its body.
async fn usable_method(
    state: &ServerState,
    session: &Session,
    request: Request,
) -> Result<Request, Response> {
    let refused = || {
        failure(
            StatusCode::FORBIDDEN,
            "method_not_usable",
            "this method cannot sign in to the account",
        )
    };
    let db_error = |_: String| internal("the account database failed");
    if request.uri().path() == "/api/v1/vault/passphrase" {
        let email = state
            .accounts
            .email_of(session.account_id)
            .map_err(db_error)?;
        return if email.is_some() {
            Ok(request)
        } else {
            Err(refused())
        };
    }
    let Some(pubkey) = state
        .accounts
        .pubkey_of(session.account_id)
        .map_err(db_error)?
    else {
        return Err(refused());
    };
    let (parts, body) = request.into_parts();
    let bytes = axum::body::to_bytes(body, MAX_AUTH_BODY)
        .await
        .map_err(|_| {
            failure(
                StatusCode::BAD_REQUEST,
                "bad_request",
                "the body is too large",
            )
        })?;
    let given = serde_json::from_slice::<serde_json::Value>(&bytes)
        .ok()
        .and_then(|value| value["pubkey"].as_str().map(str::to_ascii_lowercase));
    if given.as_deref() != Some(pubkey.as_str()) {
        return Err(refused());
    }
    Ok(Request::from_parts(parts, axum::body::Body::from(bytes)))
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

/// Find the session of a request, and count the request. A session that
/// is over goes out, and when it was the last session of the user, the
/// vault locks (ADR 0055 item 16).
///
/// # Errors
///
/// Gives a 500 response when the database fails.
pub async fn resolve(state: &ServerState, headers: &HeaderMap) -> Result<Resolved, Response> {
    let Some(token) = token_from(headers) else {
        return Ok(Resolved::Missing);
    };
    let hash = accounts::sha256_hex(&token);
    let db_error = |_: String| internal("the account database failed");
    let Some(row) = state.accounts.session(&hash).map_err(db_error)? else {
        return Ok(Resolved::Missing);
    };
    let now = accounts::now();
    if now - row.last_seen > state.session_idle || now - row.created_at > state.session_max {
        state.accounts.delete_session(&hash).map_err(db_error)?;
        state.nip46.remove_session(&hash);
        lock_if_no_session(state, row.account_id).await;
        return Ok(Resolved::Ended);
    }
    state
        .accounts
        .touch_session(&hash, now, TOUCH_GAP)
        .map_err(db_error)?;
    Ok(Resolved::Active(Session {
        token,
        account_id: row.account_id,
        restricted: row.restricted,
    }))
}

/// Lock the vault of an account that has no session. The call does
/// nothing for an account that is not open, because its vault is closed.
pub async fn lock_if_no_session(state: &ServerState, account_id: i64) {
    let over = state
        .accounts
        .has_session(account_id, state.session_idle, state.session_max)
        .is_ok_and(|active| !active);
    if over && let Some(user) = state.users.peek(account_id).await {
        let _ = user.handle.lock().await;
    }
}

/// The active session of a request, or the response to send.
///
/// # Errors
///
/// Gives 401 for a request with no session or an ended session.
pub async fn active_session(state: &ServerState, headers: &HeaderMap) -> Result<Session, Response> {
    match resolve(state, headers).await? {
        Resolved::Active(session) => Ok(session),
        Resolved::Ended => Err(failure(
            StatusCode::UNAUTHORIZED,
            "session_ended",
            "the session ended: sign in again",
        )),
        Resolved::Missing => Err(failure(
            StatusCode::UNAUTHORIZED,
            "session_required",
            "sign in first",
        )),
    }
}

/// Check the `X-Studio-CSRF` header against the session (ADR 0055 item 12).
///
/// # Errors
///
/// Gives 403 `csrf_required` when the header is missing or not correct.
pub fn check_csrf(session: &Session, headers: &HeaderMap) -> Result<(), Response> {
    let sent = headers
        .get("x-studio-csrf")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if same(sent, &csrf_of(&session.token)) {
        Ok(())
    } else {
        Err(failure(
            StatusCode::FORBIDDEN,
            "csrf_required",
            "send the X-Studio-CSRF header of the session",
        ))
    }
}

/// Compare two texts in constant time for equal lengths.
pub fn same(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0, |d, (x, y)| d | (x ^ y)) == 0
}

/// The `/api/` layer: find the session, check the CSRF header, and send
/// the request to the router of the user (ADR 0055 item 5).
pub async fn api(State(state): State<Shared>, request: Request) -> Response {
    let session = match active_session(&state, request.headers()).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    if let Err(response) = check_csrf(&session, request.headers()) {
        return response;
    }
    if let Err(response) = check_restricted(&session, request.method(), request.uri().path()) {
        return response;
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
    // An unlock with a passphrase derives a key with Argon2id. The permit
    // stays until the router has answered (ADR 0055 item 22).
    let _permit = if needs_derivation(&request) {
        match limits::derivation_permit(&state).await {
            Ok(permit) => Some(permit),
            Err(response) => return response,
        }
    } else {
        None
    };
    let lifts = session.restricted && adds_method(request.method(), request.uri().path());
    let request = if lifts {
        match usable_method(&state, &session, request).await {
            Ok(request) => request,
            Err(response) => return response,
        }
    } else {
        request
    };
    let response = match user.router.oneshot(request).await {
        Ok(response) => response,
        Err(never) => match never {},
    };
    // The limit ends only when the method was added.
    if lifts && response.status().is_success() {
        let hash = accounts::sha256_hex(&session.token);
        if state.accounts.lift_restriction(&hash).is_err() {
            return internal("the account database failed");
        }
    }
    response
}

/// True for the request that derives a key with Argon2id: the unlock with
/// a passphrase and the change of the passphrase.
fn needs_derivation(request: &Request) -> bool {
    request.method() == Method::POST
        && matches!(
            request.uri().path(),
            "/api/v1/vault/unlock" | "/api/v1/vault/passphrase"
        )
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

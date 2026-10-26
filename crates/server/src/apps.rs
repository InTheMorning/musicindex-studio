//! The `/app/v1/` layer of the server (ADR 0058 items 16 to 19). An app
//! token finds its user, and the request goes to the workspace of that
//! user.

use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use tower::ServiceExt;

use crate::accounts;
use crate::limits;
use crate::session::{self, Shared, failure};

/// The 401 response for a request without a usable app token. The message
/// names no token.
pub fn unauthorized() -> Response {
    failure(
        StatusCode::UNAUTHORIZED,
        "app_unauthorized",
        "ADR 0058: this route needs the token of a connected app",
    )
}

/// The hash of the bearer token of a request, when it has one.
fn bearer_hash(headers: &HeaderMap) -> Option<String> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = value.strip_prefix("Bearer ")?.trim();
    if token.is_empty() {
        return None;
    }
    Some(accounts::sha256_hex(token))
}

/// True when the request has a bearer token that is an app token. The
/// `/api/` layer refuses such a request (ADR 0058 item 19).
///
/// # Errors
///
/// Gives a 500 response when the database fails.
pub fn is_app_token(state: &session::ServerState, headers: &HeaderMap) -> Result<bool, Response> {
    let Some(hash) = bearer_hash(headers) else {
        return Ok(false);
    };
    state
        .accounts
        .app_account(&hash)
        .map(|found| found.is_some())
        .map_err(|_| session::internal("the account database failed"))
}

/// The `/app/v1/` layer: find the user of the app token, limit the
/// requests, and send the request to the router of the user. The outer
/// layer has already checked the `Host`.
pub async fn app(State(state): State<Shared>, request: Request) -> Response {
    // An app does not use a session. A request with a cookie and no
    // `Authorization` header has no bearer token, so it gives 401 here.
    let address = limits::client_address(state.client_header.as_deref(), &request);
    let Some(hash) = bearer_hash(request.headers()) else {
        return unauthorized();
    };
    if state.app_limits.unknown_blocked(&address) {
        return limits::too_many_attempts();
    }
    let Ok(found) = state.accounts.app_account(&hash) else {
        return session::internal("the account database failed");
    };
    let Some((account_id, _app_id)) = found else {
        // Only the client address counts. No account is known.
        state.app_limits.record_unknown(&address);
        return unauthorized();
    };
    if !state.app_limits.allow_app(&hash) {
        return failure(
            StatusCode::TOO_MANY_REQUESTS,
            "too_many_attempts",
            "too many requests from this app: try again in a short time",
        );
    }
    let folder = match state.accounts.folder_of(account_id) {
        Ok(Some(folder)) => folder,
        Ok(None) => return unauthorized(),
        Err(_) => return session::internal("the account database failed"),
    };
    let user = match state.users.get(account_id, &folder).await {
        Ok(user) => user,
        Err(error) => return error.into_response(),
    };
    match user.router.oneshot(request).await {
        Ok(response) => response,
        Err(never) => match never {},
    }
}

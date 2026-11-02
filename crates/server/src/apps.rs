//! The `/app/v1/` layer of the server (ADR 0110 items 12, 13, 39 and 40).
//! The public key of the signed request finds the account of the app. A
//! pairing request finds its account from the code. The request then goes to
//! the router of that user, which checks the signature, the time and the
//! replay store.

use axum::extract::{Request, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use studio_api::ClientAddress;
use tower::ServiceExt;

use crate::accounts;
use crate::limits;
use crate::session::{self, Shared, failure};

/// The largest body of a pairing request, as in the workspace (ADR 0110
/// item 14).
const MAX_PAIR_BODY: usize = 64 * 1024;

/// The 401 response for a request with no usable signature or key. The
/// message names no event, key or address.
pub fn signature_invalid() -> Response {
    failure(
        StatusCode::UNAUTHORIZED,
        "app_signature_invalid",
        "ADR 0110 item 13: Studio does not accept this signed request. Sign a new request with the key of the app",
    )
}

/// The 401 response for a pairing code that Studio does not know.
fn code_invalid() -> Response {
    failure(
        StatusCode::UNAUTHORIZED,
        "app_code_invalid",
        "the pairing code is not valid",
    )
}

/// True when the request has the header `Authorization: Nostr`. The
/// `/api/` and `/auth/` layers refuse such a request (ADR 0110 item 16).
pub fn has_app_header(headers: &HeaderMap) -> bool {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("Nostr "))
}

/// The body of the pairing request. Only the code counts here.
#[derive(Deserialize)]
struct PairBody {
    #[serde(default)]
    code: String,
}

/// The `/app/v1/` layer: find the account of the app, limit the requests,
/// and send the request to the router of the user. The outer layer has
/// already checked the `Host`.
pub async fn app(State(state): State<Shared>, request: Request) -> Response {
    let address = limits::client_address(state.client_header.as_deref(), &request);
    if state.app_limits.unknown_blocked(&address) {
        return limits::too_many_attempts();
    }
    let text = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    // ADR 0110 item 37: a token from before gets its own answer.
    if text.as_deref().is_some_and(|t| t.starts_with("Bearer ")) {
        return failure(
            StatusCode::UNAUTHORIZED,
            "app_key_required",
            "ADR 0110 item 37: this route needs a request that the key of a connected app signed. Connect the app again",
        );
    }
    // The signature is examined here once, so that a forged key finds no
    // account and holds no claim. The workspace examines the rest.
    let event = text
        .as_deref()
        .and_then(studio_core::signed_request::parse_header)
        .filter(|event| event.verify().is_ok());
    let Some(event) = event else {
        state.app_limits.record_unknown(&address);
        return signature_invalid();
    };
    let pubkey = event.pubkey.to_hex();
    let is_pair = request.method() == Method::POST && request.uri().path() == "/app/v1/pair";
    let response = if is_pair {
        pair(&state, request, &pubkey, &address).await
    } else {
        known_app(&state, request, &pubkey, &address).await
    };
    if response.status() == StatusCode::UNAUTHORIZED {
        // An unknown key, a signature that is not correct or a wrong code
        // counts for the client address (ADR 0110 item 40).
        state.app_limits.record_unknown(&address);
    }
    response
}

/// A request of a key that an active app has.
async fn known_app(state: &Shared, request: Request, pubkey: &str, address: &str) -> Response {
    let Ok(found) = state.accounts.app_key_account(pubkey) else {
        return session::internal("the account database failed");
    };
    let Some((account_id, _app_id)) = found else {
        return signature_invalid();
    };
    if !state.app_limits.allow_app(pubkey) {
        return too_many_requests();
    }
    forward(state, account_id, request, address).await
}

fn too_many_requests() -> Response {
    failure(
        StatusCode::TOO_MANY_REQUESTS,
        "too_many_attempts",
        "too many requests from this app: try again in a short time",
    )
}

/// The pairing request: the code finds the account. The key is a claim
/// until the workspace answers (ADR 0110 items 4 and 11).
async fn pair(state: &Shared, request: Request, pubkey: &str, address: &str) -> Response {
    let (parts, body) = request.into_parts();
    let Ok(bytes) = axum::body::to_bytes(body, MAX_PAIR_BODY).await else {
        return failure(
            StatusCode::PAYLOAD_TOO_LARGE,
            "body_too_large",
            "ADR 0110 item 14: the body of an app request has 64 KiB at most",
        );
    };
    let hash = serde_json::from_slice::<PairBody>(&bytes)
        .ok()
        .and_then(|body| studio_core::apps::hash_of_code(&body.code));
    let Some(hash) = hash else {
        return code_invalid();
    };
    let Ok(found) = state.accounts.app_code_account(&hash, accounts::now()) else {
        return session::internal("the account database failed");
    };
    let Some((account_id, _app_id)) = found else {
        return code_invalid();
    };
    if !state.app_limits.allow_app(pubkey) {
        return too_many_requests();
    }
    // ADR 0110 item 4: the sign-in key of an account cannot be an app key.
    match state.accounts.account_by_pubkey(pubkey) {
        Ok(None) => {}
        Ok(Some(_)) => {
            return failure(
                StatusCode::BAD_REQUEST,
                "app_key_is_sign_in_key",
                "ADR 0110 item 4: an app needs its own key, not the sign-in key of an account",
            );
        }
        Err(_) => return session::internal("the account database failed"),
    }
    match state.accounts.claim_app_key(pubkey, account_id) {
        Ok(true) => {}
        Ok(false) => {
            return failure(
                StatusCode::CONFLICT,
                "app_key_in_use",
                "a different account uses this key: make a new key and pair again",
            );
        }
        Err(_) => return session::internal("the account database failed"),
    }
    let request = Request::from_parts(parts, axum::body::Body::from(bytes));
    let response = forward(state, account_id, request, address).await;
    if !response.status().is_success() {
        // A failed pairing keeps no claim.
        let _ = state.accounts.release_app_key_claim(pubkey, account_id);
    }
    response
}

/// Send the request to the router of the user.
async fn forward(state: &Shared, account_id: i64, mut request: Request, address: &str) -> Response {
    let folder = match state.accounts.folder_of(account_id) {
        Ok(Some(folder)) => folder,
        Ok(None) => return signature_invalid(),
        Err(_) => return session::internal("the account database failed"),
    };
    let user = match state.users.get(account_id, &folder).await {
        Ok(user) => user,
        Err(error) => return error.into_response(),
    };
    request
        .extensions_mut()
        .insert(ClientAddress(address.to_owned()));
    match user.router.oneshot(request).await {
        Ok(response) => response,
        Err(never) => match never {},
    }
}

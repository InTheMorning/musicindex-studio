//! The routes `/auth/v1/` and the router of the server (ADR 0055 item 20,
//! ADR 0056 items 4, 7 and 8).

use axum::Router;
use axum::extract::{Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::Deserialize;
use serde_json::json;
use studio_api::ui_router;
use studio_core::vault::{MIN_PASSPHRASE_CHARS, passphrase_strength};

use crate::accounts::{self, SignUpError};
use crate::limits;
use crate::session::{
    self, Resolved, Shared, cleared_cookie, csrf_of, failure, internal, read_json, session_cookie,
};

/// The router of the server: the UI, the auth routes and the `/api/`
/// layer, under the outer layer.
pub fn server_router(state: &Shared) -> Router {
    let routes: Router = Router::new()
        .route("/auth/v1/session", get(get_session))
        .route("/auth/v1/signup/email", post(sign_up))
        .route("/auth/v1/signin/email", post(sign_in))
        .route("/auth/v1/signout", post(sign_out))
        .route("/auth/v1/account/delete", post(delete_account))
        // A wildcard route would add a path parameter to the request, and the
        // routes of the user would then read two parameters. A fallback adds none.
        .fallback(session::dispatch)
        .with_state(state.clone());
    ui_router()
        .merge(routes)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            session::outer,
        ))
}

/// Add the cookie header to a response.
fn with_cookie(mut response: Response, cookie: &str) -> Response {
    if let Ok(value) = HeaderValue::from_str(cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
    response
}

/// An email address in lowercase with no space at the start or the end
/// (ADR 0055 item 8). Gives `None` when it is not an address.
fn normalize_email(email: &str) -> Option<String> {
    let email = email.trim().to_lowercase();
    let (name, host) = email.split_once('@')?;
    let valid = !name.is_empty()
        && !host.is_empty()
        && !host.contains('@')
        && email.len() <= 254
        && !email.chars().any(|c| c.is_whitespace() || c.is_control());
    valid.then_some(email)
}

fn sign_in_failed() -> Response {
    failure(
        StatusCode::UNAUTHORIZED,
        "sign_in_failed",
        "the email address or the passphrase is not correct",
    )
}

/// Make a session, and give the response with its cookie.
fn start_session(
    state: &Shared,
    account_id: i64,
    status: StatusCode,
    mut body: serde_json::Value,
) -> Response {
    let Ok(token) = accounts::random_hex(32) else {
        return internal("the random source is not available");
    };
    if state
        .accounts
        .create_session(&accounts::sha256_hex(&token), account_id)
        .is_err()
    {
        return internal("the account database failed");
    }
    body["csrf"] = json!(csrf_of(&token));
    with_cookie(
        (status, axum::Json(body)).into_response(),
        &session_cookie(&token),
    )
}

async fn get_session(State(state): State<Shared>, request: Request) -> Response {
    let resolved = match session::resolve(&state, request.headers()).await {
        Ok(resolved) => resolved,
        Err(response) => return response,
    };
    let signed_out = || {
        axum::Json(json!({
            "mode": "server",
            "signup": state.signup.name(),
            "signed_in": false,
        }))
        .into_response()
    };
    let Resolved::Active(session) = resolved else {
        return signed_out();
    };
    let (Ok(Some(email)), Ok(Some(folder))) = (
        state.accounts.email_of(session.account_id),
        state.accounts.folder_of(session.account_id),
    ) else {
        return signed_out();
    };
    let user = match state.users.get(session.account_id, &folder).await {
        Ok(user) => user,
        Err(error) => return error.into_response(),
    };
    let vault = match user.handle.vault_state().await {
        Ok(vault) => vault,
        Err(error) => return error.into_response(),
    };
    axum::Json(json!({
        "mode": "server",
        "signed_in": true,
        "csrf": csrf_of(&session.token),
        "email": email,
        "vault": vault,
    }))
    .into_response()
}

#[derive(Deserialize)]
struct SignUp {
    email: String,
    passphrase: String,
    #[serde(default)]
    invite: Option<String>,
    #[serde(default)]
    recovery_key: bool,
}

async fn sign_up(State(state): State<Shared>, request: Request) -> Response {
    let body: SignUp = match read_json(request).await {
        Ok(body) => body,
        Err(response) => return response,
    };
    let invite = match state.signup {
        crate::config::Signup::Open => None,
        crate::config::Signup::Invite => {
            match body
                .invite
                .as_deref()
                .map(str::trim)
                .filter(|c| !c.is_empty())
            {
                Some(code) => Some(code.to_owned()),
                None => return invite_required(),
            }
        }
    };
    let Some(email) = normalize_email(&body.email) else {
        return failure(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_email",
            "give an email address",
        );
    };
    // The check comes first, so that a weak passphrase does not use the invite.
    let strength = passphrase_strength(&body.passphrase);
    if !strength.acceptable {
        return if strength.chars < MIN_PASSPHRASE_CHARS {
            failure(
                StatusCode::UNPROCESSABLE_ENTITY,
                "passphrase_too_short",
                "the passphrase is too short",
            )
        } else {
            failure(
                StatusCode::UNPROCESSABLE_ENTITY,
                "passphrase_too_weak",
                "the passphrase is too weak",
            )
        };
    }
    // The Argon2id derivation of the new vault needs a permit. The request
    // waits before it uses the invite (ADR 0055 item 22).
    let _permit = match limits::derivation_permit(&state).await {
        Ok(permit) => permit,
        Err(response) => return response,
    };
    let Ok(folder) = accounts::random_hex(16) else {
        return internal("the random source is not available");
    };
    let account_id = match state.accounts.sign_up(invite.as_deref(), &email, &folder) {
        Ok(id) => id,
        Err(SignUpError::InviteRequired) => return invite_required(),
        Err(SignUpError::EmailInUse) => {
            return failure(
                StatusCode::CONFLICT,
                "email_in_use",
                "another account has this email address",
            );
        }
        Err(SignUpError::Database(_)) => return internal("the account database failed"),
    };
    match make_user(&state, account_id, &folder, &body).await {
        Ok(recovery_key) => start_session(
            &state,
            account_id,
            StatusCode::CREATED,
            json!({ "recovery_key": recovery_key }),
        ),
        Err(response) => {
            // Remove the half-made account, and give the invite back.
            state.users.forget(account_id).await;
            let _ = state.accounts.undo_sign_up(account_id, invite.as_deref());
            let _ = std::fs::remove_dir_all(state.users.folder_path(&folder));
            response
        }
    }
}

fn invite_required() -> Response {
    failure(
        StatusCode::FORBIDDEN,
        "invite_required",
        "sign-up needs an unused invite code",
    )
}

/// Open the new user, give it the library folder and make its vault.
async fn make_user(
    state: &Shared,
    account_id: i64,
    folder: &str,
    body: &SignUp,
) -> Result<Option<String>, Response> {
    let user = state
        .users
        .get(account_id, folder)
        .await
        .map_err(IntoResponse::into_response)?;
    let library = state.users.folder_path(folder).join("library");
    user.handle
        .set_library_root(&library)
        .await
        .map_err(IntoResponse::into_response)?;
    user.handle
        .create_vault(Some(&body.passphrase), body.recovery_key)
        .await
        .map_err(IntoResponse::into_response)
}

#[derive(Deserialize)]
struct SignIn {
    email: String,
    passphrase: String,
}

async fn sign_in(State(state): State<Shared>, request: Request) -> Response {
    let address = limits::client_address(state.client_header.as_deref(), &request);
    let body: SignIn = match read_json(request).await {
        Ok(body) => body,
        Err(response) => return response,
    };
    let email = normalize_email(&body.email);
    // The counter uses the email text, also for an unknown address, so that
    // the answer does not show which addresses have an account.
    let account_key: String = email
        .clone()
        .unwrap_or_else(|| body.email.trim().to_lowercase())
        .chars()
        .take(254)
        .collect();
    if state.attempts.blocked(&account_key, &address) {
        return limits::too_many_attempts();
    }
    let _permit = match limits::derivation_permit(&state).await {
        Ok(permit) => permit,
        Err(response) => return response,
    };
    let failed = || {
        state.attempts.record_failure(&account_key, &address);
        sign_in_failed()
    };
    let found = match email {
        Some(email) => match state.accounts.account_by_email(&email) {
            Ok(found) => found.map(|(id, folder)| (id, folder, email)),
            Err(_) => return internal("the account database failed"),
        },
        None => None,
    };
    let Some((account_id, folder, email)) = found else {
        // Do the same work as for a known address (ADR 0056 item 8).
        let dummy = state.dummy_wrapper.clone();
        let passphrase = body.passphrase;
        let _ = tokio::task::spawn_blocking(move || dummy.open_with_passphrase(&passphrase)).await;
        return failed();
    };
    let user = match state.users.get(account_id, &folder).await {
        Ok(user) => user,
        Err(error) => return error.into_response(),
    };
    if let Err(error) = user.handle.unlock_with_passphrase(&body.passphrase).await {
        let response = error.into_response();
        // A wrong passphrase gives 422, and a missing vault gives 404.
        return match response.status() {
            StatusCode::UNPROCESSABLE_ENTITY | StatusCode::NOT_FOUND => failed(),
            _ => internal("the vault could not be opened"),
        };
    }
    let vault = match user.handle.vault_state().await {
        Ok(vault) => vault,
        Err(error) => return error.into_response(),
    };
    start_session(
        &state,
        account_id,
        StatusCode::OK,
        json!({ "email": email, "vault": vault }),
    )
}

async fn sign_out(State(state): State<Shared>, request: Request) -> Response {
    if let Ok(Resolved::Active(session)) = session::resolve(&state, request.headers()).await {
        let _ = state
            .accounts
            .delete_session(&accounts::sha256_hex(&session.token));
        // The last session of the user ends here, so the vault locks
        // (ADR 0055 item 16).
        session::lock_if_no_session(&state, session.account_id).await;
    }
    with_cookie(
        axum::Json(json!({ "signed_in": false })).into_response(),
        &cleared_cookie(),
    )
}

#[derive(Deserialize)]
struct DeleteAccount {
    confirm: String,
}

/// Delete the account of the session (ADR 0055 item 9). The vault must be
/// open, and the body repeats the email address of the account.
async fn delete_account(State(state): State<Shared>, request: Request) -> Response {
    let session = match session::active_session(&state, request.headers()).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    if let Err(response) = session::check_csrf(&session, request.headers()) {
        return response;
    }
    let body: DeleteAccount = match read_json(request).await {
        Ok(body) => body,
        Err(response) => return response,
    };
    let (Ok(Some(email)), Ok(Some(folder))) = (
        state.accounts.email_of(session.account_id),
        state.accounts.folder_of(session.account_id),
    ) else {
        return failure(
            StatusCode::UNAUTHORIZED,
            "session_required",
            "the account does not exist",
        );
    };
    if body.confirm.trim().to_lowercase() != email {
        return failure(
            StatusCode::UNPROCESSABLE_ENTITY,
            "confirm_mismatch",
            "the confirmation is not the email address of the account",
        );
    }
    let user = match state.users.get(session.account_id, &folder).await {
        Ok(user) => user,
        Err(error) => return error.into_response(),
    };
    match user.handle.vault_state().await {
        Ok(studio_api::VaultState::Open) => {}
        Ok(_) => {
            return failure(StatusCode::LOCKED, "vault_locked", "unlock the vault first");
        }
        Err(error) => return error.into_response(),
    }
    if state.accounts.delete_account(session.account_id).is_err() {
        return internal("the account database failed");
    }
    // The account is gone, so no request can reach the user. A folder that
    // stays on the disk is an operator task and not an error for the user.
    if let Err(error) = user.handle.delete_folder().await {
        eprintln!("studio-server: a user folder was not removed: {error:?}");
    }
    state.users.forget(session.account_id).await;
    with_cookie(StatusCode::NO_CONTENT.into_response(), &cleared_cookie())
}

//! The routes `/auth/v1/` and the router of the server (ADR 0055 item 20,
//! ADR 0056 items 4, 7 and 8).

use axum::Router;
use axum::extract::{Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;
use serde_json::json;
use studio_api::ui_router;
use studio_core::vault::{MIN_PASSPHRASE_CHARS, passphrase_strength};

use zeroize::Zeroizing;

use crate::accounts::{self, PasskeyRow, SignUpError};
use crate::limits;
use crate::session::{
    self, Resolved, Session, Shared, cleared_cookie, csrf_of, failure, internal, read_json,
    session_cookie,
};
use crate::webauthn::{self, Expected, WebAuthnError};

/// The router of the server: the UI, the auth routes and the `/api/`
/// layer, under the outer layer.
pub fn server_router(state: &Shared) -> Router {
    let routes: Router = Router::new()
        .route("/auth/v1/session", get(get_session))
        .route("/auth/v1/signup/email", post(sign_up))
        .route("/auth/v1/signin/email", post(sign_in))
        .route("/auth/v1/strength", post(strength))
        .route("/auth/v1/challenge", post(challenge))
        .route("/auth/v1/nostr", post(nostr_sign_in))
        .route("/auth/v1/recovery", post(recovery_sign_in))
        .route("/auth/v1/passkey/options", post(passkey_sign_in_options))
        .route("/auth/v1/passkey", post(passkey_sign_in))
        .route("/auth/v1/passkeys/options", post(passkey_add_options))
        .route("/auth/v1/passkeys", get(passkey_list).post(passkey_add))
        .route(
            "/auth/v1/passkeys/{credential_id}",
            axum::routing::delete(passkey_remove),
        )
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
    body: serde_json::Value,
) -> Response {
    start_session_with(state, account_id, false, status, body)
}

/// Make a session with or without the limit of a recovery sign-in, and
/// give the response with its cookie.
fn start_session_with(
    state: &Shared,
    account_id: i64,
    restricted: bool,
    status: StatusCode,
    mut body: serde_json::Value,
) -> Response {
    let Ok(token) = accounts::random_hex(32) else {
        return internal("the random source is not available");
    };
    if state
        .accounts
        .create_session(&accounts::sha256_hex(&token), account_id, restricted)
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
    let (Ok(identity), Ok(Some(folder))) = (
        identity_of(&state, session.account_id),
        state.accounts.folder_of(session.account_id),
    ) else {
        return signed_out();
    };
    let Some((email, pubkey)) = identity else {
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
    let Ok(passkeys) = state.accounts.passkeys_of(session.account_id) else {
        return internal("the account database failed");
    };
    axum::Json(json!({
        "mode": "server",
        "signed_in": true,
        "passkeys": passkeys.len(),
        "csrf": csrf_of(&session.token),
        "email": email,
        "pubkey": pubkey,
        "vault": vault,
        "recovery": session.restricted,
    }))
    .into_response()
}

/// The email address and the Nostr public key of an account.
type Identity = (Option<String>, Option<String>);

/// The email address and the Nostr public key of an account. One of them
/// is `None`. The outer `None` means that the account does not exist.
fn identity_of(state: &Shared, account_id: i64) -> Result<Option<Identity>, String> {
    let email = state.accounts.email_of(account_id)?;
    let pubkey = state.accounts.pubkey_of(account_id)?;
    Ok((email.is_some() || pubkey.is_some()).then_some((email, pubkey)))
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
    if let Err(response) =
        session::check_restricted(&session, request.method(), request.uri().path())
    {
        return response;
    }
    let body: DeleteAccount = match read_json(request).await {
        Ok(body) => body,
        Err(response) => return response,
    };
    let (Ok(Some((email, pubkey))), Ok(Some(folder))) = (
        identity_of(&state, session.account_id),
        state.accounts.folder_of(session.account_id),
    ) else {
        return failure(
            StatusCode::UNAUTHORIZED,
            "session_required",
            "the account does not exist",
        );
    };
    // An email account confirms with its address. A Nostr account confirms
    // with its `npub` (task 034d).
    let confirmed = match (email, pubkey) {
        (Some(email), _) => body.confirm.trim().to_lowercase() == email,
        (None, Some(pubkey)) => {
            use nostr::prelude::FromBech32;
            nostr::prelude::PublicKey::from_bech32(body.confirm.trim())
                .is_ok_and(|key| key.to_hex() == pubkey)
        }
        (None, None) => false,
    };
    if !confirmed {
        return failure(
            StatusCode::UNPROCESSABLE_ENTITY,
            "confirm_mismatch",
            "the confirmation is not the email address or the public key of the account",
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

#[derive(Deserialize)]
struct Strength {
    passphrase: String,
}

/// The strength of a passphrase for the sign-up page, before the user has a
/// session (ADR 0018). The result is the same as the result of the vault
/// route of the desktop. The route does no Argon2id work. It refuses a
/// client address that has too many failed sign-ins, and it counts no failure.
async fn strength(State(state): State<Shared>, request: Request) -> Response {
    let address = limits::client_address(state.client_header.as_deref(), &request);
    if state.attempts.blocked("", &address) {
        return limits::too_many_attempts();
    }
    let body: Strength = match read_json(request).await {
        Ok(body) => body,
        Err(response) => return response,
    };
    axum::Json(passphrase_strength(&body.passphrase)).into_response()
}

/// Give a challenge for a sign-in event (ADR 0056 item 5).
async fn challenge(State(state): State<Shared>) -> Response {
    match state.accounts.create_challenge() {
        Ok(Some((challenge, expires_at))) => axum::Json(json!({
            "challenge": challenge,
            "expires_at": expires_at,
        }))
        .into_response(),
        Ok(None) => limits::too_many_attempts(),
        Err(_) => internal("the account database failed"),
    }
}

/// The kind of a sign-in event (NIP-98).
const KIND_HTTP_AUTH: u16 = 27_235;
/// A sign-in event can differ this many seconds from the server time.
const EVENT_WINDOW: u64 = 60;

#[derive(Deserialize)]
struct NostrSignIn {
    event: serde_json::Value,
    #[serde(default)]
    invite: Option<String>,
    #[serde(default)]
    recovery_key: bool,
}

fn event_invalid() -> Response {
    failure(
        StatusCode::UNAUTHORIZED,
        "event_invalid",
        "the sign-in event is not correct",
    )
}

fn challenge_invalid() -> Response {
    failure(
        StatusCode::UNAUTHORIZED,
        "challenge_invalid",
        "the challenge is missing, used or expired",
    )
}

/// The first value of the tag `name`.
fn tag_value<'a>(event: &'a nostr::prelude::Event, name: &str) -> Option<&'a str> {
    event.tags.iter().find_map(|tag| match tag.as_slice() {
        [first, value, ..] if first == name => Some(value.as_str()),
        _ => None,
    })
}

/// Check the kind, the ID, the signature, the tags and the time of a
/// sign-in event (ADR 0056 item 6). Gives the challenge of the event.
fn check_event<'a>(state: &Shared, event: &'a nostr::prelude::Event) -> Option<&'a str> {
    let url = format!("{}/auth/v1/nostr", state.origin.text);
    let now = accounts::now().unsigned_abs();
    let near = event.created_at.as_secs().abs_diff(now) <= EVENT_WINDOW;
    let correct = event.kind.as_u16() == KIND_HTTP_AUTH
        && near
        && tag_value(event, "u") == Some(url.as_str())
        && tag_value(event, "method") == Some("POST")
        && event.verify().is_ok();
    if correct {
        tag_value(event, "challenge")
    } else {
        None
    }
}

/// Sign up or sign in with a NIP-07 signer (ADR 0056 items 5, 6, 9 and 11).
async fn nostr_sign_in(State(state): State<Shared>, request: Request) -> Response {
    let address = limits::client_address(state.client_header.as_deref(), &request);
    let body: NostrSignIn = match read_json(request).await {
        Ok(body) => body,
        Err(response) => return response,
    };
    let event = serde_json::from_value::<nostr::prelude::Event>(body.event).ok();
    // The counter uses the public key only when the signature is correct.
    // Thus a person cannot block the account of a different key. An event
    // with no correct signature counts for the address only.
    let account_key = event
        .as_ref()
        .filter(|event| event.verify().is_ok())
        .map_or_else(
            || format!("nostr-unverified:{address}"),
            |event| event.pubkey.to_hex(),
        );
    if state.attempts.blocked(&account_key, &address) {
        return limits::too_many_attempts();
    }
    let failed = |response: Response| {
        state.attempts.record_failure(&account_key, &address);
        response
    };
    let Some(event) = event else {
        return failed(event_invalid());
    };
    // The server removes the challenge after the checks, so that a wrong
    // event does not use a challenge.
    let Some(challenge) = check_event(&state, &event) else {
        return failed(event_invalid());
    };
    match state.accounts.take_challenge(challenge) {
        Ok(true) => {}
        Ok(false) => return failed(challenge_invalid()),
        Err(_) => return internal("the account database failed"),
    }
    let pubkey = event.pubkey.to_hex();
    let Ok(known) = state.accounts.account_by_pubkey(&pubkey) else {
        return internal("the account database failed");
    };
    if let Some((account_id, folder)) = known {
        return nostr_known(&state, account_id, &folder).await;
    }
    nostr_new(&state, &pubkey, body.invite.as_deref(), body.recovery_key).await
}

/// A sign-in of a known key. The vault stays closed when its file exists.
/// An account with no vault file gets a new vault with no wrapper.
async fn nostr_known(state: &Shared, account_id: i64, folder: &str) -> Response {
    let user = match state.users.get(account_id, folder).await {
        Ok(user) => user,
        Err(error) => return error.into_response(),
    };
    let current = match user.handle.vault_state().await {
        Ok(current) => current,
        Err(error) => return error.into_response(),
    };
    if current == studio_api::VaultState::Absent
        && let Err(error) = user.handle.create_vault(None, false).await
        && error.into_response().status() != StatusCode::CONFLICT
    {
        return internal("the vault could not be made");
    }
    let vault = match user.handle.vault_state().await {
        Ok(vault) => vault,
        Err(error) => return error.into_response(),
    };
    start_session(
        state,
        account_id,
        StatusCode::OK,
        json!({ "new_account": false, "recovery_key": null, "vault": vault }),
    )
}

/// The sign-up of an unknown key. The rules of the email sign-up apply.
async fn nostr_new(
    state: &Shared,
    pubkey: &str,
    invite: Option<&str>,
    recovery_key: bool,
) -> Response {
    let invite = match state.signup {
        crate::config::Signup::Open => None,
        crate::config::Signup::Invite => match invite.map(str::trim).filter(|c| !c.is_empty()) {
            Some(code) => Some(code.to_owned()),
            None => return invite_required(),
        },
    };
    // The recovery key uses an Argon2id derivation, and it needs a permit.
    let _permit = if recovery_key {
        match limits::derivation_permit(state).await {
            Ok(permit) => Some(permit),
            Err(response) => return response,
        }
    } else {
        None
    };
    let Ok(folder) = accounts::random_hex(16) else {
        return internal("the random source is not available");
    };
    let account_id = match state
        .accounts
        .sign_up_nostr(invite.as_deref(), pubkey, &folder)
    {
        Ok(id) => id,
        Err(SignUpError::InviteRequired) => return invite_required(),
        Err(SignUpError::EmailInUse) => {
            return failure(
                StatusCode::CONFLICT,
                "pubkey_in_use",
                "another account has this public key",
            );
        }
        Err(SignUpError::Database(_)) => return internal("the account database failed"),
    };
    let made = async {
        let user = state
            .users
            .get(account_id, &folder)
            .await
            .map_err(IntoResponse::into_response)?;
        user.handle
            .set_library_root(&state.users.folder_path(&folder).join("library"))
            .await
            .map_err(IntoResponse::into_response)?;
        user.handle
            .create_vault(None, recovery_key)
            .await
            .map_err(IntoResponse::into_response)
    }
    .await;
    match made {
        Ok(recovery) => start_session(
            state,
            account_id,
            StatusCode::CREATED,
            json!({ "new_account": true, "recovery_key": recovery, "vault": "open" }),
        ),
        Err(response) => {
            state.users.forget(account_id).await;
            let _ = state.accounts.undo_sign_up(account_id, invite.as_deref());
            let _ = std::fs::remove_dir_all(state.users.folder_path(&folder));
            response
        }
    }
}

#[derive(Deserialize)]
struct RecoverySignIn {
    account: String,
    recovery_key: String,
}

/// The account of a recovery sign-in: an email address or an `npub`.
enum RecoveryAccount {
    Email(String),
    Pubkey(String),
}

impl RecoveryAccount {
    /// Read the account text. Gives `None` for other text.
    fn parse(text: &str) -> Option<Self> {
        use nostr::prelude::FromBech32;
        let text = text.trim();
        if text.to_ascii_lowercase().starts_with("npub1") {
            return nostr::prelude::PublicKey::from_bech32(text)
                .ok()
                .map(|key| Self::Pubkey(key.to_hex()));
        }
        normalize_email(text).map(Self::Email)
    }

    /// The key of the failure counter.
    fn counter_key(&self) -> &str {
        match self {
            Self::Email(value) | Self::Pubkey(value) => value,
        }
    }
}

/// Sign in with the recovery key (ADR 0056 items 3 and 8). The session has
/// a limit until the user adds a new sign-in method.
async fn recovery_sign_in(State(state): State<Shared>, request: Request) -> Response {
    let address = limits::client_address(state.client_header.as_deref(), &request);
    let body: RecoverySignIn = match read_json(request).await {
        Ok(body) => body,
        Err(response) => return response,
    };
    let account = RecoveryAccount::parse(&body.account);
    // Other text counts for the client address only.
    let account_key = account.as_ref().map_or_else(
        || format!("recovery-unknown:{address}"),
        |account| account.counter_key().to_owned(),
    );
    if state.attempts.blocked(&account_key, &address) {
        return limits::too_many_attempts();
    }
    // A recovery wrapper of the earlier type uses Argon2id.
    let _permit = match limits::derivation_permit(&state).await {
        Ok(permit) => permit,
        Err(response) => return response,
    };
    let failed = || {
        state.attempts.record_failure(&account_key, &address);
        sign_in_failed()
    };
    let found = match &account {
        Some(RecoveryAccount::Email(email)) => state.accounts.account_by_email(email),
        Some(RecoveryAccount::Pubkey(pubkey)) => state.accounts.account_by_pubkey(pubkey),
        None => Ok(None),
    };
    let Ok(found) = found else {
        return internal("the account database failed");
    };
    // Do the same work for an unknown account and for an account with no
    // vault file (ADR 0056 item 8).
    let dummy_work = || async {
        let dummy = state.dummy_wrapper.clone();
        let key = body.recovery_key.clone();
        let _ = tokio::task::spawn_blocking(move || dummy.open_with_recovery_key(&key)).await;
    };
    let Some((account_id, folder)) = found else {
        dummy_work().await;
        return failed();
    };
    let user = match state.users.get(account_id, &folder).await {
        Ok(user) => user,
        Err(error) => return error.into_response(),
    };
    match user.handle.vault_state().await {
        Ok(studio_api::VaultState::Absent) => {
            dummy_work().await;
            return failed();
        }
        Ok(_) => {}
        Err(error) => return error.into_response(),
    }
    if let Err(error) = user
        .handle
        .unlock_with_recovery_key(&body.recovery_key)
        .await
    {
        let response = error.into_response();
        // A wrong key gives 422, and a missing vault gives 404.
        return match response.status() {
            StatusCode::UNPROCESSABLE_ENTITY | StatusCode::NOT_FOUND => failed(),
            _ => internal("the vault could not be opened"),
        };
    }
    let vault = match user.handle.vault_state().await {
        Ok(vault) => vault,
        Err(error) => return error.into_response(),
    };
    start_session_with(
        &state,
        account_id,
        true,
        StatusCode::OK,
        json!({ "recovery": true, "vault": vault }),
    )
}

// ---------------------------------------------------------------------------
// Passkeys with PRF (ADR 0056 items 20 to 27)
// ---------------------------------------------------------------------------

/// The text that the PRF salt comes from. See [`prf_salt`].
const PRF_SALT_TEXT: &str = "musicindex-studio passkey prf salt v1";
/// The longest name of a passkey, in characters.
const MAX_PASSKEY_NAME: usize = 64;

/// The PRF salt of each passkey of this server. A sign-in with no session
/// does not know the credential before the assertion, so the browser needs
/// one salt for every credential. The PRF output still differs for each
/// credential, because the secret of the authenticator differs.
fn prf_salt() -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::digest(PRF_SALT_TEXT.as_bytes()).into()
}

/// The relying party ID: the host of the public origin.
fn rp_id(state: &Shared) -> String {
    let authority = state.origin.authority.as_str();
    match authority.strip_prefix('[') {
        Some(rest) => rest.split_once(']').map_or("", |(host, _)| host),
        None => authority.split(':').next().unwrap_or(""),
    }
    .to_owned()
}

fn decode(text: &str) -> Option<Vec<u8>> {
    URL_SAFE_NO_PAD.decode(text.as_bytes()).ok()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) || !text.is_ascii() {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

/// The PRF output of a request, as 32 bytes in a buffer that clears itself.
/// Gives `None` for a missing output or an output of another length.
fn decode_prf(text: Option<&Zeroizing<String>>) -> Option<Zeroizing<[u8; 32]>> {
    let bytes = Zeroizing::new(URL_SAFE_NO_PAD.decode(text?.as_bytes()).ok()?);
    let mut output = Zeroizing::new([0_u8; 32]);
    if bytes.len() != output.len() {
        return None;
    }
    output.copy_from_slice(&bytes);
    Some(output)
}

fn passkey_no_prf() -> Response {
    failure(
        StatusCode::UNPROCESSABLE_ENTITY,
        "passkey_no_prf",
        "this passkey gave no PRF output",
    )
}

fn passkey_invalid(status: StatusCode) -> Response {
    failure(
        status,
        "passkey_invalid",
        "the passkey response is not correct",
    )
}

/// The session of a request. The request needs the CSRF header, and a
/// restricted session reaches only the routes of its list.
async fn passkey_session(
    state: &Shared,
    headers: &axum::http::HeaderMap,
    method: &axum::http::Method,
    path: &str,
) -> Result<Session, Response> {
    let session = session::active_session(state, headers).await?;
    session::check_csrf(&session, headers)?;
    session::check_restricted(&session, method, path)?;
    Ok(session)
}

/// The open user of a session, when the vault is open.
async fn open_vault_user(
    state: &Shared,
    session: &Session,
) -> Result<crate::users::OpenUser, Response> {
    let folder = match state.accounts.folder_of(session.account_id) {
        Ok(Some(folder)) => folder,
        Ok(None) => {
            return Err(failure(
                StatusCode::UNAUTHORIZED,
                "session_required",
                "the account does not exist",
            ));
        }
        Err(_) => return Err(internal("the account database failed")),
    };
    let user = state
        .users
        .get(session.account_id, &folder)
        .await
        .map_err(IntoResponse::into_response)?;
    match user.handle.vault_state().await {
        Ok(studio_api::VaultState::Open) => Ok(user),
        Ok(_) => Err(failure(
            StatusCode::LOCKED,
            "vault_locked",
            "unlock the vault first",
        )),
        Err(error) => Err(error.into_response()),
    }
}

/// The options for a new passkey (ADR 0056 items 20 and 21).
async fn passkey_add_options(State(state): State<Shared>, request: Request) -> Response {
    let session = match passkey_session(
        &state,
        request.headers(),
        request.method(),
        request.uri().path(),
    )
    .await
    {
        Ok(session) => session,
        Err(response) => return response,
    };
    if let Err(response) = open_vault_user(&state, &session).await {
        return response;
    }
    let db_failed = |_: String| internal("the account database failed");
    let (email, pubkey) = match identity_of(&state, session.account_id) {
        Ok(Some(identity)) => identity,
        Ok(None) => return internal("the account does not exist"),
        Err(_) => return internal("the account database failed"),
    };
    let user_name = email.unwrap_or_else(|| {
        format!(
            "nostr:{}",
            pubkey
                .as_deref()
                .unwrap_or_default()
                .chars()
                .take(8)
                .collect::<String>()
        )
    });
    let handle = match state.accounts.user_handle(session.account_id) {
        Ok(handle) => handle,
        Err(error) => return db_failed(error),
    };
    let existing = match state.accounts.passkeys_of(session.account_id) {
        Ok(rows) => rows,
        Err(error) => return db_failed(error),
    };
    let challenges = match state.accounts.create_passkey_challenges(session.account_id) {
        Ok(Some(challenges)) => challenges,
        Ok(None) => return limits::too_many_attempts(),
        Err(error) => return db_failed(error),
    };
    let encode = |hex: &str| unhex(hex).map(|bytes| URL_SAFE_NO_PAD.encode(bytes));
    let (Some(challenge), Some(assertion_challenge)) = (
        encode(&challenges.challenge),
        encode(&challenges.assertion_challenge),
    ) else {
        return internal("the challenge is not correct");
    };
    axum::Json(json!({
        "challenge": challenge,
        "assertion_challenge": assertion_challenge,
        "rp_id": rp_id(&state),
        "user_handle": URL_SAFE_NO_PAD.encode(handle),
        "user_name": user_name,
        "prf_salt": URL_SAFE_NO_PAD.encode(prf_salt()),
        "exclude": existing.iter().map(|row| row.credential_id.as_str()).collect::<Vec<_>>(),
        "expires_at": challenges.expires_at,
    }))
    .into_response()
}

#[derive(Deserialize)]
struct RegistrationBody {
    client_data: String,
    authenticator_data: String,
    public_key: String,
    algorithm: i64,
}

#[derive(Deserialize)]
struct AssertionBody {
    credential_id: String,
    client_data: String,
    authenticator_data: String,
    signature: String,
}

#[derive(Deserialize)]
struct AddPasskey {
    name: String,
    registration: RegistrationBody,
    assertion: AssertionBody,
    #[serde(default)]
    prf_output: Option<Zeroizing<String>>,
}

/// Check the registration and then the assertion of a new passkey. The
/// function removes the stored challenges first, so that they work one time
/// only. Gives the new credential and the counter of its assertion.
fn check_new_passkey(
    state: &Shared,
    account_id: i64,
    body: &AddPasskey,
) -> Result<(webauthn::Registered, u32), Response> {
    let bad = || passkey_invalid(StatusCode::UNPROCESSABLE_ENTITY);
    let (Some(reg_client), Some(reg_auth), Some(spki), Some(as_client), Some(as_auth), Some(sig)) = (
        decode(&body.registration.client_data),
        decode(&body.registration.authenticator_data),
        decode(&body.registration.public_key),
        decode(&body.assertion.client_data),
        decode(&body.assertion.authenticator_data),
        decode(&body.assertion.signature),
    ) else {
        return Err(bad());
    };
    let Some(assertion_id) = decode(&body.assertion.credential_id) else {
        return Err(bad());
    };
    // The challenge text of the response only finds the stored row. The
    // checks below compare the stored bytes.
    let Some(lookup) = webauthn::challenge_of(&reg_client) else {
        return Err(bad());
    };
    let pending = match state
        .accounts
        .take_passkey_challenges(account_id, &accounts::hex(&lookup))
    {
        Ok(Some(pending)) => pending,
        Ok(None) => return Err(challenge_invalid()),
        Err(_) => return Err(internal("the account database failed")),
    };
    let (Some(registration_challenge), Some(assertion_challenge)) = (
        unhex(&pending.challenge),
        unhex(&pending.assertion_challenge),
    ) else {
        return Err(internal("the challenge is not correct"));
    };
    let rp = rp_id(state);
    let origin = state.origin.text.as_str();
    let expected = |challenge| Expected {
        origin,
        rp_id: &rp,
        challenge,
    };
    let Ok(registered) = webauthn::verify_registration(
        &expected(&registration_challenge),
        &reg_client,
        &reg_auth,
        &spki,
        body.registration.algorithm,
    ) else {
        return Err(bad());
    };
    // The assertion must come from the credential that the registration made.
    if assertion_id != registered.credential_id {
        return Err(bad());
    }
    let Ok(counter) = webauthn::verify_assertion(
        &expected(&assertion_challenge),
        &as_client,
        &as_auth,
        &sig,
        &registered.public_key,
        registered.counter,
    ) else {
        return Err(bad());
    };
    Ok((registered, counter))
}

/// Add a passkey (ADR 0056 items 21 to 25). The server checks the
/// registration and then the assertion of the same credential. It trusts
/// the PRF output only after the assertion is correct.
async fn passkey_add(State(state): State<Shared>, request: Request) -> Response {
    let session = match passkey_session(
        &state,
        request.headers(),
        request.method(),
        request.uri().path(),
    )
    .await
    {
        Ok(session) => session,
        Err(response) => return response,
    };
    let body: AddPasskey = match read_json(request).await {
        Ok(body) => body,
        Err(response) => return response,
    };
    let user = match open_vault_user(&state, &session).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let Some(prf_output) = decode_prf(body.prf_output.as_ref()) else {
        return passkey_no_prf();
    };
    let name = body.name.trim();
    if name.is_empty()
        || name.chars().count() > MAX_PASSKEY_NAME
        || name.chars().any(char::is_control)
    {
        return failure(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_name",
            "give a name of 1 to 64 characters",
        );
    }
    let (registered, counter) = match check_new_passkey(&state, session.account_id, &body) {
        Ok(checked) => checked,
        Err(response) => return response,
    };
    let credential_id = URL_SAFE_NO_PAD.encode(&registered.credential_id);
    let row = PasskeyRow {
        credential_id: credential_id.clone(),
        account_id: session.account_id,
        public_key: registered.public_key,
        counter,
        prf_salt: URL_SAFE_NO_PAD.encode(prf_salt()),
        name: name.to_owned(),
        created_at: accounts::now(),
    };
    match state.accounts.add_passkey(&row) {
        Ok(true) => {}
        Ok(false) => {
            return failure(
                StatusCode::CONFLICT,
                "passkey_exists",
                "this passkey is already in use",
            );
        }
        Err(_) => return internal("the account database failed"),
    }
    if let Err(error) = user
        .handle
        .add_passkey_wrapper(&credential_id, prf_salt(), prf_output)
        .await
    {
        let _ = state
            .accounts
            .delete_passkey(session.account_id, &credential_id);
        return error.into_response();
    }
    // A passkey is a method that signs in to the account, so it removes the
    // limit of a recovery session.
    if session.restricted
        && state
            .accounts
            .lift_restriction(&accounts::sha256_hex(&session.token))
            .is_err()
    {
        return internal("the account database failed");
    }
    (
        StatusCode::CREATED,
        axum::Json(json!({
            "credential_id": credential_id,
            "name": row.name,
            "created_at": row.created_at,
        })),
    )
        .into_response()
}

/// The passkeys of the account: name, creation time and credential ID.
async fn passkey_list(State(state): State<Shared>, request: Request) -> Response {
    let session = match passkey_session(
        &state,
        request.headers(),
        request.method(),
        request.uri().path(),
    )
    .await
    {
        Ok(session) => session,
        Err(response) => return response,
    };
    match state.accounts.passkeys_of(session.account_id) {
        Ok(rows) => axum::Json(
            rows.iter()
                .map(|row| {
                    json!({
                        "credential_id": row.credential_id,
                        "name": row.name,
                        "created_at": row.created_at,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .into_response(),
        Err(_) => internal("the account database failed"),
    }
}

/// Remove a passkey and its wrapper. The server refuses the last method of
/// the account (ADR 0056 item 3).
async fn passkey_remove(
    State(state): State<Shared>,
    axum::extract::Path(credential_id): axum::extract::Path<String>,
    request: Request,
) -> Response {
    let session = match passkey_session(
        &state,
        request.headers(),
        request.method(),
        request.uri().path(),
    )
    .await
    {
        Ok(session) => session,
        Err(response) => return response,
    };
    let user = match open_vault_user(&state, &session).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    match state.accounts.passkey(&credential_id) {
        Ok(Some(row)) if row.account_id == session.account_id => {}
        Ok(_) => {
            return failure(StatusCode::NOT_FOUND, "not_found", "no such passkey");
        }
        Err(_) => return internal("the account database failed"),
    }
    match state.accounts.method_count(session.account_id) {
        Ok(count) if count > 1 => {}
        Ok(_) => {
            return failure(
                StatusCode::CONFLICT,
                "last_method",
                "this is the last sign-in method of the account",
            );
        }
        Err(_) => return internal("the account database failed"),
    }
    if let Err(error) = user.handle.remove_passkey_wrapper(&credential_id).await {
        return error.into_response();
    }
    match state
        .accounts
        .delete_passkey(session.account_id, &credential_id)
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => internal("the account database failed"),
    }
}

/// A challenge for a passkey sign-in, or for an unlock with a session
/// (ADR 0056 items 5 and 24). With a session, the options name the
/// passkeys of the account.
async fn passkey_sign_in_options(State(state): State<Shared>, request: Request) -> Response {
    let allow = match session::resolve(&state, request.headers()).await {
        Ok(Resolved::Active(session)) => match state.accounts.passkeys_of(session.account_id) {
            Ok(rows) => rows.into_iter().map(|row| row.credential_id).collect(),
            Err(_) => return internal("the account database failed"),
        },
        Ok(_) => Vec::new(),
        Err(response) => return response,
    };
    let (challenge, expires_at) = match state.accounts.create_challenge() {
        Ok(Some(made)) => made,
        Ok(None) => return limits::too_many_attempts(),
        Err(_) => return internal("the account database failed"),
    };
    let Some(bytes) = unhex(&challenge) else {
        return internal("the challenge is not correct");
    };
    axum::Json(json!({
        "challenge": URL_SAFE_NO_PAD.encode(bytes),
        "rp_id": rp_id(&state),
        "prf_salt": URL_SAFE_NO_PAD.encode(prf_salt()),
        "allow": allow,
        "expires_at": expires_at,
    }))
    .into_response()
}

#[derive(Deserialize)]
struct PasskeySignIn {
    assertion: AssertionBody,
    #[serde(default)]
    prf_output: Option<Zeroizing<String>>,
}

/// Check the assertion of a passkey sign-in. The function removes the
/// challenge first, so that it works one time only. A failure before the
/// signature is correct counts for the client address only. Gives the row,
/// the new counter and the credential ID.
fn check_assertion(
    state: &Shared,
    address: &str,
    unverified: &str,
    body: &PasskeySignIn,
) -> Result<(PasskeyRow, u32, String), Response> {
    let failed_unverified = || {
        state.attempts.record_failure(unverified, address);
        passkey_invalid(StatusCode::UNAUTHORIZED)
    };
    let (Some(client), Some(auth), Some(sig), Some(id)) = (
        decode(&body.assertion.client_data),
        decode(&body.assertion.authenticator_data),
        decode(&body.assertion.signature),
        decode(&body.assertion.credential_id),
    ) else {
        return Err(failed_unverified());
    };
    // The challenge text of the response only finds the stored challenge.
    // The server removes it before the other checks, so it works one time.
    let Some(lookup) = webauthn::challenge_of(&client) else {
        return Err(failed_unverified());
    };
    let stored = match state.accounts.take_challenge(&accounts::hex(&lookup)) {
        Ok(true) => lookup,
        Ok(false) => {
            state.attempts.record_failure(unverified, address);
            return Err(challenge_invalid());
        }
        Err(_) => return Err(internal("the account database failed")),
    };
    let credential_id = URL_SAFE_NO_PAD.encode(&id);
    let row = match state.accounts.passkey(&credential_id) {
        Ok(Some(row)) => row,
        Ok(None) => return Err(failed_unverified()),
        Err(_) => return Err(internal("the account database failed")),
    };
    let rp = rp_id(state);
    let expected = Expected {
        origin: &state.origin.text,
        rp_id: &rp,
        challenge: &stored,
    };
    match webauthn::verify_assertion(
        &expected,
        &client,
        &auth,
        &sig,
        &row.public_key,
        row.counter,
    ) {
        Ok(counter) => Ok((row, counter, credential_id)),
        // The signature was correct, so the credential is the holder.
        Err(WebAuthnError::Counter) => {
            state
                .attempts
                .record_failure(&format!("passkey:{}", row.account_id), address);
            Err(passkey_invalid(StatusCode::UNAUTHORIZED))
        }
        Err(_) => Err(failed_unverified()),
    }
}

/// Sign in with a passkey, or unlock the vault with it (ADR 0056 items 22
/// and 24). The assertion proves the sign-in. The PRF output opens the vault.
async fn passkey_sign_in(State(state): State<Shared>, request: Request) -> Response {
    let address = limits::client_address(state.client_header.as_deref(), &request);
    let current = match session::resolve(&state, request.headers()).await {
        Ok(Resolved::Active(session)) => Some(session),
        Ok(_) => None,
        Err(response) => return response,
    };
    let body: PasskeySignIn = match read_json(request).await {
        Ok(body) => body,
        Err(response) => return response,
    };
    // A person can fail before the server knows the account. These failures
    // count for the client address only, so that a false assertion cannot
    // block a different account.
    let unverified = format!("passkey-unverified:{address}");
    if state.attempts.blocked(&unverified, &address) {
        return limits::too_many_attempts();
    }
    let (row, counter, credential_id) = match check_assertion(&state, &address, &unverified, &body)
    {
        Ok(checked) => checked,
        Err(response) => return response,
    };
    // From here the signature is correct, so the failures count for the account.
    let account_key = format!("passkey:{}", row.account_id);
    if state.attempts.blocked(&account_key, &address) {
        return limits::too_many_attempts();
    }
    let failed = || {
        state.attempts.record_failure(&account_key, &address);
        passkey_invalid(StatusCode::UNAUTHORIZED)
    };
    let Some(prf_output) = decode_prf(body.prf_output.as_ref()) else {
        return passkey_no_prf();
    };
    match state
        .accounts
        .update_passkey_counter(&credential_id, row.counter, counter)
    {
        Ok(true) => {}
        Ok(false) => return failed(),
        Err(_) => return internal("the account database failed"),
    }
    let folder = match state.accounts.folder_of(row.account_id) {
        Ok(Some(folder)) => folder,
        Ok(None) => return failed(),
        Err(_) => return internal("the account database failed"),
    };
    let user = match state.users.get(row.account_id, &folder).await {
        Ok(user) => user,
        Err(error) => return error.into_response(),
    };
    if let Err(error) = user
        .handle
        .unlock_with_passkey(&credential_id, prf_output)
        .await
    {
        let response = error.into_response();
        // A wrong PRF output gives 422, and a missing vault gives 404.
        return match response.status() {
            StatusCode::UNPROCESSABLE_ENTITY | StatusCode::NOT_FOUND => failed(),
            _ => internal("the vault could not be opened"),
        };
    }
    let vault = match user.handle.vault_state().await {
        Ok(vault) => vault,
        Err(error) => return error.into_response(),
    };
    // An unlock keeps the session. A session of another account, or a session
    // with a limit, does not count: the passkey makes a new session.
    if current
        .as_ref()
        .is_some_and(|s| s.account_id == row.account_id && !s.restricted)
    {
        return axum::Json(json!({ "vault": vault })).into_response();
    }
    start_session(
        &state,
        row.account_id,
        StatusCode::OK,
        json!({ "vault": vault }),
    )
}

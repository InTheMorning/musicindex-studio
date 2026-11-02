//! The import of an account at sign-up (ADR 0107 items 9 to 14).
//!
//! The routes `/auth/v1/import` need no session. The browser starts an
//! import, sends the archive in parts (ADR 0059), asks for a check, and
//! opens the vault of the archive. The server makes the account when the
//! vault opens, moves the unpacked files to the new user folder, and starts
//! a session. A failure after the account exists removes the account and its
//! folder.

use std::path::{Path as FilePath, PathBuf};
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;
use studio_api::{ApiError, Credential, ImportOpen, Staged, verify_credential};
use studio_core::account_archive::LibraryPolicy;
use studio_core::vault::{VaultFile, VaultMethod};
use zeroize::Zeroizing;

use crate::accounts::{self, SignUpError};
use crate::auth::{
    SignerLink, blocking_signer, connection_unknown, invite_required, normalize_email,
    start_session_full,
};
use crate::limits;
use crate::nip46_sessions::Checkout;
use crate::session::{self, Shared, failure, internal, read_json};

fn from_api(error: ApiError) -> Response {
    error.into_response()
}

/// Run store work on a blocking thread.
async fn store<T: Send + 'static>(
    state: &Shared,
    work: impl FnOnce(&studio_api::ImportStore) -> Result<T, ApiError> + Send + 'static,
) -> Result<T, Response> {
    let state = Arc::clone(state);
    tokio::task::spawn_blocking(move || work(&state.imports))
        .await
        .map_err(|_| internal("the import task failed"))?
        .map_err(from_api)
}

#[derive(Deserialize)]
struct StartBody {
    size: u64,
    #[serde(default)]
    invite: Option<String>,
}

/// `POST /auth/v1/import`: start an import. A server with invite codes needs
/// an unused code, as a sign-up does (ADR 0107 item 9). The size of the file
/// must fit the storage limit of a new account (ADR 0107 item 15).
pub async fn start(State(state): State<Shared>, request: Request) -> Response {
    let address = limits::client_address(state.client_header.as_deref(), &request);
    if !state.import_starts.allow(&address) {
        return limits::too_many_attempts();
    }
    let body: StartBody = match read_json(request).await {
        Ok(body) => body,
        Err(response) => return response,
    };
    let invite = match state.signup {
        crate::config::Signup::Open => None,
        crate::config::Signup::Invite => {
            let Some(code) = body
                .invite
                .as_deref()
                .map(str::trim)
                .filter(|c| !c.is_empty())
            else {
                return invite_required();
            };
            match state.accounts.invite_unused(code) {
                Ok(true) => Some(code.to_owned()),
                Ok(false) => return invite_required(),
                Err(_) => return internal("the account database failed"),
            }
        }
    };
    let limit = state.users.storage_limit();
    match store(&state, move |store| store.start(body.size, limit, invite)).await {
        Ok(started) => (StatusCode::CREATED, axum::Json(started)).into_response(),
        Err(response) => response,
    }
}

/// `PUT /auth/v1/import/{upload}/parts/{n}`: send part `n`.
pub async fn put_part(
    State(state): State<Shared>,
    Path((upload, n)): Path<(String, u32)>,
    body: Bytes,
) -> Response {
    match store(&state, move |store| store.put_part(&upload, n, &body)).await {
        Ok(status) => axum::Json(status).into_response(),
        Err(response) => response,
    }
}

/// `GET /auth/v1/import/{upload}`: the size and the parts that the server has.
pub async fn status(State(state): State<Shared>, Path(upload): Path<String>) -> Response {
    match store(&state, move |store| store.status(&upload)).await {
        Ok(status) => axum::Json(status).into_response(),
        Err(response) => response,
    }
}

/// `DELETE /auth/v1/import/{upload}`: stop an import.
pub async fn cancel(State(state): State<Shared>, Path(upload): Path<String>) -> Response {
    match store(&state, move |store| store.cancel(&upload)).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(response) => response,
    }
}

/// `POST /auth/v1/import/{upload}/check`: unpack the file, and give the
/// facts of the screen "Open the vault of the file", or 422
/// `archive_invalid` with the cause (ADR 0107 item 11).
pub async fn check(State(state): State<Shared>, Path(upload): Path<String>) -> Response {
    let limit = state.users.storage_limit();
    match store(&state, move |store| {
        store.check(&upload, limit, LibraryPolicy::Own)
    })
    .await
    {
        Ok(facts) => axum::Json(facts).into_response(),
        Err(response) => response,
    }
}

/// Who the new account is.
enum Identity {
    Email(String),
    Pubkey(String),
}

/// The identity of the new account: the Nostr key of the body, or the
/// email of the manifest, or the email of the body when the manifest has
/// none (ADR 0107 item 12).
fn identity_of(
    staged: &Staged,
    body: &ImportOpen,
    signer_key: Option<&str>,
) -> Result<Identity, Response> {
    let wanted = signer_key
        .map(str::to_owned)
        .or_else(|| {
            body.pubkey
                .as_ref()
                .map(|key| key.trim().to_ascii_lowercase())
        })
        // An archive of a Nostr account has no email and one key.
        .or_else(|| match staged.manifest.nostr_keys.as_slice() {
            [only] if staged.manifest.email.is_none() && body.email.is_none() => Some(only.clone()),
            _ => None,
        });
    if let Some(pubkey) = wanted {
        let known = staged.manifest.nostr_keys.contains(&pubkey)
            || staged
                .methods
                .iter()
                .any(|m| matches!(m, VaultMethod::Nostr { pubkey: p } if *p == pubkey));
        if !known {
            return Err(failure(
                StatusCode::UNPROCESSABLE_ENTITY,
                "pubkey_not_in_archive",
                "the file has no Nostr method for this public key",
            ));
        }
        return Ok(Identity::Pubkey(pubkey));
    }
    let email = staged
        .manifest
        .email
        .as_deref()
        .or(body.email.as_deref())
        .ok_or_else(|| {
            failure(
                StatusCode::UNPROCESSABLE_ENTITY,
                "email_required",
                "the file has no email address: give one",
            )
        })?;
    normalize_email(email).map(Identity::Email).ok_or_else(|| {
        failure(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_email",
            "give an email address",
        )
    })
}

fn account_exists() -> Response {
    failure(
        StatusCode::CONFLICT,
        "account_exists",
        "an account on this server has this email address or Nostr key",
    )
}

/// The credential of a NIP-46 connection: the signer decrypts the Nostr
/// wrapper of the vault, and the plain text is the vault key.
async fn signer_credential(
    state: &Shared,
    staged: &Staged,
    connection: &str,
) -> Result<(Credential, String, SignerLink), Response> {
    let (signer, pubkey) = match state.nip46.checkout(connection) {
        Ok(taken) => taken,
        Err(Checkout::Unknown) => return Err(connection_unknown()),
        Err(Checkout::NotReady) => {
            return Err(failure(
                StatusCode::CONFLICT,
                "connection_not_ready",
                "the signer is not connected",
            ));
        }
        Err(Checkout::Busy) => {
            return Err(failure(
                StatusCode::CONFLICT,
                "connection_busy",
                "another sign-in uses this signer",
            ));
        }
    };
    let release = || state.nip46.release(connection);
    let ciphertext = VaultFile::load(&staged.folder.join("vault.json"))
        .ok()
        .and_then(|file| file.nostr_ciphertext(&pubkey).map(str::to_owned));
    let Some(ciphertext) = ciphertext else {
        release();
        return Err(failure(
            StatusCode::UNPROCESSABLE_ENTITY,
            "no_nostr_wrapper",
            "the vault of the file has no Nostr method for this signer",
        ));
    };
    let Ok(key) = nostr::prelude::PublicKey::from_hex(&pubkey) else {
        release();
        return Err(internal("the public key is not correct"));
    };
    let for_signer = Arc::clone(&signer);
    let plain = match blocking_signer(move || for_signer.nip44_decrypt(&key, &ciphertext)).await {
        Ok(plain) => plain,
        Err(response) => {
            release();
            return Err(response);
        }
    };
    let link = SignerLink {
        connection: connection.to_owned(),
        signer,
    };
    Ok((
        Credential::Key(Zeroizing::new(plain.to_string())),
        pubkey,
        link,
    ))
}

/// `POST /auth/v1/import/{upload}/open`: open the vault of the file with a
/// passphrase, a recovery key, a vault key from a NIP-07 signer or a NIP-46
/// connection. The server then makes the account, moves the files to its
/// user folder and starts a session (ADR 0107 items 12 and 13).
pub async fn open(
    State(state): State<Shared>,
    Path(upload): Path<String>,
    request: Request,
) -> Response {
    let address = limits::client_address(state.client_header.as_deref(), &request);
    let body: ImportOpen = match read_json(request).await {
        Ok(body) => body,
        Err(response) => return response,
    };
    let staged = match store(&state, {
        let upload = upload.clone();
        move |store| store.staged(&upload)
    })
    .await
    {
        Ok(staged) => staged,
        Err(response) => return response,
    };
    let release_link = |link: &Option<SignerLink>| {
        if let Some(link) = link {
            state.nip46.release(&link.connection);
        }
    };
    // A NIP-46 connection gives the vault key and the public key at once.
    let (signer_credential, signer_key, link) = match &body.connection {
        Some(connection) => {
            if body.passphrase.is_some() || body.recovery_key.is_some() || body.key.is_some() {
                return failure(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "bad_request",
                    "give a connection or a credential, not both",
                );
            }
            match signer_credential(&state, &staged, connection).await {
                Ok((credential, pubkey, link)) => (Some(credential), Some(pubkey), Some(link)),
                Err(response) => return response,
            }
        }
        None => (None, None, None),
    };
    // The account check comes before the key derivation. It makes no
    // folder and no account.
    let identity = match identity_of(&staged, &body, signer_key.as_deref()) {
        Ok(identity) => identity,
        Err(response) => {
            release_link(&link);
            return response;
        }
    };
    let found = match &identity {
        Identity::Email(email) => state.accounts.account_by_email(email),
        Identity::Pubkey(pubkey) => state.accounts.account_by_pubkey(pubkey),
    };
    match found {
        Ok(None) => {}
        Ok(Some(_)) => {
            release_link(&link);
            return account_exists();
        }
        Err(_) => {
            release_link(&link);
            return internal("the account database failed");
        }
    }
    let counter = format!("import:{upload}");
    if state.attempts.blocked(&counter, &address) {
        release_link(&link);
        return limits::too_many_attempts();
    }
    let credential = match signer_credential {
        Some(credential) => credential,
        None => match body.credential() {
            Ok(credential) => credential,
            Err(error) => return from_api(error),
        },
    };
    let key = match verify(&state, &staged, credential, &counter, &address).await {
        Ok(key) => key,
        Err(response) => {
            release_link(&link);
            return response;
        }
    };
    make_account(&state, &upload, &staged, &identity, &key, link).await
}

/// Open the vault of the staged files with `credential`. A passphrase or a
/// recovery key needs a permit for the Argon2id derivation. A wrong
/// credential counts as a failed sign-in.
async fn verify(
    state: &Shared,
    staged: &Staged,
    credential: Credential,
    counter: &str,
    address: &str,
) -> Result<Zeroizing<String>, Response> {
    let _permit = if matches!(credential, Credential::Key(_)) {
        None
    } else {
        Some(limits::derivation_permit(state).await?)
    };
    let folder = staged.folder.clone();
    let verified =
        tokio::task::spawn_blocking(move || verify_credential(&folder, &credential)).await;
    match verified {
        Ok(Ok(key)) => Ok(key),
        Ok(Err(error)) => {
            let response = from_api(error);
            if response.status() == StatusCode::UNPROCESSABLE_ENTITY {
                state.attempts.record_failure(counter, address);
            }
            Err(response)
        }
        Err(_) => Err(internal("the import task failed")),
    }
}

/// Make the account, move the files and start the session. A failure
/// removes the account, its folder and the invite use.
async fn make_account(
    state: &Shared,
    upload: &str,
    staged: &Staged,
    identity: &Identity,
    key: &str,
    link: Option<SignerLink>,
) -> Response {
    let Ok(folder) = accounts::random_hex(16) else {
        return internal("the random source is not available");
    };
    let invite = staged.invite.as_deref();
    let made = match identity {
        Identity::Email(email) => state.accounts.sign_up(invite, email, &folder),
        Identity::Pubkey(pubkey) => state.accounts.sign_up_nostr(invite, pubkey, &folder),
    };
    let account_id = match made {
        Ok(id) => id,
        Err(SignUpError::InviteRequired) => return invite_required(),
        Err(SignUpError::EmailInUse) => return account_exists(),
        Err(SignUpError::Database(_)) => return internal("the account database failed"),
    };
    let target = state.users.folder_path(&folder);
    let result = open_new_user(
        state,
        account_id,
        &folder,
        &target,
        staged,
        key,
        link.as_ref(),
    )
    .await;
    match result {
        Ok(()) => {
            state.imports.finish(upload);
            start_session_full(
                state,
                account_id,
                false,
                link,
                StatusCode::CREATED,
                json!({ "new_account": true, "recovery_key": null, "vault": "open", "imported": true }),
            )
        }
        Err(response) => {
            // ADR 0107 item 13 and task 071c item 4: a failed import leaves
            // no account and no user folder.
            state.users.forget(account_id).await;
            let _ = state.accounts.undo_sign_up(account_id, invite);
            let remove = target.clone();
            let _ = tokio::task::spawn_blocking(move || std::fs::remove_dir_all(remove)).await;
            state.imports.finish(upload);
            if let Some(link) = &link {
                state.nip46.release(&link.connection);
            }
            response
        }
    }
}

async fn open_new_user(
    state: &Shared,
    account_id: i64,
    folder: &str,
    target: &FilePath,
    staged: &Staged,
    key: &str,
    link: Option<&SignerLink>,
) -> Result<(), Response> {
    let (from, to) = (staged.folder.clone(), target.to_path_buf());
    let moved = tokio::task::spawn_blocking(move || -> std::io::Result<PathBuf> {
        std::fs::rename(&from, &to)?;
        let library = to.join("library");
        studio_api::private_dir(&library)?;
        Ok(library)
    })
    .await
    .map_err(|_| internal("the import task failed"))?;
    let library = moved.map_err(|_| internal("the files of the import did not move"))?;
    let user = state
        .users
        .get(account_id, folder)
        .await
        .map_err(IntoResponse::into_response)?;
    // The user folder owns the library. This holds also when the collection
    // of the archive named a different one.
    user.handle
        .set_library_root(&library)
        .await
        .map_err(from_api)?;
    // ADR 0110 item 41: each app of the archive needs to connect again, and
    // no grant stays. This comes before the unlock, so that the sync of the
    // unlock makes no grant vault.
    user.handle
        .reset_apps_after_import()
        .await
        .map_err(from_api)?;
    user.handle.unlock_with_key(key).await.map_err(from_api)?;
    if let Some(link) = link {
        user.handle
            .set_remote_signer(Arc::clone(&link.signer))
            .await
            .map_err(from_api)?;
    }
    // The index follows the collection.
    crate::users::rebuild_app_index_of(
        &state.accounts,
        account_id,
        &state.users.folder_path(folder),
    )
    .map_err(|_| internal("the index of the apps did not change"))?;
    state
        .accounts
        .add_import_notice(account_id)
        .map_err(|_| internal("the account database failed"))?;
    Ok(())
}

/// `POST /auth/v1/import-notice/close`: close the notice of an import
/// (ADR 0107 item 14). The route needs a session and the CSRF header.
pub async fn close_notice(State(state): State<Shared>, request: Request) -> Response {
    let session = match session::active_session(&state, request.headers()).await {
        Ok(session) => session,
        Err(response) => return response,
    };
    if let Err(response) = session::check_csrf(&session, request.headers()) {
        return response;
    }
    match state.accounts.clear_import_notice(session.account_id) {
        Ok(_) => axum::Json(json!({ "imported": false })).into_response(),
        Err(_) => internal("the account database failed"),
    }
}

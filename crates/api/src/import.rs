//! The import of an account (ADR 0107 items 9 to 13).
//!
//! The browser sends the archive in parts, as ADR 0059 does for a music
//! file. [`ImportStore`] keeps the open imports of one place: the server
//! keeps one store for the sign-up page, and the desktop keeps one in its
//! workspace. A check unpacks the archive into a staging folder. The open
//! step moves the staged files to their place after the vault opened.
//!
//! The import writes nothing to a host and changes no feed (ADR 0107
//! item 13).

use std::collections::{BTreeSet, HashMap};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path as UrlPath, State};
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};
use studio_core::account_archive::{
    ArchiveCause, ArchiveError, LibraryPolicy, Limits, Manifest, clear_queues, reset_library_root,
    unpack_archive_with,
};
use studio_core::collection::Collection;
use studio_core::vault::{VaultFile, VaultMethod};
use zeroize::Zeroizing;

use crate::error::ApiError;
use crate::routes::{ApiResult, Shared, blocking, blocking_unlocked};
use crate::workspace::{private_dir, storage_full};

/// The size of a part of an import upload (ADR 0059 item 1).
pub const PART_SIZE: u64 = crate::uploads::PART_SIZE;

/// The most imports that wait at one time in one store.
const MAX_OPEN: usize = 8;

/// An import with no new request for this time expires.
const IDLE: Duration = Duration::from_hours(1);

/// What a check found, and what the open step needs.
#[derive(Debug, Clone)]
pub struct Staged {
    /// The staging folder with the unpacked files.
    pub folder: PathBuf,
    /// The manifest of the archive.
    pub manifest: Manifest,
    /// The unlock methods of the vault in the archive.
    pub methods: Vec<VaultMethod>,
    /// The invite code that the import started with.
    pub invite: Option<String>,
}

#[derive(Debug)]
struct Item {
    size: u64,
    received: BTreeSet<u32>,
    touched: Instant,
    invite: Option<String>,
    staged: Option<(Staged, Value)>,
}

#[derive(Debug, Default)]
struct Inner {
    swept: bool,
    items: HashMap<String, Item>,
}

/// The imports that wait for a check or an open. The store does file
/// work, so each call runs on a blocking thread.
#[derive(Debug)]
pub struct ImportStore {
    dir: PathBuf,
    state: Mutex<Inner>,
}

fn part_count(size: u64) -> u64 {
    size.div_ceil(PART_SIZE)
}

fn not_found() -> ApiError {
    ApiError::new(
        StatusCode::NOT_FOUND,
        "not_found",
        "the import is not there: start a new import",
    )
}

fn archive_invalid(error: &ArchiveError) -> ApiError {
    ApiError::new(
        StatusCode::UNPROCESSABLE_ENTITY,
        "archive_invalid",
        error.to_string(),
    )
}

fn io_error(error: &std::io::Error) -> ApiError {
    ApiError::internal(format!("the import file: {error}"))
}

impl ImportStore {
    /// Make a store that keeps its files in `dir`. The call does no file
    /// work. The folder comes at the first import.
    #[must_use]
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            state: Mutex::new(Inner::default()),
        }
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Inner>, ApiError> {
        self.state
            .lock()
            .map_err(|_| ApiError::internal("the import lock is poisoned"))
    }

    fn part_path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.part"))
    }

    fn staging_path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.dir"))
    }

    fn remove_files(&self, id: &str) {
        let _ = std::fs::remove_file(self.part_path(id));
        let _ = std::fs::remove_dir_all(self.staging_path(id));
    }

    /// Start an import of `size` bytes. `limit` is the storage limit
    /// (ADR 0107 item 15). `invite` is kept for the open step.
    ///
    /// # Errors
    ///
    /// Gives 507 `storage_full` when `size` is over the limit, and 429
    /// `too_many_imports` when too many imports wait.
    pub fn start(&self, size: u64, limit: u64, invite: Option<String>) -> Result<Value, ApiError> {
        if size == 0 {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "bad_size",
                "the file is empty",
            ));
        }
        if size > limit {
            return Err(storage_full());
        }
        private_dir(&self.dir).map_err(|e| io_error(&e))?;
        let mut state = self.lock()?;
        if !state.swept {
            // After a restart the files of older imports have no owner.
            if let Ok(entries) = std::fs::read_dir(&self.dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let _ = std::fs::remove_dir_all(path);
                    } else {
                        let _ = std::fs::remove_file(path);
                    }
                }
            }
            state.swept = true;
        }
        let now = Instant::now();
        let old: Vec<String> = state
            .items
            .iter()
            .filter(|(_, item)| now.duration_since(item.touched) > IDLE)
            .map(|(id, _)| id.clone())
            .collect();
        for id in old {
            state.items.remove(&id);
            self.remove_files(&id);
        }
        if state.items.len() >= MAX_OPEN {
            return Err(ApiError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "too_many_imports",
                "too many imports wait: try again later",
            ));
        }
        let id = crate::access::random_hex::<16>()?;
        let file = std::fs::File::create(self.part_path(&id)).map_err(|e| io_error(&e))?;
        file.set_len(size).map_err(|e| io_error(&e))?;
        state.items.insert(
            id.clone(),
            Item {
                size,
                received: BTreeSet::new(),
                touched: now,
                invite,
                staged: None,
            },
        );
        Ok(json!({ "upload": id, "part_size": PART_SIZE }))
    }

    /// Write part `n` of an import.
    ///
    /// # Errors
    ///
    /// Gives 404 for an unknown import and 422 `bad_part` for a part with
    /// a wrong number or size.
    pub fn put_part(&self, id: &str, n: u32, body: &[u8]) -> Result<Value, ApiError> {
        let size = {
            let mut state = self.lock()?;
            let item = state.items.get_mut(id).ok_or_else(not_found)?;
            if item.staged.is_some() {
                return Err(ApiError::new(
                    StatusCode::CONFLICT,
                    "already_checked",
                    "the file is checked already: start a new import",
                ));
            }
            item.touched = Instant::now();
            item.size
        };
        let bad =
            |message: String| ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, "bad_part", message);
        if u64::from(n) >= part_count(size) {
            return Err(bad(format!("part {n} is outside the file")));
        }
        let offset = u64::from(n) * PART_SIZE;
        let expected = (size - offset).min(PART_SIZE);
        if body.len() as u64 != expected {
            return Err(bad(format!(
                "part {n} must have {expected} bytes, and it has {}",
                body.len()
            )));
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .open(self.part_path(id))
            .map_err(|e| io_error(&e))?;
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| io_error(&e))?;
        file.write_all(body).map_err(|e| io_error(&e))?;
        file.flush().map_err(|e| io_error(&e))?;
        drop(file);
        let mut state = self.lock()?;
        let item = state.items.get_mut(id).ok_or_else(not_found)?;
        item.received.insert(n);
        Ok(status_of(id, item))
    }

    /// The size and the parts of an import.
    ///
    /// # Errors
    ///
    /// Gives 404 for an unknown import.
    pub fn status(&self, id: &str) -> Result<Value, ApiError> {
        let mut state = self.lock()?;
        let item = state.items.get_mut(id).ok_or_else(not_found)?;
        item.touched = Instant::now();
        Ok(status_of(id, item))
    }

    /// Stop an import and remove its files.
    ///
    /// # Errors
    ///
    /// Gives 404 for an unknown import.
    pub fn cancel(&self, id: &str) -> Result<(), ApiError> {
        let mut state = self.lock()?;
        state.items.remove(id).ok_or_else(not_found)?;
        self.remove_files(id);
        Ok(())
    }

    /// The import is complete, or it failed: remove its files.
    pub fn finish(&self, id: &str) {
        if let Ok(mut state) = self.lock() {
            state.items.remove(id);
        }
        self.remove_files(id);
    }

    /// Check the file of an import: unpack it into the staging folder
    /// (ADR 0107 item 11), fix its library path, and read its facts. A
    /// refusal removes the import.
    ///
    /// # Errors
    ///
    /// Gives 404 for an unknown import, 409 `upload_incomplete` for a
    /// missing part, and 422 `archive_invalid` with the cause.
    pub fn check(&self, id: &str, limit: u64, policy: LibraryPolicy) -> Result<Value, ApiError> {
        let invite = {
            let mut state = self.lock()?;
            let item = state.items.get_mut(id).ok_or_else(not_found)?;
            item.touched = Instant::now();
            if let Some((_, facts)) = &item.staged {
                return Ok(facts.clone());
            }
            let parts = u32::try_from(part_count(item.size)).unwrap_or(u32::MAX);
            if let Some(missing) = (0..parts).find(|n| !item.received.contains(n)) {
                return Err(ApiError::new(
                    StatusCode::CONFLICT,
                    "upload_incomplete",
                    format!("part {missing} is missing"),
                ));
            }
            item.invite.clone()
        };
        let staging = self.staging_path(id);
        let _ = std::fs::remove_dir_all(&staging);
        let result = self.unpack_and_read(id, &staging, limit, policy);
        match result {
            Ok((manifest, methods, facts)) => {
                let _ = std::fs::remove_file(self.part_path(id));
                let staged = Staged {
                    folder: staging,
                    manifest,
                    methods,
                    invite,
                };
                let mut state = self.lock()?;
                let item = state.items.get_mut(id).ok_or_else(not_found)?;
                item.staged = Some((staged, facts.clone()));
                Ok(facts)
            }
            Err(error) => {
                // A refusal ends the import. The user starts a new one.
                self.finish(id);
                Err(error)
            }
        }
    }

    fn unpack_and_read(
        &self,
        id: &str,
        staging: &Path,
        limit: u64,
        policy: LibraryPolicy,
    ) -> Result<(Manifest, Vec<VaultMethod>, Value), ApiError> {
        let file = std::fs::File::open(self.part_path(id)).map_err(|e| io_error(&e))?;
        let limits = Limits {
            max_total_bytes: limit,
            ..Limits::DEFAULT
        };
        let manifest = unpack_archive_with(std::io::BufReader::new(file), staging, limits)
            .map_err(|e| {
                if e.is_refusal() {
                    archive_invalid(&e)
                } else {
                    ApiError::internal(e.to_string())
                }
            })?;
        let refuse = |e: ArchiveError| {
            if e.is_refusal() {
                archive_invalid(&e)
            } else {
                ApiError::internal(e.to_string())
            }
        };
        reset_library_root(staging, policy).map_err(refuse)?;
        // ADR 0107 item 13: the import sends no podping and starts no
        // download, so the queues of the file do not come in.
        clear_queues(staging).map_err(refuse)?;
        let invalid = |cause: &str, message: &str| {
            ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "archive_invalid",
                format!("{cause}: {message}"),
            )
        };
        // The archive check does not read the vault. A vault that no
        // method of a different server opens is refused here.
        let vault = std::fs::read_to_string(staging.join("vault.json"))
            .map_err(|e| io_error(&e))
            .and_then(|text| {
                VaultFile::from_json(&text).map_err(|_| {
                    invalid(
                        ArchiveCause::MissingFile.code(),
                        "ADR 0107 item 6: vault.json is not a vault of Studio",
                    )
                })
            })?;
        let methods = vault.methods();
        let portable = methods.iter().any(|m| {
            matches!(
                m,
                VaultMethod::Passphrase | VaultMethod::Recovery | VaultMethod::Nostr { .. }
            )
        });
        if !portable {
            return Err(invalid(
                "no_portable_method",
                "ADR 0107 item 4: the vault has only passkeys, and a passkey does not work on a different server. Make a recovery key on the old server, and export again",
            ));
        }
        let counts = Collection::open(&staging.join("collection.sqlite"))
            .and_then(|collection| collection.account_counts())
            .map_err(|e| {
                invalid(
                    ArchiveCause::InvalidCollection.code(),
                    &format!("ADR 0107 item 11: collection.sqlite cannot be read: {e}"),
                )
            })?;
        let mut kinds = Vec::new();
        for method in &methods {
            let name = match method {
                VaultMethod::Passphrase => "passphrase",
                VaultMethod::Recovery => "recovery",
                VaultMethod::Nostr { .. } => "nostr",
                VaultMethod::Passkey { .. } => continue,
            };
            if !kinds.contains(&name) {
                kinds.push(name);
            }
        }
        let wrappers: Vec<Value> = methods
            .iter()
            .filter_map(|method| match method {
                VaultMethod::Nostr { pubkey } => vault
                    .nostr_ciphertext(pubkey)
                    .map(|ciphertext| json!({ "pubkey": pubkey, "ciphertext": ciphertext })),
                _ => None,
            })
            .collect();
        let facts = json!({
            "upload": id,
            "account": { "email": manifest.email, "nostr_keys": manifest.nostr_keys },
            "needs_email": manifest.email.is_none(),
            "studio_version": manifest.studio_version,
            "exported_at": manifest.exported_at,
            "counts": counts,
            "methods": kinds,
            "nostr_wrappers": wrappers,
        });
        Ok((manifest, methods, facts))
    }

    /// The checked import.
    ///
    /// # Errors
    ///
    /// Gives 404 for an unknown import and 409 `not_checked` before the
    /// check.
    pub fn staged(&self, id: &str) -> Result<Staged, ApiError> {
        let mut state = self.lock()?;
        let item = state.items.get_mut(id).ok_or_else(not_found)?;
        item.touched = Instant::now();
        item.staged
            .as_ref()
            .map(|(staged, _)| staged.clone())
            .ok_or_else(|| {
                ApiError::new(StatusCode::CONFLICT, "not_checked", "check the file first")
            })
    }
}

fn status_of(id: &str, item: &Item) -> Value {
    json!({
        "upload": id,
        "size": item.size,
        "part_size": PART_SIZE,
        "received": item.received,
        "checked": item.staged.is_some(),
    })
}

/// How the user opens the vault of an archive (ADR 0107 item 12).
#[derive(Debug)]
pub enum Credential {
    /// The passphrase of the vault.
    Passphrase(Zeroizing<String>),
    /// The recovery key.
    RecoveryKey(Zeroizing<String>),
    /// The vault key as base64, from a Nostr signer.
    Key(Zeroizing<String>),
}

/// The body of the open step. Exactly one of `passphrase`, `recovery_key`,
/// `key` and `connection` is set. The server reads the other fields.
#[derive(Debug, Default, Deserialize)]
pub struct ImportOpen {
    /// The passphrase.
    #[serde(default)]
    pub passphrase: Option<Zeroizing<String>>,
    /// The recovery key.
    #[serde(default)]
    pub recovery_key: Option<Zeroizing<String>>,
    /// The vault key as base64, from a NIP-07 signer.
    #[serde(default)]
    pub key: Option<Zeroizing<String>>,
    /// A NIP-46 connection ID. The server only.
    #[serde(default)]
    pub connection: Option<String>,
    /// The Nostr public key of the new account. The server only.
    #[serde(default)]
    pub pubkey: Option<String>,
    /// The email address of the new account, when the manifest has none.
    /// The server only.
    #[serde(default)]
    pub email: Option<String>,
}

impl ImportOpen {
    /// The credential of the body.
    ///
    /// # Errors
    ///
    /// Gives 422 `bad_request` unless exactly one credential is set.
    pub fn credential(&self) -> Result<Credential, ApiError> {
        let given = [
            self.passphrase
                .as_ref()
                .map(|t| Credential::Passphrase(t.clone())),
            self.recovery_key
                .as_ref()
                .map(|t| Credential::RecoveryKey(t.clone())),
            self.key.as_ref().map(|t| Credential::Key(t.clone())),
        ];
        let mut given: Vec<Credential> = given.into_iter().flatten().collect();
        if given.len() == 1 && self.connection.is_none() {
            return Ok(given.remove(0));
        }
        Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "bad_request",
            "give exactly one of passphrase, recovery_key and key",
        ))
    }
}

/// Open the vault of a staged import with `credential`, and give the vault
/// key as base64. The caller opens the new workspace with the key, so that
/// one Argon2id derivation is enough.
///
/// # Errors
///
/// Gives 422 `wrong_key` when the credential does not open the vault.
pub fn verify_credential(
    folder: &Path,
    credential: &Credential,
) -> Result<Zeroizing<String>, ApiError> {
    let file = VaultFile::load(&folder.join("vault.json"))?;
    let open = match credential {
        Credential::Passphrase(text) => file.open_with_passphrase(text)?,
        Credential::RecoveryKey(text) => file.open_with_recovery_key(text)?,
        Credential::Key(text) => file.open_with_key(text)?,
    };
    Ok(open.key_for_signer())
}

// ---------------------------------------------------------------- desktop

/// Give 403 `not_on_server` on a server. A server user imports at sign-up.
fn desktop_only(w: &crate::workspace::Workspace) -> Result<(), ApiError> {
    if w.rules.is_some() {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "not_on_server",
            "a server account imports at sign-up",
        ));
    }
    Ok(())
}

/// Give 409 `collection_not_empty` when the collection has a feed
/// (ADR 0107 item 9).
fn require_empty(w: &crate::workspace::Workspace) -> Result<(), ApiError> {
    if w.collection.account_counts()?.feeds > 0 {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "collection_not_empty",
            "ADR 0107 item 9: an import cannot replace feeds. Use a new data folder",
        ));
    }
    Ok(())
}

#[derive(Deserialize)]
pub(crate) struct StartImport {
    size: u64,
}

/// `POST /import`: start an import into the data folder.
pub(crate) async fn start(
    State(shared): State<Shared>,
    Json(body): Json<StartImport>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let store = blocking(shared, |w| {
        desktop_only(w)?;
        require_empty(w)?;
        Ok(Arc::clone(&w.imports))
    })
    .await?;
    let started =
        blocking_unlocked(move || store.start(body.size, Limits::DEFAULT.max_total_bytes, None))
            .await?;
    Ok((StatusCode::CREATED, Json(started)))
}

async fn store_of(shared: Shared) -> Result<Arc<ImportStore>, ApiError> {
    blocking(shared, |w| {
        desktop_only(w)?;
        Ok(Arc::clone(&w.imports))
    })
    .await
}

/// `PUT /import/{upload}/parts/{n}`: send part `n`.
pub(crate) async fn put_part(
    State(shared): State<Shared>,
    UrlPath((upload, n)): UrlPath<(String, u32)>,
    body: Bytes,
) -> ApiResult {
    let store = store_of(shared).await?;
    blocking_unlocked(move || store.put_part(&upload, n, &body))
        .await
        .map(Json)
}

/// `GET /import/{upload}`: the size and the parts that Studio has.
pub(crate) async fn get_import(
    State(shared): State<Shared>,
    UrlPath(upload): UrlPath<String>,
) -> ApiResult {
    let store = store_of(shared).await?;
    blocking_unlocked(move || store.status(&upload))
        .await
        .map(Json)
}

/// `DELETE /import/{upload}`: stop an import.
pub(crate) async fn delete_import(
    State(shared): State<Shared>,
    UrlPath(upload): UrlPath<String>,
) -> Result<StatusCode, ApiError> {
    let store = store_of(shared).await?;
    blocking_unlocked(move || store.cancel(&upload)).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /import/{upload}/check`: unpack and read the facts.
pub(crate) async fn check_import(
    State(shared): State<Shared>,
    UrlPath(upload): UrlPath<String>,
) -> ApiResult {
    let store = blocking(shared, |w| {
        desktop_only(w)?;
        require_empty(w)?;
        Ok(Arc::clone(&w.imports))
    })
    .await?;
    blocking_unlocked(move || {
        store.check(
            &upload,
            Limits::DEFAULT.max_total_bytes,
            LibraryPolicy::Relative,
        )
    })
    .await
    .map(Json)
}

/// `POST /import/{upload}/open`: open the vault of the file, and replace
/// the vault, the grant vaults, the collection and the library of the data
/// folder with the files of the archive (ADR 0107 items 9 and 12).
pub(crate) async fn open_import(
    State(shared): State<Shared>,
    UrlPath(upload): UrlPath<String>,
    Json(body): Json<ImportOpen>,
) -> ApiResult {
    blocking(shared, move |w| {
        desktop_only(w)?;
        require_empty(w)?;
        // The import replaces the vault and the host secrets of the owner.
        // An existing vault must show a recent unlock first (ADR 0106
        // item 2, ADR 0107 item 9).
        if w.vault_state() != crate::workspace::VaultState::Absent {
            w.require_recent_unlock()?;
        }
        let credential = body.credential()?;
        let staged = w.imports.staged(&upload)?;
        let key = verify_credential(&staged.folder, &credential)?;
        w.replace_account(&staged.folder, &key)?;
        w.imports.finish(&upload);
        Ok(Json(json!({
            "vault": w.vault_state(),
            "feeds": w.collection.account_counts()?.feeds,
        })))
    })
    .await
}

#[cfg(test)]
mod debug_tests {
    use super::*;

    #[test]
    fn debug_output_of_an_import_body_shows_no_secret() {
        let body: ImportOpen = serde_json::from_str(
            r#"{"passphrase":"secret-one","recovery_key":"secret-two","key":"secret-three"}"#,
        )
        .unwrap();
        let text = format!(
            "{body:?} {:?}",
            body.credential()
                .map(|c| format!("{c:?}"))
                .unwrap_or_default()
        );
        for secret in ["secret-one", "secret-two", "secret-three"] {
            assert!(
                !text.contains(secret),
                "ADR 0013 item 14: the Debug output of ImportOpen must not show a secret. Keep each secret field in Zeroizing, which hides its text in Debug."
            );
        }
        let credential = Credential::Passphrase(Zeroizing::new("secret-four".to_owned()));
        assert!(!format!("{credential:?}").contains("secret-four"));
    }
}

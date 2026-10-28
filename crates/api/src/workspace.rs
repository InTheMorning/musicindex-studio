//! The state of one local user: the collection and the vault.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use std::collections::{BTreeSet, HashMap, VecDeque};

use axum::http::StatusCode;
use serde_json::json;
use studio_core::collection::{Collection, FeedId};
use studio_core::connector::{self, HostEnv};
use studio_core::discover::Found;
use studio_core::feed::Feed;
use studio_core::inventory::Hosts;
use studio_core::library::Library;
use studio_core::musicindex::{DEFAULT_API, MusicIndex};
use studio_core::net::NetPolicy;
use studio_core::nip46::RemoteSigner;
use studio_core::nostr_events::{LOOKUP_RELAYS, Signing};
use studio_core::pipeline::AddressCheck;
use studio_core::remote;
use studio_core::vault::{OpenVault, VaultFile};
use studio_core::web::{Web, WebImage};
use studio_core::xml::Document;
use zeroize::Zeroizing;

use studio_core::announce::{self, Timing};
use studio_core::apps;
use studio_core::grant::{self, ShowGrant};
use studio_core::grant_vault;

use crate::announcer::Announcer;
use crate::downloads::DownloadQueue;
use crate::error::ApiError;

/// The prefix of the record in the user vault that keeps the key of a grant
/// vault (ADR 0058 item 3).
const GRANT_RECORD_PREFIX: &str = "grant-vault/";

/// How often the sync of the grant vaults runs (ADR 0058 item 10).
const GRANT_SYNC_INTERVAL: Duration = Duration::from_hours(1);

/// The number of images that [`ImageCache`] keeps (ADR 0032 item 13).
const MAX_IMAGES: usize = 64;

/// The images that the core already read, by their address (ADR 0025,
/// ADR 0032 items 13 and 14). The UI gets each image through the local API,
/// so that it loads no image from another server (ADR 0016).
///
/// The cache keeps the order that each image came in, so that it can remove
/// the oldest image when it is full.
#[derive(Debug, Default)]
pub(crate) struct ImageCache {
    order: VecDeque<String>,
    images: HashMap<String, WebImage>,
}

impl ImageCache {
    /// The image at this address, when the cache has it.
    pub(crate) fn get(&self, url: &str) -> Option<&WebImage> {
        self.images.get(url)
    }

    /// Keep an image at this address. When the cache is full, it removes the
    /// oldest image.
    pub(crate) fn put(&mut self, url: String, image: WebImage) {
        if !self.images.contains_key(&url) {
            if self.order.len() >= MAX_IMAGES
                && let Some(oldest) = self.order.pop_front()
            {
                self.images.remove(&oldest);
            }
            self.order.push_back(url.clone());
        }
        self.images.insert(url, image);
    }

    /// Remove each image. A refresh of a feed may read each address again,
    /// so Studio does not keep an image that may no longer be correct
    /// (ADR 0032 item 13).
    pub(crate) fn clear(&mut self) {
        self.order.clear();
        self.images.clear();
    }
}

/// A clone of the shared workspace, for the shell that owns it. The server
/// uses it for the vault (ADR 0055 items 4 and 5). Each function runs on a
/// blocking thread.
#[derive(Debug, Clone)]
pub struct WorkspaceHandle {
    shared: Arc<Mutex<Workspace>>,
}

impl WorkspaceHandle {
    pub(crate) fn new(shared: Arc<Mutex<Workspace>>) -> Self {
        Self { shared }
    }

    async fn run<T, F>(&self, work: F) -> Result<T, ApiError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Workspace) -> Result<T, ApiError> + Send + 'static,
    {
        let shared = Arc::clone(&self.shared);
        tokio::task::spawn_blocking(move || {
            let mut workspace = lock_workspace(&shared)?;
            work(&mut workspace)
        })
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
    }

    /// Run a function on the workspace. Only a test uses it.
    #[cfg(test)]
    pub(crate) async fn with_workspace<T, F>(&self, work: F) -> T
    where
        T: Send + 'static,
        F: FnOnce(&mut Workspace) -> T + Send + 'static,
    {
        self.run(move |w| Ok(work(w))).await.unwrap()
    }

    /// The state of the vault.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the workspace lock is poisoned.
    pub async fn vault_state(&self) -> Result<VaultState, ApiError> {
        self.run(|w| Ok(w.vault_state())).await
    }

    /// Make the vault. With no passphrase, the vault has no wrapper and
    /// stays in memory only. Gives the recovery key when one was asked.
    ///
    /// # Errors
    ///
    /// Gives 409 `vault_exists` when a vault exists, and an [`ApiError`]
    /// when the passphrase is not accepted or the vault cannot be saved.
    pub async fn create_vault(
        &self,
        passphrase: Option<&str>,
        recovery_key: bool,
    ) -> Result<Option<String>, ApiError> {
        let passphrase = passphrase.map(str::to_owned);
        self.run(move |w| w.create_vault(passphrase.as_deref(), recovery_key))
            .await
    }

    /// Open the vault with its passphrase.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when no vault file exists or the passphrase
    /// is not correct.
    pub async fn unlock_with_passphrase(&self, passphrase: &str) -> Result<(), ApiError> {
        let passphrase = passphrase.to_owned();
        self.run(move |w| w.unlock_with_passphrase(&passphrase))
            .await
    }

    /// Open the vault with the vault key, as base64.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when no vault file exists or the key is not
    /// correct.
    pub async fn unlock_with_key(&self, key_base64: &str) -> Result<(), ApiError> {
        let key = key_base64.to_owned();
        self.run(move |w| w.unlock_with_key(&key)).await
    }

    /// Open the vault with its recovery key.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when no vault file exists or the recovery key
    /// is not correct.
    pub async fn unlock_with_recovery_key(&self, recovery_key: &str) -> Result<(), ApiError> {
        let recovery_key = recovery_key.to_owned();
        self.run(move |w| w.unlock_with_recovery_key(&recovery_key))
            .await
    }

    /// Open the vault with the PRF output of a passkey (ADR 0056 item 22).
    /// The call clears the PRF output after use.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when no vault file exists, when the vault has
    /// no wrapper for the credential, or when the PRF output is not correct.
    pub async fn unlock_with_passkey(
        &self,
        credential_id: &str,
        prf_output: Zeroizing<[u8; 32]>,
    ) -> Result<(), ApiError> {
        let credential_id = credential_id.to_owned();
        self.run(move |w| w.unlock_with_passkey(&credential_id, &prf_output))
            .await
    }

    /// Add the wrapper of a passkey to the open vault, and save the vault.
    /// The call clears the PRF output after use.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the vault is not open or cannot be saved.
    pub async fn add_passkey_wrapper(
        &self,
        credential_id: &str,
        salt: [u8; 32],
        prf_output: Zeroizing<[u8; 32]>,
    ) -> Result<(), ApiError> {
        let credential_id = credential_id.to_owned();
        self.run(move |w| {
            w.require_open_vault()?;
            let vault = w
                .vault
                .as_mut()
                .ok_or_else(|| ApiError::internal("the vault closed"))?;
            vault.add_passkey_wrapper(&credential_id, &salt, &prf_output)?;
            w.touch_vault();
            w.save_vault()
        })
        .await
    }

    /// Remove the wrapper of a passkey from the open vault, and save the
    /// vault. Gives true when the vault had the wrapper.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the vault is not open or cannot be saved.
    pub async fn remove_passkey_wrapper(&self, credential_id: &str) -> Result<bool, ApiError> {
        let credential_id = credential_id.to_owned();
        self.run(move |w| {
            w.require_open_vault()?;
            let vault = w
                .vault
                .as_mut()
                .ok_or_else(|| ApiError::internal("the vault closed"))?;
            let removed = vault.remove_passkey_wrapper(&credential_id);
            w.touch_vault();
            w.save_vault()?;
            Ok(removed)
        })
        .await
    }

    /// The vault key as base64, for the remote signer of the user to
    /// encrypt with NIP-44 (ADR 0056 item 11). The text clears its memory
    /// when it goes out of scope.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the vault is not open.
    pub async fn signer_key(&self) -> Result<Zeroizing<String>, ApiError> {
        self.run(|w| {
            w.require_open_vault()?;
            let key = w
                .vault
                .as_ref()
                .ok_or_else(|| ApiError::internal("the vault closed"))?
                .key_for_signer();
            w.touch_vault();
            Ok(key)
        })
        .await
    }

    /// Add the Nostr wrapper of a public key to the open vault, and save
    /// the vault.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the vault is not open or cannot be saved.
    pub async fn add_nostr_wrapper(&self, pubkey: &str, ciphertext: &str) -> Result<(), ApiError> {
        let (pubkey, ciphertext) = (pubkey.to_owned(), ciphertext.to_owned());
        self.run(move |w| {
            w.require_open_vault()?;
            let vault = w
                .vault
                .as_mut()
                .ok_or_else(|| ApiError::internal("the vault closed"))?;
            vault.add_nostr_wrapper(&pubkey, &ciphertext);
            w.touch_vault();
            w.save_vault()
        })
        .await
    }

    /// The NIP-44 ciphertext of the Nostr wrapper for a public key. The
    /// vault can be closed. Gives `None` when the vault has no such wrapper.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when no vault file exists.
    pub async fn nostr_ciphertext(&self, pubkey: &str) -> Result<Option<String>, ApiError> {
        let pubkey = pubkey.to_owned();
        self.run(move |w| {
            if w.vault_state() == VaultState::Absent {
                return Err(ApiError::vault_required());
            }
            Ok(w.vault_file()?.nostr_ciphertext(&pubkey).map(str::to_owned))
        })
        .await
    }

    /// Keep a connected NIP-46 signer for the signing of a write (ADR 0056
    /// item 17). A pending connection ends.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the workspace lock is poisoned.
    pub async fn set_remote_signer(&self, signer: Arc<RemoteSigner>) -> Result<(), ApiError> {
        self.run(move |w| {
            w.signer.clear();
            w.signer = SignerSlot::Connected(signer);
            Ok(())
        })
        .await
    }

    /// Remove the NIP-46 signer when it is `signer`. A different signer
    /// stays, because another session of the user may have connected it.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the workspace lock is poisoned.
    pub async fn clear_remote_signer(&self, signer: &Arc<RemoteSigner>) -> Result<(), ApiError> {
        let signer = Arc::clone(signer);
        self.run(move |w| {
            if matches!(&w.signer, SignerSlot::Connected(own) if Arc::ptr_eq(own, &signer)) {
                w.signer = SignerSlot::None;
            }
            Ok(())
        })
        .await
    }

    /// Close the vault.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the workspace lock is poisoned.
    pub async fn lock(&self) -> Result<(), ApiError> {
        self.run(|w| {
            w.lock();
            Ok(())
        })
        .await
    }

    /// Close the vault when nothing has used a secret for `limit`. Gives
    /// true when this call closed the vault (ADR 0055 item 16).
    pub async fn lock_if_idle(&self, limit: Duration) -> bool {
        matches!(self.run(move |w| Ok(w.lock_if_idle(limit))).await, Ok(true))
    }

    /// The bytes in the user folder. Gives 0 when the workspace has no
    /// server rules.
    pub async fn storage_used(&self) -> u64 {
        self.run(|w| Ok(w.rules.as_ref().map_or(0, ServerRules::used)))
            .await
            .unwrap_or(0)
    }

    /// Close the vault and remove the user folder (ADR 0055 item 9). The
    /// workspace must have server rules, because they name the folder.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the workspace has no server rules or the
    /// folder cannot be removed.
    pub async fn delete_folder(&self) -> Result<(), ApiError> {
        self.run(|w| {
            w.lock();
            let rules = w
                .rules
                .as_ref()
                .ok_or_else(|| ApiError::internal("the workspace has no user folder"))?;
            std::fs::remove_dir_all(&rules.folder)
                .map_err(|e| ApiError::internal(format!("the user folder: {e}")))
        })
        .await
    }

    /// Give the workspace its library folder.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the collection cannot save the setting.
    pub async fn set_library_root(&self, root: &Path) -> Result<(), ApiError> {
        let root = root.to_path_buf();
        self.run(move |w| w.set_library_root(&root)).await
    }
}

pub(crate) fn lock_workspace(
    shared: &Arc<Mutex<Workspace>>,
) -> Result<MutexGuard<'_, Workspace>, ApiError> {
    shared
        .lock()
        .map_err(|_| ApiError::internal("the workspace lock is poisoned"))
}

/// The bytes that a storage check counts between two checks of a transfer
/// (ADR 0055 item 21).
pub(crate) const STORAGE_CHECK_BYTES: u64 = 8 * 1024 * 1024;

/// The rules of the server for one workspace (ADR 0055 items 21, 24 and
/// 25). The desktop sets none.
#[derive(Debug, Clone)]
pub struct ServerRules {
    /// The most bytes that the user folder can hold.
    pub storage_limit: u64,
    /// The user folder. The storage limit counts the files in it.
    pub folder: PathBuf,
    /// The only library folder that the user can have.
    pub library_root: PathBuf,
}

impl ServerRules {
    /// The bytes of the files in the user folder.
    #[must_use]
    pub fn used(&self) -> u64 {
        folder_bytes(&self.folder)
    }

    /// Give 507 `storage_full` when the user folder holds `limit` bytes or
    /// more.
    pub(crate) fn check_storage(&self) -> Result<(), ApiError> {
        if self.used() >= self.storage_limit {
            Err(storage_full())
        } else {
            Ok(())
        }
    }
}

/// The error of a user folder that is over its limit.
pub(crate) fn storage_full() -> ApiError {
    ApiError::new(
        StatusCode::INSUFFICIENT_STORAGE,
        "storage_full",
        "the storage limit of this account is reached",
    )
}

/// The sum of the sizes of the files under `root`. A link is not followed.
fn folder_bytes(root: &Path) -> u64 {
    let mut total = 0_u64;
    let mut pending = vec![root.to_path_buf()];
    while let Some(folder) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                pending.push(entry.path());
            } else if kind.is_file() {
                total = total.saturating_add(entry.metadata().map_or(0, |m| m.len()));
            }
        }
    }
    total
}

/// The state of the vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VaultState {
    /// No vault file exists. The first-run wizard makes one.
    Absent,
    /// The vault file exists and is closed.
    Locked,
    /// The vault is open in memory.
    Open,
}

/// A replacement for the address check of the write. Only a test sets it,
/// so that no test sends a request to the network.
#[derive(Clone)]
pub(crate) struct AddressChecker(pub(crate) Arc<dyn AddressCheck + Send + Sync>);

impl std::fmt::Debug for AddressChecker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AddressChecker")
    }
}

/// A change of the list of connected apps. The hash is the SHA-256 of the
/// token, as the `connected_apps` table keeps it. It is never the token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppChange {
    /// The user added an app.
    Added {
        /// The ID of the app.
        app_id: i64,
        /// The hash of the token of the app.
        token_hash: String,
    },
    /// The user removed an app.
    Removed {
        /// The ID of the app.
        app_id: i64,
        /// The hash of the token of the app.
        token_hash: String,
    },
}

/// A function that the workspace calls after a change of the app list. The
/// server gives one. The desktop gives none (ADR 0058 item 16).
#[derive(Clone)]
pub struct AppHook(Arc<dyn Fn(&AppChange) + Send + Sync>);

impl AppHook {
    /// Make a hook from a function.
    pub fn new(hook: impl Fn(&AppChange) + Send + Sync + 'static) -> Self {
        Self(Arc::new(hook))
    }
}

impl std::fmt::Debug for AppHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AppHook(..)")
    }
}

/// The collection and the vault in one data folder.
#[derive(Debug)]
pub struct Workspace {
    vault_path: PathBuf,
    /// The folder of the grant vaults: `grants/` in the data folder
    /// (ADR 0058 item 1).
    grants_dir: PathBuf,
    pub(crate) collection: Collection,
    pub(crate) vault: Option<OpenVault>,
    pub(crate) web: Web,
    /// The client for the hub. The operator gives the hub address, so the
    /// client keeps the policy `Open` (ADR 0057).
    pub(crate) hub_web: Web,
    /// The rule for the addresses that a user gives (ADR 0057).
    pub(crate) net_policy: NetPolicy,
    /// The address check of the write. `None` means that the write uses
    /// `web`.
    pub(crate) address_check: Option<AddressChecker>,
    pub(crate) index: MusicIndex,
    /// The relays that give the relay list and the server list of a key
    /// (ADR 0048 item 3). A test sets the address of a test relay.
    pub(crate) lookup_relays: Vec<String>,
    /// The scheme of a NIP-05 request: `https`. A test sets `http`.
    pub(crate) nip05_scheme: &'static str,
    /// The feeds of the last discovery, by address, so that an import does
    /// not read them again.
    pub(crate) discovered: HashMap<String, Found>,
    /// The images that the core already read (ADR 0025, ADR 0032 item 13).
    pub(crate) images: ImageCache,
    /// The media tickets of the audio player (ADR 0062 item 29).
    pub(crate) tickets: crate::media::Tickets,
    /// The staging folder for the body of an upload, before its import
    /// (ADR 0026, ADR 0033 item 1). Task 021e removes the pending-file
    /// store: an import now writes directly into the release folder.
    pub(crate) incoming_dir: PathBuf,
    /// The folder that held the pending files before the library existed
    /// (ADR 0036 item 5). The library route passes this to
    /// `migrate::into_library`, at the first start with a library folder.
    /// A data folder with no such files has no such folder either, and
    /// the move then does nothing for it.
    pub(crate) imports_dir: PathBuf,
    /// The queue of downloads (ADR 0038 items 7 and 9). A route clones the
    /// `Arc` and then unlocks the workspace.
    pub(crate) downloads: Arc<DownloadQueue>,
    /// The hub that gets the announcements (ADR 0052 item 1).
    pub(crate) hub: String,
    /// The times of the announcement queue (ADR 0052 items 14 and 15).
    pub(crate) announce_timing: Timing,
    /// The worker of the announcement queue.
    pub(crate) announcer: Arc<Announcer>,
    /// The rules of the server. `None` on the desktop.
    pub(crate) rules: Option<ServerRules>,
    /// The function for the changes of the app list. `None` on the desktop.
    pub(crate) app_hook: Option<AppHook>,
    /// The time of the last use of a vault secret (ADR 0055 item 16).
    vault_used: Mutex<Instant>,
    /// The NIP-46 signer of the user, or the wait for one (ADR 0056
    /// item 17). A route clones the `Arc` and then unlocks the workspace.
    pub(crate) signer: SignerSlot,
    /// The relays for a `nostrconnect://` address (ADR 0056 item 13).
    pub(crate) nip46_relays: Vec<String>,
    /// Counts the connections, so that a late answer of an old connection
    /// changes nothing.
    pub(crate) signer_generation: u64,
}

/// The default relay for a `nostrconnect://` address.
pub(crate) const DEFAULT_NIP46_RELAY: &str = "wss://relay.nsec.app";

/// What the workspace has for a NIP-46 signer.
#[derive(Debug, Default)]
pub(crate) enum SignerSlot {
    /// No signer.
    #[default]
    None,
    /// The signer has not answered the `nostrconnect://` address.
    Pending {
        /// Ends the wait for the signer.
        cancel: Arc<AtomicBool>,
        /// The connection that the wait belongs to.
        generation: u64,
    },
    /// The signer answered.
    Connected(Arc<RemoteSigner>),
    /// The last connection failed. The text is the error code.
    Failed(&'static str),
}

impl SignerSlot {
    /// Stop a pending wait. The slot is empty after the call.
    pub(crate) fn clear(&mut self) {
        if let Self::Pending { cancel, .. } = self {
            cancel.store(true, std::sync::atomic::Ordering::SeqCst);
        }
        *self = Self::None;
    }
}

/// The hub of a new workspace (ADR 0052 item 1).
#[cfg(not(test))]
const NEW_HUB: &str = announce::DEFAULT_HUB;

/// The hub of a new workspace in a test build. Port 9 answers nothing, so
/// that no test sends an announcement to the real hub (ADR 0045).
#[cfg(test)]
const NEW_HUB: &str = "http://127.0.0.1:9/pubnotify";

impl Workspace {
    /// Open the workspace in `data_dir`, and make the folder if necessary.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the folder or the collection cannot be opened.
    pub fn open(data_dir: &Path) -> Result<Self, ApiError> {
        std::fs::create_dir_all(data_dir)
            .map_err(|e| ApiError::internal(format!("{}: {e}", data_dir.display())))?;
        Ok(Self {
            vault_path: data_dir.join("vault.json"),
            grants_dir: data_dir.join("grants"),
            collection: Collection::open(&data_dir.join("collection.sqlite"))?,
            vault: None,
            web: Web::new(),
            hub_web: Web::new(),
            net_policy: NetPolicy::Open,
            address_check: None,
            index: MusicIndex::new(Web::new(), DEFAULT_API),
            lookup_relays: LOOKUP_RELAYS.iter().map(|r| (*r).to_owned()).collect(),
            nip05_scheme: "https",
            discovered: HashMap::new(),
            images: ImageCache::default(),
            tickets: crate::media::Tickets::default(),
            incoming_dir: data_dir.join("incoming"),
            imports_dir: data_dir.join("imports"),
            downloads: DownloadQueue::new(),
            hub: NEW_HUB.to_owned(),
            announce_timing: Timing::DEFAULT,
            announcer: Announcer::new(),
            rules: None,
            app_hook: None,
            vault_used: Mutex::new(Instant::now()),
            signer: SignerSlot::None,
            nip46_relays: vec![DEFAULT_NIP46_RELAY.to_owned()],
            signer_generation: 0,
        })
    }

    /// Use the rules of the server: no local folder host, one library
    /// folder and a storage limit (ADR 0055 items 21, 24 and 25).
    #[must_use]
    pub fn with_server_rules(mut self, rules: ServerRules) -> Self {
        self.rules = Some(rules);
        self
    }

    /// Call `hook` after each add and each removal of a connected app
    /// (ADR 0058 item 16).
    #[must_use]
    pub fn with_app_hook(mut self, hook: AppHook) -> Self {
        self.app_hook = Some(hook);
        self
    }

    /// Tell the hook about a change of the app list.
    pub(crate) fn app_changed(&self, change: &AppChange) {
        if let Some(hook) = &self.app_hook {
            (hook.0)(change);
        }
    }

    /// Record that a secret of the vault is in use now.
    pub(crate) fn touch_vault(&self) {
        *self
            .vault_used
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Instant::now();
    }

    /// Close the vault when no secret has been in use for `limit`. Gives
    /// true when it closed the vault.
    pub(crate) fn lock_if_idle(&mut self, limit: Duration) -> bool {
        let used = *self
            .vault_used
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.vault.is_some() && used.elapsed() >= limit {
            self.lock();
            true
        } else {
            false
        }
    }

    /// Use a different musicindex API, such as a test server. The
    /// operator gives this address, so the client keeps the policy `Open`.
    #[must_use]
    pub fn with_musicindex(mut self, base: &str) -> Self {
        self.index = MusicIndex::new(Web::new(), base);
        self
    }

    /// Use the network policy `policy` for each address that a user gives
    /// (ADR 0057). The feed client, the host connectors and the
    /// fingerprint route follow it. The musicindex client and the hub
    /// client keep `Open`.
    #[must_use]
    pub fn with_net_policy(mut self, policy: NetPolicy) -> Self {
        self.web = Web::with_policy(&policy);
        self.net_policy = policy;
        self
    }

    /// Make a media ticket valid for `lifetime`. A test uses this to make
    /// an expired ticket.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_ticket_lifetime(mut self, lifetime: Duration) -> Self {
        self.tickets = crate::media::Tickets::new(lifetime);
        self
    }

    /// Use these relays for a `nostrconnect://` address. Each relay goes
    /// through the network policy of the workspace.
    #[must_use]
    pub fn with_nip46_relays(mut self, relays: Vec<String>) -> Self {
        self.nip46_relays = relays;
        self
    }

    /// Use a different hub for the announcements, such as a test hub.
    #[must_use]
    pub fn with_hub(mut self, url: &str) -> Self {
        url.clone_into(&mut self.hub);
        self
    }

    /// Use other times for the announcement queue, such as short retry
    /// waits.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_announce_timing(mut self, timing: Timing) -> Self {
        self.announce_timing = timing;
        self
    }

    /// Use other lookup relays, such as test relays.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_lookup_relays(mut self, relays: Vec<String>) -> Self {
        self.lookup_relays = relays;
        self
    }

    /// Send the NIP-05 requests over `http`, so that a test server on
    /// loopback can answer them.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_plain_nip05(mut self) -> Self {
        self.nip05_scheme = "http";
        self
    }

    /// Use a different address check for the write, such as a test double.
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn with_address_check(mut self, check: Arc<dyn AddressCheck + Send + Sync>) -> Self {
        self.address_check = Some(AddressChecker(check));
        self
    }

    pub(crate) fn vault_state(&self) -> VaultState {
        if self.vault.is_some() {
            VaultState::Open
        } else if self.vault_path.exists() {
            VaultState::Locked
        } else {
            VaultState::Absent
        }
    }

    pub(crate) fn vault_file(&self) -> Result<VaultFile, ApiError> {
        Ok(VaultFile::load(&self.vault_path)?)
    }

    /// Write the vault to the disk. A vault with no wrapper stays in
    /// memory only, because no key could open the stored form.
    pub(crate) fn save_vault(&self) -> Result<(), ApiError> {
        if let Some(vault) = &self.vault
            && vault.file().has_wrapper()
        {
            vault.file().save(&self.vault_path)?;
        }
        Ok(())
    }

    /// Make the vault. With no passphrase, the vault has no wrapper and
    /// stays in memory only. Gives the recovery key when one was asked.
    /// The recovery key is in this one result only (ADR 0013).
    pub(crate) fn create_vault(
        &mut self,
        passphrase: Option<&str>,
        recovery_key: bool,
    ) -> Result<Option<String>, ApiError> {
        if self.vault_state() != VaultState::Absent {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "vault_exists",
                "the vault already exists",
            ));
        }
        let mut vault = match passphrase {
            Some(passphrase) => OpenVault::create(passphrase)?,
            None => OpenVault::create_with_no_wrapper()?,
        };
        let recovery = if recovery_key {
            Some(vault.add_recovery_key()?.as_str().to_owned())
        } else {
            None
        };
        self.vault = Some(vault);
        self.touch_vault();
        self.save_vault()?;
        Ok(recovery)
    }

    pub(crate) fn unlock_with_passphrase(&mut self, passphrase: &str) -> Result<(), ApiError> {
        let file = self.vault_file()?;
        self.vault = Some(file.open_with_passphrase(passphrase)?);
        self.touch_vault();
        self.sync_grant_vaults(None);
        Ok(())
    }

    pub(crate) fn unlock_with_key(&mut self, key_base64: &str) -> Result<(), ApiError> {
        let file = self.vault_file()?;
        self.vault = Some(file.open_with_key(key_base64)?);
        self.touch_vault();
        self.sync_grant_vaults(None);
        Ok(())
    }

    pub(crate) fn unlock_with_passkey(
        &mut self,
        credential_id: &str,
        prf_output: &[u8; 32],
    ) -> Result<(), ApiError> {
        let file = self.vault_file()?;
        self.vault = Some(file.open_with_passkey(credential_id, prf_output)?);
        self.touch_vault();
        self.sync_grant_vaults(None);
        Ok(())
    }

    /// Give 423 or 404 when the vault is not open.
    pub(crate) fn require_open_vault(&self) -> Result<(), ApiError> {
        if self.vault.is_some() {
            return Ok(());
        }
        Err(match self.vault_state() {
            VaultState::Absent => ApiError::vault_required(),
            _ => ApiError::vault_locked(),
        })
    }

    pub(crate) fn unlock_with_recovery_key(&mut self, recovery_key: &str) -> Result<(), ApiError> {
        let file = self.vault_file()?;
        self.vault = Some(file.open_with_recovery_key(recovery_key)?);
        self.touch_vault();
        self.sync_grant_vaults(None);
        Ok(())
    }

    /// Replace the passphrase of the open vault, and save the vault.
    pub(crate) fn set_passphrase(&mut self, passphrase: &str) -> Result<(), ApiError> {
        if self.vault.is_none() {
            return Err(match self.vault_state() {
                VaultState::Absent => ApiError::vault_required(),
                _ => ApiError::vault_locked(),
            });
        }
        let vault = self
            .vault
            .as_mut()
            .ok_or_else(|| ApiError::internal("the vault closed"))?;
        vault.set_passphrase(passphrase)?;
        self.touch_vault();
        self.save_vault()
    }

    pub(crate) fn lock(&mut self) {
        self.vault = None;
    }

    /// Give the workspace its library folder. A new account has no library
    /// folder yet, and the first start of the library does the move of
    /// the files (ADR 0036 item 1).
    pub(crate) fn set_library_root(&self, root: &Path) -> Result<(), ApiError> {
        let text = root
            .to_str()
            .ok_or_else(|| ApiError::internal("the library folder is not valid text"))?;
        self.collection.set_setting("library_root", &json!(text))?;
        Ok(())
    }

    /// The connector of each host, for the diff and the write
    /// (ADR 0036 item 6).
    ///
    /// A host that needs a secret while the vault is locked is left out
    /// of the set, and marked instead (ADR 0017). An operation that then
    /// needs that host gives `vault_locked`, through
    /// [`studio_core::inventory::InventoryErrorKind::VaultLocked`].
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the collection or a connector fails.
    pub(crate) fn hosts(&self) -> Result<Hosts, ApiError> {
        self.hosts_with(Signing::default())
    }

    /// The connector of one row, with the network policy of the workspace
    /// (ADR 0057).
    ///
    /// # Errors
    ///
    /// Gives the errors of `connector::from_row`.
    pub(crate) fn connector_of(
        &self,
        row: &studio_core::collection::ConnectorRow,
    ) -> Result<Box<dyn connector::Connector>, studio_core::connector::ConnectorError> {
        if row.secret_ref.is_some() && self.vault.is_some() {
            self.touch_vault();
        }
        connector::from_row_in(
            row,
            &HostEnv {
                vault: self.vault.as_ref(),
                policy: self.net_policy.clone(),
                ..HostEnv::default()
            },
        )
    }

    /// The connector of each host, like [`Workspace::hosts`], with the
    /// events that the user signed for a write (ADR 0050 item 4).
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the collection or a connector fails.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "each host gets its own copy of the signing"
    )]
    pub(crate) fn hosts_with(&self, signing: Signing) -> Result<Hosts, ApiError> {
        let mut hosts = Hosts::new();
        for row in self.collection.connectors()? {
            if row.secret_ref.is_some() && self.vault.is_none() {
                hosts.insert_locked(row.id);
                continue;
            }
            if row.secret_ref.is_some() && self.vault.is_some() {
                self.touch_vault();
            }
            let env = HostEnv {
                vault: self.vault.as_ref(),
                blobs: self.collection.kept_blobs(row.id)?,
                signing: signing.clone(),
                policy: self.net_policy.clone(),
            };
            let made = connector::from_row_in(&row, &env)?;
            hosts.insert(row.id, made);
        }
        Ok(hosts)
    }

    /// The connector of each host for a granted write (ADR 0058 item 11).
    /// The connectors get their secrets from `vault`, which is the grant
    /// vault. This function never reads the user vault, also when it is
    /// open. A host whose secret `vault` does not hold is marked as locked.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the collection or a connector fails.
    pub(crate) fn hosts_with_grant_vault(
        &self,
        vault: Option<&OpenVault>,
        signing: &Signing,
    ) -> Result<Hosts, ApiError> {
        let mut hosts = Hosts::new();
        for row in self.collection.connectors()? {
            if let Some(secret_ref) = row.secret_ref.as_deref() {
                let held = vault.is_some_and(|v| matches!(v.get(secret_ref), Ok(Some(_))));
                if !held {
                    hosts.insert_locked(row.id);
                    continue;
                }
            }
            let env = HostEnv {
                vault,
                blobs: self.collection.kept_blobs(row.id)?,
                signing: signing.clone(),
                policy: self.net_policy.clone(),
            };
            hosts.insert(row.id, connector::from_row_in(&row, &env)?);
        }
        Ok(hosts)
    }

    /// The secret IDs of a grant: the `secret_ref` of the connector of the
    /// feed XML. A host with no secret gives an empty list (ADR 0058
    /// item 2).
    fn grant_secret_ids(&self, feed_id: FeedId) -> Result<Vec<String>, ApiError> {
        let Some(connector_id) = self.collection.feed(feed_id)?.connector_id else {
            return Ok(Vec::new());
        };
        Ok(self
            .collection
            .connector(connector_id)?
            .secret_ref
            .into_iter()
            .collect())
    }

    fn grant_path(&self, grant_id: i64) -> PathBuf {
        self.grants_dir.join(format!("{grant_id}.json"))
    }

    /// Open the grant vault of a show with the token of a connected app.
    /// Gives `None` when the host of the feed XML needs no secret.
    ///
    /// # Errors
    ///
    /// Gives 423 `vault_locked` when the host needs a secret and the grant
    /// vault is not there, when the app has no wrapper in it, when the
    /// token is not the token of the app, or when the grant vault does not
    /// hold the secret (ADR 0058 item 13).
    pub(crate) fn open_grant_vault(
        &self,
        show: &ShowGrant,
        app_id: i64,
        token: &str,
    ) -> Result<Option<OpenVault>, ApiError> {
        let secret_ids = self.grant_secret_ids(show.feed_id)?;
        if secret_ids.is_empty() {
            return Ok(None);
        }
        let file =
            VaultFile::load(&self.grant_path(show.id)).map_err(|_| ApiError::vault_locked())?;
        let vault = grant_vault::open(file, app_id, token).map_err(|_| ApiError::vault_locked())?;
        for id in &secret_ids {
            if !matches!(vault.get(id), Ok(Some(_))) {
                return Err(ApiError::vault_locked());
            }
        }
        Ok(Some(vault))
    }

    /// Keep the grant vaults current (ADR 0058 items 6 to 10). The function
    /// does each step that is possible now, and nothing else:
    ///
    /// - With the user vault open, it makes each missing grant vault of an
    ///   active grant, makes again the grant vaults of the `refresh` feed,
    ///   and adds each missing wrapper of an app that has a key.
    /// - Always, it removes the wrapper of each removed app, and it removes
    ///   the grant vault and the record of each grant that is not active.
    ///
    /// An error goes into the log with no secret, and the next grant still
    /// runs. The function never fails its caller.
    pub(crate) fn sync_grant_vaults(&mut self, refresh: Option<FeedId>) {
        let apps = match apps::apps(&self.collection) {
            Ok(apps) => apps,
            Err(error) => {
                eprintln!("studio: the grant vaults cannot read the apps: {error}");
                return;
            }
        };
        let open = match grant::open_grants(&self.collection) {
            Ok(open) => open,
            Err(error) => {
                eprintln!("studio: the grant vaults cannot read the grants: {error}");
                return;
            }
        };
        let now = crate::announcer::now();
        let shows: Vec<ShowGrant> = open
            .into_iter()
            .filter(|show| !self.grant_is_over(show, now))
            .collect();
        let keep: BTreeSet<i64> = shows.iter().map(|show| show.id).collect();
        self.remove_stale_grant_vaults(&keep);
        for show in &shows {
            if let Err(message) = self.sync_one_grant(show, &apps, refresh) {
                eprintln!(
                    "studio: the grant vault of grant {} failed: {message}",
                    show.id
                );
            }
        }
    }

    /// True when the grant is over for certain. The check reads the feed XML
    /// in the release folder, so that it needs no host and no secret. When
    /// the function cannot tell, it gives `false`, and the grant stays.
    fn grant_is_over(&self, show: &ShowGrant, now: u64) -> bool {
        if show.ended_at.is_some() {
            return true;
        }
        let Ok(row) = self.collection.feed(show.feed_id) else {
            return false;
        };
        let Ok(library) = self.library() else {
            return false;
        };
        let Ok(feed) = library_feed(&library, &row) else {
            return false;
        };
        if grant::live_item(&feed, &show.item_guid).is_none() {
            return false;
        }
        grant::active(&self.collection, show.feed_id, &show.item_guid, &feed, now)
            .is_ok_and(|active| active.is_none())
    }

    /// Remove each grant vault file and each user vault record of a grant
    /// that is not in `keep`. The record goes only while the user vault is
    /// open.
    fn remove_stale_grant_vaults(&mut self, keep: &BTreeSet<i64>) {
        if let Ok(entries) = std::fs::read_dir(&self.grants_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let stale = match name.strip_suffix(".json") {
                    Some(id) => id.parse::<i64>().map_or(true, |id| !keep.contains(&id)),
                    // A temporary file of an interrupted save.
                    None => name.ends_with(".json.tmp"),
                };
                if stale && let Err(error) = std::fs::remove_file(entry.path()) {
                    eprintln!("studio: a grant vault file cannot be removed: {error}");
                }
            }
        }
        let Some(vault) = self.vault.as_mut() else {
            return;
        };
        let stale: Vec<String> = vault
            .ids()
            .filter(|id| {
                id.strip_prefix(GRANT_RECORD_PREFIX)
                    .is_some_and(|n| n.parse::<i64>().map_or(true, |n| !keep.contains(&n)))
            })
            .map(str::to_owned)
            .collect();
        if stale.is_empty() {
            return;
        }
        for id in &stale {
            vault.remove(id);
        }
        if let Err(error) = self.save_vault() {
            eprintln!("studio: the vault cannot be saved after a grant ended: {error:?}");
        }
    }

    /// The steps of [`Workspace::sync_grant_vaults`] for one active grant.
    fn sync_one_grant(
        &mut self,
        show: &ShowGrant,
        apps: &[apps::ConnectedApp],
        refresh: Option<FeedId>,
    ) -> Result<(), String> {
        let secret_ids = self.grant_secret_ids(show.feed_id).map_err(|e| e.code())?;
        let path = self.grant_path(show.id);
        let record = format!("{GRANT_RECORD_PREFIX}{}", show.id);
        if secret_ids.is_empty() {
            // A host with no secret needs no grant vault.
            let _ = std::fs::remove_file(&path);
            if let Some(vault) = self.vault.as_mut()
                && vault.remove(&record)
            {
                self.save_vault().map_err(|e| e.code())?;
            }
            return Ok(());
        }
        // The key of the grant vault, while the user vault is open.
        let key: Option<Zeroizing<String>> = match &self.vault {
            Some(vault) => vault
                .get(&record)
                .map_err(|e| e.to_string())?
                .and_then(|bytes| String::from_utf8(bytes.to_vec()).ok())
                .map(Zeroizing::new),
            None => None,
        };
        let mut file = VaultFile::load(&path).ok();
        if self.vault.is_some() {
            let usable = match (&file, &key) {
                (Some(file), Some(key)) => file.clone().open_with_key(key).is_ok(),
                _ => false,
            };
            if refresh == Some(show.feed_id) || !usable {
                return self.make_grant_vault(&secret_ids, apps, &path, &record);
            }
        }
        let Some(file) = file.as_mut() else {
            return Ok(());
        };
        let mut changed = false;
        for id in file.app_ids() {
            if !apps.iter().any(|app| app.id == id && app.has_key) {
                changed |= grant_vault::remove_wrapper(file, id);
            }
        }
        if let Some(key) = &key {
            for app in apps.iter().filter(|app| app.has_key) {
                if file.app_wrapper(app.id).is_none() {
                    grant_vault::add_wrapper(file, key, app).map_err(|e| e.to_string())?;
                    changed = true;
                }
            }
        }
        if changed {
            file.save(&path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Make the grant vault of a grant from the open user vault, and keep
    /// its key in the user vault.
    fn make_grant_vault(
        &mut self,
        secret_ids: &[String],
        apps: &[apps::ConnectedApp],
        path: &Path,
        record: &str,
    ) -> Result<(), String> {
        let vault = self
            .vault
            .as_mut()
            .ok_or_else(|| "the user vault is closed".to_owned())?;
        let ids: Vec<&str> = secret_ids.iter().map(String::as_str).collect();
        let (file, key) = grant_vault::make(vault, &ids, apps).map_err(|e| e.to_string())?;
        // The key goes into the user vault first. A file that this key does
        // not open is made again at the next sync.
        vault
            .put(record, key.as_bytes())
            .map_err(|e| e.to_string())?;
        self.save_vault().map_err(|e| e.code())?;
        std::fs::create_dir_all(&self.grants_dir).map_err(|e| e.to_string())?;
        file.save(path).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// The library folder, from the `library_root` setting (ADR 0036
    /// item 1).
    ///
    /// # Errors
    ///
    /// Gives 409 `no_library` when the setting is absent, or an
    /// [`ApiError`] when the collection fails or Studio cannot open the
    /// folder.
    pub(crate) fn library(&self) -> Result<Library, ApiError> {
        let root = self.collection.setting("library_root")?.ok_or_else(|| {
            ApiError::new(
                StatusCode::CONFLICT,
                "no_library",
                "Studio has no library folder yet. Set one first.",
            )
        })?;
        let root = root
            .as_str()
            .ok_or_else(|| ApiError::internal("the library_root setting is not a string"))?;
        Library::open(root).map_err(|error| ApiError::internal(error.to_string()))
    }

    /// Keep the remote file records of one feed current, with `feed` as
    /// the hosted feed (ADR 0038 item 3). A feed with no release folder
    /// has no records.
    ///
    /// An error goes into the log and does not stop the caller. The
    /// function never writes to a host.
    pub(crate) fn record_remote_files(&self, id: FeedId, feed: &Feed) {
        self.record_remote_files_with(id, feed, || self.hosts());
    }

    /// Like [`Workspace::record_remote_files`], with the hosts that `hosts`
    /// makes. A granted write gives the hosts of its grant vault.
    pub(crate) fn record_remote_files_with(
        &self,
        id: FeedId,
        feed: &Feed,
        hosts: impl FnOnce() -> Result<Hosts, ApiError>,
    ) {
        let result = (|| -> Result<(), String> {
            let row = self.collection.feed(id).map_err(|e| e.to_string())?;
            if row.folder.is_none() {
                return Ok(());
            }
            let hosts = hosts().map_err(|e| format!("{e:?}"))?;
            remote::record_remote_files(&self.collection, &hosts, id, feed)
                .map_err(|e| e.to_string())?;
            Ok(())
        })();
        if let Err(message) = result {
            eprintln!(
                "studio: the remote file records of feed {} failed: {message}",
                id.0
            );
        }
    }

    /// Keep the remote file records of each feed with a release folder
    /// current. Each feed gives its hosted feed from the feed XML in its
    /// release folder (ADR 0038 item 3). The function does nothing when
    /// Studio has no library folder yet.
    ///
    /// An error for one feed goes into the log, and the other feeds still
    /// run.
    pub fn record_all_remote_files(&self) {
        let Ok(library) = self.library() else {
            return;
        };
        let feeds = match self.collection.feeds() {
            Ok(feeds) => feeds,
            Err(error) => {
                eprintln!("studio: the list of feeds failed: {error}");
                return;
            }
        };
        for row in feeds {
            if row.folder.is_none() {
                continue;
            }
            match library_feed(&library, &row) {
                Ok(feed) => self.record_remote_files(row.id, &feed),
                Err(message) => {
                    eprintln!("studio: feed {}: {message}", row.id.0);
                }
            }
        }
    }

    /// Put each download that the collection kept as `queued` into the
    /// queue again (ADR 0038 item 9). The workers start when the router
    /// is ready. A download that failed stays failed, until the user
    /// starts it again.
    ///
    /// An error goes into the log and does not stop the caller.
    pub fn resume_downloads(&self) {
        let rows = match self.collection.downloads() {
            Ok(rows) => rows,
            Err(error) => {
                eprintln!("studio: the list of downloads failed: {error}");
                return;
            }
        };
        for row in rows.into_iter().filter(|row| row.state == "queued") {
            match self.collection.library_file(row.library_file_id) {
                Ok(file) if !file.local => {
                    self.downloads.add(file.id, file.feed_id, file.size);
                }
                Ok(_) => {
                    let _ = self.collection.remove_download(row.library_file_id);
                }
                Err(error) => eprintln!(
                    "studio: download of library file {} failed to resume: {error}",
                    row.library_file_id.0
                ),
            }
        }
    }

    /// Prepare the announcement queue at the start (ADR 0052 item 18). An
    /// interrupted announcement waits again, and a waiting announcement
    /// older than the maximum age fails. The worker starts when the router
    /// is ready, and it then sends each due announcement.
    ///
    /// An error goes into the log and does not stop the caller.
    pub fn resume_announcements(&self) {
        if let Err(error) = announce::expire(
            &self.collection,
            &self.announce_timing,
            crate::announcer::now(),
        ) {
            eprintln!("studio: the announcement queue failed to resume: {error}");
        }
        self.announcer.wake();
    }
}

/// The hosted feed of a feed row, from the feed XML in its release folder
/// (ADR 0036 item 3).
///
/// The feed XML has the name of the last part of the feed path or of the
/// feed address.
///
/// # Errors
///
/// Gives a message when the feed has no release folder, the folder has no
/// feed XML, or the XML is not a feed.
pub(crate) fn library_feed(
    library: &Library,
    row: &studio_core::collection::FeedRow,
) -> Result<Feed, String> {
    let folder = row
        .folder
        .as_deref()
        .ok_or_else(|| "the feed has no release folder".to_owned())?;
    let names = [row.path.as_deref(), row.public_url.as_deref()];
    let bytes = names
        .into_iter()
        .flatten()
        .map(|name| name.rsplit_once('/').map_or(name, |(_, last)| last))
        .find_map(|name| std::fs::read(library.path(folder, name)).ok())
        .ok_or_else(|| "the release folder has no feed XML".to_owned())?;
    let doc = Document::parse_bytes(&bytes).map_err(|e| e.to_string())?;
    Feed::read(&doc).map_err(|e| format!("the feed XML is not a feed: {e}"))
}

/// Start the thread that keeps the grant vaults current at the start and
/// each hour (ADR 0058 item 10). The thread holds a weak link, and it ends
/// when the workspace is gone.
pub(crate) fn spawn_grant_sync(shared: &Arc<Mutex<Workspace>>) {
    let weak = Arc::downgrade(shared);
    std::thread::spawn(move || {
        loop {
            let Some(shared) = weak.upgrade() else {
                return;
            };
            if let Ok(mut w) = lock_workspace(&shared) {
                w.sync_grant_vaults(None);
            }
            drop(shared);
            std::thread::sleep(GRANT_SYNC_INTERVAL);
        }
    });
}

//! The open users: one workspace for each account (ADR 0055 items 1 to 5).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::response::{IntoResponse, Response};
use studio_api::{
    AccountIdentity, ApiError, AppChange, AppHook, ServerRules, Workspace, WorkspaceHandle,
    workspace_router,
};
use studio_core::collection::Collection;
use studio_core::net::{NetPolicy, PublicRules};
use tokio::sync::Mutex;

use crate::accounts::Accounts;

/// The router and the handle of one open user.
#[derive(Debug, Clone)]
pub struct OpenUser {
    /// The `/api/v1` routes of the workspace of the user.
    pub router: Router,
    /// The vault functions of the workspace.
    pub handle: WorkspaceHandle,
}

/// The settings that each new workspace gets.
#[derive(Debug, Clone)]
pub struct UserSettings {
    /// The data folder. The user folders are in `<data>/users/`.
    pub data_dir: PathBuf,
    /// A different musicindex API.
    pub musicindex: Option<String>,
    /// A different hub for the announcements.
    pub hub: Option<String>,
    /// The rules of the policy `Public`. A test build adds the address of
    /// a local test server.
    pub rules: PublicRules,
    /// The most bytes in the folder of a user (ADR 0055 item 21).
    pub storage_limit: u64,
    /// False for `--hold-assets off` (ADR 0105 item 1).
    pub hold_assets: bool,
    /// The relays for a `nostrconnect://` address (ADR 0056 item 13).
    pub nip46_relays: Vec<String>,
    /// The database that keeps the index of the apps (ADR 0110 item 39).
    pub accounts: Arc<Accounts>,
    /// The public origin: the scheme, the host and the port. The signed
    /// requests of the apps and the pairing text use it (ADR 0110 items 6
    /// and 12).
    pub public_origin: String,
}

/// The open users, by account ID.
#[derive(Debug)]
pub struct Users {
    settings: UserSettings,
    open: Mutex<HashMap<i64, OpenUser>>,
    /// A test sets this, so that a write does not request a public address.
    #[cfg(test)]
    pub pass_all_addresses: std::sync::atomic::AtomicBool,
    /// A test sets this, so that it controls the time of the recent unlock
    /// (ADR 0106 item 2). A workspace that opens later uses it.
    #[cfg(test)]
    pub test_clock: std::sync::Mutex<Option<studio_api::Clock>>,
}

impl Users {
    /// Make an empty set of open users.
    #[must_use]
    pub fn new(settings: UserSettings) -> Self {
        Self {
            settings,
            open: Mutex::new(HashMap::new()),
            #[cfg(test)]
            pass_all_addresses: std::sync::atomic::AtomicBool::new(false),
            #[cfg(test)]
            test_clock: std::sync::Mutex::new(None),
        }
    }

    /// The most bytes in the folder of a user (ADR 0055 item 21). The import
    /// uses it as the limit of the archive (ADR 0107 item 15).
    #[must_use]
    pub fn storage_limit(&self) -> u64 {
        self.settings.storage_limit
    }

    /// The folder of a user.
    #[must_use]
    pub fn folder_path(&self, folder: &str) -> PathBuf {
        self.settings.data_dir.join("users").join(folder)
    }

    /// The open user of an account. The first call opens the workspace
    /// (ADR 0055 item 4).
    ///
    /// # Errors
    ///
    /// Gives the error response when the workspace cannot be opened.
    pub async fn get(&self, account_id: i64, folder: &str) -> Result<OpenUser, Response> {
        // The lock stays during the open, so that two requests of one
        // account never open two workspaces.
        let mut open = self.open.lock().await;
        if let Some(user) = open.get(&account_id) {
            return Ok(user.clone());
        }
        let path = self.folder_path(folder);
        let settings = self.settings.clone();
        #[cfg(test)]
        let pass_all = self
            .pass_all_addresses
            .load(std::sync::atomic::Ordering::SeqCst);
        #[cfg(test)]
        let clock = self.test_clock.lock().unwrap().clone();
        let user = tokio::task::spawn_blocking(move || {
            open_user(
                &settings,
                &path,
                account_id,
                #[cfg(test)]
                pass_all,
                #[cfg(test)]
                clock,
            )
        })
        .await
        .map_err(|_| crate::session::internal("the workspace did not open"))?
        .map_err(IntoResponse::into_response)?;
        open.insert(account_id, user.clone());
        Ok(user)
    }

    /// The open user of an account, when the account is open. This call
    /// never opens a workspace.
    pub async fn peek(&self, account_id: i64) -> Option<OpenUser> {
        self.open.lock().await.get(&account_id).cloned()
    }

    /// Each open user, with the account ID.
    pub async fn snapshot(&self) -> Vec<(i64, OpenUser)> {
        self.open
            .lock()
            .await
            .iter()
            .map(|(id, user)| (*id, user.clone()))
            .collect()
    }

    /// True when the folder has a waiting announcement or a queued
    /// download (ADR 0055 item 4).
    pub async fn is_busy_folder(&self, folder: &str) -> bool {
        let path = self.folder_path(folder);
        tokio::task::spawn_blocking(move || is_busy(&path))
            .await
            .unwrap_or(true)
    }

    /// Forget the open user of an account.
    pub async fn forget(&self, account_id: i64) {
        self.open.lock().await.remove(&account_id);
    }

    /// Open each user whose collection has a waiting announcement or a
    /// queued download (ADR 0055 item 4). The workers of the user then run.
    pub async fn open_busy(&self, accounts: &[(i64, String)]) {
        for (id, folder) in accounts {
            let path = self.folder_path(folder);
            let busy = tokio::task::spawn_blocking(move || is_busy(&path))
                .await
                .unwrap_or(false);
            if busy && let Err(error) = self.get(*id, folder).await {
                eprintln!(
                    "studio-server: a user workspace did not open: {}",
                    error.status()
                );
            }
        }
    }
}

fn is_busy(folder: &std::path::Path) -> bool {
    let file = folder.join("collection.sqlite");
    if !file.exists() {
        return false;
    }
    let Ok(collection) = Collection::open(&file) else {
        return false;
    };
    let announcing = matches!(collection.earliest_waiting_announcement(), Ok(Some(_)));
    let downloading = collection
        .downloads()
        .is_ok_and(|rows| rows.iter().any(|row| row.state == "queued"));
    announcing || downloading
}

fn open_user(
    settings: &UserSettings,
    path: &std::path::Path,
    account_id: i64,
    #[cfg(test)] pass_all: bool,
    #[cfg(test)] clock: Option<studio_api::Clock>,
) -> Result<OpenUser, ApiError> {
    // The manifest of an export holds the email and the Nostr key (ADR 0107
    // item 6).
    let identity = AccountIdentity {
        email: settings.accounts.email_of(account_id).ok().flatten(),
        pubkey: settings.accounts.pubkey_of(account_id).ok().flatten(),
    };
    let mut workspace = Workspace::open(path)?
        .with_identity(identity)
        .with_net_policy(NetPolicy::Public(settings.rules.clone()))
        .with_server_rules(ServerRules {
            storage_limit: settings.storage_limit,
            folder: path.to_path_buf(),
            library_root: path.join("library"),
            hold_assets: settings.hold_assets,
        })?;
    #[cfg(test)]
    if pass_all {
        workspace = workspace.with_address_check(Arc::new(PassAll));
    }
    #[cfg(test)]
    if let Some(clock) = clock {
        workspace = workspace.with_clock(clock);
    }
    workspace = workspace.with_nip46_relays(settings.nip46_relays.clone());
    workspace = workspace.with_public_origin(&settings.public_origin);
    // The index of the apps follows the changes of the app list (ADR 0110
    // item 39). The hook holds no secret.
    let index = Arc::clone(&settings.accounts);
    workspace = workspace.with_app_hook(AppHook::new(move |change| {
        let result = match change {
            AppChange::Added {
                app_id,
                code_hash,
                code_expires_at,
            } => index.add_app_code(code_hash, account_id, *app_id, *code_expires_at),
            AppChange::Paired { app_id, pubkey } => {
                index.finish_app_key(pubkey, account_id, *app_id)
            }
            AppChange::Removed { app_id } => index.remove_app_rows(account_id, *app_id),
        };
        if result.is_err() {
            eprintln!("studio-server: the index of the apps did not change");
        }
    }));
    if let Some(base) = &settings.musicindex {
        workspace = workspace.with_musicindex(base);
    }
    if let Some(url) = &settings.hub {
        workspace = workspace.with_hub(url);
    }
    // The same steps as the desktop does at the start.
    workspace.record_all_remote_files();
    workspace.resume_downloads();
    workspace.resume_announcements();
    let (router, handle) = workspace_router(workspace);
    Ok(OpenUser { router, handle })
}

/// Make the index of the apps again from the collection of each user
/// (ADR 0110 item 39). The public key of each active app and the code of
/// each waiting app are in the index after the call.
pub fn rebuild_app_index(users: &Users, accounts: &Accounts) -> Result<(), String> {
    for (id, folder) in accounts.folders()? {
        rebuild_app_index_of(accounts, id, &users.folder_path(&folder))?;
    }
    Ok(())
}

/// Make the rows of one account again from its collection. An import calls
/// it, so that the index follows the apps of the imported collection.
pub fn rebuild_app_index_of(
    accounts: &Accounts,
    id: i64,
    folder: &std::path::Path,
) -> Result<(), String> {
    let file = folder.join("collection.sqlite");
    let (keys, codes) = if file.exists() {
        let collection = Collection::open(&file).map_err(|e| e.to_string())?;
        let keys = studio_core::apps::apps(&collection)
            .map_err(|e| e.to_string())?
            .into_iter()
            .filter(|app| app.state == studio_core::apps::AppState::Active)
            .filter_map(|app| app.pubkey.map(|key| (app.id, key)))
            .collect::<Vec<_>>();
        let codes = studio_core::apps::known_codes(&collection)
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|code| (code.app_id, code.hash, code.expires_at))
            .collect::<Vec<_>>();
        (keys, codes)
    } else {
        (Vec::new(), Vec::new())
    };
    accounts.replace_app_index(id, &keys, &codes)
}

/// An address check that passes each address. A test uses it, so that no
/// test sends a request to the network.
#[cfg(test)]
struct PassAll;

#[cfg(test)]
impl studio_core::pipeline::AddressCheck for PassAll {
    fn check(&self, _url: &str) -> Result<(), String> {
        Ok(())
    }
}

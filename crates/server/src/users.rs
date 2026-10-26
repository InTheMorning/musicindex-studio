//! The open users: one workspace for each account (ADR 0055 items 1 to 5).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::response::{IntoResponse, Response};
use studio_api::{
    ApiError, AppChange, AppHook, ServerRules, Workspace, WorkspaceHandle, workspace_router,
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
    /// The relays for a `nostrconnect://` address (ADR 0056 item 13).
    pub nip46_relays: Vec<String>,
    /// The database that keeps the index of the app tokens (ADR 0058
    /// item 16).
    pub accounts: Arc<Accounts>,
}

/// The open users, by account ID.
#[derive(Debug)]
pub struct Users {
    settings: UserSettings,
    open: Mutex<HashMap<i64, OpenUser>>,
    /// A test sets this, so that a write does not request a public address.
    #[cfg(test)]
    pub pass_all_addresses: std::sync::atomic::AtomicBool,
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
        }
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
        let user = tokio::task::spawn_blocking(move || {
            open_user(
                &settings,
                &path,
                account_id,
                #[cfg(test)]
                pass_all,
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
) -> Result<OpenUser, ApiError> {
    // The workspace tells the index when the user adds or removes an app.
    let accounts = Arc::clone(&settings.accounts);
    let hook = AppHook::new(move |change| {
        let done = match change {
            AppChange::Added { app_id, token_hash } => {
                accounts.add_app_token(token_hash, account_id, *app_id)
            }
            AppChange::Removed { token_hash, .. } => accounts.remove_app_token(token_hash),
        };
        if done.is_err() {
            // The start of the server makes the rows again, so the app
            // works after a restart.
            eprintln!("studio-server: the index of the app tokens did not change");
        }
    });
    let mut workspace = Workspace::open(path)?
        .with_app_hook(hook)
        .with_net_policy(NetPolicy::Public(settings.rules.clone()))
        .with_server_rules(ServerRules {
            storage_limit: settings.storage_limit,
            folder: path.to_path_buf(),
            library_root: path.join("library"),
        });
    #[cfg(test)]
    if pass_all {
        workspace = workspace.with_address_check(Arc::new(PassAll));
    }
    workspace = workspace.with_nip46_relays(settings.nip46_relays.clone());
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

/// Make the rows of the app tokens again from the `connected_apps` table of
/// each user folder (ADR 0058 item 16). A folder with no collection, or
/// one that cannot be read, keeps its rows.
pub fn rebuild_app_tokens(users: &Users, accounts: &Accounts) -> Result<(), String> {
    for (id, folder) in accounts.folders()? {
        let file = users.folder_path(&folder).join("collection.sqlite");
        if !file.exists() {
            continue;
        }
        let Ok(collection) = Collection::open(&file) else {
            continue;
        };
        let Ok(rows) = studio_core::apps::token_hashes(&collection) else {
            continue;
        };
        accounts.replace_app_tokens(id, &rows)?;
    }
    Ok(())
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

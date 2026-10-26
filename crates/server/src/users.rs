//! The open users: one workspace for each account (ADR 0055 items 1 to 5).

use std::collections::HashMap;
use std::path::PathBuf;

use axum::Router;
use axum::response::{IntoResponse, Response};
use studio_api::{ApiError, ServerRules, Workspace, WorkspaceHandle, workspace_router};
use studio_core::collection::Collection;
use studio_core::net::{NetPolicy, PublicRules};
use tokio::sync::Mutex;

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
}

/// The open users, by account ID.
#[derive(Debug)]
pub struct Users {
    settings: UserSettings,
    open: Mutex<HashMap<i64, OpenUser>>,
}

impl Users {
    /// Make an empty set of open users.
    #[must_use]
    pub fn new(settings: UserSettings) -> Self {
        Self {
            settings,
            open: Mutex::new(HashMap::new()),
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
        let user = tokio::task::spawn_blocking(move || open_user(&settings, &path))
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

fn open_user(settings: &UserSettings, path: &std::path::Path) -> Result<OpenUser, ApiError> {
    let mut workspace = Workspace::open(path)?
        .with_net_policy(NetPolicy::Public(settings.rules.clone()))
        .with_server_rules(ServerRules {
            storage_limit: settings.storage_limit,
            folder: path.to_path_buf(),
            library_root: path.join("library"),
        });
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

//! The JSON routes of the local API.

use std::sync::{Arc, Mutex, MutexGuard};

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};
use studio_core::collection::{ConnectorId, FeedId};
use studio_core::connector::{self, LocalFolder};
use studio_core::pipeline;
use studio_core::vault::OpenVault;

use crate::error::ApiError;
use crate::workspace::Workspace;

/// The shared workspace of the local API.
pub(crate) type Shared = Arc<Mutex<Workspace>>;

type ApiResult = Result<Json<Value>, ApiError>;

/// Run blocking core work on a blocking thread (ADR 0015).
async fn blocking<T, F>(shared: Shared, work: F) -> Result<T, ApiError>
where
    T: Send + 'static,
    F: FnOnce(&mut Workspace) -> Result<T, ApiError> + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let mut workspace = lock(&shared)?;
        work(&mut workspace)
    })
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?
}

fn lock(shared: &Shared) -> Result<MutexGuard<'_, Workspace>, ApiError> {
    shared
        .lock()
        .map_err(|_| ApiError::internal("the workspace lock is poisoned"))
}

fn to_json(value: impl serde::Serialize) -> ApiResult {
    serde_json::to_value(value)
        .map(Json)
        .map_err(|e| ApiError::internal(e.to_string()))
}

pub(crate) async fn status(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        Ok(Json(json!({
            "version": env!("CARGO_PKG_VERSION"),
            "vault": w.vault_state(),
            "connectors": w.collection.connectors()?.len(),
            "feeds": w.collection.feeds()?.len(),
        })))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct CreateVault {
    passphrase: String,
    #[serde(default)]
    recovery_key: bool,
}

pub(crate) async fn create_vault(
    State(shared): State<Shared>,
    Json(body): Json<CreateVault>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        if w.vault_state() != crate::workspace::VaultState::Absent {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "vault_exists",
                "the vault already exists",
            ));
        }
        let mut vault = OpenVault::create(&body.passphrase)?;
        let recovery = if body.recovery_key {
            Some(vault.add_recovery_key()?)
        } else {
            None
        };
        w.vault = Some(vault);
        w.save_vault()?;
        // The recovery key is in this one response only. Studio cannot show
        // it again (ADR 0013).
        Ok((
            StatusCode::CREATED,
            Json(json!({ "recovery_key": recovery.as_ref().map(studio_core::vault::RecoveryKey::as_str) })),
        ))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct Unlock {
    passphrase: String,
}

pub(crate) async fn unlock_vault(
    State(shared): State<Shared>,
    Json(body): Json<Unlock>,
) -> ApiResult {
    blocking(shared, move |w| {
        let file = w.vault_file()?;
        w.vault = Some(file.open_with_passphrase(&body.passphrase)?);
        Ok(Json(json!({ "vault": w.vault_state() })))
    })
    .await
}

pub(crate) async fn lock_vault(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| {
        w.vault = None;
        Ok(Json(json!({ "vault": w.vault_state() })))
    })
    .await
}

pub(crate) async fn list_connectors(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| to_json(w.collection.connectors()?)).await
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum NewConnector {
    Local {
        name: String,
        root: String,
        base_url: Option<String>,
    },
}

pub(crate) async fn add_connector(
    State(shared): State<Shared>,
    Json(body): Json<NewConnector>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let NewConnector::Local {
            name,
            root,
            base_url,
        } = body;
        if !std::path::Path::new(&root).is_absolute() {
            return Err(ApiError::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "invalid_config",
                "the folder must be an absolute path",
            ));
        }
        let base_url = base_url.filter(|u| !u.trim().is_empty());
        let folder = LocalFolder::new(root, base_url);
        let id = w
            .collection
            .add_connector(LocalFolder::KIND, &name, &folder.config(), None)?;
        Ok((StatusCode::CREATED, to_json(w.collection.connector(id)?)?))
    })
    .await
}

pub(crate) async fn list_feeds(State(shared): State<Shared>) -> ApiResult {
    blocking(shared, |w| to_json(w.collection.feeds()?)).await
}

#[derive(Deserialize)]
pub(crate) struct NewFeed {
    public_url: Option<String>,
    connector_id: Option<i64>,
    path: Option<String>,
}

pub(crate) async fn add_feed(
    State(shared): State<Shared>,
    Json(body): Json<NewFeed>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let connector = match (body.connector_id, body.path.as_deref()) {
            (Some(id), Some(path)) => {
                let row = w.collection.connector(ConnectorId(id))?;
                let made = connector::from_row(&row, w.vault.as_ref())?;
                Some((row.id, path.to_owned(), made.public_url(path)))
            }
            (None, None) => None,
            _ => {
                return Err(ApiError::new(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "invalid_feed",
                    "give both a connector and a path, or neither",
                ));
            }
        };
        let public_url = body
            .public_url
            .filter(|u| !u.trim().is_empty())
            .or_else(|| connector.as_ref().and_then(|(_, _, url)| url.clone()));
        let id = w.collection.add_feed(
            public_url.as_deref(),
            connector.as_ref().map(|(c, p, _)| (*c, p.as_str())),
        )?;
        Ok((StatusCode::CREATED, to_json(w.collection.feed(id)?)?))
    })
    .await
}

pub(crate) async fn refresh_feed(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        let row = w.collection.feed(id)?;
        let Some(connector_id) = row.connector_id else {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "read_only",
                "this feed has no connector",
            ));
        };
        let connector_row = w.collection.connector(connector_id)?;
        if connector_row.secret_ref.is_some() && w.vault.is_none() {
            return Err(ApiError::vault_locked());
        }
        let made = connector::from_row(&connector_row, w.vault.as_ref())?;
        pipeline::refresh(&mut w.collection, made.as_ref(), id)?;
        to_json(w.collection.feed(id)?)
    })
    .await
}

pub(crate) async fn delete_feed(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        w.collection.delete_feed(FeedId(id))?;
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

pub(crate) async fn delete_connector(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    blocking(shared, move |w| {
        let row = w.collection.connector(ConnectorId(id))?;
        if row.secret_ref.is_some() && w.vault.is_none() {
            return Err(ApiError::vault_locked());
        }
        w.collection.delete_connector(row.id)?;
        if let (Some(secret_ref), Some(vault)) = (&row.secret_ref, w.vault.as_mut()) {
            vault.remove(secret_ref);
            w.save_vault()?;
        }
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}

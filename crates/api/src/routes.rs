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
pub(crate) struct Strength {
    passphrase: String,
}

/// The strength of a passphrase (ADR 0018). The response contains no part
/// of the passphrase.
pub(crate) async fn passphrase_strength(Json(body): Json<Strength>) -> ApiResult {
    to_json(studio_core::vault::passphrase_strength(&body.passphrase))
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
            // A read-only feed reads from its public address (ADR 0020).
            pipeline::refresh_public(&mut w.collection, &w.web, id)?;
            return to_json(w.collection.feed(id)?);
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

/// The feed files on the host of a connector (ADR 0019). Each candidate
/// tells if the collection already has it.
pub(crate) async fn connector_feeds(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> ApiResult {
    blocking(shared, move |w| {
        let row = w.collection.connector(ConnectorId(id))?;
        if row.secret_ref.is_some() && w.vault.is_none() {
            return Err(ApiError::vault_locked());
        }
        let made = connector::from_row(&row, w.vault.as_ref())?;
        let candidates = studio_core::discover::feed_candidates(made.as_ref())?;
        let known: Vec<String> = w
            .collection
            .feeds()?
            .into_iter()
            .filter(|f| f.connector_id == Some(row.id))
            .filter_map(|f| f.path)
            .collect();
        let feeds: Vec<Value> = candidates
            .feeds
            .iter()
            .map(|c| {
                let mut value = serde_json::to_value(c).unwrap_or(Value::Null);
                value["in_collection"] = Value::Bool(known.contains(&c.path));
                value
            })
            .collect();
        Ok(Json(
            json!({ "feeds": feeds, "truncated": candidates.truncated }),
        ))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct Discover {
    #[serde(default)]
    addresses: Vec<String>,
    #[serde(default)]
    guids: Vec<String>,
}

/// The URLs and the feed GUIDs that the collection has.
fn collection_keys(w: &Workspace) -> Result<(Vec<String>, Vec<String>), ApiError> {
    let feeds = w.collection.feeds()?;
    let urls = feeds.iter().filter_map(|f| f.public_url.clone()).collect();
    let guids = feeds
        .iter()
        .filter_map(|f| f.podcast_guid.as_deref().map(str::to_ascii_lowercase))
        .collect();
    Ok((urls, guids))
}

fn in_collection(found: &studio_core::discover::Found, keys: &(Vec<String>, Vec<String>)) -> bool {
    keys.0.contains(&found.url)
        || found
            .podcast_guid
            .as_deref()
            .is_some_and(|g| keys.1.contains(&g.to_ascii_lowercase()))
}

/// Find feeds from addresses and GUIDs (ADR 0020).
pub(crate) async fn discover(
    State(shared): State<Shared>,
    Json(body): Json<Discover>,
) -> ApiResult {
    blocking(shared, move |w| {
        let found =
            studio_core::discover::from_sources(&w.web, &w.index, &body.addresses, &body.guids);
        let keys = collection_keys(w)?;
        let feeds: Vec<Value> = found
            .feeds
            .iter()
            .map(|f| {
                let mut value = serde_json::to_value(f).unwrap_or(Value::Null);
                value["in_collection"] = Value::Bool(in_collection(f, &keys));
                value
            })
            .collect();
        w.discovered = found
            .feeds
            .into_iter()
            .map(|f| (f.url.clone(), f))
            .collect();
        Ok(Json(
            json!({ "feeds": feeds, "failed": found.failed, "truncated": found.truncated }),
        ))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct Search {
    q: String,
}

/// Search musicindex for feeds by name.
pub(crate) async fn search(
    State(shared): State<Shared>,
    axum::extract::Query(query): axum::extract::Query<Search>,
) -> ApiResult {
    blocking(shared, move |w| to_json(w.index.search(&query.q)?)).await
}

#[derive(Deserialize)]
pub(crate) struct Import {
    urls: Vec<String>,
}

/// Add read-only feeds by address (ADR 0020). A feed that the collection
/// has is not added again.
pub(crate) async fn import_feeds(
    State(shared): State<Shared>,
    Json(body): Json<Import>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        let mut added = Vec::new();
        let mut failed = Vec::new();
        for url in body.urls {
            let keys = collection_keys(w)?;
            let found = match w.discovered.get(&url) {
                Some(found) => (found.feed.clone(), found.version.clone()),
                None => match studio_core::discover::read_public(&w.web, &url) {
                    Ok(read) => read,
                    Err(message) => {
                        failed.push(json!({ "source": url, "message": message }));
                        continue;
                    }
                },
            };
            let guid = found.0.podcast_guid.as_deref().map(str::to_ascii_lowercase);
            if keys.0.contains(&url) || guid.is_some_and(|g| keys.1.contains(&g)) {
                continue;
            }
            let id = w.collection.add_feed(Some(&url), None)?;
            w.collection.record_state(id, &found.0, &found.1)?;
            added.push(w.collection.feed(id)?);
        }
        Ok((
            StatusCode::CREATED,
            Json(json!({ "added": added, "failed": failed })),
        ))
    })
    .await
}

/// Link each read-only feed to its file in the folder of a connector, by
/// `podcast:guid` (ADR 0019, ADR 0020).
pub(crate) async fn match_connector(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
) -> ApiResult {
    blocking(shared, move |w| {
        let row = w.collection.connector(ConnectorId(id))?;
        if row.secret_ref.is_some() && w.vault.is_none() {
            return Err(ApiError::vault_locked());
        }
        let made = connector::from_row(&row, w.vault.as_ref())?;
        let candidates = studio_core::discover::feed_candidates(made.as_ref())?;
        let mut linked = Vec::new();
        for feed in w.collection.feeds()? {
            if feed.connector_id.is_some() {
                continue;
            }
            let Some(guid) = feed.podcast_guid.as_deref() else {
                continue;
            };
            let file = candidates.feeds.iter().find(|c| {
                c.podcast_guid
                    .as_deref()
                    .is_some_and(|g| g.eq_ignore_ascii_case(guid))
            });
            if let Some(file) = file {
                w.collection
                    .set_feed_connector(feed.id, row.id, &file.path)?;
                linked.push(w.collection.feed(feed.id)?);
            }
        }
        Ok(Json(json!({ "linked": linked })))
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct MspImport {
    backups: Vec<Value>,
}

/// Import MSP 2.0 backups (ADR 0020). Each feed is read-only, and each edit
/// token goes into the vault. The response contains no edit token.
pub(crate) async fn import_msp(
    State(shared): State<Shared>,
    Json(body): Json<MspImport>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    blocking(shared, move |w| {
        if w.vault.is_none() {
            return Err(match w.vault_state() {
                crate::workspace::VaultState::Absent => ApiError::vault_required(),
                _ => ApiError::vault_locked(),
            });
        }
        let mut added = Vec::new();
        let mut failed = Vec::new();
        for value in &body.backups {
            let backup = match studio_core::msp::MspBackup::from_json(value) {
                Ok(backup) => backup,
                Err(message) => {
                    failed.push(json!({ "source": "backup file", "message": message }));
                    continue;
                }
            };
            if let Some(vault) = w.vault.as_mut() {
                vault.put(&backup.secret_ref(), backup.edit_token.as_bytes())?;
            }
            let keys = collection_keys(w)?;
            if keys.0.contains(&backup.feed_url) {
                continue;
            }
            match studio_core::discover::read_public(&w.web, &backup.feed_url) {
                Ok((feed, version)) => {
                    let guid = feed.podcast_guid.as_deref().map(str::to_ascii_lowercase);
                    if guid.is_some_and(|g| keys.1.contains(&g)) {
                        continue;
                    }
                    let id = w.collection.add_feed(Some(&backup.feed_url), None)?;
                    w.collection.record_state(id, &feed, &version)?;
                    added.push(w.collection.feed(id)?);
                }
                Err(message) => {
                    failed.push(json!({ "source": backup.feed_url, "message": message }));
                }
            }
        }
        w.save_vault()?;
        Ok((
            StatusCode::CREATED,
            Json(json!({ "added": added, "failed": failed })),
        ))
    })
    .await
}

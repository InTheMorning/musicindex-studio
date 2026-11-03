//! The temporary area of a server (ADR 0122 items 9 and 10).
//!
//! The area is a folder in the user folder. Its files go 7 days after the
//! last write. Each feed of the area keeps its content in a draft first.

use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

use axum::http::StatusCode;
use studio_core::collection::{ConnectorId, ConnectorRow, FeedId};
use studio_core::connector::{self, TEMPORARY_KIND};
use studio_core::pipeline::{self, PipelineErrorKind};

use crate::error::ApiError;
use crate::workspace::Workspace;

/// The time after the last write when the files of an area go (ADR 0122
/// item 10).
const KEEP: Duration = Duration::from_hours(7 * 24);

/// True when a row is a temporary area that the server has expired. The
/// config holds the time of the expiry as `expired_at`.
pub(crate) fn is_expired(row: &ConnectorRow) -> bool {
    connector::is_expired_temporary(row)
}

/// The IDs of the expired temporary areas.
///
/// # Errors
///
/// Gives an [`ApiError`] when the collection fails.
pub(crate) fn expired_areas(w: &Workspace) -> Result<Vec<ConnectorId>, ApiError> {
    Ok(w.collection
        .connectors()?
        .iter()
        .filter(|row| is_expired(row))
        .map(|row| row.id)
        .collect())
}

/// Refuse a write whose file host is an expired temporary area (ADR 0122
/// item 10). The file host is the target of the draft feed move, or else
/// the connector of the feed.
///
/// # Errors
///
/// Gives 409 `temporary_expired` for an expired area, and an [`ApiError`]
/// when the collection fails.
pub(crate) fn check_write_target(w: &Workspace, id: FeedId) -> Result<(), ApiError> {
    let target = match w.collection.draft_feed_move(id)? {
        Some(moved) => moved.connector_id,
        None => match w.collection.feed(id)?.connector_id {
            Some(connector_id) => connector_id,
            None => return Ok(()),
        },
    };
    if is_expired(&w.collection.connector(target)?) {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "temporary_expired",
            "The temporary area has expired. Move this feed to a real file host.",
        ));
    }
    Ok(())
}

/// Remove the files of each temporary area with no write for 7 days (ADR
/// 0122 item 10). First, each written feed of the area with no draft gets
/// a draft from its file, so that the feed keeps its content. Then the
/// area loses its files and subfolders, and its config gets the time of the
/// expiry. The folder stays. The function reads no secret.
///
/// # Errors
///
/// Gives an [`ApiError`] when the collection or the file system fails. The
/// files then stay, and the next sweep tries again.
pub(crate) fn sweep(w: &Workspace, now: u64) -> Result<(), ApiError> {
    let Some(root) = w.temporary_root() else {
        return Ok(());
    };
    for row in w.collection.connectors()? {
        if row.kind != TEMPORARY_KIND || is_expired(&row) {
            continue;
        }
        let Some(last) = last_write(&root)? else {
            continue;
        };
        if now.saturating_sub(last) < KEEP.as_secs() {
            continue;
        }
        let made = w.connector_of(&row)?;
        keep_drafts(w, &row, made.as_ref())?;
        clear_folder(&root)?;
        let config = serde_json::json!({ "expired_at": now }).to_string();
        w.collection.set_connector_config(row.id, &config)?;
    }
    Ok(())
}

/// Give each feed of the area that has a file and no draft a draft from
/// its file. A feed with no file on the area has no content to keep.
fn keep_drafts(
    w: &Workspace,
    row: &ConnectorRow,
    made: &dyn connector::Connector,
) -> Result<(), ApiError> {
    for feed in w.collection.feeds()? {
        if feed.connector_id != Some(row.id)
            || feed.path.is_none()
            || w.collection.draft(feed.id)?.is_some()
        {
            continue;
        }
        match pipeline::start_draft(&w.collection, made, feed.id) {
            Ok(_) => {}
            Err(error) if error.kind() == PipelineErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

/// The newest modification time of a file under `root`, in seconds since
/// the Unix epoch. Gives `None` when the folder has no file. A link is not
/// followed.
fn last_write(root: &Path) -> Result<Option<u64>, ApiError> {
    if !root.is_dir() {
        return Ok(None);
    }
    let mut newest: Option<u64> = None;
    let mut pending = vec![root.to_path_buf()];
    while let Some(folder) = pending.pop() {
        for entry in std::fs::read_dir(&folder).map_err(|e| io_error(&e))? {
            let entry = entry.map_err(|e| io_error(&e))?;
            let meta = entry.metadata().map_err(|e| io_error(&e))?;
            if meta.is_dir() {
                pending.push(entry.path());
            } else if meta.is_file() {
                let modified = meta.modified().map_err(|e| io_error(&e))?;
                let seconds = modified
                    .duration_since(UNIX_EPOCH)
                    .map_or(0, |time| time.as_secs());
                newest = Some(newest.map_or(seconds, |old| old.max(seconds)));
            }
        }
    }
    Ok(newest)
}

/// Remove each file and each subfolder of `root`. The folder stays.
fn clear_folder(root: &Path) -> Result<(), ApiError> {
    if !root.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(root).map_err(|e| io_error(&e))? {
        let entry = entry.map_err(|e| io_error(&e))?;
        let kind = entry.file_type().map_err(|e| io_error(&e))?;
        if kind.is_dir() {
            std::fs::remove_dir_all(entry.path()).map_err(|e| io_error(&e))?;
        } else {
            std::fs::remove_file(entry.path()).map_err(|e| io_error(&e))?;
        }
    }
    Ok(())
}

fn io_error(error: &std::io::Error) -> ApiError {
    ApiError::internal(format!("the temporary area: {error}"))
}

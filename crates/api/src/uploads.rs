//! Uploads in parts (ADR 0059 items 1 to 7).
//!
//! The UI sends a music file in parts of 8 MiB. Studio writes each part at
//! its position in one file in `incoming/`, and records the part in the
//! collection. `finish` runs the import of ADR 0026 on the full file.

use std::io::SeekFrom;
use std::path::{Path as FilePath, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};
use studio_core::collection::{FeedId, UploadRow};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};

use crate::error::ApiError;
use crate::routes::{self, ApiResult, Shared, blocking, to_json, upload_file_name};
use crate::workspace::{ServerRules, Workspace, lock_workspace, storage_full};

/// The size of a part (ADR 0059 item 1). It is also the body limit of the
/// part route.
pub(crate) const PART_SIZE: u64 = 8 * 1024 * 1024;

/// The most bytes of one file (ADR 0059 item 3).
const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// An upload with no new part for this time expires (ADR 0059 item 4).
const EXPIRY_SECONDS: u64 = 24 * 60 * 60;

/// The time between two clean steps.
const CLEAN_INTERVAL: Duration = Duration::from_hours(1);

/// The number of parts of a file.
fn part_count(size: u64) -> u64 {
    size.div_ceil(PART_SIZE)
}

/// The staging file of an upload, while parts come.
fn part_path(incoming: &FilePath, id: &str) -> PathBuf {
    incoming.join(format!("{id}.part"))
}

fn upload_json(row: &UploadRow) -> Value {
    json!({
        "upload": row.id,
        "feed": row.feed_id.0,
        "name": row.name,
        "size": row.size,
        "part_size": PART_SIZE,
        "received": row.received,
        "replace": row.replace.is_some(),
    })
}

fn random_id() -> Result<String, ApiError> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes)
        .map_err(|_| ApiError::internal("the random source is not available"))?;
    Ok(bytes.iter().fold(String::with_capacity(32), |mut text, b| {
        use std::fmt::Write as _;
        let _ = write!(text, "{b:02x}");
        text
    }))
}

fn bad_part(message: impl Into<String>) -> ApiError {
    ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, "bad_part", message)
}

fn io_error(error: &std::io::Error) -> ApiError {
    if error.kind() == std::io::ErrorKind::NotFound {
        ApiError::new(
            StatusCode::NOT_FOUND,
            "not_found",
            "the upload is not there",
        )
    } else {
        ApiError::internal(error.to_string())
    }
}

/// Give 507 when the user folder, with `extra` more bytes, goes over the
/// storage limit (ADR 0055 item 21). The size of an open upload is in the
/// folder from its start, because the staging file has its full size.
async fn check_room(rules: Option<ServerRules>, extra: u64) -> Result<(), ApiError> {
    let Some(rules) = rules else {
        return Ok(());
    };
    tokio::task::spawn_blocking(move || {
        let used = rules.used();
        if used >= rules.storage_limit || used.saturating_add(extra) > rules.storage_limit {
            Err(storage_full())
        } else {
            Ok(())
        }
    })
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?
}

/// The body of the route that starts an upload.
#[derive(Deserialize)]
pub(crate) struct StartUpload {
    name: String,
    size: u64,
    /// True for the new audio file of a track (ADR 0040 item 3).
    #[serde(default)]
    replace: bool,
}

/// `POST /feeds/{id}/uploads`: start an upload (ADR 0059 item 1).
pub(crate) async fn start_upload(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<StartUpload>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let name = upload_file_name(&body.name)?;
    if body.size > MAX_FILE_BYTES {
        return Err(ApiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "too_large",
            "a file can have 2 GiB at most",
        ));
    }
    let (incoming, rules) = blocking(Arc::clone(&shared), |w| {
        Ok((w.incoming_dir.clone(), w.rules.clone()))
    })
    .await?;
    check_room(rules, body.size).await?;
    let upload = random_id()?;
    let row = UploadRow {
        id: upload.clone(),
        feed_id: FeedId(id),
        name,
        size: body.size,
        replace: body.replace.then(|| "true".to_owned()),
        received: Vec::new(),
        updated_at: crate::announcer::now(),
    };
    blocking(Arc::clone(&shared), move |w| {
        w.collection.feed(FeedId(id))?;
        Ok(())
    })
    .await?;
    tokio::fs::create_dir_all(&incoming)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let path = part_path(&incoming, &upload);
    let file = tokio::fs::File::create(&path)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let sized = file.set_len(row.size).await;
    drop(file);
    let added = match sized {
        Ok(()) => blocking(shared, move |w| Ok(w.collection.add_upload(&row)?)).await,
        Err(e) => Err(ApiError::internal(e.to_string())),
    };
    if let Err(error) = added {
        let _ = tokio::fs::remove_file(&path).await;
        return Err(error);
    }
    Ok((
        StatusCode::CREATED,
        Json(json!({ "upload": upload, "part_size": PART_SIZE })),
    ))
}

/// `PUT /uploads/{upload}/parts/{n}`: send part `n` (ADR 0059 items 1, 2
/// and 5). The write of the bytes runs with no lock of the workspace. The
/// record of the part is one statement, so two parts at the same time do
/// not lose each other.
pub(crate) async fn put_part(
    State(shared): State<Shared>,
    Path((upload, n)): Path<(String, u32)>,
    body: Bytes,
) -> ApiResult {
    let (row, incoming, rules) = blocking(Arc::clone(&shared), {
        let upload = upload.clone();
        move |w| {
            Ok((
                w.collection.upload(&upload)?,
                w.incoming_dir.clone(),
                w.rules.clone(),
            ))
        }
    })
    .await?;
    if u64::from(n) >= part_count(row.size) {
        return Err(bad_part(format!(
            "part {n} is outside the file, which has {} parts",
            part_count(row.size)
        )));
    }
    let offset = u64::from(n) * PART_SIZE;
    let expected = (row.size - offset).min(PART_SIZE);
    if body.len() as u64 != expected {
        return Err(bad_part(format!(
            "part {n} must have {expected} bytes, and it has {}",
            body.len()
        )));
    }
    // The staging file has its full size already, so the check counts the
    // folder only (ADR 0055 item 21).
    check_room(rules, 0).await?;
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(part_path(&incoming, &row.id))
        .await
        .map_err(|e| io_error(&e))?;
    file.seek(SeekFrom::Start(offset))
        .await
        .map_err(|e| io_error(&e))?;
    file.write_all(&body).await.map_err(|e| io_error(&e))?;
    file.flush().await.map_err(|e| io_error(&e))?;
    drop(file);
    blocking(shared, move |w| {
        w.collection
            .record_upload_part(&upload, n, crate::announcer::now())?;
        to_json(upload_json(&w.collection.upload(&upload)?))
    })
    .await
}

/// `GET /uploads/{upload}`: the size and the parts that Studio has.
pub(crate) async fn get_upload(
    State(shared): State<Shared>,
    Path(upload): Path<String>,
) -> ApiResult {
    blocking(shared, move |w| {
        to_json(upload_json(&w.collection.upload(&upload)?))
    })
    .await
}

/// `GET /feeds/{id}/uploads`: the open uploads of a feed.
pub(crate) async fn list_uploads(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let rows = w.collection.uploads_of_feed(FeedId(id))?;
        to_json(rows.iter().map(upload_json).collect::<Vec<_>>())
    })
    .await
}

/// `POST /uploads/{upload}/finish`: import the full file (ADR 0059 item
/// 3). The response is the response of the import (ADR 0026). The upload
/// row goes out when the import starts, so a second `finish` finds no
/// upload.
pub(crate) async fn finish_upload(
    State(shared): State<Shared>,
    Path(upload): Path<String>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let (row, incoming) = blocking(Arc::clone(&shared), move |w| {
        let row = w.collection.upload(&upload)?;
        let parts = u32::try_from(part_count(row.size)).unwrap_or(u32::MAX);
        if let Some(missing) = (0..parts).find(|n| !row.received.contains(n)) {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "upload_incomplete",
                format!("part {missing} is missing"),
            ));
        }
        if !w.collection.remove_upload(&row.id)? {
            return Err(ApiError::new(
                StatusCode::NOT_FOUND,
                "not_found",
                "the upload is not there",
            ));
        }
        Ok((row, w.incoming_dir.clone()))
    })
    .await?;
    let staged = part_path(&incoming, &row.id);
    // The guard removes the staging file after an error before the import.
    let _guard = routes::TempFile(staged.clone());
    // `media::import` finds the media type partly from the extension of
    // its source file, so the file gets the extension of the name.
    let final_path = match FilePath::new(&row.name)
        .extension()
        .and_then(|e| e.to_str())
        .filter(|e| e.chars().all(|c| c.is_ascii_alphanumeric()))
    {
        Some(extension) => incoming.join(format!("{}.{extension}", row.id)),
        None => incoming.join(&row.id),
    };
    tokio::fs::rename(&staged, &final_path)
        .await
        .map_err(|e| io_error(&e))?;
    routes::import_file(
        shared,
        row.feed_id.0,
        final_path,
        row.name,
        row.replace.is_some(),
    )
    .await
}

/// `DELETE /uploads/{upload}`: stop an upload and remove its file.
pub(crate) async fn delete_upload(
    State(shared): State<Shared>,
    Path(upload): Path<String>,
) -> Result<StatusCode, ApiError> {
    let incoming = blocking(shared, move |w| {
        let row = w.collection.upload(&upload)?;
        w.collection.remove_upload(&row.id)?;
        Ok(part_path(&w.incoming_dir, &row.id))
    })
    .await?;
    let _ = tokio::fs::remove_file(incoming).await;
    Ok(StatusCode::NO_CONTENT)
}

/// Remove each upload with no new part since 24 hours before `now`, with
/// its file (ADR 0059 item 4).
pub(crate) fn clean_expired(w: &Workspace, now: u64) -> Result<usize, ApiError> {
    let rows = w
        .collection
        .uploads_older_than(now.saturating_sub(EXPIRY_SECONDS))?;
    for row in &rows {
        w.collection.remove_upload(&row.id)?;
        let _ = std::fs::remove_file(part_path(&w.incoming_dir, &row.id));
    }
    Ok(rows.len())
}

/// Start the thread that cleans expired uploads at the start and each
/// hour. The thread holds a weak link, and it ends when the workspace is
/// gone.
pub(crate) fn spawn_cleaner(shared: &Arc<Mutex<Workspace>>) {
    let weak = Arc::downgrade(shared);
    std::thread::spawn(move || {
        loop {
            let Some(shared) = weak.upgrade() else {
                return;
            };
            if let Ok(w) = lock_workspace(&shared) {
                let _ = clean_expired(&w, crate::announcer::now());
            }
            drop(shared);
            std::thread::sleep(CLEAN_INTERVAL);
        }
    });
}

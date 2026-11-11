//! The routes of the retire and of the delete of a feed (ADR 0131). The
//! retire changes the draft only. The delete draft and the restore draft
//! write nothing on a host. The write is the usual write route with the ID of
//! the diff that the user approved (ADR 0007).
//!
//! Only the route of the retire state sends a request to the index. It reads
//! with no lock of the workspace (ADR 0015): the route takes the facts under
//! the lock, releases it, reads, and then answers.

use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use serde_json::json;
use studio_core::collection::FeedId;
use studio_core::feed_delete;
use studio_core::feed_retire;

use crate::routes::{ApiResult, Shared, blocking, blocking_unlocked, diff_json, draft_json};
use crate::routes::{ensure_writable_draft, writer_check};

/// The state of the three steps of a retire. The route reads the index one
/// time. The page calls it when it opens and at **Check Again** (ADR 0131
/// item 6). The route starts no draft.
pub(crate) async fn retire_state(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    let (facts, index) = blocking(shared, move |w| {
        let id = FeedId(id);
        let library = w.library()?;
        let facts = feed_retire::retire_facts(&w.collection, &library, id)?;
        Ok((facts, w.index.clone()))
    })
    .await?;
    let guid = facts.guid.clone();
    let removal =
        blocking_unlocked(move || Ok(feed_retire::read_removal(guid.as_deref(), &index))).await?;
    Ok(Json(json!(feed_retire::retire_state(&facts, removal))))
}

/// Set "Hide from apps" in the draft (ADR 0131 item 5). The answer is the
/// draft.
pub(crate) async fn retire_on(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    set_retire(shared, id, true).await
}

/// Clear "Hide from apps" in the draft (ADR 0131 item 8). The answer is the
/// draft.
pub(crate) async fn retire_off(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    set_retire(shared, id, false).await
}

async fn set_retire(shared: Shared, id: i64, on: bool) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        ensure_writable_draft(w, id)?;
        feed_retire::set_retire(&w.collection, id, on)?;
        draft_json(w, id)
    })
    .await
}

#[derive(Deserialize)]
pub(crate) struct SentDelete {
    /// The check box of ADR 0131 item 11. Off when the request omits it.
    #[serde(default)]
    with_files: bool,
}

/// Start the delete draft and give its diff, its files and the names of
/// the final sheet (ADR 0131 items 10, 11 and 13). The route writes nothing
/// on a host.
pub(crate) async fn delete_draft(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<SentDelete>,
) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        writer_check(w, id)?;
        let hosts = w.hosts()?;
        let library = w.library()?;
        let started =
            feed_delete::start_delete_draft(&w.collection, &hosts, &library, id, body.with_files)?;
        let names = feed_retire::delete_names(&w.collection, &library, id)?;
        let mut answer = diff_json(w, id, None)?.0;
        answer["summary"] = json!(started.summary);
        answer["names"] = json!(names);
        Ok(Json(answer))
    })
    .await
}

/// Start the draft that brings the deleted file back to its address (ADR
/// 0131 item 14). The answer is its diff.
pub(crate) async fn restore_draft(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        let hosts = w.hosts()?;
        let library = w.library()?;
        feed_delete::start_restore_draft(&w.collection, &hosts, &library, id)?;
        diff_json(w, id, None)
    })
    .await
}

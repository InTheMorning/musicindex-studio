//! The routes of the feed check (ADR 0174). The check changes no feed. A
//! repair changes the draft only, and it never writes to a host.
//!
//! Each read runs with no lock of the workspace (ADR 0015): a route takes
//! the facts of the draft under the lock, releases it, reads, and then
//! stores the result.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::json;
use studio_core::collection::FeedId;
use studio_core::feed_check::{self, FindingKind};
use studio_core::feed_reads::{self, NetworkFacts};
use studio_core::pipeline::{self, PipelineError};

use crate::error::ApiError;
use crate::routes::{ApiResult, Shared, blocking, blocking_unlocked, draft_json, ensure_draft};
use crate::workspace::Workspace;

/// True when Studio has no host to write the feed to (ADR 0067). A feed
/// with a pending move has a host.
fn is_read_only(w: &Workspace, id: FeedId) -> Result<bool, ApiError> {
    let row = w.collection.feed(id)?;
    Ok(row.connector_id.is_none() && w.collection.draft_feed_move(id)?.is_none())
}

/// The last network facts of a feed. They belong to the base file of the
/// draft, so a change of the feed file drops them (ADR 0174 item 18).
fn stored<'a>(w: &'a Workspace, id: FeedId, base_key: &str) -> Option<&'a NetworkFacts> {
    w.check_facts
        .get(&id.0)
        .filter(|(key, _)| key == base_key)
        .map(|(_, net)| net)
}

/// The findings of the draft: the local findings, and the last read facts
/// from memory. A read-only feed gives each finding with no repair.
fn check_json(w: &Workspace, id: FeedId) -> ApiResult {
    let hosts = w.hosts()?;
    let inputs = pipeline::check_inputs(&w.collection, &hosts, id)?;
    let net = stored(w, id, &inputs.base_key);
    let mut result = feed_reads::check_with(&inputs.links, net);
    let read_only = is_read_only(w, id)?;
    if read_only {
        for finding in &mut result.findings {
            finding.repair = None;
        }
    }
    let guid = inputs.links.feed.podcast_guid.as_deref();
    let line = feed_reads::index_line_of(net, guid, inputs.base_build_date.as_deref());
    // The report of ADR 0175 item 8 is for the user to copy. The core
    // gives its text.
    let findings: Vec<serde_json::Value> = result
        .findings
        .iter()
        .map(|finding| {
            let mut value = serde_json::to_value(finding).unwrap_or_default();
            if finding.kind == FindingKind::CopyOtherRecipients
                && let Some(copy) = finding.url.as_deref()
            {
                value["report"] = json!(feed_check::copy_report(
                    guid,
                    inputs.links.url.as_deref(),
                    copy
                ));
            }
            value
        })
        .collect();
    Ok(Json(json!({
        "revision": inputs.revision,
        "read_only": read_only,
        "read": net.is_some(),
        "findings": findings,
        "unreadable": result.unreadable,
        "index_line": line.map(feed_check::IndexLine::text),
    })))
}

/// The findings of a feed, with no read. The facts of the last read come
/// from memory (ADR 0174 items 16 and 18).
pub(crate) async fn get_check(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    blocking(shared, move |w| {
        let id = FeedId(id);
        ensure_draft(w, id)?;
        check_json(w, id)
    })
    .await
}

/// Run the reads of the check, keep their facts in memory, and give the
/// findings. The page calls this route when it opens and at **Check
/// Again** (ADR 0174 item 16).
pub(crate) async fn run_reads(State(shared): State<Shared>, Path(id): Path<i64>) -> ApiResult {
    let (inputs, feeds, files, index) = blocking(shared.clone(), move |w| {
        let id = FeedId(id);
        ensure_draft(w, id)?;
        let hosts = w.hosts()?;
        let inputs = pipeline::check_inputs(&w.collection, &hosts, id)?;
        Ok((
            inputs,
            w.link_reader.clone(),
            w.file_reader.clone(),
            w.index.clone(),
        ))
    })
    .await?;
    let key = inputs.base_key.clone();
    let net = blocking_unlocked(move || {
        let mut net = feed_reads::gather(&inputs.links, feeds.0.as_ref(), files.0.as_ref());
        // The reads of the index go out with the other reads (ADR 0175
        // item 1).
        feed_reads::gather_index(&inputs.links, &index, &mut net);
        Ok(net)
    })
    .await?;
    blocking(shared, move |w| {
        let id = FeedId(id);
        w.check_facts.insert(id.0, (key, net));
        check_json(w, id)
    })
    .await
}

/// The repair that the user selected: the finding by its kind, its entry
/// and its address, and the revision of the draft that the user saw.
#[derive(Deserialize)]
pub(crate) struct SentRepair {
    kind: FindingKind,
    #[serde(default)]
    entry: Option<String>,
    #[serde(default)]
    url: Option<String>,
    revision: String,
}

fn refused(error: &PipelineError) -> ApiError {
    ApiError::new(
        StatusCode::UNPROCESSABLE_ENTITY,
        "repair_refused",
        error.to_string(),
    )
}

/// Apply the repair of one finding to the draft and give the draft and the
/// new findings. A read-only feed gives 409 `read_only`. A repair that
/// needs a read sends it with no lock, and it changes the draft only when
/// the read confirms the fact (ADR 0174 items 10 and 15).
pub(crate) async fn repair(
    State(shared): State<Shared>,
    Path(id): Path<i64>,
    Json(body): Json<SentRepair>,
) -> ApiResult {
    let sent = body.revision.clone();
    let (finding, step, feeds, files) = blocking(shared.clone(), move |w| {
        let id = FeedId(id);
        if is_read_only(w, id)? {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "read_only",
                format!("feed {} has no host, so Studio cannot repair it", id.0),
            ));
        }
        ensure_draft(w, id)?;
        let hosts = w.hosts()?;
        let inputs = pipeline::check_inputs(&w.collection, &hosts, id)?;
        if inputs.revision != body.revision {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "stale_revision",
                "the draft changed after this model was read; read the draft again",
            ));
        }
        let result = feed_reads::check_with(&inputs.links, stored(w, id, &inputs.base_key));
        let finding = result
            .findings
            .into_iter()
            .find(|f| f.kind == body.kind && f.entry == body.entry && f.url == body.url)
            .ok_or_else(|| {
                ApiError::new(
                    StatusCode::CONFLICT,
                    "no_such_finding",
                    "the check has no such finding now; check again",
                )
            })?;
        let step = pipeline::repair_step(&inputs, &finding).map_err(|e| refused(&e))?;
        Ok((finding, step, w.link_reader.clone(), w.file_reader.clone()))
    })
    .await?;
    let read_step = step.clone();
    let proof = blocking_unlocked(move || {
        pipeline::run_step(&read_step, feeds.0.as_ref(), files.0.as_ref()).map_err(|e| refused(&e))
    })
    .await?;
    blocking(shared, move |w| {
        let id = FeedId(id);
        let hosts = w.hosts()?;
        pipeline::apply_check_repair(&w.collection, &hosts, id, &finding, &sent, &proof)?;
        let draft = draft_json(w, id)?.0;
        let check = check_json(w, id)?.0;
        Ok(Json(json!({ "draft": draft, "check": check })))
    })
    .await
}

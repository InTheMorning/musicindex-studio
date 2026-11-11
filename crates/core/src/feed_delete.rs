//! The delete of a feed from its file host (ADR 0131 items 11 to 14).
//!
//! A delete is a draft with a mark. The draft has no edit. The write goes
//! through [`pipeline::write_draft`] with the ID of the diff that the user
//! approved (ADR 0007). The write takes a snapshot of the feed file first.
//! The conflict check stops the write when the file changed.

use crate::collection::{Collection, ConnectorId, DraftOperationKind, DraftRow, FeedId};
use crate::feed::Feed;
use crate::inventory::Hosts;
use crate::library::Library;
use crate::pipeline::{self, Diff, HostOperation, OperationKind, PipelineError, PipelineErrorKind};
use crate::xml::Document;

/// Why a file stays on the host at a delete (ADR 0131 item 11).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "reason", content = "feed_titles")]
pub enum StayReason {
    /// The file is a pointer file of a move. Studio keeps it (ADR 0038
    /// item 20).
    PointerFile,
    /// A different feed uses the file. The value is the title of each feed.
    UsedByFeeds(Vec<String>),
}

/// A file that stays on its host at a delete.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct StayingFile {
    /// The host.
    pub connector_id: ConnectorId,
    /// The path of the file on the host.
    pub path: String,
    /// Why the file stays.
    #[serde(flatten)]
    pub reason: StayReason,
}

/// The files of a delete draft, for the final sheet (ADR 0131 items 10
/// and 11).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DeleteSummary {
    /// Each file that the write deletes. The feed file is the last one.
    pub deletes: Vec<HostOperation>,
    /// Each file that stays, with the reason. The sheet counts them.
    pub stays: Vec<StayingFile>,
}

/// A delete draft with its diff.
#[derive(Debug, Clone)]
pub struct DeleteDraft {
    /// The diff that the user approves.
    pub diff: Diff,
    /// The files that go and the files that stay.
    pub summary: DeleteSummary,
}

/// The files that go and the files that stay in a diff of a delete.
#[must_use]
pub fn summary(diff: &Diff) -> DeleteSummary {
    DeleteSummary {
        deletes: diff
            .operations
            .iter()
            .filter(|operation| operation.kind == OperationKind::Delete)
            .cloned()
            .collect(),
        stays: diff
            .blocked
            .iter()
            .map(|blocked| StayingFile {
                connector_id: blocked.connector_id,
                path: blocked.path.clone(),
                reason: if blocked.pointer {
                    StayReason::PointerFile
                } else {
                    StayReason::UsedByFeeds(blocked.feed_titles.clone())
                },
            })
            .collect(),
    }
}

/// Start the delete draft of a feed, and give its diff and its files.
///
/// The draft starts from the live feed file. With `with_files`, the draft
/// also names each audio and image file that the feed refers to on a host,
/// and each pointer file of the feed (ADR 0131 item 11). A pointer file and
/// a file that a different feed uses stay. The diff counts them. The
/// function writes nothing on a host.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::ReadOnly`] when the feed has no connector, and
/// [`PipelineErrorKind::NotFound`] when the feed file is not on its host.
/// Gives [`PipelineErrorKind::InvalidOperation`] when the draft has a change
/// that the user did not write yet, or the feed file is a pointer file or
/// the file of a different feed (ADR 0131 item 13).
pub fn start_delete_draft(
    collection: &Collection,
    hosts: &Hosts,
    library: &Library,
    id: FeedId,
    with_files: bool,
) -> Result<DeleteDraft, PipelineError> {
    if collection.has_draft_feed_delete(id)? {
        pipeline::discard_draft(collection, id)?;
    } else if collection.draft(id)?.is_some() && pipeline::draft_changed(collection, id)? {
        return Err(PipelineError::new(
            PipelineErrorKind::InvalidOperation,
            "ADR 0131 item 12: write or discard the draft before a delete",
        ));
    }
    let row = collection.feed(id)?;
    let Some(connector_id) = row.connector_id else {
        return Err(PipelineError::new(
            PipelineErrorKind::ReadOnly,
            format!("feed {} has no connector, so Studio cannot delete it", id.0),
        ));
    };
    let draft = pipeline::start_draft(collection, hosts.get(connector_id)?, id)?;
    let started = (|| {
        collection.set_draft_feed_delete(id)?;
        if with_files {
            add_file_deletes(collection, hosts, &draft, id)?;
        }
        let diff = pipeline::diff(collection, hosts, library, id)?;
        Ok(DeleteDraft {
            summary: summary(&diff),
            diff,
        })
    })();
    if started.is_err() {
        pipeline::discard_draft(collection, id)?;
    }
    started
}

/// Add a delete of each audio file, each image file and each pointer file of
/// the feed to its draft. The diff then keeps the files that stay.
fn add_file_deletes(
    collection: &Collection,
    hosts: &Hosts,
    draft: &DraftRow,
    id: FeedId,
) -> Result<(), PipelineError> {
    let row = collection.feed(id)?;
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    let addresses = pipeline::feed_addresses(&feed);
    let mut targets: Vec<(ConnectorId, String)> = Vec::new();
    for connector_id in hosts.ids().collect::<Vec<_>>() {
        let connector = hosts.get(connector_id)?;
        for file in collection.host_files(connector_id)? {
            let is_feed_file =
                row.connector_id == Some(connector_id) && row.path.as_deref() == Some(&file.path);
            let referred = connector
                .public_url(&file.path)
                .is_some_and(|url| addresses.iter().any(|address| *address == url));
            if referred && !is_feed_file {
                targets.push((connector_id, file.path));
            }
        }
    }
    targets.extend(collection.pointers_of_feed(id)?);
    targets.sort();
    targets.dedup();
    for (connector_id, path) in targets {
        collection.add_draft_operation(
            id,
            DraftOperationKind::Delete,
            None,
            connector_id,
            &path,
        )?;
    }
    Ok(())
}

/// Start the draft that writes the last copy of a deleted feed back to the
/// same host and path (ADR 0131 item 14). The draft has no hosted file, as
/// the draft of a new feed has. The user approves its diff and writes it
/// through [`pipeline::write_draft`]. The write puts the file back and ends
/// the record of the delete.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::NotFound`] when the feed has no record of a
/// delete, or its release folder has no last copy. Gives
/// [`PipelineErrorKind::Conflict`] when the last copy is not the file that
/// Studio last wrote, and [`PipelineErrorKind::FileExists`] when a file is
/// at the path on the host.
pub fn start_restore_draft(
    collection: &Collection,
    hosts: &Hosts,
    library: &Library,
    id: FeedId,
) -> Result<DraftRow, PipelineError> {
    let row = collection.feed(id)?;
    let (connector_id, path) = collection.feed_host_delete(id)?.ok_or_else(|| {
        PipelineError::new(
            PipelineErrorKind::NotFound,
            format!("feed {} has no deleted feed file", id.0),
        )
    })?;
    let not_found = || {
        PipelineError::new(
            PipelineErrorKind::NotFound,
            format!("feed {} has no last copy in its release folder", id.0),
        )
    };
    let folder = row.folder.as_deref().ok_or_else(not_found)?;
    let file_name = path
        .rsplit_once('/')
        .map_or(path.as_str(), |(_, name)| name);
    let bytes = library
        .read_feed_xml(folder, file_name)
        .map_err(|_| not_found())?;
    if Some(&crate::digest::content_version(&bytes)) != row.last_version.as_ref() {
        return Err(PipelineError::new(
            PipelineErrorKind::Conflict,
            "the release folder has a different feed XML than the last read",
        ));
    }
    if hosts.get(connector_id)?.read(&path)?.is_some() {
        return Err(PipelineError::new(
            PipelineErrorKind::FileExists,
            format!("a file exists at {path} on the file host already"),
        ));
    }
    collection.restore_feed_location(id, connector_id, &path)?;
    let draft = DraftRow {
        base_version: None,
        base_xml: Vec::new(),
        draft_xml: bytes,
    };
    collection.save_draft(id, &draft)?;
    Ok(draft)
}

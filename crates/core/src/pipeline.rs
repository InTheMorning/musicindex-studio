//! The write pipeline: draft, diff, write and snapshot (ADR 0007, ADR 0015).
//!
//! A write to a host occurs only through [`write_draft`], and only with the
//! ID of the diff that the user approved.

use std::fmt;

use similar::TextDiff;

use crate::collection::{Collection, CollectionError, DraftRow, FeedId, FeedRow, SnapshotId};
use crate::connector::{Connector, ConnectorError, ConnectorErrorKind, Expected, Version};
use crate::digest::sha256_hex;
use crate::feed::{Feed, FeedError};
use crate::xml::{Document, ParseError};

/// The kind of a [`PipelineError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PipelineErrorKind {
    /// The feed has no connector or no path, so Studio cannot write it.
    ReadOnly,
    /// The feed file does not exist on the host.
    NotFound,
    /// The feed has no draft.
    NoDraft,
    /// The approved diff is not the current diff of the draft.
    DiffNotApproved,
    /// The live file changed after the draft started.
    Conflict,
    /// The feed file is not a feed that Studio can read.
    InvalidFeed,
    /// The connector failed.
    Connector,
    /// The collection database failed.
    Collection,
}

/// An error from the write pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineError {
    kind: PipelineErrorKind,
    message: String,
}

impl PipelineError {
    fn new(kind: PipelineErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// The kind of the error.
    #[must_use]
    pub fn kind(&self) -> PipelineErrorKind {
        self.kind
    }
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for PipelineError {}

impl From<CollectionError> for PipelineError {
    fn from(error: CollectionError) -> Self {
        Self::new(PipelineErrorKind::Collection, error.to_string())
    }
}

impl From<ConnectorError> for PipelineError {
    fn from(error: ConnectorError) -> Self {
        let kind = if error.kind() == ConnectorErrorKind::Conflict {
            PipelineErrorKind::Conflict
        } else {
            PipelineErrorKind::Connector
        };
        Self::new(kind, error.to_string())
    }
}

impl From<ParseError> for PipelineError {
    fn from(error: ParseError) -> Self {
        Self::new(PipelineErrorKind::InvalidFeed, error.to_string())
    }
}

impl From<FeedError> for PipelineError {
    fn from(error: FeedError) -> Self {
        Self::new(PipelineErrorKind::InvalidFeed, error.to_string())
    }
}

/// The ID of a diff: a hash of the base version and the draft bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiffId(String);

impl DiffId {
    /// Make a diff ID from its text, as a client sends it back.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// The text of the ID.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A line diff between the live file and the draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    /// The ID that the user approves.
    pub id: DiffId,
    /// The diff in unified format.
    pub unified: String,
    /// True when the draft is different from the live file.
    pub changed: bool,
}

fn writable(row: &FeedRow) -> Result<&str, PipelineError> {
    match (row.connector_id, row.path.as_deref()) {
        (Some(_), Some(path)) => Ok(path),
        _ => Err(PipelineError::new(
            PipelineErrorKind::ReadOnly,
            format!(
                "feed {} has no connector, so Studio cannot write it",
                row.id.0
            ),
        )),
    }
}

/// Read the live feed from its host, and keep its facts in the collection.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the feed is read-only, the file does not
/// exist, or the file is not a feed.
pub fn refresh(
    collection: &mut Collection,
    connector: &dyn Connector,
    id: FeedId,
) -> Result<Feed, PipelineError> {
    let row = collection.feed(id)?;
    let path = writable(&row)?;
    let file = connector.read(path)?.ok_or_else(|| {
        PipelineError::new(
            PipelineErrorKind::NotFound,
            format!("{path} does not exist on the host"),
        )
    })?;
    let feed = Feed::read(&Document::parse_bytes(&file.bytes)?)?;
    collection.record_state(id, &feed, &file.version)?;
    Ok(feed)
}

/// Start a draft from the live file. It replaces an older draft.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the feed is read-only or the file does not
/// exist.
pub fn start_draft(
    collection: &Collection,
    connector: &dyn Connector,
    id: FeedId,
) -> Result<DraftRow, PipelineError> {
    let row = collection.feed(id)?;
    let path = writable(&row)?;
    let file = connector.read(path)?.ok_or_else(|| {
        PipelineError::new(
            PipelineErrorKind::NotFound,
            format!("{path} does not exist on the host"),
        )
    })?;
    let draft = DraftRow {
        base_version: Some(file.version),
        base_xml: file.bytes.clone(),
        draft_xml: file.bytes,
    };
    collection.save_draft(id, &draft)?;
    Ok(draft)
}

/// Change the draft through the typed model.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the feed has no draft or the draft is not
/// a feed.
pub fn edit_draft(
    collection: &Collection,
    id: FeedId,
    change: impl FnOnce(&mut Feed),
) -> Result<DraftRow, PipelineError> {
    let mut draft = current_draft(collection, id)?;
    let mut doc = Document::parse_bytes(&draft.draft_xml)?;
    let mut feed = Feed::read(&doc)?;
    change(&mut feed);
    feed.write(&mut doc)?;
    draft.draft_xml = doc.write().into_bytes();
    collection.save_draft(id, &draft)?;
    Ok(draft)
}

/// The diff between the live file at the start of the draft and the draft.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the feed has no draft.
pub fn diff(collection: &Collection, id: FeedId) -> Result<Diff, PipelineError> {
    let draft = current_draft(collection, id)?;
    let base = String::from_utf8_lossy(&draft.base_xml);
    let new = String::from_utf8_lossy(&draft.draft_xml);
    let unified = TextDiff::from_lines(base.as_ref(), new.as_ref())
        .unified_diff()
        .context_radius(3)
        .header("live", "draft")
        .to_string();
    Ok(Diff {
        id: diff_id(&draft),
        unified,
        changed: draft.base_xml != draft.draft_xml,
    })
}

/// Write the draft to the host (ADR 0007).
///
/// The function writes only when `approved` is the ID of the current diff,
/// and when the live file has the version that the draft started from. It
/// keeps a snapshot of the live file first. After the write, the draft goes
/// out and the collection keeps the new state.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::DiffNotApproved`] or
/// [`PipelineErrorKind::Conflict`], and the file on the host then does not
/// change.
pub fn write_draft(
    collection: &mut Collection,
    connector: &dyn Connector,
    id: FeedId,
    approved: &DiffId,
) -> Result<Version, PipelineError> {
    let row = collection.feed(id)?;
    let path = writable(&row)?.to_owned();
    let draft = current_draft(collection, id)?;
    if &diff_id(&draft) != approved {
        return Err(PipelineError::new(
            PipelineErrorKind::DiffNotApproved,
            "the draft changed after the diff was approved; approve the new diff",
        ));
    }
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    let live = connector.read(&path)?;
    let live_version = live.as_ref().map(|f| &f.version);
    if live_version != draft.base_version.as_ref() {
        return Err(PipelineError::new(
            PipelineErrorKind::Conflict,
            format!("{path} changed on the host after the draft started"),
        ));
    }
    let expected = match &live {
        Some(file) => {
            collection.add_snapshot(id, &file.version, &file.bytes)?;
            Expected::Version(file.version.clone())
        }
        None => Expected::Absent,
    };
    let version = connector.write(&path, &draft.draft_xml, &expected)?;
    collection.record_state(id, &feed, &version)?;
    collection.delete_draft(id)?;
    Ok(version)
}

/// Start a draft from a snapshot, to roll the feed back. The user approves
/// its diff and writes it like each other draft.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the feed has no such snapshot or the file
/// does not exist.
pub fn draft_from_snapshot(
    collection: &Collection,
    connector: &dyn Connector,
    id: FeedId,
    snapshot: SnapshotId,
) -> Result<DraftRow, PipelineError> {
    let xml = collection.snapshot_xml(id, snapshot)?;
    let mut draft = start_draft(collection, connector, id)?;
    draft.draft_xml = xml;
    collection.save_draft(id, &draft)?;
    Ok(draft)
}

fn current_draft(collection: &Collection, id: FeedId) -> Result<DraftRow, PipelineError> {
    collection.draft(id)?.ok_or_else(|| {
        PipelineError::new(
            PipelineErrorKind::NoDraft,
            format!("feed {} has no draft", id.0),
        )
    })
}

fn diff_id(draft: &DraftRow) -> DiffId {
    let base = draft
        .base_version
        .as_ref()
        .map_or("absent", Version::as_str);
    DiffId(sha256_hex(&[base.as_bytes(), &[0], &draft.draft_xml]))
}

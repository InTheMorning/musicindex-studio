//! The write pipeline: draft, diff, write and snapshot (ADR 0007, ADR 0015).
//!
//! A write to a host occurs only through [`write_draft`], and only with the
//! ID of the diff that the user approved.

use std::fmt;

use similar::TextDiff;

use crate::address::AddressSource;
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
    /// A web read failed.
    Network,
    /// The feed has no draft.
    NoDraft,
    /// The approved diff is not the current diff of the draft.
    DiffNotApproved,
    /// The sent model is from a revision of the draft that is not current
    /// (ADR 0022).
    StaleRevision,
    /// The sent model does not agree with the draft: a key that is not in
    /// the draft, or a changed GUID (ADR 0022).
    InvalidModel,
    /// An address field of the draft is not a correct web address
    /// (ADR 0023).
    BadAddress,
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
    collection.set_address(
        id,
        AddressSource::Folder,
        connector.public_url(path).as_deref(),
    )?;
    Ok(feed)
}

/// Read a feed that has no connector from its public address, and keep its
/// facts in the collection (ADR 0020).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the feed has no public address, the read
/// fails, or the file is not a feed.
pub fn refresh_public(
    collection: &mut Collection,
    web: &crate::web::Web,
    id: FeedId,
) -> Result<Feed, PipelineError> {
    let row = collection.feed(id)?;
    let url = row.public_url.as_deref().ok_or_else(|| {
        PipelineError::new(
            PipelineErrorKind::ReadOnly,
            format!("feed {} has no connector and no public address", id.0),
        )
    })?;
    let read = crate::discover::read_public(web, url)
        .map_err(|message| PipelineError::new(PipelineErrorKind::Network, message))?;
    collection.record_state(id, &read.feed, &read.version)?;
    collection.set_address(id, AddressSource::Read, Some(&read.url))?;
    Ok(read.feed)
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

/// The revision of a draft: a hash of its bytes (ADR 0022 item 5).
#[must_use]
pub fn revision(draft: &DraftRow) -> String {
    sha256_hex(&[&draft.draft_xml])
}

/// The typed model of a draft, and its revision.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the feed has no draft or the draft is not
/// a feed.
pub fn draft_model(collection: &Collection, id: FeedId) -> Result<(Feed, String), PipelineError> {
    let draft = current_draft(collection, id)?;
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    Ok((feed, revision(&draft)))
}

/// Replace the model of a draft with a model that the UI sent (ADR 0022).
///
/// The function finds the source element of each sent entry by its key. It
/// gives a new item a random UUID as its GUID. Gives the new model and its
/// revision.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::StaleRevision`] when `revision` is not the
/// current revision, and [`PipelineErrorKind::InvalidModel`] for a key that
/// is not in the draft or a changed GUID. The draft then does not change.
pub fn replace_model(
    collection: &Collection,
    id: FeedId,
    sent_revision: &str,
    mut sent: Feed,
) -> Result<(Feed, String), PipelineError> {
    let mut draft = current_draft(collection, id)?;
    if revision(&draft) != sent_revision {
        return Err(PipelineError::new(
            PipelineErrorKind::StaleRevision,
            "the draft changed after this model was read; read the draft again",
        ));
    }
    let mut doc = Document::parse_bytes(&draft.draft_xml)?;
    let current = Feed::read(&doc)?;
    let invalid = |message: String| PipelineError::new(PipelineErrorKind::InvalidModel, message);
    sent.adopt(&current).map_err(|e| invalid(e.to_string()))?;
    keep_identity(&current, &mut sent).map_err(invalid)?;
    number_tracks(&current, &mut sent);
    sent.write(&mut doc)?;
    draft.draft_xml = doc.write().into_bytes();
    collection.save_draft(id, &draft)?;
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    Ok((feed, revision(&draft)))
}

/// Give each track its position as its `podcast:episode` when the sent model
/// changes the sequence of the tracks (ADR 0024). A `pubDate` does not
/// change.
fn number_tracks(current: &Feed, sent: &mut Feed) {
    let same = sent.items.len() == current.items.len()
        && sent
            .items
            .iter()
            .enumerate()
            .all(|(i, e)| e.key() == Some(i));
    if same {
        return;
    }
    for (number, item) in (1_usize..).zip(sent.items.iter_mut()) {
        item.episode = Some(number.to_string());
    }
}

/// Refuse a change of an existing GUID, and give each new item a GUID
/// (ADR 0008, ADR 0022 items 7 and 8).
fn keep_identity(current: &Feed, sent: &mut Feed) -> Result<(), String> {
    if current.podcast_guid.is_some() && sent.podcast_guid != current.podcast_guid {
        return Err("the feed GUID does not change through the editor".to_owned());
    }
    let items = sent
        .items
        .iter_mut()
        .map(|e| {
            (
                e.key().and_then(|k| current.items.get(k)).map(|c| &c.guid),
                &mut e.value,
            )
        })
        .chain(sent.live_items.iter_mut().map(|e| {
            (
                e.key()
                    .and_then(|k| current.live_items.get(k))
                    .map(|c| &c.item.guid),
                &mut e.value.item,
            )
        }));
    for (before, item) in items {
        match before {
            Some(Some(guid)) if item.guid.as_ref() != Some(guid) => {
                return Err(format!(
                    "the item GUID {guid} does not change through the editor"
                ));
            }
            None if item.guid.as_deref().is_none_or(|g| g.trim().is_empty()) => {
                item.guid = Some(uuid::Uuid::new_v4().to_string());
            }
            Some(_) | None => {}
        }
    }
    Ok(())
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
/// Gives [`PipelineErrorKind::DiffNotApproved`],
/// [`PipelineErrorKind::BadAddress`] or [`PipelineErrorKind::Conflict`], and
/// the file on the host then does not change.
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
    let bad = feed.bad_addresses();
    if !bad.is_empty() {
        let fields: Vec<String> = bad
            .iter()
            .map(|b| format!("{} ({:?})", b.field, b.value))
            .collect();
        return Err(PipelineError::new(
            PipelineErrorKind::BadAddress,
            format!(
                "these address fields are not full http or https addresses: {}",
                fields.join(", ")
            ),
        ));
    }
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

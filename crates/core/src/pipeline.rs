//! The write pipeline: draft, diff, write and snapshot (ADR 0007, ADR 0015).
//!
//! A write to a host occurs only through [`write_draft`], and only with the
//! ID of the diff that the user approved.

use std::collections::HashSet;
use std::fmt;
use std::path::Path;

use similar::TextDiff;

use crate::address::AddressSource;
use crate::collection::{
    Collection, CollectionError, ConnectorId, DraftOperationKind, DraftRow, FeedId, FeedRow,
    LibraryFile, LibraryFileId, SnapshotId,
};
use crate::connector::{Connector, ConnectorError, ConnectorErrorKind, Expected, Version};
use crate::digest::sha256_hex;
use crate::feed::{Feed, FeedError};
use crate::guid::podcast_guid_for_url;
use crate::inventory::{self, Hosts, InventoryError, InventoryErrorKind};
use crate::library::{Library, file_stamp};
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
    /// The connector gives no public address for the path of a new feed
    /// (ADR 0032 item 8).
    NoPublicAddress,
    /// A file already exists at the path of a new feed (ADR 0032 item 8),
    /// at the path of a pending file (ADR 0026 item 7), or at the name of
    /// a file of the release folder, with different bytes (ADR 0036
    /// item 5).
    FileExists,
    /// The feed has no release folder yet, so Studio cannot import a file
    /// into it (ADR 0036 item 5).
    NoFolder,
    /// A new address of the feed is not on its host after the uploads
    /// and the copies. The write stops before the deletions (ADR 0036
    /// item 15).
    NotReachable,
    /// A host operation of the draft waits for the download of a remote
    /// library file (ADR 0038 item 12).
    WaitingForDownload,
    /// A host needs a secret from the vault, and the vault is locked
    /// (ADR 0017).
    VaultLocked,
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

impl From<InventoryError> for PipelineError {
    fn from(error: InventoryError) -> Self {
        let kind = match error.kind() {
            InventoryErrorKind::Collection => PipelineErrorKind::Collection,
            InventoryErrorKind::VaultLocked => PipelineErrorKind::VaultLocked,
            InventoryErrorKind::UnknownHost
            | InventoryErrorKind::NoHost
            | InventoryErrorKind::Connector => PipelineErrorKind::Connector,
        };
        Self::new(kind, error.to_string())
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

/// A library file that an upload of the write will put on a host
/// (ADR 0036 item 10). The UI review sheet reads [`Diff::files`].
///
/// Task 021c keeps this list until task 021g, which removes it in favor
/// of [`Diff::operations`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DiffFile {
    /// The path of the file in its release folder.
    pub path: String,
    /// The size of the file, in bytes.
    pub size: u64,
    /// The media type, such as `audio/mpeg`.
    pub media_type: String,
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
    /// Each library file that an upload of `operations` will put on a
    /// host (ADR 0036 item 10). Task 021g removes this field, in favor of
    /// `operations`.
    pub files: Vec<DiffFile>,
    /// Each upload, copy and deletion that the write does on a host
    /// (ADR 0036 item 14).
    pub operations: Vec<HostOperation>,
    /// Each deletion that Studio refuses because a feed refers to the
    /// file (ADR 0036 item 13).
    pub blocked: Vec<BlockedDelete>,
}

/// The kind of a [`HostOperation`] (ADR 0036 items 10 and 12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    /// Put a library file on its host for the first time (ADR 0036
    /// item 10).
    Upload,
    /// Put a library file on a different host (ADR 0036 item 12).
    Copy,
    /// Delete a file from a host (ADR 0036 item 12).
    Delete,
}

impl OperationKind {
    /// The text of the kind, for the diff ID.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Upload => "upload",
            Self::Copy => "copy",
            Self::Delete => "delete",
        }
    }
}

/// An upload, a copy or a deletion that the write does on a host
/// (ADR 0036 item 14).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HostOperation {
    /// The kind of operation.
    pub kind: OperationKind,
    /// The host.
    pub connector_id: ConnectorId,
    /// The path of the file on the host.
    pub path: String,
    /// The size of the file, in bytes.
    pub size: u64,
    /// The library file of an upload or a copy. `None` for a deletion.
    pub library_file_id: Option<i64>,
    /// The public address of the file, when the host gives one.
    pub url: Option<String>,
    /// The draft operation that gives this operation, so that the user can
    /// remove it. `None` for an upload, which comes from the draft itself.
    pub operation_id: Option<i64>,
    /// True when the library file is remote: its bytes are not in the
    /// library yet, and the write refuses the draft until the download is
    /// complete (ADR 0038 item 12).
    pub waiting: bool,
}

/// A deletion that Studio refuses because a feed refers to the file
/// (ADR 0036 item 13).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct BlockedDelete {
    /// The host.
    pub connector_id: ConnectorId,
    /// The path of the file on the host.
    pub path: String,
    /// The title of each feed that refers to the file. A feed with no
    /// title shows its ID instead.
    pub feed_titles: Vec<String>,
    /// The draft operation of the deletion, so that the user can remove it.
    pub operation_id: i64,
}

/// The result of an import of a file into the library (ADR 0036 item 5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedFile {
    /// The ID of the row of the library file.
    pub library_file_id: LibraryFileId,
    /// The path, the host and the address where Studio plans to put the
    /// file (ADR 0036 item 9).
    pub planned: inventory::PlannedPath,
}

/// The host and the path of a feed that Studio can write.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::ReadOnly`] when the feed has no connector or
/// no path.
fn writable(row: &FeedRow) -> Result<(ConnectorId, &str), PipelineError> {
    match (row.connector_id, row.path.as_deref()) {
        (Some(connector_id), Some(path)) => Ok((connector_id, path)),
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
    let (_, path) = writable(&row)?;
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
    let (_, path) = writable(&row)?;
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

/// The template of a new feed: `rss` version 2.0 with an empty `channel`
/// (ADR 0032 item 7). [`Feed::write`] fills it, and adds the `podcast` and
/// the `itunes` namespace when the feed needs them.
/// The file of a new feed. It has the XML declaration, the namespaces and
/// the channel elements that Studio sets, with the indentation of a usual
/// feed. The writer then sets the text of each element. Each element that
/// the user adds later copies this indentation, so that a later diff
/// changes only its own lines.
const NEW_FEED_TEMPLATE: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
    "<rss version=\"2.0\" xmlns:podcast=\"https://podcastindex.org/namespace/1.0\" ",
    "xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\">\n",
    "    <channel>\n",
    "        <title></title>\n",
    "        <itunes:author></itunes:author>\n",
    "        <language></language>\n",
    "        <podcast:guid></podcast:guid>\n",
    "        <podcast:medium></podcast:medium>\n",
    "    </channel>\n",
    "</rss>\n",
);

/// Make a new album feed in a connected folder, as a draft with no hosted
/// file (ADR 0032 items 6 to 9).
///
/// The feed GUID is the `UUIDv5` of the public address of `path`
/// ([`podcast_guid_for_url`]). The draft sets `podcast:medium` to `music`
/// and `language` to `en`. The user sets each other field in the editor,
/// examines the diff, and writes it. The write puts the file only when no
/// file exists at the path (ADR 0007).
///
/// # Errors
///
/// Gives [`PipelineErrorKind::NoPublicAddress`] when the connector gives no
/// public address for `path`, and [`PipelineErrorKind::FileExists`] when a
/// file exists there already.
pub fn new_feed(
    collection: &mut Collection,
    connector: &dyn Connector,
    connector_id: ConnectorId,
    path: &str,
    title: &str,
    author: &str,
) -> Result<FeedId, PipelineError> {
    let address = connector.public_url(path).ok_or_else(|| {
        PipelineError::new(
            PipelineErrorKind::NoPublicAddress,
            format!("{path} has no public address; give the host a public address first"),
        )
    })?;
    if connector.read(path)?.is_some() {
        return Err(PipelineError::new(
            PipelineErrorKind::FileExists,
            format!("a file exists at {path} already"),
        ));
    }
    let mut doc = Document::parse(NEW_FEED_TEMPLATE)?;
    let feed = Feed {
        title: Some(title.to_owned()),
        author: Some(author.to_owned()),
        podcast_guid: Some(podcast_guid_for_url(&address)),
        medium: Some("music".to_owned()),
        language: Some("en".to_owned()),
        ..Feed::default()
    };
    feed.write(&mut doc)?;
    let id = collection.add_feed(None, Some((connector_id, path)))?;
    collection.set_address(id, AddressSource::Folder, Some(&address))?;
    collection.set_new_feed(id, title, "music")?;
    collection.save_draft(
        id,
        &DraftRow {
            base_version: None,
            base_xml: Vec::new(),
            draft_xml: doc.write().into_bytes(),
        },
    )?;
    Ok(id)
}

/// Parse the hosted file that the draft started from, for `write_hosted`
/// (ADR 0031).
///
/// Gives none when `base_xml` is empty or is not a feed that the core can
/// parse. The writer then uses the current rules, with no hosted position.
fn hosted_document(draft: &DraftRow) -> Option<Document> {
    if draft.base_xml.is_empty() {
        return None;
    }
    Document::parse_bytes(&draft.base_xml).ok()
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
    let hosted = hosted_document(&draft);
    feed.write_hosted(&mut doc, hosted.as_ref())?;
    draft.draft_xml = doc.write().into_bytes();
    collection.save_draft(id, &draft)?;
    Ok(draft)
}

/// The revision of a draft: a hash of its bytes (ADR 0022 item 5).
#[must_use]
pub fn revision(draft: &DraftRow) -> String {
    sha256_hex(&[&draft.draft_xml])
}

/// True when the draft of a feed is different from the live file that it
/// started from, or when it has a copy or a deletion on a host.
///
/// This check needs no host, and no library: a caller that only shows the
/// draft, such as the editor, does not need either one.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the feed has no draft.
pub fn draft_changed(collection: &Collection, id: FeedId) -> Result<bool, PipelineError> {
    let draft = current_draft(collection, id)?;
    // A copy or a deletion on a host is a change also when the feed XML
    // does not change (ADR 0036 item 12).
    Ok(draft.base_xml != draft.draft_xml || !collection.draft_operations(id)?.is_empty())
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
    let hosted = hosted_document(&draft);
    sent.write_hosted(&mut doc, hosted.as_ref())?;
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

/// The path of an imported file in the release folder of a feed file
/// (ADR 0032 item 4). The folder has the name of the feed file with no
/// extension: `album/my-album.xml` and `01 A.mp3` give
/// `album/my-album/01 A.mp3`.
#[must_use]
pub fn release_path(feed_path: &str, file_name: &str) -> String {
    let (parent, file) = feed_path
        .rsplit_once('/')
        .map_or(("", feed_path), |(parent, file)| (parent, file));
    let stem = file.rsplit_once('.').map_or(file, |(stem, _)| stem);
    if parent.is_empty() {
        format!("{stem}/{file_name}")
    } else {
        format!("{parent}/{stem}/{file_name}")
    }
}

/// The result of a compare of a file on the host with a local file, for an
/// import that may reuse the file on the host (ADR 0034).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostFile {
    /// No file is at the path on the host.
    Absent,
    /// The file on the host has the same SHA-256 as the local file.
    Same,
    /// The file on the host has a different SHA-256 than the local file.
    Different,
}

/// Compare the file at `path` on the host with the local file at `local`, by
/// the SHA-256 of each (ADR 0034).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the read of the host file fails, or
/// `local` cannot be read.
pub fn host_file_matches(
    connector: &dyn Connector,
    path: &str,
    local: &Path,
) -> Result<HostFile, PipelineError> {
    let Some(hosted) = connector.read(path)? else {
        return Ok(HostFile::Absent);
    };
    let local_bytes = std::fs::read(local).map_err(|error| {
        PipelineError::new(
            PipelineErrorKind::Connector,
            format!("{}: cannot read the local file: {error}", local.display()),
        )
    })?;
    if sha256_hex(&[&hosted.bytes]) == sha256_hex(&[&local_bytes]) {
        Ok(HostFile::Same)
    } else {
        Ok(HostFile::Different)
    }
}

/// Each address that the draft gives for the channel or for a track: the
/// `url` of an enclosure, the `uri` of a `podcast:source`, or the `href`
/// of `itunes:image` of the channel or of an item (ADR 0032 item 10,
/// ADR 0033 item 7, ADR 0036 item 7 and item 10).
#[must_use]
pub fn feed_addresses(feed: &Feed) -> Vec<&str> {
    let mut urls: Vec<&str> = Vec::new();
    urls.extend(feed.image.as_deref());
    for item in &feed.items {
        urls.extend(
            item.value
                .enclosure
                .as_ref()
                .and_then(|enclosure| enclosure.url.as_deref()),
        );
        urls.extend(item.value.image.as_deref());
        for enclosure in &item.value.alternate_enclosures {
            for source in &enclosure.sources {
                urls.extend(source.uri.as_deref());
            }
        }
    }
    urls
}

/// The diff between the live file at the start of the draft and the draft,
/// with each host operation that the write would do (ADR 0036 item 14).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the feed has no draft.
pub fn diff(
    collection: &Collection,
    hosts: &Hosts,
    library: &Library,
    id: FeedId,
) -> Result<Diff, PipelineError> {
    let feed_row = collection.feed(id)?;
    let draft = current_draft(collection, id)?;
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    let (operations, blocked) = host_operations(collection, library, hosts, &feed_row, &feed)?;
    let extra = operation_bytes(collection, hosts, &operations)?;
    let files = upload_files(collection, &operations)?;
    let base = String::from_utf8_lossy(&draft.base_xml);
    let new = String::from_utf8_lossy(&draft.draft_xml);
    let unified = TextDiff::from_lines(base.as_ref(), new.as_ref())
        .unified_diff()
        .context_radius(3)
        .header("live", "draft")
        .to_string();
    Ok(Diff {
        id: diff_id(&draft, &extra),
        unified,
        changed: draft.base_xml != draft.draft_xml,
        files,
        operations,
        blocked,
    })
}

/// Each library file that an upload of `operations` will put on a host,
/// for [`Diff::files`] (ADR 0036 item 10).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails.
fn upload_files(
    collection: &Collection,
    operations: &[HostOperation],
) -> Result<Vec<DiffFile>, PipelineError> {
    let mut files = Vec::new();
    for operation in operations {
        if operation.kind != OperationKind::Upload {
            continue;
        }
        let library_file_id = operation
            .library_file_id
            .expect("an upload names its library file");
        let file = collection.library_file(LibraryFileId(library_file_id))?;
        files.push(DiffFile {
            path: file.path,
            size: file.size,
            media_type: file.media_type,
        });
    }
    Ok(files)
}

/// The title of a feed, for a message that names it. A feed with no title
/// yet shows its ID instead.
fn feed_title(row: &FeedRow) -> String {
    row.title
        .clone()
        .unwrap_or_else(|| format!("feed {}", row.id.0))
}

/// True when the hosted XML that Studio last put into the release folder
/// of `row` gives `url` as an address (ADR 0036 item 3 and item 13).
///
/// The function gives `false` when `row` has no release folder yet, or
/// Studio has not yet put a hosted file there: it is not a feed that this
/// Studio has confirmed, so it does not block a deletion.
fn feed_refers_in_library(library: &Library, row: &FeedRow, url: &str) -> bool {
    let (Some(folder), Some(path)) = (row.folder.as_deref(), row.path.as_deref()) else {
        return false;
    };
    let file_name = path.rsplit_once('/').map_or(path, |(_, file)| file);
    let Ok(bytes) = std::fs::read(library.path(folder, file_name)) else {
        return false;
    };
    let Ok(doc) = Document::parse_bytes(&bytes) else {
        return false;
    };
    let Ok(feed) = Feed::read(&doc) else {
        return false;
    };
    feed_addresses(&feed).contains(&url)
}

/// True when a host already has a file at `path` with this size and this
/// SHA-256 (ADR 0036 item 9). The function reads the SHA-256 of the host
/// file only when the size agrees, through [`inventory::host_sha`]
/// (ADR 0036 item 6).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection or the host fails.
fn host_has_file(
    collection: &Collection,
    hosts: &Hosts,
    connector_id: ConnectorId,
    path: &str,
    size: u64,
    sha256: &str,
) -> Result<bool, PipelineError> {
    let Some(existing) = collection.host_file(connector_id, path)? else {
        return Ok(false);
    };
    if existing.size != size {
        return Ok(false);
    }
    let found = match existing.sha256 {
        Some(found) => Some(found),
        None => inventory::host_sha(collection, hosts, connector_id, path)?,
    };
    Ok(found.as_deref() == Some(sha256))
}

/// The host operation of a draft copy. Gives `None` when the copy has no
/// library file, or the host has the file at the path already.
///
/// A copy of a remote file waits for its download (ADR 0038 item 12).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection or a host fails.
fn copy_operation(
    collection: &Collection,
    hosts: &Hosts,
    operation: &crate::collection::DraftOperation,
) -> Result<Option<HostOperation>, PipelineError> {
    let Some(library_file_id) = operation.library_file_id else {
        return Ok(None);
    };
    let file = collection.library_file(library_file_id)?;
    let waiting = !file.local || file.sha256.is_none();
    if !waiting {
        let sha256 = file.sha256.as_deref().unwrap_or_default();
        let on_host = host_has_file(
            collection,
            hosts,
            operation.connector_id,
            &operation.path,
            file.size,
            sha256,
        )?;
        if on_host {
            return Ok(None);
        }
    }
    let connector = hosts.get(operation.connector_id)?;
    Ok(Some(HostOperation {
        kind: OperationKind::Copy,
        connector_id: operation.connector_id,
        path: operation.path.clone(),
        size: file.size,
        library_file_id: Some(file.id.0),
        url: connector.public_url(&operation.path),
        operation_id: Some(operation.id.0),
        waiting,
    }))
}

/// Each host operation of the draft of a feed, and each deletion that
/// Studio refuses because a feed refers to the file (ADR 0036 items 10,
/// 12, 13 and 14).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection or a host fails.
fn host_operations(
    collection: &Collection,
    library: &Library,
    hosts: &Hosts,
    feed_row: &FeedRow,
    feed: &Feed,
) -> Result<(Vec<HostOperation>, Vec<BlockedDelete>), PipelineError> {
    let urls = feed_addresses(feed);
    let mut operations = Vec::new();

    // Each library file whose planned address is in the draft, with no
    // copy of the same bytes at its planned path yet, is an upload
    // (ADR 0036 item 10). A draft that refers to a copy on a different
    // host uses that copy, so it needs no upload.
    for file in collection.library_files(feed_row.id)? {
        let planned = inventory::planned_path(collection, hosts, feed_row, &file)?;
        let planned_in_use = planned
            .url
            .as_deref()
            .is_some_and(|url| urls.contains(&url));
        // A remote file has no bytes in the library, so it is never an
        // upload (ADR 0038 item 1).
        if planned_in_use && !planned.on_host && file.sha256.is_some() {
            operations.push(HostOperation {
                kind: OperationKind::Upload,
                connector_id: planned.connector_id,
                path: planned.path,
                size: file.size,
                library_file_id: Some(file.id.0),
                url: planned.url,
                operation_id: None,
                waiting: false,
            });
        }
    }

    // Each draft operation gives a copy, or a deletion, or a blocked
    // deletion (ADR 0036 items 12 and 13).
    let all_feeds = collection.feeds()?;
    let mut blocked = Vec::new();
    for operation in collection.draft_operations(feed_row.id)? {
        match operation.kind {
            DraftOperationKind::Copy => {
                if let Some(copy) = copy_operation(collection, hosts, &operation)? {
                    operations.push(copy);
                }
            }
            DraftOperationKind::Delete => {
                let connector = hosts.get(operation.connector_id)?;
                let url = connector.public_url(&operation.path);
                let mut referring: Vec<String> = Vec::new();
                if let Some(url) = url.as_deref() {
                    if urls.contains(&url) {
                        referring.push(feed_title(feed_row));
                    }
                    for other in &all_feeds {
                        if other.id != feed_row.id && feed_refers_in_library(library, other, url) {
                            referring.push(feed_title(other));
                        }
                    }
                }
                if referring.is_empty() {
                    let size = collection
                        .host_file(operation.connector_id, &operation.path)?
                        .map_or(0, |file| file.size);
                    operations.push(HostOperation {
                        kind: OperationKind::Delete,
                        connector_id: operation.connector_id,
                        path: operation.path.clone(),
                        size,
                        library_file_id: None,
                        url,
                        operation_id: Some(operation.id.0),
                        waiting: false,
                    });
                } else {
                    blocked.push(BlockedDelete {
                        connector_id: operation.connector_id,
                        path: operation.path.clone(),
                        feed_titles: referring,
                        operation_id: operation.id.0,
                    });
                }
            }
        }
    }

    Ok((operations, blocked))
}

/// The bytes that [`diff_id`] adds for each host operation: its kind, its
/// host, its path and its SHA-256, in sequence (ADR 0036 item 14). The
/// bytes change when an operation changes.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection or a host fails.
fn operation_bytes(
    collection: &Collection,
    hosts: &Hosts,
    operations: &[HostOperation],
) -> Result<Vec<u8>, PipelineError> {
    let mut bytes = Vec::new();
    for operation in operations {
        bytes.push(0);
        bytes.extend_from_slice(operation.kind.as_str().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(operation.connector_id.0.to_string().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(operation.path.as_bytes());
        bytes.push(0);
        let sha256 = match operation.library_file_id {
            Some(id) => collection
                .library_file(LibraryFileId(id))?
                .sha256
                .unwrap_or_default(),
            None => {
                inventory::host_sha(collection, hosts, operation.connector_id, &operation.path)?
                    .unwrap_or_default()
            }
        };
        bytes.extend_from_slice(sha256.as_bytes());
    }
    Ok(bytes)
}

/// Put a file into the release folder of a feed, and keep its row in the
/// library (ADR 0036 item 5).
///
/// A file with the same name as a file that Studio put there already,
/// and with the same SHA-256, is the same file: the function gives its
/// row again, and does not move `source`. A file with the same name and
/// different bytes gives [`PipelineErrorKind::FileExists`], and `source`
/// stays where it was.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::NoFolder`] when the feed has no release
/// folder yet. Gives [`PipelineErrorKind::FileExists`] when a different
/// file is at `file_name` already. Gives a [`PipelineError`] when Studio
/// cannot read `source`, or cannot write the release folder, or the
/// collection fails.
pub fn import_into_library(
    collection: &Collection,
    library: &Library,
    hosts: &Hosts,
    id: FeedId,
    source: &Path,
    file_name: &str,
    media_type: &str,
) -> Result<ImportedFile, PipelineError> {
    let feed_row = collection.feed(id)?;
    let folder = feed_row.folder.clone().ok_or_else(|| {
        PipelineError::new(
            PipelineErrorKind::NoFolder,
            format!("feed {} has no release folder yet", id.0),
        )
    })?;

    // Read the SHA-256 of the source file first, to compare it with a
    // file that is already at this name, before Studio moves it into the
    // library (ADR 0036 item 5).
    let bytes = std::fs::read(source).map_err(|error| {
        PipelineError::new(
            PipelineErrorKind::Connector,
            format!(
                "{}: cannot read the file to import: {error}",
                source.display()
            ),
        )
    })?;
    let sha256 = sha256_hex(&[&bytes]);
    let size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);

    let existing = collection
        .library_files(id)?
        .into_iter()
        .find(|file| file.path == file_name);
    let library_file_id = match existing {
        Some(file) if file.sha256.as_deref() == Some(sha256.as_str()) => {
            // The same file, imported again. Studio keeps the one copy,
            // and removes the file that the caller sent (ADR 0036 item 5).
            let _ = std::fs::remove_file(source);
            file.id
        }
        Some(_) => {
            return Err(PipelineError::new(
                PipelineErrorKind::FileExists,
                format!("{file_name} is in the release folder already, with different bytes"),
            ));
        }
        None => {
            library
                .put_file(&folder, file_name, source)
                .map_err(|error| {
                    PipelineError::new(PipelineErrorKind::Connector, error.to_string())
                })?;
            let stamp = file_stamp(&library.path(&folder, file_name));
            collection.add_library_file(
                id,
                file_name,
                size,
                Some(&sha256),
                media_type,
                true,
                None,
                stamp.as_deref(),
            )?
        }
    };

    let file = collection.library_file(library_file_id)?;
    let planned = inventory::planned_path(collection, hosts, &feed_row, &file)?;
    Ok(ImportedFile {
        library_file_id,
        planned,
    })
}

/// True when a library file is in use by the draft of its feed: its
/// planned URL, or the URL of one of its copies, is an address of the
/// draft (ADR 0036 item 11).
///
/// A feed with no draft refers to no address, so the function gives
/// `false`.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection or a host fails.
pub fn library_file_in_use(
    collection: &Collection,
    hosts: &Hosts,
    file: &LibraryFile,
) -> Result<bool, PipelineError> {
    let feed_row = collection.feed(file.feed_id)?;
    let urls: Vec<String> = match draft_model(collection, file.feed_id) {
        Ok((feed, _)) => feed_addresses(&feed)
            .into_iter()
            .map(str::to_owned)
            .collect(),
        Err(error) if error.kind() == PipelineErrorKind::NoDraft => Vec::new(),
        Err(error) => return Err(error),
    };
    let planned = inventory::planned_path(collection, hosts, &feed_row, file)?;
    if planned.url.is_some_and(|url| urls.contains(&url)) {
        return Ok(true);
    }
    for copy in inventory::copies(collection, hosts, file)? {
        if copy.url.is_some_and(|url| urls.contains(&url)) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Write the draft to the host. The write also does each host operation of
/// the draft (ADR 0007, ADR 0036 items 14 and 15).
///
/// The function writes only when `approved` is the ID of the current diff.
/// It also checks that the live file still has the version that the draft
/// started from. The function keeps a snapshot of the live file first.
///
/// The sequence of the write is: each upload and copy, the feed file, a
/// check of each new address, then each deletion. After the write, the
/// draft and its host operations go out. Studio then scans each host that
/// the write touched.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::DiffNotApproved`],
/// [`PipelineErrorKind::WaitingForDownload`],
/// [`PipelineErrorKind::BadAddress`] or [`PipelineErrorKind::Conflict`]
/// before any file changes on a host.
///
/// Gives [`PipelineErrorKind::FileExists`] when a file exists already at
/// the planned path of an upload or a copy. Gives
/// [`PipelineErrorKind::NotReachable`] when a new address is not on its
/// host after the uploads. The feed file is already written in both cases.
///
/// Gives [`PipelineErrorKind::Conflict`] when a host file changed after
/// the scan, before its deletion. The feed file stays written.
pub fn write_draft(
    collection: &mut Collection,
    hosts: &Hosts,
    library: &Library,
    id: FeedId,
    approved: &DiffId,
) -> Result<Version, PipelineError> {
    let row = collection.feed(id)?;
    let (connector_id, path) = writable(&row)?;
    let path = path.to_owned();
    let connector = hosts.get(connector_id)?;
    let draft = current_draft(collection, id)?;
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    let (operations, _blocked) = host_operations(collection, library, hosts, &row, &feed)?;
    let extra = operation_bytes(collection, hosts, &operations)?;
    if &diff_id(&draft, &extra) != approved {
        return Err(PipelineError::new(
            PipelineErrorKind::DiffNotApproved,
            "the draft changed after the diff was approved; approve the new diff",
        ));
    }
    // ADR 0038 item 12: no file changes while a download is not complete.
    if let Some(waiting) = operations.iter().find(|operation| operation.waiting) {
        return Err(PipelineError::new(
            PipelineErrorKind::WaitingForDownload,
            format!(
                "{} waits for the download of a file; write the draft after the download is complete",
                waiting.path
            ),
        ));
    }
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

    // Each host that the write puts a file on, deletes a file from, or
    // writes the feed file to. Studio scans each one after the write
    // (ADR 0036 item 6).
    let mut touched: HashSet<ConnectorId> = HashSet::new();
    touched.insert(connector_id);

    // Step 1: each upload and copy puts a library file on its host.
    put_uploads_and_copies(collection, hosts, library, &row, &operations, &mut touched)?;

    // Step 2: the feed file, as before. The release folder then keeps the
    // same bytes as the hosted file (ADR 0036 item 3).
    let expected = match &live {
        Some(file) => {
            collection.add_snapshot(id, &file.version, &file.bytes)?;
            Expected::Version(file.version.clone())
        }
        None => Expected::Absent,
    };
    let version = connector.write(&path, &draft.draft_xml, &expected)?;
    collection.record_state(id, &feed, &version)?;
    if let Some(folder) = row.folder.as_deref() {
        let file_name = path
            .rsplit_once('/')
            .map_or(path.as_str(), |(_, name)| name);
        library
            .write_feed_xml(folder, file_name, &draft.draft_xml)
            .map_err(|error| PipelineError::new(PipelineErrorKind::Connector, error.to_string()))?;
    }

    // Step 3: confirm that each upload and each copy is on its host,
    // before Studio deletes any file (ADR 0036 item 15).
    confirm_new_addresses(hosts, &operations)?;

    // Step 4: each deletion, with the stamp from the inventory
    // (ADR 0036 item 16). `host_operations` ran again above, with the
    // current state of each feed. A blocked deletion is already out of
    // `operations` (ADR 0036 item 13).
    delete_each_operation(collection, hosts, &operations, &mut touched)?;

    // Step 5: the draft and its host operations go out.
    collection.remove_draft_operations(id)?;
    collection.delete_draft(id)?;

    for connector_id in touched {
        inventory::scan(collection, hosts, connector_id)?;
    }

    Ok(version)
}

/// Put each upload and each copy of `operations` on its host: the first
/// step of [`write_draft`] (ADR 0036 item 15).
///
/// The write does not replace a file that is on the host already. After
/// each file, the function records it in the inventory, with the
/// SHA-256 of the library file (ADR 0036 item 6).
///
/// # Errors
///
/// Gives [`PipelineErrorKind::NoFolder`] when the feed has no release
/// folder, and [`PipelineErrorKind::FileExists`] when a file is on the
/// host at the planned path already.
fn put_uploads_and_copies(
    collection: &Collection,
    hosts: &Hosts,
    library: &Library,
    row: &FeedRow,
    operations: &[HostOperation],
    touched: &mut HashSet<ConnectorId>,
) -> Result<(), PipelineError> {
    for operation in operations {
        if !matches!(operation.kind, OperationKind::Upload | OperationKind::Copy) {
            continue;
        }
        let library_file_id = operation
            .library_file_id
            .expect("an upload or a copy names its library file");
        let file = collection.library_file(LibraryFileId(library_file_id))?;
        let folder = row.folder.as_deref().ok_or_else(|| {
            PipelineError::new(
                PipelineErrorKind::NoFolder,
                format!("feed {} has no release folder yet", row.id.0),
            )
        })?;
        let bytes = std::fs::read(library.path(folder, &file.path)).map_err(|error| {
            PipelineError::new(
                PipelineErrorKind::Connector,
                format!("{}: cannot read the library file: {error}", file.path),
            )
        })?;
        let target = hosts.get(operation.connector_id)?;
        match target.write(&operation.path, &bytes, &Expected::Absent) {
            Ok(_) => {}
            Err(error) if error.kind() == ConnectorErrorKind::Conflict => {
                return Err(PipelineError::new(
                    PipelineErrorKind::FileExists,
                    format!("a file exists at {} already", operation.path),
                ));
            }
            Err(error) => return Err(error.into()),
        }
        let listed = target.stat(&operation.path)?.ok_or_else(|| {
            PipelineError::new(
                PipelineErrorKind::Connector,
                format!("{} is not on the host after the write", operation.path),
            )
        })?;
        collection.put_host_file(
            operation.connector_id,
            &operation.path,
            listed.size,
            &listed.stamp,
        )?;
        let sha256 = file.sha256.clone().unwrap_or_else(|| sha256_hex(&[&bytes]));
        collection.set_host_file_sha(operation.connector_id, &operation.path, &sha256)?;
        touched.insert(operation.connector_id);
    }
    Ok(())
}

/// Confirm that each upload and each copy of `operations` is on its
/// host: the third step of [`write_draft`], before any deletion
/// (ADR 0036 item 15).
///
/// # Errors
///
/// Gives [`PipelineErrorKind::NotReachable`] when a file is not on its
/// host.
fn confirm_new_addresses(hosts: &Hosts, operations: &[HostOperation]) -> Result<(), PipelineError> {
    for operation in operations {
        if !matches!(operation.kind, OperationKind::Upload | OperationKind::Copy) {
            continue;
        }
        let target = hosts.get(operation.connector_id)?;
        if target.read(&operation.path)?.is_none() {
            let name = operation.url.as_deref().unwrap_or(&operation.path);
            return Err(PipelineError::new(
                PipelineErrorKind::NotReachable,
                format!("{name} is not on its host after the uploads"),
            ));
        }
    }
    Ok(())
}

/// Delete each deletion of `operations`, with the stamp from the
/// inventory: the fourth step of [`write_draft`] (ADR 0036 items 13 and
/// 16).
///
/// A deletion with no row in the inventory is already gone, and the
/// function does nothing for it.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::Conflict`] when a host file has a
/// different stamp than the inventory.
fn delete_each_operation(
    collection: &Collection,
    hosts: &Hosts,
    operations: &[HostOperation],
    touched: &mut HashSet<ConnectorId>,
) -> Result<(), PipelineError> {
    for operation in operations {
        if operation.kind != OperationKind::Delete {
            continue;
        }
        let target = hosts.get(operation.connector_id)?;
        let Some(host_file) = collection.host_file(operation.connector_id, &operation.path)? else {
            continue;
        };
        match target.delete(&operation.path, &host_file.stamp) {
            Ok(()) => {
                collection.remove_host_file(operation.connector_id, &operation.path)?;
                touched.insert(operation.connector_id);
            }
            Err(error) if error.kind() == ConnectorErrorKind::Conflict => {
                return Err(PipelineError::new(
                    PipelineErrorKind::Conflict,
                    format!("{} changed on the host after the scan", operation.path),
                ));
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

/// Remove the host operations that the user started in the draft of a
/// feed (ADR 0036 item 12).
///
/// A route calls this before it discards a draft, before it puts a new
/// draft from a snapshot, and before it deletes a feed. Task 021e removes
/// the pending-file store, so the function no longer touches a pending
/// file.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails.
pub fn discard_draft(collection: &Collection, id: FeedId) -> Result<(), PipelineError> {
    collection.remove_draft_operations(id)?;
    Ok(())
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

/// The ID of the diff of a draft, from its bytes and `extra`. The ID
/// changes when `extra` changes.
///
/// `extra` is the bytes of [`operation_bytes`], or an empty slice for a
/// caller that does not yet know the host operations of the draft
/// (ADR 0036 item 14).
fn diff_id(draft: &DraftRow, extra: &[u8]) -> DiffId {
    let base = draft
        .base_version
        .as_ref()
        .map_or("absent", Version::as_str);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(base.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&draft.draft_xml);
    bytes.extend_from_slice(extra);
    DiffId(sha256_hex(&[&bytes]))
}

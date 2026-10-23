//! The write pipeline: draft, diff, write and snapshot (ADR 0007, ADR 0015).
//!
//! A write to a host occurs only through [`write_draft`], and only with the
//! ID of the diff that the user approved.

use std::fmt;

use similar::TextDiff;

use crate::address::AddressSource;
use crate::collection::{
    Collection, CollectionError, ConnectorId, DraftFile, DraftRow, FeedId, FeedRow, SnapshotId,
};
use crate::connector::{Connector, ConnectorError, ConnectorErrorKind, Expected, Version};
use crate::digest::sha256_hex;
use crate::feed::{Feed, FeedError};
use crate::guid::podcast_guid_for_url;
use crate::imports;
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
    /// or at the path of a pending file (ADR 0026 item 7).
    FileExists,
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

/// A pending file that the write puts on the host before the feed file
/// (ADR 0026 item 6, ADR 0032 item 10).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DiffFile {
    /// The path of the file in the release folder.
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
    /// Each pending file that the write puts on the host (ADR 0026 item 6).
    pub files: Vec<DiffFile>,
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

/// Each pending file of a draft that the draft refers to, as the address of
/// an enclosure or of an image (ADR 0032 item 10).
///
/// The draft refers to a pending file when `connector.public_url` of its
/// path is the `url` of an enclosure. It also refers to a file that is the
/// `href` of `itunes:image` of the channel or of an item.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails.
pub fn referenced_files(
    collection: &Collection,
    connector: &dyn Connector,
    id: FeedId,
    feed: &Feed,
) -> Result<Vec<DraftFile>, PipelineError> {
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
    }
    let files = collection
        .draft_files(id)?
        .into_iter()
        .filter(|file| {
            connector
                .public_url(&file.path)
                .is_some_and(|url| urls.contains(&url.as_str()))
        })
        .collect();
    Ok(files)
}

/// The diff between the live file at the start of the draft and the draft.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the feed has no draft.
pub fn diff(
    collection: &Collection,
    connector: &dyn Connector,
    id: FeedId,
) -> Result<Diff, PipelineError> {
    let draft = current_draft(collection, id)?;
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    let referenced = referenced_files(collection, connector, id, &feed)?;
    let base = String::from_utf8_lossy(&draft.base_xml);
    let new = String::from_utf8_lossy(&draft.draft_xml);
    let unified = TextDiff::from_lines(base.as_ref(), new.as_ref())
        .unified_diff()
        .context_radius(3)
        .header("live", "draft")
        .to_string();
    Ok(Diff {
        id: diff_id(&draft, &referenced),
        unified,
        changed: draft.base_xml != draft.draft_xml,
        files: referenced
            .into_iter()
            .map(|file| DiffFile {
                path: file.path,
                size: file.size,
                media_type: file.media_type,
            })
            .collect(),
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
    store: &imports::Store,
    id: FeedId,
    approved: &DiffId,
) -> Result<Version, PipelineError> {
    let row = collection.feed(id)?;
    let path = writable(&row)?.to_owned();
    let draft = current_draft(collection, id)?;
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    let referenced = referenced_files(collection, connector, id, &feed)?;
    if &diff_id(&draft, &referenced) != approved {
        return Err(PipelineError::new(
            PipelineErrorKind::DiffNotApproved,
            "the draft changed after the diff was approved; approve the new diff",
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

    // Put each file that the draft refers to on the host first. The write
    // does not replace a file that exists there already (ADR 0026 items 5
    // to 7, ADR 0032 items 10 and 11). When a file exists, the write stops
    // here, and the feed file does not change.
    for file in &referenced {
        let bytes = store.open(id, &file.sha256).map_err(|error| {
            PipelineError::new(
                PipelineErrorKind::Connector,
                format!("{}: cannot read the pending file: {error}", file.path),
            )
        })?;
        match connector.write(&file.path, &bytes, &Expected::Absent) {
            Ok(_) => {}
            Err(error) if error.kind() == ConnectorErrorKind::Conflict => {
                return Err(PipelineError::new(
                    PipelineErrorKind::FileExists,
                    format!("a file exists at {} already", file.path),
                ));
            }
            Err(error) => return Err(error.into()),
        }
        // The file is on the host now, so Studio removes its row and its
        // copy. A write that stops after this point puts the file again.
        remove_pending_file(collection, store, id, file)?;
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
    // Remove the row and the copy of each pending file that the draft did
    // not refer to (ADR 0032 item 11).
    discard_draft(collection, store, id)?;
    Ok(version)
}

/// Remove one pending file of a draft: its row, and its copy when no other
/// pending file of the draft has the same bytes.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails.
pub fn remove_pending_file(
    collection: &Collection,
    store: &imports::Store,
    id: FeedId,
    file: &DraftFile,
) -> Result<(), PipelineError> {
    collection.remove_draft_file(id, &file.path)?;
    // Two pending files with the same bytes share one copy. The copy goes
    // only with the last file that uses it.
    if !collection.draft_file_sha_in_use(id, &file.sha256)? {
        store.remove(id, &file.sha256);
    }
    Ok(())
}

/// Remove the pending files of a feed: their rows and their copies in the
/// data folder (ADR 0026 item 8, ADR 0032 item 12).
///
/// A route calls this before it discards a draft, before it puts a version
/// in a draft, and before it deletes a feed.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails.
pub fn discard_draft(
    collection: &Collection,
    store: &imports::Store,
    id: FeedId,
) -> Result<(), PipelineError> {
    for file in collection.draft_files(id)? {
        store.remove(id, &file.sha256);
    }
    collection.remove_draft_files(id)?;
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

/// The ID of the diff of a draft, from its bytes and the path, the size and
/// the SHA-256 of each file that it refers to, in sequence (ADR 0032
/// item 10). The ID changes when a referenced file changes.
fn diff_id(draft: &DraftRow, referenced: &[DraftFile]) -> DiffId {
    let base = draft
        .base_version
        .as_ref()
        .map_or("absent", Version::as_str);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(base.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&draft.draft_xml);
    for file in referenced {
        bytes.push(0);
        bytes.extend_from_slice(file.path.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(file.size.to_string().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(file.sha256.as_bytes());
    }
    DiffId(sha256_hex(&[&bytes]))
}

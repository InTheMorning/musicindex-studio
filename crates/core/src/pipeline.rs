//! The write pipeline: draft, diff, write and snapshot (ADR 0007, ADR 0015).
//!
//! A write to a host occurs only through [`write_draft`], and only with the
//! ID of the diff that the user approved.

use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::path::Path;

use similar::TextDiff;

use crate::address::AddressSource;
use crate::announce::{self, AnnounceSettings, AnnounceSkip, PlannedAnnouncement, Timing};
use crate::collection::{
    Collection, CollectionError, ConnectorId, DraftFeedMove, DraftOperationKind, DraftRow, FeedId,
    FeedRow, LibraryFile, LibraryFileId, SnapshotId,
};
use crate::connector::{
    Connector, ConnectorError, ConnectorErrorKind, Expected, HostChanges, PlannedDelete,
    PlannedWrite, Version,
};
use crate::digest::sha256_hex;
use crate::feed::{AtomLink, Entry, Feed, FeedError, Item, Person};
use crate::grant::{self, GrantChange};
use crate::guid::podcast_guid_for_url;
use crate::inventory::{self, Hosts, InventoryError, InventoryErrorKind};
use crate::library::{Library, file_stamp};
use crate::live::{self, LiveStatus};
use crate::media::{op3_prefix, plain_address};
use crate::move_link::{self, BadMoveLink, FeedReader, LinkReport, OwnedDraftLinks, Repair};
use crate::nostr_events::npub_text;
use crate::roles;
use crate::xml::{Document, Element, ParseError};

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
    /// A key field of the draft is not a Nostr key, and the hosted feed
    /// does not have the same text in a key field (ADR 0074 item 5).
    BadKey,
    /// A live item has no start, no content link, or a status that is not
    /// `pending`, `live` or `ended` (ADR 0054 item 4).
    BadLiveItem,
    /// A live value has a long address or protocol, or a control character
    /// (task 072d).
    InvalidLiveValue,
    /// The live file changed after the draft started.
    Conflict,
    /// The feed file is not a feed that Studio can read.
    InvalidFeed,
    /// The connector failed.
    Connector,
    /// The collection database failed.
    Collection,
    /// The connector gives no public address for the path of a feed that
    /// moves (ADR 0038).
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
    /// The draft refers to a pending file that Studio removed from the
    /// server at the end of its 7 days (ADR 0105 item 10).
    FileMissing,
    /// A host needs a secret from the vault, and the vault is locked
    /// (ADR 0017).
    VaultLocked,
    /// A host with "My signer" has a planned event that the user did not
    /// sign (ADR 0050 item 12).
    SignatureMissing,
    /// A signed event has a bad signature or a different public key
    /// (ADR 0050 item 12).
    SignatureInvalid,
    /// The signing plan of the feed is missing, or it expired. The user
    /// must review the diff again (ADR 0050 item 12).
    PlanExpired,
    /// The live item has no show grant, or its grant ended (ADR 0054
    /// item 14).
    GrantNotActive,
    /// The feed has a draft with a change that the user can approve, so a
    /// granted write cannot occur (ADR 0054 item 16).
    DraftOpen,
    /// The granted change makes a change that the grant does not cover
    /// (ADR 0054 item 13).
    NotCovered,
    /// A change set operation is not valid: no part, no type, no address,
    /// or a value that the person does not have (ADR 0091, ADR 0098 item
    /// 20).
    InvalidOperation,
    /// The status of the live item cannot change to the requested status
    /// (ADR 0054 item 5).
    BadStatus,
    /// A show grant of the draft names an app that is not active now, so
    /// the write would drop the grant with no message (ADR 0110 item 19).
    AppNotActive,
    /// A link of the draft is not correct, so stage 1 refuses the write.
    /// [`PipelineError::bad_move_link`] gives the condition and each hop
    /// (ADR 0126 items 9 and 10).
    BadMoveLink,
}

/// An error from the write pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineError {
    kind: PipelineErrorKind,
    message: String,
    bad_move_link: Option<Box<BadMoveLink>>,
}

impl PipelineError {
    pub(crate) fn new(kind: PipelineErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            bad_move_link: None,
        }
    }

    /// The error of a refusal in stage 1 (ADR 0126 item 10).
    pub(crate) fn refused(refusal: BadMoveLink) -> Self {
        Self {
            kind: PipelineErrorKind::BadMoveLink,
            message: refusal.to_string(),
            bad_move_link: Some(Box::new(refusal)),
        }
    }

    /// The kind of the error.
    #[must_use]
    pub fn kind(&self) -> PipelineErrorKind {
        self.kind
    }

    /// The condition and each hop of a [`PipelineErrorKind::BadMoveLink`]
    /// error.
    #[must_use]
    pub fn bad_move_link(&self) -> Option<&BadMoveLink> {
        self.bad_move_link.as_deref()
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
        let kind = match error.kind() {
            ConnectorErrorKind::Conflict => PipelineErrorKind::Conflict,
            ConnectorErrorKind::SignatureMissing => PipelineErrorKind::SignatureMissing,
            _ => PipelineErrorKind::Connector,
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

/// A Nostr event that the write signs, with no signature (ADR 0050
/// item 11).
#[derive(Debug, Clone)]
pub struct DiffEvent {
    /// The host that publishes the event.
    pub connector_id: ConnectorId,
    /// True for a host with "My signer": the user signs the event.
    pub user_signs: bool,
    /// The event, with no signature.
    pub event: nostr::prelude::UnsignedEvent,
}

impl DiffEvent {
    /// The event as a JSON object that a NIP-07 `signEvent` accepts, with
    /// its ID.
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "pubkey": self.event.pubkey.to_hex(),
            "created_at": self.event.created_at.as_secs(),
            "kind": self.event.kind.as_u16(),
            "tags": self
                .event
                .tags
                .iter()
                .map(|tag| tag.as_slice().to_vec())
                .collect::<Vec<_>>(),
            "content": self.event.content,
            "id": self.event.compute_id().to_hex(),
        })
    }
}

/// Read an event that the user signed, from its JSON text.
///
/// # Errors
///
/// Gives the reason as text when the text is not a correct Nostr event.
pub fn signed_event_from_json(json: &str) -> Result<nostr::prelude::Event, String> {
    nostr::prelude::Event::from_json(json).map_err(|error| error.to_string())
}

/// How long a signing plan lasts, in seconds. An authorization event of
/// a plan expires one hour after the plan time (ADR 0050 item 13).
const PLAN_LIFETIME: u64 = 3600;
/// A diff reuses its plan time while this many seconds or more remain
/// (ADR 0050 item 10).
const PLAN_REUSE_MARGIN: u64 = 600;
/// A write stops when fewer seconds than this remain in the plan.
const PLAN_WRITE_MARGIN: u64 = 60;

/// A line diff between the live file and the draft.
#[derive(Debug, Clone)]
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
    /// The move of the feed XML to a different host, when the draft has
    /// one (ADR 0038 item 18).
    pub feed_move: Option<FeedMove>,
    /// Each announcement that the write puts into the queue
    /// (ADR 0052 items 7 to 9).
    pub announcements: Vec<PlannedAnnouncement>,
    /// The reason why `announcements` is empty, or `None` when it is not
    /// empty.
    pub announce_skip: Option<AnnounceSkip>,
    /// Each Nostr event that the write signs, with no signature
    /// (ADR 0050 item 11).
    pub events: Vec<DiffEvent>,
    /// The `guid` of each live item that the write gives a show grant: each
    /// grant of the draft for a `pending` live item of the draft
    /// (ADR 0054 item 11).
    pub grants: Vec<String>,
    /// The field changes of the draft, in groups (ADR 0114 item 1).
    pub fields: Vec<crate::review::FieldGroup>,
    /// Each line of the live file and the draft, with its marked parts
    /// (ADR 0114 item 6).
    pub lines: Vec<crate::review::DiffLine>,
    /// The number of changes that the field changes do not explain
    /// (ADR 0114 item 4).
    pub uncovered: usize,
}

/// How the host from before points to the new address of the feed XML
/// (ADR 0038 item 18).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedMoveMethod {
    /// The host makes an HTTP 301 redirect.
    Redirect,
    /// Studio writes a pointer file with `itunes:new-feed-url`.
    Pointer,
    /// Studio writes nothing at the address before the move. The feed had
    /// no host, so Studio has no access to that address (ADR 0067
    /// item 8).
    None,
}

impl FeedMoveMethod {
    /// The text of the method, for the diff ID.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Redirect => "redirect",
            Self::Pointer => "pointer",
            Self::None => "none",
        }
    }
}

/// The move of the feed XML to a different host (ADR 0038 items 16
/// to 19).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FeedMove {
    /// The new host.
    pub connector_id: ConnectorId,
    /// The path of the feed XML. It is the same on both hosts.
    pub path: String,
    /// The new address of the feed XML.
    pub url: Option<String>,
    /// The address of the feed XML before the move.
    pub from_url: Option<String>,
    /// How the host from before points to the new address.
    pub method: FeedMoveMethod,
    /// The file at the path on the new host is a pointer file of this feed,
    /// and the feed XML replaces it (ADR 0126 item 21).
    pub replaces_pointer: bool,
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
    /// Make sure that the host gives a file that an earlier write put
    /// there (ADR 0043 item 9).
    Confirm,
}

impl OperationKind {
    /// The text of the kind, for the diff ID.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Upload => "upload",
            Self::Copy => "copy",
            Self::Delete => "delete",
            Self::Confirm => "confirm",
        }
    }
}

/// Request a public address, and give an error when the host does not
/// give a success status (ADR 0043 item 8).
pub trait AddressCheck {
    /// Request `url`.
    ///
    /// # Errors
    ///
    /// Gives the reason as text when the host does not give the file.
    fn check(&self, url: &str) -> Result<(), String>;
}

/// An upload, a copy, a confirmation or a deletion that the write does
/// on a host (ADR 0036 item 14).
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
    /// The library file of an upload, a copy or a confirmation. `None`
    /// for a deletion.
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
    /// True when the file is a pointer file of Studio. Studio keeps it:
    /// it tells the apps the new address of the feed (ADR 0038 item 20).
    pub pointer: bool,
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
            format!("{path} does not exist on the file host"),
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
            format!("{path} does not exist on the file host"),
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

/// The file name of a feed with no connector: the last segment of its
/// public address, as in [`crate::library::give_published_folder`].
fn public_file_name(row: &FeedRow) -> Option<&str> {
    let url = row.public_url.as_deref()?;
    let name = url.rsplit_once('/').map_or(url, |(_, name)| name);
    (!name.is_empty()).then_some(name)
}

/// Start a draft of a feed with no connector from the feed XML in its
/// release folder. It replaces an older draft (ADR 0067 items 3 and 4).
///
/// # Errors
///
/// Gives [`PipelineErrorKind::Conflict`] when the feed has a host, or the
/// release folder has a different feed XML than the last read. Gives
/// [`PipelineErrorKind::NotFound`] when the feed has no release folder, no
/// public address, or no file in the folder.
pub fn start_public_draft(
    collection: &Collection,
    library: &Library,
    id: FeedId,
) -> Result<DraftRow, PipelineError> {
    let row = collection.feed(id)?;
    if row.connector_id.is_some() {
        return Err(PipelineError::new(
            PipelineErrorKind::Conflict,
            format!(
                "feed {} has a file host; start the draft from the file host",
                id.0
            ),
        ));
    }
    let (Some(folder), Some(file_name)) = (row.folder.as_deref(), public_file_name(&row)) else {
        return Err(PipelineError::new(
            PipelineErrorKind::NotFound,
            format!("feed {} has no release folder or no public address", id.0),
        ));
    };
    let bytes = library.read_feed_xml(folder, file_name).map_err(|error| {
        let kind = if error.kind() == std::io::ErrorKind::NotFound {
            PipelineErrorKind::NotFound
        } else {
            PipelineErrorKind::Connector
        };
        PipelineError::new(kind, error.to_string())
    })?;
    if Some(&crate::digest::content_version(&bytes)) != row.last_version.as_ref() {
        return Err(PipelineError::new(
            PipelineErrorKind::Conflict,
            "the release folder has a different feed XML; read the feed again",
        ));
    }
    let draft = DraftRow {
        base_version: row.last_version,
        base_xml: bytes.clone(),
        draft_xml: bytes,
    };
    collection.save_draft(id, &draft)?;
    Ok(draft)
}

/// Prepare the draft of a feed that a folder match has linked to a host
/// (ADR 0067 item 13). Call it after
/// [`Collection::set_feed_connector`] gives the feed its connector.
///
/// The function removes the host choice and puts the `self` link back, as
/// [`clear_feed_host`] does. When the host file has the same SHA-256 as
/// the base of the draft, the base version becomes the version of the host
/// file. Otherwise the base version stays, and a write gives a conflict.
/// A feed with no draft does not change.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::ReadOnly`] when the feed has no connector or
/// no path. Gives a [`PipelineError`] when the host or the collection
/// fails.
pub fn rebase_linked_draft(
    collection: &Collection,
    connector: &dyn Connector,
    id: FeedId,
) -> Result<(), PipelineError> {
    let Some(draft) = collection.draft(id)? else {
        return Ok(());
    };
    let row = collection.feed(id)?;
    let (_, path) = writable(&row)?;
    remove_draft_move(collection, id, &draft)?;
    let Some(file) = connector.read(path)? else {
        return Ok(());
    };
    if sha256_hex(&[&file.bytes]) == sha256_hex(&[&draft.base_xml]) {
        let mut kept = current_draft(collection, id)?;
        kept.base_version = Some(file.version);
        collection.save_draft(id, &kept)?;
    }
    Ok(())
}

/// The host and the path that the draft of a feed with no connector moves
/// the feed to. Gives `None` for a feed with a connector, or a draft with
/// no host choice (ADR 0067 item 7).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails.
fn public_target(
    collection: &Collection,
    row: &FeedRow,
) -> Result<Option<(ConnectorId, String)>, PipelineError> {
    if row.connector_id.is_some() {
        return Ok(None);
    }
    Ok(collection
        .draft_feed_move(row.id)?
        .map(|DraftFeedMove { connector_id, path }| (connector_id, path)))
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

/// The placeholder GUID of a feed on a file host with no public address
/// (ADR 0122 item 5). Each feed gets its own placeholder, so that the links
/// between the feeds still work.
fn placeholder_guid() -> String {
    format!("pending-{}", uuid::Uuid::new_v4())
}

/// True when a GUID is a placeholder. A placeholder starts with `pending-`
/// (ADR 0122 item 5). The core replaces it when the feed gets a public
/// address (ADR 0122 item 8).
#[must_use]
pub fn has_placeholder_guid(guid: &str) -> bool {
    guid.starts_with("pending-")
}

/// Make a new feed in a connected folder, as a draft with no hosted
/// file (ADR 0032 items 6 to 9).
///
/// The feed GUID is the `UUIDv5` of the public address of `path`
/// ([`podcast_guid_for_url`]). With no public address, the feed gets a
/// placeholder GUID, and its draft has no `self` link (ADR 0122 items 5
/// and 6). The core gives the real GUID when the feed gets a public address
/// (ADR 0122 item 8). The draft sets `podcast:medium` to `medium`
/// and `language` to `en`. The caller gives `music` for an album feed
/// and `publisher` for a publisher feed (ADR 0041 item 2). The user sets each other field in the editor,
/// examines the diff, and writes it. The write puts the file only when no
/// file exists at the path (ADR 0007).
///
/// # Errors
///
/// Gives [`PipelineErrorKind::FileExists`] when a file exists there already.
pub fn new_feed(
    collection: &mut Collection,
    connector: &dyn Connector,
    connector_id: ConnectorId,
    path: &str,
    title: &str,
    author: &str,
    medium: &str,
) -> Result<FeedId, PipelineError> {
    let address = connector.public_url(path);
    if connector.read(path)?.is_some() {
        return Err(PipelineError::new(
            PipelineErrorKind::FileExists,
            format!("a file exists at {path} already"),
        ));
    }
    let mut doc = Document::parse(NEW_FEED_TEMPLATE)?;
    let guid = address
        .as_deref()
        .map_or_else(placeholder_guid, podcast_guid_for_url);
    let feed = Feed {
        title: Some(title.to_owned()),
        author: Some(author.to_owned()),
        podcast_guid: Some(guid),
        medium: Some(medium.to_owned()),
        language: Some("en".to_owned()),
        ..Feed::default()
    };
    feed.write(&mut doc)?;
    let id = collection.add_feed(None, Some((connector_id, path)))?;
    collection.set_address(id, AddressSource::Folder, address.as_deref())?;
    collection.set_new_feed(id, title, medium)?;
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

/// Make a new album feed from the draft of a source album (ADR 0077).
///
/// The checks are those of [`new_feed`]. The copy keeps each field and each
/// track of the source draft, with the changes of ADR 0077 item 6: the sent
/// `title` and `author`, the feed GUID of the new address, a new random GUID
/// for each track, and the new address in the `atom:link` with
/// `rel="self"`. The copy has no `podcast:txt` except the artist key
/// (`purpose="npub"`), no publisher reference, no live item and no
/// `itunes:new-feed-url`. An OP3 prefix with the GUID of the source gets the
/// GUID of the copy. The copy makes no library file and no host operation.
/// The source draft does not change.
///
/// # Errors
///
/// Gives the errors of [`new_feed`], and [`PipelineErrorKind::NoDraft`] when
/// the source has no draft.
pub fn copy_feed(
    collection: &mut Collection,
    connector: &dyn Connector,
    connector_id: ConnectorId,
    path: &str,
    title: &str,
    author: &str,
    source: FeedId,
) -> Result<FeedId, PipelineError> {
    let address = connector.public_url(path);
    if connector.read(path)?.is_some() {
        return Err(PipelineError::new(
            PipelineErrorKind::FileExists,
            format!("a file exists at {path} already"),
        ));
    }
    let draft = current_draft(collection, source)?;
    let mut doc = Document::parse_bytes(&draft.draft_xml)?;
    let mut feed = Feed::read(&doc)?;
    let new_guid = address
        .as_deref()
        .map_or_else(placeholder_guid, podcast_guid_for_url);
    let old_guid = feed.podcast_guid.clone();
    feed.title = Some(title.to_owned());
    feed.author = Some(author.to_owned());
    feed.podcast_guid = Some(new_guid.clone());
    feed.new_feed_url = None;
    feed.publisher = None;
    feed.live_items.clear();
    feed.txts.retain(|t| t.purpose.as_deref() == Some("npub"));
    match &address {
        Some(address) => {
            for link in &mut feed.atom_links {
                if link.value.rel.as_deref() == Some("self") {
                    link.value.href = Some(address.clone());
                }
            }
        }
        // A feed with a placeholder GUID has no `self` link (ADR 0122 item 5).
        None => feed
            .atom_links
            .retain(|link| link.value.rel.as_deref() != Some("self")),
    }
    for entry in &mut feed.items {
        entry.value.guid = Some(uuid::Uuid::new_v4().to_string());
    }
    swap_op3_prefixes(&mut feed, old_guid.as_deref(), &new_guid);
    feed.write(&mut doc)?;
    let id = collection.add_feed(None, Some((connector_id, path)))?;
    collection.set_address(id, AddressSource::Folder, address.as_deref())?;
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

/// The change of one feed under [`assign_real_guids`] (ADR 0122 items 8
/// and 15).
struct FeedChange {
    /// The feed.
    id: FeedId,
    /// The placeholder GUID and the real GUID. `None` when the feed keeps
    /// its GUID.
    guid: Option<(String, String)>,
    /// The public address. It is the `self` link of the feed when the GUID
    /// changes.
    address: String,
    /// Each placeholder address of the draft, with its public address.
    addresses: Vec<(String, String)>,
}

/// Give each feed of `ids` a real GUID in place of its placeholder, and
/// each placeholder address its public address, when its file host now
/// gives a public address (ADR 0122 items 8 and 15). The file host of a
/// feed is the target of its draft feed move, else its connector. The real
/// GUID is the `UUIDv5` of that address, and the feed gets it with its
/// `self` link and its OP3 prefixes (ADR 0076).
///
/// Each `feedGuid` that names an old placeholder changes in each draft of
/// the collection, with its `feedUrl`. A written feed with a placeholder GUID
/// has no draft, because a write removes the draft. For such a feed, the
/// function reads the live file from its open file host, and starts a draft
/// from it only when the feed changes. All changed drafts are saved in one
/// transaction. Studio writes no file. The user writes the changed drafts
/// through Review (ADR 0007).
///
/// A feed with no file host, or with a host that is locked, unknown or
/// that does not give its file, keeps its placeholders.
///
/// # Errors
///
/// Gives a [`PipelineError`] when a draft is not a feed, or when SQLite
/// fails. No draft changes then. Gives the IDs of the changed drafts.
pub fn assign_real_guids(
    collection: &Collection,
    hosts: &Hosts,
    library: Option<&Library>,
    ids: &[FeedId],
) -> Result<Vec<FeedId>, PipelineError> {
    let drafts = placeholder_candidates(collection, hosts, library)?;
    let mut changes = Vec::new();
    for &id in ids {
        let Some((_, draft)) = drafts.iter().find(|(candidate, _)| *candidate == id) else {
            continue;
        };
        if let Some(change) = feed_change(collection, hosts, id, draft)? {
            changes.push(change);
        }
    }
    if changes.is_empty() {
        return Ok(Vec::new());
    }
    let mut saved = Vec::new();
    for (id, draft) in drafts {
        let mut next = draft.clone();
        if let Some(own) = changes.iter().find(|change| change.id == id) {
            next = changed_draft(next, |feed| apply_feed_change(feed, own))?;
        }
        for change in &changes {
            next = changed_draft(next, |feed| relink(feed, change))?;
        }
        if next.draft_xml != draft.draft_xml {
            saved.push((id, next));
        }
    }
    collection.save_drafts(&saved)?;
    Ok(saved.iter().map(|(id, _)| *id).collect())
}

/// The drafts that [`assign_real_guids`] can change: each draft of the
/// collection, and a draft from the live file of each written feed with no
/// draft that has a placeholder GUID or whose hosted copy names a placeholder
/// GUID in a `remoteItem` (ADR 0122 item 8). The collection keeps the GUID of
/// the last write, so the function reads only the files of such feeds.
///
/// # Errors
///
/// Gives a [`PipelineError`] when SQLite fails.
fn placeholder_candidates(
    collection: &Collection,
    hosts: &Hosts,
    library: Option<&Library>,
) -> Result<Vec<(FeedId, DraftRow)>, PipelineError> {
    let mut drafts = Vec::new();
    for row in collection.feeds()? {
        if let Some(draft) = collection.draft(row.id)? {
            drafts.push((row.id, draft));
            continue;
        }
        let placeholder_guid = row
            .podcast_guid
            .as_deref()
            .is_some_and(has_placeholder_guid);
        let links_placeholder =
            library.is_some_and(|library| hosted_copy_names_placeholder(library, &row));
        if !placeholder_guid && !links_placeholder {
            continue;
        }
        let (Some(connector_id), Some(path)) = (row.connector_id, row.path.as_deref()) else {
            continue;
        };
        // A host that is locked, unknown or that does not give the file
        // leaves the feed as it is.
        let Ok(host) = hosts.get(connector_id) else {
            continue;
        };
        let Ok(Some(file)) = host.read(path) else {
            continue;
        };
        drafts.push((
            row.id,
            DraftRow {
                base_version: Some(file.version),
                base_xml: file.bytes.clone(),
                draft_xml: file.bytes,
            },
        ));
    }
    Ok(drafts)
}

/// The change of one feed, when its file host gives a public address and
/// the draft has a placeholder GUID or a placeholder address. Gives none
/// otherwise.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the draft is not a feed, or when SQLite
/// fails.
fn feed_change(
    collection: &Collection,
    hosts: &Hosts,
    id: FeedId,
    draft: &DraftRow,
) -> Result<Option<FeedChange>, PipelineError> {
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    let old_guid = feed
        .podcast_guid
        .clone()
        .filter(|guid| has_placeholder_guid(guid));
    let row = collection.feed(id)?;
    let target = match collection.draft_feed_move(id)? {
        Some(moved) => Some((moved.connector_id, moved.path)),
        None => row.connector_id.zip(row.path.clone()),
    };
    let Some((connector_id, path)) = target else {
        return Ok(None);
    };
    // A host that is locked or unknown gives no address yet.
    let Ok(host) = hosts.get(connector_id) else {
        return Ok(None);
    };
    let Some(address) = host.public_url(&path) else {
        return Ok(None);
    };
    // The library files are planned for the target file host, as
    // `host_operations` plans them (ADR 0122 item 15).
    let planning = FeedRow {
        files_connector_id: Some(connector_id),
        path: Some(path),
        ..row
    };
    let addresses = placeholder_addresses(collection, hosts, &planning, &feed)?;
    if old_guid.is_none() && addresses.is_empty() {
        return Ok(None);
    }
    Ok(Some(FeedChange {
        id,
        guid: old_guid.map(|old| (old, podcast_guid_for_url(&address))),
        address,
        addresses,
    }))
}

/// Each placeholder address of `feed` with the public address of its library
/// file (ADR 0122 item 15). The public address is the planned address of
/// the file with the planning row. A placeholder with no library file
/// stays, and so does one whose file has no public address.
///
/// # Errors
///
/// Gives a [`PipelineError`] when SQLite fails.
fn placeholder_addresses(
    collection: &Collection,
    hosts: &Hosts,
    planning: &FeedRow,
    feed: &Feed,
) -> Result<Vec<(String, String)>, PipelineError> {
    let placeholders: Vec<String> = feed_addresses(feed)
        .into_iter()
        .filter(|url| inventory::is_placeholder_address(url))
        .map(Cow::into_owned)
        .collect();
    if placeholders.is_empty() {
        return Ok(Vec::new());
    }
    let mut public: BTreeMap<String, String> = BTreeMap::new();
    for file in collection.library_files(planning.id)? {
        // A file that the hosts cannot plan keeps its placeholder.
        let Ok(planned) = inventory::planned_path(collection, hosts, planning, &file) else {
            continue;
        };
        if let Some(url) = planned
            .url
            .filter(|url| !inventory::is_placeholder_address(url))
        {
            public.insert(planned.path, url);
        }
    }
    Ok(placeholders
        .into_iter()
        .filter_map(|placeholder| {
            let path = placeholder.strip_prefix(inventory::PLACEHOLDER_ADDRESS_PREFIX)?;
            let url = public.get(path)?.clone();
            Some((placeholder, url))
        })
        .collect())
}

/// Give the feed its real GUID, its `self` link, its OP3 prefixes and its
/// public addresses (ADR 0122 items 8 and 15).
fn apply_feed_change(feed: &mut Feed, change: &FeedChange) {
    if let Some((old, new)) = &change.guid {
        feed.podcast_guid = Some(new.clone());
        swap_op3_prefixes(feed, Some(old), new);
        set_self_link(feed, &change.address);
    }
    replace_placeholder_addresses(feed, &change.addresses);
}

/// Give the feed a `self` link to `address`. The feed gets a new link when
/// it has none.
fn set_self_link(feed: &mut Feed, address: &str) {
    if !feed.set_self_url(address) {
        feed.atom_links.push(Entry::new(AtomLink {
            href: Some(address.to_owned()),
            rel: Some("self".to_owned()),
            ..AtomLink::default()
        }));
    }
}

/// Replace each placeholder address of `feed` with its public address (ADR
/// 0122 item 15). The OP3 switch is off for a feed with placeholder tracks,
/// because `Feed::read` reads it from the enclosures, so no prefix is added here.
fn replace_placeholder_addresses(feed: &mut Feed, changes: &[(String, String)]) {
    if changes.is_empty() {
        return;
    }
    if let Some(url) = feed.image.as_mut()
        && let Some(new) = public_of(url, changes)
    {
        *url = new;
    }
    if let Some(url) = feed.rss_image.as_mut().and_then(|image| image.url.as_mut())
        && let Some(new) = public_of(url, changes)
    {
        *url = new;
    }
    for entry in &mut feed.items {
        let item = &mut entry.value;
        if let Some(url) = item.enclosure.as_mut().and_then(|e| e.url.as_mut())
            && let Some(new) = public_of(url, changes)
        {
            *url = new;
        }
        replace_item_addresses(item, changes);
    }
    for entry in &mut feed.live_items {
        let item = &mut entry.value.item;
        if let Some(url) = item.enclosure.as_mut().and_then(|e| e.url.as_mut())
            && let Some(new) = public_of(url, changes)
        {
            *url = new;
        }
        replace_item_addresses(item, changes);
    }
}

/// Replace each placeholder address of one item that is not an enclosure:
/// the image, and the `podcast:source` of each alternate enclosure.
fn replace_item_addresses(item: &mut Item, changes: &[(String, String)]) {
    if let Some(url) = item.image.as_mut()
        && let Some(new) = public_of(url, changes)
    {
        *url = new;
    }
    for alternate in &mut item.alternate_enclosures {
        for source in &mut alternate.value.sources {
            if let Some(url) = source.value.uri.as_mut()
                && let Some(new) = public_of(url, changes)
            {
                *url = new;
            }
        }
    }
}

/// The public address of the placeholder `url`, when `changes` has one.
fn public_of(url: &str, changes: &[(String, String)]) -> Option<String> {
    changes
        .iter()
        .find(|(placeholder, _)| placeholder == url)
        .map(|(_, public)| public.clone())
}

/// Change each remote item that names the placeholder GUID of `change`. The
/// item gets the real GUID and the real address of the feed.
fn relink(feed: &mut Feed, change: &FeedChange) {
    let Some((old, new)) = &change.guid else {
        return;
    };
    let items = feed.remote_items.iter_mut().chain(
        feed.publisher
            .iter_mut()
            .flat_map(|links| links.remote_items.iter_mut()),
    );
    for entry in items {
        if entry.value.feed_guid.as_deref() == Some(old.as_str()) {
            entry.value.feed_guid = Some(new.clone());
            entry.value.feed_url = Some(change.address.clone());
        }
    }
}

/// Change the OP3 prefix of each enclosure from the old GUID to the new
/// GUID (ADR 0076). A URL with another prefix keeps its text.
fn swap_op3_prefixes(feed: &mut Feed, old_guid: Option<&str>, new_guid: &str) {
    let old_prefix = old_guid.map(|guid| format!("https://op3.dev/e,pg={guid}/"));
    let new_prefix = format!("https://op3.dev/e,pg={new_guid}/");
    let swap = |url: &mut Option<String>| {
        if let (Some(old), Some(text)) = (old_prefix.as_deref(), url.as_deref())
            && let Some(rest) = text.strip_prefix(old)
        {
            *url = Some(format!("{new_prefix}{rest}"));
        }
    };
    for entry in &mut feed.items {
        let item = &mut entry.value;
        if let Some(enclosure) = item.enclosure.as_mut() {
            swap(&mut enclosure.url);
        }
        for alternate in &mut item.alternate_enclosures {
            for source in &mut alternate.value.sources {
                swap(&mut source.value.uri);
            }
        }
    }
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
    let draft = changed_draft(current_draft(collection, id)?, change)?;
    collection.save_draft(id, &draft)?;
    Ok(draft)
}

/// The draft with the change applied, in memory. The draft is not saved.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the draft is not a feed.
fn changed_draft(
    mut draft: DraftRow,
    change: impl FnOnce(&mut Feed),
) -> Result<DraftRow, PipelineError> {
    let mut doc = Document::parse_bytes(&draft.draft_xml)?;
    let mut feed = Feed::read(&doc)?;
    change(&mut feed);
    let hosted = hosted_document(&draft);
    feed.write_hosted(&mut doc, hosted.as_ref())?;
    draft.draft_xml = doc.write().into_bytes();
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
    Ok(draft.base_xml != draft.draft_xml
        || !collection.draft_operations(id)?.is_empty()
        || collection.draft_feed_move(id)?.is_some())
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
    live::check_live_values(&sent)
        .map_err(|e| PipelineError::new(PipelineErrorKind::InvalidLiveValue, e.to_string()))?;
    number_tracks(&current, &mut sent);
    keep_cover_form(&current, &mut sent);
    keep_explicit_form(&current, &mut sent);
    apply_op3(&current, &mut sent);
    apply_pub_date_time(&current, &mut sent).map_err(invalid)?;
    apply_duration_seconds(&current, &mut sent);
    keep_key_form(&current, &mut sent);
    apply_role_group(&current, &mut sent);
    let hosted = hosted_document(&draft);
    sent.write_hosted(&mut doc, hosted.as_ref())?;
    draft.draft_xml = doc.write().into_bytes();
    collection.save_draft(id, &draft)?;
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    Ok((feed, revision(&draft)))
}

/// Apply the OP3 switch to the enclosure of each track (ADR 0076 items 8
/// to 10). A sent `true` adds the prefix of the feed to each enclosure with
/// no prefix. This includes a new track. A sent `false` over a draft
/// `true` removes the prefix from each enclosure. A sent `false` over a
/// draft `false` changes no address. The step does not change an alternate
/// enclosure, an image or a live item. A placeholder address gets no
/// prefix (ADR 0122 item 13).
fn apply_op3(current: &Feed, sent: &mut Feed) {
    let guid = current.podcast_guid.as_deref().unwrap_or_default();
    for entry in &mut sent.items {
        let Some(url) = entry.value.enclosure.as_mut().and_then(|e| e.url.as_mut()) else {
            continue;
        };
        if sent.op3 {
            if !url.is_empty() && !inventory::is_placeholder_address(url) {
                *url = op3_prefix(url, guid);
            }
        } else if current.op3 {
            *url = plain_address(url).into_owned();
        }
    }
}

/// Write `pubDate` from `pub_date_time` (ADR 0093 items 6 and 7). The step
/// compares the sent `pub_date_time` with the value that the core computes
/// from the draft `pubDate`. A sent value that is equal keeps the draft
/// `pubDate`, and a new item gets none. A sent time becomes RFC 2822 text
/// with the sent offset. A sent null removes `pubDate`. The step ignores
/// the sent `pub_date`. It applies to each item and to the item of each
/// live item.
fn apply_pub_date_time(current: &Feed, sent: &mut Feed) -> Result<(), String> {
    let items = sent
        .items
        .iter_mut()
        .map(|e| {
            (
                e.key().and_then(|k| current.items.get(k)).map(|c| &c.value),
                &mut e.value,
            )
        })
        .chain(sent.live_items.iter_mut().map(|e| {
            (
                e.key()
                    .and_then(|k| current.live_items.get(k))
                    .map(|c| &c.item),
                &mut e.value.item,
            )
        }));
    for (before, item) in items {
        let expected = before.and_then(|c| c.pub_date_time.as_ref());
        if item.pub_date_time.as_ref() == expected {
            item.pub_date = before.and_then(|c| c.pub_date.clone());
        } else if let Some(text) = &item.pub_date_time {
            let moment = crate::datetime::parse_rfc3339(text)
                .ok_or("pub_date_time is not an RFC 3339 time with a zone")?;
            item.pub_date = Some(crate::datetime::format_rfc2822(moment));
        } else {
            item.pub_date = None;
        }
    }
    Ok(())
}

/// Write `itunes:duration` from `duration_seconds` (ADR 0094 item 2). The
/// step compares the sent `duration_seconds` with the value that the core
/// reads from the draft `itunes:duration`. A sent value that is equal keeps
/// the draft `itunes:duration`, byte for byte, and a new item gets none. A
/// different number becomes `HH:MM:SS` text. A sent null removes
/// `itunes:duration` when the draft text has a value that the core reads.
/// A draft text that the core cannot read stays. The step ignores the sent
/// `duration`. It applies to each item and to the item of each live item.
fn apply_duration_seconds(current: &Feed, sent: &mut Feed) {
    let items = sent
        .items
        .iter_mut()
        .map(|e| {
            (
                e.key().and_then(|k| current.items.get(k)).map(|c| &c.value),
                &mut e.value,
            )
        })
        .chain(sent.live_items.iter_mut().map(|e| {
            (
                e.key()
                    .and_then(|k| current.live_items.get(k))
                    .map(|c| &c.item),
                &mut e.value.item,
            )
        }));
    for (before, item) in items {
        let expected = before.and_then(|c| c.duration_seconds);
        item.duration = if item.duration_seconds == expected {
            before.and_then(|c| c.duration.clone())
        } else {
            item.duration_seconds.map(crate::duration::format_duration)
        };
    }
}

/// Keep the form of the cover when the sent `image` is a new address
/// (ADR 0060 items 3 to 5, ADR 0065 item 6). The step does nothing when
/// `image` does not change. An empty `image` removes `image` and
/// `rss_image`. A feed with only the RSS image keeps only that tag. A feed
/// with both tags gets the address in both tags. Else the address stays in
/// `image`. The step compares the texts as they are.
fn keep_cover_form(current: &Feed, sent: &mut Feed) {
    if sent.image == current.image {
        return;
    }
    let Some(address) = sent.image.clone().filter(|a| !a.is_empty()) else {
        sent.image = None;
        sent.rss_image = None;
        return;
    };
    if current.rss_image.is_none() {
        return;
    }
    let mut rss = sent
        .rss_image
        .take()
        .or_else(|| current.rss_image.clone())
        .unwrap_or_default();
    rss.url = Some(address);
    sent.rss_image = Some(rss);
    if current.image.is_none() {
        sent.image = None;
    }
}

/// Keep the form of each `itunes:explicit` text (ADR 0064 item 4). The UI
/// sends `true` or `false`. A sent value that differs from the draft value
/// becomes `yes` or `no` when the draft value is `yes` or `no` in any letter
/// case, after a trim. A capital first letter of the draft value stays a
/// capital letter. In each other case the sent value stays. A new item has
/// no draft item, so it keeps its sent value.
fn keep_explicit_form(current: &Feed, sent: &mut Feed) {
    keep_form(current.explicit.as_deref(), &mut sent.explicit);
    for entry in &mut sent.items {
        let before = entry.key().and_then(|k| current.items.get(k));
        keep_form(
            before.and_then(|c| c.explicit.as_deref()),
            &mut entry.value.explicit,
        );
    }
}

fn keep_form(draft: Option<&str>, sent: &mut Option<String>) {
    let Some(draft) = draft else { return };
    let Some(value) = sent.as_deref() else { return };
    if value == draft {
        return;
    }
    let word = match value {
        "true" => "yes",
        "false" => "no",
        _ => return,
    };
    let trimmed = draft.trim();
    if !trimmed.eq_ignore_ascii_case("yes") && !trimmed.eq_ignore_ascii_case("no") {
        return;
    }
    let mut word = word.to_owned();
    if trimmed.starts_with(|c: char| c.is_ascii_uppercase()) {
        word[..1].make_ascii_uppercase();
    }
    *sent = Some(word);
}

/// Give each changed Nostr key the `npub` form (ADR 0074 items 4 and 5). A
/// key field is the `npub` of a person of the channel, of an item or of a
/// live item, or the text of a channel `podcast:txt` with `purpose` `npub`.
/// A value that equals the value of the same entry in the draft does not
/// change. A new entry has no draft value. For each other value,
/// [`npub_text`] gives the new value. When it gives `None`, the value stays
/// as the user typed it, and `Feed::bad_keys` lists it.
fn keep_key_form(current: &Feed, sent: &mut Feed) {
    fn form(before: Option<&str>, value: &mut String) {
        if before == Some(value.as_str()) {
            return;
        }
        if let Some(npub) = npub_text(value) {
            *value = npub;
        }
    }
    fn persons(before: Option<&[Entry<Person>]>, sent: &mut [Entry<Person>]) {
        for entry in sent {
            let old = entry
                .key()
                .and_then(|k| before.and_then(|list| list.get(k)))
                .and_then(|p| p.npub.as_deref());
            if let Some(value) = entry.value.npub.as_mut() {
                form(old, value);
            }
        }
    }
    persons(Some(&current.persons), &mut sent.persons);
    for entry in &mut sent.txts {
        if entry.value.purpose.as_deref() != Some("npub") {
            continue;
        }
        let old = entry
            .key()
            .and_then(|k| current.txts.get(k))
            .map(|t| t.value.value.as_str());
        form(old, &mut entry.value.value);
    }
    for entry in &mut sent.items {
        let before = entry.key().and_then(|k| current.items.get(k));
        persons(
            before.map(|i| i.persons.as_slice()),
            &mut entry.value.persons,
        );
    }
    for entry in &mut sent.live_items {
        let before = entry.key().and_then(|k| current.live_items.get(k));
        persons(
            before.map(|l| l.item.persons.as_slice()),
            &mut entry.value.item.persons,
        );
    }
}

/// Write the role and the group of a credit with a role of the list
/// (ADR 0112 item 4). The step acts on a new credit, and on a credit whose
/// `role` or `group` differs from the credit with the same key in the draft.
/// For such a credit with a role of the list, a sent group that is a group
/// of the role stays. Else the group is the first group of the role. The
/// role and the group become lowercase text. A credit with a role that is
/// not of the list keeps its role and its group, in lowercase text
/// (ADR 0112 item 5). A credit that does not change keeps
/// its bytes. The step applies to the channel, to each item and to each
/// live item.
fn apply_role_group(current: &Feed, sent: &mut Feed) {
    fn persons(before: Option<&[Entry<Person>]>, sent: &mut [Entry<Person>]) {
        for entry in sent {
            let old = entry
                .key()
                .and_then(|k| before.and_then(|list| list.get(k)));
            if old.is_some_and(|o| o.role == entry.value.role && o.group == entry.value.group) {
                continue;
            }
            let Some(role) = entry.value.role.as_deref() else {
                continue;
            };
            let groups = roles::groups_of(role);
            // A role that is not of the list keeps its text and its group,
            // in the form of the wire (ADR 0112 item 5).
            let Some(first) = groups.first() else {
                let role = roles::wire(role);
                let group = entry.value.group.as_deref().map(roles::wire);
                entry.value.role = Some(role);
                entry.value.group = group;
                continue;
            };
            let group = entry
                .value
                .group
                .as_deref()
                .and_then(|g| groups.iter().find(|name| roles::is_group(g, name)))
                .unwrap_or(first);
            let (role, group) = (roles::wire(role), roles::wire(group));
            entry.value.role = Some(role);
            entry.value.group = Some(group);
        }
    }
    persons(Some(&current.persons), &mut sent.persons);
    for entry in &mut sent.items {
        let before = entry.key().and_then(|k| current.items.get(k));
        persons(
            before.map(|i| i.persons.as_slice()),
            &mut entry.value.persons,
        );
    }
    for entry in &mut sent.live_items {
        let before = entry.key().and_then(|k| current.live_items.get(k));
        persons(
            before.map(|l| l.item.persons.as_slice()),
            &mut entry.value.item.persons,
        );
    }
}

/// Give each track its position as its `podcast:episode` and its
/// `itunes:episode` when the sent model changes the sequence of the tracks
/// (ADR 0024, ADR 0079). A `pubDate` does not change.
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
        item.itunes_episode = Some(number.to_string());
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

/// Each address that the draft gives for the channel, for a track or for a
/// live item: the `url` of an enclosure, the `uri` of a `podcast:source`,
/// the `url` of the channel `image`, or the `href` of `itunes:image` of the
/// channel or of an item (ADR 0032 item 10, ADR 0033 item 7, ADR 0036
/// item 7 and item 10, ADR 0060). The recording of
/// a live item is a file of the feed, as the audio of a track is (ADR 0054
/// item 6).
#[must_use]
pub fn feed_addresses(feed: &Feed) -> Vec<Cow<'_, str>> {
    let mut urls: Vec<Cow<'_, str>> = Vec::new();
    urls.extend(feed.image.as_deref().map(plain_address));
    urls.extend(
        feed.rss_image
            .as_ref()
            .and_then(|i| i.url.as_deref())
            .map(plain_address),
    );
    let items = feed.items.iter().map(|entry| &entry.value);
    let live_items = feed.live_items.iter().map(|entry| &entry.value.item);
    for item in items.chain(live_items) {
        urls.extend(
            item.enclosure
                .as_ref()
                .and_then(|enclosure| enclosure.url.as_deref())
                .map(plain_address),
        );
        urls.extend(item.image.as_deref().map(plain_address));
        for enclosure in &item.alternate_enclosures {
            for source in &enclosure.sources {
                urls.extend(source.uri.as_deref().map(plain_address));
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
    diff_at(collection, hosts, library, id, unix_now())
}

/// The time of the computer, in seconds after the Unix epoch.
fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// The same as [`diff`], with the time `now` in seconds after the Unix
/// epoch. The diff gives the planned Nostr events of each host. The
/// collection keeps the plan time, so that a later diff of the same ID
/// gives the same events (ADR 0050 item 10).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the feed has no draft, or a host cannot
/// plan its events.
pub fn diff_at(
    collection: &Collection,
    hosts: &Hosts,
    library: &Library,
    id: FeedId,
    now: u64,
) -> Result<Diff, PipelineError> {
    let feed_row = collection.feed(id)?;
    let draft = one_cover(current_draft(collection, id)?)?;
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    let (operations, blocked) = host_operations(collection, library, hosts, &feed_row, &feed)?;
    let feed_move = feed_move(collection, hosts, &feed_row)?;
    let mut extra = operation_bytes(collection, hosts, &operations)?;
    extra.extend(feed_move_bytes(feed_move.as_ref()));
    let (announcements, announce_skip) =
        plan_announcements(collection, hosts, &feed_row, feed_move.as_ref())?;
    extra.extend(announce_bytes(&announcements, announce_skip));
    let grants = draft_grant_list(collection, id, &feed)?;
    extra.extend(grant_bytes(&grant_apps(collection, id, &grants)?));
    let files = upload_files(collection, &operations)?;
    let diff_id = diff_id(&draft, &extra);
    let changes = host_changes(
        collection,
        library,
        &feed_row,
        &draft,
        &operations,
        feed_move.as_ref(),
    )?;
    let plan_time = match collection.signing_plan(id)? {
        Some((kept_id, plan_time))
            if kept_id == diff_id.as_str()
                && plan_time + PLAN_LIFETIME - PLAN_REUSE_MARGIN > now =>
        {
            plan_time
        }
        _ => {
            collection.put_signing_plan(id, diff_id.as_str(), now)?;
            now
        }
    };
    let mut events = Vec::new();
    for (connector_id, host_changes) in &changes {
        for planned in hosts
            .get(*connector_id)?
            .plan_events(host_changes, plan_time)?
        {
            events.push(DiffEvent {
                connector_id: *connector_id,
                user_signs: planned.user_signs,
                event: planned.event,
            });
        }
    }
    let base = String::from_utf8_lossy(&draft.base_xml);
    let new = String::from_utf8_lossy(&draft.draft_xml);
    let unified = TextDiff::from_lines(base.as_ref(), new.as_ref())
        .unified_diff()
        .context_radius(3)
        .header("live", "draft")
        .to_string();
    let review = crate::review::review(&draft.base_xml, &draft.draft_xml);
    Ok(Diff {
        id: diff_id,
        unified,
        changed: draft.base_xml != draft.draft_xml,
        files,
        operations,
        blocked,
        feed_move,
        announcements,
        announce_skip,
        events,
        grants,
        fields: review.fields,
        lines: review.lines,
        uncovered: review.uncovered,
    })
}

/// The changes of a write on each host, in the sequence of the host IDs
/// (ADR 0050 item 11). A confirmation gives no change.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails, or a library file
/// has no SHA-256 and Studio cannot read it.
fn host_changes(
    collection: &Collection,
    library: &Library,
    row: &FeedRow,
    draft: &DraftRow,
    operations: &[HostOperation],
    feed_move: Option<&FeedMove>,
) -> Result<BTreeMap<ConnectorId, HostChanges>, PipelineError> {
    let mut changes: BTreeMap<ConnectorId, HostChanges> = BTreeMap::new();
    for operation in operations {
        match operation.kind {
            OperationKind::Upload | OperationKind::Copy => {
                let library_file_id = operation
                    .library_file_id
                    .expect("an upload or a copy names its library file");
                let file = collection.library_file(LibraryFileId(library_file_id))?;
                let sha256 = match file.sha256 {
                    Some(sha256) => sha256,
                    // A remote file has no bytes yet. The write refuses it.
                    None if !file.local => continue,
                    None => {
                        let folder = row.folder.as_deref().ok_or_else(|| {
                            PipelineError::new(
                                PipelineErrorKind::NoFolder,
                                format!("feed {} has no release folder yet", row.id.0),
                            )
                        })?;
                        crate::digest::sha256_file(&library.path(folder, &file.path)).map_err(
                            |error| {
                                PipelineError::new(
                                    PipelineErrorKind::Connector,
                                    format!("{}: cannot read the library file: {error}", file.path),
                                )
                            },
                        )?
                    }
                };
                changes
                    .entry(operation.connector_id)
                    .or_default()
                    .writes
                    .push(PlannedWrite {
                        path: operation.path.clone(),
                        sha256,
                        size: file.size,
                    });
            }
            OperationKind::Delete => {
                let sha256 = collection
                    .host_file(operation.connector_id, &operation.path)?
                    .and_then(|file| file.sha256);
                changes
                    .entry(operation.connector_id)
                    .or_default()
                    .deletes
                    .push(PlannedDelete {
                        path: operation.path.clone(),
                        sha256,
                    });
            }
            OperationKind::Confirm => {}
        }
    }
    // The feed XML goes to the host of the feed, or to the new host of a
    // feed move. A feed with no host and no move has no feed XML write.
    let from = writable(row).ok();
    let Some((feed_host, feed_path)) = feed_move
        .map(|m| (m.connector_id, m.path.as_str()))
        .or(from)
    else {
        return Ok(changes);
    };
    changes
        .entry(feed_host)
        .or_default()
        .writes
        .push(planned_write(feed_path, &draft.draft_xml));
    // The pointer file stays on the host from before.
    if let Some(moved) = feed_move
        && moved.method == FeedMoveMethod::Pointer
        && let Some((from_id, from_path)) = from
        && let Some(new_url) = moved.url.as_deref()
    {
        let pointer = pointer_bytes(draft, new_url)?;
        changes
            .entry(from_id)
            .or_default()
            .writes
            .push(planned_write(from_path, &pointer));
    }
    Ok(changes)
}

/// A planned write of `bytes` to `path`.
fn planned_write(path: &str, bytes: &[u8]) -> PlannedWrite {
    PlannedWrite {
        path: path.to_owned(),
        sha256: sha256_hex(&[bytes]),
        size: bytes.len() as u64,
    }
}

/// The move of the feed XML in the draft of a feed, with its new address
/// and its method. Gives `None` when the draft has no move, or the move
/// names the host of the feed (ADR 0038 items 16 and 18).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection or a host fails.
fn feed_move(
    collection: &Collection,
    hosts: &Hosts,
    row: &FeedRow,
) -> Result<Option<FeedMove>, PipelineError> {
    let Some(DraftFeedMove { connector_id, path }) = collection.draft_feed_move(row.id)? else {
        return Ok(None);
    };
    if row.connector_id.is_none() {
        // A feed with no host: Studio writes nothing at the old address.
        return Ok(Some(FeedMove {
            connector_id,
            url: hosts.get(connector_id)?.public_url(&path),
            path,
            from_url: row.public_url.clone(),
            method: FeedMoveMethod::None,
            replaces_pointer: false,
        }));
    }
    let (from_id, from_path) = writable(row)?;
    if connector_id == from_id {
        return Ok(None);
    }
    let from = hosts.get(from_id)?;
    let method = if from.can_redirect() {
        FeedMoveMethod::Redirect
    } else {
        FeedMoveMethod::Pointer
    };
    let replaces_pointer = collection.is_pointer_of_feed(connector_id, &path, row.id)?;
    Ok(Some(FeedMove {
        connector_id,
        url: hosts.get(connector_id)?.public_url(&path),
        path,
        from_url: from
            .public_url(from_path)
            .or_else(|| row.public_url.clone()),
        method,
        replaces_pointer,
    }))
}

/// The bytes that [`diff_id`] adds for a feed move: its host, its path and
/// its method. The bytes are empty when there is no move.
fn feed_move_bytes(feed_move: Option<&FeedMove>) -> Vec<u8> {
    let Some(feed_move) = feed_move else {
        return Vec::new();
    };
    let mut bytes = Vec::new();
    bytes.push(0);
    bytes.extend_from_slice(b"feed-move");
    bytes.push(0);
    bytes.extend_from_slice(feed_move.connector_id.0.to_string().as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(feed_move.path.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(feed_move.method.as_str().as_bytes());
    // Only a move that replaces a pointer file adds this value, so the
    // diff ID of any other move does not change (ADR 0126 item 21).
    if feed_move.replaces_pointer {
        bytes.push(0);
        bytes.extend_from_slice(b"replaces-pointer");
    }
    bytes
}

/// The addresses that a write announces, before the first change on a host
/// (ADR 0052 items 7 and 8). Without a feed move, it is the public address
/// of the feed file. With a feed move, it is the new address and then the
/// address from before. An address that is not known goes out.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection or the host set fails.
fn announce_addresses(
    collection: &Collection,
    hosts: &Hosts,
    row: &FeedRow,
    feed_move: Option<&FeedMove>,
) -> Result<Vec<String>, PipelineError> {
    if let Some(moved) = feed_move {
        return Ok([moved.url.clone(), moved.from_url.clone()]
            .into_iter()
            .flatten()
            .collect());
    }
    // A feed with no host has only the address of its row.
    let url = match writable(row) {
        Ok(host) => written_feed_url(collection, hosts, row.id, host, None)?,
        Err(_) => row.public_url.clone(),
    };
    Ok(url.into_iter().collect())
}

/// The announcements of a write. [`diff_at`] and [`write_draft_signed`]
/// both call this function, so that they give the same plan (ADR 0052
/// item 10).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection or the host set fails, or
/// a setting of the announcements is not a boolean.
fn plan_announcements(
    collection: &Collection,
    hosts: &Hosts,
    row: &FeedRow,
    feed_move: Option<&FeedMove>,
) -> Result<(Vec<PlannedAnnouncement>, Option<AnnounceSkip>), PipelineError> {
    let settings = AnnounceSettings::load(collection)?;
    let addresses = announce_addresses(collection, hosts, row, feed_move)?;
    Ok(announce::plan(&settings, &addresses))
}

/// The bytes that [`diff_id`] adds for the announcements: each address with
/// its podping flag, and the reason for none. A change of a setting after
/// the diff changes the ID.
fn announce_bytes(planned: &[PlannedAnnouncement], skip: Option<AnnounceSkip>) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.push(0);
    bytes.extend_from_slice(b"announce");
    for item in planned {
        bytes.push(0);
        bytes.extend_from_slice(item.url.as_bytes());
        bytes.push(0);
        bytes.push(if item.podping { b'1' } else { b'0' });
    }
    if let Some(skip) = skip {
        bytes.push(0);
        bytes.extend_from_slice(skip.as_str().as_bytes());
    }
    bytes
}

/// The `guid` of each live item of `feed` that gets a show grant at the
/// write: each grant of the draft for a `pending` live item of `feed`, in
/// the sequence of the live items (ADR 0054 item 11). [`diff_at`] and
/// [`write_draft_signed`] both call this function.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails.
fn draft_grant_list(
    collection: &Collection,
    id: FeedId,
    feed: &Feed,
) -> Result<Vec<String>, PipelineError> {
    let asked: HashSet<String> = collection.draft_grants(id)?.into_iter().collect();
    let mut grants: Vec<String> = Vec::new();
    for entry in &feed.live_items {
        let Some(guid) = entry.value.item.guid.as_deref() else {
            continue;
        };
        if asked.contains(guid)
            && grant::is_pending_live_item(feed, guid)
            && !grants.iter().any(|g| g == guid)
        {
            grants.push(guid.to_owned());
        }
    }
    Ok(grants)
}

/// The app that each grant of `grants` names (ADR 0110 item 19).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails.
fn grant_apps(
    collection: &Collection,
    id: FeedId,
    grants: &[String],
) -> Result<Vec<(String, Option<i64>)>, PipelineError> {
    grants
        .iter()
        .map(|guid| Ok((guid.clone(), collection.draft_grant_app(id, guid)?)))
        .collect()
}

/// Give [`PipelineErrorKind::AppNotActive`] when an app of `grants` is not
/// active now, or has no live value origin (ADR 0110 items 19 and 25).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails or an app is not
/// active.
fn ensure_grant_apps_active(
    collection: &Collection,
    grants: &[(String, Option<i64>)],
) -> Result<(), PipelineError> {
    for (guid, app_id) in grants {
        let active = match app_id {
            Some(app_id) => crate::apps::app(collection, *app_id)?.is_some_and(|app| {
                app.state == crate::apps::AppState::Active && app.live_value_origin.is_some()
            }),
            None => false,
        };
        if !active {
            return Err(PipelineError::new(
                PipelineErrorKind::AppNotActive,
                format!(
                    "ADR 0110 item 19: the show grant of {guid} names an app that is not active. Select an active app for the grant, or remove the grant"
                ),
            ));
        }
    }
    Ok(())
}

/// The bytes that [`diff_id`] adds for the show grants, in the form of
/// [`announce_bytes`]. A change of a grant of the draft, or of its app, after
/// the diff changes the ID (ADR 0054 item 11, ADR 0110 item 19).
fn grant_bytes(grants: &[(String, Option<i64>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.push(0);
    bytes.extend_from_slice(b"grants");
    for (guid, app_id) in grants {
        bytes.push(0);
        bytes.extend_from_slice(guid.as_bytes());
        bytes.push(0);
        if let Some(app_id) = app_id {
            bytes.extend_from_slice(app_id.to_string().as_bytes());
        }
    }
    bytes
}

/// Put the move of the feed XML to `target` into the draft, and set the
/// `self` link of the draft to the new address, when the draft has a
/// `self` link (ADR 0038 item 16). The path stays the path from before.
///
/// The host of the feed gives no move. The function then removes the move,
/// and puts the `self` link and the tag back to those of the live file.
///
/// A feed with no connector has no host to name. Its path is the release
/// folder, `/`, and the file name of its public address (ADR 0067
/// item 7).
///
/// The feed then gets its real GUID through [`assign_real_guids`].
/// `library` gives the hosted copies that name the feeds with a link to a
/// placeholder. With `None`, a written feed with a real GUID keeps its link.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::FileExists`] when the inventory of the new
/// host has a file at the path, and [`PipelineErrorKind::NoPublicAddress`]
/// when the new host gives no public address for the path. Gives
/// [`PipelineErrorKind::NotFound`] when the feed has no hosted file yet,
/// or a feed with no connector has no release folder or no public
/// address. The draft then does not change.
pub fn set_feed_host(
    collection: &Collection,
    hosts: &Hosts,
    library: Option<&Library>,
    id: FeedId,
    target: ConnectorId,
) -> Result<(), PipelineError> {
    let row = collection.feed(id)?;
    let (from_id, path) = if row.connector_id.is_some() {
        let (from_id, path) = writable(&row)?;
        (Some(from_id), path.to_owned())
    } else {
        let (Some(folder), Some(file_name)) = (row.folder.as_deref(), public_file_name(&row))
        else {
            return Err(PipelineError::new(
                PipelineErrorKind::NotFound,
                format!("feed {} has no release folder or no public address", id.0),
            ));
        };
        (None, format!("{folder}/{file_name}"))
    };
    let path = path.as_str();
    let draft = current_draft(collection, id)?;
    if Some(target) == from_id {
        remove_draft_move(collection, id, &draft)?;
        return Ok(());
    }
    if draft.base_version.is_none() {
        return Err(PipelineError::new(
            PipelineErrorKind::NotFound,
            format!("{path} is not on its file host yet, so there is no feed XML to move"),
        ));
    }
    // ADR 0126 items 19 and 21: a placed file or a pointer file of this
    // feed is not a different file.
    if collection.host_file(target, path)?.is_some()
        && !collection.is_placed_feed_file(target, path, id)?
        && !collection.is_pointer_of_feed(target, path, id)?
    {
        return Err(PipelineError::new(
            PipelineErrorKind::FileExists,
            format!("a file exists at {path} on the new file host already"),
        ));
    }
    let url = hosts.get(target)?.public_url(path).ok_or_else(|| {
        PipelineError::new(
            PipelineErrorKind::NoPublicAddress,
            format!(
                "{path} has no public address on the new file host; give the file host a public address first"
            ),
        )
    })?;
    collection.set_draft_feed_move(id, target, path)?;
    // ADR 0128 item 1: the feed names its own new URL by the tag, also
    // when it has no self link. A tag of the draft gets the new value.
    edit_draft(collection, id, |feed| {
        feed.set_self_url(&url);
        feed.new_feed_url = Some(url.clone());
    })?;
    // The new file host has a public address, so the feed gets its real
    // GUID (ADR 0122 item 8).
    assign_real_guids(collection, hosts, library, &[id])?;
    Ok(())
}

/// Remove the feed move of a draft, and put back the links that the move
/// set: the `self` link of the base, when the base has one, and, when the
/// draft had a move, the `itunes:new-feed-url` tag of the base, or no tag
/// when the base has none (ADR 0128 item 4).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the draft is not a feed.
fn remove_draft_move(
    collection: &Collection,
    id: FeedId,
    draft: &DraftRow,
) -> Result<(), PipelineError> {
    let had_move = collection.draft_feed_move(id)?.is_some();
    collection.remove_draft_feed_move(id)?;
    let base = Feed::read(&Document::parse_bytes(&draft.base_xml)?).ok();
    let back = base
        .as_ref()
        .and_then(|base| base.self_url().map(str::to_owned));
    let tag = base.and_then(|base| base.new_feed_url);
    if back.is_some() || had_move {
        let no_tag = tag.is_none();
        let mut restored = edit_draft(collection, id, |feed| {
            if let Some(back) = &back {
                feed.set_self_url(back);
            }
            if had_move {
                feed.new_feed_url = tag;
            }
        })?;
        // The move can add the `xmlns:itunes` declaration for the tag. A
        // base with no tag then gets its root element back (ADR 0128 item 4).
        if had_move && no_tag {
            let base_doc = Document::parse_bytes(&draft.base_xml)?;
            let mut doc = Document::parse_bytes(&restored.draft_xml)?;
            if drop_added_declarations(&mut doc, base_doc.root()) {
                restored.draft_xml = doc.write().into_bytes();
                collection.save_draft(id, &restored)?;
            }
        }
    }
    Ok(())
}

/// Remove each `xmlns:` declaration of the root element of `doc` that
/// `base` does not have, when no element or attribute of `doc` uses its
/// prefix. Gives true when a declaration went out.
fn drop_added_declarations(doc: &mut Document, base: &Element) -> bool {
    fn uses(element: &Element, prefix: &str) -> bool {
        let start = format!("{prefix}:");
        element.name().starts_with(&start)
            || element
                .attributes()
                .iter()
                .any(|a| a.name().starts_with(&start))
            || element.elements().any(|child| uses(child, prefix))
    }
    let added: Vec<String> = doc
        .root()
        .attributes()
        .iter()
        .filter_map(|a| {
            let prefix = a.name().strip_prefix("xmlns:")?;
            (base.attribute(a.name()).is_none() && !uses(doc.root(), prefix))
                .then(|| a.name().to_owned())
        })
        .collect();
    for name in &added {
        doc.root_mut().set_attribute(name, None);
    }
    !added.is_empty()
}

/// Remove the host choice from the draft of a feed with no connector, and
/// put the `self` link of the draft back to the link of the base (ADR 0067
/// item 7).
///
/// # Errors
///
/// Gives [`PipelineErrorKind::Conflict`] when the feed has a connector, and
/// [`PipelineErrorKind::NoDraft`] when it has no draft.
pub fn clear_feed_host(collection: &Collection, id: FeedId) -> Result<(), PipelineError> {
    let row = collection.feed(id)?;
    if row.connector_id.is_some() {
        return Err(PipelineError::new(
            PipelineErrorKind::Conflict,
            format!(
                "feed {} has a file host; choose its file host with set_feed_host",
                id.0
            ),
        ));
    }
    let draft = current_draft(collection, id)?;
    remove_draft_move(collection, id, &draft)
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
    hosted_copy(library, row)
        .is_some_and(|feed| feed_addresses(&feed).iter().any(|address| address == url))
}

/// The hosted copy of `row` in its release folder, as the feed that Studio
/// last put there. Gives `None` when the row has no release folder, or the
/// copy is absent or is not a feed.
fn hosted_copy(library: &Library, row: &FeedRow) -> Option<Feed> {
    let (folder, path) = (row.folder.as_deref()?, row.path.as_deref()?);
    let file_name = path.rsplit_once('/').map_or(path, |(_, file)| file);
    let bytes = std::fs::read(library.path(folder, file_name)).ok()?;
    let doc = Document::parse_bytes(&bytes).ok()?;
    Feed::read(&doc).ok()
}

/// True when the hosted copy of `row` names a placeholder GUID in a
/// `remoteItem` of the feed or of its publisher (ADR 0122 item 8). A feed
/// with no hosted copy gives `false`.
fn hosted_copy_names_placeholder(library: &Library, row: &FeedRow) -> bool {
    let Some(feed) = hosted_copy(library, row) else {
        return false;
    };
    let publisher_items = feed
        .publisher
        .iter()
        .flat_map(|links| links.remote_items.iter());
    feed.remote_items
        .iter()
        .chain(publisher_items)
        .any(|entry| {
            entry
                .value
                .feed_guid
                .as_deref()
                .is_some_and(has_placeholder_guid)
        })
}

/// True when the draft of `row` gives `url` as an address. A copy of an
/// album has a draft but no hosted file. Its draft keeps the files of its
/// source in use (ADR 0077 item 8). A feed with no draft gives `false`.
fn feed_draft_refers(collection: &Collection, row: &FeedRow, url: &str) -> bool {
    draft_model(collection, row.id)
        .is_ok_and(|(feed, _)| feed_addresses(&feed).iter().any(|address| address == url))
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
    // ADR 0105 item 6: with `off`, the copy of a file that a host has reads
    // the file from that host, so it does not wait for a download.
    let from_host = !file.local
        && file.removed_at.is_none()
        && collection.asset_rules().is_some()
        && crate::assets::recorded_copy(collection, &file)
            .ok()
            .flatten()
            .is_some();
    let waiting = (!file.local && !from_host) || file.sha256.is_none();
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

/// The host operation of a draft confirmation. It does no write and no
/// deletion. It only makes the write request the file again
/// (ADR 0043 item 9).
///
/// The size is the size of the library file, so that the check of the
/// write compares the host file with the library. A confirmation with no
/// library file uses the size in the inventory.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection or a host fails.
fn confirm_operation(
    collection: &Collection,
    hosts: &Hosts,
    operation: &crate::collection::DraftOperation,
) -> Result<HostOperation, PipelineError> {
    let size = match operation.library_file_id {
        Some(file) => collection.library_file(file)?.size,
        None => collection
            .host_file(operation.connector_id, &operation.path)?
            .map_or(0, |file| file.size),
    };
    Ok(HostOperation {
        kind: OperationKind::Confirm,
        connector_id: operation.connector_id,
        path: operation.path.clone(),
        size,
        library_file_id: operation.library_file_id.map(|file| file.0),
        url: hosts
            .get(operation.connector_id)?
            .public_url(&operation.path),
        operation_id: Some(operation.id.0),
        waiting: false,
    })
}

/// Each host operation of the draft of a feed, and each deletion that
/// Studio refuses because a feed refers to the file (ADR 0036 items 10,
/// 12, 13 and 14).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection or a host fails.
#[expect(
    clippy::too_many_lines,
    reason = "the uploads and the draft operations are two loops of one plan"
)]
fn host_operations(
    collection: &Collection,
    library: &Library,
    hosts: &Hosts,
    feed_row: &FeedRow,
    feed: &Feed,
) -> Result<(Vec<HostOperation>, Vec<BlockedDelete>), PipelineError> {
    let urls = feed_addresses(feed);
    let mut operations = Vec::new();
    let mut blocked = Vec::new();

    // A feed plans its files for the host of its draft feed move (ADR 0067
    // item 6, ADR 0122 item 15). A feed with a connector also keeps the
    // planned addresses of its own host, because a track imported before the
    // move has them. A file that the draft refers to at a planned address is
    // uploaded to the host of that address.
    let moved_row;
    let mut planning_rows: Vec<&FeedRow> = Vec::new();
    match collection.draft_feed_move(feed_row.id)? {
        Some(DraftFeedMove { connector_id, path }) => {
            moved_row = FeedRow {
                files_connector_id: Some(connector_id),
                path: Some(path),
                ..feed_row.clone()
            };
            planning_rows.push(&moved_row);
            if feed_row.connector_id.is_some() {
                planning_rows.push(feed_row);
            }
        }
        None => planning_rows.push(feed_row),
    }

    // Each library file whose planned address is in the draft, with no
    // copy of the same bytes at its planned path yet, is an upload
    // (ADR 0036 item 10). A draft that refers to a copy on a different
    // host uses that copy, so it needs no upload.
    for file in collection.library_files(feed_row.id)? {
        for planning_row in &planning_rows {
            let planned = match inventory::planned_path(collection, hosts, planning_row, &file) {
                Ok(planned) => planned,
                Err(error)
                    if error.kind() == InventoryErrorKind::NoHost
                        && feed_row.connector_id.is_none() =>
                {
                    continue;
                }
                Err(error) => return Err(error.into()),
            };
            let planned_in_use = planned
                .url
                .as_deref()
                .is_some_and(|url| urls.iter().any(|address| address == url));
            let already = operations.iter().any(|op: &HostOperation| {
                op.kind == OperationKind::Upload
                    && op.connector_id == planned.connector_id
                    && op.path == planned.path
            });
            // A remote file has no bytes in the library, so it is never an
            // upload (ADR 0038 item 1).
            if planned_in_use && !planned.on_host && file.sha256.is_some() && !already {
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
    }

    // Each draft operation gives a copy, or a deletion, or a blocked
    // deletion (ADR 0036 items 12 and 13).
    let all_feeds = collection.feeds()?;
    for operation in collection.draft_operations(feed_row.id)? {
        match operation.kind {
            DraftOperationKind::Copy => {
                if let Some(copy) = copy_operation(collection, hosts, &operation)? {
                    operations.push(copy);
                }
            }
            DraftOperationKind::Confirm => {
                operations.push(confirm_operation(collection, hosts, &operation)?);
            }
            DraftOperationKind::Delete => {
                let connector = hosts.get(operation.connector_id)?;
                let url = connector.public_url(&operation.path);
                let mut referring: Vec<String> = Vec::new();
                // The feed XML of a feed of the library is the feed itself.
                // A feed does not give its own address in `urls`, so this
                // check is necessary (ADR 0035 item 17).
                for other in &all_feeds {
                    if other.connector_id == Some(operation.connector_id)
                        && other.path.as_deref() == Some(operation.path.as_str())
                    {
                        referring.push(feed_title(other));
                    }
                }
                if let Some(url) = url.as_deref() {
                    if urls.iter().any(|address| address == url) {
                        referring.push(feed_title(feed_row));
                    }
                    for other in &all_feeds {
                        if other.id != feed_row.id
                            && (feed_refers_in_library(library, other, url)
                                || feed_draft_refers(collection, other, url))
                        {
                            referring.push(feed_title(other));
                        }
                    }
                }
                if collection.is_feed_pointer(operation.connector_id, &operation.path)? {
                    blocked.push(BlockedDelete {
                        connector_id: operation.connector_id,
                        path: operation.path.clone(),
                        feed_titles: Vec::new(),
                        operation_id: operation.id.0,
                        pointer: true,
                    });
                } else if referring.is_empty() {
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
                        pointer: false,
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
    import_file(
        collection,
        library,
        hosts,
        id,
        source,
        file_name,
        media_type,
        NameConflict::Refuse,
    )
}

/// Put the new audio file of a track into the release folder, as
/// [`import_into_library`] does, for a replacement of the audio of the
/// track (ADR 0040 items 2 and 3).
///
/// When a different file is at `file_name`, the new file gets the hash
/// name of ADR 0035 item 16 in the release folder, for example
/// `01 Song.3f9a2c1d.mp3`. Studio does not write over the file from
/// before.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::NoFolder`] when the feed has no release
/// folder yet. Gives [`PipelineErrorKind::FileExists`] when a different
/// file is at the hash name too. Gives a [`PipelineError`] when Studio
/// cannot read `source`, or cannot write the release folder, or the
/// collection fails.
pub fn import_replacement_into_library(
    collection: &Collection,
    library: &Library,
    hosts: &Hosts,
    id: FeedId,
    source: &Path,
    file_name: &str,
    media_type: &str,
) -> Result<ImportedFile, PipelineError> {
    import_file(
        collection,
        library,
        hosts,
        id,
        source,
        file_name,
        media_type,
        NameConflict::HashName,
    )
}

/// Check that one of the new files is the removed pending file at `path`:
/// it has the same name and the same SHA-256 (ADR 0105 item 10). The user
/// adds the file again with an upload. `files` gives the name and the
/// path of each file that the import will put in the library.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::NotFound`] when the feed has no removed
/// pending file at `path`. Gives [`PipelineErrorKind::FileExists`] when no
/// new file has the name and the SHA-256 of that file.
pub fn check_restore(
    collection: &Collection,
    id: FeedId,
    path: &str,
    files: &[(&str, &Path)],
) -> Result<(), PipelineError> {
    let row = collection
        .library_files(id)?
        .into_iter()
        .find(|file| file.path == path && !file.local && file.removed_at.is_some())
        .ok_or_else(|| {
            PipelineError::new(
                PipelineErrorKind::NotFound,
                format!("{path} is not a file that Studio removed"),
            )
        })?;
    let wrong = || {
        PipelineError::new(
            PipelineErrorKind::FileExists,
            format!(
                "this is not the file {path} that Studio removed: the name or the bytes differ"
            ),
        )
    };
    let (_, source) = files
        .iter()
        .find(|(name, _)| *name == path)
        .ok_or_else(wrong)?;
    let sha256 = crate::digest::sha256_file(source).map_err(|error| {
        PipelineError::new(
            PipelineErrorKind::Connector,
            format!("{}: cannot read the file: {error}", source.display()),
        )
    })?;
    if row.sha256.as_deref() == Some(sha256.as_str()) {
        Ok(())
    } else {
        Err(wrong())
    }
}

/// What an import does when a different file is at its name in the
/// release folder.
#[derive(Clone, Copy, PartialEq, Eq)]
enum NameConflict {
    /// Give [`PipelineErrorKind::FileExists`] (ADR 0036 item 5).
    Refuse,
    /// Use the hash name of ADR 0035 item 16 (ADR 0040 item 3).
    HashName,
}

#[allow(clippy::too_many_arguments)]
fn import_file(
    collection: &Collection,
    library: &Library,
    hosts: &Hosts,
    id: FeedId,
    source: &Path,
    file_name: &str,
    media_type: &str,
    conflict: NameConflict,
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

    let files = collection.library_files(id)?;
    let mut file_name = file_name.to_owned();
    if conflict == NameConflict::HashName
        && files
            .iter()
            .any(|file| file.path == file_name && file.sha256.as_deref() != Some(sha256.as_str()))
    {
        file_name = inventory::hash_name(&file_name, &sha256);
    }
    let file_name = file_name.as_str();
    let existing = files.into_iter().find(|file| file.path == file_name);
    let library_file_id = match existing {
        Some(file) if file.sha256.as_deref() == Some(sha256.as_str()) => {
            if file.removed_at.is_some() && !file.local {
                // ADR 0105 item 10: the user adds a removed pending file
                // again. The row gets its bytes, a new import time and no
                // removal time.
                library
                    .put_file(&folder, file_name, source)
                    .map_err(|error| {
                        PipelineError::new(PipelineErrorKind::Connector, error.to_string())
                    })?;
                let stamp = file_stamp(&library.path(&folder, file_name));
                collection.restore_library_file(file.id, stamp.as_deref())?;
            } else {
                // The same file, imported again. Studio keeps the one copy,
                // and removes the file that the caller sent (ADR 0036
                // item 5).
                let _ = std::fs::remove_file(source);
            }
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
            .map(Cow::into_owned)
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
/// check of each new address, then each deletion. With a feed move, the
/// feed file goes to the new host, then the redirect or the pointer file
/// goes to the host from before, and then the feed gets its new location
/// (ADR 0038 item 19). After the write, the
/// draft and its host operations go out. Studio then scans each host that
/// the write touched.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::DiffNotApproved`],
/// [`PipelineErrorKind::BadMoveLink`],
/// [`PipelineErrorKind::WaitingForDownload`],
/// [`PipelineErrorKind::BadAddress`], [`PipelineErrorKind::BadKey`] or
/// [`PipelineErrorKind::Conflict`] before any file changes on a host.
///
/// Gives [`PipelineErrorKind::FileExists`] when a file exists already at
/// the planned path of an upload or a copy. Gives
/// [`PipelineErrorKind::NotReachable`] when a new file does not pass the
/// check of its size or its public address, or when the public address of
/// the feed file fails. The feed file is already written in both cases.
/// In the second case, Studio deletes no file. The draft then starts from
/// the written feed file. It keeps each deletion, and it gets a
/// confirmation for each new file (ADR 0043 item 9).
///
/// Gives [`PipelineErrorKind::Conflict`] when a host file changed after
/// the scan, before its deletion. The feed file stays written.
pub fn write_draft(
    collection: &mut Collection,
    hosts: &Hosts,
    library: &Library,
    addresses: &dyn AddressCheck,
    reader: &dyn FeedReader,
    id: FeedId,
    approved: &DiffId,
) -> Result<Version, PipelineError> {
    write_draft_signed(
        collection,
        hosts,
        library,
        addresses,
        LinkCheck::Check(reader),
        id,
        approved,
        &[],
        unix_now(),
        None,
    )
}

/// The same as [`write_draft`], for a feed with no connector. The draft
/// needs a host choice. `web` reads the public address of the feed, so that
/// the network policy of the caller applies (ADR 0067 items 8 to 11).
///
/// # Errors
///
/// Gives the errors of [`write_draft_signed`].
#[expect(
    clippy::too_many_arguments,
    reason = "write_draft gives its arguments and the client of the public address"
)]
pub fn write_public_draft(
    collection: &mut Collection,
    hosts: &Hosts,
    library: &Library,
    addresses: &dyn AddressCheck,
    reader: &dyn FeedReader,
    web: &crate::web::Web,
    id: FeedId,
    approved: &DiffId,
) -> Result<Version, PipelineError> {
    write_draft_signed(
        collection,
        hosts,
        library,
        addresses,
        LinkCheck::Check(reader),
        id,
        approved,
        &[],
        unix_now(),
        Some(web),
    )
}

/// How a write treats the links of its draft (ADR 0126 stage 1).
#[derive(Clone, Copy)]
pub enum LinkCheck<'a> {
    /// Run stage 1. The reader reads each followed URL.
    Check(&'a dyn FeedReader),
    /// A granted write: it changes no link, makes no read and gets no
    /// refusal from stage 1 (ADR 0126 item 26). Only `write_granted` passes
    /// this value.
    GrantedWrite,
}

/// The URL of a draft (ADR 0126 item 4): the destination of the move, else
/// the public address of the feed (ADR 0021). Gives `None` for a feed on a
/// file host with no public address.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the host set fails.
fn draft_url(
    hosts: &Hosts,
    row: &FeedRow,
    feed_move: Option<&FeedMove>,
) -> Result<Option<String>, PipelineError> {
    if let Some(moved) = feed_move {
        return Ok(moved.url.clone());
    }
    if let Some(url) = &row.public_url {
        return Ok(Some(url.clone()));
    }
    Ok(match writable(row) {
        Ok((connector_id, path)) => hosts.get(connector_id)?.public_url(path),
        Err(_) => None,
    })
}

/// The owned facts of a draft that stage 1 of ADR 0126 examines. The self
/// link counts as changed when it differs from the self link of the base.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the host set fails.
fn link_inputs(
    hosts: &Hosts,
    row: &FeedRow,
    draft: &DraftRow,
    feed: &Feed,
    feed_move: Option<&FeedMove>,
) -> Result<OwnedDraftLinks, PipelineError> {
    let url = draft_url(hosts, row, feed_move)?;
    let base_self = Document::parse_bytes(&draft.base_xml)
        .ok()
        .and_then(|doc| Feed::read(&doc).ok())
        .and_then(|base| base.self_url().map(str::to_owned));
    let self_changed = feed.self_url() != base_self.as_deref();
    let destination = feed_move.and_then(|moved| moved.url.clone());
    Ok(OwnedDraftLinks {
        feed: feed.clone(),
        url,
        self_changed,
        destination,
    })
}

/// Run stage 1 of ADR 0126 on a draft.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the host set fails.
fn link_report(
    hosts: &Hosts,
    reader: &dyn FeedReader,
    row: &FeedRow,
    draft: &DraftRow,
    feed: &Feed,
    feed_move: Option<&FeedMove>,
) -> Result<LinkReport, PipelineError> {
    Ok(link_inputs(hosts, row, draft, feed, feed_move)?.report(reader))
}

/// The facts of the draft of a feed for the check of stage 1. The function
/// makes no network read, so a caller can hold a lock while it runs, and
/// then release the lock before [`OwnedDraftLinks::report`] reads the
/// network.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the feed has no draft, or the collection
/// or the host set fails.
pub fn link_inputs_for(
    collection: &Collection,
    hosts: &Hosts,
    id: FeedId,
) -> Result<OwnedDraftLinks, PipelineError> {
    let row = collection.feed(id)?;
    let draft = current_draft(collection, id)?;
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    let moved = feed_move(collection, hosts, &row)?;
    link_inputs(hosts, &row, &draft, &feed, moved.as_ref())
}

/// The refusals and the notices of stage 1 for the draft of a feed, for
/// Review (ADR 0126 items 6 and 13). The function changes nothing. It
/// makes the same reads as the start of a write.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the feed has no draft, or the collection
/// or the host set fails.
pub fn check_links(
    collection: &Collection,
    hosts: &Hosts,
    reader: &dyn FeedReader,
    id: FeedId,
) -> Result<LinkReport, PipelineError> {
    Ok(link_inputs_for(collection, hosts, id)?.report(reader))
}

/// Apply a repair of ADR 0126 item 20 to the draft of a feed. "Use the
/// Public Address" sets the self link to the URL of the draft. "Remove the
/// Tag" removes `itunes:new-feed-url`. The draft changes through the model,
/// so the diff shows only this change. The function writes no file.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::StaleRevision`] when `sent_revision` is not
/// the revision of the draft, and [`PipelineErrorKind::InvalidOperation`]
/// when the repair does not apply to the draft. The draft then does not
/// change. It also gives an error when the feed has no draft.
pub fn apply_repair(
    collection: &Collection,
    hosts: &Hosts,
    id: FeedId,
    repair: Repair,
    sent_revision: &str,
) -> Result<DraftRow, PipelineError> {
    let row = collection.feed(id)?;
    let draft = current_draft(collection, id)?;
    if revision(&draft) != sent_revision {
        return Err(PipelineError::new(
            PipelineErrorKind::StaleRevision,
            "the draft changed after this model was read; read the draft again",
        ));
    }
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    let moved = feed_move(collection, hosts, &row)?;
    let url = draft_url(hosts, &row, moved.as_ref())?;
    if !move_link::repair_applies(repair, &feed, url.as_deref()) {
        return Err(PipelineError::new(
            PipelineErrorKind::InvalidOperation,
            "this repair does not apply to the draft",
        ));
    }
    let changed = changed_draft(draft, |feed| match repair {
        Repair::UsePublicAddress => {
            if let Some(url) = &url {
                set_self_link(feed, url);
            }
        }
        Repair::RemoveTag => feed.new_feed_url = None,
    })?;
    collection.save_draft(id, &changed)?;
    Ok(changed)
}

/// The plan time for a write: the plan time of the feed when the plan has
/// the diff ID `approved` and has not expired at `now` (ADR 0050 item 12).
/// The write route makes the hosts with this time.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails.
pub fn plan_time_for_write(
    collection: &Collection,
    id: FeedId,
    approved: &DiffId,
    now: u64,
) -> Result<Option<u64>, PipelineError> {
    Ok(collection
        .signing_plan(id)?
        .filter(|(kept_id, plan_time)| {
            kept_id == approved.as_str() && now < plan_time + PLAN_LIFETIME - PLAN_WRITE_MARGIN
        })
        .map(|(_, plan_time)| plan_time))
}

/// Make the planned events of each host again, and check them before the
/// first change on a host (ADR 0050 item 12).
///
/// Each connector keeps the plan that it gets here for the steps of the
/// write.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::PlanExpired`] when a host has events and the
/// feed has no plan for `approved`, or the plan expired. Gives
/// [`PipelineErrorKind::SignatureMissing`] when `signed` has no event for
/// a planned event of the user, and [`PipelineErrorKind::SignatureInvalid`]
/// when its signature or its public key is not correct.
fn check_signing_plan(
    collection: &Collection,
    hosts: &Hosts,
    changes: &BTreeMap<ConnectorId, HostChanges>,
    id: FeedId,
    approved: &DiffId,
    signed: &[nostr::prelude::Event],
    now: u64,
) -> Result<(), PipelineError> {
    let kept = plan_time_for_write(collection, id, approved, now)?;
    let mut events = Vec::new();
    for (connector_id, host_changes) in changes {
        let planned = hosts
            .get(*connector_id)?
            .plan_events(host_changes, kept.unwrap_or(now))?;
        events.extend(planned);
    }
    if events.is_empty() {
        return Ok(());
    }
    if kept.is_none() {
        return Err(PipelineError::new(
            PipelineErrorKind::PlanExpired,
            "the signing plan is missing or expired: review the diff again",
        ));
    }
    for planned in events.iter().filter(|planned| planned.user_signs) {
        let planned_id = planned.event.compute_id();
        let Some(event) = signed.iter().find(|event| event.id == planned_id) else {
            // The same event with a different public key has a different ID.
            let other_key = signed.iter().any(|event| {
                event.kind == planned.event.kind
                    && event.created_at == planned.event.created_at
                    && event.tags == planned.event.tags
                    && event.content == planned.event.content
            });
            return Err(if other_key {
                invalid_signature("a signed event has a different public key")
            } else {
                PipelineError::new(
                    PipelineErrorKind::SignatureMissing,
                    "the write needs an event that the user did not sign",
                )
            });
        };
        if event.pubkey != planned.event.pubkey {
            return Err(invalid_signature(
                "a signed event has a different public key",
            ));
        }
        if event.verify().is_err() {
            return Err(invalid_signature(
                "the signature of an event is not correct",
            ));
        }
    }
    Ok(())
}

fn invalid_signature(message: &str) -> PipelineError {
    PipelineError::new(PipelineErrorKind::SignatureInvalid, message)
}

/// The same as [`write_draft`], with the events that the user signed and
/// the time `now` in seconds after the Unix epoch (ADR 0050 item 12).
/// After the check of the diff ID, and before the first change on a host,
/// the function makes the planned events again and checks each event.
///
/// A feed with no connector needs a draft feed move and `public`, the
/// client that reads its public address. The feed XML goes to the host of
/// the move, and Studio writes nothing at the address before the move
/// (ADR 0067 items 8 to 11). A feed with a connector ignores `public`.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::ReadOnly`] when a feed with no connector has
/// no draft feed move, or `public` is `None`. Gives
/// [`PipelineErrorKind::Conflict`] when its public address gives other
/// bytes than the base of the draft, and [`PipelineErrorKind::FileExists`]
/// when the new host has a file at the path. Gives the errors of
/// [`write_draft`]. It also gives
/// [`PipelineErrorKind::PlanExpired`], [`PipelineErrorKind::SignatureMissing`]
/// and [`PipelineErrorKind::SignatureInvalid`] before any file changes on
/// a host.
#[expect(
    clippy::too_many_arguments,
    reason = "write_draft gives its arguments and the two new values"
)]
#[expect(
    clippy::too_many_lines,
    reason = "the write is one sequence of steps, and it reads best in one function"
)]
pub fn write_draft_signed(
    collection: &mut Collection,
    hosts: &Hosts,
    library: &Library,
    addresses: &dyn AddressCheck,
    links: LinkCheck<'_>,
    id: FeedId,
    approved: &DiffId,
    signed: &[nostr::prelude::Event],
    now: u64,
    public: Option<&crate::web::Web>,
) -> Result<Version, PipelineError> {
    let row = collection.feed(id)?;
    // A feed with no connector writes to the host of its draft feed move.
    let from_host = if row.connector_id.is_some() {
        let (connector_id, path) = writable(&row)?;
        Some((connector_id, path.to_owned()))
    } else {
        None
    };
    let (connector_id, path) = if let Some(host) = &from_host {
        host.clone()
    } else {
        let target = public_target(collection, &row)?.ok_or_else(|| {
            PipelineError::new(
                PipelineErrorKind::ReadOnly,
                "choose a file host for this feed first",
            )
        })?;
        if public.is_none() {
            return Err(PipelineError::new(
                PipelineErrorKind::ReadOnly,
                format!(
                    "feed {} has no connector, so Studio cannot write it here",
                    id.0
                ),
            ));
        }
        target
    };
    let connector = hosts.get(connector_id)?;
    let draft = one_cover(current_draft(collection, id)?)?;
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    let (operations, _blocked) = host_operations(collection, library, hosts, &row, &feed)?;
    let feed_move = feed_move(collection, hosts, &row)?;
    let mut extra = operation_bytes(collection, hosts, &operations)?;
    extra.extend(feed_move_bytes(feed_move.as_ref()));
    let (announcements, announce_skip) =
        plan_announcements(collection, hosts, &row, feed_move.as_ref())?;
    extra.extend(announce_bytes(&announcements, announce_skip));
    let grants = draft_grant_list(collection, id, &feed)?;
    // The apps of the grants are read now. The draft goes out later in this
    // function, and its grants go with it.
    let grant_apps = grant_apps(collection, id, &grants)?;
    extra.extend(grant_bytes(&grant_apps));
    // ADR 0110 item 19: a grant needs an active app. No grant is dropped
    // with no message.
    ensure_grant_apps_active(collection, &grant_apps)?;
    if &diff_id(&draft, &extra) != approved {
        return Err(PipelineError::new(
            PipelineErrorKind::DiffNotApproved,
            "the draft changed after the diff was approved; approve the new diff",
        ));
    }
    // ADR 0126 stage 1: a bad link stops the write before any file
    // changes. A granted write changes no link and makes no read
    // (ADR 0126 item 26).
    if let LinkCheck::Check(reader) = links {
        let report = link_report(hosts, reader, &row, &draft, &feed, feed_move.as_ref())?;
        if let Some(refusal) = report.refusal {
            return Err(PipelineError::refused(refusal));
        }
    }
    // ADR 0105 item 10: a pending file that Studio removed stops the write.
    let missing = removed_pending_files(collection, &operations)?;
    if !missing.is_empty() {
        return Err(PipelineError::new(
            PipelineErrorKind::FileMissing,
            format!(
                "these files are not on the server any more; add them again: {}",
                missing.join(", ")
            ),
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
    // ADR 0074 item 5: a new bad key stops the write. A bad key text that
    // the hosted feed has does not stop it.
    let bad_keys = feed.bad_keys();
    if !bad_keys.is_empty() {
        let hosted = hosted_document(&draft)
            .and_then(|doc| Feed::read(&doc).ok())
            .map(|f| f.key_texts())
            .unwrap_or_default();
        let new: Vec<String> = bad_keys
            .iter()
            .filter(|b| !hosted.contains(&b.value))
            .map(|b| format!("{} ({:?})", b.field, b.value))
            .collect();
        if !new.is_empty() {
            return Err(PipelineError::new(
                PipelineErrorKind::BadKey,
                format!("these key fields are not Nostr keys: {}", new.join(", ")),
            ));
        }
    }
    // ADR 0054 item 4: a live item with a problem stops the write before
    // any change on a host.
    let live_problems = crate::live::problems(&feed);
    if !live_problems.is_empty() {
        let texts: Vec<String> = live_problems
            .iter()
            .map(|p| format!("{} ({})", p.text, p.guid))
            .collect();
        return Err(PipelineError::new(
            PipelineErrorKind::BadLiveItem,
            format!("a live item is not correct: {}", texts.join(" ")),
        ));
    }
    // ADR 0122 item 10: the file of an expired temporary area is gone, so
    // the draft of its feed has no file to compare with.
    let source_gone = match &from_host {
        Some((source, _)) => {
            crate::connector::is_expired_temporary(&collection.connector(*source)?)
        }
        None => false,
    };
    let live = if from_host.is_some() {
        let live = connector.read(&path)?;
        let live_version = live.as_ref().map(|f| &f.version);
        let gone = source_gone && live.is_none();
        if !gone && live_version != draft.base_version.as_ref() {
            return Err(PipelineError::new(
                PipelineErrorKind::Conflict,
                format!("{path} changed on the file host after the draft started"),
            ));
        }
        live
    } else {
        // The feed XML at the public address is the base of the draft, and
        // the new host has no file at the path (ADR 0067 items 9 and 10).
        if let Some(web) = public {
            check_public_base(&row, web, &draft)?;
        }
        // A placed file of this feed is replaced (ADR 0126 item 19).
        if connector.read(&path)?.is_some()
            && !collection.is_placed_feed_file(connector_id, &path, id)?
        {
            return Err(PipelineError::new(
                PipelineErrorKind::FileExists,
                format!("a file exists at {path} on the new file host already"),
            ));
        }
        None
    };
    // ADR 0050 item 12: the planned events are correct before the first
    // change on a host.
    let changes = host_changes(
        collection,
        library,
        &row,
        &draft,
        &operations,
        feed_move.as_ref(),
    )?;
    check_signing_plan(collection, hosts, &changes, id, approved, signed, now)?;

    // Each host that the write puts a file on, deletes a file from, or
    // writes the feed file to. Studio scans each one after the write
    // (ADR 0036 item 6).
    let mut touched: HashSet<ConnectorId> = HashSet::new();
    touched.insert(connector_id);

    // Step 1: each upload and copy puts a library file on its host.
    put_uploads_and_copies(collection, hosts, library, &row, &operations, &mut touched)?;

    // Step 2: the feed file. With a feed move, it goes to the new host,
    // and then the host from before points to it (ADR 0038 item 19). The
    // release folder then keeps the same bytes as the hosted file
    // (ADR 0036 item 3).
    // Each move runs stage 2 of ADR 0126 on the new file, before the
    // pointer, the snapshot and the new location of the feed (items 14
    // and 16).
    let stage_two = StageTwo { links, feed: &feed };
    let version = if from_host.is_none() {
        let target = PublicTarget {
            connector,
            connector_id,
            path: &path,
        };
        put_public_feed_file(
            collection,
            hosts,
            id,
            &target,
            &draft,
            feed_move.as_ref(),
            &stage_two,
        )
    } else if let Some(moved) = &feed_move {
        if source_gone {
            put_feed_from_expired(collection, hosts, id, &draft, moved, &stage_two)
        } else {
            let old = FeedHost {
                connector,
                connector_id,
                path: &path,
            };
            put_moved_feed_file(
                collection,
                hosts,
                id,
                &old,
                &draft,
                live.as_ref(),
                moved,
                &stage_two,
            )
        }
    } else {
        let expected = match &live {
            Some(file) => {
                collection.add_snapshot(id, &file.version, &file.bytes)?;
                Expected::Version(file.version.clone())
            }
            None => Expected::Absent,
        };
        connector
            .write(&path, &draft.draft_xml, &expected)
            .map_err(PipelineError::from)
    };
    let version = match version {
        Ok(version) => version,
        Err(error) => {
            // ADR 0126 item 18: the destination failed stage 2. The feed
            // keeps its location and its draft move, and the host from
            // before keeps its file.
            if error.kind() == PipelineErrorKind::BadMoveLink
                && let Some(moved) = &feed_move
            {
                keep_refused_destination(collection, hosts, id, moved, &operations, &mut touched)?;
            }
            return Err(error);
        }
    };
    if let Some(moved) = &feed_move {
        // The destination passed stage 2, so its file is the feed now.
        collection.remove_placed_feed_file(moved.connector_id, &moved.path)?;
        // The path is no longer a pointer file (ADR 0126 item 22). The
        // record stays until here, so a stopped write that continues has
        // the same diff.
        collection.remove_feed_pointer(moved.connector_id, &moved.path)?;
    }
    touched.insert(feed_move.as_ref().map_or(connector_id, |m| m.connector_id));

    // Publish the changes of steps 1 and 2 on each host. A failure here
    // leaves the feed state, the feed location and the draft as they were
    // (ADR 0050 item 7).
    finish_hosts(collection, hosts, &touched)?;

    // Step 4: the feed gets its new host and address, and the library
    // keeps the bytes of the file on the new host.
    if let Some(moved) = &feed_move {
        collection.set_feed_location(id, moved.connector_id, &moved.path, moved.url.as_deref())?;
        if from_host.is_none() {
            collection.set_files_connector(id, Some(moved.connector_id))?;
        }
    }
    collection.record_state(id, &feed, &version)?;
    if let Some(folder) = row.folder.as_deref() {
        let file_name = path
            .rsplit_once('/')
            .map_or(path.as_str(), |(_, name)| name);
        library
            .write_feed_xml(folder, file_name, &draft.draft_xml)
            .map_err(|error| PipelineError::new(PipelineErrorKind::Connector, error.to_string()))?;
    }
    // Step 3: confirm that each new file is on its host, and that the
    // public address of each file works, before Studio deletes any file
    // (ADR 0036 item 15, ADR 0043 items 7 and 8).
    let feed_url = written_feed_url(
        collection,
        hosts,
        id,
        (connector_id, &path),
        feed_move.as_ref(),
    )?;
    if let Err(error) = confirm_new_addresses(hosts, addresses, &operations, feed_url.as_deref()) {
        keep_draft_after_failed_confirmation(
            collection,
            hosts,
            id,
            &draft,
            &version,
            &operations,
            &touched,
        )?;
        return Err(error);
    }

    // Step 4: each deletion, with the stamp from the inventory
    // (ADR 0036 item 16). `host_operations` ran again above, with the
    // current state of each feed. A blocked deletion is already out of
    // `operations` (ADR 0036 item 13).
    let mut deleted_on: HashSet<ConnectorId> = HashSet::new();
    delete_each_operation(
        collection,
        hosts,
        &operations,
        &mut touched,
        &mut deleted_on,
    )?;
    finish_hosts(collection, hosts, &deleted_on)?;

    // Step 5: the draft, its host operations, its feed move and its
    // grants go out.
    collection.remove_draft_operations(id)?;
    collection.remove_draft_feed_move(id)?;
    collection.remove_signing_plan(id)?;
    collection.delete_draft(id)?;

    // The write records the grants of the diff (ADR 0054 item 12). It ends
    // each grant whose live item is not in the written feed, or has the
    // status `ended` (ADR 0054 item 14). This occurs before the scans, so
    // that a failed scan does not lose the record.
    record_grants(collection, id, &feed, &grant_apps, now)?;

    // Step 6: the feed is written, and each step on a host passed. The
    // announcements go into the queue before the scans, so that a failed
    // scan does not lose them (ADR 0052 item 13).
    announce::queue(collection, id, &announcements, &Timing::DEFAULT, now)?;

    // The feed is written, so each open change set row of the feed is done.
    // A change set with no more work goes (ADR 0091 item 19).
    collection.mark_change_set_written(id, i64::try_from(now).unwrap_or(i64::MAX))?;
    collection.delete_closed_change_sets()?;

    for connector_id in touched {
        inventory::scan(collection, hosts, connector_id)?;
    }

    // ADR 0105 item 5: with `off`, the bytes of each sent asset go from the
    // server. The feed is written, so an error here does not fail the
    // write. The next start removes the bytes (ADR 0105 item 13).
    remove_sent_assets(collection, hosts, library, &operations);

    Ok(version)
}

/// Remove the local bytes of each asset that the write sent, when a host
/// has a copy with the same SHA-256 (ADR 0105 item 5). The function reads
/// the host copy again through its connector, so that the removal never
/// depends on a stale record.
fn remove_sent_assets(
    collection: &Collection,
    hosts: &Hosts,
    library: &Library,
    operations: &[HostOperation],
) {
    if collection.asset_rules().is_none() {
        return;
    }
    let sent: Vec<LibraryFileId> = operations
        .iter()
        .filter(|operation| matches!(operation.kind, OperationKind::Upload | OperationKind::Copy))
        .filter_map(|operation| operation.library_file_id.map(LibraryFileId))
        .collect();
    let find =
        |file: &crate::collection::LibraryFile| crate::assets::find_copy(collection, hosts, file);
    let _ = crate::assets::remove_hosted(collection, library, Some(&sent), &find);
}

/// Write one change that a show grant covers, with no diff that the user
/// examines (ADR 0054 items 15 and 16). `now` is the time in seconds since
/// the Unix epoch.
///
/// The steps are:
///
/// 1. The grant of the live item must be active at `now`, with the feed
///    that the host gives now.
/// 2. A draft of the user with no change goes out. A draft with a change
///    stops the write, and it does not change.
/// 3. Studio makes a draft from the hosted file, and applies `change` to
///    it.
/// 4. The draft must change only what the grant covers
///    ([`grant::covered`]). The write must also have no host operation and
///    no feed move.
/// 5. Studio makes the diff and writes it through [`write_draft_signed`],
///    with the ID of that diff. The snapshot, the conflict check and the
///    announcements are the same as for each write.
/// 6. Studio records the granted write. After [`GrantChange::End`], the
///    grant ends.
///
/// After step 3, each error removes the draft of step 3 and its signing
/// plan. Thus the feed keeps no draft that the user did not make.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::GrantNotActive`] when the grant is not
/// active. Gives [`PipelineErrorKind::DraftOpen`] when the feed has a draft
/// with a change. Gives [`PipelineErrorKind::BadStatus`] when the status
/// cannot change to the requested status. Gives
/// [`PipelineErrorKind::NotCovered`] when the draft changes more than the
/// grant covers. Also gives the errors of [`write_draft_signed`], such as
/// [`PipelineErrorKind::VaultLocked`] and [`PipelineErrorKind::Conflict`].
#[expect(
    clippy::too_many_arguments,
    reason = "the write needs the arguments of write_draft and the grant"
)]
pub fn write_granted(
    collection: &mut Collection,
    hosts: &Hosts,
    library: &Library,
    addresses: &dyn AddressCheck,
    feed_id: FeedId,
    item_guid: &str,
    change: &GrantChange,
    app_name: &str,
    now: u64,
) -> Result<Version, PipelineError> {
    let apply = |feed: &mut Feed| apply_grant_change(feed, item_guid, change);
    let written_at = i64::try_from(now).unwrap_or(i64::MAX);
    // ADR 0110 item 26: the live value check is in the core. It runs
    // before the host read, with the origin that the grant keeps.
    let checked = check_granted_change(collection, feed_id, item_guid, change);
    let version = match checked.and_then(|()| {
        write_granted_with(
            collection, hosts, library, addresses, feed_id, item_guid, apply, now,
        )
    }) {
        Ok(version) => version,
        Err(error) => {
            // ADR 0110 item 43: a refused write adds its cause to the list.
            if let Some(cause) = refusal_of(error.kind())
                && let Some(grant) = grant::grant_for(collection, feed_id, item_guid)?
            {
                grant::add_refusal(collection, grant.id, app_name, change, cause, written_at)?;
            }
            return Err(error);
        }
    };
    let grant = grant::grant_for(collection, feed_id, item_guid)?.ok_or_else(|| {
        PipelineError::new(PipelineErrorKind::GrantNotActive, "the grant is not there")
    })?;
    // Step 6: the record of the granted write (ADR 0054 item 17).
    grant::add_write(collection, grant.id, app_name, change, written_at)?;
    if *change == GrantChange::End {
        grant::end(collection, grant.id, written_at)?;
    }
    Ok(version)
}

/// The same as the steps 1 to 5 of [`write_granted`], with a change of the
/// typed model in place of a [`GrantChange`]. A test uses it to make a
/// change that the grant does not cover. Step 4 still refuses that change.
///
/// # Errors
///
/// Gives the errors of [`write_granted`].
#[cfg(any(test, feature = "testing"))]
#[expect(
    clippy::too_many_arguments,
    reason = "the function has the arguments of write_granted"
)]
pub fn write_granted_edit(
    collection: &mut Collection,
    hosts: &Hosts,
    library: &Library,
    addresses: &dyn AddressCheck,
    feed_id: FeedId,
    item_guid: &str,
    edit: impl FnOnce(&mut Feed) -> Result<(), PipelineError>,
    now: u64,
) -> Result<Version, PipelineError> {
    write_granted_with(
        collection, hosts, library, addresses, feed_id, item_guid, edit, now,
    )
}

/// Check a granted live value against the live value origin that the grant
/// keeps (ADR 0110 item 26). Another change needs no check. A grant that is
/// not there, or has ended, gives no error here: the write then gives
/// [`PipelineErrorKind::GrantNotActive`].
///
/// # Errors
///
/// Gives [`PipelineErrorKind::InvalidLiveValue`] for a live value that
/// [`live::check_granted_live_value`] refuses.
fn check_granted_change(
    collection: &Collection,
    feed_id: FeedId,
    item_guid: &str,
    change: &GrantChange,
) -> Result<(), PipelineError> {
    let GrantChange::SetLiveValue { uri, protocol } = change else {
        return Ok(());
    };
    let Some(grant) = grant::grant_for(collection, feed_id, item_guid)? else {
        return Ok(());
    };
    if grant.ended_at.is_some() {
        return Ok(());
    }
    // A grant with no origin has no address that it permits.
    let origin = grant.live_value_origin.as_deref().unwrap_or_default();
    live::check_granted_live_value(uri, protocol, origin)
        .map_err(|e| PipelineError::new(PipelineErrorKind::InvalidLiveValue, e.to_string()))
}

/// Apply a granted change to the live item with this `guid`.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::GrantNotActive`] when the feed has no such
/// live item, and [`PipelineErrorKind::BadStatus`] when the status cannot
/// change to the requested status.
fn apply_grant_change(
    feed: &mut Feed,
    item_guid: &str,
    change: &GrantChange,
) -> Result<(), PipelineError> {
    let live_item = feed
        .live_items
        .iter_mut()
        .map(|entry| &mut entry.value)
        .find(|live| live.item.guid.as_deref() == Some(item_guid))
        .ok_or_else(|| {
            PipelineError::new(
                PipelineErrorKind::GrantNotActive,
                format!("the hosted feed has no live item {item_guid}"),
            )
        })?;
    let bad_status = |error: live::LiveError| {
        PipelineError::new(PipelineErrorKind::BadStatus, error.to_string())
    };
    match change {
        GrantChange::GoLive => live::set_status(live_item, LiveStatus::Live).map_err(bad_status),
        GrantChange::End => live::set_status(live_item, LiveStatus::Ended).map_err(bad_status),
        GrantChange::SetLiveValue { uri, protocol } => {
            live::check_live_value(uri, protocol).map_err(|e| {
                PipelineError::new(PipelineErrorKind::InvalidLiveValue, e.to_string())
            })?;
            live_item.live_value = Some(crate::feed::LiveValue {
                uri: Some(uri.clone()),
                protocol: Some(protocol.clone()),
            });
            Ok(())
        }
        GrantChange::RemoveLiveValue => {
            live_item.live_value = None;
            Ok(())
        }
    }
}

/// The cause of a refused granted write, from the error of the write
/// (ADR 0110 item 43). An error with no cause in the list gives `None`.
fn refusal_of(kind: PipelineErrorKind) -> Option<grant::Refusal> {
    match kind {
        PipelineErrorKind::DraftOpen => Some(grant::Refusal::DraftWithChanges),
        PipelineErrorKind::GrantNotActive => Some(grant::Refusal::GrantEnded),
        PipelineErrorKind::InvalidLiveValue => Some(grant::Refusal::LiveValueRefused),
        _ => None,
    }
}

/// The steps 1 to 5 of [`write_granted`].
#[expect(
    clippy::too_many_arguments,
    reason = "the function has the arguments of write_granted"
)]
fn write_granted_with(
    collection: &mut Collection,
    hosts: &Hosts,
    library: &Library,
    addresses: &dyn AddressCheck,
    feed_id: FeedId,
    item_guid: &str,
    edit: impl FnOnce(&mut Feed) -> Result<(), PipelineError>,
    now: u64,
) -> Result<Version, PipelineError> {
    let not_active = || {
        PipelineError::new(
            PipelineErrorKind::GrantNotActive,
            format!("the show {item_guid} has no active grant"),
        )
    };
    // Step 1: the grant must be active. A grant that ended needs no read
    // of the host.
    if grant::grant_for(collection, feed_id, item_guid)?
        .is_none_or(|grant| grant.ended_at.is_some())
    {
        return Err(not_active());
    }
    let row = collection.feed(feed_id)?;
    let (connector_id, path) = writable(&row)?;
    let hosted = hosts
        .get(connector_id)?
        .read(path)?
        .ok_or_else(not_active)?;
    let hosted_doc = Document::parse_bytes(&hosted.bytes)?;
    let hosted_feed = Feed::read(&hosted_doc)?;
    if grant::active(collection, feed_id, item_guid, &hosted_feed, now)?.is_none() {
        return Err(not_active());
    }

    // Step 2: a draft of the user never goes to a host through a grant.
    if let Some(draft) = collection.draft(feed_id)? {
        if user_draft_has_change(collection, feed_id, &draft, &hosted.bytes)? {
            return Err(PipelineError::new(
                PipelineErrorKind::DraftOpen,
                "the feed has a draft with changes; write or discard the draft first",
            ));
        }
        discard_draft(collection, feed_id)?;
        collection.delete_draft(feed_id)?;
    }

    // Step 3: the draft of the grant, from the hosted file.
    let draft = DraftRow {
        base_version: Some(hosted.version.clone()),
        base_xml: hosted.bytes.clone(),
        draft_xml: hosted.bytes.clone(),
    };
    collection.save_draft(feed_id, &draft)?;
    let result = (|| {
        let mut doc = hosted_doc.clone();
        let mut changed = hosted_feed.clone();
        edit(&mut changed)?;
        changed.write_hosted(&mut doc, Some(&hosted_doc))?;
        let draft_xml = doc.write().into_bytes();
        collection.save_draft(
            feed_id,
            &DraftRow {
                draft_xml: draft_xml.clone(),
                ..draft.clone()
            },
        )?;

        // Step 4: the change must be one that the grant covers. The compare
        // uses the model that Studio reads from the new draft bytes.
        let after = Feed::read(&Document::parse_bytes(&draft_xml)?)?;
        let not_covered = || {
            PipelineError::new(
                PipelineErrorKind::NotCovered,
                "the change is not one that the show grant covers",
            )
        };
        if !grant::covered(&hosted_feed, &after, item_guid) {
            return Err(not_covered());
        }

        // Step 5: the diff and the write. A plan of an earlier diff does
        // not apply to this draft.
        collection.remove_signing_plan(feed_id)?;
        let diff = diff_at(collection, hosts, library, feed_id, now)?;
        if !diff.operations.is_empty() || !diff.blocked.is_empty() || diff.feed_move.is_some() {
            return Err(not_covered());
        }
        write_draft_signed(
            collection,
            hosts,
            library,
            addresses,
            LinkCheck::GrantedWrite,
            feed_id,
            &diff.id,
            &[],
            now,
            None,
        )
    })();
    if result.is_err() {
        // The feed keeps no draft that the user did not make.
        discard_draft(collection, feed_id)?;
        collection.delete_draft(feed_id)?;
    }
    result
}

/// True when the draft of the user has a change that the user can
/// approve: different bytes than the hosted file, a host operation, a feed
/// move, or a show grant for a `pending` live item of the draft.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails.
fn user_draft_has_change(
    collection: &Collection,
    id: FeedId,
    draft: &DraftRow,
    hosted: &[u8],
) -> Result<bool, PipelineError> {
    if draft.draft_xml != hosted
        || !collection.draft_operations(id)?.is_empty()
        || collection.draft_feed_move(id)?.is_some()
    {
        return Ok(true);
    }
    // The bytes are the bytes of the hosted file, so they are a feed.
    let feed = Feed::read(&Document::parse_bytes(&draft.draft_xml)?)?;
    Ok(!draft_grant_list(collection, id, &feed)?.is_empty())
}

/// The state of a feed whose active show grant a draft with changes blocks
/// (ADR 0110 item 42). The UI shows a notice with these values and keeps no
/// copy of the rule of ADR 0054 item 16.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct GrantBlock {
    /// The state name in the feed state: always `grant_blocked_by_draft`.
    pub state: &'static str,
    /// The ID of the blocked grant.
    pub grant_id: i64,
    /// The name of the app that the grant names.
    pub app_name: String,
    /// The title of the show.
    pub show_title: Option<String>,
    /// The start time of the show, as the feed gives it.
    pub start: Option<String>,
}

/// The state `grant_blocked_by_draft` of a feed (ADR 0110 item 42): an
/// active show grant exists and the feed has a draft with changes. `hosted`
/// is the feed file that the host gives now, and `now` is the time in
/// seconds since the Unix epoch. Gives `None` when no active grant is
/// blocked.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the hosted file is not a feed or the
/// collection fails.
pub fn grant_blocked_by_draft(
    collection: &Collection,
    id: FeedId,
    hosted: &[u8],
    now: u64,
) -> Result<Option<GrantBlock>, PipelineError> {
    let open: Vec<_> = grant::grants(collection, id)?
        .into_iter()
        .filter(|show| show.ended_at.is_none())
        .collect();
    if open.is_empty() {
        return Ok(None);
    }
    let hosted_feed = Feed::read(&Document::parse_bytes(hosted)?)?;
    let mut blocked = None;
    for show in open {
        if grant::active(collection, id, &show.item_guid, &hosted_feed, now)?.is_some() {
            blocked = Some(show);
            break;
        }
    }
    let Some(show) = blocked else {
        return Ok(None);
    };
    let Some(draft) = collection.draft(id)? else {
        return Ok(None);
    };
    if !user_draft_has_change(collection, id, &draft, hosted)? {
        return Ok(None);
    }
    let live = grant::live_item(&hosted_feed, &show.item_guid);
    let app_name = match show.app_id {
        Some(app_id) => crate::apps::app(collection, app_id)?.map(|app| app.name),
        None => None,
    };
    Ok(Some(GrantBlock {
        state: "grant_blocked_by_draft",
        grant_id: show.id,
        app_name: app_name.unwrap_or_default(),
        show_title: live.and_then(|live| live.item.title.clone()),
        start: live.and_then(|live| live.start.clone()),
    }))
}

/// Record the grants of a write, and end each grant that the written feed
/// ends (ADR 0054 items 12 and 14). Each grant names its app and keeps the
/// live value origin of that app at the approval (ADR 0110 items 19 and
/// 25). The write refused a grant whose app is not active before it ran.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails.
fn record_grants(
    collection: &Collection,
    id: FeedId,
    written: &Feed,
    grants: &[(String, Option<i64>)],
    now: u64,
) -> Result<(), PipelineError> {
    let now = i64::try_from(now).unwrap_or(i64::MAX);
    for (guid, app_id) in grants {
        // `ensure_grant_apps_active` examined each app before the write.
        let app = app_id
            .map(|app_id| crate::apps::app(collection, app_id))
            .transpose()?
            .flatten();
        if let Some(app) = app
            && let (crate::apps::AppState::Active, Some(origin)) =
                (app.state, &app.live_value_origin)
        {
            grant::record(collection, id, guid, app.id, origin, now)?;
        }
    }
    for open in grant::grants(collection, id)? {
        if open.ended_at.is_some() {
            continue;
        }
        let ended = grant::live_item(written, &open.item_guid).is_none_or(|live| {
            live.status.as_deref().and_then(LiveStatus::parse) == Some(LiveStatus::Ended)
        });
        if ended {
            grant::end(collection, open.id, now)?;
        }
    }
    Ok(())
}

/// Call `finish` on each host of `ids`, in the sequence of the host IDs.
/// After each call, the collection keeps the blob records of the host when
/// the host gives them (ADR 0050 items 6 and 7).
///
/// # Errors
///
/// Gives a [`PipelineError`] when a `finish` call or the collection fails.
fn finish_hosts(
    collection: &mut Collection,
    hosts: &Hosts,
    ids: &HashSet<ConnectorId>,
) -> Result<(), PipelineError> {
    let mut ordered: Vec<ConnectorId> = ids.iter().copied().collect();
    ordered.sort_by_key(|id| id.0);
    for id in ordered {
        let host = hosts.get(id)?;
        host.finish()?;
        if let Some(blobs) = host.kept_blobs() {
            collection.replace_kept_blobs(id, &blobs)?;
        }
    }
    Ok(())
}

/// The host of a feed XML, and its path on the host.
struct FeedHost<'a> {
    connector: &'a dyn Connector,
    connector_id: ConnectorId,
    path: &'a str,
}

/// The public address of a moved feed on its new host.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::NoPublicAddress`] when the new host gives none.
fn new_address(moved: &FeedMove) -> Result<&str, PipelineError> {
    moved.url.as_deref().ok_or_else(|| {
        PipelineError::new(
            PipelineErrorKind::NoPublicAddress,
            format!("{} has no public address on the new file host", moved.path),
        )
    })
}

/// Stage 2 of ADR 0126 for one write: the facts that it needs.
struct StageTwo<'a> {
    links: LinkCheck<'a>,
    /// The draft as a typed feed.
    feed: &'a Feed,
}

impl StageTwo<'_> {
    /// Read the destination of `moved` after the feed XML is on the new
    /// host (ADR 0126 items 14 and 15). A granted write never moves a feed
    /// and makes no read. A destination with no address has no read.
    ///
    /// The function publishes the changes of the new host first, so that a
    /// connector that publishes at the end of a step serves the new file.
    ///
    /// # Errors
    ///
    /// Gives [`PipelineErrorKind::BadMoveLink`] when the destination does
    /// not pass.
    fn run(&self, hosts: &Hosts, moved: &FeedMove) -> Result<(), PipelineError> {
        let LinkCheck::Check(reader) = self.links else {
            return Ok(());
        };
        let Some(url) = moved.url.as_deref() else {
            return Ok(());
        };
        hosts.get(moved.connector_id)?.finish()?;
        move_link::check_destination(reader, self.feed, url).map_err(PipelineError::refused)
    }
}

/// True when `found` is a feed with the `podcast:guid` of `draft`. A draft
/// with no `podcast:guid` has no such file (ADR 0126 item 19).
fn has_guid_of(found: &[u8], draft: &DraftRow) -> bool {
    let guid_of = |bytes: &[u8]| {
        Document::parse_bytes(bytes)
            .ok()
            .and_then(|doc| Feed::read(&doc).ok())
            .and_then(|feed| feed.podcast_guid)
    };
    guid_of(&draft.draft_xml).is_some_and(|guid| guid_of(found).as_deref() == Some(guid.as_str()))
}

/// Put `draft` on a new host at `path`. A file with the same bytes from an
/// earlier, stopped write is a finished step. A placed file of this feed
/// or pointer file of this feed with the `podcast:guid` of the draft is
/// replaced (ADR 0126 items 19 and 21).
///
/// # Errors
///
/// Gives [`PipelineErrorKind::FileExists`] when another file exists at the
/// path. The file does not change then.
fn place_feed_file(
    collection: &Collection,
    id: FeedId,
    (host, connector_id): (&dyn Connector, ConnectorId),
    path: &str,
    draft: &DraftRow,
) -> Result<Version, PipelineError> {
    let exists = || {
        PipelineError::new(
            PipelineErrorKind::FileExists,
            format!("a file exists at {path} on the new file host already"),
        )
    };
    match host.write(path, &draft.draft_xml, &Expected::Absent) {
        Ok(version) => Ok(version),
        Err(error) if error.kind() == ConnectorErrorKind::Conflict => {
            // A write that stopped after this step left the draft XML on
            // the new host. The same bytes let the write continue, so the
            // user can finish the move. Other bytes are a different file,
            // unless they are a placed file of this feed.
            let Some(found) = host.read(path)? else {
                return Err(exists());
            };
            if found.bytes == draft.draft_xml {
                return Ok(found.version);
            }
            if (collection.is_placed_feed_file(connector_id, path, id)?
                || collection.is_pointer_of_feed(connector_id, path, id)?)
                && has_guid_of(&found.bytes, draft)
            {
                return host
                    .write(path, &draft.draft_xml, &Expected::Version(found.version))
                    .map_err(|error| {
                        if error.kind() == ConnectorErrorKind::Conflict {
                            exists()
                        } else {
                            error.into()
                        }
                    });
            }
            Err(exists())
        }
        Err(error) => Err(error.into()),
    }
}

/// Write the feed XML of a draft to the new host of a feed move.
///
/// # Errors
///
/// Gives the errors of [`place_feed_file`].
fn write_to_new_host(
    collection: &Collection,
    hosts: &Hosts,
    id: FeedId,
    moved: &FeedMove,
    draft: &DraftRow,
) -> Result<Version, PipelineError> {
    let new_host = hosts.get(moved.connector_id)?;
    place_feed_file(
        collection,
        id,
        (new_host, moved.connector_id),
        &moved.path,
        draft,
    )
}

/// Put the feed XML of a feed that moves off an expired temporary area on
/// its new host (ADR 0122 item 10). The old file is gone, so the write makes
/// no snapshot of it and no pointer or redirect on the old host. Stage 2
/// runs on the new file (ADR 0126 item 14).
///
/// # Errors
///
/// Gives the errors of [`write_to_new_host`], and
/// [`PipelineErrorKind::NoPublicAddress`] when the new host gives no address.
/// Gives [`PipelineErrorKind::BadMoveLink`] when stage 2 refuses.
fn put_feed_from_expired(
    collection: &Collection,
    hosts: &Hosts,
    id: FeedId,
    draft: &DraftRow,
    moved: &FeedMove,
    stage_two: &StageTwo<'_>,
) -> Result<Version, PipelineError> {
    new_address(moved)?;
    let version = write_to_new_host(collection, hosts, id, moved, draft)?;
    stage_two.run(hosts, moved)?;
    Ok(version)
}

/// Put the feed XML on the new host of a feed move, then make the host from
/// before point to it: the second step of [`write_draft`] with a feed move
/// (ADR 0038 items 19 and 20). Gives the version of the file on the new
/// host.
///
/// The new host gets the file only when it has no file at the path. The
/// host from before gets a redirect, or a pointer file that Studio keeps.
/// Stage 2 runs before the snapshot and the pointer (ADR 0126 item 16).
///
/// # Errors
///
/// Gives [`PipelineErrorKind::FileExists`] when a file exists on the new
/// host at the path, before any file changes. Gives
/// [`PipelineErrorKind::BadMoveLink`] when stage 2 refuses.
#[expect(
    clippy::too_many_arguments,
    reason = "the move needs the old host, the new host and stage 2"
)]
fn put_moved_feed_file(
    collection: &Collection,
    hosts: &Hosts,
    id: FeedId,
    old: &FeedHost<'_>,
    draft: &DraftRow,
    live: Option<&crate::connector::FileRead>,
    moved: &FeedMove,
    stage_two: &StageTwo<'_>,
) -> Result<Version, PipelineError> {
    let live_file = live.ok_or_else(|| {
        PipelineError::new(
            PipelineErrorKind::NotFound,
            format!("{} does not exist on the file host", old.path),
        )
    })?;
    let new_url = new_address(moved)?;
    let version = write_to_new_host(collection, hosts, id, moved, draft)?;
    stage_two.run(hosts, moved)?;
    collection.add_snapshot(id, &live_file.version, &live_file.bytes)?;
    match moved.method {
        FeedMoveMethod::Redirect => old.connector.set_redirect(old.path, new_url)?,
        FeedMoveMethod::Pointer => {
            let pointer = pointer_bytes(draft, new_url)?;
            old.connector.write(
                old.path,
                &pointer,
                &Expected::Version(live_file.version.clone()),
            )?;
            collection.add_feed_pointer(old.connector_id, old.path, id)?;
        }
        // A feed with a host never has this method.
        FeedMoveMethod::None => {}
    }
    Ok(version)
}

/// Keep the state after stage 2 refused the destination (ADR 0126 item 18).
///
/// The feed XML on the new host becomes a placed file of the feed. The
/// feed keeps its location and its draft move, and the draft does not
/// change. The uploads and the copies get a confirmation, as in
/// [`keep_draft_after_failed_confirmation`]. Then the function publishes
/// and scans each host in `touched`.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection or a host fails.
fn keep_refused_destination(
    collection: &mut Collection,
    hosts: &Hosts,
    id: FeedId,
    moved: &FeedMove,
    operations: &[HostOperation],
    touched: &mut HashSet<ConnectorId>,
) -> Result<(), PipelineError> {
    collection.add_placed_feed_file(moved.connector_id, &moved.path, id)?;
    // A pointer file that the feed XML replaced is a placed file now, and
    // no code treats it as a pointer (ADR 0126 items 19 and 22).
    collection.remove_feed_pointer(moved.connector_id, &moved.path)?;
    touched.insert(moved.connector_id);
    finish_hosts(collection, hosts, touched)?;
    confirm_files_on_next_write(collection, id, operations)?;
    for &connector_id in touched.iter() {
        inventory::scan(collection, hosts, connector_id)?;
    }
    Ok(())
}

/// Check that the public address of a feed with no connector still gives
/// the base of the draft (ADR 0067 item 9). `web` applies the network
/// policy of the caller.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::Conflict`] when the bytes differ from the
/// base, and [`PipelineErrorKind::Network`] when the read fails.
fn check_public_base(
    row: &FeedRow,
    web: &crate::web::Web,
    draft: &DraftRow,
) -> Result<(), PipelineError> {
    let url = row.public_url.as_deref().ok_or_else(|| {
        PipelineError::new(
            PipelineErrorKind::ReadOnly,
            format!("feed {} has no public address", row.id.0),
        )
    })?;
    let read = crate::discover::read_public(web, url)
        .map_err(|message| PipelineError::new(PipelineErrorKind::Network, message))?;
    if Some(&read.version) != draft.base_version.as_ref() {
        return Err(PipelineError::new(
            PipelineErrorKind::Conflict,
            "the feed changed at its public address after the draft started",
        ));
    }
    Ok(())
}

/// The new host of a feed with no connector, and the path on it.
struct PublicTarget<'a> {
    connector: &'a dyn Connector,
    connector_id: ConnectorId,
    path: &'a str,
}

/// Put the feed XML of a feed with no connector on its new host. Studio
/// keeps the base of the draft as a snapshot. It writes nothing at the
/// address before the move (ADR 0067 item 8). Stage 2 runs before the
/// snapshot (ADR 0126 item 14). Gives the version of the new file.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::FileExists`] when a file is at the path, and
/// [`PipelineErrorKind::BadMoveLink`] when stage 2 refuses.
fn put_public_feed_file(
    collection: &Collection,
    hosts: &Hosts,
    id: FeedId,
    target: &PublicTarget<'_>,
    draft: &DraftRow,
    moved: Option<&FeedMove>,
    stage_two: &StageTwo<'_>,
) -> Result<Version, PipelineError> {
    let version = match target
        .connector
        .write(target.path, &draft.draft_xml, &Expected::Absent)
    {
        Ok(version) => version,
        Err(error) if error.kind() == ConnectorErrorKind::Conflict => {
            // Only a placed file of this feed is replaced, and the same
            // bytes are not a finished step here.
            let found = target.connector.read(target.path)?;
            let replaced = match found {
                Some(found)
                    if collection.is_placed_feed_file(target.connector_id, target.path, id)?
                        && has_guid_of(&found.bytes, draft) =>
                {
                    Some(target.connector.write(
                        target.path,
                        &draft.draft_xml,
                        &Expected::Version(found.version),
                    ))
                }
                _ => None,
            };
            match replaced {
                Some(Ok(version)) => version,
                Some(Err(error)) if error.kind() != ConnectorErrorKind::Conflict => {
                    return Err(error.into());
                }
                _ => {
                    return Err(PipelineError::new(
                        PipelineErrorKind::FileExists,
                        format!(
                            "a file exists at {} on the new file host already",
                            target.path
                        ),
                    ));
                }
            }
        }
        Err(error) => return Err(error.into()),
    };
    if let Some(moved) = moved {
        stage_two.run(hosts, moved)?;
    }
    if let Some(base) = &draft.base_version {
        collection.add_snapshot(id, base, &draft.base_xml)?;
    }
    Ok(version)
}

/// The bytes of a pointer file: the draft with `itunes:new-feed-url` set
/// to `new_url` (ADR 0038 item 19).
///
/// # Errors
///
/// Gives a [`PipelineError`] when the draft is not a feed.
fn pointer_bytes(draft: &DraftRow, new_url: &str) -> Result<Vec<u8>, PipelineError> {
    let mut doc = Document::parse_bytes(&draft.draft_xml)?;
    let mut feed = Feed::read(&doc)?;
    feed.new_feed_url = Some(new_url.to_owned());
    let hosted = hosted_document(draft);
    feed.write_hosted(&mut doc, hosted.as_ref())?;
    Ok(doc.write().into_bytes())
}

/// The paths of the files of `operations` that Studio removed from the
/// server at the end of their 7 days (ADR 0105 item 10).
fn removed_pending_files(
    collection: &Collection,
    operations: &[HostOperation],
) -> Result<Vec<String>, PipelineError> {
    let mut paths = Vec::new();
    for operation in operations {
        if !matches!(operation.kind, OperationKind::Upload | OperationKind::Copy) {
            continue;
        }
        let Some(id) = operation.library_file_id else {
            continue;
        };
        let file = collection.library_file(LibraryFileId(id))?;
        if !file.local && file.removed_at.is_some() {
            paths.push(file.path);
        }
    }
    Ok(paths)
}

/// Removes its file when it goes out of scope. A copy or a move of a file
/// that the server does not hold uses one, in `incoming/`. The file goes
/// after the operation, also after an error (ADR 0105 item 6).
struct TemporaryFile(std::path::PathBuf);

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Read the copy of a file that is not in the library from its host into a
/// temporary file in the `incoming/` folder (ADR 0105 item 6).
fn temporary_copy(
    collection: &Collection,
    hosts: &Hosts,
    file: &crate::collection::LibraryFile,
    incoming: &Path,
) -> Result<TemporaryFile, PipelineError> {
    let missing = || {
        PipelineError::new(
            PipelineErrorKind::Connector,
            format!(
                "{} is not on the server and no file host has a copy",
                file.path
            ),
        )
    };
    let copy = inventory::copies(collection, hosts, file)?
        .into_iter()
        .next()
        .ok_or_else(missing)?;
    let read = hosts
        .get(copy.connector_id)?
        .read(&copy.path)?
        .ok_or_else(missing)?;
    let io_error = |error: std::io::Error| {
        PipelineError::new(
            PipelineErrorKind::Connector,
            format!("{}: cannot write the temporary file: {error}", file.path),
        )
    };
    std::fs::create_dir_all(incoming).map_err(io_error)?;
    let name = format!("copy-{}-{}.tmp", file.id.0, unix_now());
    let guard = TemporaryFile(incoming.join(name));
    std::fs::write(&guard.0, &read.bytes).map_err(io_error)?;
    Ok(guard)
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
        // ADR 0105 item 6: a file that the server does not hold comes from
        // its host. The temporary file goes at the end of this step.
        let temporary = match collection.asset_rules() {
            Some(rules) if !file.local && file.removed_at.is_none() => {
                Some(temporary_copy(collection, hosts, &file, &rules.incoming)?)
            }
            _ => None,
        };
        let library_path = temporary
            .as_ref()
            .map_or_else(|| library.path(folder, &file.path), |t| t.0.clone());
        let read_error = |error: std::io::Error| {
            PipelineError::new(
                PipelineErrorKind::Connector,
                format!("{}: cannot read the library file: {error}", file.path),
            )
        };
        let mut source = std::fs::File::open(&library_path).map_err(read_error)?;
        let length = source.metadata().map_err(read_error)?.len();
        let target = hosts.get(operation.connector_id)?;
        match target.write_from(&operation.path, &mut source, length, &Expected::Absent) {
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
                format!("{} is not on the file host after the write", operation.path),
            )
        })?;
        collection.put_host_file(
            operation.connector_id,
            &operation.path,
            listed.size,
            &listed.stamp,
        )?;
        let sha256 = match file.sha256.clone() {
            Some(sha256) => sha256,
            None => crate::digest::sha256_file(&library_path).map_err(read_error)?,
        };
        collection.set_host_file_sha(operation.connector_id, &operation.path, &sha256)?;
        touched.insert(operation.connector_id);
    }
    Ok(())
}

/// The public address of the feed file after the write: the address of
/// the feed row, or else the address that its host gives for its path
/// (ADR 0021). With a feed move, the host and the path are the new ones.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection or the host set fails.
fn written_feed_url(
    collection: &Collection,
    hosts: &Hosts,
    id: FeedId,
    (connector_id, path): (ConnectorId, &str),
    feed_move: Option<&FeedMove>,
) -> Result<Option<String>, PipelineError> {
    if let Some(url) = collection.feed(id)?.public_url {
        return Ok(Some(url));
    }
    let (connector_id, path) =
        feed_move.map_or((connector_id, path), |m| (m.connector_id, m.path.as_str()));
    Ok(hosts.get(connector_id)?.public_url(path))
}

/// Confirm each new file of `operations`, and the feed file: the third
/// step of [`write_draft`], before any deletion (ADR 0036 item 15,
/// ADR 0043 items 7 and 8).
///
/// A new file is an upload, a copy or a confirmation. `stat` of its host
/// must give a file with the size of the operation. Then the public
/// address of the file must pass `addresses`, when the file has one.
/// The function also requests `feed_url`, when it is not `None`. It
/// examines each file before it gives an error. It does not read the
/// content of a file.
///
/// # Errors
///
/// Gives [`PipelineErrorKind::NotReachable`] with the name of each file
/// that did not pass.
fn confirm_new_addresses(
    hosts: &Hosts,
    addresses: &dyn AddressCheck,
    operations: &[HostOperation],
    feed_url: Option<&str>,
) -> Result<(), PipelineError> {
    let mut failed: Vec<String> = Vec::new();
    for operation in operations {
        if !matches!(
            operation.kind,
            OperationKind::Upload | OperationKind::Copy | OperationKind::Confirm
        ) {
            continue;
        }
        let target = hosts.get(operation.connector_id)?;
        let listed = target.stat(&operation.path)?;
        match listed {
            None => {
                failed.push(format!("{} is not on its file host", operation.path));
                continue;
            }
            Some(file) if file.size != operation.size => {
                failed.push(format!(
                    "{} has {} bytes on its file host, not {}",
                    operation.path, file.size, operation.size
                ));
                continue;
            }
            Some(_) => {}
        }
        // A placeholder address has no public address to check (ADR 0122
        // item 12).
        if let Some(url) = operation.url.as_deref()
            && !inventory::is_placeholder_address(url)
            && let Err(reason) = addresses.check(url)
        {
            failed.push(format!("{url} does not pass: {reason}"));
        }
    }
    if let Some(url) = feed_url
        && let Err(reason) = addresses.check(url)
    {
        failed.push(format!("{url} does not pass: {reason}"));
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(PipelineError::new(
            PipelineErrorKind::NotReachable,
            format!(
                "these files did not pass the check, and Studio deleted nothing: {}",
                failed.join("; ")
            ),
        ))
    }
}

/// Keep the draft after a confirmation that failed (ADR 0043 item 9).
/// Then scan each host in `touched`.
///
/// The draft XML does not change. The written version and the written
/// bytes become the base of the draft. The draft loses its copies and its
/// feed move, and keeps its deletions. It gets a confirmation for each
/// upload, copy and confirmation of `operations`, so that the next write
/// requests the files again.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails.
fn keep_draft_after_failed_confirmation(
    collection: &Collection,
    hosts: &Hosts,
    id: FeedId,
    draft: &DraftRow,
    version: &Version,
    operations: &[HostOperation],
    touched: &HashSet<ConnectorId>,
) -> Result<(), PipelineError> {
    collection.save_draft(
        id,
        &DraftRow {
            base_version: Some(version.clone()),
            base_xml: draft.draft_xml.clone(),
            draft_xml: draft.draft_xml.clone(),
        },
    )?;
    confirm_files_on_next_write(collection, id, operations)?;
    collection.remove_draft_feed_move(id)?;
    for &connector_id in touched {
        inventory::scan(collection, hosts, connector_id)?;
    }
    Ok(())
}

/// Give the draft a confirmation for each upload, copy and confirmation of
/// `operations`, so that the next write requests the files again. The
/// draft loses its copies, and keeps its deletions.
///
/// # Errors
///
/// Gives a [`PipelineError`] when the collection fails.
fn confirm_files_on_next_write(
    collection: &Collection,
    id: FeedId,
    operations: &[HostOperation],
) -> Result<(), PipelineError> {
    for operation in collection.draft_operations(id)? {
        if matches!(
            operation.kind,
            DraftOperationKind::Copy | DraftOperationKind::Confirm
        ) {
            collection.remove_draft_operation(id, operation.id)?;
        }
    }
    for operation in operations {
        if matches!(
            operation.kind,
            OperationKind::Upload | OperationKind::Copy | OperationKind::Confirm
        ) {
            collection.add_draft_operation(
                id,
                DraftOperationKind::Confirm,
                operation.library_file_id.map(LibraryFileId),
                operation.connector_id,
                &operation.path,
            )?;
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
    deleted_on: &mut HashSet<ConnectorId>,
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
                collection.remove_placed_feed_file(operation.connector_id, &operation.path)?;
                touched.insert(operation.connector_id);
                deleted_on.insert(operation.connector_id);
            }
            Err(error) if error.kind() == ConnectorErrorKind::Conflict => {
                return Err(PipelineError::new(
                    PipelineErrorKind::Conflict,
                    format!("{} changed on the file host after the scan", operation.path),
                ));
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

/// Remove the host operations that the user started in the draft of a
/// feed (ADR 0036 item 12), and the show grants that the draft asks for
/// (ADR 0054 item 11).
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
    collection.remove_draft_feed_move(id)?;
    collection.remove_signing_plan(id)?;
    collection.remove_draft_grants(id)?;
    // The draft is gone, so each open change set row of the feed is closed
    // and a later Discard of the change set does not put back an old draft
    // (ADR 0091 item 20).
    collection.mark_change_set_written(id, crate::collection::now())?;
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

/// The draft with one channel cover (ADR 0060 item 8). When the channel
/// `itunes:image` and the `url` of the RSS `<image>` are two different
/// addresses, the `url` gets the address of `itunes:image`. The diff and
/// the write both use these bytes, so the diff shows the change.
fn one_cover(mut draft: DraftRow) -> Result<DraftRow, PipelineError> {
    let mut doc = Document::parse_bytes(&draft.draft_xml)?;
    let mut feed = Feed::read(&doc)?;
    let present = |text: &Option<String>| text.as_deref().is_some_and(|t| !t.is_empty());
    let Some(rss) = feed.rss_image.as_mut() else {
        return Ok(draft);
    };
    if !present(&feed.image) || !present(&rss.url) || rss.url == feed.image {
        return Ok(draft);
    }
    rss.url.clone_from(&feed.image);
    feed.write(&mut doc)?;
    draft.draft_xml = doc.write().into_bytes();
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

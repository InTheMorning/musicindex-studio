//! The collection of a user in SQLite (ADR 0005, ADR 0014).
//!
//! The collection keeps the connectors, the feeds, the item GUIDs of each
//! feed, the drafts and the snapshots. It keeps no secret: a connector row
//! gives only the ID of its vault record.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use serde_json::Value;

use crate::address::{self, Address, AddressSource};
use crate::connector::{Stamp, Version};
use crate::feed::{Feed, Recipient};

/// The schema migrations. Migration `n` takes the schema from version `n`
/// to version `n + 1` (ADR 0014).
const MIGRATIONS: &[&str] = &[
    r"
CREATE TABLE connectors (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    config TEXT NOT NULL,
    secret_ref TEXT,
    created_at INTEGER NOT NULL
);
CREATE TABLE feeds (
    id INTEGER PRIMARY KEY,
    public_url TEXT,
    connector_id INTEGER REFERENCES connectors(id),
    path TEXT,
    podcast_guid TEXT,
    title TEXT,
    medium TEXT,
    last_version TEXT,
    last_read_at INTEGER,
    created_at INTEGER NOT NULL
);
CREATE TABLE feed_items (
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    guid TEXT NOT NULL,
    title TEXT,
    PRIMARY KEY (feed_id, position)
);
CREATE TABLE drafts (
    feed_id INTEGER PRIMARY KEY REFERENCES feeds(id) ON DELETE CASCADE,
    base_version TEXT,
    base_xml BLOB NOT NULL,
    draft_xml BLOB NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE TABLE snapshots (
    id INTEGER PRIMARY KEY,
    feed_id INTEGER NOT NULL REFERENCES feeds(id),
    version TEXT NOT NULL,
    xml BLOB NOT NULL,
    taken_at INTEGER NOT NULL
);
CREATE TRIGGER snapshots_do_not_change BEFORE UPDATE ON snapshots
BEGIN
    SELECT RAISE(ABORT, 'ADR 0014: a snapshot row does not change after it is made');
END;
",
    r"
CREATE TABLE feed_addresses (
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    source TEXT NOT NULL,
    url TEXT NOT NULL,
    PRIMARY KEY (feed_id, source)
);
ALTER TABLE feeds ADD COLUMN address_source TEXT;
INSERT INTO feed_addresses (feed_id, source, url)
SELECT f.id,
    CASE WHEN c.kind = 'local'
        AND f.public_url = rtrim(json_extract(c.config, '$.base_url'), '/') || '/' || f.path
    THEN 'folder' ELSE 'read' END,
    f.public_url
FROM feeds f LEFT JOIN connectors c ON c.id = f.connector_id
WHERE f.public_url IS NOT NULL;
UPDATE feeds SET address_source = (SELECT source FROM feed_addresses WHERE feed_id = feeds.id);
",
    r"
CREATE TABLE feed_recipients (
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    address TEXT NOT NULL,
    custom_key TEXT NOT NULL DEFAULT '',
    custom_value TEXT NOT NULL DEFAULT '',
    name TEXT,
    recipient_type TEXT,
    seen_at INTEGER NOT NULL,
    PRIMARY KEY (feed_id, address, custom_key, custom_value)
);
CREATE TABLE saved_recipients (
    id INTEGER PRIMARY KEY,
    address TEXT NOT NULL,
    custom_key TEXT NOT NULL DEFAULT '',
    custom_value TEXT NOT NULL DEFAULT '',
    name TEXT,
    recipient_type TEXT,
    saved_at INTEGER NOT NULL,
    UNIQUE (address, custom_key, custom_value)
);
",
    r"
CREATE TABLE draft_files (
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    size INTEGER NOT NULL,
    media_type TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    added_at INTEGER NOT NULL,
    PRIMARY KEY (feed_id, path)
);
",
    r"
CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
ALTER TABLE feeds ADD COLUMN folder TEXT;
ALTER TABLE feeds ADD COLUMN files_connector_id INTEGER REFERENCES connectors(id);
CREATE UNIQUE INDEX feeds_folder ON feeds(folder) WHERE folder IS NOT NULL;
CREATE TABLE library_files (
    id INTEGER PRIMARY KEY,
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    size INTEGER NOT NULL,
    sha256 TEXT NOT NULL,
    media_type TEXT NOT NULL,
    local INTEGER NOT NULL,
    source_url TEXT,
    connector_id INTEGER REFERENCES connectors(id),
    added_at INTEGER NOT NULL,
    UNIQUE (feed_id, path)
);
CREATE TABLE host_files (
    connector_id INTEGER NOT NULL REFERENCES connectors(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    size INTEGER NOT NULL,
    version TEXT NOT NULL,
    sha256 TEXT,
    seen_at INTEGER NOT NULL,
    PRIMARY KEY (connector_id, path)
);
CREATE TABLE draft_operations (
    id INTEGER PRIMARY KEY,
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('copy', 'delete')),
    library_file_id INTEGER REFERENCES library_files(id) ON DELETE CASCADE,
    connector_id INTEGER NOT NULL REFERENCES connectors(id),
    path TEXT NOT NULL,
    added_at INTEGER NOT NULL
);
",
];

/// The schema version that this code writes.
pub const SCHEMA_VERSION: usize = MIGRATIONS.len();

/// The kind of a [`CollectionError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CollectionErrorKind {
    /// The row does not exist.
    NotFound,
    /// Other rows use the row, so it cannot go out.
    InUse,
    /// The collection refuses the sent value.
    Invalid,
    /// SQLite failed, or the schema is not known.
    Database,
}

/// An error from the collection database.
#[derive(Debug)]
pub struct CollectionError {
    kind: CollectionErrorKind,
    message: String,
}

impl CollectionError {
    fn new(message: impl Into<String>) -> Self {
        Self::with_kind(CollectionErrorKind::Database, message)
    }

    fn with_kind(kind: CollectionErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// The kind of the error.
    #[must_use]
    pub fn kind(&self) -> CollectionErrorKind {
        self.kind
    }
}

impl fmt::Display for CollectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CollectionError {}

impl From<rusqlite::Error> for CollectionError {
    fn from(error: rusqlite::Error) -> Self {
        Self::new(error.to_string())
    }
}

/// The ID of a connector in the collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct ConnectorId(pub i64);

/// The ID of a feed in the collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct FeedId(pub i64);

/// The ID of a snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct SnapshotId(pub i64);

/// The ID of a library file in the collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct LibraryFileId(pub i64);

/// The ID of a draft operation in the collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct DraftOperationId(pub i64);

/// A connector as the collection keeps it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConnectorRow {
    /// The ID.
    pub id: ConnectorId,
    /// The connector kind, such as `local`.
    pub kind: String,
    /// A name that the user gives.
    pub name: String,
    /// The configuration as JSON. It contains no secret.
    pub config: String,
    /// The ID of the vault record that holds the secret, if there is one.
    pub secret_ref: Option<String>,
}

/// A feed as the collection keeps it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FeedRow {
    /// The ID.
    pub id: FeedId,
    /// The public address of the feed (ADR 0021).
    pub public_url: Option<String>,
    /// The source of the public address.
    pub address_source: Option<AddressSource>,
    /// Each address of a different source that is different from the public
    /// address (ADR 0021 items 3 and 4).
    pub address_differences: Vec<Address>,
    /// The connector that writes the feed. `None` means read-only.
    pub connector_id: Option<ConnectorId>,
    /// The path of the feed file in the connector.
    pub path: Option<String>,
    /// `podcast:guid`, from the last read.
    pub podcast_guid: Option<String>,
    /// The title, from the last read.
    pub title: Option<String>,
    /// `podcast:medium`, from the last read.
    pub medium: Option<String>,
    /// The version marker from the last read or write.
    pub last_version: Option<Version>,
    /// The release folder of the feed in the library (ADR 0036 item 4).
    pub folder: Option<String>,
    /// The default host of the files of the release. `None` means that
    /// the write pipeline still decides a default elsewhere (ADR 0036
    /// item 4 and item 8).
    pub files_connector_id: Option<ConnectorId>,
}

/// A draft of a feed (ADR 0007).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftRow {
    /// The version of the live file when the draft started. `None` means
    /// that the file did not exist.
    pub base_version: Option<Version>,
    /// The bytes of the live file when the draft started.
    pub base_xml: Vec<u8>,
    /// The bytes of the draft.
    pub draft_xml: Vec<u8>,
}

/// The facts of a snapshot, without its bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SnapshotInfo {
    /// The ID.
    pub id: SnapshotId,
    /// The version of the file in the snapshot.
    pub version: Version,
    /// The time of the snapshot, in seconds after the Unix epoch.
    pub taken_at: i64,
}

/// A file that an import keeps in the data folder, until the write puts it
/// on the host (ADR 0026 item 5, ADR 0032 items 10 and 11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DraftFile {
    /// The path of the file in the release folder.
    pub path: String,
    /// The size of the file, in bytes.
    pub size: u64,
    /// The media type, such as `audio/mpeg`.
    pub media_type: String,
    /// The SHA-256 of the file, as the name of its copy in the data folder.
    pub sha256: String,
}

/// A file of a release folder in the library (ADR 0036 item 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LibraryFile {
    /// The ID.
    pub id: LibraryFileId,
    /// The feed of the release.
    pub feed_id: FeedId,
    /// The path of the file in the release folder.
    pub path: String,
    /// The size of the file, in bytes.
    pub size: u64,
    /// The SHA-256 of the file.
    pub sha256: String,
    /// The media type, such as `audio/mpeg`.
    pub media_type: String,
    /// True when the bytes of the file are in the library.
    pub local: bool,
    /// The address that the file came from, when Studio knows one.
    pub source_url: Option<String>,
    /// The host that the user selected for the file. `None` means the
    /// default host of the release (ADR 0036 item 4 and item 8).
    pub connector_id: Option<ConnectorId>,
    /// The time that Studio added the row, in seconds after the Unix
    /// epoch.
    pub added_at: i64,
}

/// A file that Studio saw on a host, kept in the inventory (ADR 0036 item
/// 4 and item 6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HostFile {
    /// The host.
    pub connector_id: ConnectorId,
    /// The path of the file on the host.
    pub path: String,
    /// The size of the file, in bytes.
    pub size: u64,
    /// The stamp of the file from the host list. The column has the name
    /// `version`.
    pub stamp: Stamp,
    /// The SHA-256 of the file, when Studio read it. Studio reads it only
    /// when the size agrees with a library file (ADR 0036 item 6).
    pub sha256: Option<String>,
    /// The time that Studio last saw the file, in seconds after the Unix
    /// epoch.
    pub seen_at: i64,
}

/// The kind of a [`DraftOperation`] (ADR 0036 item 4 and item 12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftOperationKind {
    /// Copy a library file to a host.
    Copy,
    /// Delete a file from a host.
    Delete,
}

impl DraftOperationKind {
    /// The text of the kind in the collection.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Copy => "copy",
            Self::Delete => "delete",
        }
    }

    /// The kind for its text in the collection.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "copy" => Some(Self::Copy),
            "delete" => Some(Self::Delete),
            _ => None,
        }
    }
}

/// A host operation that the user started in a draft: a copy of a
/// library file to a host, or a deletion of a host file (ADR 0036 item 4
/// and item 12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DraftOperation {
    /// The ID.
    pub id: DraftOperationId,
    /// The feed of the draft.
    pub feed_id: FeedId,
    /// The kind of operation.
    pub kind: DraftOperationKind,
    /// The library file to copy. `None` for a deletion.
    pub library_file_id: Option<LibraryFileId>,
    /// The host.
    pub connector_id: ConnectorId,
    /// The path of the file on the host.
    pub path: String,
    /// The time that the user started the operation, in seconds after
    /// the Unix epoch.
    pub added_at: i64,
}

/// A recipient that the collection knows, from a feed that Studio read, or
/// from a save by the user (ADR 0027).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KnownRecipient {
    /// The name, from the most recent read or save.
    pub name: Option<String>,
    /// The recipient type, from the most recent read or save.
    pub recipient_type: Option<String>,
    /// The payment address.
    pub address: String,
    /// `customKey`, when the recipient has one.
    pub custom_key: Option<String>,
    /// `customValue`, when the recipient has one.
    pub custom_value: Option<String>,
    /// The number of feeds of the collection that have this recipient.
    pub feeds: usize,
    /// The ID of the saved recipient row, when the user saved it.
    pub saved_id: Option<i64>,
}

/// The collection database.
pub struct Collection {
    db: Connection,
}

impl fmt::Debug for Collection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Collection").finish_non_exhaustive()
    }
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

impl Collection {
    /// Open a collection file, and make or migrate its schema.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when the file cannot be opened, or when it
    /// has a schema version newer than this code.
    pub fn open(path: &Path) -> Result<Self, CollectionError> {
        Self::prepare(Connection::open(path)?)
    }

    /// Open a collection in memory, for tests and previews.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn open_in_memory() -> Result<Self, CollectionError> {
        Self::prepare(Connection::open_in_memory()?)
    }

    fn prepare(db: Connection) -> Result<Self, CollectionError> {
        db.pragma_update(None, "foreign_keys", true)?;
        let mut collection = Self { db };
        collection.migrate()?;
        Ok(collection)
    }

    /// The schema version of the database.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn schema_version(&self) -> Result<usize, CollectionError> {
        let version: i64 = self
            .db
            .pragma_query_value(None, "user_version", |r| r.get(0))?;
        usize::try_from(version).map_err(|_| CollectionError::new("the schema version is negative"))
    }

    fn migrate(&mut self) -> Result<(), CollectionError> {
        let current = self.schema_version()?;
        if current > SCHEMA_VERSION {
            return Err(CollectionError::new(format!(
                "the collection has schema version {current}, and this Studio knows only {SCHEMA_VERSION}"
            )));
        }
        for (index, migration) in MIGRATIONS.iter().enumerate().skip(current) {
            let tx = self.db.transaction()?;
            tx.execute_batch(migration)?;
            let version = i64::try_from(index + 1)
                .map_err(|_| CollectionError::new("too many migrations"))?;
            tx.pragma_update(None, "user_version", version)?;
            tx.commit()?;
        }
        Ok(())
    }

    /// A setting value, or `None` when it is not set (ADR 0036 item 1).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails, or when the stored
    /// value is not valid JSON.
    pub fn setting(&self, key: &str) -> Result<Option<Value>, CollectionError> {
        let text: Option<String> = self
            .db
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
                r.get(0)
            })
            .optional()?;
        text.map(|text| {
            serde_json::from_str(&text)
                .map_err(|e| CollectionError::new(format!("setting {key:?}: {e}")))
        })
        .transpose()
    }

    /// Keep a setting value (ADR 0036 item 1).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn set_setting(&self, key: &str, value: &Value) -> Result<(), CollectionError> {
        let text = value.to_string();
        self.db.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT (key) DO UPDATE SET value = ?2",
            params![key, text],
        )?;
        Ok(())
    }

    /// Add a connector. The configuration must contain no secret.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn add_connector(
        &self,
        kind: &str,
        name: &str,
        config: &str,
        secret_ref: Option<&str>,
    ) -> Result<ConnectorId, CollectionError> {
        self.db.execute(
            "INSERT INTO connectors (kind, name, config, secret_ref, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![kind, name, config, secret_ref, now()],
        )?;
        Ok(ConnectorId(self.db.last_insert_rowid()))
    }

    /// One connector.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when the connector does not exist.
    pub fn connector(&self, id: ConnectorId) -> Result<ConnectorRow, CollectionError> {
        self.db
            .query_row(
                "SELECT id, kind, name, config, secret_ref FROM connectors WHERE id = ?1",
                [id.0],
                connector_row,
            )
            .optional()?
            .ok_or_else(|| {
                CollectionError::with_kind(
                    CollectionErrorKind::NotFound,
                    format!("connector {} does not exist", id.0),
                )
            })
    }

    /// All connectors.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn connectors(&self) -> Result<Vec<ConnectorRow>, CollectionError> {
        let mut statement = self
            .db
            .prepare("SELECT id, kind, name, config, secret_ref FROM connectors ORDER BY id")?;
        let rows = statement.query_map([], connector_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Add a feed. A feed with no connector is read-only.
    ///
    /// `read_url` is the address where Studio read the feed, after each
    /// redirect (ADR 0021 source 1).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn add_feed(
        &mut self,
        read_url: Option<&str>,
        connector: Option<(ConnectorId, &str)>,
    ) -> Result<FeedId, CollectionError> {
        self.db.execute(
            "INSERT INTO feeds (connector_id, path, created_at) VALUES (?1, ?2, ?3)",
            params![
                connector.map(|(c, _)| c.0),
                connector.map(|(_, p)| p),
                now()
            ],
        )?;
        let id = FeedId(self.db.last_insert_rowid());
        self.set_address(id, AddressSource::Read, read_url)?;
        Ok(id)
    }

    /// Put the title and the medium of a new feed into its row, before
    /// Studio ever reads or writes its file. The tree and the editor can
    /// then show the feed before its first write (ADR 0032 item 7).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn set_new_feed(
        &self,
        id: FeedId,
        title: &str,
        medium: &str,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "UPDATE feeds SET title = ?2, medium = ?3 WHERE id = ?1",
            params![id.0, title, medium],
        )?;
        Ok(())
    }

    /// Keep the address that one source gives for a feed, or remove it with
    /// `None`. The public address then comes from the first source that
    /// gives one (ADR 0021).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn set_address(
        &mut self,
        id: FeedId,
        source: AddressSource,
        url: Option<&str>,
    ) -> Result<(), CollectionError> {
        let tx = self.db.transaction()?;
        put_address(&tx, id, source, url)?;
        choose_address(&tx, id)?;
        tx.commit()?;
        Ok(())
    }

    /// One feed.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when the feed does not exist.
    pub fn feed(&self, id: FeedId) -> Result<FeedRow, CollectionError> {
        let mut row = self
            .db
            .query_row(&format!("{FEED_SELECT} WHERE id = ?1"), [id.0], feed_row)
            .optional()?
            .ok_or_else(|| {
                CollectionError::with_kind(
                    CollectionErrorKind::NotFound,
                    format!("feed {} does not exist", id.0),
                )
            })?;
        row.address_differences = address::differences(&addresses(&self.db, id)?);
        Ok(row)
    }

    /// All feeds.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn feeds(&self) -> Result<Vec<FeedRow>, CollectionError> {
        let mut statement = self.db.prepare(&format!("{FEED_SELECT} ORDER BY id"))?;
        let mut rows: Vec<FeedRow> = statement
            .query_map([], feed_row)?
            .collect::<Result<_, _>>()?;
        for row in &mut rows {
            row.address_differences = address::differences(&addresses(&self.db, row.id)?);
        }
        Ok(rows)
    }

    /// Keep the facts of a feed after a read or a write: its identity, its
    /// title, its version, the GUID of each item (ADR 0008), and the
    /// addresses that the feed gives for itself (ADR 0021).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn record_state(
        &mut self,
        id: FeedId,
        feed: &Feed,
        version: &Version,
    ) -> Result<(), CollectionError> {
        let tx = self.db.transaction()?;
        tx.execute(
            "UPDATE feeds SET podcast_guid = ?2, title = ?3, medium = ?4, last_version = ?5, last_read_at = ?6 WHERE id = ?1",
            params![
                id.0,
                feed.podcast_guid,
                feed.title,
                feed.medium,
                version.as_str(),
                now()
            ],
        )?;
        tx.execute("DELETE FROM feed_items WHERE feed_id = ?1", [id.0])?;
        for (position, item) in (0_i64..).zip(feed.items.iter()) {
            let Some(guid) = &item.guid else { continue };
            tx.execute(
                "INSERT INTO feed_items (feed_id, position, guid, title) VALUES (?1, ?2, ?3, ?4)",
                params![id.0, position, guid, item.title],
            )?;
        }
        put_address(&tx, id, AddressSource::SelfLink, feed.self_url())?;
        put_address(
            &tx,
            id,
            AddressSource::NewFeedUrl,
            feed.new_feed_url.as_deref(),
        )?;
        choose_address(&tx, id)?;
        put_recipients(&tx, id, feed.recipients())?;
        tx.commit()?;
        Ok(())
    }

    /// The item GUIDs of a feed, in feed sequence.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn item_guids(&self, id: FeedId) -> Result<Vec<String>, CollectionError> {
        let mut statement = self
            .db
            .prepare("SELECT guid FROM feed_items WHERE feed_id = ?1 ORDER BY position")?;
        let rows = statement.query_map([id.0], |r| r.get(0))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// The draft of a feed.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn draft(&self, id: FeedId) -> Result<Option<DraftRow>, CollectionError> {
        Ok(self
            .db
            .query_row(
                "SELECT base_version, base_xml, draft_xml FROM drafts WHERE feed_id = ?1",
                [id.0],
                |r| {
                    Ok(DraftRow {
                        base_version: r.get::<_, Option<String>>(0)?.map(Version::new),
                        base_xml: r.get(1)?,
                        draft_xml: r.get(2)?,
                    })
                },
            )
            .optional()?)
    }

    /// Add or replace the draft of a feed.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn save_draft(&self, id: FeedId, draft: &DraftRow) -> Result<(), CollectionError> {
        self.db.execute(
            "INSERT INTO drafts (feed_id, base_version, base_xml, draft_xml, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT (feed_id) DO UPDATE SET base_version = ?2, base_xml = ?3, draft_xml = ?4, updated_at = ?5",
            params![
                id.0,
                draft.base_version.as_ref().map(Version::as_str),
                draft.base_xml,
                draft.draft_xml,
                now()
            ],
        )?;
        Ok(())
    }

    /// Give a feed the connector and the path that write it.
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::NotFound`] when the feed or the
    /// connector does not exist.
    pub fn set_feed_connector(
        &self,
        id: FeedId,
        connector: ConnectorId,
        path: &str,
    ) -> Result<(), CollectionError> {
        self.feed(id)?;
        self.connector(connector)?;
        self.db.execute(
            "UPDATE feeds SET connector_id = ?2, path = ?3 WHERE id = ?1",
            params![id.0, connector.0, path],
        )?;
        Ok(())
    }

    /// Give a feed its release folder in the library (ADR 0035 item 2,
    /// ADR 0036 item 4).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails, or when a different
    /// feed already has this folder.
    pub fn set_feed_folder(&self, id: FeedId, folder: &str) -> Result<(), CollectionError> {
        self.db.execute(
            "UPDATE feeds SET folder = ?2 WHERE id = ?1",
            params![id.0, folder],
        )?;
        Ok(())
    }

    /// Give a feed its default host for files. `None` means that the
    /// write pipeline decides the default elsewhere (ADR 0036 item 4 and
    /// item 8).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn set_files_connector(
        &self,
        id: FeedId,
        connector: Option<ConnectorId>,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "UPDATE feeds SET files_connector_id = ?2 WHERE id = ?1",
            params![id.0, connector.map(|c| c.0)],
        )?;
        Ok(())
    }

    /// Replace the configuration of a connector. The configuration must
    /// contain no secret.
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::NotFound`] when the connector does not
    /// exist.
    pub fn set_connector_config(
        &self,
        id: ConnectorId,
        config: &str,
    ) -> Result<(), CollectionError> {
        self.connector(id)?;
        self.db.execute(
            "UPDATE connectors SET config = ?2 WHERE id = ?1",
            params![id.0, config],
        )?;
        Ok(())
    }

    /// Remove a feed from the collection, with its item GUIDs, its draft and
    /// its snapshots. The feed file on the host does not change.
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::NotFound`] when the feed does not exist.
    pub fn delete_feed(&mut self, id: FeedId) -> Result<(), CollectionError> {
        self.feed(id)?;
        let tx = self.db.transaction()?;
        tx.execute("DELETE FROM snapshots WHERE feed_id = ?1", [id.0])?;
        tx.execute("DELETE FROM feeds WHERE id = ?1", [id.0])?;
        tx.commit()?;
        Ok(())
    }

    /// Remove a connector that no feed uses.
    ///
    /// The caller removes the vault record of the connector.
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::InUse`] when a feed uses the connector,
    /// and [`CollectionErrorKind::NotFound`] when it does not exist.
    pub fn delete_connector(&self, id: ConnectorId) -> Result<(), CollectionError> {
        self.connector(id)?;
        let users: i64 = self.db.query_row(
            "SELECT COUNT(*) FROM feeds WHERE connector_id = ?1",
            [id.0],
            |r| r.get(0),
        )?;
        if users > 0 {
            return Err(CollectionError::with_kind(
                CollectionErrorKind::InUse,
                format!("{users} feeds use connector {}", id.0),
            ));
        }
        self.db
            .execute("DELETE FROM connectors WHERE id = ?1", [id.0])?;
        Ok(())
    }

    /// Remove the draft of a feed.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn delete_draft(&self, id: FeedId) -> Result<(), CollectionError> {
        self.db
            .execute("DELETE FROM drafts WHERE feed_id = ?1", [id.0])?;
        Ok(())
    }

    /// Keep the row of a pending file of a draft (ADR 0026 item 5).
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::InUse`] when the draft has a pending
    /// file at this path already.
    pub fn add_draft_file(&self, id: FeedId, file: &DraftFile) -> Result<(), CollectionError> {
        let size = i64::try_from(file.size).unwrap_or(i64::MAX);
        let changed = self.db.execute(
            "INSERT OR IGNORE INTO draft_files (feed_id, path, size, media_type, sha256, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id.0, file.path, size, file.media_type, file.sha256, now()],
        )?;
        if changed == 0 {
            return Err(CollectionError::with_kind(
                CollectionErrorKind::InUse,
                format!(
                    "the draft of feed {} has a pending file at {}",
                    id.0, file.path
                ),
            ));
        }
        Ok(())
    }

    /// The pending files of a draft, by path (ADR 0026 item 6).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn draft_files(&self, id: FeedId) -> Result<Vec<DraftFile>, CollectionError> {
        let mut statement = self.db.prepare(
            "SELECT path, size, media_type, sha256 FROM draft_files
             WHERE feed_id = ?1 ORDER BY path",
        )?;
        let rows = statement.query_map([id.0], draft_file_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Remove the row of one pending file of a draft. A path with no row is
    /// not an error (ADR 0026 item 8).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn remove_draft_file(&self, id: FeedId, path: &str) -> Result<(), CollectionError> {
        self.db.execute(
            "DELETE FROM draft_files WHERE feed_id = ?1 AND path = ?2",
            params![id.0, path],
        )?;
        Ok(())
    }

    /// Remove the table of the pending files, once `migrate::into_library`
    /// has moved each row into the library (ADR 0036 item 5). A second
    /// call does nothing.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub(crate) fn drop_draft_files_table(&self) -> Result<(), CollectionError> {
        self.db.execute("DROP TABLE IF EXISTS draft_files", [])?;
        Ok(())
    }

    /// Keep the row of a file of a release folder (ADR 0036 item 4).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails, or when the feed
    /// already has a file at this path.
    #[allow(clippy::too_many_arguments)]
    pub fn add_library_file(
        &self,
        feed: FeedId,
        path: &str,
        size: u64,
        sha256: &str,
        media_type: &str,
        local: bool,
        source_url: Option<&str>,
    ) -> Result<LibraryFileId, CollectionError> {
        let size = i64::try_from(size).unwrap_or(i64::MAX);
        self.db.execute(
            "INSERT INTO library_files (feed_id, path, size, sha256, media_type, local, source_url, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![feed.0, path, size, sha256, media_type, local, source_url, now()],
        )?;
        Ok(LibraryFileId(self.db.last_insert_rowid()))
    }

    /// Each file of a release folder, by path (ADR 0036 item 4).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn library_files(&self, feed: FeedId) -> Result<Vec<LibraryFile>, CollectionError> {
        let mut statement = self.db.prepare(&format!(
            "{LIBRARY_FILE_SELECT} WHERE feed_id = ?1 ORDER BY path"
        ))?;
        let rows = statement.query_map([feed.0], library_file_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// One file of a release folder.
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::NotFound`] when the file does not
    /// exist.
    pub fn library_file(&self, id: LibraryFileId) -> Result<LibraryFile, CollectionError> {
        self.db
            .query_row(
                &format!("{LIBRARY_FILE_SELECT} WHERE id = ?1"),
                [id.0],
                library_file_row,
            )
            .optional()?
            .ok_or_else(|| {
                CollectionError::with_kind(
                    CollectionErrorKind::NotFound,
                    format!("library file {} does not exist", id.0),
                )
            })
    }

    /// Give a library file the host that the user selected, or `None`
    /// for the default host of the release (ADR 0036 item 4 and item 8).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn set_library_file_connector(
        &self,
        id: LibraryFileId,
        connector: Option<ConnectorId>,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "UPDATE library_files SET connector_id = ?2 WHERE id = ?1",
            params![id.0, connector.map(|c| c.0)],
        )?;
        Ok(())
    }

    /// Remove the row of a file of a release folder. The file in the
    /// folder does not change.
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::NotFound`] when the file does not
    /// exist.
    pub fn remove_library_file(&self, id: LibraryFileId) -> Result<(), CollectionError> {
        let changed = self
            .db
            .execute("DELETE FROM library_files WHERE id = ?1", [id.0])?;
        if changed == 0 {
            return Err(CollectionError::with_kind(
                CollectionErrorKind::NotFound,
                format!("library file {} does not exist", id.0),
            ));
        }
        Ok(())
    }

    /// Keep the row of a file that Studio saw on a host. A later call
    /// with the same host and path replaces the row, and clears a
    /// SHA-256 from an earlier call (ADR 0036 item 6).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn put_host_file(
        &self,
        connector: ConnectorId,
        path: &str,
        size: u64,
        stamp: &Stamp,
    ) -> Result<(), CollectionError> {
        let size = i64::try_from(size).unwrap_or(i64::MAX);
        self.db.execute(
            "INSERT INTO host_files (connector_id, path, size, version, sha256, seen_at)
             VALUES (?1, ?2, ?3, ?4, NULL, ?5)
             ON CONFLICT (connector_id, path)
             DO UPDATE SET size = ?3, version = ?4, seen_at = ?5,
                 sha256 = CASE WHEN host_files.version = ?4 THEN host_files.sha256 ELSE NULL END",
            params![connector.0, path, size, stamp.as_str(), now()],
        )?;
        Ok(())
    }

    /// One file that Studio saw on a host, or `None` when it has no row
    /// at this path (ADR 0036 item 4 and item 6).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn host_file(
        &self,
        connector: ConnectorId,
        path: &str,
    ) -> Result<Option<HostFile>, CollectionError> {
        Ok(self
            .db
            .query_row(
                "SELECT connector_id, path, size, version, sha256, seen_at FROM host_files
                 WHERE connector_id = ?1 AND path = ?2",
                params![connector.0, path],
                host_file_row,
            )
            .optional()?)
    }

    /// Each file that Studio saw on a host, by path (ADR 0036 item 4 and
    /// item 6).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn host_files(&self, connector: ConnectorId) -> Result<Vec<HostFile>, CollectionError> {
        let mut statement = self.db.prepare(
            "SELECT connector_id, path, size, version, sha256, seen_at FROM host_files
             WHERE connector_id = ?1 ORDER BY path",
        )?;
        let rows = statement.query_map([connector.0], host_file_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Remove the row of a file that Studio no longer sees on a host. A
    /// path with no row is not an error.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn remove_host_file(
        &self,
        connector: ConnectorId,
        path: &str,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "DELETE FROM host_files WHERE connector_id = ?1 AND path = ?2",
            params![connector.0, path],
        )?;
        Ok(())
    }

    /// Keep the SHA-256 that Studio read from a host file. Studio reads
    /// it only when the size agrees with a library file (ADR 0036 item
    /// 6).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn set_host_file_sha(
        &self,
        connector: ConnectorId,
        path: &str,
        sha256: &str,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "UPDATE host_files SET sha256 = ?3 WHERE connector_id = ?1 AND path = ?2",
            params![connector.0, path, sha256],
        )?;
        Ok(())
    }

    /// Keep the row of a host operation that the user started in a draft
    /// (ADR 0036 item 4 and item 12).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn add_draft_operation(
        &self,
        feed: FeedId,
        kind: DraftOperationKind,
        library_file: Option<LibraryFileId>,
        connector: ConnectorId,
        path: &str,
    ) -> Result<DraftOperationId, CollectionError> {
        self.db.execute(
            "INSERT INTO draft_operations (feed_id, kind, library_file_id, connector_id, path, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                feed.0,
                kind.as_str(),
                library_file.map(|f| f.0),
                connector.0,
                path,
                now()
            ],
        )?;
        Ok(DraftOperationId(self.db.last_insert_rowid()))
    }

    /// Each host operation of a draft, in the sequence that the user
    /// started them (ADR 0036 item 4 and item 12).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn draft_operations(&self, feed: FeedId) -> Result<Vec<DraftOperation>, CollectionError> {
        let mut statement = self.db.prepare(
            "SELECT id, feed_id, kind, library_file_id, connector_id, path, added_at
             FROM draft_operations WHERE feed_id = ?1 ORDER BY id",
        )?;
        let rows = statement.query_map([feed.0], draft_operation_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Remove each host operation of a draft (ADR 0036 item 5).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn remove_draft_operations(&self, feed: FeedId) -> Result<(), CollectionError> {
        self.db
            .execute("DELETE FROM draft_operations WHERE feed_id = ?1", [feed.0])?;
        Ok(())
    }

    /// Keep a snapshot of a live file. A snapshot does not change after this.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn add_snapshot(
        &self,
        id: FeedId,
        version: &Version,
        xml: &[u8],
    ) -> Result<SnapshotId, CollectionError> {
        self.db.execute(
            "INSERT INTO snapshots (feed_id, version, xml, taken_at) VALUES (?1, ?2, ?3, ?4)",
            params![id.0, version.as_str(), xml, now()],
        )?;
        Ok(SnapshotId(self.db.last_insert_rowid()))
    }

    /// The snapshots of a feed, the newest first.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn snapshots(&self, id: FeedId) -> Result<Vec<SnapshotInfo>, CollectionError> {
        let mut statement = self.db.prepare(
            "SELECT id, version, taken_at FROM snapshots WHERE feed_id = ?1 ORDER BY id DESC",
        )?;
        let rows = statement.query_map([id.0], |r| {
            Ok(SnapshotInfo {
                id: SnapshotId(r.get(0)?),
                version: Version::new(r.get::<_, String>(1)?),
                taken_at: r.get(2)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// The bytes of a snapshot of a feed.
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::NotFound`] when the feed has no such
    /// snapshot.
    pub fn snapshot_xml(
        &self,
        id: FeedId,
        snapshot: SnapshotId,
    ) -> Result<Vec<u8>, CollectionError> {
        self.db
            .query_row(
                "SELECT xml FROM snapshots WHERE id = ?1 AND feed_id = ?2",
                [snapshot.0, id.0],
                |r| r.get(0),
            )
            .optional()?
            .ok_or_else(|| {
                CollectionError::with_kind(
                    CollectionErrorKind::NotFound,
                    format!("feed {} has no snapshot {}", id.0, snapshot.0),
                )
            })
    }

    /// Each recipient that the collection knows: from a feed that Studio
    /// read, or from a save (ADR 0027 items 2 to 5).
    ///
    /// The list has each identity one time. The name and the type come from
    /// the row with the highest `seen_at` or `saved_at`. The sequence is by
    /// name, then by address, with no regard to capital letters.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn known_recipients(&self) -> Result<Vec<KnownRecipient>, CollectionError> {
        let mut found: HashMap<(String, String, String), RecipientAgg> = HashMap::new();

        let mut statement = self.db.prepare(
            "SELECT feed_id, address, custom_key, custom_value, name, recipient_type, seen_at
             FROM feed_recipients",
        )?;
        let rows = statement.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, i64>(6)?,
            ))
        })?;
        for row in rows {
            let (feed_id, address, custom_key, custom_value, name, recipient_type, seen_at) = row?;
            let key = (address.clone(), custom_key.clone(), custom_value.clone());
            let agg = found
                .entry(key)
                .or_insert_with(|| RecipientAgg::new(address, custom_key, custom_value));
            agg.feeds.insert(feed_id);
            agg.consider(seen_at, name, recipient_type);
        }
        drop(statement);

        let mut statement = self.db.prepare(
            "SELECT id, address, custom_key, custom_value, name, recipient_type, saved_at
             FROM saved_recipients",
        )?;
        let rows = statement.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, i64>(6)?,
            ))
        })?;
        for row in rows {
            let (saved_id, address, custom_key, custom_value, name, recipient_type, saved_at) =
                row?;
            let key = (address.clone(), custom_key.clone(), custom_value.clone());
            let agg = found
                .entry(key)
                .or_insert_with(|| RecipientAgg::new(address, custom_key, custom_value));
            agg.saved_id = Some(saved_id);
            agg.consider(saved_at, name, recipient_type);
        }

        let mut list: Vec<KnownRecipient> = found.into_values().map(RecipientAgg::finish).collect();
        list.sort_by(|a, b| {
            let a_name = a.name.as_deref().unwrap_or_default().to_lowercase();
            let b_name = b.name.as_deref().unwrap_or_default().to_lowercase();
            a_name.cmp(&b_name).then_with(|| a.address.cmp(&b.address))
        });
        Ok(list)
    }

    /// Save a recipient. When its identity is already saved, change the
    /// name and the type of that row (ADR 0027 item 5).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] of kind [`CollectionErrorKind::Invalid`]
    /// when the recipient has no address.
    pub fn save_recipient(&self, recipient: &Recipient) -> Result<i64, CollectionError> {
        // The same rule as `Feed::recipients`: an address of only spaces is no
        // address.
        let Some(address) = recipient
            .address
            .as_deref()
            .filter(|a| !a.trim().is_empty())
        else {
            return Err(CollectionError::with_kind(
                CollectionErrorKind::Invalid,
                "a saved recipient needs an address",
            ));
        };
        let custom_key = recipient.custom_key.clone().unwrap_or_default();
        let custom_value = recipient.custom_value.clone().unwrap_or_default();
        self.db.execute(
            "INSERT INTO saved_recipients (address, custom_key, custom_value, name, recipient_type, saved_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT (address, custom_key, custom_value)
             DO UPDATE SET name = ?4, recipient_type = ?5, saved_at = ?6",
            params![
                address,
                custom_key,
                custom_value,
                recipient.name,
                recipient.recipient_type,
                now()
            ],
        )?;
        Ok(self.db.query_row(
            "SELECT id FROM saved_recipients WHERE address = ?1 AND custom_key = ?2 AND custom_value = ?3",
            params![address, custom_key, custom_value],
            |r| r.get(0),
        )?)
    }

    /// Remove a saved recipient. The `feed_recipients` rows do not
    /// change.
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::NotFound`] when the ID does not exist.
    pub fn remove_saved_recipient(&self, id: i64) -> Result<(), CollectionError> {
        let changed = self
            .db
            .execute("DELETE FROM saved_recipients WHERE id = ?1", [id])?;
        if changed == 0 {
            return Err(CollectionError::with_kind(
                CollectionErrorKind::NotFound,
                format!("saved recipient {id} does not exist"),
            ));
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn db(&self) -> &Connection {
        &self.db
    }
}

/// One recipient identity. [`Collection::known_recipients`] uses it to
/// combine the rows of the two recipient tables.
struct RecipientAgg {
    address: String,
    custom_key: String,
    custom_value: String,
    feeds: HashSet<i64>,
    saved_id: Option<i64>,
    best_at: i64,
    name: Option<String>,
    recipient_type: Option<String>,
}

impl RecipientAgg {
    fn new(address: String, custom_key: String, custom_value: String) -> Self {
        Self {
            address,
            custom_key,
            custom_value,
            feeds: HashSet::new(),
            saved_id: None,
            best_at: i64::MIN,
            name: None,
            recipient_type: None,
        }
    }

    /// Keep `name` and `recipient_type` from the row with the highest `at`
    /// value.
    fn consider(&mut self, at: i64, name: Option<String>, recipient_type: Option<String>) {
        if at >= self.best_at {
            self.best_at = at;
            self.name = name;
            self.recipient_type = recipient_type;
        }
    }

    fn finish(self) -> KnownRecipient {
        KnownRecipient {
            name: self.name,
            recipient_type: self.recipient_type,
            address: self.address,
            custom_key: non_empty(self.custom_key),
            custom_value: non_empty(self.custom_value),
            feeds: self.feeds.len(),
            saved_id: self.saved_id,
        }
    }
}

/// Turn the empty text that the database uses for a missing custom key or
/// custom value back into `None`.
fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

const FEED_SELECT: &str = "SELECT id, public_url, connector_id, path, podcast_guid, title, medium, last_version, address_source, folder, files_connector_id FROM feeds";

/// Keep or remove the address of one source. An empty address is no address.
fn put_address(
    db: &Connection,
    id: FeedId,
    source: AddressSource,
    url: Option<&str>,
) -> Result<(), CollectionError> {
    match url.map(str::trim).filter(|u| !u.is_empty()) {
        Some(url) => db.execute(
            "INSERT INTO feed_addresses (feed_id, source, url) VALUES (?1, ?2, ?3)
             ON CONFLICT (feed_id, source) DO UPDATE SET url = ?3",
            params![id.0, source.as_str(), url],
        )?,
        None => db.execute(
            "DELETE FROM feed_addresses WHERE feed_id = ?1 AND source = ?2",
            params![id.0, source.as_str()],
        )?,
    };
    Ok(())
}

/// Replace the `feed_recipients` rows of a feed with its current
/// recipients (ADR 0027 item 4).
///
/// A recipient that occurs two times in `recipients` gives one row. The
/// last occurrence gives the name and the type.
fn put_recipients(
    db: &Connection,
    id: FeedId,
    recipients: Vec<&Recipient>,
) -> Result<(), CollectionError> {
    db.execute("DELETE FROM feed_recipients WHERE feed_id = ?1", [id.0])?;
    let seen_at = now();
    let mut rows: Vec<RecipientRow> = Vec::new();
    for recipient in recipients {
        // `Feed::recipients` gives only a recipient with an address.
        let Some(address) = recipient.address.clone() else {
            continue;
        };
        let custom_key = recipient.custom_key.clone().unwrap_or_default();
        let custom_value = recipient.custom_value.clone().unwrap_or_default();
        let same = rows.iter_mut().find(|r| {
            r.address == address && r.custom_key == custom_key && r.custom_value == custom_value
        });
        match same {
            // A recipient that occurs two times in one feed gives one row.
            // The last occurrence gives the name and the type.
            Some(row) => {
                row.name.clone_from(&recipient.name);
                row.recipient_type.clone_from(&recipient.recipient_type);
            }
            None => rows.push(RecipientRow {
                address,
                custom_key,
                custom_value,
                name: recipient.name.clone(),
                recipient_type: recipient.recipient_type.clone(),
            }),
        }
    }
    for row in rows {
        db.execute(
            "INSERT INTO feed_recipients
                 (feed_id, address, custom_key, custom_value, name, recipient_type, seen_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id.0,
                row.address,
                row.custom_key,
                row.custom_value,
                row.name,
                row.recipient_type,
                seen_at
            ],
        )?;
    }
    Ok(())
}

/// One row of `feed_recipients`. [`put_recipients`] uses it to give each
/// identity of a feed one row.
struct RecipientRow {
    address: String,
    custom_key: String,
    custom_value: String,
    name: Option<String>,
    recipient_type: Option<String>,
}

/// The address of each source of a feed.
fn addresses(db: &Connection, id: FeedId) -> Result<Vec<Address>, CollectionError> {
    let mut statement = db.prepare("SELECT source, url FROM feed_addresses WHERE feed_id = ?1")?;
    let rows = statement.query_map([id.0], |r| Ok((r.get::<_, String>(0)?, r.get(1)?)))?;
    let mut found = Vec::new();
    for row in rows {
        let (source, url) = row?;
        if let Some(source) = AddressSource::parse(&source) {
            found.push(Address { source, url });
        }
    }
    Ok(found)
}

/// Set the public address of a feed from its sources (ADR 0021 item 1).
fn choose_address(db: &Connection, id: FeedId) -> Result<(), CollectionError> {
    let all = addresses(db, id)?;
    let chosen = address::choose(&all);
    db.execute(
        "UPDATE feeds SET public_url = ?2, address_source = ?3 WHERE id = ?1",
        params![
            id.0,
            chosen.map(|a| a.url.as_str()),
            chosen.map(|a| a.source.as_str())
        ],
    )?;
    Ok(())
}

fn connector_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<ConnectorRow> {
    Ok(ConnectorRow {
        id: ConnectorId(r.get(0)?),
        kind: r.get(1)?,
        name: r.get(2)?,
        config: r.get(3)?,
        secret_ref: r.get(4)?,
    })
}

fn draft_file_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<DraftFile> {
    let size: i64 = r.get(1)?;
    Ok(DraftFile {
        path: r.get(0)?,
        size: u64::try_from(size).unwrap_or(0),
        media_type: r.get(2)?,
        sha256: r.get(3)?,
    })
}

fn feed_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<FeedRow> {
    Ok(FeedRow {
        id: FeedId(r.get(0)?),
        public_url: r.get(1)?,
        connector_id: r.get::<_, Option<i64>>(2)?.map(ConnectorId),
        path: r.get(3)?,
        podcast_guid: r.get(4)?,
        title: r.get(5)?,
        medium: r.get(6)?,
        last_version: r.get::<_, Option<String>>(7)?.map(Version::new),
        address_source: r
            .get::<_, Option<String>>(8)?
            .as_deref()
            .and_then(AddressSource::parse),
        address_differences: Vec::new(),
        folder: r.get(9)?,
        files_connector_id: r.get::<_, Option<i64>>(10)?.map(ConnectorId),
    })
}

fn library_file_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<LibraryFile> {
    let size: i64 = r.get(3)?;
    Ok(LibraryFile {
        id: LibraryFileId(r.get(0)?),
        feed_id: FeedId(r.get(1)?),
        path: r.get(2)?,
        size: u64::try_from(size).unwrap_or(0),
        sha256: r.get(4)?,
        media_type: r.get(5)?,
        local: r.get(6)?,
        source_url: r.get(7)?,
        connector_id: r.get::<_, Option<i64>>(8)?.map(ConnectorId),
        added_at: r.get(9)?,
    })
}

const LIBRARY_FILE_SELECT: &str = "SELECT id, feed_id, path, size, sha256, media_type, local, source_url, connector_id, added_at FROM library_files";

fn host_file_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<HostFile> {
    let size: i64 = r.get(2)?;
    Ok(HostFile {
        connector_id: ConnectorId(r.get(0)?),
        path: r.get(1)?,
        size: u64::try_from(size).unwrap_or(0),
        stamp: Stamp::new(r.get::<_, String>(3)?),
        sha256: r.get(4)?,
        seen_at: r.get(5)?,
    })
}

fn draft_operation_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<DraftOperation> {
    let kind: String = r.get(2)?;
    Ok(DraftOperation {
        id: DraftOperationId(r.get(0)?),
        feed_id: FeedId(r.get(1)?),
        kind: DraftOperationKind::parse(&kind).expect("the kind column allows only copy or delete"),
        library_file_id: r.get::<_, Option<i64>>(3)?.map(LibraryFileId),
        connector_id: ConnectorId(r.get(4)?),
        path: r.get(5)?,
        added_at: r.get(6)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::OpenVault;
    use crate::xml::Document;

    #[test]
    fn a_new_database_gets_the_schema_and_opens_again_with_no_change() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let collection = Collection::open(&path).unwrap();
        assert_eq!(collection.schema_version().unwrap(), SCHEMA_VERSION);
        collection
            .add_connector("local", "Home", "{}", None)
            .unwrap();
        drop(collection);
        let collection = Collection::open(&path).unwrap();
        assert_eq!(collection.schema_version().unwrap(), SCHEMA_VERSION);
        assert_eq!(collection.connectors().unwrap().len(), 1);
    }

    #[test]
    fn the_address_migration_gives_each_old_address_its_source() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = Connection::open(&path).unwrap();
        db.execute_batch(MIGRATIONS[0]).unwrap();
        db.pragma_update(None, "user_version", 1).unwrap();
        db.execute_batch(
            r#"INSERT INTO connectors VALUES (1, 'local', 'Home', '{"root":"/srv","base_url":"https://m.example/"}', NULL, 0);
               INSERT INTO feeds (id, public_url, connector_id, path, created_at) VALUES
                 (1, 'https://m.example/a.xml', 1, 'a.xml', 0),
                 (2, 'https://other.example/b.xml', 1, 'b.xml', 0),
                 (3, 'https://other.example/c.xml', NULL, NULL, 0),
                 (4, NULL, 1, 'd.xml', 0);"#,
        )
        .unwrap();
        drop(db);
        let collection = Collection::open(&path).unwrap();
        let sources: Vec<_> = collection
            .feeds()
            .unwrap()
            .iter()
            .map(|f| (f.public_url.clone(), f.address_source))
            .collect();
        assert_eq!(
            sources,
            [
                (
                    Some("https://m.example/a.xml".to_owned()),
                    Some(AddressSource::Folder)
                ),
                (
                    Some("https://other.example/b.xml".to_owned()),
                    Some(AddressSource::Read)
                ),
                (
                    Some("https://other.example/c.xml".to_owned()),
                    Some(AddressSource::Read)
                ),
                (None, None),
            ]
        );
    }

    #[test]
    fn the_read_address_stays_and_each_other_address_is_a_difference() {
        let mut collection = Collection::open_in_memory().unwrap();
        let id = collection
            .add_feed(Some("https://new.example/a.xml"), None)
            .unwrap();
        let doc = Document::parse(
            "<rss xmlns:atom=\"http://www.w3.org/2005/Atom\" xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\">\
             <channel><title>A</title><itunes:new-feed-url>https://next.example/a.xml</itunes:new-feed-url>\
             <atom:link rel=\"self\" href=\"https://old.example/a.xml\"/></channel></rss>",
        )
        .unwrap();
        collection
            .record_state(id, &Feed::read(&doc).unwrap(), &Version::new("v1"))
            .unwrap();
        let row = collection.feed(id).unwrap();
        assert_eq!(row.public_url.as_deref(), Some("https://new.example/a.xml"));
        assert_eq!(row.address_source, Some(AddressSource::Read));
        let differences: Vec<_> = row
            .address_differences
            .iter()
            .map(|d| (d.source, d.url.as_str()))
            .collect();
        assert!(differences.contains(&(AddressSource::SelfLink, "https://old.example/a.xml")));
        assert!(differences.contains(&(AddressSource::NewFeedUrl, "https://next.example/a.xml")));
        assert_eq!(differences.len(), 2);

        collection
            .set_address(id, AddressSource::Read, None)
            .unwrap();
        let row = collection.feed(id).unwrap();
        assert_eq!(row.public_url.as_deref(), Some("https://old.example/a.xml"));
        assert_eq!(row.address_source, Some(AddressSource::SelfLink));
    }

    #[test]
    fn a_database_from_a_newer_studio_gives_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = Connection::open(&path).unwrap();
        let newer = i64::try_from(SCHEMA_VERSION + 1).unwrap();
        db.pragma_update(None, "user_version", newer).unwrap();
        drop(db);
        assert!(Collection::open(&path).is_err());
    }

    #[test]
    fn the_collection_keeps_connectors_feeds_and_item_guids() {
        let mut collection = Collection::open_in_memory().unwrap();
        let connector = collection
            .add_connector("local", "Home", r#"{"root":"/srv"}"#, None)
            .unwrap();
        let feed_id = collection
            .add_feed(
                Some("https://example.com/a.xml"),
                Some((connector, "a.xml")),
            )
            .unwrap();
        let read_only = collection
            .add_feed(Some("https://example.com/b.xml"), None)
            .unwrap();
        let doc = Document::parse(
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><title>A</title>\
             <podcast:guid>g</podcast:guid><item><guid>one</guid></item><item><guid>two</guid></item>\
             </channel></rss>",
        )
        .unwrap();
        let feed = Feed::read(&doc).unwrap();
        collection
            .record_state(feed_id, &feed, &Version::new("v1"))
            .unwrap();
        let row = collection.feed(feed_id).unwrap();
        assert_eq!(row.connector_id, Some(connector));
        assert_eq!(row.path.as_deref(), Some("a.xml"));
        assert_eq!(row.podcast_guid.as_deref(), Some("g"));
        assert_eq!(row.title.as_deref(), Some("A"));
        assert_eq!(row.last_version, Some(Version::new("v1")));
        assert_eq!(collection.item_guids(feed_id).unwrap(), ["one", "two"]);
        assert_eq!(collection.feed(read_only).unwrap().connector_id, None);
        assert_eq!(collection.feeds().unwrap().len(), 2);
    }

    #[test]
    fn a_feed_goes_out_with_its_rows_and_a_used_connector_stays() {
        let mut collection = Collection::open_in_memory().unwrap();
        let connector = collection
            .add_connector("local", "Home", "{}", None)
            .unwrap();
        let feed = collection
            .add_feed(None, Some((connector, "a.xml")))
            .unwrap();
        collection
            .add_snapshot(feed, &Version::new("v1"), b"<rss/>")
            .unwrap();
        let error = collection.delete_connector(connector).unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::InUse);
        collection.delete_feed(feed).unwrap();
        assert!(collection.feeds().unwrap().is_empty());
        assert_eq!(
            collection.delete_feed(feed).unwrap_err().kind(),
            CollectionErrorKind::NotFound
        );
        collection.delete_connector(connector).unwrap();
        assert!(collection.connectors().unwrap().is_empty());
    }

    #[test]
    fn an_update_of_a_snapshot_gives_an_error() {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        let snapshot = collection
            .add_snapshot(feed, &Version::new("v1"), b"<rss/>")
            .unwrap();
        let error = collection
            .db()
            .execute(
                "UPDATE snapshots SET xml = x'00' WHERE id = ?1",
                [snapshot.0],
            )
            .unwrap_err();
        assert!(error.to_string().contains("ADR 0014"));
        assert_eq!(collection.snapshot_xml(feed, snapshot).unwrap(), b"<rss/>");
    }

    #[test]
    fn the_database_file_contains_no_connector_secret() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let secret = "s3-secret-key-9b1e7d";
        let mut vault = OpenVault::create("correct horse battery").unwrap();
        vault.put("connector:1", secret.as_bytes()).unwrap();
        let mut collection = Collection::open(&path).unwrap();
        collection
            .add_connector("s3", "Bucket", r#"{"bucket":"music"}"#, Some("connector:1"))
            .unwrap();
        let feed_id = collection.add_feed(None, None).unwrap();
        let doc = Document::parse(
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><title>A</title>\
             <podcast:value type=\"lightning\" method=\"keysend\">\
             <podcast:valueRecipient address=\"addr1\" type=\"node\" split=\"100\"/>\
             </podcast:value></channel></rss>",
        )
        .unwrap();
        collection
            .record_state(feed_id, &Feed::read(&doc).unwrap(), &Version::new("v1"))
            .unwrap();
        collection
            .save_recipient(&Recipient {
                address: Some("addr2".to_owned()),
                ..Recipient::default()
            })
            .unwrap();
        drop(collection);
        let bytes = std::fs::read(&path).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("connector:1"));
        assert!(!text.contains(secret));
    }

    #[test]
    fn a_database_at_schema_version_2_opens_and_gets_the_recipient_tables() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = Connection::open(&path).unwrap();
        db.execute_batch(MIGRATIONS[0]).unwrap();
        db.execute_batch(MIGRATIONS[1]).unwrap();
        db.pragma_update(None, "user_version", 2).unwrap();
        db.execute(
            "INSERT INTO feeds (id, public_url, created_at) VALUES (1, 'https://example.com/a.xml', 0)",
            [],
        )
        .unwrap();
        drop(db);
        let collection = Collection::open(&path).unwrap();
        assert_eq!(collection.schema_version().unwrap(), SCHEMA_VERSION);
        assert_eq!(collection.feeds().unwrap().len(), 1);
        assert!(collection.known_recipients().unwrap().is_empty());
        collection
            .save_recipient(&Recipient {
                address: Some("addr".to_owned()),
                ..Recipient::default()
            })
            .unwrap();
        assert_eq!(collection.known_recipients().unwrap().len(), 1);
    }

    fn value_recipient_doc(address: &str, extra: &str) -> Document {
        Document::parse(&format!(
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><title>A</title>\
             <podcast:value type=\"lightning\" method=\"keysend\">\
             <podcast:valueRecipient address=\"{address}\" type=\"node\" split=\"100\" {extra}/>\
             </podcast:value>\
             <item><guid>one</guid>\
             <podcast:value type=\"lightning\" method=\"keysend\">\
             <podcast:valueRecipient address=\"it\" name=\"Item\" type=\"node\" split=\"100\" customKey=\"7629169\" customValue=\"acct1\"/>\
             </podcast:value>\
             </item></channel></rss>"
        ))
        .unwrap()
    }

    #[test]
    fn known_recipients_gives_the_channel_and_item_recipients_of_one_feed_one_time_each() {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        let doc = value_recipient_doc("ch", "name=\"Channel\"");
        collection
            .record_state(feed, &Feed::read(&doc).unwrap(), &Version::new("v1"))
            .unwrap();
        let mut known = collection.known_recipients().unwrap();
        known.sort_by(|a, b| a.address.cmp(&b.address));
        assert_eq!(known.len(), 2);
        for recipient in &known {
            assert_eq!(recipient.feeds, 1);
            assert_eq!(recipient.saved_id, None);
        }
        let channel = known.iter().find(|r| r.address == "ch").unwrap();
        assert_eq!(channel.name.as_deref(), Some("Channel"));
        assert_eq!(channel.custom_key, None);
        assert_eq!(channel.custom_value, None);
        let item = known.iter().find(|r| r.address == "it").unwrap();
        assert_eq!(item.custom_key.as_deref(), Some("7629169"));
        assert_eq!(item.custom_value.as_deref(), Some("acct1"));
    }

    #[test]
    fn two_feeds_with_the_same_identity_give_one_entry_with_the_name_of_the_newest_read() {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed_a = collection.add_feed(None, None).unwrap();
        let feed_b = collection.add_feed(None, None).unwrap();
        collection
            .record_state(
                feed_a,
                &Feed::read(&value_recipient_doc("shared", "name=\"Old\"")).unwrap(),
                &Version::new("v1"),
            )
            .unwrap();
        collection
            .db()
            .execute(
                "UPDATE feed_recipients SET seen_at = 1 WHERE feed_id = ?1 AND address = 'shared'",
                [feed_a.0],
            )
            .unwrap();
        collection
            .record_state(
                feed_b,
                &Feed::read(&value_recipient_doc("shared", "name=\"New\"")).unwrap(),
                &Version::new("v1"),
            )
            .unwrap();
        collection
            .db()
            .execute(
                "UPDATE feed_recipients SET seen_at = 2 WHERE feed_id = ?1 AND address = 'shared'",
                [feed_b.0],
            )
            .unwrap();
        let known = collection.known_recipients().unwrap();
        let shared = known.iter().find(|r| r.address == "shared").unwrap();
        assert_eq!(shared.feeds, 2);
        assert_eq!(shared.name.as_deref(), Some("New"));
    }

    #[test]
    fn a_recipient_goes_out_of_the_list_when_its_feed_loses_it_or_the_feed_is_deleted() {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        let with_recipient = Document::parse(
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><title>A</title>\
             <podcast:value type=\"lightning\" method=\"keysend\">\
             <podcast:valueRecipient address=\"only\" type=\"node\" split=\"100\"/>\
             </podcast:value></channel></rss>",
        )
        .unwrap();
        let without_recipient = Document::parse(
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><title>A</title></channel></rss>",
        )
        .unwrap();

        collection
            .record_state(
                feed,
                &Feed::read(&with_recipient).unwrap(),
                &Version::new("v1"),
            )
            .unwrap();
        assert_eq!(collection.known_recipients().unwrap().len(), 1);

        collection
            .record_state(
                feed,
                &Feed::read(&without_recipient).unwrap(),
                &Version::new("v2"),
            )
            .unwrap();
        assert!(collection.known_recipients().unwrap().is_empty());

        collection
            .record_state(
                feed,
                &Feed::read(&with_recipient).unwrap(),
                &Version::new("v3"),
            )
            .unwrap();
        assert_eq!(collection.known_recipients().unwrap().len(), 1);
        collection.delete_feed(feed).unwrap();
        assert!(collection.known_recipients().unwrap().is_empty());
    }

    #[test]
    fn a_saved_recipient_stays_with_no_feed_and_goes_out_when_removed() {
        let collection = Collection::open_in_memory().unwrap();
        let id = collection
            .save_recipient(&Recipient {
                address: Some("saved".to_owned()),
                name: Some("Ann".to_owned()),
                ..Recipient::default()
            })
            .unwrap();
        let known = collection.known_recipients().unwrap();
        assert_eq!(known.len(), 1);
        assert_eq!(known[0].feeds, 0);
        assert_eq!(known[0].saved_id, Some(id));
        collection.remove_saved_recipient(id).unwrap();
        assert!(collection.known_recipients().unwrap().is_empty());
        assert_eq!(
            collection.remove_saved_recipient(id).unwrap_err().kind(),
            CollectionErrorKind::NotFound
        );
    }

    #[test]
    fn a_database_at_schema_version_3_opens_and_gets_the_pending_files_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = Connection::open(&path).unwrap();
        db.execute_batch(MIGRATIONS[0]).unwrap();
        db.execute_batch(MIGRATIONS[1]).unwrap();
        db.execute_batch(MIGRATIONS[2]).unwrap();
        db.pragma_update(None, "user_version", 3).unwrap();
        db.execute(
            "INSERT INTO feeds (id, public_url, created_at) VALUES (1, 'https://example.com/a.xml', 0)",
            [],
        )
        .unwrap();
        drop(db);
        let collection = Collection::open(&path).unwrap();
        assert_eq!(collection.schema_version().unwrap(), SCHEMA_VERSION);
        let file = DraftFile {
            path: "album/track.mp3".to_owned(),
            size: 10,
            media_type: "audio/mpeg".to_owned(),
            sha256: "abc".to_owned(),
        };
        collection.add_draft_file(FeedId(1), &file).unwrap();
        assert_eq!(collection.draft_files(FeedId(1)).unwrap(), [file]);
    }

    #[test]
    fn a_database_at_schema_version_4_opens_and_gets_the_library_records() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = Connection::open(&path).unwrap();
        for migration in &MIGRATIONS[..4] {
            db.execute_batch(migration).unwrap();
        }
        db.pragma_update(None, "user_version", 4).unwrap();
        db.execute(
            "INSERT INTO feeds (id, public_url, created_at) VALUES (1, 'https://example.com/a.xml', 0)",
            [],
        )
        .unwrap();
        drop(db);

        let collection = Collection::open(&path).unwrap();
        assert_eq!(collection.schema_version().unwrap(), SCHEMA_VERSION);

        // Settings.
        assert!(collection.setting("library_root").unwrap().is_none());
        collection
            .set_setting("library_root", &serde_json::json!("/srv/library"))
            .unwrap();
        assert_eq!(
            collection.setting("library_root").unwrap(),
            Some(serde_json::json!("/srv/library"))
        );

        // The feed folder and the default host of its files.
        let feed = FeedId(1);
        assert_eq!(collection.feed(feed).unwrap().folder, None);
        collection.set_feed_folder(feed, "my-album").unwrap();
        let connector = collection
            .add_connector("local", "Home", "{}", None)
            .unwrap();
        collection
            .set_files_connector(feed, Some(connector))
            .unwrap();
        let row = collection.feed(feed).unwrap();
        assert_eq!(row.folder.as_deref(), Some("my-album"));
        assert_eq!(row.files_connector_id, Some(connector));

        // Library files.
        let file = collection
            .add_library_file(feed, "01 Song.mp3", 10, "abc", "audio/mpeg", true, None)
            .unwrap();
        assert_eq!(
            collection.library_files(feed).unwrap(),
            [collection.library_file(file).unwrap()]
        );
        assert_eq!(collection.library_file(file).unwrap().path, "01 Song.mp3");
        collection
            .set_library_file_connector(file, Some(connector))
            .unwrap();
        assert_eq!(
            collection.library_file(file).unwrap().connector_id,
            Some(connector)
        );

        // The host inventory.
        collection
            .put_host_file(connector, "a.mp3", 10, &Stamp::new("10-1"))
            .unwrap();
        assert_eq!(collection.host_files(connector).unwrap().len(), 1);
        collection
            .set_host_file_sha(connector, "a.mp3", "abc")
            .unwrap();
        assert_eq!(
            collection.host_files(connector).unwrap()[0]
                .sha256
                .as_deref(),
            Some("abc")
        );
        // A scan with the same version keeps the SHA-256. A new version
        // clears it (ADR 0036 item 6).
        collection
            .put_host_file(connector, "a.mp3", 10, &Stamp::new("10-1"))
            .unwrap();
        assert_eq!(
            collection.host_files(connector).unwrap()[0]
                .sha256
                .as_deref(),
            Some("abc")
        );
        collection
            .put_host_file(connector, "a.mp3", 11, &Stamp::new("11-2"))
            .unwrap();
        assert_eq!(collection.host_files(connector).unwrap()[0].sha256, None);
        collection.remove_host_file(connector, "a.mp3").unwrap();
        assert!(collection.host_files(connector).unwrap().is_empty());

        // Draft operations.
        let operation = collection
            .add_draft_operation(
                feed,
                DraftOperationKind::Copy,
                Some(file),
                connector,
                "a.mp3",
            )
            .unwrap();
        let operations = collection.draft_operations(feed).unwrap();
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].id, operation);
        assert_eq!(operations[0].kind, DraftOperationKind::Copy);
        collection.remove_draft_operations(feed).unwrap();
        assert!(collection.draft_operations(feed).unwrap().is_empty());

        collection.remove_library_file(file).unwrap();
        assert_eq!(
            collection.remove_library_file(file).unwrap_err().kind(),
            CollectionErrorKind::NotFound
        );
    }

    #[test]
    fn a_pending_file_at_a_path_that_the_draft_has_gives_in_use() {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        let file = DraftFile {
            path: "a.mp3".to_owned(),
            size: 1,
            media_type: "audio/mpeg".to_owned(),
            sha256: "x".to_owned(),
        };
        collection.add_draft_file(feed, &file).unwrap();
        let error = collection.add_draft_file(feed, &file).unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::InUse);
    }

    #[test]
    fn remove_draft_file_clears_its_row_and_keeps_the_rest() {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        let a = DraftFile {
            path: "a.mp3".to_owned(),
            size: 1,
            media_type: "audio/mpeg".to_owned(),
            sha256: "a".to_owned(),
        };
        let b = DraftFile {
            path: "b.mp3".to_owned(),
            size: 2,
            media_type: "audio/mpeg".to_owned(),
            sha256: "b".to_owned(),
        };
        collection.add_draft_file(feed, &a).unwrap();
        collection.add_draft_file(feed, &b).unwrap();
        collection.remove_draft_file(feed, "a.mp3").unwrap();
        assert_eq!(collection.draft_files(feed).unwrap(), [b]);
    }

    #[test]
    fn drop_draft_files_table_removes_the_table_and_a_second_call_does_nothing() {
        let collection = Collection::open_in_memory().unwrap();
        collection.drop_draft_files_table().unwrap();
        assert!(collection.draft_files(FeedId(1)).is_err());
        // A second call does nothing: it does not give an error.
        collection.drop_draft_files_table().unwrap();
    }

    #[test]
    fn a_deleted_feed_removes_the_rows_of_its_pending_files() {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        let file = DraftFile {
            path: "a.mp3".to_owned(),
            size: 1,
            media_type: "audio/mpeg".to_owned(),
            sha256: "a".to_owned(),
        };
        collection.add_draft_file(feed, &file).unwrap();
        collection.delete_feed(feed).unwrap();
        assert!(collection.draft_files(feed).unwrap().is_empty());
    }

    #[test]
    fn save_recipient_needs_an_address_and_updates_a_saved_identity() {
        let collection = Collection::open_in_memory().unwrap();
        let error = collection
            .save_recipient(&Recipient::default())
            .unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Invalid);
        let spaces = Recipient {
            address: Some("   ".to_owned()),
            ..Recipient::default()
        };
        let error = collection.save_recipient(&spaces).unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Invalid);

        let recipient = Recipient {
            address: Some("x".to_owned()),
            name: Some("Old".to_owned()),
            ..Recipient::default()
        };
        let id = collection.save_recipient(&recipient).unwrap();
        let updated = Recipient {
            name: Some("New".to_owned()),
            ..recipient
        };
        assert_eq!(collection.save_recipient(&updated).unwrap(), id);
        let known = collection.known_recipients().unwrap();
        assert_eq!(known.len(), 1);
        assert_eq!(known[0].name.as_deref(), Some("New"));
    }
}

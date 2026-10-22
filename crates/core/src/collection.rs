//! The collection of a user in SQLite (ADR 0005, ADR 0014).
//!
//! The collection keeps the connectors, the feeds, the item GUIDs of each
//! feed, the drafts and the snapshots. It keeps no secret: a connector row
//! gives only the ID of its vault record.

use std::fmt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

use crate::address::{self, Address, AddressSource};
use crate::connector::Version;
use crate::feed::Feed;

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

    #[cfg(test)]
    pub(crate) fn db(&self) -> &Connection {
        &self.db
    }
}

const FEED_SELECT: &str = "SELECT id, public_url, connector_id, path, podcast_guid, title, medium, last_version, address_source FROM feeds";

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
        let collection = Collection::open(&path).unwrap();
        collection
            .add_connector("s3", "Bucket", r#"{"bucket":"music"}"#, Some("connector:1"))
            .unwrap();
        drop(collection);
        let bytes = std::fs::read(&path).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("connector:1"));
        assert!(!text.contains(secret));
    }
}

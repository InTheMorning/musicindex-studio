//! The collection of a user in SQLite (ADR 0005, ADR 0014).
//!
//! The collection keeps the connectors, the feeds, the item GUIDs of each
//! feed, the drafts and the snapshots. It keeps no secret: a connector row
//! gives only the ID of its vault record.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::address::{self, Address, AddressSource};
use crate::announce::{Announcement, AnnouncementState};
use crate::connector::{KeptBlob, Stamp, Version};
use crate::feed::{CreditPlace, Feed, Person, Recipient};
use crate::keysend::{Lookup, LookupStatus};
use crate::library::Library;
use crate::xml::Document;

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
    r"
CREATE TABLE library_files_new (
    id INTEGER PRIMARY KEY,
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    size INTEGER NOT NULL,
    sha256 TEXT,
    media_type TEXT NOT NULL,
    local INTEGER NOT NULL,
    source_url TEXT,
    connector_id INTEGER REFERENCES connectors(id),
    added_at INTEGER NOT NULL,
    stamp TEXT,
    UNIQUE (feed_id, path)
);
INSERT INTO library_files_new
    (id, feed_id, path, size, sha256, media_type, local, source_url, connector_id, added_at, stamp)
SELECT id, feed_id, path, size, sha256, media_type, local, source_url, connector_id, added_at, NULL
FROM library_files;
DROP TABLE library_files;
ALTER TABLE library_files_new RENAME TO library_files;
CREATE TABLE downloads (
    library_file_id INTEGER PRIMARY KEY REFERENCES library_files(id) ON DELETE CASCADE,
    state TEXT NOT NULL CHECK (state IN ('queued', 'failed')),
    message TEXT,
    added_at INTEGER NOT NULL
);
CREATE TABLE draft_feed_moves (
    feed_id INTEGER PRIMARY KEY REFERENCES feeds(id) ON DELETE CASCADE,
    connector_id INTEGER NOT NULL REFERENCES connectors(id),
    path TEXT NOT NULL,
    added_at INTEGER NOT NULL
);
CREATE TABLE feed_pointers (
    connector_id INTEGER NOT NULL REFERENCES connectors(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    added_at INTEGER NOT NULL,
    PRIMARY KEY (connector_id, path)
);
",
    r"
ALTER TABLE feeds ADD COLUMN publisher_role TEXT CHECK (publisher_role IN ('artist', 'label'));
",
    r"
CREATE TABLE draft_operations_new (
    id INTEGER PRIMARY KEY,
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('copy', 'delete', 'confirm')),
    library_file_id INTEGER REFERENCES library_files(id) ON DELETE CASCADE,
    connector_id INTEGER NOT NULL REFERENCES connectors(id),
    path TEXT NOT NULL,
    added_at INTEGER NOT NULL
);
INSERT INTO draft_operations_new
    (id, feed_id, kind, library_file_id, connector_id, path, added_at)
SELECT id, feed_id, kind, library_file_id, connector_id, path, added_at
FROM draft_operations;
DROP TABLE draft_operations;
ALTER TABLE draft_operations_new RENAME TO draft_operations;
",
    r"
CREATE TABLE kept_blobs (
    connector_id INTEGER NOT NULL REFERENCES connectors(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    size INTEGER NOT NULL,
    PRIMARY KEY (connector_id, path)
);
",
    r"
CREATE TABLE signing_plans (
    feed_id INTEGER PRIMARY KEY REFERENCES feeds(id) ON DELETE CASCADE,
    diff_id TEXT NOT NULL,
    plan_time INTEGER NOT NULL
);
",
    r"
CREATE TABLE announcements (
    id INTEGER PRIMARY KEY,
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    podping INTEGER NOT NULL,
    state TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    next_at INTEGER NOT NULL,
    finished_at INTEGER,
    last_error TEXT
);
",
    r"
CREATE TABLE connected_apps (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL
);
",
    r"
CREATE TABLE draft_grants (
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    item_guid TEXT NOT NULL,
    PRIMARY KEY (feed_id, item_guid)
);
CREATE TABLE show_grants (
    id INTEGER PRIMARY KEY,
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    item_guid TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    ended_at INTEGER,
    UNIQUE (feed_id, item_guid)
);
CREATE TABLE grant_writes (
    id INTEGER PRIMARY KEY,
    grant_id INTEGER NOT NULL REFERENCES show_grants(id) ON DELETE CASCADE,
    app_name TEXT NOT NULL,
    change TEXT NOT NULL,
    written_at INTEGER NOT NULL
);
",
    r"
CREATE TABLE uploads (
    id TEXT PRIMARY KEY,
    feed_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    size INTEGER NOT NULL,
    replace TEXT,
    received TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);
",
    r"
ALTER TABLE connected_apps ADD COLUMN app_pubkey TEXT;
",
    r"
ALTER TABLE feeds ADD COLUMN publisher_roles TEXT;
ALTER TABLE feeds ADD COLUMN roles_confirmed INTEGER NOT NULL DEFAULT 0;
UPDATE feeds SET publisher_roles = publisher_role, roles_confirmed = 1 WHERE publisher_role IS NOT NULL;
ALTER TABLE feeds DROP COLUMN publisher_role;
",
    r"
CREATE TABLE change_sets (
    id INTEGER PRIMARY KEY,
    operation TEXT NOT NULL,
    recipient TEXT NOT NULL,
    made_at INTEGER NOT NULL
);
CREATE TABLE change_set_feeds (
    change_set_id INTEGER NOT NULL REFERENCES change_sets(id) ON DELETE CASCADE,
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    outcome TEXT NOT NULL,
    before_xml BLOB,
    before_base_xml BLOB,
    before_base_version TEXT,
    had_draft INTEGER NOT NULL,
    written_at INTEGER,
    PRIMARY KEY (change_set_id, feed_id)
);
",
    r"
ALTER TABLE change_sets ADD COLUMN saved_before TEXT;
",
    r"
CREATE TABLE keysend_lookups (
    address TEXT PRIMARY KEY,
    pubkey TEXT,
    custom_key TEXT NOT NULL DEFAULT '',
    custom_value TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL,
    checked_at INTEGER NOT NULL
);
",
    r"
CREATE TABLE feed_credits (
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    name TEXT NOT NULL DEFAULT '',
    role TEXT NOT NULL DEFAULT '',
    grp TEXT NOT NULL DEFAULT '',
    img TEXT NOT NULL DEFAULT '',
    href TEXT NOT NULL DEFAULT '',
    npub TEXT NOT NULL DEFAULT '',
    seen_at INTEGER NOT NULL,
    PRIMARY KEY (feed_id, name, role, grp, href, npub)
);
",
    r"
CREATE TABLE people (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    usual_role TEXT NOT NULL DEFAULT '',
    img TEXT NOT NULL DEFAULT '',
    href TEXT NOT NULL DEFAULT '',
    default_address TEXT,
    default_custom_key TEXT NOT NULL DEFAULT '',
    default_custom_value TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL
);
CREATE TABLE person_wallets (
    id INTEGER PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    address TEXT NOT NULL,
    custom_key TEXT NOT NULL DEFAULT '',
    custom_value TEXT NOT NULL DEFAULT '',
    recipient_type TEXT NOT NULL DEFAULT '',
    position INTEGER NOT NULL,
    evidence TEXT NOT NULL,
    linked_at INTEGER NOT NULL,
    UNIQUE (address, custom_key, custom_value)
);
CREATE TABLE person_keys (
    id INTEGER PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    npub TEXT NOT NULL UNIQUE,
    evidence TEXT NOT NULL,
    linked_at INTEGER NOT NULL
);
CREATE TABLE person_credit_names (
    id INTEGER PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    identity TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL DEFAULT '',
    href TEXT NOT NULL DEFAULT '',
    npub TEXT NOT NULL DEFAULT '',
    evidence TEXT NOT NULL,
    linked_at INTEGER NOT NULL
);
CREATE TABLE person_support_links (
    id INTEGER PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    label TEXT NOT NULL DEFAULT '',
    url TEXT NOT NULL,
    position INTEGER NOT NULL
);
CREATE TABLE person_members (
    person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    PRIMARY KEY (person_id, feed_id)
);
CREATE TABLE person_dismissals (
    identity TEXT PRIMARY KEY,
    items TEXT NOT NULL,
    dismissed_at INTEGER NOT NULL
);
-- A saved Lightning address with a found lookup joins the person of the
-- saved node key of that lookup (ADR 0098 item 11).
CREATE TEMP TABLE merged_saved AS
SELECT s.id AS saved_id,
       t.id AS target_id,
       row_number() OVER (PARTITION BY t.id ORDER BY s.id) AS position
FROM saved_recipients s
JOIN keysend_lookups k
  ON k.address = lower(trim(s.address)) AND k.status = 'found'
JOIN saved_recipients t
  ON t.address = k.pubkey
 AND t.custom_key = k.custom_key
 AND t.custom_value = k.custom_value
WHERE t.id != s.id
  AND t.id NOT IN (
      SELECT s2.id FROM saved_recipients s2
      JOIN keysend_lookups k2
        ON k2.address = lower(trim(s2.address)) AND k2.status = 'found'
      JOIN saved_recipients t2
        ON t2.address = k2.pubkey
       AND t2.custom_key = k2.custom_key
       AND t2.custom_value = k2.custom_value
      WHERE t2.id != s2.id);
INSERT INTO people (id, name, created_at)
SELECT t.id,
       COALESCE(
           NULLIF(trim(t.name), ''),
           (SELECT NULLIF(trim(s.name), '')
            FROM saved_recipients s JOIN merged_saved m ON m.saved_id = s.id
            WHERE m.target_id = t.id AND NULLIF(trim(s.name), '') IS NOT NULL
            ORDER BY s.id LIMIT 1),
           t.address),
       t.saved_at
FROM saved_recipients t
WHERE t.id NOT IN (SELECT saved_id FROM merged_saved);
INSERT INTO person_wallets
    (person_id, address, custom_key, custom_value, recipient_type, position, evidence, linked_at)
SELECT t.id, t.address, t.custom_key, t.custom_value,
       COALESCE(t.recipient_type, ''), 0, 'user', t.saved_at
FROM saved_recipients t
WHERE t.id NOT IN (SELECT saved_id FROM merged_saved);
INSERT INTO person_wallets
    (person_id, address, custom_key, custom_value, recipient_type, position, evidence, linked_at)
SELECT m.target_id, s.address, s.custom_key, s.custom_value,
       COALESCE(s.recipient_type, ''), m.position, 'user', s.saved_at
FROM saved_recipients s JOIN merged_saved m ON m.saved_id = s.id;
DROP TABLE merged_saved;
DELETE FROM saved_recipients;
",
    r"
ALTER TABLE change_sets ADD COLUMN person_id INTEGER REFERENCES people(id) ON DELETE SET NULL;
",
    // ADR 0105 item 8 and item 10: the time of the import and the time of
    // the removal of a pending file. A row from before gets the time of
    // this migration.
    r"
ALTER TABLE library_files ADD COLUMN imported_at INTEGER;
ALTER TABLE library_files ADD COLUMN removed_at INTEGER;
UPDATE library_files SET imported_at = CAST(strftime('%s', 'now') AS INTEGER);
",
    // ADR 0107 item 1: `library_root` becomes a path relative to the data
    // folder. SQL cannot do it. `run_migrations` does it after this entry.
    "SELECT 1;",
    // ADR 0110 items 37 and 38: a connected app has a key pair. Each app from
    // before gets the state `token` and no key. Its grants end. Studio removes
    // the wrappers from the grant vault files after this entry, in code,
    // because SQL cannot read a file. `connected_apps` is made again, because
    // SQLite cannot drop the column `token_hash` that has a UNIQUE rule.
    r"
CREATE TABLE connected_apps_new (
    -- AUTOINCREMENT: the ID of a removed app is never used again, because a
    -- grant vault file names a wrapper by app ID.
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    pubkey TEXT,
    state TEXT NOT NULL,
    code_hash TEXT,
    code_expires_at INTEGER,
    live_value_origin TEXT,
    last_used_at INTEGER,
    last_address TEXT,
    latest_event_at INTEGER
);
INSERT INTO connected_apps_new (id, name, created_at, state)
SELECT id, name, created_at, 'token' FROM connected_apps;
DROP TABLE connected_apps;
ALTER TABLE connected_apps_new RENAME TO connected_apps;
CREATE UNIQUE INDEX connected_apps_pubkey ON connected_apps (pubkey) WHERE pubkey IS NOT NULL;
CREATE UNIQUE INDEX connected_apps_code ON connected_apps (code_hash) WHERE code_hash IS NOT NULL;
CREATE TABLE app_events (
    event_id TEXT PRIMARY KEY,
    expires_at INTEGER NOT NULL
);
ALTER TABLE show_grants ADD COLUMN app_id INTEGER;
ALTER TABLE show_grants ADD COLUMN live_value_origin TEXT;
UPDATE show_grants SET ended_at = CAST(strftime('%s', 'now') AS INTEGER) WHERE ended_at IS NULL;
DELETE FROM draft_grants;
ALTER TABLE draft_grants ADD COLUMN app_id INTEGER;
ALTER TABLE grant_writes ADD COLUMN cause TEXT;
",
    // ADR 0112 item 7: a person keeps a usual group for a usual role. A
    // person from before gets an empty group.
    r"
ALTER TABLE people ADD COLUMN usual_group TEXT NOT NULL DEFAULT '';
",
    // ADR 0098 item 18: the user can dismiss the Lightning address of a
    // profile for one person. The address then does not come back for that
    // person.
    r"
CREATE TABLE person_address_dismissals (
    person_id INTEGER NOT NULL REFERENCES people(id) ON DELETE CASCADE,
    address TEXT NOT NULL,
    dismissed_at INTEGER NOT NULL,
    PRIMARY KEY (person_id, address)
);
",
    // ADR 0120 item 1: the credit index keeps the places of each credit. A
    // credit from before has no places until the refill after this
    // migration (ADR 0120 item 4).
    r"
ALTER TABLE feed_credits ADD COLUMN places TEXT NOT NULL DEFAULT '[]';
",
    // ADR 0126 item 19: the feed XML that stage 2 refused stays on the new
    // host as a placed file of the feed. The next write replaces it.
    r"
CREATE TABLE placed_feed_files (
    connector_id INTEGER NOT NULL REFERENCES connectors(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    added_at INTEGER NOT NULL,
    PRIMARY KEY (connector_id, path)
);
",
];

/// `root` as a relative path under `data_dir`, when it is a folder inside
/// it. The comparison also tries the resolved paths.
fn relative_to_data(data_dir: Option<&Path>, root: &Path) -> Option<String> {
    let data_dir = data_dir?;
    let plain = |relative: &Path| {
        let text = relative.to_str()?;
        (!text.is_empty()
            && relative
                .components()
                .all(|c| matches!(c, std::path::Component::Normal(_))))
        .then(|| text.to_owned())
    };
    if let Ok(relative) = root.strip_prefix(data_dir)
        && let Some(text) = plain(relative)
    {
        return Some(text);
    }
    let data = data_dir.canonicalize().ok()?;
    let root = root.canonicalize().ok()?;
    plain(root.strip_prefix(&data).ok()?)
}

/// The setting that holds the time that the mode `off` of ADR 0105 began.
const ASSETS_OFF_SINCE: &str = "assets_off_since";

/// The index in `MIGRATIONS` of the migration that makes the people. The
/// credit index is refilled after the migrations of a run (ADR 0120 item 4),
/// because SQL cannot read a feed.
#[cfg(test)]
const PEOPLE_MIGRATION: usize = 20;

/// The index in `MIGRATIONS` of the migration that makes `library_root`
/// relative (ADR 0107 item 1). SQL cannot read the data folder, so the
/// migration has no statement. Studio does the change after it, in code.
const RELATIVE_LIBRARY_MIGRATION: usize = 23;

/// The index in `MIGRATIONS` of the migration that gives a connected app a
/// key pair (ADR 0110 items 37 and 38). SQL cannot change a grant vault
/// file, so Studio removes the wrappers after it, in code.
const APP_KEYS_MIGRATION: usize = 24;

/// The index in `MIGRATIONS` of the migration that adds the places of each
/// credit (ADR 0120 item 1). A run that applies it refills the credit index
/// from the library copies (ADR 0120 item 4).
const CREDIT_PLACES_MIGRATION: usize = 27;

/// An upload in parts that is not finished (ADR 0059 item 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadRow {
    /// The random ID, as hexadecimal text.
    pub id: String,
    /// The feed that receives the file.
    pub feed_id: FeedId,
    /// The file name that the user selected.
    pub name: String,
    /// The size of the full file in bytes.
    pub size: u64,
    /// `Some` when the file replaces the bytes of a track (ADR 0040 item 3).
    pub replace: Option<String>,
    /// The numbers of the parts that Studio has, from small to large.
    pub received: Vec<u32>,
    /// The Unix time of the start, or of the last new part.
    pub updated_at: u64,
}

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
    /// The value is a link of a different person already.
    Conflict,
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

    pub(crate) fn with_kind(kind: CollectionErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// An error for a value that the collection refuses.
    pub(crate) fn invalid(message: impl Into<String>) -> Self {
        Self::with_kind(CollectionErrorKind::Invalid, message)
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ConnectorId(pub i64);

/// The ID of a feed in the collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// A role of a publisher feed (ADR 0082 item 1). The variants are in the
/// sequence of musicindex ADR 0036 R-ROLE-1, so a sort gives that sequence
/// (ADR 0083 item 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublisherRole {
    /// The publisher feed of an artist.
    Artist,
    /// The publisher feed of a label.
    Label,
    /// A hosting service.
    Hosting,
    /// A producer.
    Producer,
    /// A host.
    Host,
    /// An author.
    Author,
    /// A network.
    Network,
    /// A sponsor.
    Sponsor,
}

impl PublisherRole {
    /// The eight roles, in their sequence.
    pub const ALL: [Self; 8] = [
        Self::Artist,
        Self::Label,
        Self::Hosting,
        Self::Producer,
        Self::Host,
        Self::Author,
        Self::Network,
        Self::Sponsor,
    ];

    /// The text of the role in the collection and in the `rel` of a link.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Artist => "artist",
            Self::Label => "label",
            Self::Host => "host",
            Self::Author => "author",
            Self::Producer => "producer",
            Self::Network => "network",
            Self::Hosting => "hosting",
            Self::Sponsor => "sponsor",
        }
    }

    /// The role for its text. Gives none for another text.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|role| role.as_str() == text)
    }
}

/// The roles of a `rel` value, in their sequence, each one time (ADR 0082
/// item 3, musicindex ADR 0011 item 1). A value with a comma is a comma
/// list, and each other value is a list separated by white space. A word
/// that is not a role is not in the result.
#[must_use]
pub fn roles_from_rel(text: &str) -> Vec<PublisherRole> {
    let words: Vec<&str> = if text.contains(',') {
        text.split(',').collect()
    } else {
        text.split_whitespace().collect()
    };
    let mut roles: Vec<PublisherRole> = words
        .iter()
        .filter_map(|word| PublisherRole::parse(&word.trim().to_ascii_lowercase()))
        .collect();
    roles.sort_unstable();
    roles.dedup();
    roles
}

/// The roles that the channel `remoteItem` elements with `medium="music"`
/// or `medium="musicL"` of a publisher feed give: each role of each `rel`,
/// in their sequence (ADR 0082 item 4, ADR 0086 item 3).
#[must_use]
pub fn inferred_roles(feed: &Feed) -> Vec<PublisherRole> {
    let mut roles: Vec<PublisherRole> = feed
        .remote_items
        .iter()
        .filter(|item| matches!(item.medium.as_deref(), Some("music" | "musicL")))
        .filter_map(|item| item.rel.as_deref())
        .flat_map(roles_from_rel)
        .collect();
    roles.sort_unstable();
    roles.dedup();
    roles
}

/// The `rel` value of a set of roles: the roles in their sequence, with
/// one space between two roles (ADR 0082 item 8).
#[must_use]
pub fn rel_of(roles: &[PublisherRole]) -> String {
    let mut sorted = roles.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    sorted
        .iter()
        .map(|role| role.as_str())
        .collect::<Vec<_>>()
        .join(" ")
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
    /// The roles of a publisher feed, in their sequence (ADR 0082 item 1).
    /// Empty for a feed that has no recorded role.
    pub publisher_roles: Vec<PublisherRole>,
    /// True when the user or a link confirmed the roles. False when Studio
    /// inferred them from the feed (ADR 0082 items 4 and 5).
    pub roles_confirmed: bool,
    /// The time that the feed joined the collection, in seconds since the
    /// Unix epoch. A feed that joins later has a greater or equal value.
    pub added_at: i64,
}

/// The move of the feed XML to a different host, in the draft of a feed
/// (ADR 0038 item 16).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftFeedMove {
    /// The new host of the feed XML.
    pub connector_id: ConnectorId,
    /// The path of the feed XML on the new host.
    pub path: String,
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
    /// The SHA-256 of the file. `None` for a remote file, whose bytes
    /// are not in the library (ADR 0038 item 1).
    pub sha256: Option<String>,
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
    /// The size and the modification time of the file in the library,
    /// as `{size}-{mtime_ns}`. `None` for a remote file, and for a row
    /// from before schema version 6 (ADR 0038 item 13).
    pub stamp: Option<String>,
    /// The time of the import, an upload `finish` or a replacement, in
    /// seconds after the Unix epoch. A row from before the migration has
    /// the time of the migration (ADR 0105 item 8).
    pub imported_at: i64,
    /// The time that Studio removed the bytes of a pending file at the end
    /// of its 7 days (ADR 0105 item 10). `None` for each other file.
    pub removed_at: Option<i64>,
}

/// A download that is not complete (ADR 0038 item 9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DownloadRow {
    /// The library file that Studio downloads.
    pub library_file_id: LibraryFileId,
    /// `queued`, or `failed` with a message.
    pub state: String,
    /// The reason of a failure.
    pub message: Option<String>,
    /// The time that the download came into the queue, in seconds after
    /// the Unix epoch.
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
    /// Make sure that the host gives a file that an earlier write put
    /// there (ADR 0043 item 9).
    Confirm,
}

impl DraftOperationKind {
    /// The text of the kind in the collection.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Copy => "copy",
            Self::Delete => "delete",
            Self::Confirm => "confirm",
        }
    }

    /// The kind for its text in the collection.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "copy" => Some(Self::Copy),
            "delete" => Some(Self::Delete),
            "confirm" => Some(Self::Confirm),
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
    /// The rules of a server that holds no assets. `None` means that the
    /// workspace holds assets (ADR 0105 item 1).
    asset_rules: Option<AssetRules>,
    /// The folder of the collection file. A relative `library_root` is
    /// relative to it (ADR 0107 item 1). `None` for a collection in memory.
    data_dir: Option<PathBuf>,
}

/// The counts of an account (ADR 0107 items 4 and 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AccountCounts {
    /// The feeds in the collection.
    pub feeds: u64,
    /// The feeds with a draft.
    pub drafts: u64,
    /// The versions that Studio kept (the snapshots).
    pub versions: u64,
    /// The people.
    pub people: u64,
}

/// The rules of a workspace that holds no assets (ADR 0105). A workspace
/// that holds assets has none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetRules {
    /// The folder of the temporary files of a copy and a move (ADR 0105
    /// item 6).
    pub incoming: std::path::PathBuf,
}

impl fmt::Debug for Collection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Collection").finish_non_exhaustive()
    }
}

/// The columns and the table of an announcement query.
const ANNOUNCEMENT_COLUMNS: &str = "SELECT id, feed_id, url, podping, state, attempts, created_at,
     next_at, finished_at, last_error FROM announcements";

/// The result of one attempt to send an announcement.
#[derive(Debug, Clone, Copy)]
pub struct AnnouncementAttempt<'a> {
    /// The new state.
    pub state: AnnouncementState,
    /// The number of attempts, with this one.
    pub attempts: u32,
    /// The time of the next attempt.
    pub next_at: i64,
    /// The time of the end, for `sent` and `failed`.
    pub finished_at: Option<i64>,
    /// The error text, for a failed attempt.
    pub error: Option<&'a str>,
}

fn announcement_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Announcement> {
    let state: String = r.get(4)?;
    Ok(Announcement {
        id: r.get(0)?,
        feed_id: FeedId(r.get(1)?),
        url: r.get(2)?,
        podping: r.get(3)?,
        state: AnnouncementState::from_name(&state).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                format!("announcement state {state:?}").into(),
            )
        })?,
        attempts: r.get(5)?,
        created_at: r.get(6)?,
        next_at: r.get(7)?,
        finished_at: r.get(8)?,
        last_error: r.get(9)?,
    })
}

pub(crate) fn now() -> i64 {
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
        let data_dir = path
            .parent()
            .map(Path::to_path_buf)
            .filter(|dir| !dir.as_os_str().is_empty())
            .or_else(|| Some(PathBuf::from(".")));
        Self::prepare(Connection::open(path)?, data_dir)
    }

    /// Open a collection in memory, for tests and previews.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn open_in_memory() -> Result<Self, CollectionError> {
        Self::prepare(Connection::open_in_memory()?, None)
    }

    fn prepare(db: Connection, data_dir: Option<PathBuf>) -> Result<Self, CollectionError> {
        db.pragma_update(None, "foreign_keys", true)?;
        let mut collection = Self {
            db,
            asset_rules: None,
            data_dir,
        };
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
        // A migration can drop a table that other tables refer to. With
        // foreign keys on, the drop would delete the rows that cascade.
        // The pragma has no effect inside a transaction, so set it here.
        self.db.pragma_update(None, "foreign_keys", false)?;
        let result = self.run_migrations(current);
        self.db.pragma_update(None, "foreign_keys", true)?;
        result
    }

    fn run_migrations(&mut self, current: usize) -> Result<(), CollectionError> {
        for (index, migration) in MIGRATIONS.iter().enumerate().skip(current) {
            let tx = self.db.transaction()?;
            tx.execute_batch(migration)?;
            let version = i64::try_from(index + 1)
                .map_err(|_| CollectionError::new("too many migrations"))?;
            tx.pragma_update(None, "user_version", version)?;
            tx.commit()?;
            if index == RELATIVE_LIBRARY_MIGRATION {
                self.relativize_library_root()?;
            }
            if index == APP_KEYS_MIGRATION {
                self.remove_all_grant_wrappers();
            }
        }
        // The refill writes the places of each credit. Those places exist
        // from CREDIT_PLACES_MIGRATION on, so the refill runs after the last
        // migration of this run, and only when this run applied that
        // migration (ADR 0120 item 4).
        if current <= CREDIT_PLACES_MIGRATION {
            self.refill_credits()?;
        }
        Ok(())
    }

    /// The library folder, from the setting `library_root`. A relative
    /// setting is relative to the data folder (ADR 0107 item 1).
    ///
    /// A relative setting with a segment other than a plain name is
    /// refused, because an imported collection is not trusted.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails, when the setting is
    /// not text, or when a relative setting is not safe.
    pub fn library_root(&self) -> Result<Option<PathBuf>, CollectionError> {
        let Some(value) = self.setting("library_root")? else {
            return Ok(None);
        };
        let text = value
            .as_str()
            .ok_or_else(|| CollectionError::new("the library_root setting is not a string"))?;
        let path = Path::new(text);
        if path.is_absolute() {
            return Ok(Some(path.to_path_buf()));
        }
        let plain = !text.is_empty()
            && path
                .components()
                .all(|c| matches!(c, std::path::Component::Normal(_)));
        if !plain {
            return Err(CollectionError::new(
                "ADR 0107 item 1: a relative library_root must hold plain folder names only",
            ));
        }
        Ok(Some(match &self.data_dir {
            Some(dir) => dir.join(path),
            None => path.to_path_buf(),
        }))
    }

    /// The text that Studio stores for `root` as `library_root`: a path
    /// relative to the data folder when `root` is in it, else `root` as it
    /// is (ADR 0107 item 1).
    #[must_use]
    pub fn library_root_setting(&self, root: &Path) -> String {
        relative_to_data(self.data_dir.as_deref(), root)
            .unwrap_or_else(|| root.to_string_lossy().into_owned())
    }

    /// Remove each app wrapper from each grant vault file in the data
    /// folder (ADR 0110 items 37 and 38). After the migration of the apps,
    /// no app has a key that Studio accepts, so no wrapper is of use. An
    /// error for one file does not stop the others: the grants have ended,
    /// and the next sync of the grant vaults removes the file.
    fn remove_all_grant_wrappers(&self) {
        let Some(dir) = self.data_dir.as_ref().map(|dir| dir.join("grants")) else {
            return;
        };
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            let Ok(mut file) = crate::vault::VaultFile::load(&path) else {
                continue;
            };
            let ids = file.app_ids();
            if ids.is_empty() {
                continue;
            }
            for id in ids {
                file.remove_app_wrapper(id);
            }
            if let Err(error) = file.save(&path) {
                eprintln!("studio: a grant vault file did not change at the migration: {error}");
            }
        }
    }

    /// Change an absolute `library_root` in the data folder to a relative
    /// path. A path outside the data folder stays as it is.
    fn relativize_library_root(&self) -> Result<(), CollectionError> {
        let Some(Value::String(text)) = self.setting("library_root")? else {
            return Ok(());
        };
        if !Path::new(&text).is_absolute() {
            return Ok(());
        }
        if let Some(relative) = relative_to_data(self.data_dir.as_deref(), Path::new(&text)) {
            self.set_setting("library_root", &Value::String(relative))?;
        }
        Ok(())
    }

    /// Replace the `feed_credits` rows of each feed that has a library copy
    /// (ADR 0098 item 10, ADR 0120 item 4). A feed with no library copy, or
    /// with a copy that does not parse, keeps its rows as they are.
    fn refill_credits(&mut self) -> Result<(), CollectionError> {
        let Some(root) = self.library_root()?.filter(|root| root.is_dir()) else {
            return Ok(());
        };
        let Ok(library) = Library::open(root) else {
            return Ok(());
        };
        let rows: Vec<FeedRow> = self
            .feeds()?
            .into_iter()
            .filter(|row| row.folder.is_some())
            .collect();
        let tx = self.db.transaction()?;
        for row in rows {
            if let Some(feed) = library_copy(&library, &row) {
                put_credits(&tx, row.id, feed.credit_places())?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// Each stored keysend lookup (ADR 0095 item 2).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn keysend_lookups(&self) -> Result<Vec<Lookup>, CollectionError> {
        let mut statement = self.db.prepare(
            "SELECT address, pubkey, custom_key, custom_value, status, checked_at
             FROM keysend_lookups ORDER BY address",
        )?;
        let rows = statement.query_map([], |r| {
            let custom_key: String = r.get(2)?;
            let custom_value: String = r.get(3)?;
            let status: String = r.get(4)?;
            Ok(Lookup {
                address: r.get(0)?,
                pubkey: r.get(1)?,
                custom_key: Some(custom_key).filter(|t| !t.is_empty()),
                custom_value: Some(custom_value).filter(|t| !t.is_empty()),
                status: LookupStatus::parse(&status).unwrap_or(LookupStatus::Failed),
                checked_at: r.get(5)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Insert or replace the keysend lookup of an address.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn set_keysend_lookup(&self, lookup: &Lookup) -> Result<(), CollectionError> {
        self.db.execute(
            "INSERT OR REPLACE INTO keysend_lookups
             (address, pubkey, custom_key, custom_value, status, checked_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                lookup.address,
                lookup.pubkey,
                lookup.custom_key.as_deref().unwrap_or(""),
                lookup.custom_value.as_deref().unwrap_or(""),
                lookup.status.as_str(),
                lookup.checked_at
            ],
        )?;
        Ok(())
    }

    /// Change only the check time of a stored keysend lookup.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn touch_keysend_lookup(
        &self,
        address: &str,
        checked_at: i64,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "UPDATE keysend_lookups SET checked_at = ?2 WHERE address = ?1",
            params![address, checked_at],
        )?;
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

    /// Record the roles of a publisher feed, and whether they are confirmed
    /// (ADR 0082 items 2 and 5). An empty set removes the roles.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn set_publisher_roles(
        &self,
        id: FeedId,
        roles: &[PublisherRole],
        confirmed: bool,
    ) -> Result<(), CollectionError> {
        let text = (!roles.is_empty()).then(|| rel_of(roles));
        self.db.execute(
            "UPDATE feeds SET publisher_roles = ?2, roles_confirmed = ?3 WHERE id = ?1",
            params![id.0, text, confirmed && !roles.is_empty()],
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

    /// The counts that the export and the import show (ADR 0107 items 4
    /// and 5).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn account_counts(&self) -> Result<AccountCounts, CollectionError> {
        let count = |table: &str| -> Result<u64, CollectionError> {
            let n: i64 = self
                .db
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))?;
            Ok(u64::try_from(n).unwrap_or(0))
        };
        Ok(AccountCounts {
            feeds: count("feeds")?,
            drafts: count("drafts")?,
            versions: count("snapshots")?,
            people: count("people")?,
        })
    }

    /// Replace the whole content of this collection with the collection
    /// file at `source` (ADR 0107 item 9). The source is opened first, so
    /// that an older schema migrates. The settings of this collection that
    /// belong to the process, such as the asset rules, stay.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when the source cannot be opened or
    /// SQLite fails. After an error, this collection can hold part of the
    /// source.
    pub fn replace_from(&mut self, source: &Path) -> Result<(), CollectionError> {
        drop(Self::open(source)?);
        self.db.restore(
            rusqlite::MAIN_DB,
            source,
            None::<fn(rusqlite::backup::Progress)>,
        )?;
        self.db.pragma_update(None, "foreign_keys", true)?;
        Ok(())
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
        put_credits(&tx, id, feed.credit_places())?;
        // A publisher feed with no confirmed roles gets the roles that its
        // links give (ADR 0082 item 4).
        if feed.medium.as_deref() == Some("publisher") {
            let roles = rel_of(&inferred_roles(feed));
            tx.execute(
                "UPDATE feeds SET publisher_roles = NULLIF(?2, '') WHERE id = ?1 AND roles_confirmed = 0",
                params![id.0, roles],
            )?;
        }
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

    /// The GUID and the title of each item of a feed, in feed sequence.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn item_titles(
        &self,
        id: FeedId,
    ) -> Result<Vec<(String, Option<String>)>, CollectionError> {
        let mut statement = self
            .db
            .prepare("SELECT guid, title FROM feed_items WHERE feed_id = ?1 ORDER BY position")?;
        let rows = statement.query_map([id.0], |r| Ok((r.get(0)?, r.get(1)?)))?;
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

    /// Keep the move of the feed XML in the draft of a feed. A move that
    /// is there already is replaced (ADR 0038 item 16).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn set_draft_feed_move(
        &self,
        feed: FeedId,
        connector: ConnectorId,
        path: &str,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "INSERT INTO draft_feed_moves (feed_id, connector_id, path, added_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (feed_id) DO UPDATE SET connector_id = ?2, path = ?3, added_at = ?4",
            params![feed.0, connector.0, path, now()],
        )?;
        Ok(())
    }

    /// The move of the feed XML in the draft of a feed, if there is one.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn draft_feed_move(&self, feed: FeedId) -> Result<Option<DraftFeedMove>, CollectionError> {
        Ok(self
            .db
            .query_row(
                "SELECT connector_id, path FROM draft_feed_moves WHERE feed_id = ?1",
                [feed.0],
                |r| {
                    Ok(DraftFeedMove {
                        connector_id: ConnectorId(r.get(0)?),
                        path: r.get(1)?,
                    })
                },
            )
            .optional()?)
    }

    /// Remove the move of the feed XML from the draft of a feed.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn remove_draft_feed_move(&self, feed: FeedId) -> Result<(), CollectionError> {
        self.db
            .execute("DELETE FROM draft_feed_moves WHERE feed_id = ?1", [feed.0])?;
        Ok(())
    }

    /// The ID of the `waiting` announcement of a feed for an address.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn waiting_announcement(
        &self,
        feed: FeedId,
        url: &str,
    ) -> Result<Option<i64>, CollectionError> {
        Ok(self
            .db
            .query_row(
                "SELECT id FROM announcements
                 WHERE feed_id = ?1 AND url = ?2 AND state = 'waiting'",
                params![feed.0, url],
                |r| r.get(0),
            )
            .optional()?)
    }

    /// Give a `waiting` announcement a new podping value and new times. Set
    /// its attempts to 0, and remove the error of an earlier attempt.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn restart_announcement(
        &self,
        id: i64,
        podping: bool,
        created_at: i64,
        next_at: i64,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "UPDATE announcements
             SET podping = ?2, created_at = ?3, next_at = ?4, attempts = 0, last_error = NULL
             WHERE id = ?1",
            params![id, podping, created_at, next_at],
        )?;
        Ok(())
    }

    /// Add a `waiting` announcement.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn insert_announcement(
        &self,
        feed: FeedId,
        url: &str,
        podping: bool,
        created_at: i64,
        next_at: i64,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "INSERT INTO announcements (feed_id, url, podping, state, created_at, next_at)
             VALUES (?1, ?2, ?3, 'waiting', ?4, ?5)",
            params![feed.0, url, podping, created_at, next_at],
        )?;
        Ok(())
    }

    /// The oldest `waiting` announcement with a `next_at` of `now` or less.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn oldest_due_announcement(
        &self,
        now: i64,
    ) -> Result<Option<Announcement>, CollectionError> {
        Ok(self
            .db
            .query_row(
                &format!(
                    "{ANNOUNCEMENT_COLUMNS} WHERE state = 'waiting' AND next_at <= ?1
                     ORDER BY created_at, id LIMIT 1"
                ),
                [now],
                announcement_row,
            )
            .optional()?)
    }

    /// One announcement.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when the row does not exist or SQLite
    /// fails.
    pub fn announcement(&self, id: i64) -> Result<Announcement, CollectionError> {
        self.db
            .query_row(
                &format!("{ANNOUNCEMENT_COLUMNS} WHERE id = ?1"),
                [id],
                announcement_row,
            )
            .optional()?
            .ok_or_else(|| {
                CollectionError::with_kind(
                    CollectionErrorKind::NotFound,
                    format!("announcement {id}"),
                )
            })
    }

    /// Set the state of an announcement.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn set_announcement_state(
        &self,
        id: i64,
        state: AnnouncementState,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "UPDATE announcements SET state = ?2 WHERE id = ?1",
            params![id, state.as_str()],
        )?;
        Ok(())
    }

    /// The smallest `next_at` of the `waiting` announcements.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn earliest_waiting_announcement(&self) -> Result<Option<i64>, CollectionError> {
        Ok(self.db.query_row(
            "SELECT MIN(next_at) FROM announcements WHERE state = 'waiting'",
            [],
            |r| r.get(0),
        )?)
    }

    /// Record the result of an attempt: the new state, the number of
    /// attempts, the time of the next attempt, the time of the end and the
    /// error text.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn record_announcement_attempt(
        &self,
        id: i64,
        attempt: &AnnouncementAttempt<'_>,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "UPDATE announcements
             SET state = ?2, attempts = ?3, next_at = ?4, finished_at = ?5, last_error = ?6
             WHERE id = ?1",
            params![
                id,
                attempt.state.as_str(),
                attempt.attempts,
                attempt.next_at,
                attempt.finished_at,
                attempt.error
            ],
        )?;
        Ok(())
    }

    /// Set each `sending` announcement to `waiting`.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn release_sending_announcements(&self) -> Result<(), CollectionError> {
        self.db.execute(
            "UPDATE announcements SET state = 'waiting' WHERE state = 'sending'",
            [],
        )?;
        Ok(())
    }

    /// Set each `waiting` announcement with a `created_at` before `oldest`
    /// to `failed`.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn fail_old_announcements(
        &self,
        oldest: i64,
        now: i64,
        error: &str,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "UPDATE announcements
             SET state = 'failed', finished_at = ?2, last_error = ?3
             WHERE state = 'waiting' AND created_at < ?1",
            params![oldest, now, error],
        )?;
        Ok(())
    }

    /// Delete the `sent` and `failed` announcements of each feed after its
    /// newest `keep` announcements of these states.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn trim_announcements(&self, keep: i64) -> Result<(), CollectionError> {
        self.db.execute(
            "DELETE FROM announcements
             WHERE state IN ('sent', 'failed')
               AND id NOT IN (
                 SELECT id FROM (
                   SELECT id, ROW_NUMBER() OVER (
                     PARTITION BY feed_id ORDER BY created_at DESC, id DESC
                   ) AS place
                   FROM announcements
                   WHERE state IN ('sent', 'failed')
                 ) WHERE place <= ?1
               )",
            [keep],
        )?;
        Ok(())
    }

    /// The announcements of a feed, the newest first.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn announcements(&self, feed: FeedId) -> Result<Vec<Announcement>, CollectionError> {
        let mut statement = self.db.prepare(&format!(
            "{ANNOUNCEMENT_COLUMNS} WHERE feed_id = ?1 ORDER BY created_at DESC, id DESC"
        ))?;
        let rows = statement.query_map([feed.0], announcement_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Set `next_at` to `now` on each `waiting` announcement, so that a
    /// test does not wait.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    #[cfg(any(test, feature = "testing"))]
    pub fn make_announcements_due(&self, now: i64) -> Result<(), CollectionError> {
        self.db.execute(
            "UPDATE announcements SET next_at = ?1 WHERE state = 'waiting'",
            [now],
        )?;
        Ok(())
    }

    /// The signing plan of a feed: the diff ID and the plan time, in
    /// seconds after the Unix epoch (ADR 0050 item 10).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn signing_plan(&self, feed: FeedId) -> Result<Option<(String, u64)>, CollectionError> {
        Ok(self
            .db
            .query_row(
                "SELECT diff_id, plan_time FROM signing_plans WHERE feed_id = ?1",
                [feed.0],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        u64::try_from(r.get::<_, i64>(1)?).unwrap_or(0),
                    ))
                },
            )
            .optional()?)
    }

    /// Keep the signing plan of a feed. A new plan replaces the old plan.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn put_signing_plan(
        &self,
        feed: FeedId,
        diff_id: &str,
        plan_time: u64,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "INSERT INTO signing_plans (feed_id, diff_id, plan_time) VALUES (?1, ?2, ?3)
             ON CONFLICT(feed_id) DO UPDATE SET diff_id = ?2, plan_time = ?3",
            params![
                feed.0,
                diff_id,
                i64::try_from(plan_time).unwrap_or(i64::MAX)
            ],
        )?;
        Ok(())
    }

    /// Remove the signing plan of a feed.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn remove_signing_plan(&self, feed: FeedId) -> Result<(), CollectionError> {
        self.db
            .execute("DELETE FROM signing_plans WHERE feed_id = ?1", [feed.0])?;
        Ok(())
    }

    /// Record a pointer file: the old feed XML that tells the apps the new
    /// address of `feed`. Studio keeps each pointer file (ADR 0038
    /// item 20).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn add_feed_pointer(
        &self,
        connector: ConnectorId,
        path: &str,
        feed: FeedId,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "INSERT INTO feed_pointers (connector_id, path, feed_id, added_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (connector_id, path) DO UPDATE SET feed_id = ?3",
            params![connector.0, path, feed.0, now()],
        )?;
        Ok(())
    }

    /// True when the file at `path` on a host is a pointer file of Studio.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn is_feed_pointer(
        &self,
        connector: ConnectorId,
        path: &str,
    ) -> Result<bool, CollectionError> {
        Ok(self
            .db
            .query_row(
                "SELECT 1 FROM feed_pointers WHERE connector_id = ?1 AND path = ?2",
                params![connector.0, path],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }

    /// True when the file at `path` on a host is a pointer file of `feed`
    /// (ADR 0126 item 21). A pointer file of a different feed gives false.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn is_pointer_of_feed(
        &self,
        connector: ConnectorId,
        path: &str,
        feed: FeedId,
    ) -> Result<bool, CollectionError> {
        Ok(self
            .db
            .query_row(
                "SELECT 1 FROM feed_pointers WHERE connector_id = ?1 AND path = ?2 AND feed_id = ?3",
                params![connector.0, path, feed.0],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }

    /// Remove the record of a pointer file, because the feed XML replaced
    /// it (ADR 0126 item 22). The function does nothing when the file has
    /// no record.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn remove_feed_pointer(
        &self,
        connector: ConnectorId,
        path: &str,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "DELETE FROM feed_pointers WHERE connector_id = ?1 AND path = ?2",
            params![connector.0, path],
        )?;
        Ok(())
    }

    /// Record a placed file: the feed XML of `feed` that stage 2 of ADR 0126
    /// refused. It is not a pointer file (ADR 0126 item 19).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn add_placed_feed_file(
        &self,
        connector: ConnectorId,
        path: &str,
        feed: FeedId,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "INSERT INTO placed_feed_files (connector_id, path, feed_id, added_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (connector_id, path) DO UPDATE SET feed_id = ?3",
            params![connector.0, path, feed.0, now()],
        )?;
        Ok(())
    }

    /// Remove the record of a placed file. The function does nothing when
    /// the file has no record.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn remove_placed_feed_file(
        &self,
        connector: ConnectorId,
        path: &str,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "DELETE FROM placed_feed_files WHERE connector_id = ?1 AND path = ?2",
            params![connector.0, path],
        )?;
        Ok(())
    }

    /// True when the file at `path` on a host is a placed file of `feed`.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn is_placed_feed_file(
        &self,
        connector: ConnectorId,
        path: &str,
        feed: FeedId,
    ) -> Result<bool, CollectionError> {
        Ok(self
            .db
            .query_row(
                "SELECT 1 FROM placed_feed_files WHERE connector_id = ?1 AND path = ?2 AND feed_id = ?3",
                params![connector.0, path, feed.0],
                |_| Ok(()),
            )
            .optional()?
            .is_some())
    }

    /// Change the host, the path and the public address of a feed in one
    /// transaction (ADR 0038 item 19).
    ///
    /// The addresses of the sources "read" and "folder" are from the old
    /// location, so the function replaces them. The "folder" address
    /// becomes `public_url`. A feed with no default host of its files
    /// gets the host from before, so its files keep their host
    /// (ADR 0038 item 16).
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::NotFound`] when the feed or the
    /// connector does not exist.
    pub fn set_feed_location(
        &mut self,
        feed: FeedId,
        connector: ConnectorId,
        path: &str,
        public_url: Option<&str>,
    ) -> Result<(), CollectionError> {
        self.feed(feed)?;
        self.connector(connector)?;
        let tx = self.db.transaction()?;
        tx.execute(
            "UPDATE feeds SET files_connector_id = COALESCE(files_connector_id, connector_id), \
             connector_id = ?2, path = ?3 WHERE id = ?1",
            params![feed.0, connector.0, path],
        )?;
        put_address(&tx, feed, AddressSource::Read, None)?;
        put_address(&tx, feed, AddressSource::Folder, public_url)?;
        choose_address(&tx, feed)?;
        tx.commit()?;
        Ok(())
    }

    /// Add or replace the draft of a feed.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn save_draft(&self, id: FeedId, draft: &DraftRow) -> Result<(), CollectionError> {
        put_draft(&self.db, id, draft)
    }

    /// Save the drafts of several feeds in one transaction (ADR 0122 item 8).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails. No draft changes then.
    pub fn save_drafts(&self, drafts: &[(FeedId, DraftRow)]) -> Result<(), CollectionError> {
        let tx = self.db.unchecked_transaction()?;
        for (id, draft) in drafts {
            put_draft(&tx, *id, draft)?;
        }
        tx.commit()?;
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

    /// Remove the release folder of a feed, so a test can move it again.
    #[cfg(test)]
    pub(crate) fn set_feed_folder_none_for_test(&self, id: FeedId) {
        self.db
            .execute("UPDATE feeds SET folder = NULL WHERE id = ?1", [id.0])
            .unwrap();
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

    /// Give a connector a new name.
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::NotFound`] when the connector does not
    /// exist.
    pub fn set_connector_name(&self, id: ConnectorId, name: &str) -> Result<(), CollectionError> {
        self.connector(id)?;
        self.db.execute(
            "UPDATE connectors SET name = ?2 WHERE id = ?1",
            params![id.0, name],
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
        // The grants that the draft asks for go out with the draft
        // (ADR 0054 item 12).
        self.remove_draft_grants(id)?;
        self.db
            .execute("DELETE FROM drafts WHERE feed_id = ?1", [id.0])?;
        Ok(())
    }

    /// Keep a show grant that the draft of a feed asks for (ADR 0054
    /// item 11). The grant names one app (ADR 0110 item 19). A grant of the
    /// same live item that is there already takes the new app.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn add_draft_grant(
        &self,
        feed: FeedId,
        item_guid: &str,
        app_id: i64,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "INSERT INTO draft_grants (feed_id, item_guid, app_id) VALUES (?1, ?2, ?3)
             ON CONFLICT (feed_id, item_guid) DO UPDATE SET app_id = ?3",
            params![feed.0, item_guid, app_id],
        )?;
        Ok(())
    }

    /// The app that a show grant of the draft names, or `None` when the
    /// draft has no such grant (ADR 0110 item 19).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn draft_grant_app(
        &self,
        feed: FeedId,
        item_guid: &str,
    ) -> Result<Option<i64>, CollectionError> {
        Ok(self
            .db
            .query_row(
                "SELECT app_id FROM draft_grants WHERE feed_id = ?1 AND item_guid = ?2",
                params![feed.0, item_guid],
                |r| r.get::<_, Option<i64>>(0),
            )
            .optional()?
            .flatten())
    }

    /// Remove a show grant from the draft of a feed. A grant that is not
    /// there gives no error.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn remove_draft_grant(&self, feed: FeedId, item_guid: &str) -> Result<(), CollectionError> {
        self.db.execute(
            "DELETE FROM draft_grants WHERE feed_id = ?1 AND item_guid = ?2",
            params![feed.0, item_guid],
        )?;
        Ok(())
    }

    /// The item GUID of each show grant that the draft of a feed asks for,
    /// in the sequence of the GUIDs.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn draft_grants(&self, feed: FeedId) -> Result<Vec<String>, CollectionError> {
        let mut statement = self
            .db
            .prepare("SELECT item_guid FROM draft_grants WHERE feed_id = ?1 ORDER BY item_guid")?;
        let rows = statement.query_map([feed.0], |r| r.get(0))?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Remove each show grant that the draft of a feed asks for.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn remove_draft_grants(&self, feed: FeedId) -> Result<(), CollectionError> {
        self.db
            .execute("DELETE FROM draft_grants WHERE feed_id = ?1", [feed.0])?;
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

    /// The pending files of a draft, by path (ADR 0026 item 6). After the
    /// move into the library drops the table, a draft has no pending file
    /// (ADR 0036 item 5).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn draft_files(&self, id: FeedId) -> Result<Vec<DraftFile>, CollectionError> {
        if !self.has_draft_files_table()? {
            return Ok(Vec::new());
        }
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

    /// True while the table of the pending files is in the database.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub(crate) fn has_draft_files_table(&self) -> Result<bool, CollectionError> {
        Ok(self.db.query_row(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'draft_files')",
            [],
            |r| r.get(0),
        )?)
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
        sha256: Option<&str>,
        media_type: &str,
        local: bool,
        source_url: Option<&str>,
        stamp: Option<&str>,
    ) -> Result<LibraryFileId, CollectionError> {
        let size = i64::try_from(size).unwrap_or(i64::MAX);
        self.db.execute(
            "INSERT INTO library_files (feed_id, path, size, sha256, media_type, local, source_url, added_at, stamp, imported_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?8)",
            params![feed.0, path, size, sha256, media_type, local, source_url, now(), stamp],
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

    /// Record that the bytes of a library file are in the library now:
    /// `local` becomes true, with the size, the SHA-256 and the stamp of
    /// the file (ADR 0038 item 11).
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::NotFound`] when the file does not
    /// exist.
    pub fn set_library_file_local(
        &self,
        id: LibraryFileId,
        size: u64,
        sha256: &str,
        stamp: Option<&str>,
    ) -> Result<(), CollectionError> {
        let size = i64::try_from(size).unwrap_or(i64::MAX);
        let changed = self.db.execute(
            "UPDATE library_files SET local = 1, size = ?2, sha256 = ?3, stamp = ?4 WHERE id = ?1",
            params![id.0, size, sha256, stamp],
        )?;
        if changed == 0 {
            return Err(CollectionError::with_kind(
                CollectionErrorKind::NotFound,
                format!("library file {} does not exist", id.0),
            ));
        }
        Ok(())
    }

    /// Record the stamp of a library file again, with no other change.
    /// The folder scan uses this when the modification time of a file
    /// changed and its bytes did not (ADR 0038 item 13).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn set_library_file_stamp(
        &self,
        id: LibraryFileId,
        stamp: Option<&str>,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "UPDATE library_files SET stamp = ?2 WHERE id = ?1",
            params![id.0, stamp],
        )?;
        Ok(())
    }

    /// Record that the bytes of a library file are not in the library
    /// now: `local` becomes false and the stamp goes. The size and the
    /// SHA-256 stay, so that Studio can find a copy of the bytes
    /// (ADR 0038 item 15).
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::NotFound`] when the file does not
    /// exist.
    pub fn set_library_file_remote(&self, id: LibraryFileId) -> Result<(), CollectionError> {
        let changed = self.db.execute(
            "UPDATE library_files SET local = 0, stamp = NULL WHERE id = ?1",
            [id.0],
        )?;
        if changed == 0 {
            return Err(CollectionError::with_kind(
                CollectionErrorKind::NotFound,
                format!("library file {} does not exist", id.0),
            ));
        }
        Ok(())
    }

    /// Record that Studio removed the local bytes of an asset (ADR 0105
    /// items 5 and 10): `local` becomes false and the stamp goes. The
    /// size and the SHA-256 stay. With `source_url`, the row keeps the
    /// address of the copy on the host. With `removed_at`, the row records
    /// the time of the removal of a pending file.
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::NotFound`] when the file does not
    /// exist.
    pub fn set_library_file_removed(
        &self,
        id: LibraryFileId,
        source_url: Option<&str>,
        removed_at: Option<i64>,
    ) -> Result<(), CollectionError> {
        let changed = self.db.execute(
            "UPDATE library_files
             SET local = 0, stamp = NULL, source_url = COALESCE(?2, source_url), removed_at = ?3
             WHERE id = ?1",
            params![id.0, source_url, removed_at],
        )?;
        if changed == 0 {
            return Err(CollectionError::with_kind(
                CollectionErrorKind::NotFound,
                format!("library file {} does not exist", id.0),
            ));
        }
        Ok(())
    }

    /// Record that a removed pending file has its bytes again: `local`
    /// becomes true, `imported_at` is now and `removed_at` goes (ADR 0105
    /// item 10).
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::NotFound`] when the file does not
    /// exist.
    pub fn restore_library_file(
        &self,
        id: LibraryFileId,
        stamp: Option<&str>,
    ) -> Result<(), CollectionError> {
        let changed = self.db.execute(
            "UPDATE library_files
             SET local = 1, stamp = ?2, imported_at = ?3, removed_at = NULL
             WHERE id = ?1",
            params![id.0, stamp, now()],
        )?;
        if changed == 0 {
            return Err(CollectionError::with_kind(
                CollectionErrorKind::NotFound,
                format!("library file {} does not exist", id.0),
            ));
        }
        Ok(())
    }

    /// Use the rules of a workspace that holds no assets, or `None` for a
    /// workspace that holds assets (ADR 0105 item 1). With the rules,
    /// the function records the time that the mode `off` began, if it
    /// did not begin before. A pending file is a file that came in after
    /// this time. A file from before stays (ADR 0105 items 8, 12 and 13).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn set_asset_rules(
        &mut self,
        rules: Option<AssetRules>,
        now: i64,
    ) -> Result<(), CollectionError> {
        let began = self.setting(ASSETS_OFF_SINCE)?.is_some();
        match (&rules, began) {
            (Some(_), false) => self.set_setting(ASSETS_OFF_SINCE, &Value::from(now))?,
            (None, true) => {
                self.db
                    .execute("DELETE FROM settings WHERE key = ?1", [ASSETS_OFF_SINCE])?;
            }
            _ => {}
        }
        self.asset_rules = rules;
        Ok(())
    }

    /// The rules of a workspace that holds no assets. `None` means that
    /// the workspace holds assets (ADR 0105 item 1).
    #[must_use]
    pub fn asset_rules(&self) -> Option<&AssetRules> {
        self.asset_rules.as_ref()
    }

    /// The time that the mode `off` began, in seconds after the Unix
    /// epoch. `None` while the workspace holds assets.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn assets_off_since(&self) -> Result<Option<i64>, CollectionError> {
        Ok(self.setting(ASSETS_OFF_SINCE)?.and_then(|v| v.as_i64()))
    }

    /// Record a download as queued. A download that failed before is
    /// queued again, and loses its message (ADR 0038 item 9).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails, or when the library
    /// file does not exist.
    pub fn queue_download(&self, file: LibraryFileId) -> Result<(), CollectionError> {
        self.db.execute(
            "INSERT INTO downloads (library_file_id, state, message, added_at)
             VALUES (?1, 'queued', NULL, ?2)
             ON CONFLICT (library_file_id)
             DO UPDATE SET state = 'queued', message = NULL",
            params![file.0, now()],
        )?;
        Ok(())
    }

    /// Record that a download failed, with the reason.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn fail_download(&self, file: LibraryFileId, message: &str) -> Result<(), CollectionError> {
        self.db.execute(
            "UPDATE downloads SET state = 'failed', message = ?2 WHERE library_file_id = ?1",
            params![file.0, message],
        )?;
        Ok(())
    }

    /// Remove the record of a download. A file with no record is not an
    /// error.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn remove_download(&self, file: LibraryFileId) -> Result<(), CollectionError> {
        self.db
            .execute("DELETE FROM downloads WHERE library_file_id = ?1", [file.0])?;
        Ok(())
    }

    /// Each download that is not complete, the oldest first.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn downloads(&self) -> Result<Vec<DownloadRow>, CollectionError> {
        let mut statement = self.db.prepare(
            "SELECT library_file_id, state, message, added_at FROM downloads
             ORDER BY added_at, library_file_id",
        )?;
        let rows = statement.query_map([], |r| {
            Ok(DownloadRow {
                library_file_id: LibraryFileId(r.get(0)?),
                state: r.get(1)?,
                message: r.get(2)?,
                added_at: r.get(3)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
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

    /// Add an upload with no part yet (ADR 0059 item 1).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn add_upload(&self, upload: &UploadRow) -> Result<(), CollectionError> {
        self.db.execute(
            "INSERT INTO uploads (id, feed_id, name, size, replace, received, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, '[]', ?6)",
            params![
                upload.id,
                upload.feed_id.0,
                upload.name,
                i64::try_from(upload.size).unwrap_or(i64::MAX),
                upload.replace,
                i64::try_from(upload.updated_at).unwrap_or(i64::MAX)
            ],
        )?;
        Ok(())
    }

    fn upload_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<UploadRow> {
        let received: String = row.get(5)?;
        let mut received: Vec<u32> = serde_json::from_str(&received).unwrap_or_default();
        received.sort_unstable();
        received.dedup();
        Ok(UploadRow {
            id: row.get(0)?,
            feed_id: FeedId(row.get(1)?),
            name: row.get(2)?,
            size: u64::try_from(row.get::<_, i64>(3)?).unwrap_or(0),
            replace: row.get(4)?,
            received,
            updated_at: u64::try_from(row.get::<_, i64>(6)?).unwrap_or(0),
        })
    }

    /// One upload by its ID.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] of kind `NotFound` when there is no such
    /// upload, and one of kind `Database` when SQLite fails.
    pub fn upload(&self, id: &str) -> Result<UploadRow, CollectionError> {
        self.db
            .query_row(
                "SELECT id, feed_id, name, size, replace, received, updated_at
                 FROM uploads WHERE id = ?1",
                [id],
                Self::upload_from_row,
            )
            .optional()?
            .ok_or_else(|| {
                CollectionError::with_kind(CollectionErrorKind::NotFound, format!("upload {id}"))
            })
    }

    /// The open uploads of a feed, the oldest first.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn uploads_of_feed(&self, feed: FeedId) -> Result<Vec<UploadRow>, CollectionError> {
        let mut statement = self.db.prepare(
            "SELECT id, feed_id, name, size, replace, received, updated_at
             FROM uploads WHERE feed_id = ?1 ORDER BY updated_at, id",
        )?;
        let rows = statement.query_map([feed.0], Self::upload_from_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// The uploads with no new part since `before` (a Unix time).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn uploads_older_than(&self, before: u64) -> Result<Vec<UploadRow>, CollectionError> {
        let mut statement = self.db.prepare(
            "SELECT id, feed_id, name, size, replace, received, updated_at
             FROM uploads WHERE updated_at < ?1 ORDER BY updated_at, id",
        )?;
        let rows = statement.query_map(
            [i64::try_from(before).unwrap_or(i64::MAX)],
            Self::upload_from_row,
        )?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Record that Studio has part `part` of an upload, in one statement,
    /// so that two parts at the same time cannot lose one. A part that
    /// comes again changes nothing except the time.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] of kind `NotFound` when there is no such
    /// upload, and one of kind `Database` when SQLite fails.
    pub fn record_upload_part(&self, id: &str, part: u32, now: u64) -> Result<(), CollectionError> {
        let changed = self.db.execute(
            "UPDATE uploads SET
                 received = (SELECT json_group_array(v) FROM (
                     SELECT value AS v FROM json_each(uploads.received)
                     UNION SELECT ?2 ORDER BY v)),
                 updated_at = ?3
             WHERE id = ?1",
            params![id, part, i64::try_from(now).unwrap_or(i64::MAX)],
        )?;
        if changed == 0 {
            return Err(CollectionError::with_kind(
                CollectionErrorKind::NotFound,
                format!("upload {id}"),
            ));
        }
        Ok(())
    }

    /// Remove an upload row. Gives true when the row was there.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn remove_upload(&self, id: &str) -> Result<bool, CollectionError> {
        Ok(self.db.execute("DELETE FROM uploads WHERE id = ?1", [id])? > 0)
    }

    /// The blob records that a host keeps, in the sequence of the paths
    /// (ADR 0050 item 6).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn kept_blobs(&self, connector: ConnectorId) -> Result<Vec<KeptBlob>, CollectionError> {
        let mut statement = self.db.prepare(
            "SELECT path, sha256, size FROM kept_blobs WHERE connector_id = ?1 ORDER BY path",
        )?;
        let rows = statement.query_map([connector.0], |row| {
            Ok(KeptBlob {
                path: row.get(0)?,
                sha256: row.get(1)?,
                size: u64::try_from(row.get::<_, i64>(2)?).unwrap_or(0),
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Replace the blob records of a host with `blobs`, in one transaction.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails. The records then do
    /// not change.
    pub fn replace_kept_blobs(
        &mut self,
        connector: ConnectorId,
        blobs: &[KeptBlob],
    ) -> Result<(), CollectionError> {
        let tx = self.db.transaction()?;
        tx.execute(
            "DELETE FROM kept_blobs WHERE connector_id = ?1",
            [connector.0],
        )?;
        for blob in blobs {
            tx.execute(
                "INSERT INTO kept_blobs (connector_id, path, sha256, size) VALUES (?1, ?2, ?3, ?4)",
                params![
                    connector.0,
                    blob.path,
                    blob.sha256,
                    i64::try_from(blob.size).unwrap_or(i64::MAX)
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
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

    /// Remove one host operation of a draft (ADR 0036 item 12). The route
    /// that lets the user take back a copy or a deletion before the write
    /// uses this.
    ///
    /// # Errors
    ///
    /// Gives [`CollectionErrorKind::NotFound`] when the operation does not
    /// exist for this feed, or a [`CollectionError`] when SQLite fails.
    pub fn remove_draft_operation(
        &self,
        feed: FeedId,
        id: DraftOperationId,
    ) -> Result<(), CollectionError> {
        let changed = self.db.execute(
            "DELETE FROM draft_operations WHERE id = ?1 AND feed_id = ?2",
            params![id.0, feed.0],
        )?;
        if changed == 0 {
            return Err(CollectionError::with_kind(
                CollectionErrorKind::NotFound,
                format!(
                    "draft operation {} does not exist for feed {}",
                    id.0, feed.0
                ),
            ));
        }
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

    /// Each row of `feed_credits`, by feed ID and then by name (ADR 0098
    /// item 10).
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails, or when a stored list
    /// of places is not valid.
    pub fn feed_credits(&self) -> Result<Vec<CreditRow>, CollectionError> {
        let mut statement = self.db.prepare(
            "SELECT feed_id, name, role, grp, img, href, npub, places
             FROM feed_credits ORDER BY feed_id, name, role, grp, href, npub",
        )?;
        let rows = statement.query_map([], |r| {
            let row = CreditRow {
                feed_id: FeedId(r.get(0)?),
                name: r.get(1)?,
                role: r.get(2)?,
                group: r.get(3)?,
                img: r.get(4)?,
                href: r.get(5)?,
                npub: r.get(6)?,
                places: Vec::new(),
            };
            Ok((row, r.get::<_, String>(7)?))
        })?;
        let mut found = Vec::new();
        for row in rows {
            let (mut credit, places) = row?;
            credit.places = serde_json::from_str(&places).map_err(|e| {
                CollectionError::new(format!("the places of a credit are not valid: {e}"))
            })?;
            found.push(credit);
        }
        Ok(found)
    }

    /// Each recipient that the collection knows: from a feed that Studio
    /// read, or from a wallet of a person (ADR 0027 items 2 to 5, ADR 0098).
    ///
    /// The list has each identity one time. The name and the type come from
    /// the feed row with the highest `seen_at`. A wallet of a person always
    /// gives the name of the person, because the user owns that name. Its
    /// type replaces the type of the feeds when it is not empty. `saved_id`
    /// is the person ID. The sequence is by name, then by address, with no
    /// regard to capital letters.
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
            "SELECT p.id, w.address, w.custom_key, w.custom_value, p.name, w.recipient_type
             FROM person_wallets w JOIN people p ON p.id = w.person_id",
        )?;
        let rows = statement.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        })?;
        for row in rows {
            let (person_id, address, custom_key, custom_value, name, recipient_type) = row?;
            let key = (address.clone(), custom_key.clone(), custom_value.clone());
            let agg = found
                .entry(key)
                .or_insert_with(|| RecipientAgg::new(address, custom_key, custom_value));
            agg.saved_id = Some(person_id);
            let recipient_type = non_empty(recipient_type).or(agg.recipient_type.take());
            agg.consider(i64::MAX, Some(name), recipient_type);
        }
        drop(statement);

        let mut list: Vec<KnownRecipient> = found.into_values().map(RecipientAgg::finish).collect();
        list.sort_by(|a, b| {
            let a_name = a.name.as_deref().unwrap_or_default().to_lowercase();
            let b_name = b.name.as_deref().unwrap_or_default().to_lowercase();
            a_name.cmp(&b_name).then_with(|| a.address.cmp(&b.address))
        });
        Ok(list)
    }

    pub(crate) fn db(&self) -> &Connection {
        &self.db
    }
}

/// A row of `change_sets` (ADR 0091 item 19).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeSetRow {
    /// The ID.
    pub id: i64,
    /// The operation, as JSON text.
    pub operation: String,
    /// The recipient key, as JSON text.
    pub recipient: String,
    /// The Unix time when the user made the change set.
    pub made_at: i64,
    /// The person of a person operation, or `None`.
    pub person_id: Option<i64>,
}

/// A row of `change_set_feeds`: the outcome for one feed, and the draft of
/// the feed from before the change (ADR 0091 items 19 and 20).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeSetFeedRow {
    /// The feed.
    pub feed_id: FeedId,
    /// The outcome, as JSON text.
    pub outcome: String,
    /// The draft from before the change. `None` when the feed had no draft
    /// or the outcome is not a change.
    pub before: Option<DraftRow>,
    /// True when the feed had a draft before the change.
    pub had_draft: bool,
    /// The Unix time when Studio wrote the feed after the change.
    pub written_at: Option<i64>,
}

impl Collection {
    /// Add a change set. `person_id` is the person of a person operation.
    /// Gives its ID.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn add_change_set(
        &self,
        operation: &str,
        recipient: &str,
        made_at: i64,
        person_id: Option<i64>,
    ) -> Result<i64, CollectionError> {
        self.db.execute(
            "INSERT INTO change_sets (operation, recipient, made_at, person_id)
             VALUES (?1, ?2, ?3, ?4)",
            params![operation, recipient, made_at, person_id],
        )?;
        Ok(self.db.last_insert_rowid())
    }

    /// Add the row of one feed to a change set.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn add_change_set_feed(
        &self,
        change_set: i64,
        row: &ChangeSetFeedRow,
    ) -> Result<(), CollectionError> {
        self.db.execute(
            "INSERT INTO change_set_feeds
                 (change_set_id, feed_id, outcome, before_xml, before_base_xml,
                  before_base_version, had_draft, written_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                change_set,
                row.feed_id.0,
                row.outcome,
                row.before.as_ref().map(|d| d.draft_xml.as_slice()),
                row.before.as_ref().map(|d| d.base_xml.as_slice()),
                row.before
                    .as_ref()
                    .and_then(|d| d.base_version.as_ref().map(Version::as_str)),
                row.had_draft,
                row.written_at
            ],
        )?;
        Ok(())
    }

    /// Each change set, the oldest first.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn change_set_rows(&self) -> Result<Vec<ChangeSetRow>, CollectionError> {
        let mut statement = self.db.prepare(
            "SELECT id, operation, recipient, made_at, person_id FROM change_sets ORDER BY id",
        )?;
        let rows = statement
            .query_map([], |r| {
                Ok(ChangeSetRow {
                    id: r.get(0)?,
                    operation: r.get(1)?,
                    recipient: r.get(2)?,
                    made_at: r.get(3)?,
                    person_id: r.get(4)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(rows)
    }

    /// The row of one change set, or `None`.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn change_set_row(&self, id: i64) -> Result<Option<ChangeSetRow>, CollectionError> {
        Ok(self
            .db
            .query_row(
                "SELECT id, operation, recipient, made_at, person_id FROM change_sets WHERE id = ?1",
                [id],
                |r| {
                    Ok(ChangeSetRow {
                        id: r.get(0)?,
                        operation: r.get(1)?,
                        recipient: r.get(2)?,
                        made_at: r.get(3)?,
                        person_id: r.get(4)?,
                    })
                },
            )
            .optional()?)
    }

    /// The feed rows of a change set, in the sequence of the feed IDs.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn change_set_feed_rows(
        &self,
        change_set: i64,
    ) -> Result<Vec<ChangeSetFeedRow>, CollectionError> {
        let mut statement = self.db.prepare(
            "SELECT feed_id, outcome, before_xml, before_base_xml, before_base_version,
                    had_draft, written_at
             FROM change_set_feeds WHERE change_set_id = ?1 ORDER BY feed_id",
        )?;
        let rows = statement
            .query_map([change_set], |r| {
                let before_xml: Option<Vec<u8>> = r.get(2)?;
                let before_base_xml: Option<Vec<u8>> = r.get(3)?;
                let before_base_version: Option<String> = r.get(4)?;
                Ok(ChangeSetFeedRow {
                    feed_id: FeedId(r.get(0)?),
                    outcome: r.get(1)?,
                    before: before_xml.map(|draft_xml| DraftRow {
                        base_version: before_base_version.map(Version::new),
                        base_xml: before_base_xml.unwrap_or_default(),
                        draft_xml,
                    }),
                    had_draft: r.get(5)?,
                    written_at: r.get(6)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(rows)
    }

    /// Mark each feed row of an open change set as written. A row that is
    /// marked already does not change. Gives the number of rows.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn mark_change_set_written(
        &self,
        feed: FeedId,
        written_at: i64,
    ) -> Result<usize, CollectionError> {
        Ok(self.db.execute(
            "UPDATE change_set_feeds SET written_at = ?2
             WHERE feed_id = ?1 AND written_at IS NULL",
            params![feed.0, written_at],
        )?)
    }

    /// Remove each closed change set: a change set with no changed feed
    /// that is not written (ADR 0091 item 19). The feed rows go with it,
    /// through the foreign key. Gives the number of change sets.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn delete_closed_change_sets(&self) -> Result<usize, CollectionError> {
        Ok(self.db.execute(
            "DELETE FROM change_sets WHERE NOT EXISTS (
                 SELECT 1 FROM change_set_feeds f
                 WHERE f.change_set_id = change_sets.id AND f.written_at IS NULL
                   AND json_extract(f.outcome, '$.outcome') = 'changed')",
            [],
        )?)
    }

    /// Remove a change set and its feed rows.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn delete_change_set(&self, id: i64) -> Result<(), CollectionError> {
        self.db.execute(
            "DELETE FROM change_set_feeds WHERE change_set_id = ?1",
            [id],
        )?;
        self.db
            .execute("DELETE FROM change_sets WHERE id = ?1", [id])?;
        Ok(())
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

/// Add or replace the draft of a feed on a connection or a transaction.
fn put_draft(db: &Connection, id: FeedId, draft: &DraftRow) -> Result<(), CollectionError> {
    db.execute(
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

/// Turn the empty text that the database uses for a missing custom key or
/// custom value back into `None`.
fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

const FEED_SELECT: &str = "SELECT id, public_url, connector_id, path, podcast_guid, title, medium, last_version, address_source, folder, files_connector_id, publisher_roles, roles_confirmed, created_at FROM feeds";

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

/// The feed in the library copy of `row`, or `None` when the feed has no
/// release folder, no feed XML or an XML that is not a feed.
fn library_copy(library: &Library, row: &FeedRow) -> Option<Feed> {
    let folder = row.folder.as_deref()?;
    let bytes = [row.path.as_deref(), row.public_url.as_deref()]
        .into_iter()
        .flatten()
        .map(|name| name.rsplit_once('/').map_or(name, |(_, last)| last))
        .find_map(|name| std::fs::read(library.path(folder, name)).ok())?;
    let doc = Document::parse_bytes(&bytes).ok()?;
    Feed::read(&doc).ok()
}

/// One row of `feed_credits`: a credit of a feed (ADR 0098 item 10).
///
/// Each text is trimmed. A missing attribute is empty text, and `role` is
/// the raw role text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditRow {
    /// The feed that has the credit.
    pub feed_id: FeedId,
    /// The name of the person.
    pub name: String,
    /// `role`, or empty text.
    pub role: String,
    /// `group`, or empty text.
    pub group: String,
    /// `img`, or empty text.
    pub img: String,
    /// `href`, or empty text.
    pub href: String,
    /// `npub`, or empty text.
    pub npub: String,
    /// The places of the credit, in the sequence of the feed, each one time
    /// (ADR 0120 item 2).
    pub places: Vec<CreditPlace>,
}

/// One row of `feed_credits` before it is written. The key fields are the
/// primary key of the table.
struct CreditCell {
    name: String,
    role: String,
    group: String,
    img: String,
    href: String,
    npub: String,
    places: Vec<CreditPlace>,
}

impl CreditCell {
    /// True when `other` has the same primary key as this row. The picture
    /// is not part of the key.
    fn same_key(&self, other: &Self) -> bool {
        self.name == other.name
            && self.role == other.role
            && self.group == other.group
            && self.href == other.href
            && self.npub == other.npub
    }
}

/// Replace the `feed_credits` rows of a feed with its current credits
/// (ADR 0098 item 10, ADR 0120 items 2 and 3).
///
/// Two credits with the same key give one row. The last one gives `img`.
/// The row gets the places of both, in the sequence of the feed, and each
/// place one time.
fn put_credits(
    db: &Connection,
    id: FeedId,
    credits: Vec<(CreditPlace, &Person)>,
) -> Result<(), CollectionError> {
    db.execute("DELETE FROM feed_credits WHERE feed_id = ?1", [id.0])?;
    let seen_at = now();
    let text = |value: Option<&String>| value.map_or_else(String::new, |v| v.trim().to_owned());
    let mut cells: Vec<CreditCell> = Vec::new();
    for (place, person) in credits {
        let cell = CreditCell {
            name: person.name.trim().to_owned(),
            role: text(person.role.as_ref()),
            group: text(person.group.as_ref()),
            img: text(person.img.as_ref()),
            href: text(person.href.as_ref()),
            npub: text(person.npub.as_ref()),
            places: Vec::new(),
        };
        if let Some(index) = cells.iter().position(|c| c.same_key(&cell)) {
            let existing = &mut cells[index];
            existing.img = cell.img;
            if !existing.places.contains(&place) {
                existing.places.push(place);
            }
        } else {
            cells.push(CreditCell {
                places: vec![place],
                ..cell
            });
        }
    }
    for cell in cells {
        let places = serde_json::to_string(&cell.places).map_err(|e| {
            CollectionError::new(format!("the places of a credit cannot be written: {e}"))
        })?;
        db.execute(
            "INSERT INTO feed_credits
                 (feed_id, name, role, grp, img, href, npub, seen_at, places)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id.0, cell.name, cell.role, cell.group, cell.img, cell.href, cell.npub, seen_at,
                places
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
        publisher_roles: r
            .get::<_, Option<String>>(11)?
            .as_deref()
            .map(roles_from_rel)
            .unwrap_or_default(),
        roles_confirmed: r.get(12)?,
        added_at: r.get(13)?,
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
        stamp: r.get(10)?,
        imported_at: r.get::<_, Option<i64>>(11)?.unwrap_or(r.get::<_, i64>(9)?),
        removed_at: r.get(12)?,
    })
}

const LIBRARY_FILE_SELECT: &str = "SELECT id, feed_id, path, size, sha256, media_type, local, source_url, connector_id, added_at, stamp, imported_at, removed_at FROM library_files";

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
        kind: DraftOperationKind::parse(&kind)
            .expect("the kind column allows only copy, delete or confirm"),
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
        assert_eq!(collection.feeds().unwrap(), Vec::new());
        assert_eq!(
            collection.delete_feed(feed).unwrap_err().kind(),
            CollectionErrorKind::NotFound
        );
        collection.delete_connector(connector).unwrap();
        assert_eq!(collection.connectors().unwrap(), Vec::new());
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
        crate::people::save_recipient(
            &collection,
            &Recipient {
                address: Some("addr2".to_owned()),
                ..Recipient::default()
            },
        )
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
        assert_eq!(collection.known_recipients().unwrap(), Vec::new());
        crate::people::save_recipient(
            &collection,
            &Recipient {
                address: Some("addr".to_owned()),
                ..Recipient::default()
            },
        )
        .unwrap();
        assert_eq!(collection.known_recipients().unwrap().len(), 1);
    }

    /// Make a database at the version before the migration of the people.
    fn before_people(path: &Path) -> Connection {
        let db = Connection::open(path).unwrap();
        for migration in &MIGRATIONS[..PEOPLE_MIGRATION] {
            db.execute_batch(migration).unwrap();
        }
        db.pragma_update(
            None,
            "user_version",
            i64::try_from(PEOPLE_MIGRATION).unwrap(),
        )
        .unwrap();
        db
    }

    /// Make a database at the version before the relative library migration,
    /// with `library_root` set to `root`.
    fn before_relative_library(path: &Path, root: &str) {
        let db = Connection::open(path).unwrap();
        for migration in &MIGRATIONS[..RELATIVE_LIBRARY_MIGRATION] {
            db.execute_batch(migration).unwrap();
        }
        db.pragma_update(
            None,
            "user_version",
            i64::try_from(RELATIVE_LIBRARY_MIGRATION).unwrap(),
        )
        .unwrap();
        db.execute(
            "INSERT INTO settings (key, value) VALUES ('library_root', ?1)",
            [serde_json::json!(root).to_string()],
        )
        .unwrap();
    }

    #[test]
    fn the_relative_library_migration_has_its_index() {
        assert_eq!(
            MIGRATIONS[RELATIVE_LIBRARY_MIGRATION], "SELECT 1;",
            "ADR 0107 item 1: RELATIVE_LIBRARY_MIGRATION must be the index of its entry in MIGRATIONS"
        );
    }

    #[test]
    fn the_migration_makes_a_library_in_the_data_folder_relative_and_keeps_one_outside() {
        let dir = tempfile::tempdir().unwrap();
        let data = dir.path().join("data");
        std::fs::create_dir_all(data.join("library")).unwrap();
        let path = data.join("collection.sqlite");
        before_relative_library(&path, data.join("library").to_str().unwrap());
        let collection = Collection::open(&path).unwrap();
        assert_eq!(
            collection.setting("library_root").unwrap(),
            Some(serde_json::json!("library")),
            "ADR 0107 item 1: a library in the data folder must become a relative path"
        );
        assert_eq!(
            collection.library_root().unwrap(),
            Some(data.join("library"))
        );
        drop(collection);

        let outside = dir.path().join("music");
        std::fs::create_dir_all(&outside).unwrap();
        let other = dir.path().join("other");
        std::fs::create_dir_all(&other).unwrap();
        let path = other.join("collection.sqlite");
        before_relative_library(&path, outside.to_str().unwrap());
        let collection = Collection::open(&path).unwrap();
        assert_eq!(
            collection.setting("library_root").unwrap(),
            Some(serde_json::json!(outside.to_str().unwrap())),
            "ADR 0107 item 1: a library outside the data folder must stay absolute"
        );
    }

    #[test]
    fn the_people_migration_is_the_one_that_makes_the_people() {
        assert!(MIGRATIONS[PEOPLE_MIGRATION].contains("CREATE TABLE people"));
    }

    #[test]
    fn the_people_migration_makes_one_person_from_each_saved_recipient() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = before_people(&path);
        db.execute_batch(
            "INSERT INTO saved_recipients VALUES (4, 'a1', 'k', 'v', ' Ann ', 'node', 100);
             INSERT INTO saved_recipients VALUES (9, 'nameless', '', '', NULL, NULL, 200);",
        )
        .unwrap();
        drop(db);
        let collection = Collection::open(&path).unwrap();
        let people = crate::people::people(&collection).unwrap();
        assert_eq!(people.len(), 2);
        assert_eq!(people[0].id.0, 4);
        assert_eq!(people[0].name, "Ann");
        assert_eq!(people[0].wallets.len(), 1);
        assert_eq!(people[0].wallets[0].key.address, "a1");
        assert_eq!(people[0].wallets[0].key.custom_key.as_deref(), Some("k"));
        assert_eq!(people[0].wallets[0].recipient_type, "node");
        assert_eq!(people[0].wallets[0].evidence, "user");
        assert_eq!(people[0].wallets[0].linked_at, 100);
        assert_eq!(people[1].id.0, 9);
        assert_eq!(people[1].name, "nameless");
        let left: i64 = collection
            .db
            .query_row("SELECT COUNT(*) FROM saved_recipients", [], |r| r.get(0))
            .unwrap();
        assert_eq!(left, 0);
        assert_eq!(collection.known_recipients().unwrap().len(), 2);
    }

    #[test]
    fn the_people_migration_puts_a_lightning_address_in_the_person_of_its_node_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = before_people(&path);
        db.execute_batch(
            "INSERT INTO saved_recipients VALUES (1, 'ann@fountain.example', '', '', 'Ann LN', 'lnaddress', 100);
             INSERT INTO saved_recipients VALUES (2, 'node-1', 'k', 'v', '', 'node', 200);
             INSERT INTO saved_recipients VALUES (3, 'bob@fountain.example', '', '', 'Bob', 'lnaddress', 300);
             INSERT INTO keysend_lookups VALUES ('ann@fountain.example', 'node-1', 'k', 'v', 'found', 1);
             INSERT INTO keysend_lookups VALUES ('bob@fountain.example', 'node-1', 'k', 'other', 'found', 1);",
        )
        .unwrap();
        drop(db);
        let collection = Collection::open(&path).unwrap();
        let people = crate::people::people(&collection).unwrap();
        // Ann is the person of the node key. The name of the node key is
        // empty, so the person gets the name of the Lightning address.
        // Bob has a lookup with a different custom pair, so it stays alone.
        assert_eq!(people.len(), 2);
        assert_eq!(people[0].id.0, 2);
        assert_eq!(people[0].name, "Ann LN");
        let wallets: Vec<(&str, i64)> = people[0]
            .wallets
            .iter()
            .map(|w| (w.key.address.as_str(), w.position))
            .collect();
        assert_eq!(wallets, [("node-1", 0), ("ann@fountain.example", 1)]);
        assert_eq!(people[1].id.0, 3);
        assert_eq!(people[1].wallets.len(), 1);
    }

    #[test]
    fn the_people_migration_fills_the_credit_index_of_a_feed_with_a_library_copy() {
        let dir = tempfile::tempdir().unwrap();
        let library = dir.path().join("library");
        std::fs::create_dir_all(library.join("album")).unwrap();
        std::fs::write(
            library.join("album").join("a.xml"),
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><title>A</title>\
             <podcast:person role=\"Mixer\">Ann</podcast:person></channel></rss>",
        )
        .unwrap();
        std::fs::create_dir_all(library.join("broken")).unwrap();
        std::fs::write(library.join("broken").join("b.xml"), "not xml").unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = before_people(&path);
        db.execute(
            "INSERT INTO settings (key, value) VALUES ('library_root', ?1)",
            [serde_json::json!(library.to_str().unwrap()).to_string()],
        )
        .unwrap();
        db.execute_batch(
            "INSERT INTO feeds (id, path, folder, created_at) VALUES (1, 'a.xml', 'album', 0);
             INSERT INTO feeds (id, path, folder, created_at) VALUES (2, 'b.xml', 'broken', 0);
             INSERT INTO feeds (id, path, folder, created_at) VALUES (3, 'c.xml', 'missing', 0);
             INSERT INTO feeds (id, path, created_at) VALUES (4, 'd.xml', 0);",
        )
        .unwrap();
        drop(db);
        let collection = Collection::open(&path).unwrap();
        let rows = collection.feed_credits().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].feed_id, FeedId(1));
        assert_eq!(rows[0].name, "Ann");
        assert_eq!(rows[0].role, "Mixer");
    }

    fn credit_doc(persons: &str) -> Document {
        Document::parse(&format!(
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><title>A</title>\
             {persons}</channel></rss>"
        ))
        .unwrap()
    }

    #[test]
    fn record_state_keeps_one_credit_row_for_equal_credits_and_refreshes_them() {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        let three = credit_doc(
            "<podcast:person role=\"Mixer\" img=\"a.jpg\"> Ann </podcast:person>\
             <podcast:person role=\"Mixer\" img=\"b.jpg\">Ann</podcast:person>\
             <podcast:person>Bob</podcast:person>",
        );
        collection
            .record_state(feed, &Feed::read(&three).unwrap(), &Version::new("v1"))
            .unwrap();
        let rows = collection.feed_credits().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "Ann");
        assert_eq!(rows[0].img, "b.jpg");
        // A missing role stays empty text, not `host`.
        assert_eq!(rows[1].name, "Bob");
        assert_eq!(rows[1].role, "");
        let one = credit_doc("<podcast:person>Bob</podcast:person>");
        collection
            .record_state(feed, &Feed::read(&one).unwrap(), &Version::new("v2"))
            .unwrap();
        assert_eq!(collection.feed_credits().unwrap().len(), 1);
    }

    #[test]
    fn the_delete_of_a_feed_deletes_its_credit_rows() {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        let doc = credit_doc("<podcast:person>Ann</podcast:person>");
        collection
            .record_state(feed, &Feed::read(&doc).unwrap(), &Version::new("v1"))
            .unwrap();
        assert_eq!(collection.feed_credits().unwrap().len(), 1);
        collection.delete_feed(feed).unwrap();
        assert_eq!(collection.feed_credits().unwrap(), Vec::new());
    }

    #[test]
    fn put_credits_keeps_one_row_with_the_places_of_each_credit_in_sequence() {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        let doc = credit_doc(
            "<podcast:person role=\"Mixer\">Ann</podcast:person>\
             <item><guid>t1</guid><title>Track 1</title>\
             <podcast:person role=\"Mixer\">Ann</podcast:person>\
             <podcast:person role=\"Mixer\">Ann</podcast:person></item>\
             <item><guid>t2</guid><title>Track 2</title>\
             <podcast:person role=\"Mixer\">Ann</podcast:person></item>",
        );
        collection
            .record_state(feed, &Feed::read(&doc).unwrap(), &Version::new("v1"))
            .unwrap();
        let rows = collection.feed_credits().unwrap();
        assert_eq!(rows.len(), 1, "ADR 0120 item 3: equal keys give one row");
        let item = |guid: &str, title: &str| CreditPlace::Item {
            guid: guid.into(),
            title: title.into(),
        };
        assert_eq!(
            rows[0].places,
            [
                CreditPlace::Channel,
                item("t1", "Track 1"),
                item("t2", "Track 2"),
            ],
            "ADR 0120 item 2: each place is one time, in the sequence of the feed"
        );
    }

    #[test]
    fn the_credit_places_migration_has_its_index() {
        assert!(
            MIGRATIONS[CREDIT_PLACES_MIGRATION].contains("places"),
            "ADR 0120 item 1: CREDIT_PLACES_MIGRATION must be the index of its entry in MIGRATIONS"
        );
    }

    /// Make a database at the version before the migration of the places.
    fn before_credit_places(path: &Path) -> Connection {
        let db = Connection::open(path).unwrap();
        for migration in &MIGRATIONS[..CREDIT_PLACES_MIGRATION] {
            db.execute_batch(migration).unwrap();
        }
        db.pragma_update(
            None,
            "user_version",
            i64::try_from(CREDIT_PLACES_MIGRATION).unwrap(),
        )
        .unwrap();
        db
    }

    #[test]
    fn the_credit_places_migration_refills_each_feed_with_its_places() {
        let dir = tempfile::tempdir().unwrap();
        let library = dir.path().join("library");
        std::fs::create_dir_all(library.join("album")).unwrap();
        std::fs::write(
            library.join("album").join("a.xml"),
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><title>A</title>\
             <podcast:person role=\"Mixer\">Ann</podcast:person>\
             <item><guid>t1</guid><title>Track</title>\
             <podcast:person role=\"Mixer\">Ann</podcast:person></item></channel></rss>",
        )
        .unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = before_credit_places(&path);
        db.execute(
            "INSERT INTO settings (key, value) VALUES ('library_root', ?1)",
            [serde_json::json!(library.to_str().unwrap()).to_string()],
        )
        .unwrap();
        // A row from before has no places. The refill replaces it.
        db.execute_batch(
            "INSERT INTO feeds (id, path, folder, created_at) VALUES (1, 'a.xml', 'album', 0);
             INSERT INTO feed_credits (feed_id, name, role, grp, img, href, npub, seen_at)
                 VALUES (1, 'Old', '', '', '', '', '', 0);",
        )
        .unwrap();
        drop(db);
        let collection = Collection::open(&path).unwrap();
        let rows = collection.feed_credits().unwrap();
        assert_eq!(
            rows.len(),
            1,
            "ADR 0120 item 4: the refill replaces the old rows"
        );
        assert_eq!(rows[0].name, "Ann");
        assert_eq!(
            rows[0].places,
            [
                CreditPlace::Channel,
                CreditPlace::Item {
                    guid: "t1".into(),
                    title: "Track".into(),
                },
            ]
        );
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
    fn a_feed_row_gives_the_time_that_the_feed_joined_the_collection() {
        let mut collection = Collection::open_in_memory().unwrap();
        let first = collection.add_feed(None, None).unwrap();
        let second = collection.add_feed(None, None).unwrap();
        let feeds = collection.feeds().unwrap();
        let added = |id: FeedId| feeds.iter().find(|row| row.id == id).unwrap().added_at;
        assert!(added(first) > 0);
        assert!(added(second) >= added(first));
        assert_eq!(collection.feed(second).unwrap().added_at, added(second));
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
    fn a_wallet_of_a_person_gives_the_name_of_the_person_after_a_newer_feed_read() {
        let mut collection = Collection::open_in_memory().unwrap();
        let person = crate::people::add_person(
            &collection,
            &crate::people::PersonFields {
                name: "Person".to_owned(),
                ..crate::people::PersonFields::default()
            },
        )
        .unwrap();
        let key = crate::change_set::RecipientKey {
            address: "shared".to_owned(),
            custom_key: None,
            custom_value: None,
        };
        crate::people::add_wallet(
            &collection,
            person,
            &key,
            "node",
            crate::people::LinkEvidence::User,
        )
        .unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        collection
            .record_state(
                feed,
                &Feed::read(&value_recipient_doc("shared", "name=\"Feed\"")).unwrap(),
                &Version::new("v1"),
            )
            .unwrap();
        collection
            .db()
            .execute("UPDATE feed_recipients SET seen_at = ?1", [i64::MAX - 1])
            .unwrap();
        let known = collection.known_recipients().unwrap();
        let shared = known.iter().find(|r| r.address == "shared").unwrap();
        assert_eq!(shared.name.as_deref(), Some("Person"));
        assert_eq!(shared.saved_id, Some(person.0));
        assert_eq!(shared.feeds, 1);
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
        assert_eq!(collection.known_recipients().unwrap(), Vec::new());

        collection
            .record_state(
                feed,
                &Feed::read(&with_recipient).unwrap(),
                &Version::new("v3"),
            )
            .unwrap();
        assert_eq!(collection.known_recipients().unwrap().len(), 1);
        collection.delete_feed(feed).unwrap();
        assert_eq!(collection.known_recipients().unwrap(), Vec::new());
    }

    #[test]
    fn a_saved_recipient_stays_with_no_feed_and_goes_out_when_removed() {
        let collection = Collection::open_in_memory().unwrap();
        let id = crate::people::save_recipient(
            &collection,
            &Recipient {
                address: Some("saved".to_owned()),
                name: Some("Ann".to_owned()),
                ..Recipient::default()
            },
        )
        .unwrap();
        let known = collection.known_recipients().unwrap();
        assert_eq!(known.len(), 1);
        assert_eq!(known[0].feeds, 0);
        assert_eq!(known[0].saved_id, Some(id.0));
        crate::people::remove_person(&collection, id).unwrap();
        assert_eq!(collection.known_recipients().unwrap(), Vec::new());
        assert_eq!(
            crate::people::remove_person(&collection, id)
                .unwrap_err()
                .kind(),
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
    #[allow(clippy::too_many_lines)]
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
            .add_library_file(
                feed,
                "01 Song.mp3",
                10,
                Some("abc"),
                "audio/mpeg",
                true,
                None,
                None,
            )
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
        assert_eq!(collection.host_files(connector).unwrap(), Vec::new());

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
        assert_eq!(collection.draft_operations(feed).unwrap(), Vec::new());

        collection.remove_library_file(file).unwrap();
        assert_eq!(
            collection.remove_library_file(file).unwrap_err().kind(),
            CollectionErrorKind::NotFound
        );
    }

    /// A removal of one operation needs its own feed (ADR 0036 item 12).
    /// A wrong feed, or an ID that is gone already, gives `not_found`.
    #[test]
    fn remove_draft_operation_removes_one_operation_and_checks_its_feed() {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        let connector = collection
            .add_connector("local", "Home", "{}", None)
            .unwrap();
        let operation = collection
            .add_draft_operation(feed, DraftOperationKind::Delete, None, connector, "a.mp3")
            .unwrap();

        assert_eq!(
            collection
                .remove_draft_operation(FeedId(999), operation)
                .unwrap_err()
                .kind(),
            CollectionErrorKind::NotFound
        );
        assert_eq!(collection.draft_operations(feed).unwrap().len(), 1);

        collection.remove_draft_operation(feed, operation).unwrap();
        assert_eq!(collection.draft_operations(feed).unwrap(), Vec::new());
        assert_eq!(
            collection
                .remove_draft_operation(feed, operation)
                .unwrap_err()
                .kind(),
            CollectionErrorKind::NotFound
        );
    }

    #[test]
    fn a_download_is_queued_fails_and_goes_out_and_a_file_becomes_local() {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        let file = collection
            .add_library_file(
                feed,
                "a.mp3",
                5,
                None,
                "audio/mpeg",
                false,
                Some("http://x/a.mp3"),
                None,
            )
            .unwrap();
        collection.queue_download(file).unwrap();
        collection.queue_download(file).unwrap();
        collection.fail_download(file, "no network").unwrap();
        let rows = collection.downloads().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].state, "failed");
        assert_eq!(rows[0].message.as_deref(), Some("no network"));
        collection.queue_download(file).unwrap();
        assert_eq!(collection.downloads().unwrap()[0].state, "queued");
        collection
            .set_library_file_local(file, 9, "abc", Some("9-1"))
            .unwrap();
        let row = collection.library_file(file).unwrap();
        assert!(row.local);
        assert_eq!((row.size, row.sha256.as_deref()), (9, Some("abc")));
        assert_eq!(row.stamp.as_deref(), Some("9-1"));
        collection.remove_download(file).unwrap();
        collection.remove_download(file).unwrap();
        assert_eq!(collection.downloads().unwrap(), Vec::new());
    }

    #[test]
    fn a_recorded_role_becomes_a_confirmed_set_of_one_role() {
        // The collection before ADR 0082 kept one role (ADR 0082 item 10).
        let before = MIGRATIONS
            .iter()
            .position(|m| m.contains("ADD COLUMN publisher_roles"))
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = Connection::open(&path).unwrap();
        for migration in &MIGRATIONS[..before] {
            db.execute_batch(migration).unwrap();
        }
        db.pragma_update(None, "user_version", i64::try_from(before).unwrap())
            .unwrap();
        db.execute_batch(
            "INSERT INTO feeds (id, public_url, publisher_role, created_at) VALUES (1, 'https://example.com/a.xml', 'label', 0);
             INSERT INTO feeds (id, public_url, created_at) VALUES (2, 'https://example.com/b.xml', 0);",
        )
        .unwrap();
        drop(db);

        let collection = Collection::open(&path).unwrap();
        assert_eq!(collection.schema_version().unwrap(), SCHEMA_VERSION);
        let label = collection.feed(FeedId(1)).unwrap();
        assert_eq!(label.publisher_roles, [PublisherRole::Label]);
        assert!(label.roles_confirmed);
        let none = collection.feed(FeedId(2)).unwrap();
        assert_eq!(none.publisher_roles, Vec::new());
        assert!(!none.roles_confirmed);

        collection
            .set_publisher_roles(
                FeedId(2),
                &[PublisherRole::Producer, PublisherRole::Artist],
                false,
            )
            .unwrap();
        let set = collection.feed(FeedId(2)).unwrap();
        assert_eq!(
            set.publisher_roles,
            [PublisherRole::Artist, PublisherRole::Producer]
        );
        assert!(!set.roles_confirmed);
        collection
            .set_publisher_roles(FeedId(2), &[], true)
            .unwrap();
        assert_eq!(
            collection.feed(FeedId(2)).unwrap().publisher_roles,
            Vec::new()
        );
        assert!(!collection.feed(FeedId(2)).unwrap().roles_confirmed);
    }

    /// ADR 0105 item 8: the migration gives each row from before it an
    /// import time, and no removal time.
    #[test]
    fn the_asset_migration_gives_each_library_file_from_before_an_import_time() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = Connection::open(&path).unwrap();
        let before = RELATIVE_LIBRARY_MIGRATION - 1;
        for migration in &MIGRATIONS[..before] {
            db.execute_batch(migration).unwrap();
        }
        db.pragma_update(None, "user_version", i64::try_from(before).unwrap())
            .unwrap();
        db.execute_batch(
            "INSERT INTO feeds (id, public_url, created_at) VALUES (1, 'https://example.com/a.xml', 0);
             INSERT INTO library_files (id, feed_id, path, size, sha256, media_type, local, added_at)
             VALUES (1, 1, 'a.mp3', 10, 'abc', 'audio/mpeg', 1, 5),
                    (2, 1, 'b.mp3', 10, 'def', 'audio/mpeg', 1, 6);",
        )
        .unwrap();
        drop(db);

        let time = now();
        let collection = Collection::open(&path).unwrap();
        let files = collection.library_files(FeedId(1)).unwrap();
        assert_eq!(
            files.len(),
            2,
            "ADR 0105 item 8: the migration keeps each row"
        );
        for file in files {
            assert!(
                file.imported_at >= time - 5 && file.imported_at <= now() + 5,
                "ADR 0105 item 8: a row from before the migration gets the time of the migration"
            );
            assert_eq!(file.removed_at, None);
        }
    }

    #[test]
    fn a_database_at_schema_version_5_opens_at_the_current_version_and_keeps_its_library_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = Connection::open(&path).unwrap();
        for migration in &MIGRATIONS[..5] {
            db.execute_batch(migration).unwrap();
        }
        db.pragma_update(None, "user_version", 5).unwrap();
        db.execute_batch(
            "INSERT INTO connectors VALUES (1, 'local', 'Home', '{}', NULL, 0);
               INSERT INTO feeds (id, public_url, created_at) VALUES (1, 'https://example.com/a.xml', 0);
               INSERT INTO library_files VALUES (1, 1, 'a.mp3', 10, 'abc', 'audio/mpeg', 1, NULL, NULL, 5);
               INSERT INTO draft_operations VALUES (1, 1, 'copy', 1, 1, 'a.mp3', 0);",
        )
        .unwrap();
        drop(db);

        let collection = Collection::open(&path).unwrap();
        assert_eq!(collection.schema_version().unwrap(), SCHEMA_VERSION);
        let files = collection.library_files(FeedId(1)).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].sha256.as_deref(), Some("abc"));
        assert_eq!(files[0].stamp, None);
        assert_eq!(files[0].added_at, 5);
        // The drop of the old table does not remove the draft operations.
        assert_eq!(collection.draft_operations(FeedId(1)).unwrap().len(), 1);
        // A remote file has no SHA-256 and no stamp.
        let remote = collection
            .add_library_file(
                FeedId(1),
                "b.mp3",
                0,
                None,
                "audio/mpeg",
                false,
                Some("https://x/b.mp3"),
                None,
            )
            .unwrap();
        assert_eq!(collection.library_file(remote).unwrap().sha256, None);
        for table in ["downloads", "draft_feed_moves", "feed_pointers"] {
            let count: i64 = collection
                .db()
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                .unwrap();
            assert_eq!(count, 0);
        }
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
    fn the_confirm_migration_keeps_each_draft_operation_and_its_id() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = Connection::open(&path).unwrap();
        let previous = RELATIVE_LIBRARY_MIGRATION - 1;
        for migration in &MIGRATIONS[..previous] {
            db.execute_batch(migration).unwrap();
        }
        db.pragma_update(None, "user_version", i64::try_from(previous).unwrap())
            .unwrap();
        db.execute_batch(
            "INSERT INTO connectors VALUES (1, 'local', 'Home', '{}', NULL, 0);
               INSERT INTO feeds (id, public_url, created_at) VALUES (1, 'https://example.com/a.xml', 0);
               INSERT INTO library_files VALUES (1, 1, 'a.mp3', 10, 'abc', 'audio/mpeg', 1, NULL, NULL, 5, NULL);
               INSERT INTO draft_operations VALUES (7, 1, 'copy', 1, 1, 'a.mp3', 3);
               INSERT INTO draft_operations VALUES (9, 1, 'delete', NULL, 1, 'b.mp3', 4);",
        )
        .unwrap();
        drop(db);

        let collection = Collection::open(&path).unwrap();
        let operations = collection.draft_operations(FeedId(1)).unwrap();
        assert_eq!(operations.len(), 2);
        assert_eq!(operations[0].id, DraftOperationId(7));
        assert_eq!(operations[0].kind, DraftOperationKind::Copy);
        assert_eq!(operations[0].library_file_id, Some(LibraryFileId(1)));
        assert_eq!(operations[0].added_at, 3);
        assert_eq!(operations[1].id, DraftOperationId(9));
        assert_eq!(operations[1].kind, DraftOperationKind::Delete);
        // The new table accepts the confirm kind.
        let confirm = collection
            .add_draft_operation(
                FeedId(1),
                DraftOperationKind::Confirm,
                None,
                ConnectorId(1),
                "a.mp3",
            )
            .unwrap();
        assert!(confirm.0 > 9);
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
        assert!(!collection.has_draft_files_table().unwrap());
        assert_eq!(collection.draft_files(FeedId(1)).unwrap(), Vec::new());
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
        assert_eq!(collection.draft_files(feed).unwrap(), Vec::new());
    }

    #[test]
    fn save_recipient_needs_an_address_and_keeps_a_saved_wallet_as_it_is() {
        let collection = Collection::open_in_memory().unwrap();
        let save = |recipient: &Recipient| crate::people::save_recipient(&collection, recipient);
        let error = save(&Recipient::default()).unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Invalid);
        let spaces = Recipient {
            address: Some("   ".to_owned()),
            ..Recipient::default()
        };
        let error = save(&spaces).unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::Invalid);

        let recipient = Recipient {
            address: Some("x".to_owned()),
            name: Some("Old".to_owned()),
            ..Recipient::default()
        };
        let id = save(&recipient).unwrap();
        let other_name = Recipient {
            name: Some("New".to_owned()),
            ..recipient
        };
        assert_eq!(save(&other_name).unwrap(), id);
        let known = collection.known_recipients().unwrap();
        assert_eq!(known.len(), 1);
        assert_eq!(known[0].name.as_deref(), Some("Old"));
    }

    #[test]
    fn a_database_at_the_previous_schema_version_gets_the_kept_blobs_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = Connection::open(&path).unwrap();
        let previous = MIGRATIONS.len() - 1;
        for migration in &MIGRATIONS[..previous] {
            db.execute_batch(migration).unwrap();
        }
        db.pragma_update(None, "user_version", i64::try_from(previous).unwrap())
            .unwrap();
        drop(db);

        let mut collection = Collection::open(&path).unwrap();
        assert_eq!(collection.schema_version().unwrap(), SCHEMA_VERSION);
        let id = collection
            .add_connector("local", "Home", "{}", None)
            .unwrap();
        assert_eq!(collection.kept_blobs(id).unwrap().len(), 0);
        let blob = |path: &str| KeptBlob {
            path: path.to_owned(),
            sha256: "ab".repeat(32),
            size: 5,
        };
        collection
            .replace_kept_blobs(id, &[blob("b.mp3"), blob("a.mp3")])
            .unwrap();
        let paths: Vec<String> = collection
            .kept_blobs(id)
            .unwrap()
            .into_iter()
            .map(|b| b.path)
            .collect();
        assert_eq!(paths, ["a.mp3", "b.mp3"]);
        collection.replace_kept_blobs(id, &[blob("c.mp3")]).unwrap();
        assert_eq!(collection.kept_blobs(id).unwrap(), [blob("c.mp3")]);
    }

    #[test]
    fn a_database_at_the_previous_schema_version_gets_the_signing_plans_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = Connection::open(&path).unwrap();
        let previous = MIGRATIONS.len() - 1;
        for migration in &MIGRATIONS[..previous] {
            db.execute_batch(migration).unwrap();
        }
        db.pragma_update(None, "user_version", i64::try_from(previous).unwrap())
            .unwrap();
        drop(db);

        let mut collection = Collection::open(&path).unwrap();
        assert_eq!(collection.schema_version().unwrap(), SCHEMA_VERSION);
        let feed = collection.add_feed(None, None).unwrap();
        assert_eq!(collection.signing_plan(feed).unwrap(), None);
        collection.put_signing_plan(feed, "one", 100).unwrap();
        assert_eq!(
            collection.signing_plan(feed).unwrap(),
            Some(("one".to_owned(), 100))
        );
        collection.put_signing_plan(feed, "two", 200).unwrap();
        assert_eq!(
            collection.signing_plan(feed).unwrap(),
            Some(("two".to_owned(), 200))
        );
        collection.remove_signing_plan(feed).unwrap();
        assert_eq!(collection.signing_plan(feed).unwrap(), None);
    }

    #[test]
    fn a_database_at_the_previous_schema_version_gets_the_connected_apps_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = Connection::open(&path).unwrap();
        for migration in &MIGRATIONS[..APP_KEYS_MIGRATION] {
            db.execute_batch(migration).unwrap();
        }
        db.pragma_update(
            None,
            "user_version",
            i64::try_from(APP_KEYS_MIGRATION).unwrap(),
        )
        .unwrap();
        drop(db);

        let collection = Collection::open(&path).unwrap();
        assert_eq!(collection.schema_version().unwrap(), SCHEMA_VERSION);
        assert_eq!(crate::apps::apps(&collection).unwrap().len(), 0);
        let (app, code) = crate::apps::start_pairing(&collection, "Tool").unwrap();
        assert_eq!(app.state, crate::apps::AppState::Waiting);
        assert!(!code.is_empty());
    }

    #[test]
    fn the_app_keys_migration_has_its_index() {
        assert!(
            MIGRATIONS[APP_KEYS_MIGRATION].contains("connected_apps_new"),
            "ADR 0110 items 37 and 38: APP_KEYS_MIGRATION must be the index of its entry in MIGRATIONS"
        );
    }

    #[test]
    fn the_usual_group_migration_gives_each_person_an_empty_group() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = Connection::open(&path).unwrap();
        for migration in &MIGRATIONS[..=APP_KEYS_MIGRATION] {
            db.execute_batch(migration).unwrap();
        }
        db.pragma_update(
            None,
            "user_version",
            i64::try_from(APP_KEYS_MIGRATION + 1).unwrap(),
        )
        .unwrap();
        db.execute(
            "INSERT INTO people (name, usual_role, img, href, created_at) VALUES ('Ann', 'Vocalist', '', '', 0)",
            [],
        )
        .unwrap();
        drop(db);

        let collection = Collection::open(&path).unwrap();
        assert_eq!(collection.schema_version().unwrap(), SCHEMA_VERSION);
        let people = crate::people::people(&collection).unwrap();
        assert_eq!(
            people[0].usual_group, "",
            "ADR 0112 item 7: a person from before must get an empty usual group. Check the migration after APP_KEYS_MIGRATION."
        );
        assert_eq!(people[0].usual_role, "Vocalist");
    }

    /// Make a collection at the schema before the migration of the app keys,
    /// with a token app, an open grant and a grant vault file with two
    /// wrappers.
    fn before_app_keys(dir: &Path) -> PathBuf {
        let path = dir.join("collection.sqlite");
        let db = Connection::open(&path).unwrap();
        for migration in &MIGRATIONS[..APP_KEYS_MIGRATION] {
            db.execute_batch(migration).unwrap();
        }
        db.pragma_update(
            None,
            "user_version",
            i64::try_from(APP_KEYS_MIGRATION).unwrap(),
        )
        .unwrap();
        db.execute_batch(
            "INSERT INTO feeds (id, public_url, created_at) VALUES (1, 'https://example.com/a.xml', 0);
             INSERT INTO connected_apps (id, name, token_hash, created_at, app_pubkey)
             VALUES (1, 'Old', 'hash-one', 5, 'aa'), (2, 'Older', 'hash-two', 6, NULL);
             INSERT INTO show_grants (id, feed_id, item_guid, created_at) VALUES (1, 1, 'show', 7);
             INSERT INTO draft_grants (feed_id, item_guid) VALUES (1, 'show');
             INSERT INTO grant_writes (grant_id, app_name, change, written_at) VALUES (1, 'Old', 'go_live', 8);",
        )
        .unwrap();
        path
    }

    #[test]
    fn the_app_keys_migration_makes_a_token_app_without_a_key_and_ends_its_grants() {
        let dir = tempfile::tempdir().unwrap();
        let path = before_app_keys(dir.path());
        let grants = dir.path().join("grants");
        std::fs::create_dir_all(&grants).unwrap();
        let mut vault = crate::vault::OpenVault::create_with_no_wrapper().unwrap();
        vault.put("host", b"secret").unwrap();
        let mut file = vault.file().clone();
        file.set_app_wrapper(1, "sender", "ciphertext");
        file.set_app_wrapper(2, "sender", "ciphertext");
        file.save(&grants.join("1.json")).unwrap();

        let collection = Collection::open(&path).unwrap();
        assert_eq!(collection.schema_version().unwrap(), SCHEMA_VERSION);
        let apps = crate::apps::apps(&collection).unwrap();
        assert_eq!(apps.len(), 2);
        for app in &apps {
            assert_eq!(
                app.state,
                crate::apps::AppState::Token,
                "ADR 0110 item 37: an app from before must get the state token"
            );
            assert_eq!(
                app.pubkey, None,
                "ADR 0110 item 38: the migration must clear the public key of a token app"
            );
        }
        assert_eq!(apps[0].name, "Old");
        assert_eq!(apps[0].created_at, 5);
        // The old token hash does not remain in the table.
        let columns: Vec<String> = collection
            .db()
            .prepare("SELECT name FROM pragma_table_info('connected_apps')")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(
            !columns
                .iter()
                .any(|c| c.contains("token") || c == "app_pubkey")
        );
        // No token app can be found by a key, and the grants have ended.
        assert_eq!(
            crate::apps::app_for_pubkey(&collection, "aa").unwrap(),
            None
        );
        let grant = crate::grant::grant_for(&collection, FeedId(1), "show")
            .unwrap()
            .unwrap();
        assert!(
            grant.ended_at.is_some(),
            "ADR 0110 item 38: the migration must end each grant from before"
        );
        assert_eq!(grant.app_id, None);
        assert_eq!(
            collection.draft_grants(FeedId(1)).unwrap(),
            Vec::<String>::new()
        );
        // The list of granted writes keeps its entry.
        let writes = crate::grant::writes(&collection, grant.id).unwrap();
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].cause, None);
        // The wrappers went out of the grant vault file. The records stay.
        let file = crate::vault::VaultFile::load(&grants.join("1.json")).unwrap();
        assert_eq!(
            file.app_ids(),
            Vec::<i64>::new(),
            "ADR 0110 item 37: the migration must remove each wrapper from each grant vault file"
        );
        // The old key still opens the file: only the wrapper changed.
        assert!(file.open_with_key(&vault.key_for_signer()).is_ok());
        // The migration does not run a second time.
        drop(collection);
        let collection = Collection::open(&path).unwrap();
        assert_eq!(crate::apps::apps(&collection).unwrap().len(), 2);
    }

    #[test]
    fn a_database_at_the_previous_schema_version_gets_the_grant_tables() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("collection.sqlite");
        let db = Connection::open(&path).unwrap();
        let previous = MIGRATIONS.len() - 1;
        for migration in &MIGRATIONS[..previous] {
            db.execute_batch(migration).unwrap();
        }
        db.pragma_update(None, "user_version", i64::try_from(previous).unwrap())
            .unwrap();
        drop(db);

        let mut collection = Collection::open(&path).unwrap();
        assert_eq!(collection.schema_version().unwrap(), SCHEMA_VERSION);
        let feed = collection.add_feed(None, None).unwrap();
        collection.add_draft_grant(feed, "b", 1).unwrap();
        collection.add_draft_grant(feed, "a", 1).unwrap();
        collection.add_draft_grant(feed, "a", 2).unwrap();
        assert_eq!(collection.draft_grant_app(feed, "a").unwrap(), Some(2));
        assert_eq!(collection.draft_grant_app(feed, "none").unwrap(), None);
        assert_eq!(collection.draft_grants(feed).unwrap(), ["a", "b"]);
        collection.remove_draft_grant(feed, "b").unwrap();
        assert_eq!(collection.draft_grants(feed).unwrap(), ["a"]);
        // The grants of the draft go out with the draft.
        collection.delete_draft(feed).unwrap();
        assert_eq!(collection.draft_grants(feed).unwrap(), Vec::<String>::new());

        crate::grant::record(&collection, feed, "a", 1, "wss://live.example.com", 1).unwrap();
        let grant = crate::grant::grant_for(&collection, feed, "a")
            .unwrap()
            .unwrap();
        crate::grant::add_write(
            &collection,
            grant.id,
            "Tool",
            &crate::grant::GrantChange::GoLive,
            2,
        )
        .unwrap();
        collection.delete_feed(feed).unwrap();
        assert_eq!(crate::grant::grant(&collection, grant.id).unwrap(), None);
        assert_eq!(
            crate::grant::writes(&collection, grant.id).unwrap(),
            Vec::new()
        );
    }

    #[test]
    fn an_upload_row_keeps_its_parts_in_sequence_and_without_a_double() {
        let collection = Collection::open_in_memory().unwrap();
        let row = UploadRow {
            id: "ab".repeat(16),
            feed_id: FeedId(1),
            name: "a.mp3".to_owned(),
            size: 20,
            replace: None,
            received: Vec::new(),
            updated_at: 100,
        };
        collection.add_upload(&row).unwrap();
        for part in [2, 0, 2, 1, 10] {
            collection.record_upload_part(&row.id, part, 200).unwrap();
        }
        let found = collection.upload(&row.id).unwrap();
        assert_eq!(found.received, vec![0, 1, 2, 10]);
        assert_eq!(found.updated_at, 200);
        assert_eq!(collection.uploads_of_feed(FeedId(1)).unwrap().len(), 1);
        assert_eq!(collection.uploads_older_than(200).unwrap().len(), 0);
        assert_eq!(collection.uploads_older_than(201).unwrap().len(), 1);
        assert!(collection.remove_upload(&row.id).unwrap());
        assert!(!collection.remove_upload(&row.id).unwrap());
        let error = collection.upload(&row.id).unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::NotFound);
        let error = collection.record_upload_part(&row.id, 0, 1).unwrap_err();
        assert_eq!(error.kind(), CollectionErrorKind::NotFound);
    }
}

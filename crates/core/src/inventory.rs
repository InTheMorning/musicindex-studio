//! The host inventory: what Studio saw on each host, and where each
//! library file goes on its host (ADR 0036).
//!
//! A scan keeps `host_files` in step with the list of files of one host.
//! The other functions find a copy of a library file on a host, and plan
//! the address of a library file on its host (ADR 0035 items 12, 13 and
//! 16).

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::collection::{Collection, CollectionError, ConnectorId, FeedRow, LibraryFile};
use crate::connector::{Connector, ConnectorError, Stamp};
use crate::digest::sha256_hex;
use crate::pipeline::release_path;

/// The kind of an [`InventoryError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InventoryErrorKind {
    /// The set of hosts has no connector for this host.
    UnknownHost,
    /// The host needs a secret from the vault, and the vault is locked
    /// (ADR 0017).
    VaultLocked,
    /// The file and the feed give no host at all.
    NoHost,
    /// The connector failed.
    Connector,
    /// The collection database failed.
    Collection,
}

/// An error from an inventory function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryError {
    kind: InventoryErrorKind,
    message: String,
}

impl InventoryError {
    fn new(kind: InventoryErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// The kind of the error.
    #[must_use]
    pub fn kind(&self) -> InventoryErrorKind {
        self.kind
    }
}

impl fmt::Display for InventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for InventoryError {}

impl From<CollectionError> for InventoryError {
    fn from(error: CollectionError) -> Self {
        Self::new(InventoryErrorKind::Collection, error.to_string())
    }
}

impl From<ConnectorError> for InventoryError {
    fn from(error: ConnectorError) -> Self {
        Self::new(InventoryErrorKind::Connector, error.to_string())
    }
}

/// The connector of each host that an inventory function can use, keyed
/// by its ID in the collection (ADR 0036 item 6).
///
/// The API makes this set with [`crate::connector::from_row`] and the
/// vault. It leaves out a host that needs a secret while the vault is
/// locked, and marks it instead, so that [`Hosts::get`] of that host
/// gives [`InventoryErrorKind::VaultLocked`] (ADR 0017). Tests make the
/// set from local folders.
#[derive(Default)]
pub struct Hosts {
    by_id: HashMap<ConnectorId, Box<dyn Connector>>,
    locked: HashSet<ConnectorId>,
}

impl fmt::Debug for Hosts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Hosts").finish_non_exhaustive()
    }
}

impl Hosts {
    /// A set with no host.
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            locked: HashSet::new(),
        }
    }

    /// Add the connector of a host to the set.
    pub fn insert(&mut self, id: ConnectorId, connector: Box<dyn Connector>) {
        self.by_id.insert(id, connector);
    }

    /// Mark a host as needing the vault, while the vault is locked. A
    /// later [`Hosts::get`] of this host gives
    /// [`InventoryErrorKind::VaultLocked`] instead of
    /// [`InventoryErrorKind::UnknownHost`] (ADR 0017).
    pub fn insert_locked(&mut self, id: ConnectorId) {
        self.locked.insert(id);
    }

    /// The connector of a host.
    ///
    /// # Errors
    ///
    /// Gives [`InventoryErrorKind::VaultLocked`] when the host needs the
    /// vault and the vault is locked, and
    /// [`InventoryErrorKind::UnknownHost`] when the set has no connector
    /// for this host at all.
    pub fn get(&self, id: ConnectorId) -> Result<&dyn Connector, InventoryError> {
        if let Some(connector) = self.by_id.get(&id) {
            return Ok(connector.as_ref());
        }
        if self.locked.contains(&id) {
            return Err(InventoryError::new(
                InventoryErrorKind::VaultLocked,
                format!("host {} needs the vault, and the vault is locked", id.0),
            ));
        }
        Err(InventoryError::new(
            InventoryErrorKind::UnknownHost,
            format!("the set of hosts has no connector for host {}", id.0),
        ))
    }

    /// The ID of each host of the set that the vault did not lock out.
    pub fn ids(&self) -> impl Iterator<Item = ConnectorId> + '_ {
        self.by_id.keys().copied()
    }
}

/// The result of one scan of a host (ADR 0036 item 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
pub struct ScanResult {
    /// The number of files that are new since the last scan.
    pub new: usize,
    /// The number of files whose version changed since the last scan.
    pub changed: usize,
    /// The number of files that are gone since the last scan.
    pub gone: usize,
}

/// List a host, and keep its row of `host_files` in step with the list
/// (ADR 0036 item 6).
///
/// The function records the size, the version and the time of each file
/// that the list gives. A file keeps its recorded SHA-256 when its
/// version did not change, and loses it otherwise. The function removes
/// the row of each file that the list no longer has.
///
/// # Errors
///
/// Gives [`InventoryErrorKind::UnknownHost`] when the set has no
/// connector for this host, or an [`InventoryError`] when the list fails
/// or the collection database fails.
pub fn scan(
    collection: &Collection,
    hosts: &Hosts,
    connector_id: ConnectorId,
) -> Result<ScanResult, InventoryError> {
    let connector = hosts.get(connector_id)?;
    let list = connector.list()?;
    let before: HashMap<String, Stamp> = collection
        .host_files(connector_id)?
        .into_iter()
        .map(|file| (file.path, file.stamp))
        .collect();

    // The list gives the size and the stamp of each file, so the scan
    // reads no file (ADR 0036 item 6).
    let mut result = ScanResult::default();
    let mut seen: HashSet<String> = HashSet::new();
    for file in &list.files {
        seen.insert(file.path.clone());
        collection.put_host_file(connector_id, &file.path, file.size, &file.stamp)?;
        match before.get(&file.path) {
            None => result.new += 1,
            Some(old) if old != &file.stamp => result.changed += 1,
            Some(_) => {}
        }
    }
    for path in before.keys() {
        if !seen.contains(path) {
            collection.remove_host_file(connector_id, path)?;
            result.gone += 1;
        }
    }
    Ok(result)
}

/// The recorded SHA-256 of a host file, or `None` when the file is gone
/// from the host (ADR 0036 item 6).
///
/// The function gives the recorded SHA-256 of the file, when the
/// inventory has one. Otherwise it reads the file, records its SHA-256,
/// and gives it.
///
/// # Errors
///
/// Gives [`InventoryErrorKind::UnknownHost`] when the set has no
/// connector for this host, or an [`InventoryError`] when the read fails
/// or the collection database fails.
pub fn host_sha(
    collection: &Collection,
    hosts: &Hosts,
    connector_id: ConnectorId,
    path: &str,
) -> Result<Option<String>, InventoryError> {
    if let Some(sha256) = collection
        .host_file(connector_id, path)?
        .and_then(|file| file.sha256)
    {
        return Ok(Some(sha256));
    }
    let connector = hosts.get(connector_id)?;
    let Some(read) = connector.read(path)? else {
        return Ok(None);
    };
    // The stamp after the read: when the file changed during the read, the
    // stamp is new, and the next scan clears the SHA-256 of the row.
    let Some(listed) = connector.stat(path)? else {
        return Ok(None);
    };
    collection.put_host_file(connector_id, path, listed.size, &listed.stamp)?;
    let sha256 = sha256_hex(&[&read.bytes]);
    collection.set_host_file_sha(connector_id, path, &sha256)?;
    Ok(Some(sha256))
}

/// A copy of a library file on a host, with the same size and the same
/// SHA-256 (ADR 0036 item 7 and item 9).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Copy {
    /// The host.
    pub connector_id: ConnectorId,
    /// The path of the file on the host.
    pub path: String,
    /// The public address of the file, when the host gives one.
    pub url: Option<String>,
}

/// Each host file with the same size and the same SHA-256 as a library
/// file (ADR 0036 item 7 and item 9).
///
/// The function reads a SHA-256 only for a host file with the same size
/// as `file`. A file with no SHA-256 is a remote file. Its copies are the
/// host files whose public address is its `source_url` (ADR 0038 item 5).
///
/// # Errors
///
/// Gives an [`InventoryError`] when a list or a read fails, or the
/// collection database fails.
pub fn copies(
    collection: &Collection,
    hosts: &Hosts,
    file: &LibraryFile,
) -> Result<Vec<Copy>, InventoryError> {
    let mut found = Vec::new();
    for connector_id in hosts.ids() {
        for host_file in collection.host_files(connector_id)? {
            let connector = hosts.get(connector_id)?;
            let url = connector.public_url(&host_file.path);
            match &file.sha256 {
                Some(wanted) => {
                    if host_file.size != file.size {
                        continue;
                    }
                    let sha256 = match &host_file.sha256 {
                        Some(sha256) => Some(sha256.clone()),
                        None => host_sha(collection, hosts, connector_id, &host_file.path)?,
                    };
                    if sha256.as_deref() != Some(wanted.as_str()) {
                        continue;
                    }
                }
                None => {
                    if url.is_none() || url != file.source_url {
                        continue;
                    }
                }
            }
            found.push(Copy {
                connector_id,
                url,
                path: host_file.path,
            });
        }
    }
    Ok(found)
}

/// The host of a library file: its own host, or the default host of its
/// release, or the host of the feed XML (ADR 0036 item 8).
#[must_use]
pub fn host_of(feed: &FeedRow, file: &LibraryFile) -> Option<ConnectorId> {
    file.connector_id
        .or(feed.files_connector_id)
        .or(feed.connector_id)
}

/// The path, the address and the host where Studio plans to put a
/// library file (ADR 0036 item 9, ADR 0035 items 13 and 16).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PlannedPath {
    /// The host.
    pub connector_id: ConnectorId,
    /// The planned path on the host.
    pub path: String,
    /// The public address of the planned path.
    pub url: Option<String>,
    /// True when a file with the same SHA-256 as `file` is already at the
    /// path.
    pub on_host: bool,
}

/// True when the host has a file at `path` with this size and this
/// SHA-256 (ADR 0036 item 9).
fn file_matches(
    collection: &Collection,
    hosts: &Hosts,
    connector_id: ConnectorId,
    path: &str,
    size: u64,
    sha256: &str,
) -> Result<bool, InventoryError> {
    let Some(existing) = collection.host_file(connector_id, path)? else {
        return Ok(false);
    };
    if existing.size != size {
        return Ok(false);
    }
    let found_sha256 = match existing.sha256 {
        Some(found) => Some(found),
        None => host_sha(collection, hosts, connector_id, path)?,
    };
    Ok(found_sha256.as_deref() == Some(sha256))
}

/// Give a path the hash name: the first 8 hexadecimal characters of a
/// SHA-256, before the extension (ADR 0035 item 16).
pub(crate) fn hash_name(path: &str, sha256: &str) -> String {
    let short = &sha256[..sha256.len().min(8)];
    let (parent, file) = path.rsplit_once('/').map_or(("", path), |(p, f)| (p, f));
    let named = match file.rsplit_once('.') {
        Some((stem, extension)) => format!("{stem}.{short}.{extension}"),
        None => format!("{file}.{short}"),
    };
    if parent.is_empty() {
        named
    } else {
        format!("{parent}/{named}")
    }
}

/// The path, the host and the address where Studio plans to put a
/// library file (ADR 0036 item 9, ADR 0035 items 13 and 16).
///
/// The clean path is the folder of the release, then the path of the
/// file in the release folder (ADR 0035 item 13). The function gives the
/// clean path when the host has no file there, or a file with the same
/// SHA-256 as `file`. When the host has a different file at the clean
/// path, the function gives the hash name next to it instead.
///
/// # Errors
///
/// Gives [`InventoryErrorKind::NoHost`] when `file` and `feed` give no
/// host, [`InventoryErrorKind::UnknownHost`] when the set has no
/// connector for that host, or an [`InventoryError`] when a read fails or
/// the collection database fails.
pub fn planned_path(
    collection: &Collection,
    hosts: &Hosts,
    feed: &FeedRow,
    file: &LibraryFile,
) -> Result<PlannedPath, InventoryError> {
    let connector_id = host_of(feed, file).ok_or_else(|| {
        InventoryError::new(
            InventoryErrorKind::NoHost,
            "the library file and its feed give no host",
        )
    })?;
    let feed_path = feed.path.as_deref().unwrap_or("");
    let clean_path = release_path(feed_path, &file.path);

    // A remote file has no SHA-256. Studio reads no host file for it, and
    // it plans the clean path (ADR 0038 item 1).
    let Some(sha256) = file.sha256.as_deref() else {
        let url = hosts.get(connector_id)?.public_url(&clean_path);
        return Ok(PlannedPath {
            connector_id,
            path: clean_path,
            url,
            on_host: false,
        });
    };

    let clean_matches = file_matches(
        collection,
        hosts,
        connector_id,
        &clean_path,
        file.size,
        sha256,
    )?;
    let (path, on_host) = if clean_matches {
        (clean_path, true)
    } else if collection.host_file(connector_id, &clean_path)?.is_some() {
        // A different file is at the clean path. Studio names this file
        // from its own hash instead (ADR 0035 item 16).
        let hash_path = hash_name(&clean_path, sha256);
        let on_host = file_matches(
            collection,
            hosts,
            connector_id,
            &hash_path,
            file.size,
            sha256,
        )?;
        (hash_path, on_host)
    } else {
        (clean_path, false)
    };

    let connector = hosts.get(connector_id)?;
    let url = connector.public_url(&path);
    Ok(PlannedPath {
        connector_id,
        path,
        url,
        on_host,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::{Expected, LocalFolder};

    fn open_connector(dir: &std::path::Path, base_url: &str) -> LocalFolder {
        LocalFolder::new(dir, Some(base_url.to_owned()))
    }

    /// A connector that refuses each read, so a test can show that a scan
    /// reads no file (ADR 0036 item 6).
    struct NoRead(LocalFolder);

    impl Connector for NoRead {
        fn read(&self, path: &str) -> Result<Option<crate::connector::FileRead>, ConnectorError> {
            panic!("the scan read {path}");
        }
        fn write(
            &self,
            path: &str,
            bytes: &[u8],
            expected: &Expected,
        ) -> Result<crate::connector::Version, ConnectorError> {
            self.0.write(path, bytes, expected)
        }
        fn public_url(&self, path: &str) -> Option<String> {
            self.0.public_url(path)
        }
        fn list(&self) -> Result<crate::connector::FileList, ConnectorError> {
            self.0.list()
        }
        fn stat(&self, path: &str) -> Result<Option<crate::connector::ListedFile>, ConnectorError> {
            self.0.stat(path)
        }
        fn delete(&self, path: &str, expected: &Stamp) -> Result<(), ConnectorError> {
            self.0.delete(path, expected)
        }
    }

    #[test]
    fn hosts_get_gives_vault_locked_for_a_locked_host_and_unknown_host_for_the_rest() {
        let mut hosts = Hosts::new();
        let locked = ConnectorId(1);
        let unknown = ConnectorId(2);
        hosts.insert_locked(locked);

        let error = hosts.get(locked).err().unwrap();
        assert_eq!(error.kind(), InventoryErrorKind::VaultLocked);

        let error = hosts.get(unknown).err().unwrap();
        assert_eq!(error.kind(), InventoryErrorKind::UnknownHost);

        // A locked host that the caller inserts later is reachable, like
        // an ordinary host.
        let dir = tempfile::tempdir().unwrap();
        hosts.insert(
            locked,
            Box::new(open_connector(dir.path(), "https://example.com")),
        );
        assert!(hosts.get(locked).is_ok());
    }

    #[test]
    fn a_scan_reads_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let folder = open_connector(dir.path(), "https://example.com");
        folder.write("a.mp3", b"one", &Expected::Absent).unwrap();
        let collection = Collection::open_in_memory().unwrap();
        let connector_id = collection
            .add_connector("local", "Home", "{}", None)
            .unwrap();
        let mut hosts = Hosts::new();
        hosts.insert(connector_id, Box::new(NoRead(folder)));
        let result = scan(&collection, &hosts, connector_id).unwrap();
        assert_eq!(result.new, 1);
        assert_eq!(collection.host_files(connector_id).unwrap()[0].size, 3);
    }

    #[test]
    fn a_scan_records_new_files_then_reports_a_change_and_a_removal() {
        let dir = tempfile::tempdir().unwrap();
        let folder = open_connector(dir.path(), "https://example.com");
        let b_v1 = folder.write("b.mp3", b"two", &Expected::Absent).unwrap();
        folder.write("a.mp3", b"one", &Expected::Absent).unwrap();
        folder.write("c.mp3", b"three", &Expected::Absent).unwrap();

        let collection = Collection::open_in_memory().unwrap();
        let connector_id = collection
            .add_connector("local", "Home", "{}", None)
            .unwrap();
        let mut hosts = Hosts::new();
        hosts.insert(connector_id, Box::new(folder));

        let result = scan(&collection, &hosts, connector_id).unwrap();
        assert_eq!(
            result,
            ScanResult {
                new: 3,
                changed: 0,
                gone: 0
            }
        );
        assert_eq!(collection.host_files(connector_id).unwrap().len(), 3);

        // Give each row a recorded SHA-256, as a later call to
        // `host_sha` or `copies` would.
        for path in ["a.mp3", "b.mp3", "c.mp3"] {
            host_sha(&collection, &hosts, connector_id, path).unwrap();
        }

        let connector = hosts.get(connector_id).unwrap();
        connector
            .write("b.mp3", b"two, changed", &Expected::Version(b_v1))
            .unwrap();
        let c_stamp = connector.stat("c.mp3").unwrap().unwrap().stamp;
        connector.delete("c.mp3", &c_stamp).unwrap();

        let result = scan(&collection, &hosts, connector_id).unwrap();
        assert_eq!(
            result,
            ScanResult {
                new: 0,
                changed: 1,
                gone: 1
            }
        );
        let files = collection.host_files(connector_id).unwrap();
        assert_eq!(files.len(), 2);
        let a = files.iter().find(|f| f.path == "a.mp3").unwrap();
        let b = files.iter().find(|f| f.path == "b.mp3").unwrap();
        assert!(
            a.sha256.is_some(),
            "a file with no version change keeps its SHA-256"
        );
        assert!(
            b.sha256.is_none(),
            "a file with a version change has no SHA-256"
        );
    }

    #[test]
    fn copies_finds_a_same_size_same_sha_file_on_a_different_host_and_skips_a_decoy() {
        let dir_1 = tempfile::tempdir().unwrap();
        let dir_2 = tempfile::tempdir().unwrap();
        let host_1_folder = open_connector(dir_1.path(), "https://one.example");
        let host_2_folder = open_connector(dir_2.path(), "https://two.example");
        // A decoy on the first host: the same size, different bytes.
        host_1_folder
            .write("decoy.mp3", b"noise!!", &Expected::Absent)
            .unwrap();
        // The real copy, on the second host.
        host_2_folder
            .write("album/01 Song.mp3", b"music!!", &Expected::Absent)
            .unwrap();

        let mut collection = Collection::open_in_memory().unwrap();
        let host_1 = collection
            .add_connector("local", "One", "{}", None)
            .unwrap();
        let host_2 = collection
            .add_connector("local", "Two", "{}", None)
            .unwrap();
        let mut hosts = Hosts::new();
        hosts.insert(host_1, Box::new(host_1_folder));
        hosts.insert(host_2, Box::new(host_2_folder));
        scan(&collection, &hosts, host_1).unwrap();
        scan(&collection, &hosts, host_2).unwrap();

        let feed = collection.add_feed(None, None).unwrap();
        let sha256 = sha256_hex(&[b"music!!"]);
        let library_file_id = collection
            .add_library_file(
                feed,
                "01 Song.mp3",
                7,
                Some(&sha256),
                "audio/mpeg",
                true,
                None,
                None,
            )
            .unwrap();
        let library_file = collection.library_file(library_file_id).unwrap();

        let found = copies(&collection, &hosts, &library_file).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].connector_id, host_2);
        assert_eq!(found[0].path, "album/01 Song.mp3");
        assert_eq!(
            found[0].url.as_deref(),
            Some("https://two.example/album/01%20Song.mp3")
        );
    }

    #[test]
    fn host_of_gives_the_file_host_then_the_release_host_then_the_feed_host() {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed_connector = collection
            .add_connector("local", "Feed", "{}", None)
            .unwrap();
        let files_connector = collection
            .add_connector("local", "Files", "{}", None)
            .unwrap();
        let own_connector = collection
            .add_connector("local", "Own", "{}", None)
            .unwrap();

        let feed_id = collection
            .add_feed(None, Some((feed_connector, "a.xml")))
            .unwrap();
        let file_id = collection
            .add_library_file(
                feed_id,
                "a.mp3",
                1,
                Some("sha"),
                "audio/mpeg",
                true,
                None,
                None,
            )
            .unwrap();

        // No default host and no file host: Studio uses the feed's host.
        let feed = collection.feed(feed_id).unwrap();
        let file = collection.library_file(file_id).unwrap();
        assert_eq!(host_of(&feed, &file), Some(feed_connector));

        // A default host of the release wins over the feed's host.
        collection
            .set_files_connector(feed_id, Some(files_connector))
            .unwrap();
        let feed = collection.feed(feed_id).unwrap();
        assert_eq!(host_of(&feed, &file), Some(files_connector));

        // The file's own host wins over the default host.
        collection
            .set_library_file_connector(file_id, Some(own_connector))
            .unwrap();
        let file = collection.library_file(file_id).unwrap();
        assert_eq!(host_of(&feed, &file), Some(own_connector));
    }

    #[test]
    fn planned_path_gives_the_clean_path_or_the_hash_name() {
        let dir = tempfile::tempdir().unwrap();
        let folder = open_connector(dir.path(), "https://example.com");
        let mut collection = Collection::open_in_memory().unwrap();
        let connector_id = collection
            .add_connector("local", "Home", "{}", None)
            .unwrap();
        let feed_id = collection
            .add_feed(None, Some((connector_id, "album/my-album.xml")))
            .unwrap();
        let mut hosts = Hosts::new();
        hosts.insert(connector_id, Box::new(folder));

        let sha256 = sha256_hex(&[b"music!!"]);
        let library_file_id = collection
            .add_library_file(
                feed_id,
                "01 Song.mp3",
                7,
                Some(&sha256),
                "audio/mpeg",
                true,
                None,
                None,
            )
            .unwrap();
        let file = collection.library_file(library_file_id).unwrap();
        let feed = collection.feed(feed_id).unwrap();

        // An empty host: the clean path, with no file there yet.
        let planned = planned_path(&collection, &hosts, &feed, &file).unwrap();
        assert_eq!(planned.connector_id, connector_id);
        assert_eq!(planned.path, "album/my-album/01 Song.mp3");
        assert!(!planned.on_host);

        // The same file at the clean path.
        {
            let connector = hosts.get(connector_id).unwrap();
            connector
                .write("album/my-album/01 Song.mp3", b"music!!", &Expected::Absent)
                .unwrap();
        }
        scan(&collection, &hosts, connector_id).unwrap();
        let planned = planned_path(&collection, &hosts, &feed, &file).unwrap();
        assert_eq!(planned.path, "album/my-album/01 Song.mp3");
        assert!(planned.on_host);
        assert_eq!(
            planned.url.as_deref(),
            Some("https://example.com/album/my-album/01%20Song.mp3")
        );

        // A different file at the clean path: Studio uses the hash name.
        {
            let connector = hosts.get(connector_id).unwrap();
            let current = connector
                .read("album/my-album/01 Song.mp3")
                .unwrap()
                .unwrap()
                .version;
            connector
                .write(
                    "album/my-album/01 Song.mp3",
                    b"a different file",
                    &Expected::Version(current),
                )
                .unwrap();
        }
        scan(&collection, &hosts, connector_id).unwrap();
        let planned = planned_path(&collection, &hosts, &feed, &file).unwrap();
        let short = &sha256[..8];
        assert_eq!(planned.path, format!("album/my-album/01 Song.{short}.mp3"));
        assert!(!planned.on_host);
    }
}

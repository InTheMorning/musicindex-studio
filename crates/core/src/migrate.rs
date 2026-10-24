//! The move of the current collection into the library (ADR 0035 item 22,
//! ADR 0036 items 1 to 6).
//!
//! At the first start with the library, each feed with a host gets a
//! release folder, with the bytes of its hosted feed file. Each pending
//! file from before the library moves into its release folder, as a
//! library file. A read-only feed, with no host, gets a release folder
//! too, with the XML from its address.

use std::collections::HashSet;
use std::fmt;
use std::path::Path;

use crate::collection::{Collection, CollectionError, FeedId, FeedRow};
use crate::discover::MAX_FEED_BYTES;
use crate::inventory::{self, Hosts, InventoryError};
use crate::library::{self, Library};
use crate::web::Web;

/// The kind of a [`MigrateError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MigrateErrorKind {
    /// The collection database failed.
    Collection,
    /// A host scan failed, after the move of each feed.
    Inventory,
}

/// An error that stops the whole move of [`into_library`]. The error of
/// one feed does not stop the move of the other feeds: it goes into
/// [`MoveReport::errors`] instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrateError {
    kind: MigrateErrorKind,
    message: String,
}

impl MigrateError {
    fn new(kind: MigrateErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// The kind of the error.
    #[must_use]
    pub fn kind(&self) -> MigrateErrorKind {
        self.kind
    }
}

impl fmt::Display for MigrateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for MigrateError {}

impl From<CollectionError> for MigrateError {
    fn from(error: CollectionError) -> Self {
        Self::new(MigrateErrorKind::Collection, error.to_string())
    }
}

impl From<InventoryError> for MigrateError {
    fn from(error: InventoryError) -> Self {
        Self::new(MigrateErrorKind::Inventory, error.to_string())
    }
}

/// The error of one feed that [`into_library`] could not move. The move of
/// the other feeds still runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveError {
    /// The feed.
    pub feed_id: FeedId,
    /// The message.
    pub message: String,
}

/// The result of [`into_library`] (ADR 0036 items 1 to 6).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MoveReport {
    /// The number of release folders that the move made.
    pub release_folders: usize,
    /// The number of feed XML files that the move wrote.
    pub feed_files: usize,
    /// The number of pending files that the move moved into a release
    /// folder.
    pub moved_files: usize,
    /// The error of each feed that the move could not finish.
    pub errors: Vec<MoveError>,
}

/// Move the current collection into the library, at the first start with a
/// library folder (ADR 0035 item 22, ADR 0036 items 1 to 6).
///
/// For each feed with a host and a path, and no release folder yet, the
/// function:
///
/// 1. Chooses a release folder name with [`library::release_folder_name`],
///    and makes the folder.
/// 2. Reads the feed file through its connector, and writes it into the
///    release folder. A feed with no hosted file yet, a new release,
///    writes nothing.
/// 3. Sets the default host of the files of the release to the host of the
///    feed XML.
/// 4. Moves each pending file of `store_root` into the release folder,
///    with its file name, and adds a library file row for it with
///    `local` true.
/// 5. Records the folder in the collection, now that every step above
///    succeeded.
///
/// A feed with no host, a read-only feed, gets a release folder too, with
/// its XML from its address, as the last read gave it.
///
/// A feed that has a release folder already is moved already, so a second
/// call does nothing for it. A feed whose move fails keeps no release
/// folder, so a later call retries it; the chosen name stays out of reach
/// of the other feeds of this call, so a retry later does not collide with
/// a name that this call already gave out.
///
/// The function then scans each host of `hosts`, so that the inventory
/// agrees with what each host has (ADR 0036 item 6). When every feed
/// moved with no error, it also drops the table of the pending files, now
/// that the library replaces it (ADR 0036 item 5). A failed feed may
/// still have a pending file to move at a later call, so the function
/// keeps the table until then.
///
/// The function does not download an audio file or an image, and it does
/// not change a hosted file (ADR 0035 item 6).
///
/// The error of one feed goes into [`MoveReport::errors`], and the move of
/// the other feeds still runs.
///
/// # Errors
///
/// Gives a [`MigrateError`] when the collection cannot list its feeds, or
/// a host scan fails after the move of each feed.
pub fn into_library(
    collection: &Collection,
    hosts: &Hosts,
    library: &Library,
    store_root: &Path,
) -> Result<MoveReport, MigrateError> {
    let feeds = collection.feeds()?;
    let mut taken: HashSet<String> = feeds
        .iter()
        .filter_map(|feed| feed.folder.clone())
        .collect();
    let mut report = MoveReport::default();

    for feed in &feeds {
        if feed.folder.is_some() {
            // A second call does nothing for a feed that has a folder.
            continue;
        }
        if let Err(message) = move_feed(
            collection,
            hosts,
            library,
            store_root,
            feed,
            &mut taken,
            &mut report,
        ) {
            report.errors.push(MoveError {
                feed_id: feed.id,
                message,
            });
        }
    }

    for connector_id in hosts.ids() {
        inventory::scan(collection, hosts, connector_id)?;
    }
    // Drop the table only once every feed has moved: a feed whose move
    // failed may still have a row to move at the next call.
    if report.errors.is_empty() {
        collection.drop_draft_files_table()?;
    }

    Ok(report)
}

/// Move one feed into the library: steps 1 to 5 for a feed with a host, or
/// the address of a read-only feed otherwise.
fn move_feed(
    collection: &Collection,
    hosts: &Hosts,
    library: &Library,
    store_root: &Path,
    feed: &FeedRow,
    taken: &mut HashSet<String>,
    report: &mut MoveReport,
) -> Result<(), String> {
    match (feed.connector_id, feed.path.as_deref()) {
        (Some(connector_id), Some(path)) => {
            let folder = reserve_folder(library, path, taken)?;

            let connector = hosts.get(connector_id).map_err(|e| e.to_string())?;
            if let Some(read) = connector.read(path).map_err(|e| e.to_string())? {
                library
                    .write_feed_xml(&folder, file_name(path), &read.bytes)
                    .map_err(|e| e.to_string())?;
                report.feed_files += 1;
            }
            collection
                .set_files_connector(feed.id, Some(connector_id))
                .map_err(|e| e.to_string())?;
            move_pending_files(collection, library, store_root, feed.id, &folder, report)?;

            // Every step of the feed succeeded: record its folder now
            // (ADR 0036 item 4). A move that fails above keeps the feed
            // with no folder, so a later call retries it with the same
            // name, which stays reserved in `taken` for the rest of this
            // call.
            give_folder(collection, feed.id, &folder, report)
        }
        _ => move_read_only_feed(collection, library, feed, taken, report),
    }
}

/// Choose a release folder name for a feed, from `path` with
/// [`library::release_folder_name`], make the folder on disk, and keep
/// the name out of reach of the other feeds of this call. The function
/// does not record the folder in the collection: the caller does that
/// only once every other step of the feed has succeeded.
fn reserve_folder(
    library: &Library,
    path: &str,
    taken: &mut HashSet<String>,
) -> Result<String, String> {
    let folder = library::release_folder_name(path, &|name| taken.contains(name));
    taken.insert(folder.clone());
    std::fs::create_dir_all(library.root().join(&folder)).map_err(|e| e.to_string())?;
    Ok(folder)
}

/// Record a feed's release folder in the collection, and count one
/// release folder of the report.
fn give_folder(
    collection: &Collection,
    feed_id: FeedId,
    folder: &str,
    report: &mut MoveReport,
) -> Result<(), String> {
    collection
        .set_feed_folder(feed_id, folder)
        .map_err(|e| e.to_string())?;
    report.release_folders += 1;
    Ok(())
}

/// Move each pending file of a feed from `store_root` into its release
/// folder, with a library file row for each one (ADR 0036 item 4 and item
/// 5).
fn move_pending_files(
    collection: &Collection,
    library: &Library,
    store_root: &Path,
    feed_id: FeedId,
    folder: &str,
    report: &mut MoveReport,
) -> Result<(), String> {
    for pending in collection.draft_files(feed_id).map_err(|e| e.to_string())? {
        let source = store_root.join(feed_id.0.to_string()).join(&pending.sha256);
        library
            .put_file(folder, &pending.path, &source)
            .map_err(|e| e.to_string())?;
        collection
            .add_library_file(
                feed_id,
                &pending.path,
                pending.size,
                Some(&pending.sha256),
                &pending.media_type,
                true,
                None,
                None,
            )
            .map_err(|e| e.to_string())?;
        collection
            .remove_draft_file(feed_id, &pending.path)
            .map_err(|e| e.to_string())?;
        report.moved_files += 1;
    }
    Ok(())
}

/// Give a read-only feed a release folder, with its XML from its address,
/// as the last read gave it (ADR 0035 item 22, ADR 0036 item 1).
///
/// A feed with no host and no address has nothing to move, so the
/// function does nothing for it.
fn move_read_only_feed(
    collection: &Collection,
    library: &Library,
    feed: &FeedRow,
    taken: &mut HashSet<String>,
    report: &mut MoveReport,
) -> Result<(), String> {
    let Some(url) = feed.public_url.as_deref() else {
        return Ok(());
    };
    let web = Web::new();
    let limit = u64::try_from(MAX_FEED_BYTES).unwrap_or(u64::MAX);
    let file = web.get(url, limit).map_err(|e| e.to_string())?;
    // The function names the folder, writes the feed XML and records the
    // folder. The `taken` set of this call already holds each folder that
    // the collection records.
    let folder = library::give_published_folder(collection, library, feed.id, &file.bytes)
        .map_err(|e| e.to_string())?;
    taken.insert(folder);
    report.feed_files += 1;
    report.release_folders += 1;
    Ok(())
}

/// The last path segment of a feed path or a feed address, with its
/// extension, for example `my-album.xml` of `album/my-album.xml`.
fn file_name(path: &str) -> &str {
    path.rsplit_once('/').map_or(path, |(_, name)| name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::{Collection, ConnectorId, DraftFile};
    use crate::connector::{Connector, Expected, LocalFolder};
    use crate::digest::sha256_hex;
    use crate::testing::TestServer;

    fn open_connector(dir: &Path, base_url: &str) -> LocalFolder {
        LocalFolder::new(dir, Some(base_url.to_owned()))
    }

    struct Setup {
        host_dir: tempfile::TempDir,
        // No test reads `library_dir` directly, but the struct keeps it, so
        // that the temporary folder stays on disk for the life of the test.
        #[expect(dead_code)]
        library_dir: tempfile::TempDir,
        store_dir: tempfile::TempDir,
        collection: Collection,
        hosts: Hosts,
        library: Library,
        connector_id: ConnectorId,
    }

    fn setup() -> Setup {
        let host_dir = tempfile::tempdir().unwrap();
        let library_dir = tempfile::tempdir().unwrap();
        let store_dir = tempfile::tempdir().unwrap();
        let folder = open_connector(host_dir.path(), "https://music.example");
        let collection = Collection::open_in_memory().unwrap();
        let connector_id = collection
            .add_connector(LocalFolder::KIND, "Home", &folder.config(), None)
            .unwrap();
        let mut hosts = Hosts::new();
        hosts.insert(connector_id, Box::new(folder));
        let library = Library::open(library_dir.path().join("library")).unwrap();
        Setup {
            host_dir,
            library_dir,
            store_dir,
            collection,
            hosts,
            library,
            connector_id,
        }
    }

    /// A snapshot of each file of a folder, with its path and its bytes,
    /// to show that a move did not change a hosted file.
    fn snapshot_dir(dir: &Path) -> Vec<(String, Vec<u8>)> {
        fn walk(base: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
            let mut entries: Vec<_> = std::fs::read_dir(dir)
                .unwrap()
                .map(|e| e.unwrap())
                .collect();
            entries.sort_by_key(std::fs::DirEntry::file_name);
            for entry in entries {
                let path = entry.path();
                if path.is_dir() {
                    walk(base, &path, out);
                } else {
                    let name = path
                        .strip_prefix(base)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned();
                    out.push((name, std::fs::read(&path).unwrap()));
                }
            }
        }
        let mut out = Vec::new();
        walk(dir, dir, &mut out);
        out
    }

    #[test]
    fn into_library_gives_each_feed_a_folder_and_moves_one_pending_file() {
        let mut s = setup();
        {
            let connector = s.hosts.get(s.connector_id).unwrap();
            connector
                .write("album-one.xml", b"<rss>one</rss>", &Expected::Absent)
                .unwrap();
            connector
                .write("album-two.xml", b"<rss>two</rss>", &Expected::Absent)
                .unwrap();
        }
        let feed_one = s
            .collection
            .add_feed(
                Some("https://music.example/album-one.xml"),
                Some((s.connector_id, "album-one.xml")),
            )
            .unwrap();
        let feed_two = s
            .collection
            .add_feed(
                Some("https://music.example/album-two.xml"),
                Some((s.connector_id, "album-two.xml")),
            )
            .unwrap();

        // A pending file of `feed_one`, from before the library (ADR 0036
        // item 5).
        let bytes = b"music";
        let sha256 = sha256_hex(&[bytes]);
        let feed_dir = s.store_dir.path().join(feed_one.0.to_string());
        std::fs::create_dir_all(&feed_dir).unwrap();
        std::fs::write(feed_dir.join(&sha256), bytes).unwrap();
        s.collection
            .add_draft_file(
                feed_one,
                &DraftFile {
                    path: "01 Song.mp3".to_owned(),
                    size: bytes.len() as u64,
                    media_type: "audio/mpeg".to_owned(),
                    sha256: sha256.clone(),
                },
            )
            .unwrap();

        let host_before = snapshot_dir(s.host_dir.path());

        let report = into_library(&s.collection, &s.hosts, &s.library, s.store_dir.path()).unwrap();
        assert_eq!(report.release_folders, 2);
        assert_eq!(report.feed_files, 2);
        assert_eq!(report.moved_files, 1);
        assert!(report.errors.is_empty());

        let row_one = s.collection.feed(feed_one).unwrap();
        let row_two = s.collection.feed(feed_two).unwrap();
        assert_eq!(row_one.folder.as_deref(), Some("album-one"));
        assert_eq!(row_two.folder.as_deref(), Some("album-two"));
        assert_eq!(row_one.files_connector_id, Some(s.connector_id));
        assert_eq!(row_two.files_connector_id, Some(s.connector_id));

        assert_eq!(
            std::fs::read(s.library.path("album-one", "album-one.xml")).unwrap(),
            b"<rss>one</rss>"
        );
        assert_eq!(
            std::fs::read(s.library.path("album-two", "album-two.xml")).unwrap(),
            b"<rss>two</rss>"
        );
        assert_eq!(
            std::fs::read(s.library.path("album-one", "01 Song.mp3")).unwrap(),
            bytes
        );

        let files = s.collection.library_files(feed_one).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "01 Song.mp3");
        assert!(files[0].local);
        assert_eq!(files[0].sha256.as_deref(), Some(sha256.as_str()));

        // The table of the pending files is gone, now that the library
        // replaces it (ADR 0036 item 5).
        assert!(!s.collection.has_draft_files_table().unwrap());

        // The move did not change a hosted file.
        assert_eq!(snapshot_dir(s.host_dir.path()), host_before);

        // A second call does nothing.
        let second = into_library(&s.collection, &s.hosts, &s.library, s.store_dir.path()).unwrap();
        assert_eq!(second, MoveReport::default());
        let row_one_again = s.collection.feed(feed_one).unwrap();
        assert_eq!(row_one_again.folder.as_deref(), Some("album-one"));

        // A feed with no folder after the table is gone moves with no
        // pending file, for example after the user sets the library again.
        s.collection.set_feed_folder_none_for_test(feed_one);
        let third = into_library(&s.collection, &s.hosts, &s.library, s.store_dir.path()).unwrap();
        assert!(third.errors.is_empty(), "{:?}", third.errors);
        assert_eq!(third.feed_files, 1);
    }

    #[test]
    fn into_library_records_the_error_of_one_feed_and_moves_the_others() {
        let mut s = setup();
        {
            let connector = s.hosts.get(s.connector_id).unwrap();
            connector
                .write("album.xml", b"<rss/>", &Expected::Absent)
                .unwrap();
        }
        let good = s
            .collection
            .add_feed(
                Some("https://music.example/album.xml"),
                Some((s.connector_id, "album.xml")),
            )
            .unwrap();

        // A feed whose connector is not in the set of hosts: its move
        // fails, and the move of the other feed still runs.
        let missing_connector = s
            .collection
            .add_connector("local", "Gone", "{}", None)
            .unwrap();
        let broken = s
            .collection
            .add_feed(None, Some((missing_connector, "gone.xml")))
            .unwrap();

        let report = into_library(&s.collection, &s.hosts, &s.library, s.store_dir.path()).unwrap();
        assert_eq!(report.feed_files, 1);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0].feed_id, broken);
        assert!(s.collection.feed(good).unwrap().folder.is_some());
        // The failed feed gets no folder in the collection, so a later
        // call can retry it.
        assert!(s.collection.feed(broken).unwrap().folder.is_none());

        // Once the connector is in the set of hosts, a second call moves
        // the feed that failed before.
        let gone_dir = tempfile::tempdir().unwrap();
        let gone_folder = open_connector(gone_dir.path(), "https://gone.example");
        gone_folder
            .write("gone.xml", b"<rss>gone</rss>", &Expected::Absent)
            .unwrap();
        s.hosts.insert(missing_connector, Box::new(gone_folder));

        let second = into_library(&s.collection, &s.hosts, &s.library, s.store_dir.path()).unwrap();
        assert!(second.errors.is_empty());
        assert_eq!(second.feed_files, 1);
        let row = s.collection.feed(broken).unwrap();
        assert_eq!(row.folder.as_deref(), Some("gone"));
        assert_eq!(
            std::fs::read(s.library.path("gone", "gone.xml")).unwrap(),
            b"<rss>gone</rss>"
        );
    }

    #[test]
    fn into_library_gives_a_read_only_feed_its_xml_from_its_address() {
        let mut s = setup();
        let server = TestServer::start(Vec::new());
        server.set("/album.xml", 200, b"<rss>remote</rss>".to_vec());
        let url = server.url("/album.xml");
        let feed = s.collection.add_feed(Some(&url), None).unwrap();

        let report = into_library(&s.collection, &s.hosts, &s.library, s.store_dir.path()).unwrap();
        assert_eq!(report.release_folders, 1);
        assert_eq!(report.feed_files, 1);
        assert!(report.errors.is_empty());

        let row = s.collection.feed(feed).unwrap();
        let folder = row.folder.unwrap();
        assert_eq!(
            std::fs::read(s.library.path(&folder, "album.xml")).unwrap(),
            b"<rss>remote</rss>"
        );
    }
}

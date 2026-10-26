//! The scan of the release folders of the library (ADR 0038 items 13 to
//! 15).
//!
//! The scan compares each release folder with the records of its files
//! (`collection::LibraryFile`) and gives each difference. It reads a file
//! only when the stamp of the file is different from the stamp of its
//! row. It changes the stamp of a row when the bytes did not change, and
//! it changes nothing else. The user decides what Studio does with each
//! difference.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use serde::Serialize;

use crate::collection::{Collection, CollectionError, FeedId, FeedRow, LibraryFile, LibraryFileId};
use crate::digest::sha256_file;
use crate::library::{Library, file_stamp, is_plain_name, stamp_of};
use crate::remote::media_type_of_name;

/// The media type of a file that Studio cannot name from its extension.
const UNKNOWN_MEDIA_TYPE: &str = "application/octet-stream";

/// A difference between a release folder and the records of its files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Difference {
    /// The feed of the release folder.
    pub feed_id: FeedId,
    /// What is different.
    #[serde(flatten)]
    pub kind: DifferenceKind,
}

/// The kind of a [`Difference`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DifferenceKind {
    /// A file in the folder that has no row.
    New {
        /// The path of the file in the release folder.
        path: String,
        /// The size, in bytes.
        size: u64,
    },
    /// A row with `local` true that has no file in the folder.
    Missing {
        /// The row.
        file: LibraryFileId,
    },
    /// A row with `local` true whose file has a different stamp and a
    /// different SHA-256.
    Changed {
        /// The row.
        file: LibraryFileId,
        /// The size of the file now, in bytes.
        size: u64,
        /// The SHA-256 of the file now.
        sha256: String,
    },
}

/// The error of a folder scan function.
#[derive(Debug)]
pub enum ScanError {
    /// The collection database failed.
    Collection(CollectionError),
    /// Studio could not read the folder or a file.
    Io(io::Error),
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Collection(error) => error.fmt(f),
            Self::Io(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ScanError {}

impl From<CollectionError> for ScanError {
    fn from(error: CollectionError) -> Self {
        Self::Collection(error)
    }
}

impl From<io::Error> for ScanError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Wrap an I/O error so its message names the path that failed.
fn named(path: &Path, error: &io::Error) -> io::Error {
    io::Error::new(error.kind(), format!("{}: {error}", path.display()))
}

/// Compare the release folder of each feed with the records of its files
/// (ADR 0038 items 13 and 14). A feed with no release folder gives no
/// difference.
///
/// # Errors
///
/// Gives a [`ScanError`] when SQLite fails, or Studio cannot read a
/// folder or a file whose stamp changed.
pub fn scan(collection: &Collection, library: &Library) -> Result<Vec<Difference>, ScanError> {
    let mut differences = Vec::new();
    for row in collection.feeds()? {
        differences.extend(scan_row(collection, library, &row)?);
    }
    Ok(differences)
}

/// Compare the release folder of one feed with the records of its files.
/// See [`scan`].
///
/// # Errors
///
/// Gives a [`ScanError`] as [`scan`] does.
pub fn scan_feed(
    collection: &Collection,
    library: &Library,
    feed_id: FeedId,
) -> Result<Vec<Difference>, ScanError> {
    let row = collection.feed(feed_id)?;
    scan_row(collection, library, &row)
}

/// True when `name` is not a library file: it is the feed XML of the
/// release, under each name that the feed gives, or the temporary file of
/// a feed XML write.
fn is_feed_xml(row: &FeedRow, name: &str) -> bool {
    let last = |source: &str| {
        source
            .rsplit_once('/')
            .map_or(source, |(_, l)| l)
            .to_owned()
    };
    [row.path.as_deref(), row.public_url.as_deref()]
        .into_iter()
        .flatten()
        .any(|source| last(source) == name)
        || name.ends_with(".studio-tmp")
}

fn scan_row(
    collection: &Collection,
    library: &Library,
    row: &FeedRow,
) -> Result<Vec<Difference>, ScanError> {
    let Some(folder) = row.folder.as_deref() else {
        return Ok(Vec::new());
    };
    if !is_plain_name(folder) {
        return Ok(Vec::new());
    }
    let dir = library.root().join(folder);
    let mut present: Vec<(String, fs::Metadata)> = Vec::new();
    match fs::read_dir(&dir) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry.map_err(|e| named(&dir, &e))?;
                // A name that is not text cannot be a row path.
                let Ok(name) = entry.file_name().into_string() else {
                    continue;
                };
                // A hidden file, such as a partial download, is not a
                // library file.
                if name.starts_with('.') || is_feed_xml(row, &name) {
                    continue;
                }
                let Ok(meta) = fs::metadata(entry.path()) else {
                    continue;
                };
                if meta.is_file() {
                    present.push((name, meta));
                }
            }
        }
        // A release folder that is not there has no file.
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(named(&dir, &e).into()),
    }
    present.sort_by(|a, b| a.0.cmp(&b.0));

    let rows = collection.library_files(row.id)?;
    let mut differences = Vec::new();
    for (name, meta) in &present {
        match rows.iter().find(|file| &file.path == name) {
            None => differences.push(Difference {
                feed_id: row.id,
                kind: DifferenceKind::New {
                    path: name.clone(),
                    size: meta.len(),
                },
            }),
            // The bytes of a remote row are not in the library, so its
            // file in the folder is not a difference to examine here.
            Some(file) if !file.local => {}
            Some(file) => {
                if let Some(kind) = compare(collection, library, folder, file, meta)? {
                    differences.push(Difference {
                        feed_id: row.id,
                        kind,
                    });
                }
            }
        }
    }
    for file in &rows {
        if file.local && !present.iter().any(|(name, _)| name == &file.path) {
            differences.push(Difference {
                feed_id: row.id,
                kind: DifferenceKind::Missing { file: file.id },
            });
        }
    }
    Ok(differences)
}

/// Compare one file with its row. The function reads the file only when
/// its stamp is different from the stamp of the row (ADR 0038 item 13).
fn compare(
    collection: &Collection,
    library: &Library,
    folder: &str,
    file: &LibraryFile,
    meta: &fs::Metadata,
) -> Result<Option<DifferenceKind>, ScanError> {
    // Take the stamp before the read. A write during the read then gives
    // an old stamp, and the next scan examines the file again.
    let stamp = stamp_of(meta);
    if file.stamp.as_deref() == Some(stamp.as_str()) {
        return Ok(None);
    }
    let path = library.path(folder, &file.path);
    let sha256 = sha256_file(&path).map_err(|e| named(&path, &e))?;
    if file.sha256.as_deref() == Some(sha256.as_str()) {
        collection.set_library_file_stamp(file.id, Some(&stamp))?;
        return Ok(None);
    }
    Ok(Some(DifferenceKind::Changed {
        file: file.id,
        size: meta.len(),
        sha256,
    }))
}

/// Add a file of a release folder to the library as a row with `local`
/// true: its size, SHA-256, stamp and the media type of its extension
/// (ADR 0038 item 15). The row is not an address of the feed.
///
/// # Errors
///
/// Gives a [`ScanError`] when the feed has no release folder, `path` is
/// not a plain file name, Studio cannot read the file, or SQLite fails.
pub fn add_new(
    collection: &Collection,
    library: &Library,
    feed_id: FeedId,
    path: &str,
) -> Result<LibraryFileId, ScanError> {
    let row = collection.feed(feed_id)?;
    let folder = row.folder.as_deref().filter(|f| is_plain_name(f));
    let (Some(folder), true) = (folder, is_plain_name(path)) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{path:?} is not a file of a release folder"),
        )
        .into());
    };
    let full = library.path(folder, path);
    let stamp = file_stamp(&full);
    let size = fs::metadata(&full).map_err(|e| named(&full, &e))?.len();
    let sha256 = sha256_file(&full).map_err(|e| named(&full, &e))?;
    let media_type = media_type_of_name(path).unwrap_or_else(|| UNKNOWN_MEDIA_TYPE.to_owned());
    Ok(collection.add_library_file(
        feed_id,
        path,
        size,
        Some(&sha256),
        &media_type,
        true,
        None,
        stamp.as_deref(),
    )?)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use super::*;
    use crate::digest::sha256_hex;

    struct Setup {
        _dir: tempfile::TempDir,
        collection: Collection,
        library: Library,
        feed: FeedId,
    }

    fn setup() -> Setup {
        let dir = tempfile::tempdir().unwrap();
        let library = Library::open(dir.path().join("library")).unwrap();
        let mut collection = Collection::open_in_memory().unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        collection.set_feed_folder(feed, "album").unwrap();
        collection
            .set_address(
                feed,
                crate::address::AddressSource::Folder,
                Some("https://example.com/album/album.xml"),
            )
            .unwrap();
        Setup {
            _dir: dir,
            collection,
            library,
            feed,
        }
    }

    /// Write a file in the release folder and record it like an import.
    fn record(s: &Setup, name: &str, bytes: &[u8]) -> LibraryFileId {
        let path = s.library.path("album", name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, bytes).unwrap();
        s.collection
            .add_library_file(
                s.feed,
                name,
                bytes.len() as u64,
                Some(&sha256_hex(&[bytes])),
                "audio/mpeg",
                true,
                None,
                file_stamp(&path).as_deref(),
            )
            .unwrap()
    }

    fn set_modified(path: &Path, time: SystemTime) {
        fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(time)
            .unwrap();
    }

    #[test]
    fn a_new_file_a_missing_file_and_a_changed_file_give_one_difference_each() {
        let s = setup();
        record(&s, "kept.mp3", b"kept");
        let gone = record(&s, "gone.mp3", b"gone");
        let changed = record(&s, "changed.mp3", b"before");
        fs::remove_file(s.library.path("album", "gone.mp3")).unwrap();
        let changed_path = s.library.path("album", "changed.mp3");
        fs::write(&changed_path, b"after!").unwrap();
        // The new bytes have the same size. Two writes in a short time can
        // give the same modification time, and then the stamp is the same.
        set_modified(&changed_path, SystemTime::now() - Duration::from_secs(3600));
        fs::write(s.library.path("album", "new.mp3"), b"new").unwrap();
        // These give no difference.
        fs::write(s.library.path("album", "album.xml"), b"<rss/>").unwrap();
        fs::write(s.library.path("album", "album.xml.studio-tmp"), b"x").unwrap();
        fs::write(s.library.path("album", ".hidden.mp3"), b"hidden").unwrap();
        fs::write(s.library.path("album", ".new.mp3.part"), b"part").unwrap();
        fs::create_dir(s.library.path("album", "sub")).unwrap();
        fs::write(s.library.path("album", "sub").join("deep.mp3"), b"deep").unwrap();

        let found = scan(&s.collection, &s.library).unwrap();
        let kinds: Vec<&DifferenceKind> = found.iter().map(|d| &d.kind).collect();
        assert_eq!(found.len(), 3, "{kinds:?}");
        assert!(found.iter().all(|d| d.feed_id == s.feed));
        assert!(kinds.contains(&&DifferenceKind::New {
            path: "new.mp3".to_owned(),
            size: 3
        }));
        assert!(kinds.contains(&&DifferenceKind::Missing { file: gone }));
        assert!(kinds.contains(&&DifferenceKind::Changed {
            file: changed,
            size: 6,
            sha256: sha256_hex(&[b"after!"]),
        }));
    }

    #[test]
    fn a_second_scan_reads_no_file_whose_stamp_did_not_change() {
        let s = setup();
        record(&s, "a.mp3", b"aaaa");
        let path = s.library.path("album", "a.mp3");
        let modified = fs::metadata(&path).unwrap().modified().unwrap();
        // Other bytes with the same size and the same modification time.
        // A scan that reads the file would give a changed file.
        fs::write(&path, b"bbbb").unwrap();
        set_modified(&path, modified);
        assert_eq!(
            fs::metadata(&path).unwrap().modified().unwrap(),
            modified,
            "the test needs the same modification time"
        );

        assert_eq!(scan(&s.collection, &s.library).unwrap(), Vec::new());
    }

    #[test]
    fn a_new_modification_time_with_the_same_bytes_gives_no_difference_and_a_new_stamp() {
        let s = setup();
        let id = record(&s, "a.mp3", b"aaaa");
        let path = s.library.path("album", "a.mp3");
        let before = s.collection.library_file(id).unwrap().stamp;
        set_modified(&path, SystemTime::now() - Duration::from_secs(3600));

        assert_eq!(scan(&s.collection, &s.library).unwrap(), Vec::new());
        let after = s.collection.library_file(id).unwrap().stamp;
        assert_ne!(before, after);
        assert_eq!(after, file_stamp(&path));
    }

    #[test]
    fn a_row_with_no_stamp_is_read_and_gets_its_stamp() {
        let s = setup();
        let id = record(&s, "a.mp3", b"aaaa");
        s.collection.set_library_file_stamp(id, None).unwrap();

        assert_eq!(scan(&s.collection, &s.library).unwrap(), Vec::new());
        let path = s.library.path("album", "a.mp3");
        assert_eq!(
            s.collection.library_file(id).unwrap().stamp,
            file_stamp(&path)
        );
    }

    #[test]
    fn a_remote_row_and_a_feed_with_no_folder_give_no_difference() {
        let mut s = setup();
        s.collection
            .add_library_file(s.feed, "far.mp3", 5, None, "audio/mpeg", false, None, None)
            .unwrap();
        fs::create_dir_all(s.library.path("album", "")).unwrap();
        fs::write(s.library.path("album", "far.mp3"), b"bytes").unwrap();
        s.collection.add_feed(None, None).unwrap();
        assert_eq!(scan(&s.collection, &s.library).unwrap(), Vec::new());
    }

    #[test]
    fn add_new_records_the_file_as_a_local_row() {
        let s = setup();
        fs::create_dir_all(s.library.path("album", "")).unwrap();
        fs::write(s.library.path("album", "cover.png"), b"png").unwrap();

        let id = add_new(&s.collection, &s.library, s.feed, "cover.png").unwrap();
        let row = s.collection.library_file(id).unwrap();
        assert!(row.local);
        assert_eq!(row.size, 3);
        assert_eq!(row.sha256, Some(sha256_hex(&[b"png"])));
        assert_eq!(row.media_type, "image/png");
        assert!(row.stamp.is_some());
        assert_eq!(scan(&s.collection, &s.library).unwrap(), Vec::new());
        assert!(add_new(&s.collection, &s.library, s.feed, "../x").is_err());
    }
}

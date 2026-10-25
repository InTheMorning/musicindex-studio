//! The download of a remote library file into the library (ADR 0038
//! items 6 to 12).
//!
//! A remote file has a row with `local` false. A download reads the bytes
//! from a connected host, or over HTTP, into a hidden partial file in the
//! release folder. [`finish`] then moves the bytes to the name of the
//! file and records them. The API keeps the queue (`studio-api`). This
//! module does the steps of one download. It never writes to a host.

use std::fmt;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use crate::collection::{Collection, CollectionError, LibraryFile};
use crate::digest::sha256_file;
use crate::inventory::{self, Hosts, InventoryError};
use crate::library::{Library, file_stamp};
use crate::web::{Web, WebError, WebErrorKind};

/// The default of each download limit setting, 2 GiB (ADR 0036 item 17).
pub const DEFAULT_LIMIT_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// The setting for the largest file that Studio downloads.
pub const MAX_FILE_SETTING: &str = "download_max_file_bytes";

/// The setting for the free space that a download leaves on the disk.
pub const MIN_FREE_SETTING: &str = "download_min_free_bytes";

/// The kind of a [`DownloadError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DownloadErrorKind {
    /// The file is larger than `download_max_file_bytes`.
    TooLarge,
    /// The download leaves less than `download_min_free_bytes` free.
    NoSpace,
    /// The file has no copy on a connected host and no address.
    NoSource,
    /// The caller stopped the download. The partial file stays.
    Stopped,
    /// The file system, a host or the network failed.
    Transfer,
    /// The collection database failed.
    Collection,
}

/// An error from a download function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadError {
    kind: DownloadErrorKind,
    message: String,
}

impl DownloadError {
    fn new(kind: DownloadErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// The kind of the error.
    #[must_use]
    pub fn kind(&self) -> DownloadErrorKind {
        self.kind
    }
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DownloadError {}

impl From<CollectionError> for DownloadError {
    fn from(error: CollectionError) -> Self {
        Self::new(DownloadErrorKind::Collection, error.to_string())
    }
}

impl From<InventoryError> for DownloadError {
    fn from(error: InventoryError) -> Self {
        Self::new(DownloadErrorKind::Transfer, error.to_string())
    }
}

impl From<WebError> for DownloadError {
    fn from(error: WebError) -> Self {
        let kind = match error.kind() {
            WebErrorKind::Stopped => DownloadErrorKind::Stopped,
            WebErrorKind::TooLarge => DownloadErrorKind::TooLarge,
            _ => DownloadErrorKind::Transfer,
        };
        Self::new(kind, error.to_string())
    }
}

/// Where the bytes of a remote file come from (ADR 0038 item 8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// A file on a connected host. Studio reads it through the connector.
    Host {
        /// The host.
        connector_id: crate::collection::ConnectorId,
        /// The path of the file on the host.
        path: String,
    },
    /// An address. Studio reads it over HTTP.
    Web {
        /// The address.
        url: String,
    },
}

/// The source of a remote file: its first copy on a connected host, else
/// its `source_url`. Gives `None` when the file has neither (ADR 0038
/// item 8).
///
/// # Errors
///
/// Gives a [`DownloadError`] when the collection or a host fails.
pub fn source_of(
    collection: &Collection,
    hosts: &Hosts,
    file: &LibraryFile,
) -> Result<Option<Source>, DownloadError> {
    if let Some(copy) = inventory::copies(collection, hosts, file)?
        .into_iter()
        .next()
    {
        return Ok(Some(Source::Host {
            connector_id: copy.connector_id,
            path: copy.path,
        }));
    }
    Ok(file.source_url.clone().map(|url| Source::Web { url }))
}

/// The partial file of a download: `.{name}.part` in the release folder.
#[must_use]
pub fn part_path(library: &Library, folder: &str, name: &str) -> PathBuf {
    library.path(folder, &format!(".{name}.part"))
}

fn limit(collection: &Collection, key: &str) -> Result<u64, DownloadError> {
    match collection.setting(key)? {
        Some(value) => value.as_u64().ok_or_else(|| {
            DownloadError::new(
                DownloadErrorKind::Collection,
                format!("the {key} setting is not a number"),
            )
        }),
        None => Ok(DEFAULT_LIMIT_BYTES),
    }
}

/// The largest file that Studio downloads, from the setting.
///
/// # Errors
///
/// Gives a [`DownloadError`] when the setting is not a number.
pub fn max_file_bytes(collection: &Collection) -> Result<u64, DownloadError> {
    limit(collection, MAX_FILE_SETTING)
}

/// Refuse a download of `size` bytes that breaks a limit (ADR 0038 item
/// 10). The message names the setting.
///
/// A size of 0 means that Studio does not know the size yet. Only the
/// transfer can then find a file that is too large.
///
/// # Errors
///
/// Gives [`DownloadErrorKind::TooLarge`] for a size more than
/// `download_max_file_bytes`. Gives [`DownloadErrorKind::NoSpace`] when
/// the disk of the library has less than `download_min_free_bytes` free
/// after the download.
pub fn check_limits(
    collection: &Collection,
    library: &Library,
    size: u64,
) -> Result<(), DownloadError> {
    let max = limit(collection, MAX_FILE_SETTING)?;
    if size > max {
        return Err(DownloadError::new(
            DownloadErrorKind::TooLarge,
            format!(
                "the file has {size} bytes, more than the {MAX_FILE_SETTING} setting of {max} bytes"
            ),
        ));
    }
    let min_free = limit(collection, MIN_FREE_SETTING)?;
    let available = fs4::available_space(library.root()).map_err(|e| {
        DownloadError::new(
            DownloadErrorKind::Transfer,
            format!("{}: {e}", library.root().display()),
        )
    })?;
    if available.saturating_sub(size) < min_free {
        return Err(DownloadError::new(
            DownloadErrorKind::NoSpace,
            format!(
                "a download of {size} bytes leaves less than the {MIN_FREE_SETTING} setting of {min_free} bytes free; the disk has {available} bytes free"
            ),
        ));
    }
    Ok(())
}

/// Read the bytes of `source` into `part` (ADR 0038 item 8). Gives the size
/// of the complete file.
///
/// A host source reads the whole file through its connector, and writes
/// `part` again from 0. A web source continues `part` with a range
/// request. `progress` and `stop` work as in [`Web::get_to_file`].
///
/// # Errors
///
/// Gives a [`DownloadError`] when the transfer fails, the file is larger
/// than `max_bytes`, or `stop` is set.
pub fn fetch(
    hosts: &Hosts,
    web: &Web,
    source: &Source,
    part: &Path,
    max_bytes: u64,
    progress: &mut dyn FnMut(u64, Option<u64>),
    stop: &AtomicBool,
) -> Result<u64, DownloadError> {
    if let Some(dir) = part.parent() {
        std::fs::create_dir_all(dir).map_err(|e| {
            DownloadError::new(
                DownloadErrorKind::Transfer,
                format!("{}: {e}", dir.display()),
            )
        })?;
    }
    match source {
        Source::Web { url } => Ok(web.get_to_file(url, part, max_bytes, progress, stop)?),
        Source::Host { connector_id, path } => {
            let connector = hosts.get(*connector_id)?;
            let read = connector
                .read(path)
                .map_err(|e| DownloadError::new(DownloadErrorKind::Transfer, e.to_string()))?
                .ok_or_else(|| {
                    DownloadError::new(
                        DownloadErrorKind::Transfer,
                        format!("{path} is not on host {} any more", connector_id.0),
                    )
                })?;
            let size = read.bytes.len() as u64;
            if size > max_bytes {
                return Err(DownloadError::new(
                    DownloadErrorKind::TooLarge,
                    format!("{path} has {size} bytes, more than {max_bytes} bytes"),
                ));
            }
            std::fs::File::create(part)
                .and_then(|mut file| file.write_all(&read.bytes).and_then(|()| file.flush()))
                .map_err(|e| {
                    DownloadError::new(
                        DownloadErrorKind::Transfer,
                        format!("{}: {e}", part.display()),
                    )
                })?;
            progress(size, Some(size));
            Ok(size)
        }
    }
}

/// The size and the SHA-256 of a complete partial file.
///
/// The caller does not keep the workspace lock during this read, because
/// a large file takes some seconds to read.
///
/// # Errors
///
/// Gives a [`DownloadError`] when the read fails.
pub fn hash_part(part: &Path) -> Result<PartFacts, DownloadError> {
    let io_error = |e: std::io::Error| {
        DownloadError::new(
            DownloadErrorKind::Transfer,
            format!("{}: {e}", part.display()),
        )
    };
    let size = std::fs::metadata(part).map_err(io_error)?.len();
    let sha256 = sha256_file(part).map_err(io_error)?;
    Ok(PartFacts { size, sha256 })
}

/// The size and the SHA-256 of a complete partial file, from
/// [`hash_part`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartFacts {
    /// The size, in bytes.
    pub size: u64,
    /// The SHA-256, as hexadecimal text.
    pub sha256: String,
}

/// Move `part` to the name of `file` in its release folder, record the
/// size and the SHA-256 from [`hash_part`] and the stamp, and remove the
/// download record (ADR 0038 item 11).
///
/// # Errors
///
/// Gives a [`DownloadError`] when the feed has no release folder, the
/// rename fails, or the collection fails.
pub fn finish(
    collection: &Collection,
    library: &Library,
    file: &LibraryFile,
    part: &Path,
    facts: &PartFacts,
) -> Result<(), DownloadError> {
    let io_error = |path: &Path, e: std::io::Error| {
        DownloadError::new(
            DownloadErrorKind::Transfer,
            format!("{}: {e}", path.display()),
        )
    };
    let row = collection.feed(file.feed_id)?;
    let folder = row.folder.as_deref().ok_or_else(|| {
        DownloadError::new(
            DownloadErrorKind::Transfer,
            format!("feed {} has no release folder", file.feed_id.0),
        )
    })?;
    let target = library.path(folder, &file.path);
    std::fs::rename(part, &target).map_err(|e| io_error(&target, e))?;
    let stamp = file_stamp(&target);
    collection.set_library_file_local(file.id, facts.size, &facts.sha256, stamp.as_deref())?;
    collection.remove_download(file.id)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::{ConnectorId, FeedId};
    use crate::connector::LocalFolder;
    use crate::digest::sha256_hex;
    use crate::testing::TestServer;
    use serde_json::json;

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
        Setup {
            _dir: dir,
            collection,
            library,
            feed,
        }
    }

    fn remote(s: &Setup, name: &str, url: Option<&str>) -> LibraryFile {
        let id = s
            .collection
            .add_library_file(s.feed, name, 0, None, "audio/mpeg", false, url, None)
            .unwrap();
        s.collection.library_file(id).unwrap()
    }

    #[test]
    fn the_source_is_a_copy_on_a_connected_host_else_the_address() {
        let s = setup();
        let host_dir = tempfile::tempdir().unwrap();
        std::fs::write(host_dir.path().join("a.mp3"), b"bytes").unwrap();
        let connector = s
            .collection
            .add_connector("local", "host", "{}", None)
            .unwrap();
        let mut hosts = Hosts::new();
        hosts.insert(
            connector,
            Box::new(LocalFolder::new(
                host_dir.path(),
                Some("https://cdn.example".to_owned()),
            )),
        );
        inventory::scan(&s.collection, &hosts, connector).unwrap();

        let file = remote(&s, "a.mp3", Some("https://cdn.example/a.mp3"));
        assert_eq!(
            source_of(&s.collection, &hosts, &file).unwrap(),
            Some(Source::Host {
                connector_id: connector,
                path: "a.mp3".to_owned()
            })
        );
        let other = remote(&s, "b.mp3", Some("https://other.example/b.mp3"));
        assert_eq!(
            source_of(&s.collection, &hosts, &other).unwrap(),
            Some(Source::Web {
                url: "https://other.example/b.mp3".to_owned()
            })
        );
        let none = remote(&s, "c.mp3", None);
        assert_eq!(source_of(&s.collection, &hosts, &none).unwrap(), None);
    }

    #[test]
    fn the_limits_refuse_a_large_file_and_a_full_disk_and_name_the_setting() {
        let s = setup();
        s.collection
            .set_setting(MAX_FILE_SETTING, &json!(1000))
            .unwrap();
        check_limits(&s.collection, &s.library, 1000).unwrap();
        let error = check_limits(&s.collection, &s.library, 1001).unwrap_err();
        assert_eq!(error.kind(), DownloadErrorKind::TooLarge);
        assert!(error.to_string().contains(MAX_FILE_SETTING));

        s.collection
            .set_setting(MIN_FREE_SETTING, &json!(u64::MAX))
            .unwrap();
        let error = check_limits(&s.collection, &s.library, 10).unwrap_err();
        assert_eq!(error.kind(), DownloadErrorKind::NoSpace);
        assert!(error.to_string().contains(MIN_FREE_SETTING));
    }

    #[test]
    fn a_host_source_downloads_through_the_connector_and_finish_records_the_file() {
        let s = setup();
        let host_dir = tempfile::tempdir().unwrap();
        std::fs::write(host_dir.path().join("a.mp3"), b"audio bytes").unwrap();
        let connector = ConnectorId(1);
        let mut hosts = Hosts::new();
        hosts.insert(connector, Box::new(LocalFolder::new(host_dir.path(), None)));
        let file = remote(&s, "a.mp3", None);
        s.collection.queue_download(file.id).unwrap();
        let part = part_path(&s.library, "album", "a.mp3");
        let source = Source::Host {
            connector_id: connector,
            path: "a.mp3".to_owned(),
        };
        let size = fetch(
            &hosts,
            &Web::new(),
            &source,
            &part,
            u64::MAX,
            &mut |_, _| {},
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(size, 11);

        finish(
            &s.collection,
            &s.library,
            &file,
            &part,
            &hash_part(&part).unwrap(),
        )
        .unwrap();
        let row = s.collection.library_file(file.id).unwrap();
        assert!(row.local);
        assert_eq!(row.size, 11);
        assert_eq!(row.sha256, Some(sha256_hex(&[b"audio bytes"])));
        assert!(row.stamp.is_some());
        assert_eq!(s.collection.downloads().unwrap(), Vec::new());
        assert!(!part.exists());
        assert_eq!(
            std::fs::read(s.library.path("album", "a.mp3")).unwrap(),
            b"audio bytes"
        );
    }

    #[test]
    fn a_web_source_downloads_over_http() {
        let s = setup();
        let server = TestServer::start(Vec::new());
        server.set("/a.mp3", 200, vec![3; 5000]);
        let part = part_path(&s.library, "album", "a.mp3");
        let size = fetch(
            &Hosts::new(),
            &Web::new(),
            &Source::Web {
                url: server.url("/a.mp3"),
            },
            &part,
            u64::MAX,
            &mut |_, _| {},
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(size, 5000);
    }
}

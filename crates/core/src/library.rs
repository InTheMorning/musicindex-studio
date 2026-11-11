//! The library folder on the user's machine (ADR 0035, ADR 0036).
//!
//! Studio keeps a release folder for each feed in the library: the feed
//! XML and each file of the release. The collection keeps the records of
//! each file (`collection::LibraryFile`). This module keeps the bytes.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::collection::{Collection, CollectionError, FeedId};
use crate::digest::sha256_hex;

/// The name of the file that [`Library::open`] writes and removes, to
/// check that Studio can write in the folder (ADR 0036 item 1).
const WRITE_TEST_FILE: &str = ".studio-write-test";

/// The library folder of the user (ADR 0035 item 1, ADR 0036 item 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    root: PathBuf,
}

impl Library {
    /// Open the library folder. Studio makes the folder when it is
    /// absent, then writes and removes a test file. This finds a folder
    /// with no write permission now, and not at the first write.
    ///
    /// # Errors
    ///
    /// Gives an `io::Error` that names the path when Studio cannot make
    /// or write in the folder.
    pub fn open(root: impl Into<PathBuf>) -> io::Result<Self> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(|e| named(&root, &e))?;
        let test = root.join(WRITE_TEST_FILE);
        let write = fs::File::create(&test).and_then(|mut file| file.write_all(b"studio"));
        write.map_err(|e| named(&test, &e))?;
        fs::remove_file(&test).map_err(|e| named(&test, &e))?;
        Ok(Self { root })
    }

    /// The root folder of the library.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// List library folders up to three levels below the library root.
    ///
    /// # Errors
    ///
    /// Gives an I/O error when Studio cannot read a folder.
    pub fn folders(&self) -> io::Result<Vec<String>> {
        fn visit(
            root: &Path,
            folder: &Path,
            depth: usize,
            out: &mut Vec<String>,
        ) -> io::Result<()> {
            if depth > 3 {
                return Ok(());
            }
            let mut entries = fs::read_dir(folder)
                .map_err(|error| named(folder, &error))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| named(folder, &error))?;
            entries.sort_by_key(std::fs::DirEntry::file_name);
            for entry in entries {
                if !entry
                    .file_type()
                    .map_err(|error| named(&entry.path(), &error))?
                    .is_dir()
                {
                    continue;
                }
                let path = entry.path();
                let relative = path
                    .strip_prefix(root)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
                out.push(relative.to_string_lossy().replace('\\', "/"));
                visit(root, &path, depth + 1, out)?;
            }
            Ok(())
        }

        let mut folders = Vec::new();
        visit(&self.root, &self.root, 1, &mut folders)?;
        folders.sort();
        Ok(folders)
    }

    /// The path of a file in a release folder.
    #[must_use]
    pub fn path(&self, folder: &str, name: &str) -> PathBuf {
        self.root.join(folder).join(name)
    }

    /// Check whether a relative release folder path stays below the library root.
    #[must_use]
    pub fn is_folder_path(path: &str) -> bool {
        !path.is_empty()
            && !path.contains(['\\', ':'])
            && Path::new(path)
                .components()
                .all(|part| matches!(part, std::path::Component::Normal(_)))
    }

    /// Move one release folder to another relative path in this library.
    ///
    /// # Errors
    ///
    /// Gives an I/O error when either path is invalid or the move fails.
    pub fn move_folder(&self, from: &str, to: &str) -> io::Result<()> {
        check_folder_path(from)?;
        check_folder_path(to)?;
        let source = self.root.join(from);
        let destination = self.root.join(to);
        if source == destination {
            return Ok(());
        }
        if destination.starts_with(&source) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "destination is inside source folder",
            ));
        }
        if destination.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("{} already exists", destination.display()),
            ));
        }
        let parent = destination.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "destination has no parent")
        })?;
        if let Err(error) = fs::create_dir_all(parent) {
            remove_empty_parents(parent, &self.root);
            return Err(named(parent, &error));
        }
        let root = self
            .root
            .canonicalize()
            .map_err(|error| named(&self.root, &error))?;
        let source_real = match source.canonicalize() {
            Ok(path) => path,
            Err(error) => {
                remove_empty_parents(parent, &self.root);
                return Err(named(&source, &error));
            }
        };
        let parent_real = match parent.canonicalize() {
            Ok(path) => path,
            Err(error) => {
                remove_empty_parents(parent, &self.root);
                return Err(named(parent, &error));
            }
        };
        if !source_real.starts_with(&root) || !parent_real.starts_with(&root) {
            remove_empty_parents(parent, &self.root);
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "source and destination must stay inside the library folder",
            ));
        }
        fs::rename(&source, &destination).map_err(|error| {
            remove_empty_parents(parent, &self.root);
            named(&source, &error)
        })
    }

    /// Move or copy `source` into `<root>/<folder>/<name>`. Gives the
    /// size and the SHA-256 of the file (ADR 0036 item 4).
    ///
    /// # Errors
    ///
    /// Gives an `io::Error` when `name` is not a plain file name, or
    /// Studio cannot make the release folder or read or write the file.
    pub fn put_file(&self, folder: &str, name: &str, source: &Path) -> io::Result<(u64, String)> {
        check_folder_path(folder)?;
        check_plain(name)?;
        let dir = self.root.join(folder);
        fs::create_dir_all(&dir).map_err(|e| named(&dir, &e))?;
        let full = dir.join(name);
        let bytes = fs::read(source).map_err(|e| named(source, &e))?;
        let size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        let sha256 = sha256_hex(&[&bytes]);
        fs::write(&full, &bytes).map_err(|e| named(&full, &e))?;
        // The source file does not stay at its old place after the move. A
        // source that the computer cannot remove stays there with no harm.
        // The copy in the library is already in place.
        let _ = fs::remove_file(source);
        Ok((size, sha256))
    }

    /// Write the feed XML of a release, through a temporary file and a
    /// rename, so a reader never sees a half-written file (ADR 0036 item
    /// 3).
    ///
    /// # Errors
    ///
    /// Gives an `io::Error` when Studio cannot make the release folder, or
    /// the write or the rename fails.
    pub fn write_feed_xml(&self, folder: &str, file_name: &str, bytes: &[u8]) -> io::Result<()> {
        check_folder_path(folder)?;
        check_plain(file_name)?;
        let dir = self.root.join(folder);
        fs::create_dir_all(&dir).map_err(|e| named(&dir, &e))?;
        let full = dir.join(file_name);
        let mut temporary = full.as_os_str().to_owned();
        temporary.push(".studio-tmp");
        let temporary = PathBuf::from(temporary);
        let mut file = fs::File::create(&temporary).map_err(|e| named(&temporary, &e))?;
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|e| named(&temporary, &e))?;
        drop(file);
        fs::rename(&temporary, &full).map_err(|e| named(&full, &e))?;
        Ok(())
    }

    /// Read the feed XML of a release folder. This is the reverse of
    /// [`Library::write_feed_xml`] (ADR 0067 item 5).
    ///
    /// # Errors
    ///
    /// Gives an `io::Error` when `folder` or `file_name` is not a plain
    /// name, or Studio cannot read the file.
    pub fn read_feed_xml(&self, folder: &str, file_name: &str) -> io::Result<Vec<u8>> {
        check_folder_path(folder)?;
        check_plain(file_name)?;
        let full = self.root.join(folder).join(file_name);
        fs::read(&full).map_err(|e| named(&full, &e))
    }

    /// Remove a file of a release folder. A file that does not exist is
    /// not an error.
    ///
    /// # Errors
    ///
    /// Gives an `io::Error` when `name` is not a plain file name, or
    /// Studio cannot remove the file for another reason.
    pub fn remove_file(&self, folder: &str, name: &str) -> io::Result<()> {
        check_folder_path(folder)?;
        check_plain(name)?;
        let full = self.root.join(folder).join(name);
        match fs::remove_file(&full) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(named(&full, &e)),
        }
    }
}

/// The stamp of a file: its size and its modification time in
/// nanoseconds, as `{size}-{mtime_ns}`. The local folder connector, the
/// import, the download and the folder scan all use this one function, so
/// that a stamp from one equals a stamp from another (ADR 0038 item 13).
#[must_use]
pub fn stamp_of(meta: &fs::Metadata) -> String {
    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos());
    format!("{}-{modified}", meta.len())
}

/// The stamp of the file at `path`. Gives `None` when the file has no
/// metadata.
#[must_use]
pub fn file_stamp(path: &Path) -> Option<String> {
    fs::metadata(path).ok().map(|meta| stamp_of(&meta))
}

/// Gives an error when `name` is not a plain file name or folder name, so
/// that no path goes out of the library.
fn check_plain(name: &str) -> io::Result<()> {
    if is_plain_name(name) {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name:?} is not a plain file name"),
        ))
    }
}

fn check_folder_path(path: &str) -> io::Result<()> {
    if Library::is_folder_path(path) {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{path:?} is not a library folder path"),
        ))
    }
}

/// True when `name` is a plain file name: no path separator, and not `.`
/// or `..` (ADR 0036 item 4).
pub(crate) fn is_plain_name(name: &str) -> bool {
    !name.is_empty() && !name.contains(['/', '\\']) && name != "." && name != ".."
}

/// Wrap an I/O error so its message names the path that failed.
fn named(path: &Path, error: &io::Error) -> io::Error {
    io::Error::new(error.kind(), format!("{}: {error}", path.display()))
}

fn remove_empty_parents(mut path: &Path, root: &Path) {
    while path != root && path.starts_with(root) {
        if fs::remove_dir(path).is_err() {
            break;
        }
        let Some(parent) = path.parent() else {
            break;
        };
        path = parent;
    }
}

/// Give a release folder the name of its feed file with no extension.
/// When that name is in use, add ` 2`, ` 3` and so on (ADR 0035 item 4,
/// ADR 0036 item 2).
#[must_use]
pub fn release_folder_name(feed_path: &str, taken: &dyn Fn(&str) -> bool) -> String {
    let file = feed_path.rsplit_once('/').map_or(feed_path, |(_, f)| f);
    let base = file.rsplit_once('.').map_or(file, |(stem, _)| stem);
    if !taken(base) {
        return base.to_owned();
    }
    let mut n = 2;
    loop {
        let candidate = format!("{base} {n}");
        if !taken(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

/// Give a release its default relative folder path (ADR 0167 items 1 to 3).
#[must_use]
pub fn default_release_folder(
    title: Option<&str>,
    artist: Option<&str>,
    label: Option<&str>,
    taken: &dyn Fn(&str) -> bool,
) -> String {
    let mut parts = Vec::new();
    if let Some(artist) = artist {
        if let Some(label) = label {
            parts.push(folder_part(label));
        }
        parts.push(folder_part(artist));
    } else {
        parts.push("Other".to_owned());
    }
    parts.push(folder_part(title.unwrap_or_default()));
    let base = parts.join("/");
    if !taken(&base) {
        return base;
    }
    let (parent, name) = base.rsplit_once('/').unwrap_or(("", base.as_str()));
    for number in 2_u64.. {
        let name = format!("{name} {number}");
        let candidate = if parent.is_empty() {
            name
        } else {
            format!("{parent}/{name}")
        };
        if !taken(&candidate) {
            return candidate;
        }
    }
    base
}

fn folder_part(value: &str) -> String {
    let safe: String = value
        .chars()
        .filter(|character| {
            !character.is_control()
                && !matches!(
                    character,
                    '/' | '\\' | '<' | '>' | ':' | '"' | '|' | '?' | '*'
                )
        })
        .collect();
    let safe = safe.trim();
    if safe.is_empty() || matches!(safe, "." | "..") {
        "Untitled".into()
    } else {
        safe.into()
    }
}

/// The error of [`give_published_folder`].
#[derive(Debug)]
pub enum FolderError {
    /// The collection database failed.
    Collection(CollectionError),
    /// Studio could not make the release folder or write the feed XML.
    Io(io::Error),
    /// The feed has no address and no path to name its folder from.
    NoAddress,
    /// The feed has no library folder.
    NoFolder,
}

impl std::fmt::Display for FolderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Collection(error) => error.fmt(f),
            Self::Io(error) => error.fmt(f),
            Self::NoAddress => f.write_str("the feed has no address to name its folder from"),
            Self::NoFolder => f.write_str("the feed has no library folder"),
        }
    }
}

impl std::error::Error for FolderError {}

impl From<CollectionError> for FolderError {
    fn from(error: CollectionError) -> Self {
        Self::Collection(error)
    }
}

impl From<io::Error> for FolderError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Move a release folder and record its new relative path.
///
/// # Errors
///
/// Gives an error when the feed has no folder, the move fails, or SQLite fails.
pub fn move_release_folder(
    collection: &Collection,
    library: &Library,
    feed_id: FeedId,
    destination: &str,
) -> Result<(), FolderError> {
    let row = collection.feed(feed_id)?;
    let source = row.folder.ok_or(FolderError::NoFolder)?;
    if source == destination {
        return Ok(());
    }
    library.move_folder(&source, destination)?;
    if let Err(error) = collection.set_feed_folder(feed_id, destination) {
        let _ = library.move_folder(destination, &source);
        return Err(FolderError::Collection(error));
    }
    Ok(())
}

/// Give a published feed its release folder, with its feed XML
/// (ADR 0038 item 4, ADR 0035 item 5).
///
/// The function names the folder from the feed address, or from the feed
/// path when the feed has no address, with [`release_folder_name`]. It
/// writes `bytes` as the feed XML, then records the folder in the
/// collection. A failure before the record leaves the feed with no folder.
///
/// # Errors
///
/// Gives a [`FolderError`] when the feed has no address and no path, when
/// Studio cannot write the folder, or when SQLite fails.
pub fn give_published_folder(
    collection: &Collection,
    library: &Library,
    feed_id: FeedId,
    bytes: &[u8],
) -> Result<String, FolderError> {
    let row = collection.feed(feed_id)?;
    let source = row
        .public_url
        .as_deref()
        .or(row.path.as_deref())
        .ok_or(FolderError::NoAddress)?;
    let feeds = collection.feeds()?;
    let models: Vec<_> = feeds
        .iter()
        .map(|feed| {
            let model = crate::pipeline::draft_model(collection, feed.id)
                .ok()
                .map(|(model, _)| model);
            (feed.clone(), model)
        })
        .collect();
    let incoming = crate::xml::Document::parse_bytes(bytes)
        .ok()
        .and_then(|document| crate::feed::Feed::read(&document).ok());
    let mut models = models;
    if let Some((_, model)) = models.iter_mut().find(|(feed, _)| feed.id == feed_id)
        && incoming.is_some()
    {
        model.clone_from(&incoming);
    }
    let (artist, label) =
        crate::publisher::release_levels(&models, feed_id).unwrap_or((None, None));
    let title = incoming
        .as_ref()
        .and_then(|feed| feed.title.as_deref())
        .or(row.title.as_deref());
    let taken: std::collections::HashSet<String> = feeds
        .iter()
        .filter_map(|feed| feed.folder.clone())
        .collect();
    let root = library.root();
    let folder = default_release_folder(title, artist.as_deref(), label.as_deref(), &|path| {
        taken.contains(path) || root.join(path).exists()
    });
    let file_name = source.rsplit_once('/').map_or(source, |(_, name)| name);
    library.write_feed_xml(&folder, file_name, bytes)?;
    collection.set_feed_folder(feed_id, &folder)?;
    Ok(folder)
}

/// The default library path, `~/musicindex/studio` (ADR 0036 item 1).
///
/// Gives `None` when the system has no home folder for this user.
#[must_use]
pub fn default_root() -> Option<PathBuf> {
    let home = directories::BaseDirs::new()?;
    Some(home.home_dir().join("musicindex").join("studio"))
}

/// The default folder of the pages on a file host, `~/musicindex/pages`:
/// the folder `pages` in the parent folder of [`default_root`] (ADR 0122
/// item 2).
///
/// Gives `None` when the system has no home folder for this user.
#[must_use]
pub fn default_host_root() -> Option<PathBuf> {
    Some(default_root()?.parent()?.join("pages"))
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn default_release_folder_uses_artist_label_other_and_unique_names() {
        let unused = |_: &str| false;
        assert_eq!(
            default_release_folder(Some("Album"), Some("Artist"), None, &unused),
            "Artist/Album"
        );
        assert_eq!(
            default_release_folder(Some("Album"), Some("Artist"), Some("Label"), &unused),
            "Label/Artist/Album"
        );
        assert_eq!(
            default_release_folder(Some("Album"), None, Some("Label"), &unused),
            "Other/Album"
        );
        let taken = |path: &str| matches!(path, "Artist/Album" | "Artist/Album 2");
        assert_eq!(
            default_release_folder(Some("Album"), Some("Artist"), None, &taken),
            "Artist/Album 3"
        );
        assert_eq!(
            default_release_folder(Some("/"), Some("Artist/"), None, &unused),
            "Artist/Untitled"
        );
    }

    #[test]
    fn library_folder_move_keeps_files_records_and_feed_xml_name() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("library");
        let library = Library::open(&root).unwrap();
        let mut collection = Collection::open_in_memory().unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        let source = dir.path().join("track.mp3");
        fs::write(&source, b"audio").unwrap();
        collection.set_feed_folder(feed, "Artist/Album").unwrap();
        library
            .write_feed_xml("Artist/Album", "feed.custom.xml", b"xml")
            .unwrap();
        let (size, sha) = library
            .put_file("Artist/Album", "track.mp3", &source)
            .unwrap();
        collection
            .add_library_file(
                feed,
                "track.mp3",
                size,
                Some(&sha),
                "audio/mpeg",
                true,
                None,
                None,
            )
            .unwrap();
        let before = collection.feed(feed).unwrap();

        move_release_folder(&collection, &library, feed, "Label/Artist/Album").unwrap();
        let after = collection.feed(feed).unwrap();
        assert_eq!(after.folder.as_deref(), Some("Label/Artist/Album"));
        assert_eq!(after.path, before.path);
        assert_eq!(after.public_url, before.public_url);
        assert_eq!(after.connector_id, before.connector_id);
        assert_eq!(
            fs::read(root.join("Label/Artist/Album/track.mp3")).unwrap(),
            b"audio"
        );
        assert_eq!(
            fs::read(root.join("Label/Artist/Album/feed.custom.xml")).unwrap(),
            b"xml"
        );
        assert_eq!(collection.library_files(feed).unwrap()[0].path, "track.mp3");

        for invalid in ["../outside", root.to_str().unwrap()] {
            assert!(move_release_folder(&collection, &library, feed, invalid).is_err());
            assert_eq!(
                collection.feed(feed).unwrap().folder.as_deref(),
                Some("Label/Artist/Album")
            );
        }
        fs::create_dir_all(root.join("Occupied")).unwrap();
        assert!(move_release_folder(&collection, &library, feed, "Occupied").is_err());
        assert_eq!(
            collection.feed(feed).unwrap().folder.as_deref(),
            Some("Label/Artist/Album")
        );
        assert!(root.join("Label/Artist/Album/track.mp3").exists());
    }

    #[test]
    fn opening_a_library_does_not_move_an_existing_release_folder() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("library");
        let old = root.join("feed");
        fs::create_dir_all(&old).unwrap();
        fs::write(old.join("feed.xml"), b"xml").unwrap();
        let collection_path = dir.path().join("collection.sqlite");
        let mut collection = Collection::open(&collection_path).unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        collection.set_feed_folder(feed, "feed").unwrap();
        drop(collection);
        let collection = Collection::open(&collection_path).unwrap();
        let _library = Library::open(&root).unwrap();
        assert_eq!(
            collection.feed(feed).unwrap().folder.as_deref(),
            Some("feed")
        );
        assert!(old.join("feed.xml").exists());
        assert!(!root.join("Other").exists());
    }

    #[test]
    fn open_makes_the_folder_and_removes_the_test_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("library");
        let library = Library::open(&root).unwrap();
        assert_eq!(library.root(), root);
        assert!(root.is_dir());
        assert!(!root.join(WRITE_TEST_FILE).exists());
    }

    #[test]
    fn library_folders_lists_relative_paths_through_three_levels() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("library");
        for path in [
            "Artist/Album/Disc",
            "Artist/Album/Disc/Deep",
            "Label/Release",
        ] {
            fs::create_dir_all(root.join(path)).unwrap();
        }
        let library = Library::open(&root).unwrap();

        assert_eq!(
            library.folders().unwrap(),
            [
                "Artist",
                "Artist/Album",
                "Artist/Album/Disc",
                "Label",
                "Label/Release"
            ]
        );
    }

    #[test]
    fn open_of_a_path_in_a_folder_with_no_write_permission_names_the_path() {
        let dir = tempfile::tempdir().unwrap();
        let parent = dir.path().join("locked");
        fs::create_dir_all(&parent).unwrap();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o500)).unwrap();

        let root = parent.join("library");
        let error = Library::open(&root).unwrap_err();
        assert!(error.to_string().contains(root.to_str().unwrap()));

        // Restore the permission, so the test folder can be removed.
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
    }

    #[test]
    fn put_file_and_write_feed_xml_refuse_a_folder_out_of_the_library() {
        let dir = tempfile::tempdir().unwrap();
        let library = Library::open(dir.path().join("library")).unwrap();
        let source = dir.path().join("a.mp3");
        fs::write(&source, b"music").unwrap();
        assert!(library.put_file("..", "a.mp3", &source).is_err());
        assert!(library.write_feed_xml("..", "feed.xml", b"x").is_err());
        assert!(!dir.path().join("a.mp3.studio-tmp").exists());
    }

    #[test]
    fn put_file_gives_the_size_and_the_sha256_and_moves_the_source() {
        let dir = tempfile::tempdir().unwrap();
        let library = Library::open(dir.path().join("library")).unwrap();
        let source_dir = tempfile::tempdir().unwrap();
        let source = source_dir.path().join("a.mp3");
        fs::write(&source, b"music").unwrap();

        let (size, sha256) = library
            .put_file("my-album", "01 Song.mp3", &source)
            .unwrap();
        assert_eq!(size, 5);
        assert_eq!(sha256, sha256_hex(&[b"music"]));
        assert!(!source.exists());
        assert_eq!(
            fs::read(library.path("my-album", "01 Song.mp3")).unwrap(),
            b"music"
        );
    }

    #[test]
    fn put_file_refuses_a_name_that_is_not_a_plain_file_name() {
        let dir = tempfile::tempdir().unwrap();
        let library = Library::open(dir.path().join("library")).unwrap();
        let source_dir = tempfile::tempdir().unwrap();
        let source = source_dir.path().join("a.mp3");
        fs::write(&source, b"music").unwrap();

        let error = library.put_file("my-album", "../x", &source).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(source.exists());
        assert!(!dir.path().join("x").exists());
    }

    #[test]
    fn write_feed_xml_writes_the_bytes_through_a_rename() {
        let dir = tempfile::tempdir().unwrap();
        let library = Library::open(dir.path().join("library")).unwrap();
        library
            .write_feed_xml("my-album", "my-album.xml", b"<rss/>")
            .unwrap();
        assert_eq!(
            fs::read(library.path("my-album", "my-album.xml")).unwrap(),
            b"<rss/>"
        );
        assert!(!library.path("my-album", "my-album.xml.studio-tmp").exists());
    }

    #[test]
    fn remove_file_removes_the_file_and_a_missing_file_is_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let library = Library::open(dir.path().join("library")).unwrap();
        let source_dir = tempfile::tempdir().unwrap();
        let source = source_dir.path().join("a.mp3");
        fs::write(&source, b"music").unwrap();
        library.put_file("my-album", "a.mp3", &source).unwrap();
        assert!(library.path("my-album", "a.mp3").exists());

        library.remove_file("my-album", "a.mp3").unwrap();
        assert!(!library.path("my-album", "a.mp3").exists());
        library.remove_file("my-album", "a.mp3").unwrap();
    }

    #[test]
    fn release_folder_name_gives_the_file_stem_then_a_number_when_taken() {
        assert_eq!(
            release_folder_name("album/my-album.xml", &|_| false),
            "my-album"
        );
        assert_eq!(
            release_folder_name("album/my-album.xml", &|n| n == "my-album"),
            "my-album 2"
        );
        assert_eq!(
            release_folder_name("my-album.xml", &|n| {
                n == "my-album" || n == "my-album 2"
            }),
            "my-album 3"
        );
    }
}

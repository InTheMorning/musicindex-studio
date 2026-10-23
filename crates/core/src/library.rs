//! The library folder on the user's machine (ADR 0035, ADR 0036).
//!
//! Studio keeps a release folder for each feed in the library: the feed
//! XML and each file of the release. The collection keeps the records of
//! each file (`collection::LibraryFile`). This module keeps the bytes.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

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

    /// The path of a file in a release folder.
    #[must_use]
    pub fn path(&self, folder: &str, name: &str) -> PathBuf {
        self.root.join(folder).join(name)
    }

    /// Move or copy `source` into `<root>/<folder>/<name>`. Gives the
    /// size and the SHA-256 of the file (ADR 0036 item 4).
    ///
    /// # Errors
    ///
    /// Gives an `io::Error` when `name` is not a plain file name, or
    /// Studio cannot make the release folder or read or write the file.
    pub fn put_file(&self, folder: &str, name: &str, source: &Path) -> io::Result<(u64, String)> {
        check_plain(folder)?;
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
        check_plain(folder)?;
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

/// True when `name` is a plain file name: no path separator, and not `.`
/// or `..` (ADR 0036 item 4).
fn is_plain_name(name: &str) -> bool {
    !name.is_empty() && !name.contains(['/', '\\']) && name != "." && name != ".."
}

/// Wrap an I/O error so its message names the path that failed.
fn named(path: &Path, error: &io::Error) -> io::Error {
    io::Error::new(error.kind(), format!("{}: {error}", path.display()))
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

/// The default library path, `~/musicindex/studio` (ADR 0036 item 1).
///
/// Gives `None` when the `HOME` variable is not set.
#[must_use]
pub fn default_root() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join("musicindex").join("studio"))
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

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
        assert!(library.put_file("a/b", "a.mp3", &source).is_err());
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

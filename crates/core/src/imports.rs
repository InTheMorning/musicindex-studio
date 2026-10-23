//! The pending files of a draft, in the data folder (ADR 0026, ADR 0032).
//!
//! An import keeps a file here until the write puts it on the host, or the
//! draft removes it. [`Collection`](crate::collection::Collection) keeps the
//! row of each pending file. This store keeps its bytes.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::collection::FeedId;
use crate::digest::sha256_hex;

/// The pending files of each feed, at `root/<feed id>/<sha256>`
/// (ADR 0026 item 5).
#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// Use this folder for the pending files.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The folder of this store.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn feed_dir(&self, feed: FeedId) -> PathBuf {
        self.root.join(feed.0.to_string())
    }

    /// Move or copy `source` into the pending files of `feed`. Gives its
    /// SHA-256 and its size.
    ///
    /// # Errors
    ///
    /// Gives an `io::Error` when the file cannot be read or written.
    pub fn put_file(&self, feed: FeedId, source: &Path) -> io::Result<(String, u64)> {
        let bytes = fs::read(source)?;
        let size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        let sha256 = sha256_hex(&[&bytes]);
        let dir = self.feed_dir(feed);
        fs::create_dir_all(&dir)?;
        fs::write(dir.join(&sha256), &bytes)?;
        // The source file does not stay at its old place after the move. A
        // source that the computer cannot remove stays there with no harm.
        // The copy in the store is already in place.
        let _ = fs::remove_file(source);
        Ok((sha256, size))
    }

    /// The bytes of a pending file.
    ///
    /// # Errors
    ///
    /// Gives an `io::Error` when the file does not exist or cannot be read.
    pub fn open(&self, feed: FeedId, sha256: &str) -> io::Result<Vec<u8>> {
        fs::read(self.feed_dir(feed).join(sha256))
    }

    /// Remove one pending file. A file that does not exist is not an error.
    pub fn remove(&self, feed: FeedId, sha256: &str) {
        let _ = fs::remove_file(self.feed_dir(feed).join(sha256));
    }

    /// Remove each pending file of a feed. A feed with no pending files is
    /// not an error (ADR 0026 item 8, ADR 0032 item 12).
    pub fn remove_feed(&self, feed: FeedId) {
        let _ = fs::remove_dir_all(self.feed_dir(feed));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_file_moves_the_source_and_open_gives_its_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().join("imports"));
        let source_dir = tempfile::tempdir().unwrap();
        let source = source_dir.path().join("a.mp3");
        fs::write(&source, b"music").unwrap();

        let (sha256, size) = store.put_file(FeedId(1), &source).unwrap();
        assert_eq!(size, 5);
        assert!(!source.exists());
        assert_eq!(store.open(FeedId(1), &sha256).unwrap(), b"music");

        store.remove(FeedId(1), &sha256);
        assert!(store.open(FeedId(1), &sha256).is_err());
    }

    #[test]
    fn remove_feed_removes_each_pending_file_of_the_feed() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().join("imports"));
        let source_dir = tempfile::tempdir().unwrap();
        let a = source_dir.path().join("a.mp3");
        let b = source_dir.path().join("b.mp3");
        fs::write(&a, b"one").unwrap();
        fs::write(&b, b"two").unwrap();
        let (sha_a, _) = store.put_file(FeedId(1), &a).unwrap();
        let (sha_b, _) = store.put_file(FeedId(1), &b).unwrap();

        store.remove_feed(FeedId(1));
        assert!(store.open(FeedId(1), &sha_a).is_err());
        assert!(store.open(FeedId(1), &sha_b).is_err());
    }
}

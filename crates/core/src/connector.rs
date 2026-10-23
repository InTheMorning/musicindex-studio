//! Connectors: read and write the files of one host type (ADR 0006, ADR 0015).

mod local;

use std::fmt;
use std::path::{Component, Path, PathBuf};

pub use local::LocalFolder;

use crate::collection::ConnectorRow;
use crate::vault::OpenVault;

/// A version marker of a file on a host. Each connector selects its form:
/// a content hash, an ETag or a commit ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct Version(String);

impl Version {
    /// Make a version marker from its text.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// The text of the marker.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The state that a write expects the file on the host to have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expected {
    /// The file does not exist.
    Absent,
    /// The file has this version.
    Version(Version),
}

/// A file and its version, as a connector read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRead {
    /// The bytes of the file.
    pub bytes: Vec<u8>,
    /// The version marker of the file.
    pub version: Version,
}

/// The limits of a file list (ADR 0019).
pub const LIST_MAX_DEPTH: usize = 4;
/// The maximum number of files in a list (ADR 0019).
pub const LIST_MAX_FILES: usize = 1000;

/// A change marker of a file that a list gives with no read of the file,
/// for example the size and the modification time, or an `ETag`. Two
/// different stamps of one path mean that the file changed (ADR 0036
/// item 6).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct Stamp(String);

impl Stamp {
    /// Make a stamp from its text.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// The stamp as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One file of a host list, with the facts that need no read of the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedFile {
    /// The relative path, with `/` between segments.
    pub path: String,
    /// The size of the file, in bytes.
    pub size: u64,
    /// The change marker of the file.
    pub stamp: Stamp,
}

/// The files of a host, relative to the connector root.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileList {
    /// The files, in the sorted sequence of their paths.
    pub files: Vec<ListedFile>,
    /// True when a limit stopped the list before the end.
    pub truncated: bool,
}

/// The kind of a [`ConnectorError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConnectorErrorKind {
    /// The file on the host does not have the expected version.
    Conflict,
    /// The file on the host does not exist.
    NotFound,
    /// The path is not a relative path in the connector.
    InvalidPath,
    /// The connector configuration is not valid, or the kind is not known.
    InvalidConfig,
    /// A file or network operation failed.
    Io,
}

/// An error from a connector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectorError {
    kind: ConnectorErrorKind,
    message: String,
}

impl ConnectorError {
    pub(crate) fn new(kind: ConnectorErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// The kind of the error.
    #[must_use]
    pub fn kind(&self) -> ConnectorErrorKind {
        self.kind
    }
}

impl fmt::Display for ConnectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ConnectorError {}

/// Read and write the files of one host.
///
/// The functions block. The server calls them from a blocking task
/// (ADR 0015).
pub trait Connector {
    /// Read a file. Gives `None` when the file does not exist.
    ///
    /// # Errors
    ///
    /// Gives a [`ConnectorError`] when the path is not valid or the read fails.
    fn read(&self, path: &str) -> Result<Option<FileRead>, ConnectorError>;

    /// Write a file only when the file on the host is as `expected`.
    /// Gives the new version marker.
    ///
    /// # Errors
    ///
    /// Gives [`ConnectorErrorKind::Conflict`] when the file on the host is
    /// not as `expected`. The file then does not change.
    fn write(
        &self,
        path: &str,
        bytes: &[u8],
        expected: &Expected,
    ) -> Result<Version, ConnectorError>;

    /// The public URL of a file, when the connector knows it.
    fn public_url(&self, path: &str) -> Option<String>;

    /// List the files of the host, with the limits [`LIST_MAX_DEPTH`] and
    /// [`LIST_MAX_FILES`]. Hidden folders are not in the list (ADR 0019).
    ///
    /// # Errors
    ///
    /// Gives a [`ConnectorError`] when the list fails.
    fn list(&self) -> Result<FileList, ConnectorError>;

    /// The size and the stamp of one file, with no read of the file.
    /// Gives `None` when the file does not exist (ADR 0036 item 6).
    ///
    /// # Errors
    ///
    /// Gives a [`ConnectorError`] when the path is not valid or the host
    /// fails.
    fn stat(&self, path: &str) -> Result<Option<ListedFile>, ConnectorError>;

    /// Delete a file only when the file on the host has the expected
    /// stamp. Studio reads the stamp from its inventory, so it does not
    /// delete a file that changed after the scan (ADR 0036 item 16).
    ///
    /// # Errors
    ///
    /// Gives [`ConnectorErrorKind::Conflict`] when the file on the host has
    /// a different stamp, and [`ConnectorErrorKind::NotFound`] when the
    /// file does not exist.
    fn delete(&self, path: &str, expected: &Stamp) -> Result<(), ConnectorError>;
}

/// Make the connector of a collection row.
///
/// The vault gives the secret of a connector that needs one.
///
/// # Errors
///
/// Gives [`ConnectorErrorKind::InvalidConfig`] when the kind is not known or
/// the configuration is not valid.
pub fn from_row(
    row: &ConnectorRow,
    _vault: Option<&OpenVault>,
) -> Result<Box<dyn Connector>, ConnectorError> {
    match row.kind.as_str() {
        LocalFolder::KIND => Ok(Box::new(LocalFolder::from_config(&row.config)?)),
        other => Err(ConnectorError::new(
            ConnectorErrorKind::InvalidConfig,
            format!("the connector kind {other:?} is not known"),
        )),
    }
}

/// Check that `path` is a relative path that stays in its root.
///
/// The function accepts only normal path segments: no root, no prefix, no
/// `.` and no `..`.
pub(crate) fn relative_path(path: &str) -> Result<PathBuf, ConnectorError> {
    let invalid = || {
        ConnectorError::new(
            ConnectorErrorKind::InvalidPath,
            format!("the path {path:?} is not a relative path in the connector"),
        )
    };
    if path.is_empty() || path.contains('\\') {
        return Err(invalid());
    }
    let parsed = Path::new(path);
    if !parsed
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
    {
        return Err(invalid());
    }
    Ok(parsed.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_path_accepts_only_normal_segments() {
        assert!(relative_path("feeds/album.xml").is_ok());
        for bad in [
            "",
            "/etc/passwd",
            "../x.xml",
            "a/../../x.xml",
            "./a.xml",
            "a\\b.xml",
        ] {
            assert_eq!(
                relative_path(bad).unwrap_err().kind(),
                ConnectorErrorKind::InvalidPath,
                "{bad}"
            );
        }
    }
}

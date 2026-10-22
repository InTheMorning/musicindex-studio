//! The local folder connector (ADR 0006, ADR 0015).

use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::digest::content_version;

use super::{
    Connector, ConnectorError, ConnectorErrorKind, Expected, FileList, FileRead, LIST_MAX_DEPTH,
    LIST_MAX_FILES, Version, relative_path,
};

/// A folder on the machine that runs Studio.
///
/// The version marker is the SHA-256 of the file content. A write goes to a
/// temporary file and then a rename. The connector cannot stop a change that
/// occurs between its version check and its rename (ADR 0015).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalFolder {
    root: PathBuf,
    base_url: Option<String>,
    base_url_inferred: bool,
}

#[derive(Deserialize)]
struct Config {
    root: PathBuf,
    base_url: Option<String>,
    #[serde(default)]
    base_url_inferred: bool,
}

impl LocalFolder {
    /// The connector kind in the collection.
    pub const KIND: &'static str = "local";

    /// Make a connector for a folder.
    ///
    /// `base_url` is the public URL of the folder, when a web server serves it.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, base_url: Option<String>) -> Self {
        Self {
            root: root.into(),
            base_url,
            base_url_inferred: false,
        }
    }

    /// The same folder with a public URL that Studio found from the feeds
    /// in it (ADR 0021 item 7).
    #[must_use]
    pub fn with_inferred_base_url(self, base_url: String) -> Self {
        Self {
            base_url: Some(base_url),
            base_url_inferred: true,
            ..self
        }
    }

    /// The public URL of the folder.
    #[must_use]
    pub fn base_url(&self) -> Option<&str> {
        self.base_url.as_deref()
    }

    /// True when Studio found the public URL from the feeds, and the user
    /// did not type it.
    #[must_use]
    pub fn base_url_inferred(&self) -> bool {
        self.base_url_inferred
    }

    /// The configuration text for the collection.
    ///
    /// # Panics
    ///
    /// Panics when the root path is not UTF-8.
    #[must_use]
    pub fn config(&self) -> String {
        serde_json::json!({
            "root": self.root.to_str().expect("the folder path is UTF-8"),
            "base_url": self.base_url,
            "base_url_inferred": self.base_url_inferred,
        })
        .to_string()
    }

    pub(crate) fn from_config(config: &str) -> Result<Self, ConnectorError> {
        let config: Config = serde_json::from_str(config)
            .map_err(|e| ConnectorError::new(ConnectorErrorKind::InvalidConfig, e.to_string()))?;
        Ok(Self {
            root: config.root,
            base_url: config.base_url,
            base_url_inferred: config.base_url_inferred,
        })
    }

    fn full_path(&self, path: &str) -> Result<PathBuf, ConnectorError> {
        Ok(self.root.join(relative_path(path)?))
    }
}

fn io_error(path: &Path, error: &std::io::Error) -> ConnectorError {
    ConnectorError::new(
        ConnectorErrorKind::Io,
        format!("{}: {error}", path.display()),
    )
}

fn version_of(bytes: &[u8]) -> Version {
    content_version(bytes)
}

impl Connector for LocalFolder {
    fn read(&self, path: &str) -> Result<Option<FileRead>, ConnectorError> {
        let full = self.full_path(path)?;
        match fs::read(&full) {
            Ok(bytes) => {
                let version = version_of(&bytes);
                Ok(Some(FileRead { bytes, version }))
            }
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(io_error(&full, &e)),
        }
    }

    fn write(
        &self,
        path: &str,
        bytes: &[u8],
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        let full = self.full_path(path)?;
        let current = self.read(path)?.map(|f| f.version);
        let matches = match (expected, &current) {
            (Expected::Absent, None) => true,
            (Expected::Version(want), Some(have)) => want == have,
            _ => false,
        };
        if !matches {
            return Err(ConnectorError::new(
                ConnectorErrorKind::Conflict,
                format!("{} changed on the host", full.display()),
            ));
        }
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).map_err(|e| io_error(parent, &e))?;
        }
        let mut temporary = full.as_os_str().to_owned();
        temporary.push(".studio-tmp");
        let temporary = PathBuf::from(temporary);
        let mut file = fs::File::create(&temporary).map_err(|e| io_error(&temporary, &e))?;
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|e| io_error(&temporary, &e))?;
        drop(file);
        fs::rename(&temporary, &full).map_err(|e| io_error(&full, &e))?;
        Ok(version_of(bytes))
    }

    fn public_url(&self, path: &str) -> Option<String> {
        let base = self.base_url.as_deref()?;
        Some(format!("{}/{}", base.trim_end_matches('/'), path))
    }

    fn list(&self) -> Result<FileList, ConnectorError> {
        let mut list = FileList::default();
        let mut folders = vec![(self.root.clone(), String::new(), 0_usize)];
        while let Some((folder, prefix, depth)) = folders.pop() {
            let entries = fs::read_dir(&folder).map_err(|e| io_error(&folder, &e))?;
            for entry in entries {
                let entry = entry.map_err(|e| io_error(&folder, &e))?;
                let Ok(name) = entry.file_name().into_string() else {
                    continue;
                };
                if name.starts_with('.') {
                    continue;
                }
                let path = format!("{prefix}{name}");
                let kind = entry.file_type().map_err(|e| io_error(&entry.path(), &e))?;
                if kind.is_dir() {
                    if depth + 1 < LIST_MAX_DEPTH {
                        folders.push((entry.path(), format!("{path}/"), depth + 1));
                    } else {
                        list.truncated = true;
                    }
                } else if kind.is_file() {
                    if list.paths.len() == LIST_MAX_FILES {
                        list.truncated = true;
                        list.paths.sort();
                        return Ok(list);
                    }
                    list.paths.push(path);
                }
            }
        }
        list.paths.sort();
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_reads_with_its_version_and_writes_only_with_the_expected_version() {
        let dir = tempfile::tempdir().unwrap();
        let folder = LocalFolder::new(dir.path(), Some("https://example.com/feeds/".into()));
        assert_eq!(folder.read("a/feed.xml").unwrap(), None);

        let v1 = folder
            .write("a/feed.xml", b"one", &Expected::Absent)
            .unwrap();
        let read = folder.read("a/feed.xml").unwrap().unwrap();
        assert_eq!(read.bytes, b"one");
        assert_eq!(read.version, v1);

        let error = folder
            .write("a/feed.xml", b"two", &Expected::Absent)
            .unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Conflict);

        let stale = Version::new("sha256:0");
        let error = folder
            .write("a/feed.xml", b"two", &Expected::Version(stale))
            .unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::Conflict);
        assert_eq!(fs::read(dir.path().join("a/feed.xml")).unwrap(), b"one");

        let v2 = folder
            .write("a/feed.xml", b"two", &Expected::Version(v1))
            .unwrap();
        assert_eq!(folder.read("a/feed.xml").unwrap().unwrap().version, v2);
        assert!(!dir.path().join("a/feed.xml.studio-tmp").exists());
        assert_eq!(
            folder.public_url("a/feed.xml").as_deref(),
            Some("https://example.com/feeds/a/feed.xml")
        );
    }

    #[test]
    fn a_path_out_of_the_folder_gives_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let folder = LocalFolder::new(dir.path().join("inner"), None);
        let error = folder
            .write("../escape.xml", b"x", &Expected::Absent)
            .unwrap_err();
        assert_eq!(error.kind(), ConnectorErrorKind::InvalidPath);
        assert!(!dir.path().join("escape.xml").exists());
    }

    #[test]
    fn the_list_has_the_files_without_hidden_folders_and_with_a_depth_limit() {
        let dir = tempfile::tempdir().unwrap();
        let folder = LocalFolder::new(dir.path(), None);
        for path in [
            "b.xml",
            "album/feed.xml",
            ".git/config",
            "a/b/c/d/deep.xml",
            "a/b/c/ok.xml",
        ] {
            folder.write(path, b"x", &Expected::Absent).unwrap();
        }
        let list = folder.list().unwrap();
        assert_eq!(list.paths, ["a/b/c/ok.xml", "album/feed.xml", "b.xml"]);
        assert!(list.truncated);
    }

    #[test]
    fn the_config_text_makes_the_same_connector() {
        let folder = LocalFolder::new("/srv/feeds", Some("https://example.com".into()));
        assert_eq!(LocalFolder::from_config(&folder.config()).unwrap(), folder);
        let inferred = LocalFolder::new("/srv/feeds", None)
            .with_inferred_base_url("https://example.com".into());
        assert_eq!(
            LocalFolder::from_config(&inferred.config()).unwrap(),
            inferred
        );
        let old = r#"{"root":"/srv/feeds","base_url":"https://example.com"}"#;
        assert_eq!(LocalFolder::from_config(old).unwrap(), folder);
    }
}

//! Assets on a server that holds none (ADR 0105).
//!
//! With `--hold-assets off`, the host of a file is its master. Studio
//! removes the local bytes of an asset when a host has a copy with the same
//! SHA-256, and when a pending file reaches the end of its 7 days. With
//! `on`, and on the desktop, each function here does nothing
//! ([`Collection::asset_rules`] is `None`).

use std::fmt;

use crate::collection::{Collection, CollectionError, ConnectorId, LibraryFile, LibraryFileId};
use crate::inventory::{self, Hosts};
use crate::library::Library;

/// How long a pending file stays on the server, in seconds (ADR 0105
/// item 8).
pub const PENDING_SECONDS: i64 = 7 * 24 * 60 * 60;

/// An error of the removal of assets.
#[derive(Debug)]
pub struct AssetError {
    message: String,
}

impl AssetError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AssetError {}

impl From<String> for AssetError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<CollectionError> for AssetError {
    fn from(error: CollectionError) -> Self {
        Self::new(error.to_string())
    }
}

/// A copy of a library file on a host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostCopy {
    /// The host.
    pub connector_id: ConnectorId,
    /// The path of the file on the host.
    pub path: String,
    /// The public address of the copy, when the host gives one.
    pub url: Option<String>,
}

/// A function that finds a host copy of a library file with the same
/// SHA-256.
pub type Finder<'a> = &'a dyn Fn(&LibraryFile) -> Result<Option<HostCopy>, AssetError>;

/// True for an audio file and an image. The feed XML and the other files of
/// a release folder are not assets (ADR 0105 item 3).
#[must_use]
pub fn is_asset(media_type: &str) -> bool {
    media_type.starts_with("audio/") || media_type.starts_with("image/")
}

/// The first host file that the inventory records with the size and the
/// SHA-256 of `file`. The function reads nothing from a host (ADR 0105
/// item 13, ADR 0036 items 6 and 7).
///
/// # Errors
///
/// Gives an [`AssetError`] when the collection fails.
pub fn recorded_copy(
    collection: &Collection,
    file: &LibraryFile,
) -> Result<Option<HostCopy>, AssetError> {
    let Some(wanted) = file.sha256.as_deref() else {
        return Ok(None);
    };
    for connector in collection.connectors()? {
        for host_file in collection.host_files(connector.id)? {
            if host_file.size == file.size && host_file.sha256.as_deref() == Some(wanted) {
                return Ok(Some(HostCopy {
                    connector_id: connector.id,
                    path: host_file.path,
                    url: None,
                }));
            }
        }
    }
    Ok(None)
}

/// The first copy of `file` on a connected host, with the same size and
/// SHA-256 (ADR 0036 item 7). The function reads the SHA-256 of a host file
/// with the size of `file`, when the inventory has none yet.
///
/// # Errors
///
/// Gives an [`AssetError`] when a host or the collection fails.
pub fn find_copy(
    collection: &Collection,
    hosts: &Hosts,
    file: &LibraryFile,
) -> Result<Option<HostCopy>, AssetError> {
    let copy = inventory::copies(collection, hosts, file)
        .map_err(|e| AssetError::new(e.to_string()))?
        .into_iter()
        .next();
    Ok(copy.map(|copy| HostCopy {
        connector_id: copy.connector_id,
        path: copy.path,
        url: copy.url,
    }))
}

/// Each library file with its release folder, for the files of `only`, or
/// for all files when `only` is `None`.
fn local_assets(
    collection: &Collection,
    only: Option<&[LibraryFileId]>,
) -> Result<Vec<(String, LibraryFile)>, AssetError> {
    let mut found = Vec::new();
    for feed in collection.feeds()? {
        let Some(folder) = feed.folder.clone() else {
            continue;
        };
        for file in collection.library_files(feed.id)? {
            let chosen = only.is_none_or(|ids| ids.contains(&file.id));
            if chosen && file.local && file.sha256.is_some() && is_asset(&file.media_type) {
                found.push((folder.clone(), file));
            }
        }
    }
    Ok(found)
}

fn remove_bytes(library: &Library, folder: &str, file: &LibraryFile) -> Result<(), AssetError> {
    library
        .remove_file(folder, &file.path)
        .map_err(|e| AssetError::new(e.to_string()))
}

/// Remove the local bytes of each asset that a host has with the same
/// SHA-256 (ADR 0105 items 5 and 13). `find` gives the host copy. The row
/// keeps its SHA-256 and its size, and it gets the address of the copy as
/// `source_url`. With `only`, the function examines these files only.
/// Gives the files that it removed. Does nothing when the workspace holds
/// assets.
///
/// # Errors
///
/// Gives an [`AssetError`] when the collection or the library fails.
pub fn remove_hosted(
    collection: &Collection,
    library: &Library,
    only: Option<&[LibraryFileId]>,
    find: Finder<'_>,
) -> Result<Vec<LibraryFileId>, AssetError> {
    if collection.asset_rules().is_none() {
        return Ok(Vec::new());
    }
    let mut removed = Vec::new();
    for (folder, file) in local_assets(collection, only)? {
        let Some(copy) = find(&file)? else {
            continue;
        };
        remove_bytes(library, &folder, &file)?;
        collection.set_library_file_removed(file.id, copy.url.as_deref(), None)?;
        removed.push(file.id);
    }
    Ok(removed)
}

/// Remove the bytes of each pending file that is older than 7 days at
/// `now`, and record the time on its row (ADR 0105 items 8, 10 and 11). A
/// pending file is an asset that came in after the mode `off` began and
/// that no host has. An asset from before stays on the server (ADR 0105
/// items 12 and 13). Gives the files that it removed. Does nothing when
/// the workspace holds assets.
///
/// # Errors
///
/// Gives an [`AssetError`] when the collection or the library fails.
pub fn expire_pending(
    collection: &Collection,
    library: &Library,
    now: i64,
) -> Result<Vec<LibraryFileId>, AssetError> {
    if collection.asset_rules().is_none() {
        return Ok(Vec::new());
    }
    let Some(since) = collection.assets_off_since()? else {
        return Ok(Vec::new());
    };
    let mut removed = Vec::new();
    for (folder, file) in local_assets(collection, None)? {
        let pending = file.imported_at >= since && now - file.imported_at > PENDING_SECONDS;
        if !pending || recorded_copy(collection, &file)?.is_some() {
            continue;
        }
        remove_bytes(library, &folder, &file)?;
        collection.set_library_file_removed(file.id, None, Some(now))?;
        removed.push(file.id);
    }
    Ok(removed)
}

//! The state of one local user: the collection and the vault.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use std::collections::{HashMap, VecDeque};

use axum::http::StatusCode;
use studio_core::collection::{Collection, FeedId};
use studio_core::connector::{self, HostEnv};
use studio_core::discover::Found;
use studio_core::feed::Feed;
use studio_core::inventory::Hosts;
use studio_core::library::Library;
use studio_core::musicindex::{DEFAULT_API, MusicIndex};
use studio_core::nostr_events::{LOOKUP_RELAYS, Signing};
use studio_core::pipeline::AddressCheck;
use studio_core::remote;
use studio_core::vault::{OpenVault, VaultFile};
use studio_core::web::{Web, WebImage};
use studio_core::xml::Document;

use crate::downloads::DownloadQueue;
use crate::error::ApiError;

/// The number of images that [`ImageCache`] keeps (ADR 0032 item 13).
const MAX_IMAGES: usize = 64;

/// The images that the core already read, by their address (ADR 0025,
/// ADR 0032 items 13 and 14). The UI gets each image through the local API,
/// so that it loads no image from another server (ADR 0016).
///
/// The cache keeps the order that each image came in, so that it can remove
/// the oldest image when it is full.
#[derive(Debug, Default)]
pub(crate) struct ImageCache {
    order: VecDeque<String>,
    images: HashMap<String, WebImage>,
}

impl ImageCache {
    /// The image at this address, when the cache has it.
    pub(crate) fn get(&self, url: &str) -> Option<&WebImage> {
        self.images.get(url)
    }

    /// Keep an image at this address. When the cache is full, it removes the
    /// oldest image.
    pub(crate) fn put(&mut self, url: String, image: WebImage) {
        if !self.images.contains_key(&url) {
            if self.order.len() >= MAX_IMAGES
                && let Some(oldest) = self.order.pop_front()
            {
                self.images.remove(&oldest);
            }
            self.order.push_back(url.clone());
        }
        self.images.insert(url, image);
    }

    /// Remove each image. A refresh of a feed may read each address again,
    /// so Studio does not keep an image that may no longer be correct
    /// (ADR 0032 item 13).
    pub(crate) fn clear(&mut self) {
        self.order.clear();
        self.images.clear();
    }
}

/// The state of the vault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VaultState {
    /// No vault file exists. The first-run wizard makes one.
    Absent,
    /// The vault file exists and is closed.
    Locked,
    /// The vault is open in memory.
    Open,
}

/// A replacement for the address check of the write. Only a test sets it,
/// so that no test sends a request to the network.
#[derive(Clone)]
pub(crate) struct AddressChecker(pub(crate) Arc<dyn AddressCheck + Send + Sync>);

impl std::fmt::Debug for AddressChecker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AddressChecker")
    }
}

/// The collection and the vault in one data folder.
#[derive(Debug)]
pub struct Workspace {
    vault_path: PathBuf,
    pub(crate) collection: Collection,
    pub(crate) vault: Option<OpenVault>,
    pub(crate) web: Web,
    /// The address check of the write. `None` means that the write uses
    /// `web`.
    pub(crate) address_check: Option<AddressChecker>,
    pub(crate) index: MusicIndex,
    /// The relays that give the relay list and the server list of a key
    /// (ADR 0048 item 3). A test sets the address of a test relay.
    pub(crate) lookup_relays: Vec<String>,
    /// The feeds of the last discovery, by address, so that an import does
    /// not read them again.
    pub(crate) discovered: HashMap<String, Found>,
    /// The images that the core already read (ADR 0025, ADR 0032 item 13).
    pub(crate) images: ImageCache,
    /// The staging folder for the body of an upload, before its import
    /// (ADR 0026, ADR 0033 item 1). Task 021e removes the pending-file
    /// store: an import now writes directly into the release folder.
    pub(crate) incoming_dir: PathBuf,
    /// The folder that held the pending files before the library existed
    /// (ADR 0036 item 5). The library route passes this to
    /// `migrate::into_library`, at the first start with a library folder.
    /// A data folder with no such files has no such folder either, and
    /// the move then does nothing for it.
    pub(crate) imports_dir: PathBuf,
    /// The queue of downloads (ADR 0038 items 7 and 9). A route clones the
    /// `Arc` and then unlocks the workspace.
    pub(crate) downloads: Arc<DownloadQueue>,
}

impl Workspace {
    /// Open the workspace in `data_dir`, and make the folder if necessary.
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the folder or the collection cannot be opened.
    pub fn open(data_dir: &Path) -> Result<Self, ApiError> {
        std::fs::create_dir_all(data_dir)
            .map_err(|e| ApiError::internal(format!("{}: {e}", data_dir.display())))?;
        Ok(Self {
            vault_path: data_dir.join("vault.json"),
            collection: Collection::open(&data_dir.join("collection.sqlite"))?,
            vault: None,
            web: Web::new(),
            address_check: None,
            index: MusicIndex::new(Web::new(), DEFAULT_API),
            lookup_relays: LOOKUP_RELAYS.iter().map(|r| (*r).to_owned()).collect(),
            discovered: HashMap::new(),
            images: ImageCache::default(),
            incoming_dir: data_dir.join("incoming"),
            imports_dir: data_dir.join("imports"),
            downloads: DownloadQueue::new(),
        })
    }

    /// Use a different musicindex API, such as a test server.
    #[must_use]
    pub fn with_musicindex(mut self, base: &str) -> Self {
        self.index = MusicIndex::new(self.web.clone(), base);
        self
    }

    /// Use other lookup relays, such as test relays.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_lookup_relays(mut self, relays: Vec<String>) -> Self {
        self.lookup_relays = relays;
        self
    }

    /// Use a different address check for the write, such as a test double.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn with_address_check(mut self, check: Arc<dyn AddressCheck + Send + Sync>) -> Self {
        self.address_check = Some(AddressChecker(check));
        self
    }

    pub(crate) fn vault_state(&self) -> VaultState {
        if self.vault.is_some() {
            VaultState::Open
        } else if self.vault_path.exists() {
            VaultState::Locked
        } else {
            VaultState::Absent
        }
    }

    pub(crate) fn vault_file(&self) -> Result<VaultFile, ApiError> {
        Ok(VaultFile::load(&self.vault_path)?)
    }

    pub(crate) fn save_vault(&self) -> Result<(), ApiError> {
        if let Some(vault) = &self.vault {
            vault.file().save(&self.vault_path)?;
        }
        Ok(())
    }

    /// The connector of each host, for the diff and the write
    /// (ADR 0036 item 6).
    ///
    /// A host that needs a secret while the vault is locked is left out
    /// of the set, and marked instead (ADR 0017). An operation that then
    /// needs that host gives `vault_locked`, through
    /// [`studio_core::inventory::InventoryErrorKind::VaultLocked`].
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the collection or a connector fails.
    pub(crate) fn hosts(&self) -> Result<Hosts, ApiError> {
        self.hosts_with(Signing::default())
    }

    /// The connector of each host, like [`Workspace::hosts`], with the
    /// events that the user signed for a write (ADR 0050 item 4).
    ///
    /// # Errors
    ///
    /// Gives an [`ApiError`] when the collection or a connector fails.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "each host gets its own copy of the signing"
    )]
    pub(crate) fn hosts_with(&self, signing: Signing) -> Result<Hosts, ApiError> {
        let mut hosts = Hosts::new();
        for row in self.collection.connectors()? {
            if row.secret_ref.is_some() && self.vault.is_none() {
                hosts.insert_locked(row.id);
                continue;
            }
            let env = HostEnv {
                vault: self.vault.as_ref(),
                blobs: self.collection.kept_blobs(row.id)?,
                signing: signing.clone(),
            };
            let made = connector::from_row_in(&row, &env)?;
            hosts.insert(row.id, made);
        }
        Ok(hosts)
    }

    /// The library folder, from the `library_root` setting (ADR 0036
    /// item 1).
    ///
    /// # Errors
    ///
    /// Gives 409 `no_library` when the setting is absent, or an
    /// [`ApiError`] when the collection fails or Studio cannot open the
    /// folder.
    pub(crate) fn library(&self) -> Result<Library, ApiError> {
        let root = self.collection.setting("library_root")?.ok_or_else(|| {
            ApiError::new(
                StatusCode::CONFLICT,
                "no_library",
                "Studio has no library folder yet. Set one first.",
            )
        })?;
        let root = root
            .as_str()
            .ok_or_else(|| ApiError::internal("the library_root setting is not a string"))?;
        Library::open(root).map_err(|error| ApiError::internal(error.to_string()))
    }

    /// Keep the remote file records of one feed current, with `feed` as
    /// the hosted feed (ADR 0038 item 3). A feed with no release folder
    /// has no records.
    ///
    /// An error goes into the log and does not stop the caller. The
    /// function never writes to a host.
    pub(crate) fn record_remote_files(&self, id: FeedId, feed: &Feed) {
        let result = (|| -> Result<(), String> {
            let row = self.collection.feed(id).map_err(|e| e.to_string())?;
            if row.folder.is_none() {
                return Ok(());
            }
            let hosts = self.hosts().map_err(|e| format!("{e:?}"))?;
            remote::record_remote_files(&self.collection, &hosts, id, feed)
                .map_err(|e| e.to_string())?;
            Ok(())
        })();
        if let Err(message) = result {
            eprintln!(
                "studio: the remote file records of feed {} failed: {message}",
                id.0
            );
        }
    }

    /// Keep the remote file records of each feed with a release folder
    /// current. Each feed gives its hosted feed from the feed XML in its
    /// release folder (ADR 0038 item 3). The function does nothing when
    /// Studio has no library folder yet.
    ///
    /// An error for one feed goes into the log, and the other feeds still
    /// run.
    pub fn record_all_remote_files(&self) {
        let Ok(library) = self.library() else {
            return;
        };
        let feeds = match self.collection.feeds() {
            Ok(feeds) => feeds,
            Err(error) => {
                eprintln!("studio: the list of feeds failed: {error}");
                return;
            }
        };
        for row in feeds {
            if row.folder.is_none() {
                continue;
            }
            match library_feed(&library, &row) {
                Ok(feed) => self.record_remote_files(row.id, &feed),
                Err(message) => {
                    eprintln!("studio: feed {}: {message}", row.id.0);
                }
            }
        }
    }

    /// Put each download that the collection kept as `queued` into the
    /// queue again (ADR 0038 item 9). The workers start when the router
    /// is ready. A download that failed stays failed, until the user
    /// starts it again.
    ///
    /// An error goes into the log and does not stop the caller.
    pub fn resume_downloads(&self) {
        let rows = match self.collection.downloads() {
            Ok(rows) => rows,
            Err(error) => {
                eprintln!("studio: the list of downloads failed: {error}");
                return;
            }
        };
        for row in rows.into_iter().filter(|row| row.state == "queued") {
            match self.collection.library_file(row.library_file_id) {
                Ok(file) if !file.local => {
                    self.downloads.add(file.id, file.feed_id, file.size);
                }
                Ok(_) => {
                    let _ = self.collection.remove_download(row.library_file_id);
                }
                Err(error) => eprintln!(
                    "studio: download of library file {} failed to resume: {error}",
                    row.library_file_id.0
                ),
            }
        }
    }
}

/// The hosted feed of a feed row, from the feed XML in its release folder
/// (ADR 0036 item 3).
///
/// The feed XML has the name of the last part of the feed path or of the
/// feed address.
///
/// # Errors
///
/// Gives a message when the feed has no release folder, the folder has no
/// feed XML, or the XML is not a feed.
pub(crate) fn library_feed(
    library: &Library,
    row: &studio_core::collection::FeedRow,
) -> Result<Feed, String> {
    let folder = row
        .folder
        .as_deref()
        .ok_or_else(|| "the feed has no release folder".to_owned())?;
    let names = [row.path.as_deref(), row.public_url.as_deref()];
    let bytes = names
        .into_iter()
        .flatten()
        .map(|name| name.rsplit_once('/').map_or(name, |(_, last)| last))
        .find_map(|name| std::fs::read(library.path(folder, name)).ok())
        .ok_or_else(|| "the release folder has no feed XML".to_owned())?;
    let doc = Document::parse_bytes(&bytes).map_err(|e| e.to_string())?;
    Feed::read(&doc).map_err(|e| format!("the feed XML is not a feed: {e}"))
}

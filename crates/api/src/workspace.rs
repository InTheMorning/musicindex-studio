//! The state of one local user: the collection and the vault.

use std::path::{Path, PathBuf};

use std::collections::{HashMap, VecDeque};

use studio_core::collection::Collection;
use studio_core::discover::Found;
use studio_core::imports::Store;
use studio_core::musicindex::{DEFAULT_API, MusicIndex};
use studio_core::vault::{OpenVault, VaultFile};
use studio_core::web::{Web, WebImage};

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

/// The collection and the vault in one data folder.
#[derive(Debug)]
pub struct Workspace {
    vault_path: PathBuf,
    pub(crate) collection: Collection,
    pub(crate) vault: Option<OpenVault>,
    pub(crate) web: Web,
    pub(crate) index: MusicIndex,
    /// The feeds of the last discovery, by address, so that an import does
    /// not read them again.
    pub(crate) discovered: HashMap<String, Found>,
    /// The images that the core already read (ADR 0025, ADR 0032 item 13).
    pub(crate) images: ImageCache,
    /// The pending files of each draft, in the data folder (ADR 0026,
    /// ADR 0032 items 10 to 12).
    pub(crate) store: Store,
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
            index: MusicIndex::new(Web::new(), DEFAULT_API),
            discovered: HashMap::new(),
            images: ImageCache::default(),
            store: Store::new(data_dir.join("imports")),
        })
    }

    /// Use a different musicindex API, such as a test server.
    #[must_use]
    pub fn with_musicindex(mut self, base: &str) -> Self {
        self.index = MusicIndex::new(self.web.clone(), base);
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
}

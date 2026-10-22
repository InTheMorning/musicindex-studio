//! The state of one local user: the collection and the vault.

use std::path::{Path, PathBuf};

use studio_core::collection::Collection;
use studio_core::vault::{OpenVault, VaultFile};

use crate::error::ApiError;

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
        })
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

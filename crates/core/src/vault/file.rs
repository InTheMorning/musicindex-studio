//! The stored form of a vault (ADR 0013).
//!
//! The desktop shell keeps one vault in one JSON file. The server keeps the
//! same structure for each user.

use std::fs;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::VaultError;
use super::crypto::Sealed;

/// The format version that this code writes and reads.
pub(crate) const FORMAT: u32 = 1;

/// A vault as it is stored. It contains no plain secret and no plain key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultFile {
    pub(crate) format: u32,
    pub(crate) vault_id: String,
    /// An empty value that the vault key encrypts. A key that opens it is
    /// the vault key.
    pub(crate) check: Sealed,
    pub(crate) wrappers: Vec<Wrapper>,
    pub(crate) records: Vec<Record>,
}

/// An encrypted copy of the vault key for one unlock method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum Wrapper {
    /// A key from a passphrase encrypts the vault key.
    Passphrase(DerivedWrapper),
    /// A key from the recovery key encrypts the vault key.
    Recovery(DerivedWrapper),
    /// The user's signer encrypted the vault key to the user's own public key
    /// with NIP-44.
    Nostr { pubkey: String, ciphertext: String },
}

/// A wrapper whose key comes from Argon2id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DerivedWrapper {
    pub(crate) salt: String,
    pub(crate) memory_kib: u32,
    pub(crate) passes: u32,
    pub(crate) lanes: u32,
    #[serde(flatten)]
    pub(crate) sealed: Sealed,
}

/// One encrypted secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Record {
    pub(crate) id: String,
    #[serde(flatten)]
    pub(crate) sealed: Sealed,
}

impl VaultFile {
    /// The ID of the vault.
    #[must_use]
    pub fn vault_id(&self) -> &str {
        &self.vault_id
    }

    /// The NIP-44 ciphertext of the vault key for a Nostr public key.
    ///
    /// The user's signer decrypts it, and the caller gives the result to
    /// [`VaultFile::open_with_key`](super::VaultFile::open_with_key).
    #[must_use]
    pub fn nostr_ciphertext(&self, pubkey: &str) -> Option<&str> {
        self.wrappers.iter().find_map(|w| match w {
            Wrapper::Nostr {
                pubkey: p,
                ciphertext,
            } if p == pubkey => Some(ciphertext.as_str()),
            _ => None,
        })
    }

    /// The public key and the NIP-44 ciphertext of the first Nostr wrapper.
    #[must_use]
    pub fn first_nostr_wrapper(&self) -> Option<(&str, &str)> {
        self.wrappers.iter().find_map(|w| match w {
            Wrapper::Nostr { pubkey, ciphertext } => Some((pubkey.as_str(), ciphertext.as_str())),
            _ => None,
        })
    }

    /// Read a vault from JSON.
    ///
    /// # Errors
    ///
    /// Gives a [`VaultError`] when the JSON is not a vault of a known format.
    pub fn from_json(json: &str) -> Result<Self, VaultError> {
        let file: Self = serde_json::from_str(json)
            .map_err(|_| VaultError::corrupt("the vault is not valid JSON"))?;
        if file.format != FORMAT {
            return Err(VaultError::corrupt("the vault format is not known"));
        }
        Ok(file)
    }

    /// Write the vault as JSON.
    ///
    /// # Panics
    ///
    /// Does not panic in practice: the vault holds only strings and numbers,
    /// and these always serialize.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("a vault always serializes")
    }

    /// Read a desktop vault file.
    ///
    /// # Errors
    ///
    /// Gives a [`VaultError`] when the file cannot be read or is not a vault.
    pub fn load(path: &Path) -> Result<Self, VaultError> {
        let json = fs::read_to_string(path).map_err(VaultError::io)?;
        Self::from_json(&json)
    }

    /// Write a desktop vault file.
    ///
    /// The function writes a temporary file in the same directory, flushes
    /// it to disk and then renames it. An interrupted write does not damage
    /// the vault (ADR 0013).
    ///
    /// # Errors
    ///
    /// Gives a [`VaultError`] when the file cannot be written.
    pub fn save(&self, path: &Path) -> Result<(), VaultError> {
        let mut temporary = path.as_os_str().to_owned();
        temporary.push(".tmp");
        let temporary = Path::new(&temporary);
        let mut file = fs::File::create(temporary).map_err(VaultError::io)?;
        file.write_all(self.to_json().as_bytes())
            .map_err(VaultError::io)?;
        file.sync_all().map_err(VaultError::io)?;
        drop(file);
        fs::rename(temporary, path).map_err(VaultError::io)?;
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            // A directory flush makes the rename durable. Some platforms
            // cannot open a directory, and the rename is then already done.
            if let Ok(dir) = fs::File::open(dir) {
                let _ = dir.sync_all();
            }
        }
        Ok(())
    }
}

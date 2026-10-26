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
    /// A key from HKDF-SHA-256 over the 16 bytes of the recovery key
    /// encrypts the vault key.
    RecoveryHkdf(HkdfWrapper),
    /// A key from HKDF-SHA-256 over the PRF output of a passkey encrypts the
    /// vault key.
    Passkey {
        credential_id: String,
        #[serde(flatten)]
        inner: HkdfWrapper,
    },
    /// The user's signer encrypted the vault key to the user's own public key
    /// with NIP-44.
    Nostr { pubkey: String, ciphertext: String },
    /// A connected app. The vault key as base64, encrypted with NIP-44
    /// from a random key to the public key of the app. `sender` is the
    /// public key of the random key (ADR 0058 item 5).
    App {
        app_id: i64,
        sender: String,
        ciphertext: String,
    },
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

/// A wrapper whose key comes from HKDF-SHA-256.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct HkdfWrapper {
    pub(crate) salt: String,
    #[serde(flatten)]
    pub(crate) sealed: Sealed,
}

/// One unlock method of a vault, without salt and without ciphertext.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultMethod {
    /// A passphrase.
    Passphrase,
    /// A recovery key, of either wrapper type.
    Recovery,
    /// A Nostr signer, with its public key.
    Nostr {
        /// The public key of the signer.
        pubkey: String,
    },
    /// A passkey, with its credential ID.
    Passkey {
        /// The credential ID.
        credential_id: String,
    },
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

    /// The unlock methods of the vault. The result has no salt and no
    /// ciphertext. A wrapper of a connected app is not a method of the
    /// user.
    #[must_use]
    pub fn methods(&self) -> Vec<VaultMethod> {
        self.wrappers
            .iter()
            .filter_map(|w| match w {
                Wrapper::Passphrase(_) => Some(VaultMethod::Passphrase),
                Wrapper::Recovery(_) | Wrapper::RecoveryHkdf(_) => Some(VaultMethod::Recovery),
                Wrapper::Nostr { pubkey, .. } => Some(VaultMethod::Nostr {
                    pubkey: pubkey.clone(),
                }),
                Wrapper::Passkey { credential_id, .. } => Some(VaultMethod::Passkey {
                    credential_id: credential_id.clone(),
                }),
                Wrapper::App { .. } => None,
            })
            .collect()
    }

    /// The public key of the random key and the NIP-44 ciphertext of the
    /// wrapper for a connected app.
    #[must_use]
    pub fn app_wrapper(&self, app_id: i64) -> Option<(&str, &str)> {
        self.wrappers.iter().find_map(|w| match w {
            Wrapper::App {
                app_id: id,
                sender,
                ciphertext,
            } if *id == app_id => Some((sender.as_str(), ciphertext.as_str())),
            _ => None,
        })
    }

    /// The IDs of the connected apps that have a wrapper.
    #[must_use]
    pub fn app_ids(&self) -> Vec<i64> {
        self.wrappers
            .iter()
            .filter_map(|w| match w {
                Wrapper::App { app_id, .. } => Some(*app_id),
                _ => None,
            })
            .collect()
    }

    /// Keep the wrapper of a connected app. It replaces an older wrapper
    /// of the same app.
    pub fn set_app_wrapper(&mut self, app_id: i64, sender: &str, ciphertext: &str) {
        self.remove_app_wrapper(app_id);
        self.wrappers.push(Wrapper::App {
            app_id,
            sender: sender.to_owned(),
            ciphertext: ciphertext.to_owned(),
        });
    }

    /// Remove the wrapper of a connected app. Gives `true` when the vault
    /// had it.
    pub fn remove_app_wrapper(&mut self, app_id: i64) -> bool {
        let before = self.wrappers.len();
        self.wrappers
            .retain(|w| !matches!(w, Wrapper::App { app_id: id, .. } if *id == app_id));
        before != self.wrappers.len()
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

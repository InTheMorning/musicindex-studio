//! The secret vault (ADR 0003, ADR 0013).
//!
//! A vault keeps the secrets that connectors use to write to hosts. One
//! random vault key encrypts each secret. Each unlock method keeps its own
//! encrypted copy of the vault key, called a wrapper:
//!
//! - a passphrase, through Argon2id;
//! - an optional recovery key, through Argon2id;
//! - a Nostr signer, through NIP-44. The signer in the browser does the
//!   NIP-44 step. This module only keeps its ciphertext.
//!
//! [`VaultFile`] is the stored form. [`OpenVault`] is a vault with its key,
//! and it reads and writes secrets.

mod crypto;
mod file;

use std::fmt;

use zeroize::Zeroizing;

use crypto::{KDF_LANES, KDF_MEMORY_KIB, KDF_PASSES, Key};
pub use file::VaultFile;
use file::{DerivedWrapper, FORMAT, Record, Wrapper};

/// The digits of a recovery key.
const HEX: &[u8; 16] = b"0123456789abcdef";

/// The minimum number of characters in a passphrase (ADR 0013).
pub const MIN_PASSPHRASE_CHARS: usize = 12;

/// The kind of a [`VaultError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum VaultErrorKind {
    /// The passphrase has fewer than [`MIN_PASSPHRASE_CHARS`] characters.
    PassphraseTooShort,
    /// The passphrase, the recovery key or the vault key is not correct, or
    /// the vault has no wrapper for the method.
    WrongKey,
    /// The vault data is damaged or has an unknown format.
    Corrupt,
    /// A file operation failed.
    Io,
}

/// An error from the vault.
///
/// The message does not contain a secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultError {
    kind: VaultErrorKind,
    message: String,
}

impl VaultError {
    pub(crate) fn wrong_key() -> Self {
        Self {
            kind: VaultErrorKind::WrongKey,
            message: "the key does not open the vault".to_owned(),
        }
    }

    pub(crate) fn corrupt(message: &str) -> Self {
        Self {
            kind: VaultErrorKind::Corrupt,
            message: message.to_owned(),
        }
    }

    #[expect(clippy::needless_pass_by_value, reason = "used as a map_err function")]
    pub(crate) fn io(error: std::io::Error) -> Self {
        Self {
            kind: VaultErrorKind::Io,
            message: error.to_string(),
        }
    }

    /// The kind of the error.
    #[must_use]
    pub fn kind(&self) -> VaultErrorKind {
        self.kind
    }
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for VaultError {}

/// A recovery key, as the user writes it down. Studio shows it one time.
pub struct RecoveryKey(Zeroizing<String>);

impl RecoveryKey {
    /// The text that the user writes down.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for RecoveryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RecoveryKey(..)")
    }
}

/// A vault with its key. It reads and writes secrets.
pub struct OpenVault {
    file: VaultFile,
    key: Key,
}

impl fmt::Debug for OpenVault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenVault")
            .field("vault_id", &self.file.vault_id)
            .field("records", &self.file.records.len())
            .finish_non_exhaustive()
    }
}

impl OpenVault {
    /// Make a new empty vault that a passphrase opens.
    ///
    /// # Errors
    ///
    /// Gives a [`VaultError`] when the passphrase is too short or the
    /// random source is not available.
    pub fn create(passphrase: &str) -> Result<Self, VaultError> {
        check_passphrase(passphrase)?;
        let key = Key::random()?;
        let vault_id = uuid::Uuid::new_v4().to_string();
        let check = key.seal(&[], check_aad(&vault_id).as_bytes())?;
        let mut vault = Self {
            file: VaultFile {
                format: FORMAT,
                vault_id,
                check,
                wrappers: Vec::new(),
                records: Vec::new(),
            },
            key,
        };
        vault.set_passphrase(passphrase)?;
        Ok(vault)
    }

    /// The stored form of the vault. Save it after each change.
    #[must_use]
    pub fn file(&self) -> &VaultFile {
        &self.file
    }

    /// Replace the passphrase. The vault key and the records do not change.
    ///
    /// # Errors
    ///
    /// Gives a [`VaultError`] when the passphrase is too short.
    pub fn set_passphrase(&mut self, passphrase: &str) -> Result<(), VaultError> {
        check_passphrase(passphrase)?;
        let wrapper = self.derived_wrapper("passphrase", passphrase.as_bytes())?;
        self.file
            .wrappers
            .retain(|w| !matches!(w, Wrapper::Passphrase(_)));
        self.file.wrappers.push(Wrapper::Passphrase(wrapper));
        Ok(())
    }

    /// Make a new recovery key. It replaces an older recovery key.
    ///
    /// Show the result to the user one time. Studio cannot show it again.
    ///
    /// # Errors
    ///
    /// Gives a [`VaultError`] when the random source is not available.
    pub fn add_recovery_key(&mut self) -> Result<RecoveryKey, VaultError> {
        let mut bytes = Zeroizing::new([0_u8; 16]);
        crypto::random_fill(bytes.as_mut_slice())?;
        let mut hex = Zeroizing::new(String::with_capacity(32));
        for byte in bytes.iter() {
            hex.push(char::from(HEX[usize::from(byte >> 4)]));
            hex.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        let wrapper = self.derived_wrapper("recovery", hex.as_bytes())?;
        self.file
            .wrappers
            .retain(|w| !matches!(w, Wrapper::Recovery(_)));
        self.file.wrappers.push(Wrapper::Recovery(wrapper));
        let groups: Vec<&str> = (0..hex.len()).step_by(4).map(|i| &hex[i..i + 4]).collect();
        Ok(RecoveryKey(Zeroizing::new(groups.join("-"))))
    }

    /// The vault key as base64, for the user's signer to encrypt with NIP-44.
    ///
    /// Give it only to the signer of the signed-in user, and then call
    /// [`OpenVault::add_nostr_wrapper`] with the ciphertext.
    #[must_use]
    pub fn key_for_signer(&self) -> Zeroizing<String> {
        Zeroizing::new(crypto::encode(self.key.as_bytes()))
    }

    /// Keep the NIP-44 ciphertext of the vault key for a Nostr public key.
    /// It replaces an older wrapper for the same key.
    pub fn add_nostr_wrapper(&mut self, pubkey: &str, ciphertext: &str) {
        self.file
            .wrappers
            .retain(|w| !matches!(w, Wrapper::Nostr { pubkey: p, .. } if p == pubkey));
        self.file.wrappers.push(Wrapper::Nostr {
            pubkey: pubkey.to_owned(),
            ciphertext: ciphertext.to_owned(),
        });
    }

    /// Add or replace a secret. Each write uses a new nonce.
    ///
    /// # Errors
    ///
    /// Gives a [`VaultError`] when the random source is not available.
    pub fn put(&mut self, id: &str, secret: &[u8]) -> Result<(), VaultError> {
        let sealed = self
            .key
            .seal(secret, record_aad(&self.file.vault_id, id).as_bytes())?;
        let record = Record {
            id: id.to_owned(),
            sealed,
        };
        match self.file.records.iter_mut().find(|r| r.id == id) {
            Some(existing) => *existing = record,
            None => self.file.records.push(record),
        }
        Ok(())
    }

    /// Read a secret. Gives `None` when the vault has no record with this ID.
    ///
    /// # Errors
    ///
    /// Gives a [`VaultError`] when the record does not decrypt.
    pub fn get(&self, id: &str) -> Result<Option<Zeroizing<Vec<u8>>>, VaultError> {
        let Some(record) = self.file.records.iter().find(|r| r.id == id) else {
            return Ok(None);
        };
        self.key
            .open(
                &record.sealed,
                record_aad(&self.file.vault_id, id).as_bytes(),
            )
            .map(Some)
    }

    /// Remove a secret. Gives `true` when the vault had it.
    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.file.records.len();
        self.file.records.retain(|r| r.id != id);
        before != self.file.records.len()
    }

    /// The IDs of the secrets.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.file.records.iter().map(|r| r.id.as_str())
    }

    fn derived_wrapper(&self, kind: &str, secret: &[u8]) -> Result<DerivedWrapper, VaultError> {
        let salt = crypto::random_salt()?;
        let wrapping = Key::derive(
            secret,
            &crypto::decode(&salt)?,
            KDF_MEMORY_KIB,
            KDF_PASSES,
            KDF_LANES,
        )?;
        let sealed = wrapping.seal(
            self.key.as_bytes(),
            wrapper_aad(kind, &self.file.vault_id).as_bytes(),
        )?;
        Ok(DerivedWrapper {
            salt,
            memory_kib: KDF_MEMORY_KIB,
            passes: KDF_PASSES,
            lanes: KDF_LANES,
            sealed,
        })
    }
}

impl VaultFile {
    /// Open the vault with its passphrase.
    ///
    /// # Errors
    ///
    /// Gives [`VaultErrorKind::WrongKey`] when the passphrase is not correct.
    pub fn open_with_passphrase(self, passphrase: &str) -> Result<OpenVault, VaultError> {
        self.open_derived("passphrase", passphrase.as_bytes())
    }

    /// Open the vault with its recovery key. Spaces, hyphens and the case of
    /// letters do not matter.
    ///
    /// # Errors
    ///
    /// Gives [`VaultErrorKind::WrongKey`] when the recovery key is not correct.
    pub fn open_with_recovery_key(self, recovery_key: &str) -> Result<OpenVault, VaultError> {
        let normal: Zeroizing<String> = Zeroizing::new(
            recovery_key
                .chars()
                .filter(char::is_ascii_hexdigit)
                .map(|c| c.to_ascii_lowercase())
                .collect(),
        );
        self.open_derived("recovery", normal.as_bytes())
    }

    /// Open the vault with the vault key, as base64. The user's signer gives
    /// it after it decrypts [`VaultFile::nostr_ciphertext`].
    ///
    /// # Errors
    ///
    /// Gives [`VaultErrorKind::WrongKey`] when the key is not the vault key.
    pub fn open_with_key(self, key_base64: &str) -> Result<OpenVault, VaultError> {
        let bytes =
            Zeroizing::new(crypto::decode(key_base64).map_err(|_| VaultError::wrong_key())?);
        let key = Key::from_slice(&bytes)?;
        self.open_checked(key)
    }

    fn open_derived(self, kind: &str, secret: &[u8]) -> Result<OpenVault, VaultError> {
        let aad = wrapper_aad(kind, &self.vault_id);
        let wrapper = self.wrappers.iter().find_map(|w| match (w, kind) {
            (Wrapper::Passphrase(d), "passphrase") | (Wrapper::Recovery(d), "recovery") => Some(d),
            _ => None,
        });
        let wrapper = wrapper.ok_or_else(VaultError::wrong_key)?;
        let wrapping = Key::derive(
            secret,
            &crypto::decode(&wrapper.salt)?,
            wrapper.memory_kib,
            wrapper.passes,
            wrapper.lanes,
        )?;
        let key_bytes = wrapping.open(&wrapper.sealed, aad.as_bytes())?;
        let key = Key::from_slice(&key_bytes)?;
        self.open_checked(key)
    }

    fn open_checked(self, key: Key) -> Result<OpenVault, VaultError> {
        key.open(&self.check, check_aad(&self.vault_id).as_bytes())?;
        Ok(OpenVault { file: self, key })
    }
}

fn check_passphrase(passphrase: &str) -> Result<(), VaultError> {
    if passphrase.chars().count() < MIN_PASSPHRASE_CHARS {
        return Err(VaultError {
            kind: VaultErrorKind::PassphraseTooShort,
            message: format!(
                "a passphrase must have a minimum of {MIN_PASSPHRASE_CHARS} characters"
            ),
        });
    }
    Ok(())
}

fn record_aad(vault_id: &str, record_id: &str) -> String {
    format!("musicindex-studio vault {FORMAT} record {vault_id} {record_id}")
}

fn wrapper_aad(kind: &str, vault_id: &str) -> String {
    format!("musicindex-studio vault {FORMAT} wrapper {kind} {vault_id}")
}

fn check_aad(vault_id: &str) -> String {
    format!("musicindex-studio vault {FORMAT} check {vault_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASSPHRASE: &str = "correct horse battery";
    const SECRET: &[u8] = b"sftp-password-7f3a9c";

    fn vault_with_secret() -> OpenVault {
        let mut vault = OpenVault::create(PASSPHRASE).unwrap();
        vault.put("sftp:example", SECRET).unwrap();
        vault
    }

    #[test]
    fn the_passphrase_opens_the_vault_and_a_wrong_one_does_not() {
        let file = vault_with_secret().file().clone();
        let vault = file.clone().open_with_passphrase(PASSPHRASE).unwrap();
        assert_eq!(
            vault.get("sftp:example").unwrap().unwrap().as_slice(),
            SECRET
        );
        let error = file
            .open_with_passphrase("correct horse battery!")
            .unwrap_err();
        assert_eq!(error.kind(), VaultErrorKind::WrongKey);
    }

    #[test]
    fn the_recovery_key_opens_the_vault_in_any_case_and_a_wrong_one_does_not() {
        let mut vault = vault_with_secret();
        let recovery = vault.add_recovery_key().unwrap();
        let file = vault.file().clone();
        let typed = recovery.as_str().to_uppercase().replace('-', " ");
        let opened = file.clone().open_with_recovery_key(&typed).unwrap();
        assert_eq!(
            opened.get("sftp:example").unwrap().unwrap().as_slice(),
            SECRET
        );
        let mut wrong = recovery.as_str().to_owned();
        let last = if wrong.ends_with('0') { "1" } else { "0" };
        wrong.replace_range(wrong.len() - 1.., last);
        let error = file.open_with_recovery_key(&wrong).unwrap_err();
        assert_eq!(error.kind(), VaultErrorKind::WrongKey);
    }

    #[test]
    fn the_vault_key_from_a_signer_opens_the_vault_and_a_wrong_key_does_not() {
        let mut vault = vault_with_secret();
        let key = vault.key_for_signer();
        // The signer would encrypt `key` with NIP-44. The test keeps a
        // placeholder ciphertext, because the Rust code does not do NIP-44.
        vault.add_nostr_wrapper("npub-hex", "nip44-ciphertext");
        let file = vault.file().clone();
        assert_eq!(file.nostr_ciphertext("npub-hex"), Some("nip44-ciphertext"));
        let opened = file.clone().open_with_key(&key).unwrap();
        assert_eq!(
            opened.get("sftp:example").unwrap().unwrap().as_slice(),
            SECRET
        );
        let other = OpenVault::create(PASSPHRASE).unwrap().key_for_signer();
        assert_eq!(
            file.clone().open_with_key(&other).unwrap_err().kind(),
            VaultErrorKind::WrongKey
        );
        assert_eq!(
            file.open_with_key("not base64").unwrap_err().kind(),
            VaultErrorKind::WrongKey
        );
    }

    #[test]
    fn a_short_passphrase_gives_an_error() {
        let error = OpenVault::create("eleven char").unwrap_err();
        assert_eq!(error.kind(), VaultErrorKind::PassphraseTooShort);
        let mut vault = OpenVault::create(PASSPHRASE).unwrap();
        assert_eq!(
            vault.set_passphrase("short").unwrap_err().kind(),
            VaultErrorKind::PassphraseTooShort
        );
    }

    #[test]
    fn a_record_that_moves_does_not_decrypt() {
        let vault = vault_with_secret();
        let mut file = vault.file().clone();
        file.records[0].id = "sftp:other".to_owned();
        let moved = file.open_with_passphrase(PASSPHRASE).unwrap();
        assert_eq!(
            moved.get("sftp:other").unwrap_err().kind(),
            VaultErrorKind::WrongKey
        );

        let mut other = OpenVault::create(PASSPHRASE).unwrap();
        other.file.records.push(vault.file().records[0].clone());
        assert_eq!(
            other.get("sftp:example").unwrap_err().kind(),
            VaultErrorKind::WrongKey
        );
    }

    #[test]
    fn each_write_uses_a_new_nonce() {
        let mut vault = vault_with_secret();
        let first = vault.file().records[0].sealed.clone();
        vault.put("sftp:example", SECRET).unwrap();
        let second = vault.file().records[0].sealed.clone();
        assert_ne!(first.nonce, second.nonce);
        assert_ne!(first.ciphertext, second.ciphertext);
        assert_eq!(vault.ids().count(), 1);
    }

    #[test]
    fn a_new_passphrase_keeps_the_key_and_the_records() {
        let mut vault = vault_with_secret();
        let key = vault.key_for_signer();
        vault.set_passphrase("a new and longer passphrase").unwrap();
        let file = vault.file().clone();
        assert_eq!(
            file.clone()
                .open_with_passphrase(PASSPHRASE)
                .unwrap_err()
                .kind(),
            VaultErrorKind::WrongKey
        );
        let opened = file
            .open_with_passphrase("a new and longer passphrase")
            .unwrap();
        assert_eq!(*opened.key_for_signer(), *key);
        assert_eq!(
            opened.get("sftp:example").unwrap().unwrap().as_slice(),
            SECRET
        );
    }

    #[test]
    fn the_stored_vault_and_the_debug_output_contain_no_secret_and_no_key() {
        let mut vault = vault_with_secret();
        let recovery = vault.add_recovery_key().unwrap();
        let key = vault.key_for_signer();
        let json = vault.file().to_json();
        assert!(json.contains("\"memory_kib\": 65536"));
        assert!(json.contains("\"passes\": 3"));
        let debug = format!("{vault:?} {:?} {:?} {recovery:?}", vault.file(), vault.key);
        let secret = std::str::from_utf8(SECRET).unwrap();
        for text in [&json, &debug] {
            assert!(!text.contains(secret));
            assert!(!text.contains(key.as_str()));
            assert!(!text.contains(PASSPHRASE));
            assert!(!text.contains(recovery.as_str()));
            assert!(!text.contains(&recovery.as_str().replace('-', "")));
        }
    }

    #[test]
    fn a_saved_vault_file_loads_again() {
        let dir = std::env::temp_dir().join(format!("studio-vault-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("vault.json");
        let vault = vault_with_secret();
        vault.file().save(&path).unwrap();
        let loaded = VaultFile::load(&path).unwrap();
        assert_eq!(&loaded, vault.file());
        assert!(!dir.join("vault.json.tmp").exists());
        let opened = loaded.open_with_passphrase(PASSPHRASE).unwrap();
        assert_eq!(
            opened.get("sftp:example").unwrap().unwrap().as_slice(),
            SECRET
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_vault_of_an_unknown_format_gives_an_error() {
        let json = vault_with_secret()
            .file()
            .to_json()
            .replace("\"format\": 1", "\"format\": 2");
        assert_eq!(
            VaultFile::from_json(&json).unwrap_err().kind(),
            VaultErrorKind::Corrupt
        );
    }
}

//! The secret vault (ADR 0003, ADR 0013).
//!
//! A vault keeps the secrets that connectors use to write to hosts. One
//! random vault key encrypts each secret. Each unlock method keeps its own
//! encrypted copy of the vault key, called a wrapper:
//!
//! - a passphrase, through Argon2id;
//! - an optional recovery key, through HKDF-SHA-256 (an older vault can
//!   have a recovery key through Argon2id);
//! - passkeys, through HKDF-SHA-256 over a PRF output;
//! - a Nostr signer, through NIP-44. The signer in the browser does the
//!   NIP-44 step. This module only keeps its ciphertext.
//!
//! [`VaultFile`] is the stored form. [`OpenVault`] is a vault with its key,
//! and it reads and writes secrets.

pub(crate) mod crypto;
mod file;

use std::fmt;

use zeroize::Zeroizing;

use crypto::{KDF_LANES, KDF_MEMORY_KIB, KDF_PASSES, Key};
use file::{DerivedWrapper, FORMAT, HkdfWrapper, Record, Wrapper};
pub use file::{VaultFile, VaultMethod};

/// The HKDF info text of a recovery wrapper.
const RECOVERY_INFO: &str = "musicindex-studio recovery wrapper v1";

/// The HKDF info text of a passkey wrapper.
const PASSKEY_INFO: &str = "musicindex-studio passkey wrapper v1";

/// The digits of a recovery key.
const HEX: &[u8; 16] = b"0123456789abcdef";

/// The minimum number of characters in a passphrase (ADR 0013).
pub const MIN_PASSPHRASE_CHARS: usize = 12;

/// The minimum zxcvbn score of a passphrase, from 0 to 4 (ADR 0018).
pub const MIN_PASSPHRASE_SCORE: u8 = 3;

/// The strength of a passphrase, from zxcvbn (ADR 0018).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PassphraseStrength {
    /// The zxcvbn score, from 0 to 4.
    pub score: u8,
    /// The number of characters.
    pub chars: usize,
    /// True when the vault accepts the passphrase.
    pub acceptable: bool,
    /// The zxcvbn warning, such as "This is a top-10 common password."
    pub warning: Option<String>,
    /// The zxcvbn suggestions.
    pub suggestions: Vec<String>,
}

/// Measure a passphrase. The result contains no part of the passphrase.
#[must_use]
pub fn passphrase_strength(passphrase: &str) -> PassphraseStrength {
    let chars = passphrase.chars().count();
    let entropy = zxcvbn::zxcvbn(passphrase, &[]);
    let score = u8::from(entropy.score());
    let feedback = entropy.feedback();
    PassphraseStrength {
        score,
        chars,
        acceptable: chars >= MIN_PASSPHRASE_CHARS && score >= MIN_PASSPHRASE_SCORE,
        warning: feedback
            .and_then(zxcvbn::feedback::Feedback::warning)
            .map(|w| w.to_string()),
        suggestions: feedback
            .map(|f| f.suggestions().iter().map(ToString::to_string).collect())
            .unwrap_or_default(),
    }
}

/// The kind of a [`VaultError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum VaultErrorKind {
    /// The passphrase has fewer than [`MIN_PASSPHRASE_CHARS`] characters.
    PassphraseTooShort,
    /// The passphrase has a zxcvbn score below [`MIN_PASSPHRASE_SCORE`].
    PassphraseTooWeak,
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

    /// Make a new empty vault with no wrapper. No passphrase, recovery key
    /// or Nostr key opens it. Keep it in memory only, because the stored
    /// form has no way to open it again.
    ///
    /// # Errors
    ///
    /// Gives a [`VaultError`] when the random source is not available.
    pub fn create_with_no_wrapper() -> Result<Self, VaultError> {
        let key = Key::random()?;
        let vault_id = uuid::Uuid::new_v4().to_string();
        let check = key.seal(&[], check_aad(&vault_id).as_bytes())?;
        Ok(Self {
            file: VaultFile {
                format: FORMAT,
                vault_id,
                check,
                wrappers: Vec::new(),
                records: Vec::new(),
            },
            key,
        })
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
        let wrapper = self.hkdf_wrapper("recovery_hkdf", bytes.as_slice(), RECOVERY_INFO, None)?;
        self.file
            .wrappers
            .retain(|w| !matches!(w, Wrapper::Recovery(_) | Wrapper::RecoveryHkdf(_)));
        self.file.wrappers.push(Wrapper::RecoveryHkdf(wrapper));
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

    /// Keep a wrapper for a passkey. `salt` is the PRF salt of the credential
    /// and `prf_output` is the PRF output for that salt. A wrapper with the
    /// same credential ID is replaced.
    ///
    /// # Errors
    ///
    /// Gives a [`VaultError`] when the random source is not available.
    pub fn add_passkey_wrapper(
        &mut self,
        credential_id: &str,
        salt: &[u8; 32],
        prf_output: &[u8; 32],
    ) -> Result<(), VaultError> {
        let wrapper = self.hkdf_wrapper(
            "passkey",
            prf_output,
            PASSKEY_INFO,
            Some((credential_id, salt)),
        )?;
        self.remove_passkey_wrapper(credential_id);
        self.file.wrappers.push(Wrapper::Passkey {
            credential_id: credential_id.to_owned(),
            inner: wrapper,
        });
        Ok(())
    }

    /// Remove the passkey wrapper of a credential. Gives `true` when the
    /// vault had it.
    pub fn remove_passkey_wrapper(&mut self, credential_id: &str) -> bool {
        let before = self.file.wrappers.len();
        self.file.wrappers.retain(
            |w| !matches!(w, Wrapper::Passkey { credential_id: c, .. } if c == credential_id),
        );
        before != self.file.wrappers.len()
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

    /// Make a wrapper whose key comes from HKDF-SHA-256. A passkey wrapper
    /// gives its credential ID and its PRF salt. A recovery wrapper has a
    /// new random salt.
    fn hkdf_wrapper(
        &self,
        kind: &str,
        input: &[u8],
        info: &str,
        passkey: Option<(&str, &[u8; 32])>,
    ) -> Result<HkdfWrapper, VaultError> {
        let salt = match passkey {
            Some((_, salt)) => salt.to_vec(),
            None => crypto::random_salt_bytes()?.to_vec(),
        };
        let wrapping = Key::expand(input, &salt, info);
        let sealed = wrapping.seal(
            self.key.as_bytes(),
            hkdf_aad(kind, &self.file.vault_id, passkey.map(|(id, _)| id)).as_bytes(),
        )?;
        Ok(HkdfWrapper {
            salt: crypto::encode(&salt),
            sealed,
        })
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
    /// True when at least one wrapper can open the vault.
    #[must_use]
    pub fn has_wrapper(&self) -> bool {
        !self.wrappers.is_empty()
    }

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
        if let Some(bytes) = recovery_bytes(&normal) {
            let found = self.wrappers.iter().find_map(|w| match w {
                Wrapper::RecoveryHkdf(h) => Some(h),
                _ => None,
            });
            if let Some(wrapper) = found {
                let aad = hkdf_aad("recovery_hkdf", &self.vault_id, None);
                return self.open_hkdf(wrapper, bytes.as_slice(), RECOVERY_INFO, &aad);
            }
        }
        self.open_derived("recovery", normal.as_bytes())
    }

    /// Open the vault with the PRF output of a passkey.
    ///
    /// # Errors
    ///
    /// Gives [`VaultErrorKind::WrongKey`] when the vault has no wrapper for
    /// the credential or the PRF output is not correct.
    pub fn open_with_passkey(
        self,
        credential_id: &str,
        prf_output: &[u8; 32],
    ) -> Result<OpenVault, VaultError> {
        let wrapper = self
            .wrappers
            .iter()
            .find_map(|w| match w {
                Wrapper::Passkey {
                    credential_id: c,
                    inner,
                } if c == credential_id => Some(inner),
                _ => None,
            })
            .ok_or_else(VaultError::wrong_key)?;
        let aad = hkdf_aad("passkey", &self.vault_id, Some(credential_id));
        self.open_hkdf(wrapper, prf_output, PASSKEY_INFO, &aad)
    }

    fn open_hkdf(
        &self,
        wrapper: &HkdfWrapper,
        input: &[u8],
        info: &str,
        aad: &str,
    ) -> Result<OpenVault, VaultError> {
        let wrapping = Key::expand(input, &crypto::decode(&wrapper.salt)?, info);
        let key_bytes = wrapping.open(&wrapper.sealed, aad.as_bytes())?;
        let key = Key::from_slice(&key_bytes)?;
        self.clone().open_checked(key)
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
    let strength = passphrase_strength(passphrase);
    if strength.chars < MIN_PASSPHRASE_CHARS {
        return Err(VaultError {
            kind: VaultErrorKind::PassphraseTooShort,
            message: format!(
                "a passphrase must have a minimum of {MIN_PASSPHRASE_CHARS} characters"
            ),
        });
    }
    if strength.score < MIN_PASSPHRASE_SCORE {
        return Err(VaultError {
            kind: VaultErrorKind::PassphraseTooWeak,
            message: format!(
                "the passphrase is too easy to guess (strength {} of 4, minimum {MIN_PASSPHRASE_SCORE})",
                strength.score
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

/// The associated data of an HKDF wrapper. A passkey wrapper adds its
/// credential ID.
fn hkdf_aad(kind: &str, vault_id: &str, credential_id: Option<&str>) -> String {
    let base = wrapper_aad(kind, vault_id);
    match credential_id {
        Some(id) => format!("{base} {id}"),
        None => base,
    }
}

/// The 16 bytes of a recovery key from its 32 hex digits.
fn recovery_bytes(hex: &str) -> Option<Zeroizing<[u8; 16]>> {
    if hex.len() != 32 {
        return None;
    }
    let mut out = Zeroizing::new([0_u8; 16]);
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok()?;
    }
    Some(out)
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
    fn a_vault_with_no_wrapper_keeps_secrets_and_no_key_opens_its_file() {
        let mut vault = OpenVault::create_with_no_wrapper().unwrap();
        vault.put("sftp:example", SECRET).unwrap();
        assert_eq!(
            vault.get("sftp:example").unwrap().unwrap().as_slice(),
            SECRET
        );
        assert!(!vault.file().has_wrapper());
        let error = vault
            .file()
            .clone()
            .open_with_passphrase(PASSPHRASE)
            .unwrap_err();
        assert_eq!(error.kind(), VaultErrorKind::WrongKey);
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
    fn a_long_but_weak_passphrase_gives_an_error() {
        for weak in [
            "passwordpassword",
            "aaaaaaaaaaaaaaaa",
            "qwertyuiopasdf",
            "123456789012",
        ] {
            let strength = passphrase_strength(weak);
            assert!(strength.chars >= MIN_PASSPHRASE_CHARS, "{weak}");
            assert!(!strength.acceptable, "{weak}");
            let error = OpenVault::create(weak).unwrap_err();
            assert_eq!(error.kind(), VaultErrorKind::PassphraseTooWeak, "{weak}");
        }
        let strong = passphrase_strength("Kx9#mQ2!vL7@pR4z");
        assert!(strong.acceptable);
        assert_eq!(strong.score, 4);
    }

    #[test]
    fn the_strength_contains_no_part_of_the_passphrase() {
        let strength = passphrase_strength("monkeymonkey1");
        let text = format!("{strength:?}");
        assert!(!text.contains("monkey"));
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
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.json");
        let vault = vault_with_secret();
        vault.file().save(&path).unwrap();
        let loaded = VaultFile::load(&path).unwrap();
        assert_eq!(&loaded, vault.file());
        assert!(!dir.path().join("vault.json.tmp").exists());
        let opened = loaded.open_with_passphrase(PASSPHRASE).unwrap();
        assert_eq!(
            opened.get("sftp:example").unwrap().unwrap().as_slice(),
            SECRET
        );
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

    const OLD_FILE: &str = include_str!("../tests/fixtures/vault-recovery-v1.json");
    const OLD_KEY: &str = "9411-83f3-1ad1-2dab-8213-ba0f-2c32-cacc";

    #[test]
    fn a_new_recovery_key_uses_the_hkdf_wrapper_after_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vault.json");
        let mut vault = vault_with_secret();
        let recovery = vault.add_recovery_key().unwrap();
        vault.file().save(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"kind\": \"recovery_hkdf\""));
        assert!(!text.contains("\"kind\": \"recovery\""));
        let opened = VaultFile::load(&path)
            .unwrap()
            .open_with_recovery_key(recovery.as_str())
            .unwrap();
        assert_eq!(
            opened.get("sftp:example").unwrap().unwrap().as_slice(),
            SECRET
        );
    }

    #[test]
    fn a_vault_file_with_an_argon2_recovery_wrapper_still_opens() {
        let file = VaultFile::from_json(OLD_FILE).unwrap();
        assert!(file.to_json().contains("\"kind\": \"recovery\""));
        let opened = file.clone().open_with_recovery_key(OLD_KEY).unwrap();
        assert_eq!(
            opened.get("sftp:example").unwrap().unwrap().as_slice(),
            SECRET
        );
        assert_eq!(
            file.open_with_recovery_key("0000-0000").unwrap_err().kind(),
            VaultErrorKind::WrongKey
        );
    }

    #[test]
    fn a_new_recovery_key_replaces_both_older_recovery_wrappers() {
        let mut vault = VaultFile::from_json(OLD_FILE)
            .unwrap()
            .open_with_recovery_key(OLD_KEY)
            .unwrap();
        let first = vault.add_recovery_key().unwrap();
        let second = vault.add_recovery_key().unwrap();
        let file = vault.file().clone();
        let count = file
            .methods()
            .iter()
            .filter(|m| **m == VaultMethod::Recovery)
            .count();
        assert_eq!(count, 1);
        assert!(file.clone().open_with_recovery_key(first.as_str()).is_err());
        assert!(file.open_with_recovery_key(second.as_str()).is_ok());
    }

    #[test]
    fn a_passkey_opens_the_vault_only_with_its_prf_output_and_credential() {
        let mut vault = vault_with_secret();
        vault
            .add_passkey_wrapper("cred-a", &[1; 32], &[7; 32])
            .unwrap();
        let file = vault.file().clone();
        let opened = file.clone().open_with_passkey("cred-a", &[7; 32]).unwrap();
        assert_eq!(
            opened.get("sftp:example").unwrap().unwrap().as_slice(),
            SECRET
        );
        assert_eq!(
            file.clone()
                .open_with_passkey("cred-a", &[8; 32])
                .unwrap_err()
                .kind(),
            VaultErrorKind::WrongKey
        );
        assert_eq!(
            file.open_with_passkey("cred-b", &[7; 32])
                .unwrap_err()
                .kind(),
            VaultErrorKind::WrongKey
        );
    }

    #[test]
    fn a_passkey_wrapper_with_the_same_credential_replaces_the_earlier_one() {
        let mut vault = vault_with_secret();
        vault
            .add_passkey_wrapper("cred-a", &[1; 32], &[7; 32])
            .unwrap();
        vault
            .add_passkey_wrapper("cred-a", &[2; 32], &[9; 32])
            .unwrap();
        let file = vault.file().clone();
        assert_eq!(file.methods().len(), 2);
        assert!(file.clone().open_with_passkey("cred-a", &[7; 32]).is_err());
        assert!(file.open_with_passkey("cred-a", &[9; 32]).is_ok());
    }

    #[test]
    fn a_removed_passkey_wrapper_does_not_open_the_vault() {
        let mut vault = vault_with_secret();
        vault
            .add_passkey_wrapper("cred-a", &[1; 32], &[7; 32])
            .unwrap();
        assert!(vault.remove_passkey_wrapper("cred-a"));
        assert!(!vault.remove_passkey_wrapper("cred-a"));
        assert_eq!(
            vault
                .file()
                .clone()
                .open_with_passkey("cred-a", &[7; 32])
                .unwrap_err()
                .kind(),
            VaultErrorKind::WrongKey
        );
    }

    #[test]
    fn methods_give_no_salt_and_no_ciphertext() {
        let mut vault = vault_with_secret();
        vault.add_recovery_key().unwrap();
        vault.add_nostr_wrapper("npub-hex", "nip44-ciphertext");
        vault
            .add_passkey_wrapper("cred-a", &[1; 32], &[7; 32])
            .unwrap();
        let methods = vault.file().methods();
        assert_eq!(
            methods,
            vec![
                VaultMethod::Passphrase,
                VaultMethod::Recovery,
                VaultMethod::Nostr {
                    pubkey: "npub-hex".to_owned()
                },
                VaultMethod::Passkey {
                    credential_id: "cred-a".to_owned()
                },
            ]
        );
        let text = format!("{methods:?}");
        assert!(!text.contains("nip44-ciphertext"));
        for wrapper in &vault.file().wrappers {
            if let Wrapper::Passkey { inner, .. } = wrapper {
                assert!(!text.contains(&inner.salt));
                assert!(!text.contains(&inner.sealed.ciphertext));
            }
        }
    }
}

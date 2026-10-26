//! Keys, key derivation and encryption for the vault (ADR 0013).

use std::fmt;

use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroizing;

use super::VaultError;

/// The length of the vault key in bytes.
const KEY_LEN: usize = 32;
/// The length of an XChaCha20-Poly1305 nonce in bytes.
const NONCE_LEN: usize = 24;
/// The length of an Argon2id salt in bytes.
const SALT_LEN: usize = 16;

/// The Argon2id parameters of ADR 0013: 64 MiB, 3 passes, 1 lane.
pub(crate) const KDF_MEMORY_KIB: u32 = 64 * 1024;
pub(crate) const KDF_PASSES: u32 = 3;
pub(crate) const KDF_LANES: u32 = 1;

/// A 32-byte key. Its memory is cleared when it goes out of scope.
#[derive(Clone)]
pub(crate) struct Key(Zeroizing<[u8; KEY_LEN]>);

impl Key {
    pub(crate) fn random() -> Result<Self, VaultError> {
        let mut bytes = Zeroizing::new([0_u8; KEY_LEN]);
        random_fill(bytes.as_mut_slice())?;
        Ok(Self(bytes))
    }

    pub(crate) fn from_slice(bytes: &[u8]) -> Result<Self, VaultError> {
        let array: [u8; KEY_LEN] = bytes.try_into().map_err(|_| VaultError::wrong_key())?;
        Ok(Self(Zeroizing::new(array)))
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Derive a key from a passphrase with Argon2id.
    pub(crate) fn derive(
        passphrase: &[u8],
        salt: &[u8],
        memory_kib: u32,
        passes: u32,
        lanes: u32,
    ) -> Result<Self, VaultError> {
        let params = Params::new(memory_kib, passes, lanes, Some(KEY_LEN))
            .map_err(|_| VaultError::corrupt("the key derivation parameters are not valid"))?;
        let mut out = Zeroizing::new([0_u8; KEY_LEN]);
        Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
            .hash_password_into(passphrase, salt, out.as_mut_slice())
            .map_err(|_| VaultError::corrupt("the key derivation failed"))?;
        Ok(Self(out))
    }

    /// Derive a key from input with high entropy with HKDF-SHA-256.
    ///
    /// Use it only for input that is not a passphrase, such as a recovery key
    /// or a PRF output.
    pub(crate) fn expand(input: &[u8], salt: &[u8], info: &str) -> Self {
        let mut out = Zeroizing::new([0_u8; KEY_LEN]);
        Hkdf::<Sha256>::new(Some(salt), input)
            .expand(info.as_bytes(), out.as_mut_slice())
            .expect("32 bytes are a valid HKDF-SHA-256 output length");
        Self(out)
    }

    /// Encrypt `plaintext` with a new random nonce.
    pub(crate) fn seal(&self, plaintext: &[u8], aad: &[u8]) -> Result<Sealed, VaultError> {
        let mut nonce = [0_u8; NONCE_LEN];
        random_fill(&mut nonce)?;
        let ciphertext = self
            .cipher()
            .encrypt(
                &XNonce::from(nonce),
                Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .map_err(|_| VaultError::corrupt("encryption failed"))?;
        Ok(Sealed {
            nonce: STANDARD.encode(nonce),
            ciphertext: STANDARD.encode(ciphertext),
        })
    }

    /// Decrypt a sealed value. An incorrect key or `aad` gives
    /// [`VaultError::wrong_key`].
    pub(crate) fn open(
        &self,
        sealed: &Sealed,
        aad: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, VaultError> {
        let nonce: [u8; NONCE_LEN] = decode(&sealed.nonce)?
            .try_into()
            .map_err(|_| VaultError::corrupt("a nonce has an incorrect length"))?;
        let ciphertext = decode(&sealed.ciphertext)?;
        self.cipher()
            .decrypt(
                &XNonce::from(nonce),
                Payload {
                    msg: &ciphertext,
                    aad,
                },
            )
            .map(Zeroizing::new)
            .map_err(|_| VaultError::wrong_key())
    }

    fn cipher(&self) -> XChaCha20Poly1305 {
        XChaCha20Poly1305::new(&(*self.0).into())
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Key(..)")
    }
}

/// A nonce and a ciphertext, both in standard base64.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Sealed {
    pub(crate) nonce: String,
    pub(crate) ciphertext: String,
}

pub(crate) fn random_salt_bytes() -> Result<[u8; SALT_LEN], VaultError> {
    let mut salt = [0_u8; SALT_LEN];
    random_fill(&mut salt)?;
    Ok(salt)
}

pub(crate) fn random_salt() -> Result<String, VaultError> {
    let mut salt = [0_u8; SALT_LEN];
    random_fill(&mut salt)?;
    Ok(STANDARD.encode(salt))
}

pub(crate) fn random_fill(bytes: &mut [u8]) -> Result<(), VaultError> {
    getrandom::fill(bytes).map_err(|_| VaultError::corrupt("the random source is not available"))
}

pub(crate) fn decode(text: &str) -> Result<Vec<u8>, VaultError> {
    STANDARD
        .decode(text)
        .map_err(|_| VaultError::corrupt("a value is not valid base64"))
}

pub(crate) fn encode(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

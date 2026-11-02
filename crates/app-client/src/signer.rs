//! The signer of an app (ADR 0110 items 1 and 2).

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use nostr::prelude::{
    Event, Keys, Kind, PublicKey, SecretKey, Tag, Timestamp, UnsignedEvent, nip44,
};
use secp256k1::Secp256k1;
use zeroize::Zeroizing;

use crate::error::Error;

/// What the client needs from the key of the app. An app can give its own
/// signer, for example one for the key store of the operating system. Then
/// the private key never enters this crate (ADR 0110 item 2).
pub trait AppSigner: Send + Sync {
    /// The public key of the app.
    fn public_key(&self) -> PublicKey;

    /// Sign an event of kind 27235 with the tags `tags` and `created_at`
    /// seconds since the Unix epoch.
    ///
    /// # Errors
    ///
    /// Gives [`Error::Signer`] when the signature fails. The text must hold no
    /// part of the key.
    fn sign(&self, kind: u16, tags: Vec<Tag>, created_at: u64) -> Result<Event, Error>;

    /// Decrypt a NIP-44 version 2 message from `sender` with the private key
    /// of the app. The client uses it for the wrapper of a grant vault key.
    ///
    /// # Errors
    ///
    /// Gives [`Error::Signer`] when the message cannot be decrypted.
    fn nip44_decrypt(&self, sender: &PublicKey, payload: &str) -> Result<Zeroizing<String>, Error>;
}

/// A signer that keeps a key in memory. The key comes from a file, or it is
/// new.
#[derive(Clone)]
pub struct MemorySigner {
    keys: Keys,
}

impl std::fmt::Debug for MemorySigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemorySigner")
            .field("public_key", &self.keys.public_key().to_hex())
            .finish_non_exhaustive()
    }
}

impl MemorySigner {
    /// A signer with a new random key. The key is not saved.
    ///
    /// # Errors
    ///
    /// Gives [`Error::Signer`] when the operating system gives no random bytes.
    pub fn generate() -> Result<Self, Error> {
        loop {
            let mut bytes = Zeroizing::new([0_u8; 32]);
            getrandom::fill(bytes.as_mut_slice())
                .map_err(|_| signer_error("the random source is not available"))?;
            // A value outside the curve order is very rare. Another try is correct.
            if let Ok(secret) = SecretKey::from_slice(bytes.as_ref()) {
                return Ok(Self::from_secret_key(secret));
            }
        }
    }

    /// A signer with a key that the app already has.
    #[must_use]
    pub fn from_secret_key(secret: SecretKey) -> Self {
        Self {
            keys: Keys::new(secret),
        }
    }
}

impl AppSigner for MemorySigner {
    fn public_key(&self) -> PublicKey {
        self.keys.public_key()
    }

    fn sign(&self, kind: u16, tags: Vec<Tag>, created_at: u64) -> Result<Event, Error> {
        let fail = || Error::Signer("the event cannot be signed".to_owned());
        let unsigned = UnsignedEvent::new(
            self.keys.public_key(),
            Timestamp::from_secs(created_at),
            Kind::Custom(kind),
            tags,
            "",
        );
        let mut aux = [0_u8; 32];
        getrandom::fill(&mut aux).map_err(|_| fail())?;
        let id = unsigned.compute_id();
        let signature =
            self.keys
                .sign_schnorr_with_aux_rand(&Secp256k1::signing_only(), id.as_bytes(), &aux);
        unsigned.add_signature(signature).map_err(|_| fail())
    }

    fn nip44_decrypt(&self, sender: &PublicKey, payload: &str) -> Result<Zeroizing<String>, Error> {
        nip44::decrypt(self.keys.secret_key(), sender, payload)
            .map(Zeroizing::new)
            .map_err(|_| Error::Signer("the message cannot be decrypted".to_owned()))
    }
}

/// A signer that keeps its key in a file with mode 0600, in a folder with
/// mode 0700 (ADR 0110 item 2). The file holds the private key as 64
/// hexadecimal characters. Use one file for each pairing, and one pairing for
/// each copy of the app.
///
/// ```
/// use studio_app_client::{AppSigner, FileSigner};
///
/// let folder = tempfile::tempdir()?;
/// let path = folder.path().join("keys").join("studio.key");
/// // The first call makes the key and the file. A later call reads the file.
/// let signer = FileSigner::load_or_create(&path)?;
/// let again = FileSigner::load_or_create(&path)?;
/// assert_eq!(signer.public_key(), again.public_key());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct FileSigner {
    inner: MemorySigner,
    path: PathBuf,
}

impl FileSigner {
    /// Read the key file, or make a new key and save it. A new file gets mode
    /// 0600, and a new folder gets mode 0700. The function refuses a file or
    /// folder that other users can read.
    ///
    /// # Errors
    ///
    /// Gives [`Error::Signer`] when the file cannot be read or written, when
    /// its mode is too open, or when it does not hold a key. The text holds
    /// no part of the key.
    pub fn load_or_create(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        match fs::read_to_string(path) {
            Ok(text) => {
                let text = Zeroizing::new(text);
                check_modes(path)?;
                let secret = SecretKey::from_hex(text.trim())
                    .map_err(|_| signer_error("the key file does not hold a key"))?;
                Ok(Self {
                    inner: MemorySigner::from_secret_key(secret),
                    path: path.to_owned(),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Self::create(path),
            Err(_) => Err(signer_error("the key file cannot be read")),
        }
    }

    /// The path of the key file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn create(path: &Path) -> Result<Self, Error> {
        if let Some(folder) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            make_folder(folder)?;
        }
        let inner = MemorySigner::generate()?;
        let secret = Zeroizing::new(inner.keys.secret_key().to_secret_hex());
        let mut file = open_new(path)?;
        file.write_all(secret.as_bytes())
            .and_then(|()| file.write_all(b"\n"))
            .and_then(|()| file.sync_all())
            .map_err(|_| signer_error("the key file cannot be written"))?;
        check_modes(path)?;
        Ok(Self {
            inner,
            path: path.to_owned(),
        })
    }
}

impl AppSigner for FileSigner {
    fn public_key(&self) -> PublicKey {
        self.inner.public_key()
    }

    fn sign(&self, kind: u16, tags: Vec<Tag>, created_at: u64) -> Result<Event, Error> {
        self.inner.sign(kind, tags, created_at)
    }

    fn nip44_decrypt(&self, sender: &PublicKey, payload: &str) -> Result<Zeroizing<String>, Error> {
        self.inner.nip44_decrypt(sender, payload)
    }
}

fn signer_error(why: &str) -> Error {
    Error::Signer(why.to_owned())
}

#[cfg(unix)]
fn make_folder(folder: &Path) -> Result<(), Error> {
    use std::os::unix::fs::DirBuilderExt as _;
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(folder)
        .map_err(|_| signer_error("the key folder cannot be made"))
}

#[cfg(not(unix))]
fn make_folder(folder: &Path) -> Result<(), Error> {
    fs::create_dir_all(folder).map_err(|_| signer_error("the key folder cannot be made"))
}

#[cfg(unix)]
fn open_new(path: &Path) -> Result<fs::File, Error> {
    use std::os::unix::fs::OpenOptionsExt as _;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| signer_error("the key file cannot be made"))
}

#[cfg(not(unix))]
fn open_new(path: &Path) -> Result<fs::File, Error> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| signer_error("the key file cannot be made"))
}

/// Refuse a key file that other users can read, or a folder that they can
/// open.
#[cfg(unix)]
fn check_modes(path: &Path) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt as _;
    let mode = |p: &Path| fs::metadata(p).map(|m| m.permissions().mode() & 0o777);
    let file = mode(path).map_err(|_| signer_error("the key file cannot be read"))?;
    if file & 0o077 != 0 {
        return Err(signer_error(
            "the key file is readable by other users: set its mode to 0600",
        ));
    }
    if let Some(folder) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        let folder = mode(folder).map_err(|_| signer_error("the key folder cannot be read"))?;
        if folder & 0o077 != 0 {
            return Err(signer_error(
                "the key folder is open to other users: set its mode to 0700, or use a folder only for keys",
            ));
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_modes(_path: &Path) -> Result<(), Error> {
    Ok(())
}

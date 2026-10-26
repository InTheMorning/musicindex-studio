//! The SFTP connector (ADR 0047).
//!
//! The connector logs in only to the host key of its row. A write goes to a
//! hidden temporary file and then to the target with a rename or a hard
//! link. Thus the target has all of the new data or no change.

use std::fmt;
use std::io::{ErrorKind, Read};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::digest::{content_version, finish_hex};

use super::sftp_wire::{Attrs, FileKind, Handle, SftpWire};
use super::{
    Connector, ConnectorError, ConnectorErrorKind, Expected, FileList, FileRead, LIST_MAX_DEPTH,
    LIST_MAX_FILES, ListedFile, Stamp, Version, length_error, relative_path,
};

/// The maximum time for a TCP connection.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
/// The maximum time for one SSH operation, in milliseconds.
const SESSION_TIMEOUT_MS: u32 = 120_000;
/// The size of each part that `write_from` reads.
const PART: usize = 32 * 1024;
/// The size of each request when Studio reads a file.
const READ_CHUNK: usize = 1024 * 1024;
/// The most bytes that `read` accepts from one file.
const READ_LIMIT: usize = 1024 * 1024 * 1024;
/// The OpenSSH extension that gives a durable write.
const FSYNC: &str = "fsync@openssh.com";
/// The OpenSSH extension that gives a link that fails when the target exists.
const HARDLINK: &str = "hardlink@openssh.com";
/// The OpenSSH extension that replaces a target in one step.
const POSIX_RENAME: &str = "posix-rename@openssh.com";

/// The login method of an SFTP host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SftpLogin {
    /// A private key. The vault holds the key and its passphrase.
    Key,
    /// A password. The vault holds the password.
    Password,
    /// The ssh-agent of the user. The vault holds no secret.
    Agent,
}

/// The configuration of an SFTP host. It holds no secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SftpConfig {
    /// The host name or the address.
    pub host: String,
    /// The TCP port.
    pub port: u16,
    /// The user name.
    pub user: String,
    /// The root folder on the host. A path is below it.
    pub root: String,
    /// The public address of the root folder, when the host has one.
    #[serde(default)]
    pub base_url: Option<String>,
    /// The login method.
    pub login: SftpLogin,
    /// The SHA-256 fingerprint of the host key that the user accepted, as
    /// `SHA256:` and the base64 text.
    pub host_key: String,
}

/// The secret of an SFTP host.
#[derive(Clone)]
pub enum SftpCredentials {
    /// An OpenSSH private key and its passphrase, when it has one.
    Key {
        /// The text of the private key.
        private_key: Zeroizing<String>,
        /// The passphrase of the key.
        passphrase: Option<Zeroizing<String>>,
    },
    /// A password.
    Password(Zeroizing<String>),
    /// The ssh-agent of the user.
    Agent,
}

impl fmt::Debug for SftpCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The text shows the method only. It shows no secret.
        let method = match self {
            Self::Key { .. } => "Key",
            Self::Password(_) => "Password",
            Self::Agent => "Agent",
        };
        f.debug_struct("SftpCredentials")
            .field("login", &method)
            .finish_non_exhaustive()
    }
}

impl SftpCredentials {
    /// Make the credentials of `login` from the secret JSON of the vault.
    /// A key has `{"private_key": "...", "passphrase": "..."}` and a
    /// password has `{"password": "..."}`. The agent has no secret.
    ///
    /// # Errors
    ///
    /// Gives [`ConnectorErrorKind::InvalidConfig`] when the login needs a
    /// secret and `secret` is missing or is not the right JSON. The message
    /// holds no part of the secret.
    pub fn from_secret(login: SftpLogin, secret: Option<&[u8]>) -> Result<Self, ConnectorError> {
        #[derive(Deserialize)]
        struct KeySecret {
            private_key: String,
            #[serde(default)]
            passphrase: Option<String>,
        }
        #[derive(Deserialize)]
        struct PasswordSecret {
            password: String,
        }
        let invalid =
            |message: &str| ConnectorError::new(ConnectorErrorKind::InvalidConfig, message);
        match login {
            SftpLogin::Agent => Ok(Self::Agent),
            SftpLogin::Key => {
                let secret = secret.ok_or_else(|| invalid("the SFTP connector needs a secret"))?;
                let parsed: KeySecret = serde_json::from_slice(secret).map_err(|_| {
                    invalid("the secret of the connector must be JSON with private_key")
                })?;
                Ok(Self::Key {
                    private_key: Zeroizing::new(parsed.private_key),
                    passphrase: parsed.passphrase.map(Zeroizing::new),
                })
            }
            SftpLogin::Password => {
                let secret = secret.ok_or_else(|| invalid("the SFTP connector needs a secret"))?;
                let parsed: PasswordSecret = serde_json::from_slice(secret).map_err(|_| {
                    invalid("the secret of the connector must be JSON with password")
                })?;
                Ok(Self::Password(Zeroizing::new(parsed.password)))
            }
        }
    }

    /// The login method of the credentials.
    fn login(&self) -> SftpLogin {
        match self {
            Self::Key { .. } => SftpLogin::Key,
            Self::Password(_) => SftpLogin::Password,
            Self::Agent => SftpLogin::Agent,
        }
    }
}

/// An open session: the SFTP requests and the SSH session below them.
struct Connection {
    // The wire drops first, then the session.
    wire: SftpWire,
    _session: ssh2::Session,
}

/// A folder on an SFTP host.
///
/// The version marker is the SHA-256 of the file content. The stamp is the
/// size and the modification time in seconds. The connector keeps one
/// session while it is in use. A write has a window between its check and
/// its rename, in which a different client can change the file
/// (ADR 0047 item 9).
pub struct SftpHost {
    config: SftpConfig,
    credentials: SftpCredentials,
    connection: Mutex<Option<Connection>>,
}

impl fmt::Debug for SftpHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The credentials are not in the text.
        f.debug_struct("SftpHost")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

/// A path below the root, checked on the host.
struct Resolved {
    /// The full path on the host.
    full: String,
    /// The attributes of the file, when it exists.
    attrs: Option<Attrs>,
    /// How many segments exist on the host.
    existing: usize,
}

impl SftpHost {
    /// The connector kind in the collection.
    pub const KIND: &'static str = "sftp";

    /// Make a connector. It does not connect.
    ///
    /// # Errors
    ///
    /// Gives [`ConnectorErrorKind::InvalidConfig`] in these conditions:
    /// the host, the user or the root is empty, the host key is not a
    /// `SHA256:` fingerprint, or the credentials are not of the login
    /// method.
    pub fn new(config: SftpConfig, credentials: SftpCredentials) -> Result<Self, ConnectorError> {
        let invalid =
            |message: &str| ConnectorError::new(ConnectorErrorKind::InvalidConfig, message);
        if config.host.is_empty() || config.user.is_empty() || config.root.is_empty() {
            return Err(invalid("the host, the user and the root must not be empty"));
        }
        if config.port == 0 {
            return Err(invalid("the port must not be 0"));
        }
        if !config.host_key.starts_with("SHA256:") {
            return Err(invalid("the host key must be a SHA256 fingerprint"));
        }
        if credentials.login() != config.login {
            return Err(invalid("the secret does not match the login method"));
        }
        Ok(Self {
            config,
            credentials,
            connection: Mutex::new(None),
        })
    }

    /// The configuration text for the collection.
    ///
    /// # Panics
    ///
    /// The function does not panic, because the configuration is only text.
    #[must_use]
    pub fn config(&self) -> String {
        serde_json::to_string(&self.config).expect("the configuration is JSON")
    }

    /// Make the connector from the configuration text of a row and the
    /// secret JSON of the vault.
    pub(crate) fn from_config(config: &str, secret: Option<&[u8]>) -> Result<Self, ConnectorError> {
        let config: SftpConfig = serde_json::from_str(config)
            .map_err(|e| ConnectorError::new(ConnectorErrorKind::InvalidConfig, e.to_string()))?;
        let credentials = SftpCredentials::from_secret(config.login, secret)?;
        Self::new(config, credentials)
    }

    /// The fingerprint of the host key of a server, as `SHA256:` and the
    /// base64 text. The function does the TCP connection and the SSH
    /// handshake. It does not log in.
    ///
    /// # Errors
    ///
    /// Gives [`ConnectorErrorKind::Unavailable`] when Studio cannot connect
    /// or the handshake fails.
    pub fn fingerprint_of(host: &str, port: u16) -> Result<String, ConnectorError> {
        let session = handshake(host, port)?;
        fingerprint(&session)
    }

    /// Connect and look at the root folder. The route that adds a host
    /// uses this check before it keeps a secret. The root can be a symbolic
    /// link to a folder, because the operations go through the root.
    ///
    /// # Errors
    ///
    /// Gives [`ConnectorErrorKind::NotFound`] when the root does not exist
    /// or is not a folder. The connection errors are those of the other
    /// requests.
    pub fn check_root(&self) -> Result<(), ConnectorError> {
        let root = self.full_path(&[]);
        let not_a_folder = || {
            ConnectorError::new(
                ConnectorErrorKind::NotFound,
                format!("{} is not a folder on the host", self.config.root),
            )
        };
        self.with_wire(|wire| match wire.stat(&root)?.map(|attrs| attrs.kind) {
            Some(FileKind::Folder) => Ok(()),
            // Only a folder opens as a folder.
            Some(FileKind::SymbolicLink) => match wire.read_dir(&root) {
                Ok(_) => Ok(()),
                Err(error) if error.kind() == ConnectorErrorKind::Unavailable => Err(error),
                Err(_) => Err(not_a_folder()),
            },
            _ => Err(not_a_folder()),
        })
    }

    /// Connect, check the host key, log in and open the SFTP requests.
    fn connect(&self) -> Result<Connection, ConnectorError> {
        let session = handshake(&self.config.host, self.config.port)?;
        let seen = fingerprint(&session)?;
        if seen != self.config.host_key {
            return Err(ConnectorError::new(
                ConnectorErrorKind::Unauthorized,
                format!("the host key changed: {seen}"),
            ));
        }
        self.log_in(&session)?;
        let wire = SftpWire::open(&session)?;
        if !wire.has(POSIX_RENAME) {
            return Err(ConnectorError::new(
                ConnectorErrorKind::NoConditions,
                "the server has no posix-rename, so Studio cannot write to it safely",
            ));
        }
        Ok(Connection {
            wire,
            _session: session,
        })
    }

    /// The login of the SSH session.
    fn log_in(&self, session: &ssh2::Session) -> Result<(), ConnectorError> {
        let user = self.config.user.as_str();
        let refused = || {
            ConnectorError::new(
                ConnectorErrorKind::Unauthorized,
                "the SFTP host refused the login",
            )
        };
        match &self.credentials {
            SftpCredentials::Key {
                private_key,
                passphrase,
            } => session
                .userauth_pubkey_memory(
                    user,
                    None,
                    private_key.as_str(),
                    passphrase.as_deref().map(String::as_str),
                )
                .map_err(|_| refused())?,
            SftpCredentials::Password(password) => session
                .userauth_password(user, password.as_str())
                .map_err(|_| refused())?,
            SftpCredentials::Agent => {
                let no_agent = |_| {
                    ConnectorError::new(
                        ConnectorErrorKind::Unauthorized,
                        "Studio cannot use the ssh-agent",
                    )
                };
                let mut agent = session.agent().map_err(no_agent)?;
                agent.connect().map_err(no_agent)?;
                agent.list_identities().map_err(no_agent)?;
                let identities = agent.identities().map_err(no_agent)?;
                // Try each identity of the agent until one is accepted.
                for identity in &identities {
                    if agent.userauth(user, identity).is_ok() && session.authenticated() {
                        return Ok(());
                    }
                }
                return Err(refused());
            }
        }
        if session.authenticated() {
            Ok(())
        } else {
            Err(refused())
        }
    }

    /// Do `work` on the open session. The function connects when it has no
    /// session. After a channel error or an `Io` error, the function closes
    /// the session, so the next call connects again. An `Io` error can come
    /// from a protocol error, after which the responses are out of sequence.
    fn with_wire<T>(
        &self,
        work: impl FnOnce(&mut SftpWire) -> Result<T, ConnectorError>,
    ) -> Result<T, ConnectorError> {
        let mut guard = self
            .connection
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if guard.is_none() {
            *guard = Some(self.connect()?);
        }
        let Some(connection) = guard.as_mut() else {
            return Err(unavailable("the SFTP session is not open"));
        };
        let result = work(&mut connection.wire);
        if matches!(
            &result,
            Err(e) if matches!(e.kind(), ConnectorErrorKind::Unavailable | ConnectorErrorKind::Io)
        ) {
            *guard = None;
        }
        result
    }

    /// The full path on the host of the root and the `segments` below it.
    fn full_path(&self, segments: &[&str]) -> String {
        let mut full = self.config.root.trim_end_matches('/').to_owned();
        for segment in segments {
            full.push('/');
            full.push_str(segment);
        }
        // A root of only `/` gives an empty text, which is the root folder.
        if full.is_empty() {
            full.push('/');
        }
        full
    }

    /// Look at each segment of the path on the host. A symbolic link gives
    /// `InvalidPath`.
    fn resolve(
        wire: &mut SftpWire,
        host: &Self,
        segments: &[&str],
    ) -> Result<Resolved, ConnectorError> {
        for index in 0..segments.len() {
            let full = host.full_path(&segments[..=index]);
            let Some(attrs) = wire.stat(&full)? else {
                return Ok(Resolved {
                    full: host.full_path(segments),
                    attrs: None,
                    existing: index,
                });
            };
            if attrs.kind == FileKind::SymbolicLink {
                return Err(ConnectorError::new(
                    ConnectorErrorKind::InvalidPath,
                    format!("{} is a symbolic link", segments[..=index].join("/")),
                ));
            }
            if index + 1 == segments.len() {
                return Ok(Resolved {
                    full,
                    attrs: Some(attrs),
                    existing: segments.len(),
                });
            }
            if attrs.kind != FileKind::Folder {
                return Err(ConnectorError::new(
                    ConnectorErrorKind::InvalidPath,
                    format!("{} is not a folder", segments[..=index].join("/")),
                ));
            }
        }
        Ok(Resolved {
            full: host.full_path(segments),
            attrs: None,
            existing: 0,
        })
    }

    /// The write of `length` bytes of `reader` with the sequence of
    /// ADR 0047 item 8.
    fn write_checked(
        &self,
        path: &str,
        reader: &mut dyn Read,
        length: u64,
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        let segments = segments_of(path)?;
        self.with_wire(|wire| {
            let target = Self::resolve(wire, self, &segments)?;
            let conflict = || {
                ConnectorError::new(
                    ConnectorErrorKind::Conflict,
                    format!("{path} changed on the host"),
                )
            };
            match expected {
                Expected::Absent => {
                    if target.attrs.is_some() {
                        return Err(conflict());
                    }
                }
                Expected::Version(want) => {
                    let current = match &target.attrs {
                        Some(attrs) if attrs.kind == FileKind::File => {
                            read_file(wire, &target.full)?
                        }
                        _ => None,
                    };
                    if current.is_none_or(|bytes| &content_version(&bytes) != want) {
                        return Err(conflict());
                    }
                }
            }
            // Make each missing folder of the path.
            for index in target.existing..segments.len() - 1 {
                let folder = self.full_path(&segments[..=index]);
                if let Err(error) = wire.mkdir(&folder) {
                    // A different client can make the folder in this time.
                    let is_folder = matches!(
                        wire.stat(&folder),
                        Ok(Some(Attrs {
                            kind: FileKind::Folder,
                            ..
                        }))
                    );
                    if !is_folder {
                        return Err(error);
                    }
                }
            }
            let folder = self.full_path(&segments[..segments.len() - 1]);
            let name = segments[segments.len() - 1];
            let temporary = format!("{folder}/.{name}.{}.studio-tmp", uuid::Uuid::new_v4());
            let version = upload(wire, &temporary, reader, length, path)?;
            let moved = match expected {
                Expected::Absent => link_absent(wire, &temporary, &target.full, path),
                Expected::Version(_) => wire.posix_rename(&temporary, &target.full),
            };
            if let Err(error) = moved {
                let _ = wire.remove(&temporary);
                return Err(error);
            }
            Ok(version)
        })
    }
}

/// The segments of a relative path. A segment is not empty and not `.`.
fn segments_of(path: &str) -> Result<Vec<&str>, ConnectorError> {
    relative_path(path)?;
    let segments: Vec<&str> = path.split('/').collect();
    if segments.iter().any(|s| s.is_empty() || *s == ".") {
        return Err(ConnectorError::new(
            ConnectorErrorKind::InvalidPath,
            format!("the path {path:?} is not a relative path in the connector"),
        ));
    }
    Ok(segments)
}

fn unavailable(text: &str) -> ConnectorError {
    ConnectorError::new(ConnectorErrorKind::Unavailable, text)
}

/// The TCP connection and the SSH handshake.
fn handshake(host: &str, port: u16) -> Result<ssh2::Session, ConnectorError> {
    let cannot =
        |e: &dyn fmt::Display| unavailable(&format!("cannot connect to {host}:{port}: {e}"));
    let addresses = (host, port).to_socket_addrs().map_err(|e| cannot(&e))?;
    let mut last = None;
    let mut stream = None;
    for address in addresses {
        match TcpStream::connect_timeout(&address, CONNECT_TIMEOUT) {
            Ok(connected) => {
                stream = Some(connected);
                break;
            }
            Err(e) => last = Some(e),
        }
    }
    let Some(stream) = stream else {
        return Err(match last {
            Some(e) => cannot(&e),
            None => unavailable(&format!("cannot find the address of {host}")),
        });
    };
    let mut session = ssh2::Session::new().map_err(|e| cannot(&e))?;
    session.set_timeout(SESSION_TIMEOUT_MS);
    session.set_tcp_stream(stream);
    session.handshake().map_err(|e| cannot(&e))?;
    Ok(session)
}

/// The fingerprint of the host key of a session.
fn fingerprint(session: &ssh2::Session) -> Result<String, ConnectorError> {
    let hash = session
        .host_key_hash(ssh2::HashType::Sha256)
        .ok_or_else(|| unavailable("the server gave no host key"))?;
    Ok(format!("SHA256:{}", STANDARD_NO_PAD.encode(hash)))
}

/// Read all of a file. Gives `None` when it does not exist.
fn read_file(wire: &mut SftpWire, full: &str) -> Result<Option<Vec<u8>>, ConnectorError> {
    let handle = match wire.open_read(full) {
        Ok(handle) => handle,
        Err(e) if e.kind() == ConnectorErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let read = read_all(wire, &handle);
    let closed = wire.close(&handle);
    let bytes = read?;
    closed?;
    Ok(Some(bytes))
}

fn read_all(wire: &mut SftpWire, handle: &Handle) -> Result<Vec<u8>, ConnectorError> {
    let mut bytes = Vec::new();
    loop {
        let part = wire.read(handle, bytes.len() as u64, READ_CHUNK)?;
        let end = part.len() < READ_CHUNK;
        bytes.extend_from_slice(&part);
        if bytes.len() > READ_LIMIT {
            return Err(ConnectorError::new(
                ConnectorErrorKind::Io,
                "the file is too large to read",
            ));
        }
        if end {
            return Ok(bytes);
        }
    }
}

/// Write `length` bytes of `reader`, and no more, to a new file `temporary`.
/// Gives the version of the bytes. A failure removes the file.
fn upload(
    wire: &mut SftpWire,
    temporary: &str,
    reader: &mut dyn Read,
    length: u64,
    path: &str,
) -> Result<Version, ConnectorError> {
    let handle = wire.open_create(temporary)?;
    let copied = copy_parts(wire, &handle, reader, length, path).and_then(|hex| {
        if wire.has(FSYNC) {
            wire.fsync(&handle)?;
        }
        Ok(hex)
    });
    let closed = wire.close(&handle);
    match (copied, closed) {
        (Ok(hex), Ok(())) => Ok(Version::new(format!("sha256:{hex}"))),
        (Err(error), _) | (_, Err(error)) => {
            let _ = wire.remove(temporary);
            Err(error)
        }
    }
}

/// Write `length` bytes of `reader` to `handle`, in parts of 32 KiB. Gives
/// the SHA-256 of the bytes as lowercase hexadecimal.
fn copy_parts(
    wire: &mut SftpWire,
    handle: &Handle,
    reader: &mut dyn Read,
    length: u64,
    path: &str,
) -> Result<String, ConnectorError> {
    let read_error = |e: &std::io::Error| {
        ConnectorError::new(
            ConnectorErrorKind::Io,
            format!("cannot read the data for {path}: {e}"),
        )
    };
    let mut hash = Sha256::new();
    let mut part = vec![0_u8; PART];
    let mut offset = 0_u64;
    while offset < length {
        let want = usize::try_from(length - offset).map_or(PART, |n| n.min(PART));
        let count = match reader.read(&mut part[..want]) {
            Ok(count) => count,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(read_error(&e)),
        };
        if count == 0 {
            return Err(length_error(path, length));
        }
        hash.update(&part[..count]);
        wire.write(handle, offset, &part[..count])?;
        offset += count as u64;
    }
    // The reader must have no more bytes.
    let mut extra = [0_u8; 1];
    loop {
        match reader.read(&mut extra) {
            Ok(0) => break,
            Ok(_) => return Err(length_error(path, length)),
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => return Err(read_error(&e)),
        }
    }
    Ok(finish_hex(hash))
}

/// Move the temporary file to a target that must not exist. A conflict
/// removes the temporary file.
fn link_absent(
    wire: &mut SftpWire,
    temporary: &str,
    target: &str,
    path: &str,
) -> Result<(), ConnectorError> {
    let made = if wire.has(HARDLINK) {
        wire.hardlink(temporary, target)
    } else {
        wire.rename(temporary, target)
    };
    match made {
        Ok(()) => {
            // The target has the data. A temporary file that stays is hidden
            // and causes no problem. Thus an error of the remove is not an
            // error of the write.
            if wire.has(HARDLINK) {
                let _ = wire.remove(temporary);
            }
            Ok(())
        }
        Err(e) if e.kind() == ConnectorErrorKind::Conflict => Err(ConnectorError::new(
            ConnectorErrorKind::Conflict,
            format!("{path} changed on the host"),
        )),
        Err(e) => Err(e),
    }
}

/// The stamp of a file: its size and its modification time.
fn listed(path: String, attrs: &Attrs) -> ListedFile {
    let size = attrs.size.unwrap_or(0);
    ListedFile {
        path,
        size,
        stamp: Stamp::new(format!("{size}-{}", attrs.mtime.unwrap_or(0))),
    }
}

impl Connector for SftpHost {
    fn read(&self, path: &str) -> Result<Option<FileRead>, ConnectorError> {
        let segments = segments_of(path)?;
        self.with_wire(|wire| {
            let target = Self::resolve(wire, self, &segments)?;
            match target.attrs {
                Some(Attrs {
                    kind: FileKind::File,
                    ..
                }) => {}
                _ => return Ok(None),
            }
            Ok(read_file(wire, &target.full)?.map(|bytes| {
                let version = content_version(&bytes);
                FileRead { bytes, version }
            }))
        })
    }

    fn write(
        &self,
        path: &str,
        bytes: &[u8],
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        let mut reader = bytes;
        self.write_checked(path, &mut reader, bytes.len() as u64, expected)
    }

    fn write_from(
        &self,
        path: &str,
        reader: &mut dyn Read,
        length: u64,
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        self.write_checked(path, reader, length, expected)
    }

    fn public_url(&self, path: &str) -> Option<String> {
        let base = self.config.base_url.as_deref()?;
        Some(format!(
            "{}/{}",
            base.trim_end_matches('/'),
            crate::web::url_path(path)
        ))
    }

    fn list(&self) -> Result<FileList, ConnectorError> {
        self.with_wire(|wire| {
            let mut list = FileList::default();
            let mut folders = vec![(self.full_path(&[]), String::new(), 0_usize)];
            while let Some((folder, prefix, depth)) = folders.pop() {
                for entry in wire.read_dir(&folder)? {
                    if entry.name.starts_with('.') {
                        continue;
                    }
                    let path = format!("{prefix}{}", entry.name);
                    match entry.attrs.kind {
                        FileKind::Folder => {
                            if depth + 1 < LIST_MAX_DEPTH {
                                folders.push((
                                    format!("{folder}/{}", entry.name),
                                    format!("{path}/"),
                                    depth + 1,
                                ));
                            } else {
                                list.truncated = true;
                            }
                        }
                        FileKind::File => {
                            if list.files.len() == LIST_MAX_FILES {
                                list.truncated = true;
                                list.files.sort_by(|a, b| a.path.cmp(&b.path));
                                return Ok(list);
                            }
                            list.files.push(listed(path, &entry.attrs));
                        }
                        FileKind::SymbolicLink | FileKind::Other => {}
                    }
                }
            }
            list.files.sort_by(|a, b| a.path.cmp(&b.path));
            Ok(list)
        })
    }

    fn stat(&self, path: &str) -> Result<Option<ListedFile>, ConnectorError> {
        let segments = segments_of(path)?;
        self.with_wire(|wire| {
            let target = Self::resolve(wire, self, &segments)?;
            Ok(target
                .attrs
                .filter(|attrs| attrs.kind == FileKind::File)
                .map(|attrs| listed(path.to_owned(), &attrs)))
        })
    }

    fn delete(&self, path: &str, expected: &Stamp) -> Result<(), ConnectorError> {
        let segments = segments_of(path)?;
        self.with_wire(|wire| {
            let target = Self::resolve(wire, self, &segments)?;
            let Some(attrs) = target.attrs.filter(|a| a.kind == FileKind::File) else {
                return Err(ConnectorError::new(
                    ConnectorErrorKind::NotFound,
                    format!("{path} does not exist"),
                ));
            };
            if &listed(path.to_owned(), &attrs).stamp != expected {
                return Err(ConnectorError::new(
                    ConnectorErrorKind::Conflict,
                    format!("{path} changed on the host"),
                ));
            }
            wire.remove(&target.full)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    use super::super::suite;
    use super::*;
    use crate::collection::{ConnectorId, ConnectorRow};
    use crate::testing::sshd::{KEY_PASSPHRASE, TestSshd};
    use crate::vault::OpenVault;

    fn config(server: &TestSshd, root: &Path, login: SftpLogin) -> SftpConfig {
        SftpConfig {
            host: "127.0.0.1".to_owned(),
            port: server.port(),
            user: server.user(),
            root: root.to_str().expect("a path").to_owned(),
            base_url: Some("https://example.com/feeds/".to_owned()),
            login,
            host_key: server.host_key_fingerprint(),
        }
    }

    fn key_credentials(path: &Path, passphrase: Option<&str>) -> SftpCredentials {
        SftpCredentials::Key {
            private_key: Zeroizing::new(fs::read_to_string(path).expect("a key")),
            passphrase: passphrase.map(|p| Zeroizing::new(p.to_owned())),
        }
    }

    /// A host with the plain key and a new root folder.
    fn host(server: &mut TestSshd) -> (SftpHost, std::path::PathBuf) {
        let root = server.root();
        let credentials = key_credentials(&server.key_path(), None);
        let host =
            SftpHost::new(config(server, &root, SftpLogin::Key), credentials).expect("a connector");
        (host, root)
    }

    fn names_in(root: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(root)
            .expect("a folder")
            .map(|e| {
                e.expect("an entry")
                    .file_name()
                    .into_string()
                    .expect("a name")
            })
            .collect();
        names.sort();
        names
    }

    fn no_temporary_file(root: &Path) {
        let found = names_in(root)
            .into_iter()
            .filter(|n| n.ends_with(".studio-tmp"))
            .count();
        assert_eq!(found, 0, "a temporary file stays in {}", root.display());
    }

    #[test]
    fn the_plain_key_logs_in() {
        let mut server = TestSshd::start();
        let (host, _) = host(&mut server);
        assert_eq!(host.read("absent.txt").expect("a read"), None);
    }

    #[test]
    fn the_encrypted_key_logs_in_with_its_passphrase() {
        let mut server = TestSshd::start();
        let root = server.root();
        let credentials = key_credentials(&server.encrypted_key_path(), Some(KEY_PASSPHRASE));
        let host = SftpHost::new(config(&server, &root, SftpLogin::Key), credentials)
            .expect("a connector");
        assert_eq!(host.read("absent.txt").expect("a read"), None);
    }

    #[test]
    fn the_encrypted_key_without_its_passphrase_is_refused() {
        let mut server = TestSshd::start();
        let root = server.root();
        let credentials = key_credentials(&server.encrypted_key_path(), None);
        let host = SftpHost::new(config(&server, &root, SftpLogin::Key), credentials)
            .expect("a connector");
        let error = host.read("a.txt").expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::Unauthorized);
    }

    #[test]
    fn a_wrong_key_is_refused() {
        let mut server = TestSshd::start();
        let root = server.root();
        let other = tempfile::tempdir().expect("a folder");
        let key = other.path().join("other_key");
        let status = Command::new("ssh-keygen")
            .args(["-t", "ed25519", "-N", "", "-q", "-f"])
            .arg(&key)
            .status()
            .expect("ssh-keygen");
        assert!(status.success());
        let host = SftpHost::new(
            config(&server, &root, SftpLogin::Key),
            key_credentials(&key, None),
        )
        .expect("a connector");
        let error = host.read("a.txt").expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::Unauthorized);
    }

    #[test]
    fn a_changed_host_key_is_refused_with_the_new_fingerprint_and_no_file_changes() {
        let mut server = TestSshd::start();
        let (host, root) = host(&mut server);
        fs::write(root.join("keep.txt"), b"keep").expect("a file");
        server.with_host_key_change();
        let new = server.host_key_fingerprint();
        assert_ne!(new, host.config.host_key);
        let error = host.read("keep.txt").expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::Unauthorized);
        assert!(error.to_string().contains(&new), "{error}");
        let error = host
            .write("keep.txt", b"new", &Expected::Absent)
            .expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::Unauthorized);
        assert_eq!(names_in(&root), ["keep.txt"]);
        assert_eq!(fs::read(root.join("keep.txt")).expect("a file"), b"keep");
    }

    #[test]
    fn check_root_accepts_a_folder_or_a_link_to_one_and_refuses_the_rest() {
        let mut server = TestSshd::start();
        let (host, root) = host(&mut server);
        host.check_root().expect("a folder");
        fs::write(root.join("file.txt"), b"x").expect("a file");
        fs::create_dir(root.join("folder")).expect("a folder");
        std::os::unix::fs::symlink(root.join("folder"), root.join("folder-link")).expect("a link");
        std::os::unix::fs::symlink(root.join("file.txt"), root.join("file-link")).expect("a link");
        let mut linked = config(&server, &root.join("folder-link"), SftpLogin::Key);
        linked.root = root
            .join("folder-link")
            .to_str()
            .expect("a path")
            .to_owned();
        SftpHost::new(linked, key_credentials(&server.key_path(), None))
            .expect("a connector")
            .check_root()
            .expect("a link to a folder");
        for bad in [
            root.join("missing"),
            root.join("file.txt"),
            root.join("file-link"),
        ] {
            let mut config = config(&server, &bad, SftpLogin::Key);
            config.root = bad.to_str().expect("a path").to_owned();
            let other = SftpHost::new(config, key_credentials(&server.key_path(), None))
                .expect("a connector");
            let error = other.check_root().expect_err("an error");
            assert_eq!(error.kind(), ConnectorErrorKind::NotFound, "{bad:?}");
        }
    }

    #[test]
    fn fingerprint_of_gives_the_host_key_fingerprint() {
        let server = TestSshd::start();
        let seen = SftpHost::fingerprint_of("127.0.0.1", server.port()).expect("a fingerprint");
        assert_eq!(seen, server.host_key_fingerprint());
    }

    #[test]
    fn fingerprint_of_a_closed_port_gives_unavailable() {
        let port = {
            let server = TestSshd::start();
            server.port()
        };
        let error = SftpHost::fingerprint_of("127.0.0.1", port).expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::Unavailable);
    }

    #[test]
    fn the_debug_text_shows_no_secret() {
        let server = TestSshd::start();
        let root = Path::new("/srv");
        let credentials = SftpCredentials::Key {
            private_key: Zeroizing::new("PRIVATE-KEY-TEXT".to_owned()),
            passphrase: Some(Zeroizing::new("PASSPHRASE-TEXT".to_owned())),
        };
        let shown = format!("{credentials:?}");
        let host =
            SftpHost::new(config(&server, root, SftpLogin::Key), credentials).expect("a connector");
        let password = SftpCredentials::Password(Zeroizing::new("PASSWORD-TEXT".to_owned()));
        let all = format!("{shown} {host:?} {password:?}");
        for secret in ["PRIVATE-KEY-TEXT", "PASSPHRASE-TEXT", "PASSWORD-TEXT"] {
            assert!(!all.contains(secret), "{secret} is in {all}");
        }
    }

    #[test]
    fn a_secret_error_message_shows_no_secret() {
        let error =
            SftpCredentials::from_secret(SftpLogin::Password, Some(b"{\"pass\":\"TOP-SECRET\"}"))
                .expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::InvalidConfig);
        assert!(!error.to_string().contains("TOP-SECRET"));
    }

    fn row(config: String, secret_ref: Option<&str>) -> ConnectorRow {
        ConnectorRow {
            id: ConnectorId(1),
            kind: SftpHost::KIND.to_owned(),
            name: "Host".to_owned(),
            config,
            secret_ref: secret_ref.map(str::to_owned),
        }
    }

    #[test]
    fn from_row_needs_a_secret_for_a_key_or_a_password() {
        let server = TestSshd::start();
        for login in [SftpLogin::Key, SftpLogin::Password] {
            let config =
                serde_json::to_string(&config(&server, Path::new("/srv"), login)).expect("JSON");
            let error = super::super::from_row(&row(config, None), None)
                .err()
                .expect("an error");
            assert_eq!(error.kind(), ConnectorErrorKind::InvalidConfig, "{login:?}");
        }
    }

    #[test]
    fn from_row_makes_a_working_connector_from_the_vault() {
        let mut server = TestSshd::start();
        let root = server.root();
        let key = fs::read_to_string(server.key_path()).expect("a key");
        let secret = serde_json::json!({ "private_key": key }).to_string();
        let mut vault = OpenVault::create("correct horse battery staple 2026").expect("a vault");
        vault.put("host-1", secret.as_bytes()).expect("a secret");
        let config = serde_json::to_string(&config(&server, &root, SftpLogin::Key)).expect("JSON");
        let connector = super::super::from_row(&row(config, Some("host-1")), Some(&vault))
            .expect("a connector");
        connector
            .write("a.txt", b"one", &Expected::Absent)
            .expect("a write");
        assert_eq!(fs::read(root.join("a.txt")).expect("a file"), b"one");
    }

    #[test]
    fn the_agent_login_needs_no_secret() {
        let server = TestSshd::start();
        let config = serde_json::to_string(&config(&server, Path::new("/srv"), SftpLogin::Agent))
            .expect("JSON");
        assert!(super::super::from_row(&row(config, None), None).is_ok());
    }

    #[test]
    fn a_configuration_that_is_not_valid_is_refused() {
        let server = TestSshd::start();
        let credentials = || key_credentials(&server.key_path(), None);
        let base = config(&server, Path::new("/srv"), SftpLogin::Key);
        let cases = [
            SftpConfig {
                host: String::new(),
                ..base.clone()
            },
            SftpConfig {
                user: String::new(),
                ..base.clone()
            },
            SftpConfig {
                root: String::new(),
                ..base.clone()
            },
            SftpConfig {
                port: 0,
                ..base.clone()
            },
            SftpConfig {
                host_key: "MD5:abc".to_owned(),
                ..base.clone()
            },
            SftpConfig {
                login: SftpLogin::Password,
                ..base.clone()
            },
        ];
        for case in cases {
            let error = SftpHost::new(case, credentials()).expect_err("an error");
            assert_eq!(error.kind(), ConnectorErrorKind::InvalidConfig);
        }
    }

    #[test]
    fn the_config_text_makes_the_same_connector() {
        let mut server = TestSshd::start();
        let (host, _) = host(&mut server);
        let secret = serde_json::json!({
            "private_key": fs::read_to_string(server.key_path()).expect("a key")
        })
        .to_string();
        let again = SftpHost::from_config(&host.config(), Some(secret.as_bytes())).expect("a host");
        assert_eq!(again.config, host.config);
    }

    #[test]
    fn a_write_leaves_no_temporary_file_after_a_success_or_a_conflict() {
        let mut server = TestSshd::start();
        let (host, root) = host(&mut server);
        let v1 = host
            .write("a.txt", b"one", &Expected::Absent)
            .expect("a write");
        no_temporary_file(&root);
        let error = host
            .write("a.txt", b"two", &Expected::Absent)
            .expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::Conflict);
        no_temporary_file(&root);
        let error = host
            .write(
                "a.txt",
                b"two",
                &Expected::Version(Version::new("sha256:0")),
            )
            .expect_err("an error");
        assert_eq!(error.kind(), ConnectorErrorKind::Conflict);
        no_temporary_file(&root);
        host.write("a.txt", b"two", &Expected::Version(v1))
            .expect("a write");
        no_temporary_file(&root);
        assert_eq!(names_in(&root), ["a.txt"]);
        assert_eq!(fs::read(root.join("a.txt")).expect("a file"), b"two");
    }

    #[test]
    fn a_write_from_with_a_wrong_length_leaves_no_file() {
        let mut server = TestSshd::start();
        let (host, root) = host(&mut server);
        for (data, length) in [(&b"abc"[..], 10_u64), (&b"abcdef"[..], 3)] {
            let error = host
                .write_from("a/b.bin", &mut { data }, length, &Expected::Absent)
                .expect_err("an error");
            assert_eq!(error.kind(), ConnectorErrorKind::Io);
            assert!(!root.join("a/b.bin").exists());
            no_temporary_file(&root.join("a"));
        }
    }

    #[test]
    fn a_large_write_from_gives_the_version_of_its_bytes() {
        let mut server = TestSshd::start();
        let (host, root) = host(&mut server);
        let bytes: Vec<u8> = (0..200_000_u32).map(|n| (n % 251) as u8).collect();
        let version = host
            .write_from(
                "big/b.bin",
                &mut bytes.as_slice(),
                bytes.len() as u64,
                &Expected::Absent,
            )
            .expect("a write");
        assert_eq!(version, content_version(&bytes));
        assert_eq!(fs::read(root.join("big/b.bin")).expect("a file"), bytes);
        let read = host.read("big/b.bin").expect("a read").expect("a file");
        assert_eq!(read.version, version);
        no_temporary_file(&root.join("big"));
    }

    #[test]
    fn a_path_through_a_symbolic_link_gives_invalid_path() {
        let mut server = TestSshd::start();
        let (host, root) = host(&mut server);
        let outside = server.root();
        fs::write(outside.join("file.txt"), b"outside").expect("a file");
        std::os::unix::fs::symlink(&outside, root.join("link")).expect("a link");
        std::os::unix::fs::symlink(outside.join("file.txt"), root.join("file-link"))
            .expect("a link");
        for path in ["link/new.txt", "link/file.txt", "file-link"] {
            let error = host
                .write(path, b"x", &Expected::Absent)
                .expect_err("an error");
            assert_eq!(error.kind(), ConnectorErrorKind::InvalidPath, "{path}");
            let error = host.read(path).expect_err("an error");
            assert_eq!(error.kind(), ConnectorErrorKind::InvalidPath, "{path}");
            let error = host.delete(path, &Stamp::new("0-0")).expect_err("an error");
            assert_eq!(error.kind(), ConnectorErrorKind::InvalidPath, "{path}");
        }
        assert_eq!(names_in(&outside), ["file.txt"]);
        assert_eq!(
            fs::read(outside.join("file.txt")).expect("a file"),
            b"outside"
        );
    }

    #[test]
    fn the_list_leaves_out_links_and_hidden_names_and_has_a_depth_limit() {
        let mut server = TestSshd::start();
        let (host, root) = host(&mut server);
        for path in [
            "b.xml",
            "album/feed.xml",
            ".git/config",
            ".hidden.xml",
            "a/b/c/d/deep.xml",
            "a/b/c/ok.xml",
        ] {
            host.write(path, b"x", &Expected::Absent).expect("a write");
        }
        std::os::unix::fs::symlink(root.join("b.xml"), root.join("link.xml")).expect("a link");
        std::os::unix::fs::symlink(root.join("album"), root.join("linked")).expect("a link");
        let list = host.list().expect("a list");
        let paths: Vec<&str> = list.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, ["a/b/c/ok.xml", "album/feed.xml", "b.xml"]);
        assert!(list.files.iter().all(|f| f.size == 1));
        assert!(list.truncated);
    }

    #[test]
    fn the_public_url_is_the_base_address_and_the_path() {
        let server = TestSshd::start();
        let host = SftpHost::new(
            config(&server, Path::new("/srv"), SftpLogin::Key),
            key_credentials(&server.key_path(), None),
        )
        .expect("a connector");
        assert_eq!(
            host.public_url("a/b/01 Song (Live).mp3").as_deref(),
            Some("https://example.com/feeds/a/b/01%20Song%20%28Live%29.mp3")
        );
        assert!(!host.can_redirect());
    }

    /// Do a check of the shared suite. Each connector of the check has a
    /// new folder below the root of one server.
    fn suite_check(check: fn(&dyn Fn() -> Box<dyn Connector>)) {
        let server = RefCell::new(TestSshd::start());
        check(&|| {
            let root = server.borrow_mut().root();
            let server = server.borrow();
            let credentials = key_credentials(&server.key_path(), None);
            let host = SftpHost::new(config(&server, &root, SftpLogin::Key), credentials)
                .expect("a connector");
            Box::new(host)
        });
    }

    #[test]
    fn the_suite_runs_all_checks() {
        suite_check(suite::run_all);
    }

    #[test]
    fn suite_read_of_an_absent_file_gives_none() {
        suite_check(suite::read_of_an_absent_file_gives_none);
    }

    #[test]
    fn suite_write_then_read_gives_the_bytes_and_the_version() {
        suite_check(suite::write_then_read_gives_the_bytes_and_the_version);
    }

    #[test]
    fn suite_write_over_a_file_with_absent_is_a_conflict() {
        suite_check(suite::write_over_a_file_with_absent_is_a_conflict);
    }

    #[test]
    fn suite_write_with_an_old_version_is_a_conflict() {
        suite_check(suite::write_with_an_old_version_is_a_conflict);
    }

    #[test]
    fn suite_write_with_the_current_version_replaces_the_file() {
        suite_check(suite::write_with_the_current_version_replaces_the_file);
    }

    #[test]
    fn suite_write_from_gives_the_same_file_as_write() {
        suite_check(suite::write_from_gives_the_same_file_as_write);
    }

    #[test]
    fn suite_write_from_with_a_short_reader_writes_nothing() {
        suite_check(suite::write_from_with_a_short_reader_writes_nothing);
    }

    #[test]
    fn suite_stat_gives_the_size_and_a_new_stamp_after_a_change() {
        suite_check(suite::stat_gives_the_size_and_a_new_stamp_after_a_change);
    }

    #[test]
    fn suite_list_gives_each_file_in_path_sequence() {
        suite_check(suite::list_gives_each_file_in_path_sequence);
    }

    #[test]
    fn suite_delete_with_the_current_stamp_removes_the_file() {
        suite_check(suite::delete_with_the_current_stamp_removes_the_file);
    }

    #[test]
    fn suite_delete_with_an_old_stamp_is_a_conflict() {
        suite_check(suite::delete_with_an_old_stamp_is_a_conflict);
    }

    #[test]
    fn suite_delete_of_an_absent_file_is_not_found() {
        suite_check(suite::delete_of_an_absent_file_is_not_found);
    }

    #[test]
    fn suite_a_path_out_of_the_host_is_invalid() {
        suite_check(suite::a_path_out_of_the_host_is_invalid);
    }

    #[test]
    fn suite_sha256_when_given_is_the_sha256_of_the_bytes() {
        suite_check(suite::sha256_when_given_is_the_sha256_of_the_bytes);
    }

    #[test]
    fn suite_set_redirect_without_the_ability_is_unsupported() {
        suite_check(suite::set_redirect_without_the_ability_is_unsupported);
    }
}

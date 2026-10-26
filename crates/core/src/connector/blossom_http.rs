//! The HTTP calls of the Blossom connector (ADR 0048, BUD-01, BUD-02 and
//! BUD-04).
//!
//! No error message from this module holds an `Authorization` header.

use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{ConnectorError, ConnectorErrorKind};
use crate::digest::{finish_hex, sha256_hex};

/// The size of each block that `Spooled::from_reader` reads.
const BLOCK: usize = 64 * 1024;
/// The most bytes that `get` accepts for one blob.
const GET_LIMIT: u64 = 1024 * 1024 * 1024;
/// The most bytes that Studio reads from a descriptor.
const DESCRIPTOR_LIMIT: u64 = 64 * 1024;

/// Where the bytes of a [`Spooled`] are.
#[derive(Debug)]
enum Store<'a> {
    Memory(&'a [u8]),
    File(PathBuf),
}

/// The bytes of one write, with their SHA-256 and size.
///
/// A value from `from_reader` keeps its bytes in a file of the temporary
/// folder. The file goes when the value drops.
#[derive(Debug)]
pub(crate) struct Spooled<'a> {
    sha256: String,
    size: u64,
    store: Store<'a>,
}

impl<'a> Spooled<'a> {
    /// Keep a slice in memory.
    pub(crate) fn from_bytes(bytes: &'a [u8]) -> Self {
        Self {
            sha256: sha256_hex(&[bytes]),
            size: bytes.len() as u64,
            store: Store::Memory(bytes),
        }
    }

    /// Write `length` bytes of `reader` to a file and calculate their
    /// SHA-256 at the same time.
    ///
    /// # Errors
    ///
    /// Gives `Io` when the reader gives fewer or more than `length` bytes,
    /// or when the file fails. The file then goes.
    pub(crate) fn from_reader(
        reader: &mut dyn Read,
        length: u64,
    ) -> Result<Spooled<'static>, ConnectorError> {
        let io = |e: &std::io::Error| {
            ConnectorError::new(
                ConnectorErrorKind::Io,
                format!("cannot spool the data: {e}"),
            )
        };
        let path = std::env::temp_dir().join(format!("studio-spool-{}", uuid::Uuid::new_v4()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        let mut file = options.open(&path).map_err(|e| io(&e))?;
        // The value owns the file from here, so an error removes it.
        let mut spooled = Spooled {
            sha256: String::new(),
            size: 0,
            store: Store::File(path),
        };
        let mut limited = reader.take(length.saturating_add(1));
        let mut hash = Sha256::new();
        let mut block = vec![0_u8; BLOCK];
        let mut total = 0_u64;
        loop {
            let read = limited.read(&mut block).map_err(|e| io(&e))?;
            if read == 0 {
                break;
            }
            hash.update(&block[..read]);
            std::io::Write::write_all(&mut file, &block[..read]).map_err(|e| io(&e))?;
            total += read as u64;
        }
        if total != length {
            return Err(ConnectorError::new(
                ConnectorErrorKind::Io,
                format!("the data does not have the length {length}"),
            ));
        }
        spooled.sha256 = finish_hex(hash);
        spooled.size = total;
        Ok(spooled)
    }

    /// The SHA-256 of the bytes, as lowercase hexadecimal.
    pub(crate) fn sha256(&self) -> &str {
        &self.sha256
    }

    /// The size of the bytes.
    pub(crate) fn size(&self) -> u64 {
        self.size
    }

    #[cfg(test)]
    fn file_path(&self) -> Option<&std::path::Path> {
        match &self.store {
            Store::File(path) => Some(path),
            Store::Memory(_) => None,
        }
    }
}

impl Drop for Spooled<'_> {
    fn drop(&mut self) {
        if let Store::File(path) = &self.store {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// The part of a blob descriptor that Studio uses (BUD-02).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Descriptor {
    /// The SHA-256 of the blob.
    pub(crate) sha256: String,
    /// The size of the blob, when the server gives it.
    #[serde(default)]
    pub(crate) size: Option<u64>,
}

type Response = ureq::http::Response<ureq::Body>;

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(20)))
        .timeout_recv_response(Some(Duration::from_secs(300)))
        .http_status_as_error(false)
        .max_redirects(0)
        .user_agent(concat!("musicindex-studio/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

fn unavailable() -> ConnectorError {
    ConnectorError::new(
        ConnectorErrorKind::Unavailable,
        "cannot reach the Blossom server",
    )
}

/// The error for a status that is not a success.
fn status_error(status: u16) -> ConnectorError {
    match status {
        401 | 403 => ConnectorError::new(
            ConnectorErrorKind::Unauthorized,
            "the Blossom server refused the authorization",
        ),
        413 => ConnectorError::new(
            ConnectorErrorKind::Io,
            "the server refused the size of the file",
        ),
        other => ConnectorError::new(
            ConnectorErrorKind::Io,
            format!("the Blossom server gave {other}"),
        ),
    }
}

fn send<B: ureq::AsSendBody>(
    method: &str,
    url: &str,
    headers: &[(&str, &str)],
    body: B,
) -> Result<Response, ConnectorError> {
    let mut builder = ureq::http::Request::builder().method(method).uri(url);
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    let request = builder.body(body).map_err(|_| unavailable())?;
    agent().run(request).map_err(|_| unavailable())
}

fn descriptor_of(
    mut response: Response,
    expected_sha256: &str,
) -> Result<Descriptor, ConnectorError> {
    let status = response.status().as_u16();
    if !(200..300).contains(&status) {
        return Err(status_error(status));
    }
    let bad = || {
        ConnectorError::new(
            ConnectorErrorKind::Io,
            "the Blossom server gave a descriptor that is not valid",
        )
    };
    let text = response
        .body_mut()
        .with_config()
        .limit(DESCRIPTOR_LIMIT)
        .read_to_string()
        .map_err(|_| bad())?;
    let descriptor: Descriptor = serde_json::from_str(&text).map_err(|_| bad())?;
    if !descriptor.sha256.eq_ignore_ascii_case(expected_sha256) {
        return Err(ConnectorError::new(
            ConnectorErrorKind::Io,
            "the Blossom server stored a blob with a different SHA-256",
        ));
    }
    Ok(descriptor)
}

/// `PUT <server>/upload`. The descriptor must have the SHA-256 of the
/// bytes.
pub(crate) fn upload(
    server: &str,
    spooled: &Spooled<'_>,
    media_type: &str,
    auth_header: &str,
) -> Result<Descriptor, ConnectorError> {
    let url = format!("{server}/upload");
    let length = spooled.size().to_string();
    let headers = [
        ("authorization", auth_header),
        ("content-type", media_type),
        ("content-length", length.as_str()),
    ];
    let response = match &spooled.store {
        Store::Memory(bytes) => send("PUT", &url, &headers, *bytes)?,
        Store::File(path) => {
            let file = File::open(path).map_err(|e| {
                ConnectorError::new(
                    ConnectorErrorKind::Io,
                    format!("cannot read the spooled data: {e}"),
                )
            })?;
            send("PUT", &url, &headers, file)?
        }
    };
    descriptor_of(response, spooled.sha256())
}

/// `PUT <server>/mirror`. The server fetches the blob from `source_url`.
pub(crate) fn mirror(
    server: &str,
    source_url: &str,
    sha256: &str,
    auth_header: &str,
) -> Result<Descriptor, ConnectorError> {
    let body = serde_json::json!({ "url": source_url }).to_string();
    let response = send(
        "PUT",
        &format!("{server}/mirror"),
        &[
            ("authorization", auth_header),
            ("content-type", "application/json"),
        ],
        body,
    )?;
    descriptor_of(response, sha256)
}

/// `HEAD <server>/<sha256>`. Gives the size, or `None` for a missing blob.
pub(crate) fn head(server: &str, sha256: &str) -> Result<Option<u64>, ConnectorError> {
    let response = send("HEAD", &format!("{server}/{sha256}"), &[], ())?;
    match response.status().as_u16() {
        404 => Ok(None),
        200 => Ok(response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())),
        other => Err(status_error(other)),
    }
}

/// `GET <server>/<sha256>`. Gives the bytes, or `None` for a missing blob.
pub(crate) fn get(server: &str, sha256: &str) -> Result<Option<Vec<u8>>, ConnectorError> {
    let mut response = send("GET", &format!("{server}/{sha256}"), &[], ())?;
    match response.status().as_u16() {
        404 => return Ok(None),
        200 => {}
        other => return Err(status_error(other)),
    }
    let bytes = response
        .body_mut()
        .with_config()
        .limit(GET_LIMIT)
        .read_to_vec()
        .map_err(|_| unavailable())?;
    if !sha256_hex(&[&bytes]).eq_ignore_ascii_case(sha256) {
        return Err(ConnectorError::new(
            ConnectorErrorKind::Io,
            "the Blossom server gave bytes with a different SHA-256",
        ));
    }
    Ok(Some(bytes))
}

/// `DELETE <server>/<sha256>`. Gives `false` for a missing blob.
pub(crate) fn delete(
    server: &str,
    sha256: &str,
    auth_header: &str,
) -> Result<bool, ConnectorError> {
    let response = send(
        "DELETE",
        &format!("{server}/{sha256}"),
        &[("authorization", auth_header)],
        (),
    )?;
    match response.status().as_u16() {
        404 => Ok(false),
        200..=299 => Ok(true),
        other => Err(status_error(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nostr_events::{
        AuthAction, auth_event, auth_header, parse_secret_key, server_domain, sign_with_keys,
    };
    use crate::testing::blossom::TestBlossom;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn token(server: &str, action: AuthAction, sha256: &str) -> String {
        let keys = parse_secret_key(&"01".repeat(32)).expect("keys");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("a time")
            .as_secs();
        let unsigned = auth_event(
            keys.public_key(),
            action,
            &[sha256.to_owned()],
            &server_domain(server).expect("a domain"),
            now,
        );
        auth_header(&sign_with_keys(unsigned, &keys).expect("a signed event"))
    }

    #[test]
    fn upload_head_get_and_delete_work() {
        let server = TestBlossom::start();
        let url = server.url();
        let sha = sha256_hex(&[b"abc"]);
        let spooled = Spooled::from_bytes(b"abc");
        let descriptor = upload(
            &url,
            &spooled,
            "audio/mpeg",
            &token(&url, AuthAction::Upload, &sha),
        )
        .expect("an upload");
        assert_eq!(descriptor.sha256, sha);
        assert_eq!(head(&url, &sha).expect("a head"), Some(3));
        assert_eq!(get(&url, &sha).expect("a get"), Some(b"abc".to_vec()));
        assert!(delete(&url, &sha, &token(&url, AuthAction::Delete, &sha)).expect("a delete"));
        assert_eq!(head(&url, &sha).expect("a head"), None);
        assert_eq!(get(&url, &sha).expect("a get"), None);
        assert!(!delete(&url, &sha, &token(&url, AuthAction::Delete, &sha)).expect("a delete"));
    }

    #[test]
    fn an_upload_from_a_file_gives_the_same_blob() {
        let server = TestBlossom::start();
        let url = server.url();
        let bytes: Vec<u8> = (0..200_000_u32).map(|n| (n % 241) as u8).collect();
        let spooled = Spooled::from_reader(&mut bytes.as_slice(), bytes.len() as u64)
            .expect("a spooled file");
        assert_eq!(spooled.sha256(), sha256_hex(&[&bytes]));
        upload(
            &url,
            &spooled,
            "application/octet-stream",
            &token(&url, AuthAction::Upload, spooled.sha256()),
        )
        .expect("an upload");
        assert_eq!(
            get(&url, spooled.sha256()).expect("a get"),
            Some(bytes.clone())
        );
    }

    #[test]
    fn a_mirror_copies_a_blob_between_servers() {
        let first = TestBlossom::start();
        let second = TestBlossom::start();
        let sha = sha256_hex(&[b"abc"]);
        upload(
            &first.url(),
            &Spooled::from_bytes(b"abc"),
            "audio/mpeg",
            &token(&first.url(), AuthAction::Upload, &sha),
        )
        .expect("an upload");
        let descriptor = mirror(
            &second.url(),
            &format!("{}/{sha}", first.url()),
            &sha,
            &token(&second.url(), AuthAction::Upload, &sha),
        )
        .expect("a mirror");
        assert_eq!(descriptor.sha256, sha);
        assert_eq!(second.blobs(), vec![sha]);
    }

    #[test]
    fn a_refusal_gives_unauthorized() {
        let server = TestBlossom::start();
        let url = server.url();
        let sha = sha256_hex(&[b"abc"]);
        server.refuse_uploads(true);
        let error = upload(
            &url,
            &Spooled::from_bytes(b"abc"),
            "audio/mpeg",
            &token(&url, AuthAction::Upload, &sha),
        )
        .expect_err("a refusal");
        assert_eq!(error.kind(), ConnectorErrorKind::Unauthorized);
        server.refuse_uploads(false);
        let wrong = token(&url, AuthAction::Upload, &sha256_hex(&[b"other"]));
        let error = upload(&url, &Spooled::from_bytes(b"abc"), "audio/mpeg", &wrong)
            .expect_err("a wrong token");
        assert_eq!(error.kind(), ConnectorErrorKind::Unauthorized);
        assert!(!error.to_string().contains("Nostr "));
    }

    #[test]
    fn a_closed_port_gives_unavailable() {
        let url = {
            let server = TestBlossom::start();
            server.url()
        };
        let error = head(&url, &sha256_hex(&[b"abc"])).expect_err("no server");
        assert_eq!(error.kind(), ConnectorErrorKind::Unavailable);
    }

    #[test]
    fn the_status_codes_map_to_error_kinds() {
        assert_eq!(status_error(401).kind(), ConnectorErrorKind::Unauthorized);
        assert_eq!(status_error(403).kind(), ConnectorErrorKind::Unauthorized);
        let too_big = status_error(413);
        assert_eq!(too_big.kind(), ConnectorErrorKind::Io);
        assert_eq!(
            too_big.to_string(),
            "the server refused the size of the file"
        );
        assert!(status_error(500).to_string().contains("500"));
    }

    #[test]
    fn a_spooled_file_goes_when_the_value_drops() {
        let spooled = Spooled::from_reader(&mut &b"abc"[..], 3).expect("a spooled file");
        let path = spooled.file_path().expect("a file").to_path_buf();
        assert!(path.exists());
        assert_eq!(spooled.size(), 3);
        drop(spooled);
        assert!(!path.exists());
    }

    #[test]
    fn a_reader_of_a_different_length_gives_io() {
        for (data, length) in [(&b"abc"[..], 10_u64), (&b"abcdef"[..], 3)] {
            let error = Spooled::from_reader(&mut &data[..], length).expect_err("a bad length");
            assert_eq!(error.kind(), ConnectorErrorKind::Io);
        }
    }
}

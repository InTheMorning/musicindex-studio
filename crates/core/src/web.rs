//! Read public files over HTTP and HTTPS (ADR 0020).
//!
//! The desktop shell reads the addresses that its own user gives. The
//! server shell uses the policy `Public`, so that a client refuses each
//! address in a private network, also after a redirect (ADR 0057).

use std::fmt;
use std::fmt::Write as _;
use std::io::{Read, Write as _};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use ureq::ResponseExt;

use crate::net::{self, NetPolicy};

/// The kind of a [`WebError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WebErrorKind {
    /// The address is not an `http` or `https` address.
    InvalidAddress,
    /// The server answered with a status that is not 200.
    Status,
    /// The file is larger than the limit.
    TooLarge,
    /// The connection or the transfer failed.
    Network,
    /// The response has a `Content-Type` that is not an image type
    /// (ADR 0025).
    NotImage,
    /// The caller asked to stop the transfer. The partial file stays.
    Stopped,
    /// The policy refused the address of the host or of a redirect
    /// (ADR 0057).
    NotPublic,
}

/// An error from [`Web`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebError {
    kind: WebErrorKind,
    status: Option<u16>,
    message: String,
}

impl WebError {
    fn new(kind: WebErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            status: None,
            message: message.into(),
        }
    }

    /// The kind of the error.
    #[must_use]
    pub fn kind(&self) -> WebErrorKind {
        self.kind
    }

    /// The HTTP status, for [`WebErrorKind::Status`].
    #[must_use]
    pub fn status(&self) -> Option<u16> {
        self.status
    }
}

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for WebError {}

/// A file that [`Web::get`] read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebFile {
    /// The address after each redirect.
    pub final_url: String,
    /// The bytes of the file.
    pub bytes: Vec<u8>,
}

/// An HTTP client with a time limit and a user agent.
#[derive(Debug, Clone)]
pub struct Web {
    agent: ureq::Agent,
    /// The client for [`Web::get_to_file`]. It has no time limit for the
    /// whole transfer, because a large file takes minutes.
    transfer: ureq::Agent,
}

impl Default for Web {
    fn default() -> Self {
        Self::new()
    }
}

impl Web {
    /// Make a client with the policy `Open`. Each request has a time limit
    /// of 20 seconds and a maximum of 5 redirects.
    #[must_use]
    pub fn new() -> Self {
        Self::with_policy(&NetPolicy::Open)
    }

    /// Make a client that follows `policy`. With `Public`, the client
    /// refuses each address that is not public, also the address of a
    /// redirect, and it uses no proxy from the environment (ADR 0057).
    #[must_use]
    pub fn with_policy(policy: &NetPolicy) -> Self {
        let config = ureq::Agent::config_builder()
            .proxy(net::proxy(policy))
            .timeout_global(Some(Duration::from_secs(20)))
            .http_status_as_error(false)
            .max_redirects(5)
            .user_agent(concat!("musicindex-studio/", env!("CARGO_PKG_VERSION")))
            .build();
        let transfer = ureq::Agent::config_builder()
            .proxy(net::proxy(policy))
            .timeout_connect(Some(Duration::from_secs(20)))
            .timeout_recv_response(Some(Duration::from_secs(20)))
            .http_status_as_error(false)
            .max_redirects(5)
            .user_agent(concat!("musicindex-studio/", env!("CARGO_PKG_VERSION")))
            .build();
        Self {
            agent: net::agent(config, policy),
            transfer: net::agent(transfer, policy),
        }
    }

    /// Read a file of `limit` bytes or less.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] for an address that is not `http` or `https`, a
    /// status that is not 200, a file that is too large, or a network
    /// failure.
    pub fn get(&self, url: &str, limit: u64) -> Result<WebFile, WebError> {
        self.get_with_query(url, &[], limit)
    }

    /// Read a file, with query parameters that the client encodes.
    ///
    /// # Errors
    ///
    /// The same as [`Web::get`].
    pub fn get_with_query(
        &self,
        url: &str,
        query: &[(&str, &str)],
        limit: u64,
    ) -> Result<WebFile, WebError> {
        let lower = url.trim().to_ascii_lowercase();
        if !(lower.starts_with("http://") || lower.starts_with("https://")) {
            return Err(WebError::new(
                WebErrorKind::InvalidAddress,
                format!("{url} is not an http or https address"),
            ));
        }
        let mut request = self.agent.get(url.trim());
        for (key, value) in query {
            request = request.query(*key, *value);
        }
        let mut response = request.call().map_err(|e| network_error(url, &e))?;
        let status = response.status().as_u16();
        if status != 200 {
            return Err(WebError {
                kind: WebErrorKind::Status,
                status: Some(status),
                message: format!("{url} answered with status {status}"),
            });
        }
        let final_url = response.get_uri().to_string();
        let bytes = response
            .body_mut()
            .with_config()
            .limit(limit)
            .read_to_vec()
            .map_err(|e| match e {
                ureq::Error::BodyExceedsLimit(_) => WebError::new(
                    WebErrorKind::TooLarge,
                    format!("{url} is larger than {limit} bytes"),
                ),
                other => WebError::new(WebErrorKind::Network, format!("{url}: {other}")),
            })?;
        Ok(WebFile { final_url, bytes })
    }
}

/// The limit on the size of an image that [`Web::get_image`] reads
/// (ADR 0025, ADR 0032 item 13).
pub const MAX_IMAGE_BYTES: u64 = 10 * 1024 * 1024;

/// An image that [`Web::get_image`] read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebImage {
    /// The bytes of the image.
    pub bytes: Vec<u8>,
    /// The media type, such as `image/png`, with no parameter.
    pub media_type: String,
}

impl Web {
    /// Read an image of [`MAX_IMAGE_BYTES`] or less (ADR 0025).
    ///
    /// The function follows each redirect of the agent, and reads the
    /// response only when its `Content-Type` is an image type.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] for an address that is not `http` or `https`, a
    /// status that is not 200, a response with no image media type, a file
    /// that is too large, or a network failure.
    pub fn get_image(&self, url: &str) -> Result<WebImage, WebError> {
        let lower = url.trim().to_ascii_lowercase();
        if !(lower.starts_with("http://") || lower.starts_with("https://")) {
            return Err(WebError::new(
                WebErrorKind::InvalidAddress,
                format!("{url} is not an http or https address"),
            ));
        }
        let mut response = self
            .agent
            .get(url.trim())
            .call()
            .map_err(|e| network_error(url, &e))?;
        let status = response.status().as_u16();
        if status != 200 {
            return Err(WebError {
                kind: WebErrorKind::Status,
                status: Some(status),
                message: format!("{url} answered with status {status}"),
            });
        }
        let media_type = header(&response, "content-type")
            .as_deref()
            .and_then(media_type)
            .filter(|m| m.starts_with("image/"))
            .ok_or_else(|| {
                WebError::new(
                    WebErrorKind::NotImage,
                    format!("{url} did not answer with an image media type"),
                )
            })?;
        let bytes = response
            .body_mut()
            .with_config()
            .limit(MAX_IMAGE_BYTES)
            .read_to_vec()
            .map_err(|e| match e {
                ureq::Error::BodyExceedsLimit(_) => WebError::new(
                    WebErrorKind::TooLarge,
                    format!("{url} is larger than {MAX_IMAGE_BYTES} bytes"),
                ),
                other => WebError::new(WebErrorKind::Network, format!("{url}: {other}")),
            })?;
        Ok(WebImage { bytes, media_type })
    }
}

/// The size and the media type of a file, as its host gives them
/// (ADR 0023 item 5).
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize)]
pub struct MediaFacts {
    /// The size in bytes, or `None` when the host does not give it.
    pub length: Option<u64>,
    /// The media type without parameters, such as `audio/mpeg`, or `None`.
    pub media_type: Option<String>,
}

impl crate::pipeline::AddressCheck for Web {
    fn check(&self, url: &str) -> Result<(), String> {
        self.media_facts(url)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

impl Web {
    /// Ask the host of a file for its size and its media type, and do not
    /// read the file.
    ///
    /// The function sends a `HEAD` request. When the response has no
    /// length, it sends a `GET` with `Range: bytes=0-0` and reads the total
    /// size from `Content-Range` (ADR 0023 item 5).
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] for an address that is not `http` or `https`, a
    /// status that is not a success, or a network failure.
    pub fn media_facts(&self, url: &str) -> Result<MediaFacts, WebError> {
        let lower = url.trim().to_ascii_lowercase();
        if !(lower.starts_with("http://") || lower.starts_with("https://")) {
            return Err(WebError::new(
                WebErrorKind::InvalidAddress,
                format!("{url} is not an http or https address"),
            ));
        }
        let response = self
            .agent
            .head(url.trim())
            .call()
            .map_err(|e| network_error(url, &e))?;
        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(WebError {
                kind: WebErrorKind::Status,
                status: Some(status),
                message: format!("{url} answered with status {status}"),
            });
        }
        let mut facts = MediaFacts {
            length: header(&response, "content-length").and_then(|v| v.parse().ok()),
            media_type: header(&response, "content-type")
                .as_deref()
                .and_then(media_type),
        };
        if facts.length.is_none() {
            let ranged = self
                .agent
                .get(url.trim())
                .header("Range", "bytes=0-0")
                .call()
                .map_err(|e| network_error(url, &e))?;
            facts.length = header(&ranged, "content-range").and_then(|v| {
                v.rsplit_once('/')
                    .and_then(|(_, total)| total.trim().parse().ok())
            });
            if facts.media_type.is_none() {
                facts.media_type = header(&ranged, "content-type")
                    .as_deref()
                    .and_then(media_type);
            }
        }
        Ok(facts)
    }
}

/// The size of a block that [`Web::get_to_file`] reads.
const BLOCK_BYTES: usize = 64 * 1024;

impl Web {
    /// Read a file into `part`, in blocks of 64 KiB (ADR 0038 item 8).
    ///
    /// When `part` has bytes, the request has `Range: bytes={len}-`. A 206
    /// response adds to the file. A 200 response starts the file again
    /// from 0. After each block, the function calls `progress(done, total)`
    /// and examines `stop`. Gives the size of the complete file.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] for an address that is not `http` or `https`,
    /// a status that is not 200 or 206, a file that is larger than
    /// `max_bytes`, a failure of the file or the network, or a set `stop`
    /// ([`WebErrorKind::Stopped`]). The partial file stays in each case.
    pub fn get_to_file(
        &self,
        url: &str,
        part: &Path,
        max_bytes: u64,
        progress: &mut dyn FnMut(u64, Option<u64>),
        stop: &AtomicBool,
    ) -> Result<u64, WebError> {
        let lower = url.trim().to_ascii_lowercase();
        if !(lower.starts_with("http://") || lower.starts_with("https://")) {
            return Err(WebError::new(
                WebErrorKind::InvalidAddress,
                format!("{url} is not an http or https address"),
            ));
        }
        let io_error = |e: std::io::Error| {
            WebError::new(WebErrorKind::Network, format!("{}: {e}", part.display()))
        };
        let mut have = std::fs::metadata(part).map_or(0, |m| m.len());
        let mut response = self.request_from(url, have)?;
        if response.status().as_u16() == 416 && have > 0 {
            // The partial file does not fit the file on the server. Start
            // again from 0.
            have = 0;
            response = self.request_from(url, 0)?;
        }
        let status = response.status().as_u16();
        let appending = match status {
            206 => {
                let start = header(&response, "content-range")
                    .as_deref()
                    .and_then(|v| v.strip_prefix("bytes "))
                    .and_then(|v| v.split('-').next())
                    .and_then(|v| v.trim().parse::<u64>().ok());
                if start != Some(have) {
                    return Err(WebError::new(
                        WebErrorKind::Network,
                        format!("{url} answered with a range that does not fit the partial file"),
                    ));
                }
                true
            }
            200 => false,
            _ => {
                return Err(WebError {
                    kind: WebErrorKind::Status,
                    status: Some(status),
                    message: format!("{url} answered with status {status}"),
                });
            }
        };
        let offset = if appending { have } else { 0 };
        let total = header(&response, "content-length")
            .and_then(|v| v.parse::<u64>().ok())
            .map(|length| length + offset);
        let too_large = || {
            WebError::new(
                WebErrorKind::TooLarge,
                format!("{url} is larger than {max_bytes} bytes"),
            )
        };
        if total.is_some_and(|total| total > max_bytes) {
            return Err(too_large());
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(appending)
            .truncate(!appending)
            .open(part)
            .map_err(io_error)?;
        let mut reader = response.body_mut().with_config().limit(u64::MAX).reader();
        let mut done = offset;
        let mut block = vec![0_u8; BLOCK_BYTES];
        progress(done, total);
        loop {
            if stop.load(Ordering::Relaxed) {
                return Err(WebError::new(
                    WebErrorKind::Stopped,
                    format!("{url}: the download stopped"),
                ));
            }
            let read = reader
                .read(&mut block)
                .map_err(|e| WebError::new(WebErrorKind::Network, format!("{url}: {e}")))?;
            if read == 0 {
                break;
            }
            file.write_all(&block[..read]).map_err(io_error)?;
            done += read as u64;
            if done > max_bytes {
                return Err(too_large());
            }
            progress(done, total);
        }
        file.flush().map_err(io_error)?;
        if total.is_some_and(|total| total != done) {
            return Err(WebError::new(
                WebErrorKind::Network,
                format!("{url}: the transfer ended before the end of the file"),
            ));
        }
        Ok(done)
    }

    fn request_from(
        &self,
        url: &str,
        have: u64,
    ) -> Result<ureq::http::Response<ureq::Body>, WebError> {
        let mut request = self.transfer.get(url.trim());
        if have > 0 {
            request = request.header("Range", format!("bytes={have}-"));
        }
        request.call().map_err(|e| network_error(url, &e))
    }
}

/// An open response for a file or a range of a file. The caller reads the
/// body as a stream (ADR 0062 item 27).
pub struct WebStream {
    /// The status: 200, 206 or 416.
    pub status: u16,
    /// The `Content-Range` header of the response, when it has one.
    pub content_range: Option<String>,
    /// The `Content-Length` header of the response, when it has one.
    pub content_length: Option<u64>,
    /// The media type without parameters, when the response gives one.
    pub media_type: Option<String>,
    /// The body of the response. It reads from the connection.
    pub reader: Box<dyn Read + Send>,
}

impl Web {
    /// Open a file, or a range of it, and do not read the body.
    ///
    /// `range` is the value of a `Range` header, such as `bytes=0-99`. The
    /// client follows each redirect, and the policy of the client applies
    /// to the address and to each redirect (ADR 0057). The transfer has no
    /// time limit for the whole body, because a player reads for minutes.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] for an address that is not `http` or `https`, a
    /// status that is not 200, 206 or 416, a refused address, or a network
    /// failure.
    pub fn open_range(&self, url: &str, range: Option<&str>) -> Result<WebStream, WebError> {
        let lower = url.trim().to_ascii_lowercase();
        if !(lower.starts_with("http://") || lower.starts_with("https://")) {
            return Err(WebError::new(
                WebErrorKind::InvalidAddress,
                format!("{url} is not an http or https address"),
            ));
        }
        let mut request = self.transfer.get(url.trim());
        if let Some(range) = range {
            request = request.header("Range", range);
        }
        let response = request.call().map_err(|e| network_error(url, &e))?;
        let status = response.status().as_u16();
        if !matches!(status, 200 | 206 | 416) {
            return Err(WebError {
                kind: WebErrorKind::Status,
                status: Some(status),
                message: format!("{url} answered with status {status}"),
            });
        }
        let content_range = header(&response, "content-range");
        let content_length = header(&response, "content-length").and_then(|v| v.parse().ok());
        let media_type = header(&response, "content-type")
            .as_deref()
            .and_then(media_type);
        let reader = response
            .into_body()
            .into_with_config()
            .limit(u64::MAX)
            .reader();
        Ok(WebStream {
            status,
            content_range,
            content_length,
            media_type,
            reader: Box::new(reader),
        })
    }
}

/// The error for a failed request. A host that the policy refuses gets its
/// own kind.
fn network_error(url: &str, error: &ureq::Error) -> WebError {
    match net::refused_host(error) {
        Some(host) => WebError::new(WebErrorKind::NotPublic, net::not_public_message(host)),
        None => WebError::new(WebErrorKind::Network, format!("{url}: {error}")),
    }
}

fn header<B>(response: &ureq::http::Response<B>, name: &str) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}

/// The media type of a `Content-Type` value, without its parameters.
fn media_type(value: &str) -> Option<String> {
    let essence = value.split(';').next().unwrap_or_default().trim();
    (!essence.is_empty()).then(|| essence.to_ascii_lowercase())
}

/// Encode a text as one segment of a URL path. The text can contain `/`,
/// for example the GUID of an item that is an address.
pub(crate) fn path_segment(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            // Writing to a `String` cannot fail.
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

/// Encode a host path for a URL. Each segment is encoded, and `/` stays the
/// separator. A file name with a space thus gives a correct address.
#[must_use]
pub fn url_path(path: &str) -> String {
    path.split('/')
        .map(path_segment)
        .collect::<Vec<_>>()
        .join("/")
}

/// The host name of an address, in lowercase, without a port.
#[must_use]
pub fn host_of(url: &str) -> Option<String> {
    let uri: ureq::http::Uri = url.trim().parse().ok()?;
    uri.host().map(str::to_ascii_lowercase)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TestServer;

    #[test]
    fn a_file_reads_and_errors_have_a_kind() {
        let server = TestServer::start(vec![
            ("/feed.xml", 200, b"<rss/>".to_vec()),
            ("/big.xml", 200, vec![b'x'; 2000]),
            ("/gone.xml", 404, b"no".to_vec()),
        ]);
        let web = Web::new();
        let file = web.get(&server.url("/feed.xml"), 1000).unwrap();
        assert_eq!(file.bytes, b"<rss/>");
        assert_eq!(file.final_url, server.url("/feed.xml"));
        assert_eq!(
            web.get(&server.url("/big.xml"), 1000).unwrap_err().kind(),
            WebErrorKind::TooLarge
        );
        let error = web.get(&server.url("/gone.xml"), 1000).unwrap_err();
        assert_eq!(
            (error.kind(), error.status()),
            (WebErrorKind::Status, Some(404))
        );
        assert_eq!(
            web.get("file:///etc/passwd", 1000).unwrap_err().kind(),
            WebErrorKind::InvalidAddress
        );
    }

    #[test]
    fn get_image_reads_the_bytes_and_the_type_or_gives_not_image_or_too_large() {
        let server = TestServer::start(Vec::new());
        server.set_file(
            "/cover.png",
            vec![1, 2, 3, 4],
            "image/png; charset=binary",
            true,
        );
        server.set_file("/page.html", b"<html></html>".to_vec(), "text/html", true);
        server.set_file(
            "/huge.png",
            vec![0; usize::try_from(MAX_IMAGE_BYTES + 1).unwrap()],
            "image/png",
            true,
        );
        let web = Web::new();

        let image = web.get_image(&server.url("/cover.png")).unwrap();
        assert_eq!(image.bytes, vec![1, 2, 3, 4]);
        assert_eq!(image.media_type, "image/png");

        assert_eq!(
            web.get_image(&server.url("/page.html")).unwrap_err().kind(),
            WebErrorKind::NotImage
        );
        assert_eq!(
            web.get_image(&server.url("/huge.png")).unwrap_err().kind(),
            WebErrorKind::TooLarge
        );
    }

    #[test]
    fn media_facts_come_from_head_or_from_a_range_request() {
        let server = TestServer::start(Vec::new());
        server.set_file("/a.mp3", vec![0; 1234], "audio/mpeg; charset=binary", true);
        server.set_file("/b.mp3", vec![0; 5678], "audio/mp4", false);
        server.set("/old.mp3", 301, server.url("/a.mp3").into_bytes());
        let web = Web::new();
        let a = MediaFacts {
            length: Some(1234),
            media_type: Some("audio/mpeg".to_owned()),
        };
        assert_eq!(web.media_facts(&server.url("/a.mp3")).unwrap(), a);
        assert_eq!(web.media_facts(&server.url("/old.mp3")).unwrap(), a);
        assert_eq!(
            web.media_facts(&server.url("/b.mp3")).unwrap(),
            MediaFacts {
                length: Some(5678),
                media_type: Some("audio/mp4".to_owned()),
            }
        );
        let error = web.media_facts(&server.url("/gone.mp3")).unwrap_err();
        assert_eq!(error.status(), Some(404));
    }

    #[test]
    fn a_download_continues_a_partial_file_with_a_range_request() {
        use crate::digest::sha256_hex;
        use crate::testing::RangeServer;
        use std::sync::atomic::AtomicBool;

        let body: Vec<u8> = (0..1_000_000_u32).map(|i| (i % 251) as u8).collect();
        let server = RangeServer::start(body.clone());
        server.cut_after(Some(300 * 1024));
        let dir = tempfile::tempdir().unwrap();
        let part = dir.path().join(".a.mp3.part");
        let web = Web::new();
        let stop = AtomicBool::new(false);
        let url = server.url("/a.mp3");

        let error = web
            .get_to_file(&url, &part, u64::MAX, &mut |_, _| {}, &stop)
            .unwrap_err();
        assert_eq!(error.kind(), WebErrorKind::Network);
        let have = std::fs::metadata(&part).unwrap().len();
        assert!(have > 0 && have <= 300 * 1024, "{have}");

        server.cut_after(None);
        let mut last = (0, None);
        let size = web
            .get_to_file(&url, &part, u64::MAX, &mut |d, t| last = (d, t), &stop)
            .unwrap();
        assert_eq!(size, 1_000_000);
        assert_eq!(last, (1_000_000, Some(1_000_000)));
        let ranges = server.ranges();
        assert_eq!(ranges[0], None);
        assert_eq!(
            ranges[1].as_deref(),
            Some(format!("bytes={have}-").as_str())
        );
        let bytes = std::fs::read(&part).unwrap();
        assert_eq!(sha256_hex(&[&bytes]), sha256_hex(&[&body]));
    }

    #[test]
    fn a_download_starts_again_after_a_200_answer_and_obeys_the_limit_and_stop() {
        use std::sync::atomic::AtomicBool;

        // The plain test server ignores a range request and answers 200.
        let server = TestServer::start(Vec::new());
        server.set("/a.bin", 200, vec![7; 1000]);
        let dir = tempfile::tempdir().unwrap();
        let part = dir.path().join("a.part");
        std::fs::write(&part, vec![1; 400]).unwrap();
        let web = Web::new();
        let stop = AtomicBool::new(false);
        let size = web
            .get_to_file(&server.url("/a.bin"), &part, 2000, &mut |_, _| {}, &stop)
            .unwrap();
        assert_eq!(size, 1000);
        assert_eq!(std::fs::read(&part).unwrap(), vec![7; 1000]);

        let error = web
            .get_to_file(&server.url("/a.bin"), &part, 999, &mut |_, _| {}, &stop)
            .unwrap_err();
        assert_eq!(error.kind(), WebErrorKind::TooLarge);

        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let error = web
            .get_to_file(&server.url("/a.bin"), &part, 2000, &mut |_, _| {}, &stop)
            .unwrap_err();
        assert_eq!(error.kind(), WebErrorKind::Stopped);
    }

    #[test]
    fn the_policy_public_refuses_a_loopback_address_and_open_reaches_it() {
        let server = TestServer::start(vec![("/a.xml", 200, b"<rss/>".to_vec())]);
        let url = server.url("/a.xml");
        let public = Web::with_policy(&NetPolicy::public());
        for error in [
            public.get(&url, 1000).unwrap_err(),
            public.get_image(&url).unwrap_err(),
            public.media_facts(&url).unwrap_err(),
        ] {
            assert_eq!(error.kind(), WebErrorKind::NotPublic);
            assert_eq!(
                error.to_string(),
                "Studio on a server cannot connect to 127.0.0.1, because it is not a public address."
            );
        }
        let dir = tempfile::tempdir().unwrap();
        let part = dir.path().join("a.part");
        let stop = AtomicBool::new(false);
        let error = public
            .get_to_file(&url, &part, 1000, &mut |_, _| {}, &stop)
            .unwrap_err();
        assert_eq!(error.kind(), WebErrorKind::NotPublic);
        let open = Web::with_policy(&NetPolicy::Open);
        assert_eq!(open.get(&url, 1000).unwrap().bytes, b"<rss/>");
    }

    #[test]
    fn the_policy_public_checks_the_target_of_a_redirect() {
        use crate::net::PublicRules;

        let target = TestServer::start(vec![("/b.xml", 200, b"<rss/>".to_vec())]);
        let first = TestServer::start(Vec::new());
        first.set("/a.xml", 302, target.url("/b.xml").into_bytes());
        // Only the first server is an exception. The target is not.
        let exception = std::net::SocketAddr::from(([127, 0, 0, 1], first.port()));
        let policy = NetPolicy::Public(PublicRules::with_test_exceptions(vec![exception]));
        let web = Web::with_policy(&policy);

        let error = web.get(&first.url("/a.xml"), 1000).unwrap_err();
        assert_eq!(error.kind(), WebErrorKind::NotPublic);

        // The exception alone is enough for a file with no redirect.
        first.set("/c.xml", 200, b"<rss/>".to_vec());
        assert_eq!(
            web.get(&first.url("/c.xml"), 1000).unwrap().bytes,
            b"<rss/>"
        );
    }

    #[test]
    fn host_of_gives_the_lowercase_host_without_the_port() {
        assert_eq!(
            host_of("https://WavLake.com/feed/x").as_deref(),
            Some("wavlake.com")
        );
        assert_eq!(
            host_of("http://127.0.0.1:8080/a.xml").as_deref(),
            Some("127.0.0.1")
        );
        assert_eq!(host_of("not an address"), None);
    }
}

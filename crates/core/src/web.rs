//! Read public files over HTTP and HTTPS (ADR 0020).
//!
//! The desktop shell reads the addresses that its own user gives. The
//! server shell must also refuse addresses in private networks before it
//! uses this module, or a user can make the server read them (the
//! Server Shell phase).

use std::fmt;
use std::time::Duration;

use ureq::ResponseExt;

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
}

impl Default for Web {
    fn default() -> Self {
        Self::new()
    }
}

impl Web {
    /// Make a client. Each request has a time limit of 20 seconds and a
    /// maximum of 5 redirects.
    #[must_use]
    pub fn new() -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(20)))
            .http_status_as_error(false)
            .max_redirects(5)
            .user_agent(concat!("musicindex-studio/", env!("CARGO_PKG_VERSION")))
            .build();
        Self {
            agent: config.into(),
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
        let mut response = request
            .call()
            .map_err(|e| WebError::new(WebErrorKind::Network, format!("{url}: {e}")))?;
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

/// The size and the media type of a file, as its host gives them
/// (ADR 0023 item 5).
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize)]
pub struct MediaFacts {
    /// The size in bytes, or `None` when the host does not give it.
    pub length: Option<u64>,
    /// The media type without parameters, such as `audio/mpeg`, or `None`.
    pub media_type: Option<String>,
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
            .map_err(|e| WebError::new(WebErrorKind::Network, format!("{url}: {e}")))?;
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
                .map_err(|e| WebError::new(WebErrorKind::Network, format!("{url}: {e}")))?;
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

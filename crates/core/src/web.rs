//! Read public files over HTTP and HTTPS (ADR 0020).
//!
//! The desktop shell reads the addresses that its own user gives. The
//! server shell must also refuse addresses in private networks before it
//! uses this module, or a user can make the server read them (Phase 9).

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

//! The check of a live stream (ADR 0162 items 2 and 3).
//!
//! The check reads the stream address for a short time and judges from the
//! form of the answer if the stream is live. It writes nothing. The
//! network policy of the [`Web`] client applies, so that the server refuses
//! a private address (ADR 0057).

use std::io::Read;

use std::time::{Duration, Instant};
use ureq::ResponseExt as _;

use crate::web::{Web, WebErrorKind};

/// The most seconds that a stream check reads (ADR 0162 item 2).
pub const STREAM_CHECK_SECONDS: u64 = 8;
/// The most bytes that a stream check reads (ADR 0162 item 2).
pub const STREAM_CHECK_BYTES: u64 = 256 * 1024;

/// The result of a stream check.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct StreamStatus {
    /// True when the stream is live.
    pub live: bool,
    /// The type that the stream server sends, without parameters.
    pub server_type: Option<String>,
    /// The reason in English when the stream is not live.
    pub cause: Option<String>,
}

impl StreamStatus {
    fn not_live(server_type: Option<String>, cause: impl Into<String>) -> Self {
        Self {
            live: false,
            server_type,
            cause: Some(cause.into()),
        }
    }

    fn live(server_type: Option<String>) -> Self {
        Self {
            live: true,
            server_type,
            cause: None,
        }
    }

    /// The text of the refusal of a change to `live` (ADR 0162 item 6), or
    /// `None` when the stream is live.
    #[must_use]
    pub fn refusal(&self) -> Option<String> {
        if self.live {
            return None;
        }
        let cause = self
            .cause
            .as_deref()
            .unwrap_or("The stream gives no answer.");
        Some(format!(
            "Studio cannot go live, because the stream is not live: {cause}"
        ))
    }
}

/// Find if the stream of a live show is live (ADR 0162 item 2).
pub trait StreamCheck {
    /// Check the stream at `url`. A failure is a result that is not live.
    fn check_stream(&self, url: &str) -> StreamStatus;
}

impl StreamCheck for Web {
    fn check_stream(&self, url: &str) -> StreamStatus {
        self.check_stream_within(
            url,
            Duration::from_secs(STREAM_CHECK_SECONDS),
            STREAM_CHECK_BYTES,
        )
    }
}

/// The size of the first read of an answer.
const FIRST_READ: usize = 16 * 1024;

impl Web {
    /// Check a stream with a time limit and a size limit. The time limit
    /// covers the request, the read, and the read of a variant playlist.
    #[must_use]
    pub fn check_stream_within(&self, url: &str, time: Duration, bytes: u64) -> StreamStatus {
        self.judge(url, Instant::now() + time, time, bytes, 0)
    }

    fn judge(
        &self,
        url: &str,
        deadline: Instant,
        time: Duration,
        bytes: u64,
        depth: u8,
    ) -> StreamStatus {
        let timeout = || {
            format!(
                "The stream server did not answer in {} seconds.",
                time.as_secs().max(1)
            )
        };
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return StreamStatus::not_live(None, timeout());
        }
        let mut response = match self.open_stream(url, remaining) {
            Ok(response) => response,
            Err(error) => {
                let cause = match error.kind() {
                    WebErrorKind::InvalidAddress => {
                        "The stream address is not an http or https address.".to_owned()
                    }
                    WebErrorKind::NotPublic => error.to_string(),
                    _ if error.to_string() == "timeout" => timeout(),
                    _ => format!("The stream server did not answer: {error}"),
                };
                return StreamStatus::not_live(None, cause);
            }
        };
        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            return StreamStatus::not_live(
                None,
                format!("The stream server answered with status {status}."),
            );
        }
        let server_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| {
                let essence = v.split(';').next().unwrap_or_default().trim();
                (!essence.is_empty()).then(|| essence.to_owned())
            });
        let has_length = response.headers().contains_key("content-length");
        let final_url = response.get_uri().to_string();
        let mut reader = response.body_mut().as_reader();
        let mut head = vec![0_u8; FIRST_READ];
        let read = match reader.read(&mut head) {
            Ok(read) => read,
            Err(error) if is_timeout(&error) => {
                return StreamStatus::not_live(server_type, timeout());
            }
            Err(error) => {
                return StreamStatus::not_live(
                    server_type,
                    format!("The stream server stopped the answer: {error}"),
                );
            }
        };
        head.truncate(read);
        if head.is_empty() {
            return StreamStatus::not_live(server_type, "The stream server sent no data.");
        }
        let lower_type = server_type
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let sniff = String::from_utf8_lossy(&head[..head.len().min(512)]).into_owned();
        let sniff = sniff.trim_start_matches('\u{feff}').trim_start();
        let is_hls = sniff.starts_with("#EXTM3U");
        let is_dash = !is_hls && (lower_type.contains("dash+xml") || sniff.contains("<MPD"));
        if is_hls || is_dash {
            // Read the rest of the text, within the size limit.
            let room = bytes.saturating_sub(head.len() as u64);
            let mut rest = Vec::new();
            if let Err(error) = reader.take(room).read_to_end(&mut rest)
                && is_timeout(&error)
            {
                return StreamStatus::not_live(server_type, timeout());
            }
            head.extend_from_slice(&rest);
            let text = String::from_utf8_lossy(&head).into_owned();
            return if is_hls {
                self.judge_hls(&text, &final_url, server_type, deadline, time, bytes, depth)
            } else {
                judge_dash(&text, server_type)
            };
        }
        if lower_type.starts_with("text/") || lower_type.contains("html") {
            return StreamStatus::not_live(
                server_type,
                "The address gives a web page or a text, not a stream.",
            );
        }
        // An audio answer: bytes arrive. Studio needs no more of them.
        if has_length {
            return StreamStatus::not_live(
                server_type,
                "The stream server sends a file with a length, not a live stream.",
            );
        }
        StreamStatus::live(server_type)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the variant playlist check needs the limits of the first check"
    )]
    fn judge_hls(
        &self,
        text: &str,
        url: &str,
        server_type: Option<String>,
        deadline: Instant,
        time: Duration,
        bytes: u64,
        depth: u8,
    ) -> StreamStatus {
        let lines: Vec<&str> = text.lines().map(str::trim).collect();
        if let Some(at) = lines
            .iter()
            .position(|l| l.starts_with("#EXT-X-STREAM-INF"))
        {
            if depth > 0 {
                return StreamStatus::not_live(
                    server_type,
                    "The variant playlist is a main playlist again.",
                );
            }
            let Some(variant) = lines[at + 1..]
                .iter()
                .find(|l| !l.is_empty() && !l.starts_with('#'))
            else {
                return StreamStatus::not_live(server_type, "The main playlist has no variant.");
            };
            let inner = self.judge(&resolve(url, variant), deadline, time, bytes, depth + 1);
            // The cause names the variant, so that the user does not think
            // that the main playlist failed.
            let cause = inner.cause.map(|cause| {
                format!("The main playlist answers, but its first variant {variant} is not live. {cause}")
            });
            return StreamStatus {
                server_type,
                cause,
                ..inner
            };
        }
        if lines.contains(&"#EXT-X-ENDLIST") {
            return StreamStatus::not_live(
                server_type,
                "The playlist has #EXT-X-ENDLIST, so it is a recording.",
            );
        }
        let has_segment = lines.iter().any(|l| !l.is_empty() && !l.starts_with('#'));
        if !has_segment {
            return StreamStatus::not_live(server_type, "The playlist has no segment.");
        }
        StreamStatus::live(server_type)
    }
}

/// A DASH manifest is live when its `type` is `dynamic`.
fn judge_dash(text: &str, server_type: Option<String>) -> StreamStatus {
    let tag = text
        .find("<MPD")
        .and_then(|at| text[at..].split('>').next())
        .unwrap_or_default();
    let mut tag = tag.to_owned();
    while tag.contains(" =") || tag.contains("= ") {
        tag = tag.replace(" =", "=").replace("= ", "=");
    }
    let dynamic = tag
        .split_whitespace()
        .filter_map(|token| token.strip_prefix("type="))
        .any(|value| value.trim_matches(['"', '\'', '/']) == "dynamic");
    if dynamic {
        StreamStatus::live(server_type)
    } else {
        StreamStatus::not_live(
            server_type,
            "The DASH manifest is not dynamic, so it is a recording.",
        )
    }
}

fn is_timeout(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::TimedOut
        || error.to_string().to_ascii_lowercase().contains("timeout")
}

/// The address of a playlist line, from the address of the playlist.
fn resolve(base: &str, reference: &str) -> String {
    if reference.contains("://") {
        return reference.to_owned();
    }
    let base = base.split(['?', '#']).next().unwrap_or(base);
    let (scheme, rest) = base.split_once("://").unwrap_or(("http", base));
    if let Some(network) = reference.strip_prefix("//") {
        return format!("{scheme}://{network}");
    }
    let host_end = rest.find('/').unwrap_or(rest.len());
    let (host, path) = rest.split_at(host_end);
    if reference.starts_with('/') {
        return format!("{scheme}://{host}{reference}");
    }
    let dir = path.rfind('/').map_or("/", |at| &path[..=at]);
    format!("{scheme}://{host}{dir}{reference}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_relative_variant_address_resolves_against_the_playlist() {
        let base = "https://a.example/live/main.m3u8?token=1";
        assert_eq!(resolve(base, "v1.m3u8"), "https://a.example/live/v1.m3u8");
        assert_eq!(resolve(base, "/x/v1.m3u8"), "https://a.example/x/v1.m3u8");
        assert_eq!(
            resolve(base, "//b.example/v.m3u8"),
            "https://b.example/v.m3u8"
        );
        assert_eq!(
            resolve(base, "http://c.example/v.m3u8"),
            "http://c.example/v.m3u8"
        );
    }

    #[test]
    fn a_dash_manifest_is_live_only_when_its_type_is_dynamic() {
        let live =
            r#"<?xml version="1.0"?><MPD xmlns="u" type = "dynamic" minimumUpdatePeriod="PT2S">"#;
        assert!(judge_dash(live, None).live);
        let recording = r#"<MPD type="static" mediaPresentationDuration="PT1H">"#;
        assert!(!judge_dash(recording, None).live);
        assert!(!judge_dash("<MPD xmlns='u'>", None).live);
    }
}

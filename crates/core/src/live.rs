//! The rules of a live item (ADR 0054 items 2 to 5).

use crate::datetime::parse_time;
use crate::feed::{Entry, Feed, LiveItem};

/// The status of a live item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveStatus {
    /// The show has not started.
    Pending,
    /// The show is on air.
    Live,
    /// The show is over.
    Ended,
}

impl LiveStatus {
    /// Read the text of a `status` attribute. A text that is not a status
    /// gives none.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "pending" => Some(Self::Pending),
            "live" => Some(Self::Live),
            "ended" => Some(Self::Ended),
            _ => None,
        }
    }

    /// The text of the status in a feed.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Live => "live",
            Self::Ended => "ended",
        }
    }
}

/// A status change that is not permitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveError {
    message: String,
}

impl std::fmt::Display for LiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for LiveError {}

/// How serious a problem of a live item is (ADR 0161 item 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LiveProblemLevel {
    /// The write stops.
    Refusal,
    /// The write continues.
    Warning,
}

/// A reason why a live item is not correct.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct LiveProblem {
    /// The `guid` of the live item. It is empty when the item has none.
    pub guid: String,
    /// The field with the problem: `start`, `end`, `stream`, `contentLink`
    /// or `status`.
    pub field: String,
    /// Whether the problem stops the write.
    pub level: LiveProblemLevel,
    /// The text for the user.
    pub text: String,
}

/// Make a live item with a new UUID v4 `guid`, the status `pending`, the
/// title and the start time.
#[must_use]
pub fn new_live_item(title: &str, start: &str) -> LiveItem {
    let mut live = LiveItem {
        status: Some(LiveStatus::Pending.as_str().to_owned()),
        start: Some(start.to_owned()),
        ..LiveItem::default()
    };
    live.item.title = Some(title.to_owned());
    live.item.guid = Some(uuid::Uuid::new_v4().to_string());
    live
}

/// Change the status of a live item. Only `pending` to `live` and `live`
/// to `ended` are permitted.
///
/// # Errors
///
/// Gives an error for each other change, or when the item has no status
/// that [`LiveStatus::parse`] accepts.
pub fn set_status(item: &mut LiveItem, next: LiveStatus) -> Result<(), LiveError> {
    let current = item.status.as_deref().and_then(LiveStatus::parse);
    let permitted = matches!(
        (current, next),
        (Some(LiveStatus::Pending), LiveStatus::Live) | (Some(LiveStatus::Live), LiveStatus::Ended)
    );
    if !permitted {
        let from = current.map_or("no status", LiveStatus::as_str);
        return Err(LiveError {
            message: format!("A show cannot change from {from} to {}.", next.as_str()),
        });
    }
    item.status = Some(next.as_str().to_owned());
    Ok(())
}

/// The most characters of the address of a live value.
pub const MAX_LIVE_VALUE_URI: usize = 2048;
/// The most characters of the protocol of a live value.
pub const MAX_LIVE_VALUE_PROTOCOL: usize = 64;

/// Check the address and the protocol of a live value (task 072d). The
/// rule has no test for an empty text, because a draft can hold a value that
/// the user did not finish.
///
/// # Errors
///
/// Gives an error for an address of more than [`MAX_LIVE_VALUE_URI`]
/// characters, a protocol of more than [`MAX_LIVE_VALUE_PROTOCOL`]
/// characters, or a control character in either.
pub fn check_live_value(uri: &str, protocol: &str) -> Result<(), LiveError> {
    let refuse = |message: String| Err(LiveError { message });
    if uri.chars().count() > MAX_LIVE_VALUE_URI {
        return refuse(format!(
            "The live value address has more than {MAX_LIVE_VALUE_URI} characters."
        ));
    }
    if protocol.chars().count() > MAX_LIVE_VALUE_PROTOCOL {
        return refuse(format!(
            "The live value protocol has more than {MAX_LIVE_VALUE_PROTOCOL} characters."
        ));
    }
    if uri.chars().chain(protocol.chars()).any(char::is_control) {
        return refuse("The live value has a control character.".to_owned());
    }
    Ok(())
}

/// The most characters of the protocol of a granted live value (ADR 0110
/// item 26).
pub const MAX_GRANTED_PROTOCOL: usize = 32;

/// The origin of a live value address as `scheme://host[:port]`, in
/// lowercase, with no default port. Gives `None` when the text is not a
/// `https` or `wss` address, or when it has a user name or a password.
fn origin_of(text: &str) -> Option<String> {
    let uri: ureq::http::Uri = text.parse().ok()?;
    let scheme = uri.scheme_str()?.to_ascii_lowercase();
    if scheme != "https" && scheme != "wss" {
        return None;
    }
    let authority = uri.authority()?;
    let host = authority.host();
    if authority.as_str().contains('@') || host.is_empty() {
        return None;
    }
    Some(match authority.port_u16() {
        None | Some(443) => format!("{scheme}://{}", host.to_ascii_lowercase()),
        Some(port) => format!("{scheme}://{}:{port}", host.to_ascii_lowercase()),
    })
}

/// The origin of a live value origin text: the scheme, the host and the
/// port, with no path, query, fragment or user name. The scheme must be
/// `https` or `wss` (ADR 0110 item 24). Gives the origin in the form that
/// [`check_granted_live_value`] compares, or `None`.
#[must_use]
pub fn normalize_origin(text: &str) -> Option<String> {
    let rest = text.split_once("://")?.1;
    if rest.is_empty()
        || rest.contains(['/', '?', '#', '@'])
        || text.chars().any(char::is_whitespace)
    {
        return None;
    }
    origin_of(text)
}

/// Check the live value that a connected app sets under a show grant
/// (ADR 0110 item 26). `origin` is the live value origin that the grant
/// keeps. The check first applies [`check_live_value`]. Then it refuses a
/// protocol that is not 1 to [`MAX_GRANTED_PROTOCOL`] characters of
/// letters, digits, `.` and `-`, and an address whose origin is not
/// `origin`.
///
/// # Errors
///
/// Gives an error in each of those conditions.
pub fn check_granted_live_value(uri: &str, protocol: &str, origin: &str) -> Result<(), LiveError> {
    check_live_value(uri, protocol)?;
    let refuse = |message: &str| {
        Err(LiveError {
            message: message.to_owned(),
        })
    };
    let protocol_ok = !protocol.is_empty()
        && protocol.chars().count() <= MAX_GRANTED_PROTOCOL
        && protocol
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
    if !protocol_ok {
        return refuse(
            "The live value protocol must have 1 to 32 characters: letters, digits, dots and hyphens.",
        );
    }
    if origin_of(uri).as_deref() != Some(origin) {
        return refuse("The live value address is not on the live value origin of the app.");
    }
    Ok(())
}

/// Check the live value of each live item of a feed with
/// [`check_live_value`].
///
/// # Errors
///
/// Gives the first error of [`check_live_value`].
pub fn check_live_values(feed: &Feed) -> Result<(), LiveError> {
    for Entry { value: live, .. } in &feed.live_items {
        if let Some(value) = &live.live_value {
            check_live_value(
                value.uri.as_deref().unwrap_or_default(),
                value.protocol.as_deref().unwrap_or_default(),
            )?;
        }
    }
    Ok(())
}

/// The stream types that each extension of a stream address accepts (ADR 0161
/// item 3).
const STREAM_TYPES: &[(&str, &[&str])] = &[
    (
        "m3u8",
        &[
            "application/x-mpegurl",
            "application/vnd.apple.mpegurl",
            "audio/mpegurl",
            "audio/x-mpegurl",
        ],
    ),
    ("mp3", &["audio/mpeg", "audio/mp3"]),
    ("aac", &["audio/aac", "audio/aacp"]),
    ("m4a", &["audio/mp4", "audio/x-m4a"]),
    ("ogg", &["audio/ogg", "application/ogg"]),
    ("opus", &["audio/ogg", "audio/opus"]),
    ("mpd", &["application/dash+xml"]),
];

/// The extension of the last path part of an address, in lowercase. The
/// query and the fragment do not count. An address with no path has none.
fn address_extension(url: &str) -> Option<String> {
    let url = url.split(['?', '#']).next().unwrap_or(url);
    let after_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    let (_, path) = after_scheme.split_once('/')?;
    let part = path.rsplit('/').next().unwrap_or(path);
    let (_, ext) = part.rsplit_once('.')?;
    Some(ext.to_ascii_lowercase())
}

/// Whether a stream type agrees with the extension of the stream address.
/// The case and the parameters of the type do not count.
fn stream_type_agrees(ext: &str, media_type: &str) -> bool {
    let Some((_, accepted)) = STREAM_TYPES.iter().find(|(e, _)| *e == ext) else {
        return true;
    };
    let base = media_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    accepted.contains(&base.as_str())
}

/// Give the problems of the live items of a feed.
#[must_use]
pub fn problems(feed: &Feed) -> Vec<LiveProblem> {
    use LiveProblemLevel::{Refusal, Warning};
    let mut found = Vec::new();
    for Entry { value: live, .. } in &feed.live_items {
        let guid = live.item.guid.clone().unwrap_or_default();
        let mut add = |level: LiveProblemLevel, field: &str, text: &str| {
            found.push(LiveProblem {
                guid: guid.clone(),
                field: field.to_owned(),
                level,
                text: text.to_owned(),
            });
        };
        // A blank start is the same as no start, because the editor can
        // send an empty text.
        if live.start.as_deref().is_none_or(|s| s.trim().is_empty()) {
            add(Refusal, "start", "This show has no start time.");
        }
        if let Some(start) = live.start.as_deref().filter(|s| !s.trim().is_empty())
            && parse_time(start).is_none()
        {
            add(
                Refusal,
                "start",
                "The start time is not a time with a zone.",
            );
        }
        if let Some(end) = live.end.as_deref().filter(|s| !s.trim().is_empty())
            && parse_time(end).is_none()
        {
            add(Refusal, "end", "The end time is not a time with a zone.");
        }
        let enclosure = live.item.enclosure.as_ref();
        let url = enclosure
            .and_then(|e| e.url.as_deref())
            .map(str::trim)
            .filter(|u| !u.is_empty());
        let media_type = enclosure
            .and_then(|e| e.media_type.as_deref())
            .map(str::trim)
            .filter(|t| !t.is_empty());
        match (url, media_type) {
            (None, _) => add(Refusal, "stream", "This show has no stream address."),
            (Some(_), None) => add(Refusal, "stream", "This show has no stream type."),
            (Some(url), Some(ty)) => {
                if let Some(ext) = address_extension(url)
                    && !stream_type_agrees(&ext, ty)
                {
                    add(
                        Refusal,
                        "stream",
                        &format!(
                            "The stream type {ty} does not agree with the stream address, which ends in .{ext}."
                        ),
                    );
                }
            }
        }
        if live.content_links.is_empty() {
            add(Warning, "contentLink", "This show has no content link.");
        }
        if live.status.as_deref().and_then(LiveStatus::parse).is_none() {
            add(
                Refusal,
                "status",
                "This show has a status that is not pending, live or ended.",
            );
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::Enclosure;

    fn stream(url: &str, media_type: &str) -> Enclosure {
        Enclosure {
            url: Some(url.to_owned()),
            media_type: Some(media_type.to_owned()),
            ..Default::default()
        }
    }

    /// A feed with one live item that has a stream and a content link.
    fn feed_with_stream(url: &str, media_type: &str) -> Feed {
        let mut live = new_live_item("Show", "2026-11-01T10:00:00Z");
        live.item.enclosure = Some(stream(url, media_type));
        live.content_links
            .push(Entry::new(crate::feed::ContentLink {
                href: Some("https://example.com/show".to_owned()),
                ..Default::default()
            }));
        let mut feed = Feed::default();
        feed.live_items.push(Entry::new(live));
        feed
    }

    #[test]
    fn a_live_value_has_a_limit_and_no_control_character() {
        assert!(check_live_value("wss://example.com/value", "socket.io").is_ok());
        assert!(check_live_value("", "").is_ok());
        assert!(check_live_value(&"a".repeat(MAX_LIVE_VALUE_URI), "p").is_ok());
        assert!(check_live_value(&"a".repeat(MAX_LIVE_VALUE_URI + 1), "p").is_err());
        assert!(check_live_value("u", &"a".repeat(MAX_LIVE_VALUE_PROTOCOL + 1)).is_err());
        assert!(check_live_value("wss://a\n.example", "p").is_err());
        assert!(check_live_value("u", "p\u{7}").is_err());
    }

    #[test]
    fn a_granted_live_value_must_be_on_the_origin_of_the_grant() {
        let origin = "wss://live.example.com";
        let check = |uri: &str, protocol: &str| check_granted_live_value(uri, protocol, origin);
        assert!(check("wss://live.example.com/value", "socket.io").is_ok());
        assert!(check("wss://LIVE.example.com:443/v?x=1", "a-b.c").is_ok());
        let cases = [
            ("wss://other.example.com/value", "p", "other host"),
            ("ws://live.example.com/value", "p", "ws scheme"),
            ("https://live.example.com/value", "p", "other scheme"),
            ("wss://live.example.com:8443/value", "p", "other port"),
            ("wss://live.example.com@evil.example/", "p", "user name"),
            ("wss://user@live.example.com/", "p", "user name, same host"),
            ("wss://live.example.com.evil.example/", "p", "suffix host"),
            ("/value", "p", "no host"),
            ("", "p", "empty"),
            ("wss://live.example.com/v", "", "empty protocol"),
            ("wss://live.example.com/v", "a b", "space in protocol"),
            ("wss://live.example.com/v", "a/b", "slash in protocol"),
        ];
        for (uri, protocol, name) in cases {
            assert!(
                check(uri, protocol).is_err(),
                "ADR 0110 item 26: a granted live value with {name} must be refused. Fix check_granted_live_value."
            );
        }
        assert!(check(&format!("wss://live.example.com/{}", "a".repeat(2048)), "p").is_err());
        assert!(check("wss://live.example.com/v", &"a".repeat(33)).is_err());
        assert!(check("wss://live.example.com/v", &"a".repeat(32)).is_ok());
        assert!(check("wss://live.example.com/\u{7}", "p").is_err());
    }

    #[test]
    fn an_origin_has_a_scheme_a_host_and_no_path() {
        assert_eq!(
            normalize_origin("https://Live.Example.com").as_deref(),
            Some("https://live.example.com")
        );
        assert_eq!(
            normalize_origin("wss://live.example.com:443").as_deref(),
            Some("wss://live.example.com")
        );
        assert_eq!(
            normalize_origin("https://live.example.com:8443").as_deref(),
            Some("https://live.example.com:8443")
        );
        for text in [
            "http://live.example.com",
            "ws://live.example.com",
            "https://live.example.com/",
            "https://live.example.com/path",
            "https://live.example.com?x=1",
            "https://live.example.com#x",
            "https://user@live.example.com",
            "https://",
            "live.example.com",
            "",
            "https://live example.com",
        ] {
            assert_eq!(normalize_origin(text), None, "{text}");
        }
    }

    #[test]
    fn a_new_live_item_is_pending_with_a_uuid() {
        let live = new_live_item("Show", "2026-11-01T10:00:00Z");
        assert_eq!(live.status.as_deref(), Some("pending"));
        assert_eq!(live.start.as_deref(), Some("2026-11-01T10:00:00Z"));
        assert_eq!(live.item.title.as_deref(), Some("Show"));
        let guid = live.item.guid.unwrap();
        assert!(uuid::Uuid::parse_str(&guid).is_ok());
    }

    #[test]
    fn a_blank_start_is_a_problem() {
        let mut feed = Feed::default();
        let mut live = new_live_item("Show", " ");
        live.item.enclosure = Some(stream("https://example.com/live.mp3", "audio/mpeg"));
        live.content_links
            .push(Entry::new(crate::feed::ContentLink {
                href: Some("https://example.com/show".to_owned()),
                ..Default::default()
            }));
        feed.live_items.push(Entry::new(live));
        let found = problems(&feed);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].text, "This show has no start time.");
    }

    fn feed_with_times(start: &str, end: Option<&str>) -> Feed {
        let mut feed = Feed::default();
        let mut live = new_live_item("Show", start);
        live.end = end.map(str::to_owned);
        live.item.enclosure = Some(stream("https://example.com/live.mp3", "audio/mpeg"));
        live.content_links
            .push(Entry::new(crate::feed::ContentLink {
                href: Some("https://example.com/show".to_owned()),
                ..Default::default()
            }));
        feed.live_items.push(Entry::new(live));
        feed
    }

    #[test]
    fn a_start_with_no_zone_is_a_problem() {
        let found = problems(&feed_with_times("2026-10-30T20:00", None));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].field, "start");
        assert_eq!(found[0].text, "The start time is not a time with a zone.");
    }

    #[test]
    fn an_end_that_is_not_a_time_is_a_problem_and_a_blank_end_is_not() {
        let found = problems(&feed_with_times("2026-10-30T20:00:00Z", Some("soon")));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].field, "end");
        assert_eq!(found[0].text, "The end time is not a time with a zone.");
        assert_eq!(
            problems(&feed_with_times("2026-10-30T20:00:00Z", Some(" "))),
            Vec::new()
        );
        assert_eq!(
            problems(&feed_with_times("2026-10-30T20:00:00Z", None)),
            Vec::new()
        );
    }

    #[test]
    fn the_status_changes_in_one_direction() {
        let mut live = new_live_item("Show", "2026-11-01T10:00:00Z");
        assert!(set_status(&mut live, LiveStatus::Ended).is_err());
        assert_eq!(live.status.as_deref(), Some("pending"));
        set_status(&mut live, LiveStatus::Live).unwrap();
        assert_eq!(live.status.as_deref(), Some("live"));
        let error = set_status(&mut live, LiveStatus::Pending).unwrap_err();
        assert!(error.to_string().contains("live") && error.to_string().contains("pending"));
        set_status(&mut live, LiveStatus::Ended).unwrap();
        assert_eq!(live.status.as_deref(), Some("ended"));
        let error = set_status(&mut live, LiveStatus::Live).unwrap_err();
        assert!(error.to_string().contains("ended") && error.to_string().contains("live"));
        assert_eq!(live.status.as_deref(), Some("ended"));
    }

    #[test]
    fn a_status_text_parses_only_when_it_is_a_status() {
        assert_eq!(LiveStatus::parse("live"), Some(LiveStatus::Live));
        assert_eq!(LiveStatus::parse("Live"), None);
        assert_eq!(LiveStatus::Ended.as_str(), "ended");
    }

    #[test]
    fn a_live_item_with_no_content_link_gets_a_warning() {
        let mut feed = feed_with_stream("https://example.com/live.mp3", "audio/mpeg");
        feed.live_items[0].value.content_links.clear();
        let found = problems(&feed);
        assert_eq!(
            found.len(),
            1,
            "ADR 0161 item 4: no content link is the only problem. Fix problems."
        );
        assert_eq!(found[0].field, "contentLink");
        assert_eq!(found[0].level, LiveProblemLevel::Warning);
        assert_eq!(found[0].text, "This show has no content link.");
    }

    #[test]
    fn a_new_live_item_has_no_stream_and_no_content_link() {
        let mut feed = Feed::default();
        feed.live_items
            .push(Entry::new(new_live_item("Show", "2026-11-01T10:00:00Z")));
        let found = problems(&feed);
        let fields: Vec<_> = found.iter().map(|p| (p.field.as_str(), p.level)).collect();
        assert_eq!(
            fields,
            [
                ("stream", LiveProblemLevel::Refusal),
                ("contentLink", LiveProblemLevel::Warning)
            ]
        );
        assert_eq!(found[0].text, "This show has no stream address.");
    }

    #[test]
    fn a_live_item_with_no_usable_stream_gets_a_refusal() {
        let no_type = |url: &str| {
            let mut feed = feed_with_stream(url, "audio/mpeg");
            feed.live_items[0]
                .value
                .item
                .enclosure
                .as_mut()
                .unwrap()
                .media_type = None;
            feed
        };
        let mut no_enclosure = feed_with_stream("https://example.com/live.mp3", "audio/mpeg");
        no_enclosure.live_items[0].value.item.enclosure = None;
        let mut no_url = feed_with_stream("x", "audio/mpeg");
        no_url.live_items[0]
            .value
            .item
            .enclosure
            .as_mut()
            .unwrap()
            .url = None;
        let cases = [
            (
                "no enclosure",
                no_enclosure,
                "This show has no stream address.",
            ),
            ("no url", no_url, "This show has no stream address."),
            (
                "blank url",
                feed_with_stream("  ", "audio/mpeg"),
                "This show has no stream address.",
            ),
            (
                "no type",
                no_type("https://example.com/live.mp3"),
                "This show has no stream type.",
            ),
            (
                "blank type",
                feed_with_stream("https://example.com/live.mp3", " "),
                "This show has no stream type.",
            ),
            (
                "type that disagrees",
                feed_with_stream("https://example.com/live.m3u8", "audio/mpeg"),
                "The stream type audio/mpeg does not agree with the stream address, which ends in .m3u8.",
            ),
        ];
        for (name, feed, text) in cases {
            let found = problems(&feed);
            assert_eq!(
                found.len(),
                1,
                "ADR 0161 items 1 to 3: a show with {name} must give one refusal. Fix problems."
            );
            assert_eq!(found[0].field, "stream", "{name}");
            assert_eq!(found[0].level, LiveProblemLevel::Refusal, "{name}");
            assert_eq!(found[0].text, text, "{name}");
        }
    }

    #[test]
    fn a_stream_type_that_agrees_or_has_no_known_extension_gives_no_problem() {
        let cases = [
            (
                "https://example.com/pc20/livestream?format=.mp3",
                "audio/mpeg",
            ),
            ("https://example.com/pc20/livestream#.m3u8", "audio/mpeg"),
            ("https://stream.example.com/live", "audio/mpeg"),
            ("https://stream.example.com", "audio/mpeg"),
            ("https://stream.example.com/live.pls", "audio/x-scpls"),
            ("https://example.com/a.b/live", "audio/aac"),
            (
                "https://example.com/live.M3U8",
                "application/vnd.apple.mpegurl; charset=x",
            ),
            ("https://example.com/live.m3u8?token=1", " Audio/X-MpegURL "),
            ("https://example.com/live.mp3", "audio/mp3"),
            ("https://example.com/live.opus", "audio/opus"),
            ("https://example.com/live.mpd", "application/dash+xml"),
        ];
        for (url, media_type) in cases {
            assert_eq!(
                problems(&feed_with_stream(url, media_type)),
                Vec::new(),
                "ADR 0161 item 3: {url} with {media_type} must give no problem. Fix address_extension or stream_type_agrees."
            );
        }
    }

    #[test]
    fn a_stream_type_message_gives_the_type_as_written_and_the_extension_in_lowercase() {
        let found = problems(&feed_with_stream(
            "https://example.com/live.MP3?x=1",
            "Audio/AAC; q=1",
        ));
        assert_eq!(found.len(), 1);
        assert_eq!(
            found[0].text,
            "The stream type Audio/AAC; q=1 does not agree with the stream address, which ends in .mp3."
        );
    }

    #[test]
    fn a_live_item_with_no_start_and_a_wrong_status_has_two_problems() {
        let mut live = new_live_item("Show", "x");
        live.start = None;
        live.status = Some("soon".into());
        live.item.enclosure = Some(stream("https://example.com/live.mp3", "audio/mpeg"));
        live.content_links
            .push(Entry::new(crate::feed::ContentLink::default()));
        let mut feed = Feed::default();
        feed.live_items.push(Entry::new(live));
        let fields: Vec<_> = problems(&feed).into_iter().map(|p| p.field).collect();
        assert_eq!(fields, ["start", "status"]);
    }
}

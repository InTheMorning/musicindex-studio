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

/// A reason why a live item is not correct.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct LiveProblem {
    /// The `guid` of the live item. It is empty when the item has none.
    pub guid: String,
    /// The field with the problem: `start`, `end`, `contentLink` or `status`.
    pub field: String,
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

/// Give the problems of the live items of a feed.
#[must_use]
pub fn problems(feed: &Feed) -> Vec<LiveProblem> {
    let mut found = Vec::new();
    for Entry { value: live, .. } in &feed.live_items {
        let guid = live.item.guid.clone().unwrap_or_default();
        let mut add = |field: &str, text: &str| {
            found.push(LiveProblem {
                guid: guid.clone(),
                field: field.to_owned(),
                text: text.to_owned(),
            });
        };
        // A blank start is the same as no start, because the editor can
        // send an empty text.
        if live.start.as_deref().is_none_or(|s| s.trim().is_empty()) {
            add("start", "This show has no start time.");
        }
        if let Some(start) = live.start.as_deref().filter(|s| !s.trim().is_empty())
            && parse_time(start).is_none()
        {
            add("start", "The start time is not a time with a zone.");
        }
        if let Some(end) = live.end.as_deref().filter(|s| !s.trim().is_empty())
            && parse_time(end).is_none()
        {
            add("end", "The end time is not a time with a zone.");
        }
        if live.content_links.is_empty() {
            add("contentLink", "This show has no content link.");
        }
        if live.status.as_deref().and_then(LiveStatus::parse).is_none() {
            add(
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
    fn a_new_live_item_has_no_content_link() {
        let mut feed = Feed::default();
        feed.live_items
            .push(Entry::new(new_live_item("Show", "2026-11-01T10:00:00Z")));
        let found = problems(&feed);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].field, "contentLink");
        assert_eq!(found[0].text, "This show has no content link.");
    }

    #[test]
    fn a_live_item_with_no_start_and_a_wrong_status_has_two_problems() {
        let mut live = new_live_item("Show", "x");
        live.start = None;
        live.status = Some("soon".into());
        live.content_links
            .push(Entry::new(crate::feed::ContentLink::default()));
        let mut feed = Feed::default();
        feed.live_items.push(Entry::new(live));
        let fields: Vec<_> = problems(&feed).into_iter().map(|p| p.field).collect();
        assert_eq!(fields, ["start", "status"]);
    }
}

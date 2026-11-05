//! The checked time and the new build date (ADR 0127).
//!
//! The module knows no draft, no host and no collection. It gets the URLs
//! and the clock from its caller. The checked time is the earliest of the
//! computer clock and the `Date` header of each server that answered.

use std::collections::BTreeMap;

use crate::datetime::{self, Moment};
use crate::net::NetPolicy;
use crate::web::Web;

/// The seconds that the core keeps the difference of a server (ADR 0127
/// item 13).
pub const KEEP_SECONDS: i64 = 600;

/// The seconds that the computer clock can be ahead before Review gives the
/// notice (ADR 0127 item 12). The notice applies for a larger difference.
pub const CLOCK_NOTICE_SECONDS: i64 = 300;

/// The source of the time of a server.
pub trait ServerTime: Send + Sync {
    /// The Unix seconds in the `Date` header of the server at `url`, or
    /// `None` when the server does not answer with a readable `Date`.
    fn server_time(&self, url: &str) -> Option<i64>;
}

/// The real [`ServerTime`]. It sends one `HEAD` request for each call.
#[derive(Debug, Clone)]
pub struct WebServerTime {
    web: Web,
}

impl WebServerTime {
    /// A source with the policy `Public`, in both shells. The policy of the
    /// shell does not apply (ADR 0127 item 13).
    #[must_use]
    pub fn public() -> Self {
        Self {
            web: Web::with_policy(&NetPolicy::public()),
        }
    }

    /// A source with a client that the caller makes, for a test server.
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn with_web(web: Web) -> Self {
        Self { web }
    }
}

impl ServerTime for WebServerTime {
    fn server_time(&self, url: &str) -> Option<i64> {
        // A cache gives the `Date` of its copy, and `Age` gives the seconds
        // since that time. Without `Age`, a good computer clock would seem
        // ahead (ADR 0127 item 7).
        let (date, age) = self.web.server_date(url)?;
        datetime::parse_rfc2822(&date).map(|moment| moment.seconds + age)
    }
}

/// A [`ServerTime`] for tests. It serves a time for each URL and records
/// each request.
#[cfg(any(test, feature = "testing"))]
#[derive(Debug, Default)]
pub struct ScriptedServerTime {
    times: BTreeMap<String, i64>,
    requests: std::sync::Mutex<Vec<String>>,
}

#[cfg(any(test, feature = "testing"))]
impl ScriptedServerTime {
    /// A source with no time. Each request gives `None`.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Serve `seconds` at `url`.
    #[must_use]
    pub fn with_time(mut self, url: &str, seconds: i64) -> Self {
        self.times.insert(url.to_owned(), seconds);
        self
    }

    /// The URL of each request, in sequence.
    ///
    /// # Panics
    ///
    /// Panics when the lock of the list is poisoned.
    #[must_use]
    pub fn requests(&self) -> Vec<String> {
        self.requests
            .lock()
            .expect("the request lock is not poisoned")
            .clone()
    }
}

#[cfg(any(test, feature = "testing"))]
impl ServerTime for ScriptedServerTime {
    fn server_time(&self, url: &str) -> Option<i64> {
        self.requests
            .lock()
            .expect("the request lock is not poisoned")
            .push(url.to_owned());
        self.times.get(url).copied()
    }
}

/// The checked time (ADR 0127 item 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckedTime {
    /// The Unix seconds: the earliest of the computer clock and the time of
    /// each server with a current difference.
    pub seconds: i64,
    /// The seconds that the computer clock is ahead of `seconds`. It is 0
    /// when the computer clock is the earliest.
    pub computer_ahead_seconds: i64,
}

impl CheckedTime {
    /// True when the computer clock is more than 5 minutes ahead of the
    /// checked time (ADR 0127 item 12).
    #[must_use]
    pub fn clock_notice(&self) -> bool {
        self.computer_ahead_seconds > CLOCK_NOTICE_SECONDS
    }
}

/// The differences between the servers and the computer clock, in memory
/// only (ADR 0127 item 13). A new start requests the time again.
#[derive(Debug, Clone, Default)]
pub struct TimeKeeper {
    /// For each URL: the server seconds minus the computer seconds, and the
    /// computer seconds of the measure.
    differences: BTreeMap<String, (i64, i64)>,
}

impl TimeKeeper {
    /// A keeper with no difference.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn current(&self, url: &str, now: i64) -> Option<i64> {
        let (difference, measured) = self.differences.get(url)?;
        (now.saturating_sub(*measured) < KEEP_SECONDS).then_some(*difference)
    }

    /// The URLs of `urls` that have no current difference, in the sequence
    /// of the list and with no repeat.
    #[must_use]
    pub fn missing<'a>(&self, urls: &[&'a str], now: i64) -> Vec<&'a str> {
        let mut missing: Vec<&str> = Vec::new();
        for url in urls {
            if self.current(url, now).is_none() && !missing.contains(url) {
                missing.push(url);
            }
        }
        missing
    }

    /// Keep the answer of a server: its time `server_seconds` at the
    /// computer time `now`. A server that did not answer records nothing.
    pub fn record(&mut self, url: &str, server_seconds: i64, now: i64) {
        self.differences
            .insert(url.to_owned(), (server_seconds - now, now));
    }

    /// The checked time at `now`, or `None` when no difference is current
    /// (ADR 0127 item 8). Each current difference gives the time of its
    /// server at `now`.
    #[must_use]
    pub fn checked(&self, now: i64) -> Option<CheckedTime> {
        let earliest = self
            .differences
            .keys()
            .filter_map(|url| self.current(url, now))
            .map(|difference| now + difference)
            .min()?;
        let seconds = earliest.min(now);
        Some(CheckedTime {
            seconds,
            computer_ahead_seconds: now - seconds,
        })
    }
}

/// What the core does with the build date of a draft (ADR 0127 items 9
/// to 11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildDateOutcome {
    /// Write this text as `lastBuildDate`.
    Write(String),
    /// Keep the base. Its date equals the checked time.
    KeepEqual,
    /// Keep the base. Its date is later than the checked time, and Review
    /// gives the notice of item 11. `base` is the text of the date.
    KeepLater {
        /// The text of the date in the base.
        base: String,
    },
}

/// The rule for the new build date. A date that the core cannot read
/// counts as no date.
#[must_use]
pub fn new_build_date(base: Option<&str>, checked: CheckedTime) -> BuildDateOutcome {
    if let Some(text) = base
        && let Some(moment) = datetime::parse_rfc2822(text)
    {
        if moment.seconds > checked.seconds {
            return BuildDateOutcome::KeepLater {
                base: text.to_owned(),
            };
        }
        if moment.seconds == checked.seconds {
            return BuildDateOutcome::KeepEqual;
        }
    }
    BuildDateOutcome::Write(datetime::format_rfc2822(Moment {
        seconds: checked.seconds,
        offset_minutes: 0,
    }))
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Write};
    use std::net::{SocketAddr, TcpListener};
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::net::PublicRules;

    const YEAR: i64 = 365 * 86_400;
    const NOW: i64 = 1_793_000_000;

    /// A server that answers each request from a list of
    /// `(path, status line, extra headers)` and records each path.
    struct Probe {
        port: u16,
        seen: Arc<Mutex<Vec<String>>>,
    }

    impl Probe {
        fn start(routes: Vec<(&'static str, &'static str, String)>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            let seen = Arc::new(Mutex::new(Vec::new()));
            let shared = Arc::clone(&seen);
            std::thread::spawn(move || {
                for stream in listener.incoming() {
                    let Ok(mut stream) = stream else { continue };
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    let mut header = String::new();
                    while reader.read_line(&mut header).is_ok() && header.trim() != "" {
                        header.clear();
                    }
                    let mut parts = line.split_whitespace();
                    let method = parts.next().unwrap_or_default();
                    let path = parts.next().unwrap_or("/").to_owned();
                    shared.lock().unwrap().push(format!("{method} {path}"));
                    let (status, headers) = routes
                        .iter()
                        .find(|(p, _, _)| *p == path)
                        .map_or(("404 Not Found", String::new()), |(_, s, h)| (s, h.clone()));
                    let _ = write!(
                        stream,
                        "HTTP/1.1 {status}\r\n{headers}Content-Length: 0\r\nConnection: close\r\n\r\n"
                    );
                }
            });
            Self { port, seen }
        }

        fn url(&self, path: &str) -> String {
            format!("http://127.0.0.1:{}{path}", self.port)
        }

        fn seen(&self) -> Vec<String> {
            self.seen.lock().unwrap().clone()
        }

        fn source(&self) -> WebServerTime {
            let exception = SocketAddr::from(([127, 0, 0, 1], self.port));
            let policy = NetPolicy::Public(PublicRules::with_test_exceptions(vec![exception]));
            WebServerTime::with_web(Web::with_policy(&policy))
        }
    }

    const GMT: &str = "Date: Sun, 06 Nov 1994 08:49:37 GMT\r\n";
    const GMT_SECONDS: i64 = 784_111_777;

    #[test]
    fn a_date_in_the_gmt_form_gives_the_seconds() {
        let moment = datetime::parse_rfc2822("Sun, 06 Nov 1994 08:49:37 GMT").unwrap();
        assert_eq!(moment.seconds, GMT_SECONDS);
    }

    #[test]
    fn the_web_source_reads_date_at_each_status_and_follows_no_redirect() {
        let probe = Probe::start(vec![
            ("/ok", "200 OK", GMT.to_owned()),
            ("/gone", "404 Not Found", GMT.to_owned()),
            (
                "/moved",
                "301 Moved Permanently",
                format!("{GMT}Location: /target\r\n"),
            ),
            ("/target", "200 OK", GMT.to_owned()),
            ("/nodate", "200 OK", String::new()),
            ("/bad", "200 OK", "Date: yesterday\r\n".to_owned()),
        ]);
        let source = probe.source();
        for path in ["/ok", "/gone", "/moved"] {
            assert_eq!(
                source.server_time(&probe.url(path)),
                Some(GMT_SECONDS),
                "{path}"
            );
        }
        assert_eq!(source.server_time(&probe.url("/nodate")), None);
        assert_eq!(source.server_time(&probe.url("/bad")), None);
        let seen = probe.seen();
        assert!(seen.iter().all(|r| r.starts_with("HEAD ")), "{seen:?}");
        assert!(!seen.iter().any(|r| r.contains("/target")), "{seen:?}");
    }

    #[test]
    fn the_web_source_gives_none_for_a_server_that_does_not_answer_or_a_bad_address() {
        let port = {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        };
        let exception = SocketAddr::from(([127, 0, 0, 1], port));
        let policy = NetPolicy::Public(PublicRules::with_test_exceptions(vec![exception]));
        let source = WebServerTime::with_web(Web::with_policy(&policy));
        assert_eq!(
            source.server_time(&format!("http://127.0.0.1:{port}/")),
            None
        );
        assert_eq!(source.server_time("ftp://example.com/a"), None);
        assert_eq!(source.server_time("not an address"), None);
    }

    #[test]
    fn the_public_policy_refuses_a_loopback_address_and_sends_no_request() {
        let probe = Probe::start(vec![("/ok", "200 OK", GMT.to_owned())]);
        let source = WebServerTime::public();
        assert_eq!(source.server_time(&probe.url("/ok")), None);
        assert_eq!(probe.seen(), Vec::<String>::new());
    }

    fn keeper_with(answers: &[(&str, i64)], now: i64) -> TimeKeeper {
        let mut keeper = TimeKeeper::new();
        for (url, seconds) in answers {
            keeper.record(url, *seconds, now);
        }
        keeper
    }

    #[test]
    fn the_web_source_adds_the_age_of_a_cached_answer() {
        let date = "Sun, 06 Nov 1994 08:49:37 GMT";
        let probe = Probe::start(vec![
            (
                "/cached",
                "200 OK",
                format!("Date: {date}\r\nAge: 43200\r\n"),
            ),
            (
                "/bad-age",
                "200 OK",
                format!("Date: {date}\r\nAge: soon\r\n"),
            ),
        ]);
        let source = probe.source();
        assert_eq!(
            source.server_time(&probe.url("/cached")),
            Some(784_111_777 + 43_200)
        );
        assert_eq!(
            source.server_time(&probe.url("/bad-age")),
            Some(784_111_777)
        );
    }

    #[test]
    fn a_clock_two_years_ahead_gives_the_server_time_and_the_notice() {
        let computer = NOW + 2 * YEAR;
        let checked = keeper_with(&[("a", NOW)], computer)
            .checked(computer)
            .unwrap();
        assert_eq!(checked.seconds, NOW);
        assert_eq!(checked.computer_ahead_seconds, 2 * YEAR);
        assert!(checked.clock_notice());
    }

    #[test]
    fn a_server_two_years_ahead_gives_the_computer_time_and_no_notice() {
        let checked = keeper_with(&[("a", NOW + 2 * YEAR)], NOW)
            .checked(NOW)
            .unwrap();
        assert_eq!(checked.seconds, NOW);
        assert_eq!(checked.computer_ahead_seconds, 0);
        assert!(!checked.clock_notice());
    }

    #[test]
    fn with_two_servers_the_earliest_wins() {
        let checked = keeper_with(&[("a", NOW - 1000), ("b", NOW - 4000)], NOW)
            .checked(NOW)
            .unwrap();
        assert_eq!(checked.seconds, NOW - 4000);
    }

    #[test]
    fn with_no_answer_there_is_no_checked_time() {
        assert_eq!(TimeKeeper::new().checked(NOW), None);
        assert_eq!(ScriptedServerTime::empty().server_time("a"), None);
    }

    #[test]
    fn a_difference_is_current_for_600_seconds_and_then_the_url_is_listed_again() {
        let keeper = keeper_with(&[("a", NOW - 1000)], NOW);
        assert_eq!(keeper.missing(&["a", "b", "b"], NOW + 599), ["b"]);
        assert!(keeper.checked(NOW + 599).is_some());
        assert_eq!(keeper.missing(&["a"], NOW + 600), ["a"]);
        assert_eq!(keeper.checked(NOW + 600), None);
    }

    #[test]
    fn the_difference_moves_with_the_computer_clock() {
        let keeper = keeper_with(&[("a", NOW - 1000)], NOW);
        let checked = keeper.checked(NOW + 100).unwrap();
        assert_eq!(checked.seconds, NOW + 100 - 1000);
    }

    #[test]
    fn a_slow_clock_gives_the_computer_time_and_no_notice() {
        let checked = keeper_with(&[("a", NOW + 3600)], NOW).checked(NOW).unwrap();
        assert_eq!(checked.seconds, NOW);
        assert!(!checked.clock_notice());
    }

    #[test]
    fn the_notice_needs_more_than_300_seconds() {
        let at = |ahead| CheckedTime {
            seconds: NOW,
            computer_ahead_seconds: ahead,
        };
        assert!(!at(300).clock_notice());
        assert!(at(301).clock_notice());
    }

    fn checked(seconds: i64) -> CheckedTime {
        CheckedTime {
            seconds,
            computer_ahead_seconds: 0,
        }
    }

    #[test]
    fn the_new_build_date_follows_items_9_to_11() {
        // 2026-10-30 20:00:00 UTC.
        let at = 1_793_390_400;
        let text = "Fri, 30 Oct 2026 20:00:00 +0000";
        assert_eq!(
            new_build_date(None, checked(at)),
            BuildDateOutcome::Write(text.to_owned())
        );
        assert_eq!(
            new_build_date(Some("Fri, 30 Oct 2026 19:00:00 +0000"), checked(at)),
            BuildDateOutcome::Write(text.to_owned())
        );
        assert_eq!(
            new_build_date(Some("Fri, 30 Oct 2026 21:00:00 +0100"), checked(at)),
            BuildDateOutcome::KeepEqual
        );
        let later = "Sat, 31 Oct 2026 20:00:00 +0000";
        assert_eq!(
            new_build_date(Some(later), checked(at)),
            BuildDateOutcome::KeepLater {
                base: later.to_owned()
            }
        );
        assert_eq!(
            new_build_date(Some("yesterday"), checked(at)),
            BuildDateOutcome::Write(text.to_owned())
        );
    }
}

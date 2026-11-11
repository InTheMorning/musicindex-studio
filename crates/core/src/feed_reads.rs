//! The reads of the feed check (ADR 0174 items 4 and 13 to 16).
//!
//! The module gathers the network facts of a feed and gives them as data.
//! It reads through [`FeedReader`] and [`FileReads`], so that a test sends
//! no request (ADR 0045). The real readers use the policy `Public`.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::Serialize;

use crate::discover::ReadFailure;
use crate::feed::Feed;
use crate::feed_check::{self, Facts, Finding};
use crate::move_link::{FeedBody, FeedReader, HopResult, OwnedDraftLinks, tag_chain};
use crate::net::NetPolicy;
use crate::web::Web;

/// The number of bytes that two files must share to count as the same
/// file (ADR 0174 item 14).
pub const SAME_FILE_BYTES: u64 = 64 * 1024;

/// The most reads that run at the same time.
const WORKERS: usize = 8;

/// The first bytes of a file and its total size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probe {
    /// The total size of the file in bytes.
    pub total: u64,
    /// The first bytes of the file.
    pub first: Vec<u8>,
}

/// The reads of files that the check sends (ADR 0174 items 14 and 15).
pub trait FileReads: Send + Sync {
    /// Ask the host of the file at `url` for its size with a `HEAD`
    /// request (ADR 0023 item 5). Gives none when the host gives no size.
    ///
    /// # Errors
    ///
    /// Gives the text of the failure when the read fails.
    fn size(&self, url: &str) -> Result<Option<u64>, String>;

    /// Read the first [`SAME_FILE_BYTES`] bytes of the file at `url` with a
    /// range request. Gives none when the server does not answer a range
    /// request with 206.
    ///
    /// # Errors
    ///
    /// Gives the text of the failure when the read fails.
    fn probe(&self, url: &str) -> Result<Option<Probe>, String>;
}

/// The real [`FileReads`]. It uses the policy `Public`, in both shells.
#[derive(Debug, Clone)]
pub struct WebFileReads {
    web: Web,
}

impl WebFileReads {
    /// A reader with the policy `Public`.
    #[must_use]
    pub fn public() -> Self {
        Self {
            web: Web::with_policy(&NetPolicy::public()),
        }
    }

    /// A reader with a client that the caller makes, for a test server.
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn with_web(web: Web) -> Self {
        Self { web }
    }
}

impl FileReads for WebFileReads {
    fn size(&self, url: &str) -> Result<Option<u64>, String> {
        self.web
            .media_facts(url)
            .map(|facts| facts.length)
            .map_err(|error| error.to_string())
    }

    fn probe(&self, url: &str) -> Result<Option<Probe>, String> {
        self.web
            .range_probe(url, SAME_FILE_BYTES)
            .map(|found| {
                found.map(|found| Probe {
                    total: found.total,
                    first: found.first,
                })
            })
            .map_err(|error| error.to_string())
    }
}

/// The network facts of a feed (ADR 0174 item 4).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkFacts {
    /// The links that [`crate::move_link::check`] refused in this read.
    pub refused_links: Vec<String>,
    /// See [`Facts::content_lengths`].
    pub content_lengths: HashMap<String, u64>,
    /// See [`Facts::same_https`].
    pub same_https: HashSet<String>,
    /// See [`Facts::move_chain`].
    pub move_chain: Option<Vec<String>>,
    /// The addresses that Studio could not read (ADR 0174 item 16).
    pub unreadable: Vec<String>,
}

/// The `https` address of an `http` address, or none.
#[must_use]
pub fn https_of(url: &str) -> Option<String> {
    url.strip_prefix("http://")
        .map(|rest| format!("https://{rest}"))
}

/// The facts for the check: the local facts of `links`, and the last
/// network facts `net` when a read ran. A fact about an address that the
/// draft no longer names has no effect.
#[must_use]
pub fn facts_for(links: &OwnedDraftLinks, net: Option<&NetworkFacts>) -> Facts {
    let mut refused_links = net.map(|n| n.refused_links.clone()).unwrap_or_default();
    // ADR 0126 condition 6 needs no read.
    if links.self_changed
        && let (Some(self_url), Some(url)) = (links.feed.self_url(), links.url.as_deref())
        && self_url != url
    {
        refused_links.push(self_url.to_owned());
    }
    let tag = links.feed.new_feed_url.as_deref();
    Facts {
        public_address: links.url.clone(),
        refused_links,
        self_changed: links.self_changed,
        content_lengths: net.map(|n| n.content_lengths.clone()).unwrap_or_default(),
        same_https: net.map(|n| n.same_https.clone()).unwrap_or_default(),
        move_chain: net
            .and_then(|n| n.move_chain.clone())
            .filter(|chain| chain.first().map(String::as_str) == tag),
        index: None,
    }
}

/// The result of the check: the findings, and the addresses that Studio
/// could not read.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct CheckResult {
    /// The findings.
    pub findings: Vec<Finding>,
    /// The addresses that Studio could not read.
    pub unreadable: Vec<String>,
}

/// Check the draft in `links` with the last network facts `net`. The
/// function sends no request.
#[must_use]
pub fn check_with(links: &OwnedDraftLinks, net: Option<&NetworkFacts>) -> CheckResult {
    CheckResult {
        findings: feed_check::check(&links.feed, &facts_for(links, net)),
        unreadable: net.map(|n| n.unreadable.clone()).unwrap_or_default(),
    }
}

/// A [`FeedReader`] that keeps each answer, so that one gather reads an
/// address once.
struct Remembering<'a> {
    inner: &'a dyn FeedReader,
    kept: Mutex<HashMap<String, Result<FeedBody, ReadFailure>>>,
}

impl FeedReader for Remembering<'_> {
    fn read(&self, url: &str) -> Result<FeedBody, ReadFailure> {
        if let Some(answer) = self
            .kept
            .lock()
            .expect("the kept answers are not poisoned")
            .get(url)
        {
            return answer.clone();
        }
        let answer = self.inner.read(url);
        self.kept
            .lock()
            .expect("the kept answers are not poisoned")
            .insert(url.to_owned(), answer.clone());
        answer
    }

    fn wait_between_reads(&self) {
        self.inner.wait_between_reads();
    }
}

/// One read of the gather.
enum Job {
    /// The size of an enclosure.
    Size(String),
    /// The `http` address, and its `https` address.
    SameFile(String),
}

/// What a job found.
enum Found {
    Size(String, Result<Option<u64>, String>),
    Same {
        url: String,
        same: bool,
        failed: Option<String>,
    },
}

/// Gather the network facts of the draft in `links`. Each failed read gives
/// no fact and one address in [`NetworkFacts::unreadable`].
#[must_use]
pub fn gather(
    links: &OwnedDraftLinks,
    feeds: &dyn FeedReader,
    files: &dyn FileReads,
) -> NetworkFacts {
    let mut net = NetworkFacts::default();
    let reader = Remembering {
        inner: feeds,
        kept: Mutex::default(),
    };
    gather_links(links, &reader, &mut net);
    let found = run_jobs(&jobs_of(&links.feed), files);
    for result in found {
        match result {
            Found::Size(url, Ok(Some(length))) => {
                net.content_lengths.insert(url, length);
            }
            Found::Size(_, Ok(None)) => {}
            Found::Size(url, Err(_)) => net.unreadable.push(url),
            Found::Same { url, same, failed } => {
                if same {
                    net.same_https.insert(url);
                }
                net.unreadable.extend(failed);
            }
        }
    }
    net.unreadable.sort();
    net.unreadable.dedup();
    net.refused_links.sort();
    net.refused_links.dedup();
    net
}

/// The refusals and the chain of the tag (ADR 0174 items 5 and 13).
fn gather_links(links: &OwnedDraftLinks, reader: &dyn FeedReader, net: &mut NetworkFacts) {
    let report = links.report(reader);
    if let Some(refusal) = &report.refusal {
        let count = match refusal.condition {
            crate::move_link::Condition::ConflictingNames => 2,
            _ => 1,
        };
        net.refused_links
            .extend(refusal.hops.iter().take(count).map(|hop| hop.url.clone()));
        push_failed_read(&refusal.hops, net);
        return;
    }
    let Some(tag) = links.feed.new_feed_url.as_deref() else {
        return;
    };
    if Some(tag) == links.url.as_deref() {
        return;
    }
    let chain = tag_chain(
        reader,
        links.feed.podcast_guid.as_deref(),
        tag,
        links.url.as_deref(),
    );
    if chain.stopped.is_none() {
        net.move_chain = Some(chain.urls());
    } else {
        net.refused_links.push(tag.to_owned());
        push_failed_read(&chain.hops, net);
    }
}

/// A chain that stops at a host that did not answer gives a failed read
/// (ADR 0174 item 16).
fn push_failed_read(hops: &[crate::move_link::Hop], net: &mut NetworkFacts) {
    if let Some(last) = hops.last()
        && matches!(
            last.result,
            HopResult::Unreachable | HopResult::ServerError | HopResult::Gone | HopResult::TooLarge
        )
    {
        net.unreadable.push(last.url.clone());
    }
}

/// The reads that the feed needs. An address gets one read.
fn jobs_of(feed: &Feed) -> Vec<Job> {
    let mut jobs = Vec::new();
    let mut seen = HashSet::new();
    for entry in &feed.items {
        let Some(enclosure) = &entry.value.enclosure else {
            continue;
        };
        let Some(url) = enclosure.url.as_deref() else {
            continue;
        };
        let size = enclosure
            .length
            .as_deref()
            .and_then(|l| l.trim().parse::<u64>().ok())
            .unwrap_or(0);
        // The check compares a host size only with a size that the feed
        // gives.
        if size > 0 && seen.insert(("size", url.to_owned())) {
            jobs.push(Job::Size(url.to_owned()));
        }
        if url.starts_with("http://") && seen.insert(("same", url.to_owned())) {
            jobs.push(Job::SameFile(url.to_owned()));
        }
    }
    let rss = feed.rss_image.as_ref().and_then(|i| i.url.as_deref());
    for url in [feed.image.as_deref(), rss].into_iter().flatten() {
        if url.starts_with("http://") && seen.insert(("same", url.to_owned())) {
            jobs.push(Job::SameFile(url.to_owned()));
        }
    }
    jobs
}

/// Run the jobs with at most [`WORKERS`] reads at the same time. The
/// results keep the sequence of the jobs.
fn run_jobs(jobs: &[Job], files: &dyn FileReads) -> Vec<Found> {
    let next = AtomicUsize::new(0);
    let done: Mutex<Vec<(usize, Found)>> = Mutex::default();
    std::thread::scope(|scope| {
        for _ in 0..WORKERS.min(jobs.len()) {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::SeqCst);
                    let Some(job) = jobs.get(index) else { break };
                    let found = match job {
                        Job::Size(url) => Found::Size(url.clone(), files.size(url)),
                        Job::SameFile(url) => same_file(url, files),
                    };
                    done.lock()
                        .expect("the results are not poisoned")
                        .push((index, found));
                }
            });
        }
    });
    let mut done = done.into_inner().expect("the results are not poisoned");
    done.sort_by_key(|(index, _)| *index);
    done.into_iter().map(|(_, found)| found).collect()
}

/// Compare an `http` file with its `https` file (ADR 0174 item 14). The
/// files are the same when the sizes are equal and the first 64 KiB are
/// equal. A server that does not answer a range request gives no fact.
fn same_file(url: &str, files: &dyn FileReads) -> Found {
    let mut result = Found::Same {
        url: url.to_owned(),
        same: false,
        failed: None,
    };
    let Some(secure) = https_of(url) else {
        return result;
    };
    let (http, https) = match files.probe(url) {
        Ok(Some(http)) => match files.probe(&secure) {
            Ok(Some(https)) => (http, https),
            Ok(None) => return result,
            Err(_) => {
                set_failed(&mut result, secure);
                return result;
            }
        },
        Ok(None) => return result,
        Err(_) => {
            set_failed(&mut result, url.to_owned());
            return result;
        }
    };
    if let Found::Same { same, .. } = &mut result {
        *same = http == https;
    }
    result
}

fn set_failed(result: &mut Found, address: String) {
    if let Found::Same { failed, .. } = result {
        *failed = Some(address);
    }
}

/// A [`FileReads`] for a test. It serves the facts that the test gives and
/// records each read. An address with no script gives a failed read.
#[cfg(any(test, feature = "testing"))]
#[derive(Debug, Default)]
pub struct ScriptedFileReads {
    sizes: Mutex<HashMap<String, Result<Option<u64>, String>>>,
    probes: Mutex<HashMap<String, Result<Option<Probe>, String>>>,
    reads: Mutex<Vec<String>>,
}

#[cfg(any(test, feature = "testing"))]
impl ScriptedFileReads {
    /// A reader with no script.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Serve the size `length` at `url`.
    ///
    /// # Panics
    ///
    /// Panics when another thread poisoned the lock of the script.
    #[must_use]
    pub fn with_size(self, url: &str, length: u64) -> Self {
        self.sizes
            .lock()
            .expect("the script is not poisoned")
            .insert(url.to_owned(), Ok(Some(length)));
        self
    }

    /// Serve a file at `url` that answers a range request.
    ///
    /// # Panics
    ///
    /// Panics when another thread poisoned the lock of the script.
    #[must_use]
    pub fn with_file(self, url: &str, total: u64, first: &[u8]) -> Self {
        self.probes
            .lock()
            .expect("the script is not poisoned")
            .insert(
                url.to_owned(),
                Ok(Some(Probe {
                    total,
                    first: first.to_vec(),
                })),
            );
        self
    }

    /// Serve a file at `url` that does not answer a range request.
    ///
    /// # Panics
    ///
    /// Panics when another thread poisoned the lock of the script.
    #[must_use]
    pub fn with_no_range(self, url: &str) -> Self {
        self.probes
            .lock()
            .expect("the script is not poisoned")
            .insert(url.to_owned(), Ok(None));
        self
    }

    /// The address of each read so far.
    ///
    /// # Panics
    ///
    /// Panics when another thread poisoned the lock of the reads.
    #[must_use]
    pub fn reads(&self) -> Vec<String> {
        self.reads
            .lock()
            .expect("the reads are not poisoned")
            .clone()
    }

    fn record(&self, url: &str) {
        self.reads
            .lock()
            .expect("the reads are not poisoned")
            .push(url.to_owned());
    }
}

#[cfg(any(test, feature = "testing"))]
impl FileReads for ScriptedFileReads {
    fn size(&self, url: &str) -> Result<Option<u64>, String> {
        self.record(url);
        self.sizes
            .lock()
            .expect("the script is not poisoned")
            .get(url)
            .cloned()
            .unwrap_or_else(|| Err(format!("the test reader has no size for {url}")))
    }

    fn probe(&self, url: &str) -> Result<Option<Probe>, String> {
        self.record(url);
        self.probes
            .lock()
            .expect("the script is not poisoned")
            .get(url)
            .cloned()
            .unwrap_or_else(|| Err(format!("the test reader has no file for {url}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::FailKind;
    use crate::feed::{Enclosure, Entry, Item};
    use crate::move_link::ScriptedReader;

    const URL: &str = "https://music.example/feed.xml";
    const GUID: &str = "917393e3-1b1e-5cef-ace4-edaa54e1f810";
    const NEW: &str = "https://new.example/feed.xml";
    const THIRD: &str = "https://third.example/feed.xml";

    fn links(feed: Feed) -> OwnedDraftLinks {
        OwnedDraftLinks {
            feed,
            url: Some(URL.to_owned()),
            self_changed: false,
            destination: None,
        }
    }

    fn feed_with_track(url: &str, length: Option<&str>) -> Feed {
        let mut feed = Feed {
            podcast_guid: Some(GUID.to_owned()),
            ..Feed::default()
        };
        feed.items.push(Entry::new(Item {
            guid: Some("one".to_owned()),
            enclosure: Some(Enclosure {
                url: Some(url.to_owned()),
                length: length.map(str::to_owned),
                media_type: None,
            }),
            ..Item::default()
        }));
        feed
    }

    #[test]
    fn the_chain_of_the_tag_ends_at_a_third_address() {
        let mut feed = Feed {
            podcast_guid: Some(GUID.to_owned()),
            ..Feed::default()
        };
        feed.new_feed_url = Some(NEW.to_owned());
        let feeds = ScriptedReader::empty()
            .with_feed(NEW, NEW, Some(GUID), Some(THIRD), None)
            .with_feed(THIRD, THIRD, Some(GUID), None, Some(THIRD));
        let net = gather(&links(feed.clone()), &feeds, &ScriptedFileReads::empty());
        assert_eq!(net.move_chain, Some(vec![NEW.to_owned(), THIRD.to_owned()]));
        assert!(net.refused_links.is_empty() && net.unreadable.is_empty());
        let found = feed_check::check(&feed, &facts_for(&links(feed.clone()), Some(&net)));
        assert!(
            found
                .iter()
                .any(|f| f.kind == feed_check::FindingKind::MoveNotConfirmed)
        );
    }

    #[test]
    fn a_chain_that_ends_at_the_tag_gives_no_finding_and_a_stale_chain_has_no_effect() {
        let mut feed = Feed {
            podcast_guid: Some(GUID.to_owned()),
            ..Feed::default()
        };
        feed.new_feed_url = Some(NEW.to_owned());
        let feeds = ScriptedReader::empty().with_feed(NEW, NEW, Some(GUID), None, Some(NEW));
        let net = gather(&links(feed.clone()), &feeds, &ScriptedFileReads::empty());
        assert_eq!(net.move_chain, Some(vec![NEW.to_owned()]));
        // The tag changes: the chain is not a fact about the new tag.
        feed.new_feed_url = Some(THIRD.to_owned());
        assert_eq!(facts_for(&links(feed), Some(&net)).move_chain, None);
    }

    #[test]
    fn a_refused_chain_gives_a_refused_link_and_no_chain() {
        let mut feed = Feed {
            podcast_guid: Some(GUID.to_owned()),
            ..Feed::default()
        };
        feed.new_feed_url = Some(NEW.to_owned());
        let feeds =
            ScriptedReader::empty().with_feed(NEW, NEW, Some("other-guid"), None, Some(NEW));
        let net = gather(&links(feed), &feeds, &ScriptedFileReads::empty());
        assert_eq!(net.move_chain, None);
        assert_eq!(net.refused_links, [NEW]);
        assert_eq!(net.unreadable, Vec::<String>::new());
        // A tag that does not answer is refused and also unreadable.
        let mut feed = Feed {
            podcast_guid: Some(GUID.to_owned()),
            ..Feed::default()
        };
        feed.new_feed_url = Some(NEW.to_owned());
        let feeds = ScriptedReader::empty().with_failure(NEW, FailKind::Unreachable);
        let net = gather(&links(feed), &feeds, &ScriptedFileReads::empty());
        assert_eq!(net.refused_links, [NEW]);
        assert_eq!(net.unreadable, [NEW]);
    }

    #[test]
    fn an_http_file_with_the_same_size_and_bytes_as_its_https_file_is_the_same_file() {
        let http = "http://files.example/one.mp3";
        let https = "https://files.example/one.mp3";
        let feed = feed_with_track(http, Some("500"));
        let files = ScriptedFileReads::empty()
            .with_size(http, 500)
            .with_file(http, 500, b"abc")
            .with_file(https, 500, b"abc");
        let net = gather(&links(feed.clone()), &ScriptedReader::empty(), &files);
        assert!(net.same_https.contains(http));
        assert_eq!(net.content_lengths.get(http), Some(&500));
        // A different size or different bytes is not the same file.
        for other in [(501, &b"abc"[..]), (500, &b"abd"[..])] {
            let files = ScriptedFileReads::empty()
                .with_size(http, 500)
                .with_file(http, 500, b"abc")
                .with_file(https, other.0, other.1);
            let net = gather(&links(feed.clone()), &ScriptedReader::empty(), &files);
            assert!(net.same_https.is_empty(), "{other:?}");
        }
    }

    #[test]
    fn a_server_that_gives_no_range_answer_gives_no_fact_and_a_failed_read_is_unreadable() {
        let http = "http://files.example/one.mp3";
        let https = "https://files.example/one.mp3";
        let feed = feed_with_track(http, None);
        let files = ScriptedFileReads::empty()
            .with_file(http, 500, b"abc")
            .with_no_range(https);
        let net = gather(&links(feed.clone()), &ScriptedReader::empty(), &files);
        assert!(net.same_https.is_empty() && net.unreadable.is_empty());
        // The https address fails: one entry in the list.
        let files = ScriptedFileReads::empty().with_file(http, 500, b"abc");
        let net = gather(&links(feed.clone()), &ScriptedReader::empty(), &files);
        assert!(net.same_https.is_empty());
        assert_eq!(net.unreadable, [https]);
        // The http address fails: the gather does not read the https address.
        let files = ScriptedFileReads::empty();
        let net = gather(&links(feed), &ScriptedReader::empty(), &files);
        assert_eq!(net.unreadable, [http]);
        assert_eq!(files.reads(), [http]);
    }

    #[test]
    fn a_size_is_read_only_for_an_enclosure_with_a_size_and_a_failed_read_is_unreadable() {
        let url = "https://files.example/one.mp3";
        let files = ScriptedFileReads::empty();
        let net = gather(
            &links(feed_with_track(url, None)),
            &ScriptedReader::empty(),
            &files,
        );
        assert!(files.reads().is_empty() && net.unreadable.is_empty());
        let net = gather(
            &links(feed_with_track(url, Some("500"))),
            &ScriptedReader::empty(),
            &files,
        );
        assert_eq!(net.unreadable, [url]);
        assert!(net.content_lengths.is_empty());
    }

    #[test]
    fn the_image_of_the_feed_is_compared_with_its_https_address() {
        let http = "http://files.example/cover.jpg";
        let https = "https://files.example/cover.jpg";
        let feed = Feed {
            image: Some(http.to_owned()),
            ..Feed::default()
        };
        let files = ScriptedFileReads::empty()
            .with_file(http, 9, b"x")
            .with_file(https, 9, b"x");
        let net = gather(&links(feed), &ScriptedReader::empty(), &files);
        assert!(net.same_https.contains(http));
    }
}

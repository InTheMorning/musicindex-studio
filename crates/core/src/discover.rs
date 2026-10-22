//! Find the feeds on a host (ADR 0019).

use std::collections::{HashSet, VecDeque};

use serde::Serialize;

use crate::connector::{Connector, ConnectorError, Version};
use crate::digest::content_version;
use crate::feed::{Feed, RemoteItem};
use crate::musicindex::MusicIndex;
use crate::web::{Web, host_of};
use crate::xml::Document;

/// The largest file that discovery reads, in bytes (ADR 0019).
pub const MAX_FEED_BYTES: usize = 5 * 1024 * 1024;

/// A feed file on a host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Candidate {
    /// The path relative to the connector root.
    pub path: String,
    /// The channel title.
    pub title: Option<String>,
    /// `podcast:medium`.
    pub medium: Option<String>,
    /// `podcast:guid`.
    pub podcast_guid: Option<String>,
}

/// The feed files on a host.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Candidates {
    /// The feed files, in path sequence.
    pub feeds: Vec<Candidate>,
    /// True when a limit stopped the file list before the end.
    pub truncated: bool,
}

/// List the RSS feed files on a host. A file that is not an RSS feed, or
/// that is larger than [`MAX_FEED_BYTES`], is not a candidate.
///
/// # Errors
///
/// Gives a [`ConnectorError`] when the file list fails. A file that cannot
/// be read is not a candidate.
pub fn feed_candidates(connector: &dyn Connector) -> Result<Candidates, ConnectorError> {
    let list = connector.list()?;
    let feeds = list
        .paths
        .iter()
        .filter(|p| p.to_ascii_lowercase().ends_with(".xml"))
        .filter_map(|path| {
            let file = connector.read(path).ok()??;
            if file.bytes.len() > MAX_FEED_BYTES {
                return None;
            }
            let feed = Feed::read(&Document::parse_bytes(&file.bytes).ok()?).ok()?;
            Some(Candidate {
                path: path.clone(),
                title: feed.title,
                medium: feed.medium,
                podcast_guid: feed.podcast_guid,
            })
        })
        .collect();
    Ok(Candidates {
        feeds,
        truncated: list.truncated,
    })
}

/// The maximum number of feeds that one discovery reads (ADR 0020).
pub const MAX_DISCOVERED_FEEDS: usize = 100;

/// Hosting services that Studio knows by host name, and cannot write to
/// (ADR 0020). A connector for a service removes it from this list.
const KNOWN_SERVICES: &[(&str, &str)] = &[
    ("wavlake.com", "Wavlake"),
    ("feeds.fountain.fm", "Fountain"),
    ("serve.podhome.fm", "Podhome"),
    ("musicsideproject.com", "MSP 2.0"),
    ("msp.podtards.com", "MSP 2.0"),
];

/// The name of a known hosting service for a host name.
#[must_use]
pub fn known_service(host: &str) -> Option<&'static str> {
    let host = host.trim_start_matches("www.");
    KNOWN_SERVICES
        .iter()
        .find(|(name, _)| *name == host)
        .map(|(_, service)| *service)
}

/// How discovery found a feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Via {
    /// The user gave its address or GUID.
    Given,
    /// It is the publisher feed of a given feed.
    Publisher,
    /// A publisher feed lists it.
    Release,
}

/// A feed that discovery found and read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Found {
    /// The address that Studio read.
    pub url: String,
    /// The channel title.
    pub title: Option<String>,
    /// `podcast:medium`.
    pub medium: Option<String>,
    /// `podcast:guid`.
    pub podcast_guid: Option<String>,
    /// How discovery found the feed.
    pub via: Via,
    /// The host name of the address.
    pub host: Option<String>,
    /// The known hosting service of the host, if there is one.
    pub service: Option<&'static str>,
    /// The typed model of the feed, for the collection.
    #[serde(skip)]
    pub feed: Feed,
    /// The version marker of the content that Studio read.
    #[serde(skip)]
    pub version: Version,
}

/// A source that discovery could not read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Failed {
    /// The address or GUID.
    pub source: String,
    /// Why the read failed.
    pub message: String,
}

/// The result of a discovery from addresses.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Discovery {
    /// The feeds, in the sequence that discovery found them.
    pub feeds: Vec<Found>,
    /// The sources that discovery could not read.
    pub failed: Vec<Failed>,
    /// True when [`MAX_DISCOVERED_FEEDS`] stopped the discovery.
    pub truncated: bool,
}

#[derive(Clone)]
enum Source {
    Url(String),
    Guid(String),
}

impl Source {
    fn label(&self) -> &str {
        match self {
            Source::Url(text) | Source::Guid(text) => text,
        }
    }

    fn from_remote(item: &crate::feed::Entry<RemoteItem>) -> Option<Self> {
        match (&item.feed_url, &item.feed_guid) {
            (Some(url), _) if !url.trim().is_empty() => Some(Source::Url(url.trim().to_owned())),
            (_, Some(guid)) if !guid.trim().is_empty() => {
                Some(Source::Guid(guid.trim().to_owned()))
            }
            _ => None,
        }
    }
}

/// Read one public feed by its address.
///
/// # Errors
///
/// Gives a message when the read fails or the file is not an RSS feed.
pub fn read_public(web: &Web, url: &str) -> Result<(Feed, Version), String> {
    let limit = u64::try_from(MAX_FEED_BYTES).unwrap_or(u64::MAX);
    let file = web.get(url, limit).map_err(|e| e.to_string())?;
    let doc = Document::parse_bytes(&file.bytes).map_err(|e| e.to_string())?;
    let feed = Feed::read(&doc).map_err(|e| e.to_string())?;
    Ok((feed, content_version(&file.bytes)))
}

/// The address of a source. A GUID gets its address from musicindex.
fn resolve(source: &Source, index: &MusicIndex) -> Result<String, Failed> {
    match source {
        Source::Url(url) => Ok(url.clone()),
        Source::Guid(guid) => match index.feed_url(guid) {
            Ok(Some(url)) => Ok(url),
            Ok(None) => Err(Failed {
                source: guid.clone(),
                message: "musicindex does not know this feed GUID".to_owned(),
            }),
            Err(e) => Err(Failed {
                source: guid.clone(),
                message: e.to_string(),
            }),
        },
    }
}

/// The links that discovery follows from a feed, by how it found the feed
/// (ADR 0020 items 3 and 5).
fn links(feed: &Feed, via: Via) -> Vec<(Source, Via)> {
    let releases = || {
        feed.remote_items
            .iter()
            .filter_map(Source::from_remote)
            .map(|s| (s, Via::Release))
            .collect()
    };
    let is_publisher = feed
        .medium
        .as_deref()
        .is_some_and(|m| m.eq_ignore_ascii_case("publisher"));
    match via {
        Via::Given if is_publisher => releases(),
        Via::Given => feed
            .publisher
            .iter()
            .flat_map(|p| p.remote_items.iter())
            .filter_map(Source::from_remote)
            .map(|s| (s, Via::Publisher))
            .collect(),
        Via::Publisher => releases(),
        Via::Release => Vec::new(),
    }
}

/// Find feeds from the addresses and GUIDs that the user gave (ADR 0020).
///
/// Discovery reads each given feed, and follows its links one level: the
/// releases of a publisher feed, and the publisher feed of a release with
/// the other releases of that publisher. It follows no link from a feed
/// that it found through a link, and it reads a maximum of
/// [`MAX_DISCOVERED_FEEDS`] feeds.
#[must_use]
pub fn from_sources(
    web: &Web,
    index: &MusicIndex,
    addresses: &[String],
    guids: &[String],
) -> Discovery {
    // Each queue entry: the source, how it was found, and which links to follow.
    let mut queue: VecDeque<(Source, Via)> = addresses
        .iter()
        .map(|a| a.trim())
        .filter(|a| !a.is_empty())
        .map(|a| (Source::Url(a.to_owned()), Via::Given))
        .chain(
            guids
                .iter()
                .map(|g| g.trim())
                .filter(|g| !g.is_empty())
                .map(|g| (Source::Guid(g.to_owned()), Via::Given)),
        )
        .collect();
    let mut result = Discovery::default();
    let mut seen_urls: HashSet<String> = HashSet::new();
    let mut seen_guids: HashSet<String> = HashSet::new();
    while let Some((source, via)) = queue.pop_front() {
        // A GUID that a feed already gave needs no second read.
        if let Source::Guid(guid) = &source
            && seen_guids.contains(&guid.to_ascii_lowercase())
        {
            continue;
        }
        let url = match resolve(&source, index) {
            Ok(url) => url,
            Err(failed) => {
                result.failed.push(failed);
                continue;
            }
        };
        if !seen_urls.insert(url.clone()) {
            continue;
        }
        if result.feeds.len() == MAX_DISCOVERED_FEEDS {
            result.truncated = true;
            break;
        }
        let (feed, version) = match read_public(web, &url) {
            Ok(read) => read,
            Err(message) => {
                result.failed.push(Failed {
                    source: source.label().to_owned(),
                    message,
                });
                continue;
            }
        };
        if let Some(guid) = &feed.podcast_guid {
            let new = seen_guids.insert(guid.to_ascii_lowercase());
            if !new && via != Via::Given {
                continue;
            }
        }
        queue.extend(links(&feed, via));
        let host = host_of(&url);
        result.feeds.push(Found {
            service: host.as_deref().and_then(known_service),
            host,
            url,
            title: feed.title.clone(),
            medium: feed.medium.clone(),
            podcast_guid: feed.podcast_guid.clone(),
            via,
            feed,
            version,
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::{Expected, LocalFolder};
    use crate::testing::TestServer;

    fn rss(medium: &str, guid: &str, title: &str, extra: &str) -> Vec<u8> {
        format!(
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel><title>{title}</title>\
             <podcast:guid>{guid}</podcast:guid><podcast:medium>{medium}</podcast:medium>{extra}</channel></rss>"
        )
        .into_bytes()
    }

    /// A publisher with two releases. Release A names the publisher. Release
    /// B is known to musicindex only by GUID. Release A also names a second
    /// publisher, whose releases discovery must not follow from A.
    fn server() -> TestServer {
        let server = TestServer::start(Vec::new());
        let publisher = server.url("/publisher.xml");
        let a = server.url("/a.xml");
        server.set(
            "/publisher.xml",
            200,
            rss(
                "publisher",
                "p",
                "Label",
                &format!(
                    "<podcast:remoteItem medium=\"music\" feedGuid=\"a\" feedUrl=\"{a}\"/>\
                 <podcast:remoteItem medium=\"music\" feedGuid=\"b\"/>"
                ),
            ),
        );
        server.set(
            "/a.xml",
            200,
            rss("music", "a", "Album A", &format!(
                "<podcast:publisher><podcast:remoteItem medium=\"publisher\" feedGuid=\"p\" feedUrl=\"{publisher}\"/></podcast:publisher>"
            )),
        );
        server.set("/b.xml", 200, rss("music", "b", "Album B", ""));
        server.set(
            "/v1/feeds/b",
            200,
            format!("{{\"data\":{{\"feed_url\":\"{}\"}}}}", server.url("/b.xml")).into_bytes(),
        );
        server
    }

    #[test]
    fn a_publisher_address_gives_the_publisher_and_its_releases() {
        let server = server();
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let found = from_sources(&Web::new(), &index, &[server.url("/publisher.xml")], &[]);
        let titles: Vec<_> = found
            .feeds
            .iter()
            .map(|f| (f.title.clone().unwrap(), f.via))
            .collect();
        assert_eq!(
            titles,
            [
                ("Label".to_owned(), Via::Given),
                ("Album A".to_owned(), Via::Release),
                ("Album B".to_owned(), Via::Release)
            ]
        );
        assert!(found.failed.is_empty());
    }

    #[test]
    fn a_release_address_gives_its_publisher_and_the_other_releases() {
        let server = server();
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let found = from_sources(&Web::new(), &index, &[server.url("/a.xml")], &[]);
        let titles: Vec<_> = found
            .feeds
            .iter()
            .map(|f| (f.title.clone().unwrap(), f.via))
            .collect();
        assert_eq!(
            titles,
            [
                ("Album A".to_owned(), Via::Given),
                ("Label".to_owned(), Via::Publisher),
                ("Album B".to_owned(), Via::Release)
            ]
        );
    }

    #[test]
    fn a_release_found_through_a_link_gives_no_more_links() {
        let server = server();
        // Album B names a second publisher. Discovery reaches B through a
        // link, so it must not read that publisher.
        let other = server.url("/other.xml");
        server.set(
            "/b.xml",
            200,
            rss("music", "b", "Album B", &format!(
                "<podcast:publisher><podcast:remoteItem medium=\"publisher\" feedGuid=\"o\" feedUrl=\"{other}\"/></podcast:publisher>"
            )),
        );
        server.set("/other.xml", 200, rss("publisher", "o", "Other", ""));
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let found = from_sources(&Web::new(), &index, &[server.url("/publisher.xml")], &[]);
        assert!(
            found
                .feeds
                .iter()
                .all(|f| f.title.as_deref() != Some("Other"))
        );
    }

    #[test]
    fn a_guid_resolves_through_musicindex_and_failures_are_reported() {
        let server = server();
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let found = from_sources(
            &Web::new(),
            &index,
            &[server.url("/missing.xml"), "ftp://x/y.xml".to_owned()],
            &["b".to_owned(), "unknown".to_owned()],
        );
        assert_eq!(found.feeds.len(), 1);
        assert_eq!(found.feeds[0].title.as_deref(), Some("Album B"));
        assert_eq!(found.failed.len(), 3);
    }

    #[test]
    fn discovery_stops_at_the_limit() {
        let server = TestServer::start(Vec::new());
        let items = (0..MAX_DISCOVERED_FEEDS + 5).fold(String::new(), |mut text, n| {
            use std::fmt::Write as _;
            let url = server.url(&format!("/r{n}.xml"));
            let _ = write!(
                text,
                "<podcast:remoteItem feedGuid=\"g{n}\" feedUrl=\"{url}\"/>"
            );
            text
        });
        server.set("/p.xml", 200, rss("publisher", "p", "Big", &items));
        for n in 0..MAX_DISCOVERED_FEEDS + 5 {
            let path: &'static str = Box::leak(format!("/r{n}.xml").into_boxed_str());
            server.set(path, 200, rss("music", &format!("g{n}"), "R", ""));
        }
        let index = MusicIndex::new(Web::new(), &server.url(""));
        let found = from_sources(&Web::new(), &index, &[server.url("/p.xml")], &[]);
        assert_eq!(found.feeds.len(), MAX_DISCOVERED_FEEDS);
        assert!(found.truncated);
    }

    #[test]
    fn known_services_are_found_by_host() {
        assert_eq!(known_service("wavlake.com"), Some("Wavlake"));
        assert_eq!(known_service("www.wavlake.com"), Some("Wavlake"));
        assert_eq!(known_service("feeds.fountain.fm"), Some("Fountain"));
        assert_eq!(known_service("serve.podhome.fm"), Some("Podhome"));
        assert_eq!(known_service("music.heycitizen.xyz"), None);
    }

    #[test]
    fn only_rss_feeds_are_candidates() {
        let dir = tempfile::tempdir().unwrap();
        let folder = LocalFolder::new(dir.path(), None);
        let feed = "<rss><channel><title>Cream</title></channel></rss>";
        folder
            .write("music/cream.xml", feed.as_bytes(), &Expected::Absent)
            .unwrap();
        folder
            .write("sitemap.xml", b"<urlset/>", &Expected::Absent)
            .unwrap();
        folder
            .write("broken.xml", b"<rss><channel>", &Expected::Absent)
            .unwrap();
        folder
            .write("cover.jpg", b"jpeg", &Expected::Absent)
            .unwrap();
        let candidates = feed_candidates(&folder).unwrap();
        assert_eq!(candidates.feeds.len(), 1);
        assert_eq!(candidates.feeds[0].path, "music/cream.xml");
        assert_eq!(candidates.feeds[0].title.as_deref(), Some("Cream"));
        assert!(!candidates.truncated);
    }
}

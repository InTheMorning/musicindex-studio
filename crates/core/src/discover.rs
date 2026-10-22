//! Find the feeds on a host (ADR 0019).

use serde::Serialize;

use crate::connector::{Connector, ConnectorError};
use crate::feed::Feed;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::{Expected, LocalFolder};

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

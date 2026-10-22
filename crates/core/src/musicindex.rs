//! The musicindex API (Stophammer): feed addresses by GUID and search.

use serde::{Deserialize, Serialize};

use crate::web::{Web, WebError, WebErrorKind};

/// The public musicindex API.
pub const DEFAULT_API: &str = "https://api.musicindex.org";

/// The largest musicindex response that Studio reads, in bytes.
const MAX_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;

/// A feed that a musicindex search found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchHit {
    /// `podcast:guid` of the feed.
    pub feed_guid: String,
    /// The feed title.
    pub title: Option<String>,
    /// The release artist that musicindex gives.
    pub artist: Option<String>,
    /// The address of the feed image.
    pub image_url: Option<String>,
}

#[derive(Deserialize)]
struct Envelope<T> {
    data: T,
}

#[derive(Deserialize)]
struct FeedData {
    feed_url: Option<String>,
}

#[derive(Deserialize)]
struct SearchItem {
    entity_type: String,
    entity_id: String,
    title: Option<String>,
    release_artist: Option<String>,
    feed_image_url: Option<String>,
}

/// A client of one musicindex API.
#[derive(Debug, Clone)]
pub struct MusicIndex {
    web: Web,
    base: String,
}

impl MusicIndex {
    /// Make a client for the API at `base`, such as [`DEFAULT_API`].
    #[must_use]
    pub fn new(web: Web, base: &str) -> Self {
        Self {
            web,
            base: base.trim_end_matches('/').to_owned(),
        }
    }

    /// The feed address that musicindex knows for a feed GUID. Gives `None`
    /// when musicindex does not know the feed.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request fails.
    pub fn feed_url(&self, feed_guid: &str) -> Result<Option<String>, WebError> {
        let guid: String = feed_guid
            .chars()
            .filter(|c| c.is_ascii_hexdigit() || *c == '-')
            .collect();
        let url = format!("{}/v1/feeds/{guid}", self.base);
        match self.web.get(&url, MAX_RESPONSE_BYTES) {
            Ok(file) => Ok(serde_json::from_slice::<Envelope<FeedData>>(&file.bytes)
                .ok()
                .and_then(|e| e.data.feed_url)),
            Err(e) if e.kind() == WebErrorKind::Status && e.status() == Some(404) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Search musicindex for feeds by name.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request fails.
    pub fn search(&self, query: &str) -> Result<Vec<SearchHit>, WebError> {
        let url = format!("{}/v1/search", self.base);
        let file = self.web.get_with_query(
            &url,
            &[("q", query), ("type", "feed"), ("limit", "25")],
            MAX_RESPONSE_BYTES,
        )?;
        let items = serde_json::from_slice::<Envelope<Vec<SearchItem>>>(&file.bytes)
            .map(|e| e.data)
            .unwrap_or_default();
        Ok(items
            .into_iter()
            .filter(|i| i.entity_type == "feed")
            .map(|i| SearchHit {
                feed_guid: i.entity_id,
                title: i.title,
                artist: i.release_artist,
                image_url: i.feed_image_url,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TestServer;

    #[test]
    fn a_guid_gives_its_feed_address_and_an_unknown_guid_gives_none() {
        let server = TestServer::start(vec![
            (
                "/v1/feeds/917393e3-1b1e-5cef-ace4-edaa54e1f810",
                200,
                br#"{"data":{"feed_url":"https://example.com/a.xml","title":"A"}}"#.to_vec(),
            ),
            (
                "/v1/search",
                200,
                br#"{"data":[{"entity_type":"feed","entity_id":"g1","title":"A","release_artist":"Ann"},{"entity_type":"track","entity_id":"t1"}]}"#.to_vec(),
            ),
        ]);
        let index = MusicIndex::new(Web::new(), &server.url(""));
        assert_eq!(
            index
                .feed_url("917393e3-1b1e-5cef-ace4-edaa54e1f810")
                .unwrap()
                .as_deref(),
            Some("https://example.com/a.xml")
        );
        assert_eq!(
            index
                .feed_url("00000000-0000-0000-0000-000000000000")
                .unwrap(),
            None
        );
        let hits = index.search("ann").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].feed_guid, "g1");
        assert_eq!(hits[0].artist.as_deref(), Some("Ann"));
    }
}

//! The musicindex API (Stophammer): feed addresses by GUID, search, and the
//! payment routes of a feed and of a track (ADR 0027 item 6).

use serde::{Deserialize, Serialize};

use crate::feed::Recipient;
use crate::web::{Web, WebError, WebErrorKind, path_segment};

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

/// A track that a feed lists, from musicindex.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndexTrack {
    /// The GUID of the track.
    pub track_guid: String,
    /// The title of the track.
    pub title: String,
}

/// The payment routes of a feed or of a track, from musicindex
/// (ADR 0027 item 6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IndexRoutes {
    /// The title of the feed or of the track.
    pub title: String,
    /// Each payment route, as a recipient.
    pub recipients: Vec<Recipient>,
    /// Each track of the feed. Empty for the routes of one track.
    pub tracks: Vec<IndexTrack>,
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

/// One `payment_routes` entry of a feed or a track response.
#[derive(Deserialize)]
struct RouteData {
    recipient_name: Option<String>,
    route_type: String,
    address: String,
    split: i64,
    custom_key: Option<String>,
    custom_value: Option<String>,
    fee: bool,
}

impl From<RouteData> for Recipient {
    fn from(route: RouteData) -> Self {
        Self {
            name: route.recipient_name,
            recipient_type: Some(route.route_type),
            address: Some(route.address),
            split: Some(route.split.to_string()),
            custom_key: route.custom_key,
            custom_value: route.custom_value,
            fee: route.fee.then(|| "true".to_owned()),
        }
    }
}

/// One `GET /v1/feeds/{guid}` response, with only the fields that
/// [`MusicIndex::feed_routes`] uses.
#[derive(Deserialize)]
struct FeedRoutesData {
    title: String,
    payment_routes: Option<Vec<RouteData>>,
    tracks: Option<Vec<TrackSummaryData>>,
}

#[derive(Deserialize)]
struct TrackSummaryData {
    track_guid: String,
    title: String,
}

/// One `GET /v1/feeds/{guid}/tracks/{track_guid}` response, with only the
/// fields that [`MusicIndex::track_routes`] uses.
#[derive(Deserialize)]
struct TrackRoutesData {
    title: String,
    payment_routes: Option<Vec<RouteData>>,
}

impl From<FeedRoutesData> for IndexRoutes {
    fn from(data: FeedRoutesData) -> Self {
        Self {
            title: data.title,
            recipients: data
                .payment_routes
                .unwrap_or_default()
                .into_iter()
                .map(Recipient::from)
                .collect(),
            tracks: data
                .tracks
                .unwrap_or_default()
                .into_iter()
                .map(|t| IndexTrack {
                    track_guid: t.track_guid,
                    title: t.title,
                })
                .collect(),
        }
    }
}

impl From<TrackRoutesData> for IndexRoutes {
    fn from(data: TrackRoutesData) -> Self {
        Self {
            title: data.title,
            recipients: data
                .payment_routes
                .unwrap_or_default()
                .into_iter()
                .map(Recipient::from)
                .collect(),
            tracks: Vec::new(),
        }
    }
}

/// Keep only the hex digits and the hyphens of a GUID, so that the request
/// path holds no other text.
fn filter_guid(guid: &str) -> String {
    guid.chars()
        .filter(|c| c.is_ascii_hexdigit() || *c == '-')
        .collect()
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
        let url = format!("{}/v1/feeds/{}", self.base, filter_guid(feed_guid));
        match self.web.get(&url, MAX_RESPONSE_BYTES) {
            Ok(file) => Ok(serde_json::from_slice::<Envelope<FeedData>>(&file.bytes)
                .ok()
                .and_then(|e| e.data.feed_url)),
            Err(e) if e.kind() == WebErrorKind::Status && e.status() == Some(404) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// The title, the payment routes and the tracks of a feed, from the
    /// `payment_routes` and `tracks` of the musicindex feed (ADR 0027
    /// item 6). Gives `None` when musicindex does not know the feed.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request fails.
    pub fn feed_routes(&self, feed_guid: &str) -> Result<Option<IndexRoutes>, WebError> {
        let url = format!("{}/v1/feeds/{}", self.base, filter_guid(feed_guid));
        match self.web.get_with_query(
            &url,
            &[("include", "payment_routes,tracks"), ("limit", "200")],
            MAX_RESPONSE_BYTES,
        ) {
            Ok(file) => Ok(
                serde_json::from_slice::<Envelope<FeedRoutesData>>(&file.bytes)
                    .ok()
                    .map(|e| e.data.into()),
            ),
            Err(e) if e.kind() == WebErrorKind::Status && e.status() == Some(404) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// The title and the payment routes of one track, from the
    /// `payment_routes` of the musicindex track (ADR 0027 item 6). The
    /// result has no tracks. Gives `None` when musicindex does not know the
    /// track.
    ///
    /// # Errors
    ///
    /// Gives a [`WebError`] when the request fails.
    pub fn track_routes(
        &self,
        feed_guid: &str,
        track_guid: &str,
    ) -> Result<Option<IndexRoutes>, WebError> {
        let url = format!(
            "{}/v1/feeds/{}/tracks/{}",
            self.base,
            filter_guid(feed_guid),
            path_segment(track_guid)
        );
        match self
            .web
            .get_with_query(&url, &[("include", "payment_routes")], MAX_RESPONSE_BYTES)
        {
            Ok(file) => Ok(
                serde_json::from_slice::<Envelope<TrackRoutesData>>(&file.bytes)
                    .ok()
                    .map(|e| e.data.into()),
            ),
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

    #[test]
    fn feed_routes_and_track_routes_give_the_recipients_and_the_tracks_and_a_404_gives_none() {
        let server = TestServer::start(vec![
            (
                "/v1/feeds/917393e3-1b1e-5cef-ace4-edaa54e1f810",
                200,
                br#"{"data":{"title":"Album","payment_routes":[
                    {"recipient_name":"Ann","route_type":"node","address":"02aa","split":95,"fee":false,"custom_key":null,"custom_value":null},
                    {"recipient_name":null,"route_type":"node","address":"02bb","split":5,"fee":true,"custom_key":"7629169","custom_value":"acct"}
                ],"tracks":[{"track_guid":"aa11","title":"Track One"}]}}"#
                    .to_vec(),
            ),
            (
                "/v1/feeds/917393e3-1b1e-5cef-ace4-edaa54e1f810/tracks/Track%201%2FA",
                200,
                br#"{"data":{"title":"Track One","payment_routes":[
                    {"recipient_name":"Ann","route_type":"node","address":"02aa","split":95,"fee":false,"custom_key":null,"custom_value":null}
                ]}}"#
                    .to_vec(),
            ),
        ]);
        let index = MusicIndex::new(Web::new(), &server.url(""));

        let feed = index
            .feed_routes("917393e3-1b1e-5cef-ace4-edaa54e1f810")
            .unwrap()
            .unwrap();
        assert_eq!(feed.title, "Album");
        assert_eq!(feed.recipients.len(), 2);
        assert_eq!(feed.recipients[0].name.as_deref(), Some("Ann"));
        assert_eq!(feed.recipients[0].split.as_deref(), Some("95"));
        assert_eq!(feed.recipients[0].fee, None);
        assert_eq!(feed.recipients[1].fee.as_deref(), Some("true"));
        assert_eq!(feed.recipients[1].custom_key.as_deref(), Some("7629169"));
        assert_eq!(feed.tracks.len(), 1);
        assert_eq!(feed.tracks[0].track_guid, "aa11");
        assert_eq!(feed.tracks[0].title, "Track One");

        let track = index
            .track_routes("917393e3-1b1e-5cef-ace4-edaa54e1f810", "Track 1/A")
            .unwrap()
            .unwrap();
        assert_eq!(track.title, "Track One");
        assert_eq!(track.recipients.len(), 1);
        assert!(track.tracks.is_empty());

        assert_eq!(
            index
                .feed_routes("00000000-0000-0000-0000-000000000000")
                .unwrap(),
            None
        );
        assert_eq!(
            index
                .track_routes("00000000-0000-0000-0000-000000000000", "aa11")
                .unwrap(),
            None
        );
    }
}

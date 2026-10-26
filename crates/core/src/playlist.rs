//! The tracks that a playlist can hold (ADR 0053 item 5).

use serde::Serialize;

use crate::collection::{Collection, CollectionError};

/// A track of a feed of the collection, for the track picker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PickTrack {
    /// The `podcast:guid` of the feed of the track.
    pub feed_guid: String,
    /// The public address of the feed.
    pub feed_url: Option<String>,
    /// The title of the feed.
    pub feed_title: Option<String>,
    /// The GUID of the track.
    pub item_guid: String,
    /// The title of the track.
    pub title: Option<String>,
}

/// The tracks of each music feed of the collection that has a
/// `podcast:guid`. The sequence is that of the feeds, and then that of the
/// items.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn own_tracks(collection: &Collection) -> Result<Vec<PickTrack>, CollectionError> {
    let mut tracks = Vec::new();
    for row in collection.feeds()? {
        if row.medium.as_deref() != Some("music") {
            continue;
        }
        let Some(feed_guid) = row.podcast_guid.clone() else {
            continue;
        };
        for (item_guid, title) in collection.item_titles(row.id)? {
            tracks.push(PickTrack {
                feed_guid: feed_guid.clone(),
                feed_url: row.public_url.clone(),
                feed_title: row.title.clone(),
                item_guid,
                title,
            });
        }
    }
    Ok(tracks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::Version;
    use crate::feed::Feed;
    use crate::xml::Document;
    use std::fmt::Write as _;

    fn add(collection: &mut Collection, url: &str, medium: &str, items: &[(&str, &str)]) {
        let id = collection.add_feed(Some(url), None).unwrap();
        let mut body = String::new();
        for (guid, title) in items {
            let _ = write!(
                body,
                "<item><title>{title}</title><guid>{guid}</guid></item>"
            );
        }
        let doc = Document::parse(&format!(
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel>\
             <title>Feed {url}</title><podcast:guid>guid-{url}</podcast:guid>\
             <podcast:medium>{medium}</podcast:medium>{body}</channel></rss>"
        ))
        .unwrap();
        collection
            .record_state(id, &Feed::read(&doc).unwrap(), &Version::new("v1"))
            .unwrap();
    }

    #[test]
    fn own_tracks_gives_the_tracks_of_the_music_feeds_only() {
        let mut collection = Collection::open_in_memory().unwrap();
        add(
            &mut collection,
            "https://m.example/a.xml",
            "music",
            &[("a1", "One"), ("a2", "Two")],
        );
        add(&mut collection, "https://m.example/p.xml", "publisher", &[]);
        add(
            &mut collection,
            "https://m.example/l.xml",
            "musicL",
            &[("l1", "List")],
        );
        add(
            &mut collection,
            "https://m.example/b.xml",
            "music",
            &[("b1", "Three")],
        );
        let tracks = own_tracks(&collection).unwrap();
        let keys: Vec<_> = tracks
            .iter()
            .map(|t| {
                (
                    t.feed_guid.as_str(),
                    t.item_guid.as_str(),
                    t.title.as_deref(),
                )
            })
            .collect();
        assert_eq!(
            keys,
            [
                ("guid-https://m.example/a.xml", "a1", Some("One")),
                ("guid-https://m.example/a.xml", "a2", Some("Two")),
                ("guid-https://m.example/b.xml", "b1", Some("Three")),
            ]
        );
        assert_eq!(
            tracks[0].feed_url.as_deref(),
            Some("https://m.example/a.xml")
        );
        assert_eq!(
            tracks[0].feed_title.as_deref(),
            Some("Feed https://m.example/a.xml")
        );
    }
}

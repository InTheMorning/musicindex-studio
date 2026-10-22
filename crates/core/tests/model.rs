//! Edits of a draft through the JSON model with entry keys (ADR 0022).

use studio_core::collection::{Collection, DraftRow, FeedId};
use studio_core::feed::{Enclosure, Entry, Feed, Item};
use studio_core::pipeline::{self, PipelineErrorKind};

const FEED: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<rss version=\"2.0\" xmlns:podcast=\"https://podcastindex.org/namespace/1.0\" xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\">
  <channel>
    <title>Album</title>
    <description>Old words</description>
    <podcast:guid>917393e3-1b1e-5cef-ace4-edaa54e1f810</podcast:guid>
    <item>
      <title>One</title>
      <guid isPermaLink=\"false\">track-1</guid>
      <itunes:explicit>false</itunes:explicit>
    </item>
    <item>
      <title>Two</title>
      <guid>track-2</guid>
      <podcast:season>2</podcast:season>
    </item>
  </channel>
</rss>
";

/// A collection with one feed. The draft of the feed holds [`FEED`].
fn draft() -> (Collection, FeedId) {
    let mut collection = Collection::open_in_memory().unwrap();
    let id = collection.add_feed(None, None).unwrap();
    let row = DraftRow {
        base_version: None,
        base_xml: FEED.as_bytes().to_vec(),
        draft_xml: FEED.as_bytes().to_vec(),
    };
    collection.save_draft(id, &row).unwrap();
    (collection, id)
}

/// The model as the UI gets it: JSON.
fn sent(feed: &Feed) -> Feed {
    serde_json::from_value(serde_json::to_value(feed).unwrap()).unwrap()
}

fn draft_text(collection: &Collection, id: FeedId) -> String {
    String::from_utf8(collection.draft(id).unwrap().unwrap().draft_xml).unwrap()
}

#[test]
fn each_entry_in_json_has_its_key_and_a_new_entry_has_none() {
    let (collection, id) = draft();
    let (feed, _) = pipeline::draft_model(&collection, id).unwrap();
    let json = serde_json::to_value(&feed).unwrap();
    assert_eq!(json["items"][0]["key"], 0);
    assert_eq!(json["items"][1]["key"], 1);
    assert_eq!(json["items"][1]["title"], "Two");
    let new = serde_json::to_value(Entry::new(Item::default())).unwrap();
    assert!(new.get("key").is_none());
}

#[test]
fn one_changed_field_changes_only_its_element() {
    let (collection, id) = draft();
    let (feed, revision) = pipeline::draft_model(&collection, id).unwrap();
    let mut model = sent(&feed);
    model.items[1].title = Some("Two (live)".to_owned());
    pipeline::replace_model(&collection, id, &revision, model).unwrap();
    assert_eq!(
        draft_text(&collection, id),
        FEED.replace("<title>Two</title>", "<title>Two (live)</title>")
    );
}

#[test]
fn a_moved_entry_keeps_the_elements_that_the_core_does_not_model() {
    let (collection, id) = draft();
    let (feed, revision) = pipeline::draft_model(&collection, id).unwrap();
    let mut model = sent(&feed);
    model.items.swap(0, 1);
    let (moved, _) = pipeline::replace_model(&collection, id, &revision, model).unwrap();
    assert_eq!(moved.items[0].title.as_deref(), Some("Two"));
    let text = draft_text(&collection, id);
    let two = text.find("<title>Two</title>").unwrap();
    let season = text.find("<podcast:season>2</podcast:season>").unwrap();
    let one = text.find("<title>One</title>").unwrap();
    let explicit = text
        .find("<itunes:explicit>false</itunes:explicit>")
        .unwrap();
    let guid = text
        .find("<guid isPermaLink=\"false\">track-1</guid>")
        .unwrap();
    assert!(two < season && season < one && one < guid && guid < explicit);
}

#[test]
fn an_entry_that_is_not_sent_goes_out_and_a_new_entry_gets_a_guid() {
    let (collection, id) = draft();
    let (feed, revision) = pipeline::draft_model(&collection, id).unwrap();
    let mut model = sent(&feed);
    model.items.remove(0);
    model.items.push(Entry::new(Item {
        title: Some("Three".to_owned()),
        ..Item::default()
    }));
    let (changed, _) = pipeline::replace_model(&collection, id, &revision, model).unwrap();
    let titles: Vec<_> = changed.items.iter().map(|i| i.title.clone()).collect();
    assert_eq!(titles, [Some("Two".to_owned()), Some("Three".to_owned())]);
    let guid = changed.items[1].guid.as_deref().unwrap();
    assert_eq!(uuid::Uuid::parse_str(guid).unwrap().get_version_num(), 4);
    assert!(
        draft_text(&collection, id).contains(&format!("<guid isPermaLink=\"false\">{guid}</guid>"))
    );
    assert!(!draft_text(&collection, id).contains("track-1"));
}

#[test]
fn an_unknown_or_repeated_key_gives_an_error_and_the_draft_stays() {
    let (collection, id) = draft();
    let (feed, revision) = pipeline::draft_model(&collection, id).unwrap();
    let mut json = serde_json::to_value(&feed).unwrap();
    json["items"][1]["key"] = 7.into();
    let unknown: Feed = serde_json::from_value(json).unwrap();
    let mut repeated = sent(&feed);
    repeated.items[1] = repeated.items[0].clone();
    for model in [unknown, repeated] {
        let error = pipeline::replace_model(&collection, id, &revision, model).unwrap_err();
        assert_eq!(error.kind(), PipelineErrorKind::InvalidModel);
        assert_eq!(draft_text(&collection, id), FEED);
    }
}

#[test]
fn a_model_from_a_previous_revision_gives_an_error() {
    let (collection, id) = draft();
    let (feed, revision) = pipeline::draft_model(&collection, id).unwrap();
    let mut first = sent(&feed);
    first.title = Some("New".to_owned());
    pipeline::replace_model(&collection, id, &revision, first).unwrap();
    let saved = draft_text(&collection, id);
    let error = pipeline::replace_model(&collection, id, &revision, sent(&feed)).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::StaleRevision);
    assert_eq!(draft_text(&collection, id), saved);
}

#[test]
fn a_changed_feed_guid_or_item_guid_gives_an_error() {
    let (collection, id) = draft();
    let (feed, revision) = pipeline::draft_model(&collection, id).unwrap();
    let mut feed_guid = sent(&feed);
    feed_guid.podcast_guid = Some("other".to_owned());
    let mut item_guid = sent(&feed);
    item_guid.items[0].guid = Some("other".to_owned());
    for model in [feed_guid, item_guid] {
        let error = pipeline::replace_model(&collection, id, &revision, model).unwrap_err();
        assert_eq!(error.kind(), PipelineErrorKind::InvalidModel);
    }
    assert_eq!(draft_text(&collection, id), FEED);
}

#[test]
fn a_cleared_text_field_removes_its_element() {
    let (collection, id) = draft();
    let (feed, revision) = pipeline::draft_model(&collection, id).unwrap();
    let mut json = serde_json::to_value(&feed).unwrap();
    json["description"] = serde_json::Value::Null;
    let model: Feed = serde_json::from_value(json).unwrap();
    pipeline::replace_model(&collection, id, &revision, model).unwrap();
    assert!(!draft_text(&collection, id).contains("description"));
}

#[test]
fn a_new_track_gets_the_indentation_of_the_other_tracks() {
    let (collection, id) = draft();
    let (feed, revision) = pipeline::draft_model(&collection, id).unwrap();
    let mut model = sent(&feed);
    model.items.push(Entry::new(Item {
        title: Some("Three".to_owned()),
        enclosure: Some(Enclosure {
            url: Some("https://music.example/3.mp3".to_owned()),
            length: Some("10".to_owned()),
            media_type: Some("audio/mpeg".to_owned()),
        }),
        ..Item::default()
    }));
    let (changed, _) = pipeline::replace_model(&collection, id, &revision, model).unwrap();
    let guid = changed.items[2].guid.as_deref().unwrap();
    let expected = format!(
        "      <podcast:season>2</podcast:season>\n      <podcast:episode>2</podcast:episode>\n    </item>\n    \
         <item>\n      <title>Three</title>\n      \
         <guid isPermaLink=\"false\">{guid}</guid>\n      \
         <enclosure url=\"https://music.example/3.mp3\" length=\"10\" type=\"audio/mpeg\"/>\n      \
         <podcast:episode>3</podcast:episode>\n    </item>\n  </channel>"
    );
    let text = draft_text(&collection, id);
    assert!(text.contains(&expected), "{text}");
}

#[test]
fn a_change_of_sequence_numbers_the_tracks_and_keeps_the_dates() {
    let (collection, id) = draft();
    let with_numbers = FEED
        .replace(
            "<title>One</title>",
            "<title>One</title>\n      <pubDate>Thu, 01 Oct 2026 00:00:00 GMT</pubDate>\n      <podcast:episode display=\"A1\">1</podcast:episode>",
        )
        .replace("<title>Two</title>", "<title>Two</title>\n      <podcast:episode>2</podcast:episode>");
    let row = DraftRow {
        base_version: None,
        base_xml: with_numbers.as_bytes().to_vec(),
        draft_xml: with_numbers.as_bytes().to_vec(),
    };
    collection.save_draft(id, &row).unwrap();

    // A change of a field is not a change of sequence.
    let (feed, revision) = pipeline::draft_model(&collection, id).unwrap();
    let mut model = sent(&feed);
    model.title = Some("Album 2".to_owned());
    let (feed, revision) = pipeline::replace_model(&collection, id, &revision, model).unwrap();
    assert_eq!(feed.items[0].episode.as_deref(), Some("1"));

    let mut model = sent(&feed);
    model.items.swap(0, 1);
    let (moved, _) = pipeline::replace_model(&collection, id, &revision, model).unwrap();
    let numbers: Vec<_> = moved
        .items
        .iter()
        .map(|i| (i.title.clone().unwrap(), i.episode.clone().unwrap()))
        .collect();
    assert_eq!(
        numbers,
        [
            ("Two".to_owned(), "1".to_owned()),
            ("One".to_owned(), "2".to_owned())
        ]
    );
    let text = draft_text(&collection, id);
    assert!(text.contains("<podcast:episode display=\"A1\">2</podcast:episode>"));
    assert!(text.contains("<pubDate>Thu, 01 Oct 2026 00:00:00 GMT</pubDate>"));
    assert_eq!(
        moved.items[1].pub_date.as_deref(),
        Some("Thu, 01 Oct 2026 00:00:00 GMT")
    );
}

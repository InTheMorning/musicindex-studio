//! The publisher feeds and their links, over the local folder connector
//! (ADR 0041).

use studio_core::collection::{Collection, ConnectorId, FeedId, PublisherRole};
use studio_core::connector::{self, Connector, LocalFolder};
use studio_core::feed::{Entry, Feed, Publisher, RemoteItem};
use studio_core::pipeline;
use studio_core::publisher::{self, PublisherErrorKind};

struct Setup {
    _dir: tempfile::TempDir,
    collection: Collection,
    connector: Box<dyn Connector>,
    connector_id: ConnectorId,
}

fn setup() -> Setup {
    let dir = tempfile::tempdir().unwrap();
    let folder = LocalFolder::new(dir.path(), Some("https://music.example".into()));
    let collection = Collection::open(&dir.path().join("collection.sqlite")).unwrap();
    let connector_id = collection
        .add_connector(LocalFolder::KIND, "Home folder", &folder.config(), None)
        .unwrap();
    let connector =
        connector::from_row(&collection.connector(connector_id).unwrap(), None).unwrap();
    Setup {
        _dir: dir,
        collection,
        connector,
        connector_id,
    }
}

fn new_feed(s: &mut Setup, path: &str, title: &str, medium: &str) -> FeedId {
    pipeline::new_feed(
        &mut s.collection,
        s.connector.as_ref(),
        s.connector_id,
        path,
        title,
        "Artist",
        medium,
    )
    .unwrap()
}

fn model(s: &Setup, id: FeedId) -> Feed {
    pipeline::draft_model(&s.collection, id).unwrap().0
}

/// A publisher feed and a release, both with a draft.
fn pair(s: &mut Setup) -> (FeedId, FeedId) {
    let publisher = new_feed(s, "label.xml", "Label", "publisher");
    let release = new_feed(s, "album.xml", "Album", "music");
    (publisher, release)
}

fn remote(medium: &str, guid: &str, url: &str, title: &str, rel: Option<&str>) -> RemoteItem {
    RemoteItem {
        medium: Some(medium.to_owned()),
        feed_guid: Some(guid.to_owned()),
        feed_url: Some(url.to_owned()),
        title: Some(title.to_owned()),
        rel: rel.map(str::to_owned),
        ..RemoteItem::default()
    }
}

#[test]
fn a_new_publisher_feed_is_a_publisher_row_with_no_role() {
    let mut s = setup();
    let publisher = new_feed(&mut s, "label.xml", "Label", "publisher");
    assert_eq!(model(&s, publisher).medium.as_deref(), Some("publisher"));
    let row = s.collection.feed(publisher).unwrap();
    assert_eq!(row.medium.as_deref(), Some("publisher"));
    assert_eq!(row.publisher_role, None);
}

#[test]
fn link_writes_one_item_in_each_draft_with_the_same_rel() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_role(publisher, Some(PublisherRole::Label))
        .unwrap();
    publisher::link(&s.collection, publisher, release).unwrap();

    let release_guid = model(&s, release).podcast_guid.unwrap();
    let release_url = s.collection.feed(release).unwrap().public_url.unwrap();
    let publisher_model = model(&s, publisher);
    assert_eq!(publisher_model.remote_items.len(), 1);
    let item = &publisher_model.remote_items[0];
    assert_eq!(item.medium.as_deref(), Some("music"));
    assert_eq!(item.feed_guid.as_deref(), Some(release_guid.as_str()));
    assert_eq!(item.feed_url.as_deref(), Some(release_url.as_str()));
    assert_eq!(item.title.as_deref(), Some("Album"));
    assert_eq!(item.rel.as_deref(), Some("label"));

    let publisher_guid = model(&s, publisher).podcast_guid.unwrap();
    let publisher_url = s.collection.feed(publisher).unwrap().public_url.unwrap();
    let release_model = model(&s, release);
    let links = &release_model.publisher.as_ref().unwrap().remote_items;
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].medium.as_deref(), Some("publisher"));
    assert_eq!(links[0].feed_guid.as_deref(), Some(publisher_guid.as_str()));
    assert_eq!(links[0].feed_url.as_deref(), Some(publisher_url.as_str()));
    assert_eq!(links[0].title.as_deref(), Some("Label"));
    assert_eq!(links[0].rel.as_deref(), Some("label"));

    // The role is recorded by the first link.
    assert_eq!(
        s.collection.feed(publisher).unwrap().publisher_role,
        Some(PublisherRole::Label)
    );
    assert_eq!(
        publisher::linked_releases(&s.collection, publisher).unwrap(),
        vec![release]
    );
}

#[test]
fn a_second_link_of_the_same_feeds_adds_no_item() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_role(publisher, Some(PublisherRole::Artist))
        .unwrap();
    publisher::link(&s.collection, publisher, release).unwrap();
    publisher::link(&s.collection, publisher, release).unwrap();
    assert_eq!(model(&s, publisher).remote_items.len(), 1);
    assert_eq!(model(&s, release).publisher.unwrap().remote_items.len(), 1);
}

#[test]
fn a_new_link_goes_last_after_the_links_of_the_release() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_role(publisher, Some(PublisherRole::Label))
        .unwrap();
    pipeline::edit_draft(&s.collection, release, |feed| {
        feed.publisher = Some(Publisher {
            remote_items: vec![Entry::new(remote(
                "publisher",
                "other-guid",
                "https://music.example/other.xml",
                "Other",
                Some("artist"),
            ))],
        });
    })
    .unwrap();
    publisher::link(&s.collection, publisher, release).unwrap();

    let links = model(&s, release).publisher.unwrap().remote_items;
    assert_eq!(links.len(), 2);
    assert_eq!(links[0].feed_guid.as_deref(), Some("other-guid"));
    assert_eq!(links[1].title.as_deref(), Some("Label"));
}

#[test]
fn unlink_removes_the_link_and_keeps_an_item_that_names_no_feed_of_the_collection() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_role(publisher, Some(PublisherRole::Label))
        .unwrap();
    // A link to a feed that is not in the collection, first in the list.
    pipeline::edit_draft(&s.collection, publisher, |feed| {
        feed.remote_items.push(Entry::new(remote(
            "music",
            "stranger-guid",
            "https://music.example/stranger.xml",
            "Stranger",
            Some("label"),
        )));
    })
    .unwrap();
    publisher::link(&s.collection, publisher, release).unwrap();
    publisher::unlink(&s.collection, publisher, release).unwrap();

    let remaining = model(&s, publisher).remote_items;
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].feed_guid.as_deref(), Some("stranger-guid"));
    assert!(model(&s, release).publisher.is_none());
}

#[test]
fn unlink_keeps_the_other_publisher_links_of_a_release() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_role(publisher, Some(PublisherRole::Label))
        .unwrap();
    pipeline::edit_draft(&s.collection, release, |feed| {
        feed.publisher = Some(Publisher {
            remote_items: vec![Entry::new(remote(
                "publisher",
                "other-guid",
                "https://music.example/other.xml",
                "Other",
                Some("artist"),
            ))],
        });
    })
    .unwrap();
    publisher::link(&s.collection, publisher, release).unwrap();
    publisher::unlink(&s.collection, publisher, release).unwrap();

    let links = model(&s, release).publisher.unwrap().remote_items;
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].feed_guid.as_deref(), Some("other-guid"));
}

#[test]
fn set_role_changes_the_rel_in_the_publisher_draft_and_in_each_linked_release() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_role(publisher, Some(PublisherRole::Artist))
        .unwrap();
    publisher::link(&s.collection, publisher, release).unwrap();

    publisher::set_role(&s.collection, publisher, PublisherRole::Label).unwrap();

    assert_eq!(
        model(&s, publisher).remote_items[0].rel.as_deref(),
        Some("label")
    );
    assert_eq!(
        model(&s, release).publisher.unwrap().remote_items[0]
            .rel
            .as_deref(),
        Some("label")
    );
    assert_eq!(
        s.collection.feed(publisher).unwrap().publisher_role,
        Some(PublisherRole::Label)
    );
}

#[test]
fn role_records_the_inferred_role_of_the_draft() {
    let mut s = setup();
    let (publisher, _release) = pair(&mut s);
    pipeline::edit_draft(&s.collection, publisher, |feed| {
        feed.remote_items.push(Entry::new(remote(
            "music",
            "album-guid",
            "https://music.example/album.xml",
            "Album",
            Some(" LABEL "),
        )));
    })
    .unwrap();
    assert_eq!(
        publisher::role(&s.collection, publisher).unwrap(),
        PublisherRole::Label
    );
    assert_eq!(
        s.collection.feed(publisher).unwrap().publisher_role,
        Some(PublisherRole::Label)
    );
}

#[test]
fn role_gives_no_role_for_items_with_no_rel() {
    let mut s = setup();
    let (publisher, _release) = pair(&mut s);
    pipeline::edit_draft(&s.collection, publisher, |feed| {
        feed.remote_items.push(Entry::new(remote(
            "music",
            "album-guid",
            "https://music.example/album.xml",
            "Album",
            None,
        )));
    })
    .unwrap();
    let error = publisher::role(&s.collection, publisher).unwrap_err();
    assert_eq!(error.kind(), PublisherErrorKind::NoRole);
}

#[test]
fn link_gives_the_error_kinds_of_its_checks() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    let not_publisher = publisher::link(&s.collection, release, release).unwrap_err();
    assert_eq!(not_publisher.kind(), PublisherErrorKind::NotPublisher);
    let not_release = publisher::link(&s.collection, publisher, publisher).unwrap_err();
    assert_eq!(not_release.kind(), PublisherErrorKind::NotRelease);

    // A feed with no connector is read-only.
    let read_only = s.collection.add_feed(None, None).unwrap();
    s.collection
        .set_new_feed(read_only, "Read only", "music")
        .unwrap();
    let error = publisher::link(&s.collection, publisher, read_only).unwrap_err();
    assert_eq!(error.kind(), PublisherErrorKind::ReadOnly);

    // The publisher feed has no role, and its draft has no music item.
    let error = publisher::link(&s.collection, publisher, release).unwrap_err();
    assert_eq!(error.kind(), PublisherErrorKind::NoRole);
    assert!(model(&s, release).publisher.is_none());
}

#[test]
fn inferred_role_needs_one_valid_rel_for_each_music_item() {
    let mut feed = Feed::default();
    assert_eq!(publisher::inferred_role(&feed), None);
    feed.remote_items.push(Entry::new(remote(
        "music",
        "a",
        "https://x/a.xml",
        "A",
        Some("artist"),
    )));
    assert_eq!(publisher::inferred_role(&feed), Some(PublisherRole::Artist));
    feed.remote_items.push(Entry::new(remote(
        "music",
        "b",
        "https://x/b.xml",
        "B",
        Some("label"),
    )));
    assert_eq!(publisher::inferred_role(&feed), None);
    feed.remote_items[1].rel = Some("band".to_owned());
    assert_eq!(publisher::inferred_role(&feed), None);
}

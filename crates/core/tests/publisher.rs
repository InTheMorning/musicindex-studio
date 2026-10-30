//! The publisher feeds and their links, over the local folder connector
//! (ADR 0041).

use studio_core::collection::{
    Collection, ConnectorId, FeedId, PublisherRole, inferred_roles, rel_of, roles_from_rel,
};
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
    assert_eq!(row.publisher_roles, Vec::new());
}

#[test]
fn link_writes_one_item_in_each_draft_with_the_same_rel() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(publisher, &[PublisherRole::Label], true)
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
        s.collection.feed(publisher).unwrap().publisher_roles,
        [PublisherRole::Label]
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
        .set_publisher_roles(publisher, &[PublisherRole::Artist], true)
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
        .set_publisher_roles(publisher, &[PublisherRole::Label], true)
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
        .set_publisher_roles(publisher, &[PublisherRole::Label], true)
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
        .set_publisher_roles(publisher, &[PublisherRole::Label], true)
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
fn set_roles_changes_the_rel_in_the_publisher_draft_and_in_each_linked_release() {
    let mut s = setup();
    let (publisher, release) = pair(&mut s);
    s.collection
        .set_publisher_roles(publisher, &[PublisherRole::Artist], true)
        .unwrap();
    publisher::link(&s.collection, publisher, release).unwrap();

    publisher::set_roles(&s.collection, publisher, &[PublisherRole::Label]).unwrap();

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
        s.collection.feed(publisher).unwrap().publisher_roles,
        [PublisherRole::Label]
    );
}

#[test]
fn roles_records_and_confirms_the_inferred_roles_of_the_draft() {
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
        publisher::roles(&s.collection, publisher).unwrap(),
        [PublisherRole::Label]
    );
    assert_eq!(
        s.collection.feed(publisher).unwrap().publisher_roles,
        [PublisherRole::Label]
    );
}

#[test]
fn roles_gives_no_role_for_items_with_no_rel() {
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
    let error = publisher::roles(&s.collection, publisher).unwrap_err();
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
fn inferred_roles_read_each_rel_as_a_set_of_roles() {
    let mut feed = Feed::default();
    assert_eq!(inferred_roles(&feed), Vec::new());
    // The HeyCitizen artist feed of 2026-10-29 (ADR 0082).
    feed.remote_items.push(Entry::new(remote(
        "music",
        "a",
        "https://x/a.xml",
        "A",
        Some("artist host author label producer"),
    )));
    assert_eq!(
        inferred_roles(&feed),
        [
            PublisherRole::Artist,
            PublisherRole::Label,
            PublisherRole::Producer,
            PublisherRole::Host,
            PublisherRole::Author
        ]
    );
    // A comma list, other cases, a word that is not a role, and a
    // repeated role. Each role of each music item counts one time.
    feed.remote_items.push(Entry::new(remote(
        "music",
        "b",
        "https://x/b.xml",
        "B",
        Some("Sponsor, band, HOST"),
    )));
    assert_eq!(inferred_roles(&feed).last(), Some(&PublisherRole::Sponsor));
    assert_eq!(inferred_roles(&feed).len(), 6);
}

#[test]
fn rel_of_writes_the_roles_in_their_sequence() {
    assert_eq!(
        rel_of(&[
            PublisherRole::Producer,
            PublisherRole::Artist,
            PublisherRole::Artist
        ]),
        "artist producer"
    );
    assert_eq!(
        roles_from_rel("producer,artist"),
        [PublisherRole::Artist, PublisherRole::Producer]
    );
    // Hosting comes third (musicindex ADR 0036 R-ROLE-1, ADR 0083).
    assert_eq!(
        rel_of(&[
            PublisherRole::Host,
            PublisherRole::Hosting,
            PublisherRole::Label
        ]),
        "label hosting host"
    );
}

#[test]
fn a_read_records_inferred_roles_and_keeps_confirmed_roles() {
    let mut s = setup();
    let (publisher, _release) = pair(&mut s);
    let mut feed = Feed {
        medium: Some("publisher".to_owned()),
        ..Feed::default()
    };
    feed.remote_items.push(Entry::new(remote(
        "music",
        "a",
        "https://x/a.xml",
        "A",
        Some("artist host author label producer"),
    )));
    let version = connector::Version::new("v1");
    s.collection
        .record_state(publisher, &feed, &version)
        .unwrap();
    let row = s.collection.feed(publisher).unwrap();
    assert_eq!(row.publisher_roles.first(), Some(&PublisherRole::Artist));
    assert_eq!(row.publisher_roles.len(), 5);
    assert!(!row.roles_confirmed);

    // A confirmed set does not change at the next read (ADR 0082 item 4).
    s.collection
        .set_publisher_roles(publisher, &[PublisherRole::Label], true)
        .unwrap();
    s.collection
        .record_state(publisher, &feed, &version)
        .unwrap();
    let row = s.collection.feed(publisher).unwrap();
    assert_eq!(row.publisher_roles, [PublisherRole::Label]);
    assert!(row.roles_confirmed);
}

#[test]
fn a_playlist_links_to_a_publisher_feed_as_an_album_does() {
    let mut s = setup();
    let publisher = new_feed(&mut s, "label.xml", "Label", "publisher");
    let playlist = new_feed(&mut s, "list.xml", "Mixtape", "musicL");
    s.collection
        .set_publisher_roles(publisher, &[PublisherRole::Artist], true)
        .unwrap();
    publisher::link(&s.collection, publisher, playlist).unwrap();

    // ADR 0086 item 2: the channel item has the medium of the playlist.
    let channel = &model(&s, publisher).remote_items;
    assert_eq!(channel.len(), 1);
    assert_eq!(channel[0].value.medium.as_deref(), Some("musicL"));
    assert_eq!(channel[0].value.rel.as_deref(), Some("artist"));
    let links = model(&s, playlist).publisher.unwrap().remote_items;
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].value.medium.as_deref(), Some("publisher"));

    assert_eq!(
        publisher::linked_releases(&s.collection, publisher).unwrap(),
        vec![playlist]
    );
    // A role change reaches the playlist item, and the read of the roles
    // takes its rel.
    publisher::set_roles(&s.collection, publisher, &[PublisherRole::Label]).unwrap();
    assert_eq!(
        model(&s, publisher).remote_items[0].value.rel.as_deref(),
        Some("label")
    );
    assert_eq!(
        inferred_roles(&model(&s, publisher)),
        vec![PublisherRole::Label]
    );
}

#[test]
fn catalogs_give_the_releases_of_each_publisher_and_the_publishers_of_each_release() {
    let mut s = setup();
    let artist = new_feed(&mut s, "artist.xml", "Artist", "publisher");
    let label = new_feed(&mut s, "label.xml", "Label", "publisher");
    let first = new_feed(&mut s, "first.xml", "First", "music");
    let shared = new_feed(&mut s, "shared.xml", "Shared", "music");
    let mixtape = new_feed(&mut s, "mixtape.xml", "Mixtape", "musicL");
    let half = new_feed(&mut s, "half.xml", "Half", "music");
    let alone = new_feed(&mut s, "alone.xml", "Alone", "music");
    s.collection
        .set_publisher_roles(artist, &[PublisherRole::Artist], true)
        .unwrap();
    s.collection
        .set_publisher_roles(label, &[PublisherRole::Label], true)
        .unwrap();
    publisher::link(&s.collection, artist, first).unwrap();
    publisher::link(&s.collection, artist, shared).unwrap();
    publisher::link(&s.collection, label, shared).unwrap();
    publisher::link(&s.collection, label, mixtape).unwrap();
    // A link in the draft of the release only.
    let label_model = model(&s, label);
    pipeline::edit_draft(&s.collection, half, |feed| {
        feed.publisher = Some(Publisher {
            remote_items: vec![Entry::new(remote(
                "publisher",
                label_model.podcast_guid.as_deref().unwrap(),
                "https://music.example/label.xml",
                "Label",
                Some("label"),
            ))],
        });
    })
    .unwrap();

    let feeds: Vec<_> = s
        .collection
        .feeds()
        .unwrap()
        .into_iter()
        .map(|row| {
            let feed = model(&s, row.id);
            (row, Some(feed))
        })
        .collect();
    let catalogs = publisher::catalogs(&feeds);
    let of = |id: FeedId| {
        let index = feeds.iter().position(|(row, _)| row.id == id).unwrap();
        catalogs[index].clone()
    };
    assert_eq!(of(artist).releases, vec![first, shared]);
    // ADR 0085 item 2: the listed releases in their sequence, then the
    // release that names the label only.
    assert_eq!(of(label).releases, vec![shared, mixtape, half]);
    // ADR 0085 item 5: the primary publisher comes first.
    assert_eq!(of(shared).publishers, vec![artist, label]);
    assert_eq!(of(first).publishers, vec![artist]);
    assert_eq!(of(mixtape).publishers, vec![label]);
    assert_eq!(of(half).publishers, vec![label]);
    assert_eq!(of(alone).publishers, Vec::new());
    assert_eq!(of(alone).releases, Vec::new());
}

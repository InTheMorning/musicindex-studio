//! Practice pages on a file host with no public address (ADR 0121).

use std::fs;

use studio_core::collection::{Collection, ConnectorId, FeedId, PublisherRole};
use studio_core::connector::LocalFolder;
use studio_core::feed::{Enclosure, Entry, Feed, Item};
use studio_core::guid::podcast_guid_for_url;
use studio_core::inventory::Hosts;
use studio_core::library::Library;
use studio_core::pipeline::{self, AddressCheck, DiffId, PipelineErrorKind};
use studio_core::publisher;

/// An address check that passes each address. No test sends a request to
/// the network.
struct PassAll;

impl AddressCheck for PassAll {
    fn check(&self, _url: &str) -> Result<(), String> {
        Ok(())
    }
}

/// Two file hosts over two folders. `private` has no public address, and
/// `public` has the base address `https://music.example`.
struct Setup {
    dir: tempfile::TempDir,
    collection: Collection,
    hosts: Hosts,
    library: Library,
    private: ConnectorId,
    public: ConnectorId,
}

fn setup() -> Setup {
    let dir = tempfile::tempdir().unwrap();
    let private_root = dir.path().join("private");
    let public_root = dir.path().join("public");
    fs::create_dir_all(&private_root).unwrap();
    fs::create_dir_all(&public_root).unwrap();
    let private_folder = LocalFolder::new(private_root, None);
    let public_folder = LocalFolder::new(public_root, Some("https://music.example".into()));
    let collection = Collection::open(&dir.path().join("collection.sqlite")).unwrap();
    let private = collection
        .add_connector(LocalFolder::KIND, "Private", &private_folder.config(), None)
        .unwrap();
    let public = collection
        .add_connector(LocalFolder::KIND, "Public", &public_folder.config(), None)
        .unwrap();
    let mut hosts = Hosts::new();
    hosts.insert(private, Box::new(private_folder));
    hosts.insert(public, Box::new(public_folder));
    let library = Library::open(dir.path().join("library")).unwrap();
    Setup {
        dir,
        collection,
        hosts,
        library,
        private,
        public,
    }
}

/// A new album on the private folder. It is a practice page.
fn practice_album(s: &mut Setup, path: &str) -> FeedId {
    let connector = s.hosts.get(s.private).unwrap();
    pipeline::new_feed(
        &mut s.collection,
        connector,
        s.private,
        path,
        "Album",
        "Artist",
        "music",
    )
    .unwrap()
}

/// A new artist page on the private folder, with the role artist. It is a
/// practice page.
fn practice_artist(s: &mut Setup, path: &str) -> FeedId {
    let connector = s.hosts.get(s.private).unwrap();
    let id = pipeline::new_feed(
        &mut s.collection,
        connector,
        s.private,
        path,
        "Artist",
        "Artist",
        "publisher",
    )
    .unwrap();
    s.collection
        .set_publisher_roles(id, &[PublisherRole::Artist], true)
        .unwrap();
    id
}

fn temporary_guid(s: &Setup, path: &str) -> String {
    podcast_guid_for_url(&format!("studio-practice://{}/{path}", s.private.0))
}

#[test]
fn a_new_feed_with_no_public_address_is_a_practice_page() {
    let mut s = setup();
    let id = practice_album(&mut s, "my-album.xml");
    let row = s.collection.feed(id).unwrap();
    assert_eq!(row.public_url, None);
    assert!(pipeline::is_practice(&s.collection, &s.hosts, &row));

    let (feed, _) = pipeline::draft_model(&s.collection, id).unwrap();
    assert_eq!(feed.podcast_guid, Some(temporary_guid(&s, "my-album.xml")));
    assert!(
        feed.atom_links.is_empty(),
        "a practice page has no self link"
    );
}

#[test]
fn a_new_feed_with_a_public_address_is_not_a_practice_page() {
    let mut s = setup();
    let connector = s.hosts.get(s.public).unwrap();
    let id = pipeline::new_feed(
        &mut s.collection,
        connector,
        s.public,
        "public.xml",
        "Album",
        "Artist",
        "music",
    )
    .unwrap();
    let row = s.collection.feed(id).unwrap();
    assert_eq!(
        row.public_url.as_deref(),
        Some("https://music.example/public.xml")
    );
    assert!(!pipeline::is_practice(&s.collection, &s.hosts, &row));
}

#[test]
fn write_draft_refuses_a_practice_page_and_writes_no_file() {
    let mut s = setup();
    let id = practice_album(&mut s, "my-album.xml");
    let error = pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        id,
        &DiffId::new("unchecked"),
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::PracticePage);
    assert!(!s.dir.path().join("private/my-album.xml").exists());
}

#[test]
fn publish_practice_gives_each_page_its_real_guid_and_links() {
    let mut s = setup();
    let artist = practice_artist(&mut s, "artist.xml");
    let album = practice_album(&mut s, "album.xml");
    publisher::link(&s.collection, artist, album).unwrap();
    let temporary = temporary_guid(&s, "album.xml");
    pipeline::edit_draft(&s.collection, album, |feed| {
        feed.items.push(Entry::new(Item {
            enclosure: Some(Enclosure {
                url: Some(format!(
                    "https://op3.dev/e,pg={temporary}/https://cdn.example/track.mp3"
                )),
                ..Enclosure::default()
            }),
            ..Item::default()
        }));
    })
    .unwrap();

    let ids = pipeline::publish_practice(&mut s.collection, &s.hosts, s.private, s.public).unwrap();
    assert_eq!(ids, [artist, album]);

    let artist_url = "https://music.example/artist.xml";
    let album_url = "https://music.example/album.xml";
    let (artist_model, album_model) = (
        assert_moved(&s, artist, "artist.xml", artist_url),
        assert_moved(&s, album, "album.xml", album_url),
    );

    let album_guid = podcast_guid_for_url(album_url);
    let artist_guid = podcast_guid_for_url(artist_url);
    let channel_link = &artist_model.remote_items[0].value;
    assert_eq!(channel_link.feed_guid.as_deref(), Some(album_guid.as_str()));
    assert_eq!(channel_link.feed_url.as_deref(), Some(album_url));
    let publisher_link = &album_model.publisher.as_ref().unwrap().remote_items[0].value;
    assert_eq!(
        publisher_link.feed_guid.as_deref(),
        Some(artist_guid.as_str())
    );
    assert_eq!(publisher_link.feed_url.as_deref(), Some(artist_url));

    // ADR 0076: the OP3 prefix of the enclosure gets the real GUID.
    let album_model = pipeline::draft_model(&s.collection, album).unwrap().0;
    let enclosure = album_model.items[0].value.enclosure.as_ref().unwrap();
    assert_eq!(
        enclosure.url.as_deref(),
        Some(format!("https://op3.dev/e,pg={album_guid}/https://cdn.example/track.mp3").as_str())
    );

    assert!(!s.dir.path().join("public/artist.xml").exists());
    assert!(!s.dir.path().join("public/album.xml").exists());
}

/// Check that a page moved to the public host. Gives its draft model.
fn assert_moved(s: &Setup, id: FeedId, path: &str, url: &str) -> Feed {
    let row = s.collection.feed(id).unwrap();
    assert_eq!(row.connector_id, Some(s.public));
    assert_eq!(row.path.as_deref(), Some(path));
    assert_eq!(row.public_url.as_deref(), Some(url));
    assert!(!pipeline::is_practice(&s.collection, &s.hosts, &row));
    let (feed, _) = pipeline::draft_model(&s.collection, id).unwrap();
    assert_eq!(feed.podcast_guid, Some(podcast_guid_for_url(url)));
    let self_link = feed
        .atom_links
        .iter()
        .find(|link| link.value.rel.as_deref() == Some("self"))
        .expect("a moved page has a self link");
    assert_eq!(self_link.value.href.as_deref(), Some(url));
    feed
}

#[test]
fn publish_practice_stops_at_a_file_on_the_target_and_changes_nothing() {
    let mut s = setup();
    let first = practice_album(&mut s, "a.xml");
    let second = practice_album(&mut s, "b.xml");
    let before = pipeline::draft_model(&s.collection, first).unwrap().0;
    fs::write(s.dir.path().join("public/b.xml"), "<rss/>").unwrap();

    let error =
        pipeline::publish_practice(&mut s.collection, &s.hosts, s.private, s.public).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::FileExists);
    for id in [first, second] {
        let row = s.collection.feed(id).unwrap();
        assert_eq!(row.connector_id, Some(s.private));
        assert_eq!(row.public_url, None);
    }
    assert_eq!(
        pipeline::draft_model(&s.collection, first).unwrap().0,
        before
    );
}

#[test]
fn publish_practice_needs_a_public_address_and_practice_pages() {
    let mut s = setup();
    practice_album(&mut s, "album.xml");
    let error =
        pipeline::publish_practice(&mut s.collection, &s.hosts, s.private, s.private).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::NoPublicAddress);

    let error =
        pipeline::publish_practice(&mut s.collection, &s.hosts, s.public, s.private).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::NotFound);
}

#[test]
fn copy_feed_on_a_local_folder_with_no_public_address_makes_a_practice_page() {
    let mut s = setup();
    let source = practice_album(&mut s, "album.xml");
    let copy = pipeline::copy_feed(
        &mut s.collection,
        s.hosts.get(s.private).unwrap(),
        s.private,
        "copy.xml",
        "Copy",
        "Artist",
        source,
    )
    .unwrap();
    let row = s.collection.feed(copy).unwrap();
    assert_eq!(row.public_url, None);
    assert!(pipeline::is_practice(&s.collection, &s.hosts, &row));
    let (feed, _) = pipeline::draft_model(&s.collection, copy).unwrap();
    assert_eq!(feed.podcast_guid, Some(temporary_guid(&s, "copy.xml")));
    assert!(
        feed.atom_links.is_empty(),
        "a practice page has no self link"
    );
}

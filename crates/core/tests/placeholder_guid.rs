//! Feeds on a file host with no public address get a placeholder GUID, and
//! the real GUID when the host gets a public address (ADR 0122).

use std::fs;

use studio_core::announce;
use studio_core::collection::{Collection, ConnectorId, FeedId, PublisherRole};
use studio_core::connector::LocalFolder;
use studio_core::feed::{Enclosure, Entry, Feed, Item};
use studio_core::guid::podcast_guid_for_url;
use studio_core::inventory::Hosts;
use studio_core::library::Library;
use studio_core::pipeline::{self, AddressCheck};
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

/// A new album on the private folder.
fn new_album(s: &mut Setup, path: &str) -> FeedId {
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

/// A new artist page on the private folder, with the role artist.
fn new_artist(s: &mut Setup, path: &str) -> FeedId {
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

/// Write a new feed on the private folder, with its approved diff.
fn write_new(s: &mut Setup, id: FeedId) {
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, id).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        id,
        &diff.id,
    )
    .unwrap();
}

/// Start a draft of a written feed on the private folder.
fn start(s: &Setup, id: FeedId) {
    let connector = s.hosts.get(s.private).unwrap();
    pipeline::start_draft(&s.collection, connector, id).unwrap();
}

fn draft_guid(s: &Setup, id: FeedId) -> Option<String> {
    pipeline::draft_model(&s.collection, id)
        .unwrap()
        .0
        .podcast_guid
}

#[test]
fn two_new_feeds_with_no_public_address_get_two_placeholder_guids() {
    let mut s = setup();
    let first = new_album(&mut s, "first.xml");
    let second = new_album(&mut s, "second.xml");
    let row = s.collection.feed(first).unwrap();
    assert_eq!(row.public_url, None);

    let (feed, _) = pipeline::draft_model(&s.collection, first).unwrap();
    let first_guid = feed.podcast_guid.expect("a new feed has a GUID");
    assert!(pipeline::has_placeholder_guid(&first_guid));
    assert!(
        feed.atom_links.is_empty(),
        "a feed with a placeholder GUID has no self link"
    );
    let second_guid = draft_guid(&s, second).expect("a new feed has a GUID");
    assert!(pipeline::has_placeholder_guid(&second_guid));
    assert_ne!(first_guid, second_guid, "each feed has its own placeholder");
}

#[test]
fn a_write_of_a_placeholder_feed_puts_the_file_and_queues_no_announcement() {
    let mut s = setup();
    let id = new_album(&mut s, "my-album.xml");
    write_new(&mut s, id);
    assert!(s.dir.path().join("private/my-album.xml").exists());
    assert_eq!(announce::recent(&s.collection, id).unwrap().len(), 0);
}

#[test]
fn a_move_to_a_file_host_with_a_public_address_gives_the_real_guids_and_links() {
    let mut s = setup();
    let artist = new_artist(&mut s, "artist.xml");
    let album = new_album(&mut s, "album.xml");
    publisher::link(&s.collection, artist, album).unwrap();
    write_new(&mut s, artist);
    write_new(&mut s, album);
    start(&s, artist);
    start(&s, album);

    let album_url = "https://music.example/album.xml";
    let artist_url = "https://music.example/artist.xml";
    let album_guid = podcast_guid_for_url(album_url);
    let artist_guid = podcast_guid_for_url(artist_url);

    pipeline::set_feed_host(&s.collection, &s.hosts, album, s.public).unwrap();
    let album_model: Feed = pipeline::draft_model(&s.collection, album).unwrap().0;
    assert_eq!(
        album_model.podcast_guid.as_deref(),
        Some(album_guid.as_str())
    );
    assert_eq!(
        album_model.self_url(),
        Some(album_url),
        "the moved feed has its real self link"
    );
    let artist_model = pipeline::draft_model(&s.collection, artist).unwrap().0;
    let channel_link = &artist_model.remote_items[0].value;
    assert_eq!(channel_link.feed_guid.as_deref(), Some(album_guid.as_str()));
    assert_eq!(channel_link.feed_url.as_deref(), Some(album_url));

    pipeline::set_feed_host(&s.collection, &s.hosts, artist, s.public).unwrap();
    let artist_model = pipeline::draft_model(&s.collection, artist).unwrap().0;
    assert_eq!(
        artist_model.podcast_guid.as_deref(),
        Some(artist_guid.as_str())
    );
    let album_model = pipeline::draft_model(&s.collection, album).unwrap().0;
    let publisher_link = &album_model.publisher.as_ref().unwrap().remote_items[0].value;
    assert_eq!(
        publisher_link.feed_guid.as_deref(),
        Some(artist_guid.as_str())
    );
    assert_eq!(publisher_link.feed_url.as_deref(), Some(artist_url));
}

#[test]
fn a_new_public_address_gives_the_real_guid_to_each_feed_with_a_placeholder() {
    let mut s = setup();
    let album = new_album(&mut s, "album.xml");
    write_new(&mut s, album);
    start(&s, album);

    // The private folder gets a public address.
    let mut hosts = Hosts::new();
    hosts.insert(
        s.private,
        Box::new(LocalFolder::new(
            s.dir.path().join("private"),
            Some("https://private.example".into()),
        )),
    );
    let changed = pipeline::assign_real_guids(&s.collection, &hosts, &[album]).unwrap();
    assert_eq!(changed, [album]);
    let expected = podcast_guid_for_url("https://private.example/album.xml");
    assert_eq!(draft_guid(&s, album).as_deref(), Some(expected.as_str()));
}

/// A written feed has no draft. The real GUID still comes to it, and the
/// links in other written feeds change.
#[test]
fn a_written_feed_with_no_draft_gets_its_real_guid_and_its_links() {
    let mut s = setup();
    let artist = new_artist(&mut s, "artist.xml");
    let album = new_album(&mut s, "album.xml");
    publisher::link(&s.collection, artist, album).unwrap();
    write_new(&mut s, artist);
    write_new(&mut s, album);
    assert!(s.collection.draft(artist).unwrap().is_none());
    assert!(s.collection.draft(album).unwrap().is_none());

    // A move of the album changes the link in the written artist page.
    start(&s, album);
    pipeline::set_feed_host(&s.collection, &s.hosts, album, s.public).unwrap();
    let album_url = "https://music.example/album.xml";
    let album_guid = podcast_guid_for_url(album_url);
    let artist_model = pipeline::draft_model(&s.collection, artist)
        .expect("the artist page gets a draft with the changed link")
        .0;
    let channel_link = &artist_model.remote_items[0].value;
    assert_eq!(channel_link.feed_guid.as_deref(), Some(album_guid.as_str()));
    assert_eq!(channel_link.feed_url.as_deref(), Some(album_url));
}

/// A new public address gives the real GUID to a written feed with no draft.
#[test]
fn a_new_public_address_gives_the_real_guid_to_a_written_feed_with_no_draft() {
    let mut s = setup();
    let album = new_album(&mut s, "album.xml");
    let other = new_album(&mut s, "other.xml");
    write_new(&mut s, album);
    write_new(&mut s, other);
    assert!(s.collection.draft(album).unwrap().is_none());

    let mut hosts = Hosts::new();
    hosts.insert(
        s.private,
        Box::new(LocalFolder::new(
            s.dir.path().join("private"),
            Some("https://private.example".into()),
        )),
    );
    let changed = pipeline::assign_real_guids(&s.collection, &hosts, &[album]).unwrap();
    assert_eq!(changed, [album], "only the named feed changes");
    let expected = podcast_guid_for_url("https://private.example/album.xml");
    assert_eq!(draft_guid(&s, album).as_deref(), Some(expected.as_str()));
    assert!(
        s.collection.draft(other).unwrap().is_none(),
        "a feed with no change gets no draft"
    );
}

#[test]
fn copy_feed_with_no_public_address_gets_a_placeholder_guid() {
    let mut s = setup();
    let source = new_album(&mut s, "album.xml");
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
    let guid = draft_guid(&s, copy).expect("a copy has a GUID");
    assert!(pipeline::has_placeholder_guid(&guid));
    let (feed, _) = pipeline::draft_model(&s.collection, copy).unwrap();
    assert_eq!(feed.atom_links.len(), 0, "a copy has no self link");
}

/// Import one track into the release folder `folder` of `id`, and add it to
/// the draft. Gives the address of the track in the draft.
fn add_track(s: &mut Setup, id: FeedId, folder: &str) -> String {
    s.collection.set_feed_folder(id, folder).unwrap();
    let source = s.dir.path().join("01 Song.mp3");
    fs::write(&source, b"music!!").unwrap();
    let imported = pipeline::import_into_library(
        &s.collection,
        &s.library,
        &s.hosts,
        id,
        &source,
        "01 Song.mp3",
        "audio/mpeg",
    )
    .unwrap();
    let url = imported.planned.url.clone().unwrap();
    let item = Item {
        title: Some("Song".to_owned()),
        enclosure: Some(Enclosure {
            url: Some(url.clone()),
            length: Some("7".to_owned()),
            media_type: Some("audio/mpeg".to_owned()),
        }),
        ..Item::default()
    };
    pipeline::edit_draft(&s.collection, id, move |feed| {
        feed.items.push(Entry::new(item));
    })
    .unwrap();
    url
}

#[test]
fn a_track_on_a_folder_with_no_public_address_gets_a_placeholder_address() {
    let mut s = setup();
    let album = new_album(&mut s, "my-album.xml");
    let url = add_track(&mut s, album, "my-album");
    assert_eq!(url, "pending:my-album/01 Song.mp3");
    let (feed, _) = pipeline::draft_model(&s.collection, album).unwrap();
    assert!(
        feed.bad_addresses().is_empty(),
        "a placeholder is no bad address"
    );

    write_new(&mut s, album);
    assert!(s.dir.path().join("private/my-album/01 Song.mp3").exists());
}

#[test]
fn a_move_to_a_public_host_changes_each_placeholder_address_to_its_public_address() {
    let mut s = setup();
    let album = new_album(&mut s, "album.xml");
    add_track(&mut s, album, "album");
    write_new(&mut s, album);
    start(&s, album);

    // The OP3 flag is read from the draft: it is on only when each enclosure
    // has the prefix. A placeholder track therefore has no OP3 setting here.
    pipeline::set_feed_host(&s.collection, &s.hosts, album, s.public).unwrap();
    let plain = "https://music.example/album/01%20Song.mp3";
    let (feed, _) = pipeline::draft_model(&s.collection, album).unwrap();
    let url = feed.items[0]
        .value
        .enclosure
        .as_ref()
        .and_then(|enclosure| enclosure.url.clone())
        .unwrap();
    assert_eq!(url, plain);

    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, album).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        album,
        &diff.id,
    )
    .unwrap();
    assert!(s.dir.path().join("public/album/01 Song.mp3").exists());
}

#[test]
fn a_track_imported_after_a_move_between_public_folders_is_uploaded_to_its_address() {
    let mut s = setup();
    let second_root = s.dir.path().join("second");
    fs::create_dir_all(&second_root).unwrap();
    let second_folder = LocalFolder::new(second_root, Some("https://second.example".into()));
    let second = s
        .collection
        .add_connector(LocalFolder::KIND, "Second", &second_folder.config(), None)
        .unwrap();
    s.hosts.insert(second, Box::new(second_folder));

    let album = pipeline::new_feed(
        &mut s.collection,
        s.hosts.get(s.public).unwrap(),
        s.public,
        "album.xml",
        "Album",
        "Artist",
        "music",
    )
    .unwrap();
    write_new(&mut s, album);
    let public = s.hosts.get(s.public).unwrap();
    pipeline::start_draft(&s.collection, public, album).unwrap();
    pipeline::set_feed_host(&s.collection, &s.hosts, album, second).unwrap();

    // The track is imported after the move, so its address is on the first host.
    let url = add_track(&mut s, album, "album");
    assert_eq!(url, "https://music.example/album/01%20Song.mp3");

    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, album).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        album,
        &diff.id,
    )
    .unwrap();
    assert!(s.dir.path().join("public/album/01 Song.mp3").exists());
    assert!(s.dir.path().join("second/album.xml").exists());
}

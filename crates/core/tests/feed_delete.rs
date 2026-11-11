//! The delete of a feed from its file host (ADR 0131 items 11 to 14), over
//! the local folder connector. No test sends a request outside the computer
//! (ADR 0045).

use std::fs;

use studio_core::collection::{Collection, ConnectorId, FeedId};
use studio_core::connector::{self, Connector, Expected, LocalFolder};
use studio_core::feed::{Enclosure, Entry, Item};
use studio_core::feed_delete::{self, StayReason};
use studio_core::inventory::{self, Hosts};
use studio_core::library::Library;
use studio_core::move_link::ScriptedReader;
use studio_core::pipeline::{self, AddressCheck, DiffId, PipelineErrorKind};
use studio_core::publisher;

/// An address check that passes each address.
struct PassAll;

impl AddressCheck for PassAll {
    fn check(&self, _url: &str) -> Result<(), String> {
        Ok(())
    }
}

struct Env {
    dir: tempfile::TempDir,
    collection: Collection,
    connector: Box<dyn Connector>,
    connector_id: ConnectorId,
    hosts: Hosts,
    library: Library,
}

fn env() -> Env {
    let dir = tempfile::tempdir().unwrap();
    let folder = LocalFolder::new(dir.path(), Some("https://music.example".into()));
    let collection = Collection::open(&dir.path().join("collection.sqlite")).unwrap();
    let connector_id = collection
        .add_connector(LocalFolder::KIND, "Home folder", &folder.config(), None)
        .unwrap();
    let connector = connector::from_row(
        &collection.connector(connector_id).unwrap(),
        None,
        &studio_core::net::NetPolicy::Open,
    )
    .unwrap();
    let mut hosts = Hosts::new();
    hosts.insert(connector_id, Box::new(folder));
    let library = Library::open(dir.path().join("library")).unwrap();
    Env {
        dir,
        collection,
        connector,
        connector_id,
        hosts,
        library,
    }
}

fn url(path: &str) -> String {
    format!("https://music.example/{path}")
}

fn put(e: &Env, path: &str, bytes: &[u8]) {
    e.connector.write(path, bytes, &Expected::Absent).unwrap();
}

fn write(e: &mut Env, feed: FeedId, id: &DiffId) -> Result<(), pipeline::PipelineError> {
    pipeline::write_draft(
        &mut e.collection,
        &e.hosts,
        &e.library,
        &PassAll,
        &ScriptedReader::empty(),
        feed,
        id,
    )
    .map(|_| ())
}

/// A feed on the host, with a release folder, an image and one track. The
/// write has put it on the host, and the library has its last copy.
fn hosted_feed(
    e: &mut Env,
    file: &str,
    title: &str,
    medium: &str,
    image: &str,
    audio: &str,
) -> FeedId {
    let feed = pipeline::new_feed(
        &mut e.collection,
        e.connector.as_ref(),
        e.connector_id,
        file,
        title,
        "Artist",
        medium,
    )
    .unwrap();
    e.collection
        .set_feed_folder(feed, file.trim_end_matches(".xml"))
        .unwrap();
    let (image, audio) = (url(image), url(audio));
    pipeline::edit_draft(&e.collection, feed, move |f| {
        f.image = Some(image);
        f.items.push(Entry::new(Item {
            title: Some("Track".to_owned()),
            enclosure: Some(Enclosure {
                url: Some(audio),
                length: Some("5".to_owned()),
                media_type: Some("audio/mpeg".to_owned()),
            }),
            ..Item::default()
        }));
    })
    .unwrap();
    let diff = pipeline::diff(&e.collection, &e.hosts, &e.library, feed).unwrap();
    write(e, feed, &diff.id).unwrap();
    feed
}

/// The setup of most tests: one feed with its two files on the host.
fn setup() -> (Env, FeedId) {
    let mut e = env();
    put(&e, "files/cover.jpg", b"cover");
    put(&e, "files/one.mp3", b"audio");
    inventory::scan(&e.collection, &e.hosts, e.connector_id).unwrap();
    let feed = hosted_feed(
        &mut e,
        "album.xml",
        "Album",
        "music",
        "files/cover.jpg",
        "files/one.mp3",
    );
    (e, feed)
}

fn exists(e: &Env, path: &str) -> bool {
    e.dir.path().join(path).exists()
}

#[test]
fn a_delete_needs_an_approved_diff_keeps_a_snapshot_and_removes_only_the_feed_file() {
    let (mut e, feed) = setup();
    let snapshots = e.collection.snapshots(feed).unwrap().len();
    let draft =
        feed_delete::start_delete_draft(&e.collection, &e.hosts, &e.library, feed, false).unwrap();
    let paths: Vec<_> = draft
        .summary
        .deletes
        .iter()
        .map(|op| op.path.as_str())
        .collect();
    assert_eq!(paths, ["album.xml"], "ADR 0131 item 11");
    assert_eq!(draft.summary.stays.len(), 0);

    // ADR 0131 item 12: a diff that the user did not approve writes nothing.
    let error = write(&mut e, feed, &DiffId::new("not approved")).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::DiffNotApproved);
    assert!(exists(&e, "album.xml"));

    write(&mut e, feed, &draft.diff.id).unwrap();
    assert!(!exists(&e, "album.xml"));
    assert!(exists(&e, "files/cover.jpg"));
    assert!(exists(&e, "files/one.mp3"));
    assert_eq!(e.collection.snapshots(feed).unwrap().len(), snapshots + 1);
    assert!(e.collection.draft(feed).unwrap().is_none());
    assert!(!e.collection.has_draft_feed_delete(feed).unwrap());
}

#[test]
fn a_feed_file_that_changed_on_the_host_after_the_draft_stops_the_delete() {
    let (mut e, feed) = setup();
    let draft =
        feed_delete::start_delete_draft(&e.collection, &e.hosts, &e.library, feed, false).unwrap();
    let live = fs::read_to_string(e.dir.path().join("album.xml")).unwrap();
    fs::write(
        e.dir.path().join("album.xml"),
        live.replace("Album", "Album (changed outside)"),
    )
    .unwrap();
    let error = write(&mut e, feed, &draft.diff.id).unwrap_err();
    assert_eq!(
        error.kind(),
        PipelineErrorKind::Conflict,
        "ADR 0131 item 12"
    );
    assert!(exists(&e, "album.xml"));
    assert!(e.collection.feed(feed).unwrap().connector_id.is_some());
}

#[test]
fn the_files_option_deletes_its_files_and_keeps_a_file_that_a_different_feed_uses() {
    let (mut e, feed) = setup();
    // A second feed uses the cover of the first.
    hosted_feed(
        &mut e,
        "second.xml",
        "Second Album",
        "music",
        "files/cover.jpg",
        "files/two.mp3",
    );
    let draft =
        feed_delete::start_delete_draft(&e.collection, &e.hosts, &e.library, feed, true).unwrap();
    let paths: Vec<_> = draft
        .summary
        .deletes
        .iter()
        .map(|op| op.path.as_str())
        .collect();
    assert_eq!(
        paths,
        ["files/one.mp3", "album.xml"],
        "the feed file is last"
    );
    assert_eq!(draft.summary.stays.len(), 1);
    assert_eq!(draft.summary.stays[0].path, "files/cover.jpg");
    assert_eq!(
        draft.summary.stays[0].reason,
        StayReason::UsedByFeeds(vec!["Second Album".to_owned()]),
        "ADR 0131 item 11"
    );

    write(&mut e, feed, &draft.diff.id).unwrap();
    assert!(!exists(&e, "files/one.mp3"));
    assert!(!exists(&e, "album.xml"));
    assert!(exists(&e, "files/cover.jpg"), "a different feed uses it");
    assert!(exists(&e, "second.xml"));
}

#[test]
fn a_pointer_file_of_the_feed_stays_and_the_feed_file_cannot_be_a_pointer_file() {
    let (mut e, feed) = setup();
    put(&e, "old.xml", b"<rss/>");
    inventory::scan(&e.collection, &e.hosts, e.connector_id).unwrap();
    e.collection
        .add_feed_pointer(e.connector_id, "old.xml", feed)
        .unwrap();
    let draft =
        feed_delete::start_delete_draft(&e.collection, &e.hosts, &e.library, feed, true).unwrap();
    assert_eq!(draft.summary.stays.len(), 1);
    assert_eq!(draft.summary.stays[0].path, "old.xml");
    assert_eq!(
        draft.summary.stays[0].reason,
        StayReason::PointerFile,
        "ADR 0038 item 20"
    );
    write(&mut e, feed, &draft.diff.id).unwrap();
    assert!(exists(&e, "old.xml"), "Studio keeps a pointer file");
    assert!(!exists(&e, "album.xml"));

    // ADR 0131 item 13: the feed file itself is never a pointer file.
    let (e, feed) = setup();
    e.collection
        .add_feed_pointer(e.connector_id, "album.xml", feed)
        .unwrap();
    let error = feed_delete::start_delete_draft(&e.collection, &e.hosts, &e.library, feed, false)
        .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::InvalidOperation);
    assert!(exists(&e, "album.xml"));
    assert!(!e.collection.has_draft_feed_delete(feed).unwrap());
}

#[test]
fn after_the_delete_the_feed_is_read_only_and_a_write_to_the_same_address_brings_the_file_back() {
    let (mut e, feed) = setup();
    let before = fs::read(e.dir.path().join("album.xml")).unwrap();
    let draft =
        feed_delete::start_delete_draft(&e.collection, &e.hosts, &e.library, feed, false).unwrap();
    write(&mut e, feed, &draft.diff.id).unwrap();

    // ADR 0131 item 14: read-only, with its last copy.
    let row = e.collection.feed(feed).unwrap();
    assert_eq!(row.connector_id, None);
    assert_eq!(row.title.as_deref(), Some("Album"));
    assert!(row.folder.is_some());
    let error = pipeline::start_draft(&e.collection, e.connector.as_ref(), feed).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::ReadOnly);

    let restore =
        feed_delete::start_restore_draft(&e.collection, &e.hosts, &e.library, feed).unwrap();
    assert_eq!(restore.draft_xml, before);
    let diff = pipeline::diff(&e.collection, &e.hosts, &e.library, feed).unwrap();
    write(&mut e, feed, &diff.id).unwrap();
    assert_eq!(fs::read(e.dir.path().join("album.xml")).unwrap(), before);
    let row = e.collection.feed(feed).unwrap();
    assert_eq!(row.connector_id, Some(e.connector_id));
    assert_eq!(row.path.as_deref(), Some("album.xml"));
    assert!(e.collection.feed_host_delete(feed).unwrap().is_none());
}

#[test]
fn a_restore_stops_when_a_file_is_at_the_path_and_a_feed_with_no_delete_has_none() {
    let (mut e, feed) = setup();
    let error =
        feed_delete::start_restore_draft(&e.collection, &e.hosts, &e.library, feed).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::NotFound);
    let draft =
        feed_delete::start_delete_draft(&e.collection, &e.hosts, &e.library, feed, false).unwrap();
    write(&mut e, feed, &draft.diff.id).unwrap();
    put(&e, "album.xml", b"<rss/>");
    let error =
        feed_delete::start_restore_draft(&e.collection, &e.hosts, &e.library, feed).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::FileExists);
}

#[test]
fn a_delete_draft_has_no_announcement_and_a_discard_removes_its_mark() {
    let (e, feed) = setup();
    let draft =
        feed_delete::start_delete_draft(&e.collection, &e.hosts, &e.library, feed, true).unwrap();
    assert_eq!(draft.diff.announcements.len(), 0);
    assert!(pipeline::draft_changed(&e.collection, feed).unwrap());
    pipeline::discard_draft(&e.collection, feed).unwrap();
    assert!(!e.collection.has_draft_feed_delete(feed).unwrap());
    assert_eq!(e.collection.draft_operations(feed).unwrap().len(), 0);
    assert!(exists(&e, "album.xml"));
}

#[test]
fn each_publisher_feed_that_links_to_the_feed_is_listed() {
    let (mut e, feed) = setup();
    let label = pipeline::new_feed(
        &mut e.collection,
        e.connector.as_ref(),
        e.connector_id,
        "label.xml",
        "Label",
        "Artist",
        "publisher",
    )
    .unwrap();
    let other = pipeline::new_feed(
        &mut e.collection,
        e.connector.as_ref(),
        e.connector_id,
        "other.xml",
        "Other Label",
        "Artist",
        "publisher",
    )
    .unwrap();
    assert_eq!(
        publisher::publishers_linking(&e.collection, &e.library, feed)
            .unwrap()
            .len(),
        0
    );
    // The release feed has a draft only while it is edited, so start one.
    pipeline::start_draft(&e.collection, e.connector.as_ref(), feed).unwrap();
    e.collection
        .set_publisher_roles(
            label,
            &[studio_core::collection::PublisherRole::Label],
            true,
        )
        .unwrap();
    publisher::link(&e.collection, label, feed).unwrap();
    assert_eq!(
        publisher::publishers_linking(&e.collection, &e.library, feed).unwrap(),
        [label],
        "ADR 0131 item 13"
    );
    assert!(
        !publisher::publishers_linking(&e.collection, &e.library, feed)
            .unwrap()
            .contains(&other)
    );
}

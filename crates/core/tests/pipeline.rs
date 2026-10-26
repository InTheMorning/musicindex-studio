//! The write pipeline over the local folder connector (ADR 0007, ADR 0015).

use std::fs;
use std::io::Read;
use std::sync::{Arc, Mutex};

use studio_core::collection::{Collection, ConnectorId, DraftOperationKind, FeedId, LibraryFileId};
use studio_core::connector::{
    self, Connector, ConnectorError, ConnectorErrorKind, Expected, FileList, FileRead, HostChanges,
    KeptBlob, ListedFile, LocalFolder, NsiteHost, PlannedEvent, Stamp, Version,
};
use studio_core::feed::{Enclosure, Entry, Feed, Item};
use studio_core::guid::podcast_guid_for_url;
use studio_core::inventory::{self, Hosts};
use studio_core::library::Library;
use studio_core::nostr_events::Signing;
use studio_core::pipeline::{
    self, AddressCheck, DiffFile, DiffId, OperationKind, PipelineErrorKind,
};
use studio_core::testing::blossom::TestBlossom;
use studio_core::testing::relay::TestRelay;
use studio_core::xml::Document;

const FEED: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<rss version=\"2.0\" xmlns:podcast=\"https://podcastindex.org/namespace/1.0\">
  <channel>
    <title>Album</title>
    <podcast:guid>917393e3-1b1e-5cef-ace4-edaa54e1f810</podcast:guid>
    <podcast:medium>music</podcast:medium>
    <item>
      <title>Track One</title>
      <guid>track-1</guid>
    </item>
  </channel>
</rss>
";

/// An address check that passes each address. No test sends a request to
/// the network.
struct PassAll;

impl AddressCheck for PassAll {
    fn check(&self, _url: &str) -> Result<(), String> {
        Ok(())
    }
}

struct Setup {
    dir: tempfile::TempDir,
    collection: Collection,
    connector: Box<dyn Connector>,
    hosts: Hosts,
    library: Library,
    feed: FeedId,
}

fn setup() -> Setup {
    let dir = tempfile::tempdir().unwrap();
    let folder = LocalFolder::new(dir.path(), Some("https://music.example".into()));
    folder
        .write("album.xml", FEED.as_bytes(), &Expected::Absent)
        .unwrap();
    let mut collection = Collection::open(&dir.path().join("collection.sqlite")).unwrap();
    let connector_id = collection
        .add_connector(LocalFolder::KIND, "Home folder", &folder.config(), None)
        .unwrap();
    let connector =
        connector::from_row(&collection.connector(connector_id).unwrap(), None).unwrap();
    let mut hosts = Hosts::new();
    hosts.insert(connector_id, Box::new(folder));
    let library = Library::open(dir.path().join("library")).unwrap();
    let feed = collection
        .add_feed(
            Some("https://music.example/album.xml"),
            Some((connector_id, "album.xml")),
        )
        .unwrap();
    pipeline::refresh(&mut collection, connector.as_ref(), feed).unwrap();
    Setup {
        dir,
        collection,
        connector,
        hosts,
        library,
        feed,
    }
}

fn live(s: &Setup) -> String {
    fs::read_to_string(s.dir.path().join("album.xml")).unwrap()
}

#[test]
fn an_approved_draft_writes_the_file_and_keeps_a_snapshot() {
    let mut s = setup();
    assert_eq!(s.collection.item_guids(s.feed).unwrap(), ["track-1"]);
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    pipeline::edit_draft(&s.collection, s.feed, |f| {
        f.title = Some("Album (Deluxe)".into());
    })
    .unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert!(diff.changed);
    assert!(diff.unified.contains("-    <title>Album</title>"));
    assert!(diff.unified.contains("+    <title>Album (Deluxe)</title>"));

    let version = pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();
    assert_eq!(
        live(&s),
        FEED.replace("<title>Album</title>", "<title>Album (Deluxe)</title>")
    );
    let row = s.collection.feed(s.feed).unwrap();
    assert_eq!(row.last_version, Some(version));
    assert_eq!(row.title.as_deref(), Some("Album (Deluxe)"));
    assert!(s.collection.draft(s.feed).unwrap().is_none());
    let snapshots = s.collection.snapshots(s.feed).unwrap();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(
        s.collection.snapshot_xml(s.feed, snapshots[0].id).unwrap(),
        FEED.as_bytes()
    );
}

#[test]
fn a_draft_that_changed_after_the_approval_does_not_write() {
    let mut s = setup();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    pipeline::edit_draft(&s.collection, s.feed, |f| f.title = Some("One".into())).unwrap();
    let approved = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed)
        .unwrap()
        .id;
    pipeline::edit_draft(&s.collection, s.feed, |f| f.title = Some("Two".into())).unwrap();
    let error = pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &approved,
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::DiffNotApproved);
    let error = pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &DiffId::new("x"),
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::DiffNotApproved);
    assert_eq!(live(&s), FEED);
    assert_eq!(s.collection.snapshots(s.feed).unwrap(), Vec::new());
}

#[test]
fn a_file_that_changed_on_the_host_after_the_draft_started_does_not_write() {
    let mut s = setup();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    pipeline::edit_draft(&s.collection, s.feed, |f| f.title = Some("Mine".into())).unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    let theirs = FEED.replace("Track One", "Track One (edited elsewhere)");
    fs::write(s.dir.path().join("album.xml"), &theirs).unwrap();
    let error = pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::Conflict);
    assert_eq!(live(&s), theirs);
    assert_eq!(s.collection.snapshots(s.feed).unwrap(), Vec::new());
    assert!(s.collection.draft(s.feed).unwrap().is_some());
}

#[test]
fn a_rollback_is_a_draft_from_a_snapshot_with_its_own_diff() {
    let mut s = setup();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    pipeline::edit_draft(&s.collection, s.feed, |f| f.items.clear()).unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();
    assert!(!live(&s).contains("track-1"));
    assert_eq!(
        s.collection.item_guids(s.feed).unwrap(),
        Vec::<String>::new()
    );

    let snapshot = s.collection.snapshots(s.feed).unwrap()[0].id;
    pipeline::draft_from_snapshot(&s.collection, s.connector.as_ref(), s.feed, snapshot).unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert!(diff.unified.contains("+      <guid>track-1</guid>"));
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();
    assert_eq!(live(&s), FEED);
    assert_eq!(s.collection.item_guids(s.feed).unwrap(), ["track-1"]);
    assert_eq!(s.collection.snapshots(s.feed).unwrap().len(), 2);
}

#[test]
fn a_feed_without_a_connector_is_read_only() {
    let mut s = setup();
    let read_only = s
        .collection
        .add_feed(Some("https://elsewhere.example/feed.xml"), None)
        .unwrap();
    let error = pipeline::start_draft(&s.collection, s.connector.as_ref(), read_only).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::ReadOnly);
}

/// A connector and a collection with no feed yet, for the new-feed tests
/// (ADR 0032 items 6 to 9).
fn new_setup() -> (
    tempfile::TempDir,
    Collection,
    Box<dyn Connector>,
    ConnectorId,
    Hosts,
    Library,
) {
    let dir = tempfile::tempdir().unwrap();
    let folder = LocalFolder::new(dir.path(), Some("https://music.example".into()));
    let collection = Collection::open(&dir.path().join("collection.sqlite")).unwrap();
    let connector_id = collection
        .add_connector(LocalFolder::KIND, "Home folder", &folder.config(), None)
        .unwrap();
    let connector =
        connector::from_row(&collection.connector(connector_id).unwrap(), None).unwrap();
    let mut hosts = Hosts::new();
    hosts.insert(connector_id, Box::new(folder));
    let library = Library::open(dir.path().join("library")).unwrap();
    (dir, collection, connector, connector_id, hosts, library)
}

#[test]
fn a_new_publisher_feed_has_the_publisher_medium_in_its_draft_and_row() {
    let (_dir, mut collection, connector, connector_id, _hosts, _library) = new_setup();
    let id = pipeline::new_feed(
        &mut collection,
        connector.as_ref(),
        connector_id,
        "label.xml",
        "New Label",
        "New Label",
        "publisher",
    )
    .unwrap();
    let (model, _revision) = pipeline::draft_model(&collection, id).unwrap();
    assert_eq!(model.medium.as_deref(), Some("publisher"));
    assert_eq!(
        collection.feed(id).unwrap().medium.as_deref(),
        Some("publisher")
    );
}

#[test]
fn a_new_feed_makes_a_draft_with_no_hosted_file() {
    let (dir, mut collection, connector, connector_id, _hosts, _library) = new_setup();
    let id = pipeline::new_feed(
        &mut collection,
        connector.as_ref(),
        connector_id,
        "my-album.xml",
        "New Album",
        "New Artist",
        "music",
    )
    .unwrap();
    // The folder has no file before the write.
    assert!(!dir.path().join("my-album.xml").exists());

    let (model, _revision) = pipeline::draft_model(&collection, id).unwrap();
    assert_eq!(model.title.as_deref(), Some("New Album"));
    assert_eq!(model.author.as_deref(), Some("New Artist"));
    assert_eq!(
        model.podcast_guid.as_deref(),
        Some(podcast_guid_for_url("https://music.example/my-album.xml").as_str())
    );
    assert_eq!(model.medium.as_deref(), Some("music"));
    assert_eq!(model.language.as_deref(), Some("en"));

    let row = collection.feed(id).unwrap();
    assert_eq!(
        row.public_url.as_deref(),
        Some("https://music.example/my-album.xml")
    );
    assert_eq!(row.connector_id, Some(connector_id));
    assert_eq!(row.path.as_deref(), Some("my-album.xml"));

    // On 2026-10-23 the review of task 017 found a new file on one line.
    // A later diff then showed the full file for each edit. The file has
    // the XML declaration, and each channel element is on its own line.
    let draft = collection.draft(id).unwrap().unwrap();
    let xml = String::from_utf8(draft.draft_xml).unwrap();
    assert!(
        xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"),
        "{xml}"
    );
    assert!(
        xml.contains("\n    <channel>\n        <title>New Album</title>\n"),
        "{xml}"
    );
}

#[test]
fn a_new_feed_in_a_folder_with_no_public_address_gives_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let folder = LocalFolder::new(dir.path(), None);
    let mut collection = Collection::open(&dir.path().join("collection.sqlite")).unwrap();
    let connector_id = collection
        .add_connector(LocalFolder::KIND, "Home folder", &folder.config(), None)
        .unwrap();
    let error = pipeline::new_feed(
        &mut collection,
        &folder,
        connector_id,
        "my-album.xml",
        "New Album",
        "New Artist",
        "music",
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::NoPublicAddress);
}

#[test]
fn a_new_feed_at_a_path_with_a_file_gives_an_error() {
    let (dir, mut collection, connector, connector_id, _hosts, _library) = new_setup();
    fs::write(dir.path().join("taken.xml"), b"<rss/>").unwrap();
    let error = pipeline::new_feed(
        &mut collection,
        connector.as_ref(),
        connector_id,
        "taken.xml",
        "New Album",
        "New Artist",
        "music",
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::FileExists);
}

#[test]
fn the_diff_of_a_new_feed_adds_the_full_file_and_the_write_puts_it() {
    let (dir, mut collection, connector, connector_id, hosts, library) = new_setup();
    let id = pipeline::new_feed(
        &mut collection,
        connector.as_ref(),
        connector_id,
        "my-album.xml",
        "New Album",
        "New Artist",
        "music",
    )
    .unwrap();
    let diff = pipeline::diff(&collection, &hosts, &library, id).unwrap();
    assert!(diff.changed);
    // The live file is absent, so the diff adds the full file and removes
    // nothing.
    assert!(diff.unified.contains("+<rss"));
    assert!(!diff.unified.contains("\n-<"));

    let version =
        pipeline::write_draft(&mut collection, &hosts, &library, &PassAll, id, &diff.id).unwrap();
    let bytes = fs::read(dir.path().join("my-album.xml")).unwrap();
    let read_feed = Feed::read(&Document::parse_bytes(&bytes).unwrap()).unwrap();
    assert_eq!(read_feed.title.as_deref(), Some("New Album"));
    assert_eq!(read_feed.author.as_deref(), Some("New Artist"));
    let row = collection.feed(id).unwrap();
    assert_eq!(row.last_version, Some(version));
    assert!(collection.draft(id).unwrap().is_none());
}

#[test]
fn a_file_put_by_another_tool_before_the_write_of_a_new_feed_gives_a_conflict() {
    let (dir, mut collection, connector, connector_id, hosts, library) = new_setup();
    let id = pipeline::new_feed(
        &mut collection,
        connector.as_ref(),
        connector_id,
        "my-album.xml",
        "New Album",
        "New Artist",
        "music",
    )
    .unwrap();
    let diff = pipeline::diff(&collection, &hosts, &library, id).unwrap();
    // A different tool puts a file at the path before Studio writes it.
    fs::write(
        dir.path().join("my-album.xml"),
        b"<rss><channel><title>Other</title></channel></rss>",
    )
    .unwrap();
    let error = pipeline::write_draft(&mut collection, &hosts, &library, &PassAll, id, &diff.id)
        .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::Conflict);
    assert!(collection.draft(id).unwrap().is_some());
}

#[test]
fn release_path_gives_the_file_next_to_the_feed_file_with_no_extension() {
    assert_eq!(
        pipeline::release_path("album/my-album.xml", "01 A.mp3"),
        "album/my-album/01 A.mp3"
    );
    assert_eq!(
        pipeline::release_path("my-album.xml", "a.mp3"),
        "my-album/a.mp3"
    );
}

#[test]
fn host_file_matches_gives_absent_same_and_different() {
    let dir = tempfile::tempdir().unwrap();
    let folder = LocalFolder::new(dir.path(), Some("https://music.example".into()));
    let local_dir = tempfile::tempdir().unwrap();
    let local = local_dir.path().join("a.mp3");
    fs::write(&local, b"music").unwrap();

    assert_eq!(
        pipeline::host_file_matches(&folder, "album/a.mp3", &local).unwrap(),
        pipeline::HostFile::Absent
    );

    folder
        .write("album/a.mp3", b"music", &Expected::Absent)
        .unwrap();
    assert_eq!(
        pipeline::host_file_matches(&folder, "album/a.mp3", &local).unwrap(),
        pipeline::HostFile::Same
    );

    fs::write(&local, b"different").unwrap();
    assert_eq!(
        pipeline::host_file_matches(&folder, "album/a.mp3", &local).unwrap(),
        pipeline::HostFile::Different
    );
}

// Task 021e removes the pending-file store and the `draft_files` table:
// the import route already writes directly into the release folder
// (task 021d), and `migrate::into_library` moves each pending file of a
// collection from before the library. The tests of that move are in
// `studio_core::migrate`.

// ---- The import into the library, and host operations (ADR 0036) ----

/// Import a file into the release folder of the feed of `s`. The caller
/// gives the feed a release folder first.
fn import_file(s: &Setup, name: &str, bytes: &[u8]) -> pipeline::ImportedFile {
    let source_dir = tempfile::tempdir().unwrap();
    let source = source_dir.path().join("source");
    fs::write(&source, bytes).unwrap();
    pipeline::import_into_library(
        &s.collection,
        &s.library,
        &s.hosts,
        s.feed,
        &source,
        name,
        "audio/mpeg",
    )
    .unwrap()
}

/// Give the draft of `s` an item whose enclosure is at the current
/// planned address of a library file, so the draft refers to it
/// (ADR 0036 item 10). Gives the planned path.
fn reference_planned(s: &Setup, file_id: LibraryFileId) -> inventory::PlannedPath {
    let feed_row = s.collection.feed(s.feed).unwrap();
    let file = s.collection.library_file(file_id).unwrap();
    let planned = inventory::planned_path(&s.collection, &s.hosts, &feed_row, &file).unwrap();
    let url = planned.url.clone().unwrap();
    let item = Item {
        title: Some(file.path.clone()),
        enclosure: Some(Enclosure {
            url: Some(url),
            length: Some(file.size.to_string()),
            media_type: Some(file.media_type.clone()),
        }),
        ..Item::default()
    };
    pipeline::edit_draft(&s.collection, s.feed, move |f| {
        f.items.push(Entry::new(item));
    })
    .unwrap();
    planned
}

#[test]
fn an_import_puts_the_file_in_the_release_folder_and_repeats_on_the_same_row() {
    let s = setup();
    s.collection.set_feed_folder(s.feed, "album").unwrap();

    let source_dir = tempfile::tempdir().unwrap();
    let source = source_dir.path().join("01.mp3");
    fs::write(&source, b"one").unwrap();
    let imported = pipeline::import_into_library(
        &s.collection,
        &s.library,
        &s.hosts,
        s.feed,
        &source,
        "01.mp3",
        "audio/mpeg",
    )
    .unwrap();
    assert_eq!(
        imported.planned.url.as_deref(),
        Some("https://music.example/album/01.mp3")
    );
    assert!(!source.exists(), "the import moves the source file");
    assert_eq!(fs::read(s.library.path("album", "01.mp3")).unwrap(), b"one");
    let file = s.collection.library_file(imported.library_file_id).unwrap();
    assert_eq!(file.size, 3);
    assert!(file.local);

    // An import of the same bytes at the same name again gives the same
    // row, with no second copy (ADR 0036 item 5).
    let source_2 = source_dir.path().join("01-again.mp3");
    fs::write(&source_2, b"one").unwrap();
    let imported_again = pipeline::import_into_library(
        &s.collection,
        &s.library,
        &s.hosts,
        s.feed,
        &source_2,
        "01.mp3",
        "audio/mpeg",
    )
    .unwrap();
    assert_eq!(imported_again.library_file_id, imported.library_file_id);
    assert_eq!(s.collection.library_files(s.feed).unwrap().len(), 1);
}

#[test]
fn an_import_without_a_release_folder_gives_no_folder() {
    let s = setup();
    let source_dir = tempfile::tempdir().unwrap();
    let source = source_dir.path().join("01.mp3");
    fs::write(&source, b"one").unwrap();
    let error = pipeline::import_into_library(
        &s.collection,
        &s.library,
        &s.hosts,
        s.feed,
        &source,
        "01.mp3",
        "audio/mpeg",
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::NoFolder);
}

#[test]
fn an_import_of_a_different_file_at_a_name_in_use_gives_file_exists() {
    let s = setup();
    s.collection.set_feed_folder(s.feed, "album").unwrap();
    import_file(&s, "01.mp3", b"one");

    let source_dir = tempfile::tempdir().unwrap();
    let second = source_dir.path().join("b.mp3");
    fs::write(&second, b"a different recording").unwrap();
    let error = pipeline::import_into_library(
        &s.collection,
        &s.library,
        &s.hosts,
        s.feed,
        &second,
        "01.mp3",
        "audio/mpeg",
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::FileExists);
    assert!(second.exists(), "a refused import does not move the source");
}

#[test]
fn a_draft_that_refers_to_an_imported_file_gives_one_upload_until_it_is_on_the_host() {
    let s = setup();
    s.collection.set_feed_folder(s.feed, "album").unwrap();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();

    let imported = import_file(&s, "01.mp3", b"one");
    let planned = reference_planned(&s, imported.library_file_id);
    assert_eq!(planned, imported.planned);

    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert_eq!(diff.operations.len(), 1);
    let upload = &diff.operations[0];
    assert_eq!(upload.kind, OperationKind::Upload);
    assert_eq!(upload.library_file_id, Some(imported.library_file_id.0));
    assert_eq!(upload.path, planned.path);
    assert_eq!(upload.url, planned.url);
    assert_eq!(upload.size, 3);
    assert_eq!(
        diff.files,
        [DiffFile {
            path: "01.mp3".to_owned(),
            size: 3,
            media_type: "audio/mpeg".to_owned(),
        }]
    );

    // The file goes on the host with the same bytes, and a scan records
    // it. The diff then gives no upload (ADR 0036 item 10).
    s.connector
        .write(&planned.path, b"one", &Expected::Absent)
        .unwrap();
    inventory::scan(&s.collection, &s.hosts, planned.connector_id).unwrap();

    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert_eq!(diff.operations, Vec::new());
    assert_eq!(diff.files, Vec::new());
}

#[test]
fn a_library_file_with_its_own_host_gives_an_upload_there_with_its_url() {
    let mut s = setup();
    s.collection.set_feed_folder(s.feed, "album").unwrap();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();

    let second_dir = tempfile::tempdir().unwrap();
    let second_folder = LocalFolder::new(second_dir.path(), Some("https://second.example".into()));
    let second_id = s
        .collection
        .add_connector(LocalFolder::KIND, "Second", &second_folder.config(), None)
        .unwrap();
    s.hosts.insert(second_id, Box::new(second_folder));

    let imported = import_file(&s, "01.mp3", b"one");
    s.collection
        .set_library_file_connector(imported.library_file_id, Some(second_id))
        .unwrap();
    let planned = reference_planned(&s, imported.library_file_id);
    assert_eq!(planned.connector_id, second_id);
    assert_eq!(
        planned.url.as_deref(),
        Some("https://second.example/album/01.mp3")
    );

    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert_eq!(diff.operations.len(), 1);
    let upload = &diff.operations[0];
    assert_eq!(upload.kind, OperationKind::Upload);
    assert_eq!(upload.connector_id, second_id);
    assert_eq!(upload.url, planned.url);
}

/// A draft that refers to a copy of a library file on a different host
/// uses that copy. The file needs no upload to its own host
/// (ADR 0036 item 10).
#[test]
fn a_draft_that_refers_to_a_copy_gives_no_upload() {
    let mut s = setup();
    s.collection.set_feed_folder(s.feed, "album").unwrap();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();

    // A second host already has the bytes of the track, at a path that
    // is not where Studio plans to put the library file.
    let second_dir = tempfile::tempdir().unwrap();
    let second_folder = LocalFolder::new(second_dir.path(), Some("https://second.example".into()));
    second_folder
        .write("legacy/01.mp3", b"one", &Expected::Absent)
        .unwrap();
    let second_id = s
        .collection
        .add_connector(LocalFolder::KIND, "Second", &second_folder.config(), None)
        .unwrap();
    s.hosts.insert(second_id, Box::new(second_folder));
    inventory::scan(&s.collection, &s.hosts, second_id).unwrap();

    let imported = import_file(&s, "01.mp3", b"one");
    let copy_url = s.hosts.get(second_id).unwrap().public_url("legacy/01.mp3");
    let item = Item {
        title: Some("01.mp3".to_owned()),
        enclosure: Some(Enclosure {
            url: copy_url,
            length: Some("3".to_owned()),
            media_type: Some("audio/mpeg".to_owned()),
        }),
        ..Item::default()
    };
    pipeline::edit_draft(&s.collection, s.feed, move |f| {
        f.items.push(Entry::new(item));
    })
    .unwrap();

    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert!(diff.operations.is_empty(), "{:?}", diff.operations);
    assert_ne!(imported.planned.connector_id, second_id);
}

#[test]
fn a_copy_draft_operation_gives_a_copy_when_the_host_does_not_have_the_file() {
    let mut s = setup();
    s.collection.set_feed_folder(s.feed, "album").unwrap();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();

    let imported = import_file(&s, "01.mp3", b"one");

    let second_dir = tempfile::tempdir().unwrap();
    let second_folder = LocalFolder::new(second_dir.path(), Some("https://second.example".into()));
    let second_id = s
        .collection
        .add_connector(LocalFolder::KIND, "Second", &second_folder.config(), None)
        .unwrap();
    s.hosts.insert(second_id, Box::new(second_folder));
    s.collection
        .add_draft_operation(
            s.feed,
            DraftOperationKind::Copy,
            Some(imported.library_file_id),
            second_id,
            "01.mp3",
        )
        .unwrap();

    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    let copies: Vec<_> = diff
        .operations
        .iter()
        .filter(|op| op.kind == OperationKind::Copy)
        .collect();
    assert_eq!(copies.len(), 1);
    assert_eq!(copies[0].connector_id, second_id);
    assert_eq!(copies[0].path, "01.mp3");
    assert_eq!(copies[0].library_file_id, Some(imported.library_file_id.0));
    assert_eq!(
        copies[0].url.as_deref(),
        Some("https://second.example/01.mp3")
    );
}

#[test]
fn a_copy_of_a_remote_file_waits_for_its_download_and_the_write_refuses_it() {
    use std::sync::atomic::AtomicBool;
    use studio_core::download;
    use studio_core::web::Web;

    let mut s = setup();
    s.collection.set_feed_folder(s.feed, "album").unwrap();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();

    // The bytes are on the first host. The library has only a remote row.
    fs::create_dir_all(s.dir.path().join("album")).unwrap();
    fs::write(s.dir.path().join("album/01.mp3"), b"one").unwrap();
    let first = *s.hosts.ids().collect::<Vec<_>>().first().unwrap();
    inventory::scan(&s.collection, &s.hosts, first).unwrap();
    let id = s
        .collection
        .add_library_file(
            s.feed,
            "01.mp3",
            3,
            None,
            "audio/mpeg",
            false,
            Some("https://music.example/album/01.mp3"),
            None,
        )
        .unwrap();

    let second_dir = tempfile::tempdir().unwrap();
    let second_folder = LocalFolder::new(second_dir.path(), Some("https://second.example".into()));
    let second_id = s
        .collection
        .add_connector(LocalFolder::KIND, "Second", &second_folder.config(), None)
        .unwrap();
    s.hosts.insert(second_id, Box::new(second_folder));
    s.collection
        .add_draft_operation(
            s.feed,
            DraftOperationKind::Copy,
            Some(id),
            second_id,
            "01.mp3",
        )
        .unwrap();

    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    let copies: Vec<_> = diff
        .operations
        .iter()
        .filter(|op| op.kind == OperationKind::Copy)
        .collect();
    assert_eq!(copies.len(), 1);
    assert!(copies[0].waiting);

    let error = pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::WaitingForDownload);
    assert!(!second_dir.path().join("01.mp3").exists());
    assert!(!live(&s).contains("second.example"));

    // The download reads the file through the first host. The write then
    // puts the copy.
    let file = s.collection.library_file(id).unwrap();
    let source = download::source_of(&s.collection, &s.hosts, &file)
        .unwrap()
        .unwrap();
    let part = download::part_path(&s.library, "album", "01.mp3");
    download::fetch(
        &s.hosts,
        &Web::new(),
        &source,
        &part,
        u64::MAX,
        &mut |_, _| {},
        &AtomicBool::new(false),
    )
    .unwrap();
    let facts = download::hash_part(&part).unwrap();
    download::finish(&s.collection, &s.library, &file, &part, &facts).unwrap();

    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert!(diff.operations.iter().all(|op| !op.waiting));
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();
    assert_eq!(fs::read(second_dir.path().join("01.mp3")).unwrap(), b"one");
}

#[test]
fn a_delete_draft_operation_gives_a_delete_unless_the_draft_refers_to_the_file() {
    let s = setup();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    let connector_id = s.collection.feed(s.feed).unwrap().connector_id.unwrap();

    s.connector
        .write("stray.jpg", b"stray", &Expected::Absent)
        .unwrap();
    s.connector
        .write("cover.jpg", b"cover", &Expected::Absent)
        .unwrap();
    inventory::scan(&s.collection, &s.hosts, connector_id).unwrap();

    let cover_url = s.connector.public_url("cover.jpg").unwrap();
    pipeline::edit_draft(&s.collection, s.feed, move |f| f.image = Some(cover_url)).unwrap();

    s.collection
        .add_draft_operation(
            s.feed,
            DraftOperationKind::Delete,
            None,
            connector_id,
            "stray.jpg",
        )
        .unwrap();
    s.collection
        .add_draft_operation(
            s.feed,
            DraftOperationKind::Delete,
            None,
            connector_id,
            "cover.jpg",
        )
        .unwrap();

    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    let deletes: Vec<&str> = diff
        .operations
        .iter()
        .filter(|op| op.kind == OperationKind::Delete)
        .map(|op| op.path.as_str())
        .collect();
    assert_eq!(deletes, ["stray.jpg"]);
    assert_eq!(diff.blocked.len(), 1);
    assert_eq!(diff.blocked[0].path, "cover.jpg");
    assert_eq!(diff.blocked[0].feed_titles, ["Album".to_owned()]);
}

#[test]
fn a_remote_file_is_not_an_upload_in_the_diff() {
    let s = setup();
    s.collection.set_feed_folder(s.feed, "album").unwrap();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    // The planned address of this file is in the draft, but the file has
    // no bytes in the library.
    s.collection
        .add_library_file(
            s.feed,
            "01.mp3",
            0,
            None,
            "audio/mpeg",
            false,
            Some("https://music.example/album/01.mp3"),
            None,
        )
        .unwrap();
    pipeline::edit_draft(&s.collection, s.feed, |f| {
        f.items[0].value.enclosure = Some(Enclosure {
            url: Some("https://music.example/album/01.mp3".into()),
            length: Some("1".into()),
            media_type: Some("audio/mpeg".into()),
        });
    })
    .unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert_eq!(diff.operations, Vec::new());
    assert_eq!(diff.files, Vec::new());
}

#[test]
fn the_diff_id_changes_when_a_host_operation_changes() {
    let s = setup();
    s.collection.set_feed_folder(s.feed, "album").unwrap();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();

    let first = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed)
        .unwrap()
        .id;

    let imported = import_file(&s, "01.mp3", b"one");
    reference_planned(&s, imported.library_file_id);

    let second = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed)
        .unwrap()
        .id;
    assert_ne!(first, second);
}

// ---- The write of each host operation (ADR 0036 items 14 to 16) ----

#[test]
fn a_release_with_files_on_two_hosts_writes_both_with_the_correct_address() {
    let mut s = setup();
    s.collection.set_feed_folder(s.feed, "album").unwrap();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();

    // A second host, for the second track.
    let second_dir = tempfile::tempdir().unwrap();
    let second_folder = LocalFolder::new(second_dir.path(), Some("https://second.example".into()));
    let second_id = s
        .collection
        .add_connector(LocalFolder::KIND, "Second", &second_folder.config(), None)
        .unwrap();
    s.hosts.insert(second_id, Box::new(second_folder));

    let track_a = import_file(&s, "01.mp3", b"one");
    reference_planned(&s, track_a.library_file_id);

    let track_b = import_file(&s, "02.mp3", b"two");
    s.collection
        .set_library_file_connector(track_b.library_file_id, Some(second_id))
        .unwrap();
    let planned_b = reference_planned(&s, track_b.library_file_id);
    assert_eq!(planned_b.connector_id, second_id);
    assert_eq!(planned_b.path, "album/02.mp3");

    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert_eq!(diff.operations.len(), 2);

    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();

    assert_eq!(fs::read(s.dir.path().join("album/01.mp3")).unwrap(), b"one");
    assert_eq!(
        fs::read(second_dir.path().join("album/02.mp3")).unwrap(),
        b"two"
    );

    // The library XML has the same bytes as the hosted feed file.
    let hosted = live(&s);
    let library_xml = fs::read_to_string(s.library.path("album", "album.xml")).unwrap();
    assert_eq!(hosted, library_xml);

    let feed = Feed::read(&Document::parse(&hosted).unwrap()).unwrap();
    let urls: Vec<&str> = feed
        .items
        .iter()
        .filter_map(|item| item.value.enclosure.as_ref())
        .filter_map(|enclosure| enclosure.url.as_deref())
        .collect();
    assert!(urls.contains(&"https://music.example/album/01.mp3"));
    assert!(urls.contains(&"https://second.example/album/02.mp3"));
}

#[test]
fn a_changed_library_file_gets_a_hash_name_and_the_write_deletes_the_old_file_after_the_feed() {
    let mut s = setup();
    s.collection.set_feed_folder(s.feed, "album").unwrap();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();

    // The first write puts the track at its clean path, and the feed
    // refers to it there.
    let old = import_file(&s, "01.mp3", b"one");
    let planned_old = reference_planned(&s, old.library_file_id);
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();
    assert!(s.dir.path().join(&planned_old.path).exists());

    // The track changes: a new library file row takes the same path in
    // the release folder, with different bytes.
    s.collection
        .remove_library_file(old.library_file_id)
        .unwrap();
    let source_dir = tempfile::tempdir().unwrap();
    let new_source = source_dir.path().join("01-new.mp3");
    fs::write(&new_source, b"one (re-recorded)").unwrap();
    let (new_size, new_sha) = s.library.put_file("album", "01.mp3", &new_source).unwrap();
    let new_id = s
        .collection
        .add_library_file(
            s.feed,
            "01.mp3",
            new_size,
            Some(&new_sha),
            "audio/mpeg",
            true,
            None,
            None,
        )
        .unwrap();
    let feed_row = s.collection.feed(s.feed).unwrap();
    let new_file = s.collection.library_file(new_id).unwrap();
    let planned_new =
        inventory::planned_path(&s.collection, &s.hosts, &feed_row, &new_file).unwrap();
    // The clean path already has the old file, so Studio uses the hash
    // name of the new one instead.
    assert_ne!(planned_new.path, planned_old.path);
    assert!(planned_new.path.contains(&new_sha[..8]));

    // The draft now points its track at the new address, and asks for
    // the deletion of the old one.
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    let old_url = planned_old.url.clone().unwrap();
    let new_url = planned_new.url.clone().unwrap();
    pipeline::edit_draft(&s.collection, s.feed, move |f| {
        for item in &mut f.items {
            if let Some(enclosure) = item.value.enclosure.as_mut()
                && enclosure.url.as_deref() == Some(old_url.as_str())
            {
                enclosure.url = Some(new_url.clone());
            }
        }
    })
    .unwrap();
    s.collection
        .add_draft_operation(
            s.feed,
            DraftOperationKind::Delete,
            None,
            planned_old.connector_id,
            &planned_old.path,
        )
        .unwrap();

    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert_eq!(diff.operations.len(), 2);

    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();

    // The new file is on the host, at its hash name.
    assert_eq!(
        fs::read(s.dir.path().join(&planned_new.path)).unwrap(),
        b"one (re-recorded)"
    );
    // The write deleted the old file, after the feed was written with the
    // new address.
    assert!(!s.dir.path().join(&planned_old.path).exists());
    assert!(live(&s).contains(&planned_new.url.unwrap()));
}

#[test]
fn a_delete_for_a_file_that_a_different_feed_refers_to_does_not_delete_it() {
    let mut s = setup();
    let connector_id = s.collection.feed(s.feed).unwrap().connector_id.unwrap();

    // A shared cover file, already on the host.
    s.connector
        .write("shared/cover.jpg", b"cover", &Expected::Absent)
        .unwrap();
    inventory::scan(&s.collection, &s.hosts, connector_id).unwrap();
    let cover_url = s.connector.public_url("shared/cover.jpg").unwrap();

    // A second feed of the library refers to the shared file.
    let second = pipeline::new_feed(
        &mut s.collection,
        s.connector.as_ref(),
        connector_id,
        "second.xml",
        "Second Album",
        "Second Artist",
        "music",
    )
    .unwrap();
    s.collection.set_feed_folder(second, "second").unwrap();
    let cover_for_second = cover_url.clone();
    pipeline::edit_draft(&s.collection, second, move |f| {
        f.image = Some(cover_for_second);
    })
    .unwrap();
    let diff_second = pipeline::diff(&s.collection, &s.hosts, &s.library, second).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        second,
        &diff_second.id,
    )
    .unwrap();

    // The first feed drafts a deletion of the shared file.
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    s.collection
        .add_draft_operation(
            s.feed,
            DraftOperationKind::Delete,
            None,
            connector_id,
            "shared/cover.jpg",
        )
        .unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert_eq!(diff.operations, Vec::new());
    assert_eq!(diff.blocked.len(), 1);
    assert_eq!(diff.blocked[0].feed_titles, ["Second Album".to_owned()]);

    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();

    assert!(s.dir.path().join("shared/cover.jpg").exists());
}

#[test]
fn a_host_file_that_changed_after_the_scan_stops_the_deletion_with_conflict() {
    let mut s = setup();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    let connector_id = s.collection.feed(s.feed).unwrap().connector_id.unwrap();

    s.connector
        .write("stray.jpg", b"stray", &Expected::Absent)
        .unwrap();
    inventory::scan(&s.collection, &s.hosts, connector_id).unwrap();
    pipeline::edit_draft(&s.collection, s.feed, |f| {
        f.title = Some("Album (Deluxe)".into());
    })
    .unwrap();
    s.collection
        .add_draft_operation(
            s.feed,
            DraftOperationKind::Delete,
            None,
            connector_id,
            "stray.jpg",
        )
        .unwrap();

    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert_eq!(diff.operations.len(), 1);

    // The file changes on the host after the scan, so its stamp is not
    // the one that the inventory has.
    let current_version = s.connector.read("stray.jpg").unwrap().unwrap().version;
    s.connector
        .write(
            "stray.jpg",
            b"stray, but changed",
            &Expected::Version(current_version),
        )
        .unwrap();

    let error = pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::Conflict);

    // The feed file is already written, with the title change.
    assert!(live(&s).contains("Album (Deluxe)"));
    assert!(s.dir.path().join("stray.jpg").exists());
}

/// A connector that gives a wrong size for one path in `stat`, as if a
/// host had cut an upload (ADR 0043 item 7). It panics in `read` for the
/// path `no_read`, to show that the confirmation does not read a file.
struct CheckedHost {
    inner: LocalFolder,
    wrong_size: Option<String>,
    no_read: Option<String>,
}

impl Connector for CheckedHost {
    fn read(&self, path: &str) -> Result<Option<connector::FileRead>, connector::ConnectorError> {
        assert_ne!(
            Some(path),
            self.no_read.as_deref(),
            "the confirmation must not read {path}"
        );
        self.inner.read(path)
    }

    fn write(
        &self,
        path: &str,
        bytes: &[u8],
        expected: &Expected,
    ) -> Result<connector::Version, connector::ConnectorError> {
        self.inner.write(path, bytes, expected)
    }

    fn public_url(&self, path: &str) -> Option<String> {
        self.inner.public_url(path)
    }

    fn list(&self) -> Result<connector::FileList, connector::ConnectorError> {
        self.inner.list()
    }

    fn stat(&self, path: &str) -> Result<Option<connector::ListedFile>, connector::ConnectorError> {
        let mut listed = self.inner.stat(path)?;
        if let Some(file) = listed.as_mut()
            && Some(path) == self.wrong_size.as_deref()
        {
            file.size += 1;
        }
        Ok(listed)
    }

    fn delete(
        &self,
        path: &str,
        expected: &connector::Stamp,
    ) -> Result<(), connector::ConnectorError> {
        self.inner.delete(path, expected)
    }
}

/// An address check that fails for the addresses in `failing`.
struct FailOn {
    failing: Vec<String>,
}

impl AddressCheck for FailOn {
    fn check(&self, url: &str) -> Result<(), String> {
        if self.failing.iter().any(|failing| failing == url) {
            Err("answered with status 404".to_owned())
        } else {
            Ok(())
        }
    }
}

/// A draft with an upload, a title edit and a deletion of `stray.jpg`.
struct UploadAndDelete {
    connector_id: ConnectorId,
    upload_path: String,
    upload_url: String,
    diff_id: DiffId,
}

fn upload_and_delete(s: &mut Setup) -> UploadAndDelete {
    s.collection.set_feed_folder(s.feed, "album").unwrap();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();

    let imported = import_file(s, "01.mp3", b"one");
    let planned = reference_planned(s, imported.library_file_id);
    pipeline::edit_draft(&s.collection, s.feed, |f| {
        f.title = Some("Album (Deluxe)".into());
    })
    .unwrap();

    let connector_id = s.collection.feed(s.feed).unwrap().connector_id.unwrap();
    s.connector
        .write("stray.jpg", b"stray", &Expected::Absent)
        .unwrap();
    inventory::scan(&s.collection, &s.hosts, connector_id).unwrap();
    s.collection
        .add_draft_operation(
            s.feed,
            DraftOperationKind::Delete,
            None,
            connector_id,
            "stray.jpg",
        )
        .unwrap();

    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert_eq!(diff.operations.len(), 2);
    UploadAndDelete {
        connector_id,
        upload_url: planned.url.clone().unwrap(),
        upload_path: planned.path,
        diff_id: diff.id,
    }
}

/// Write the draft of `upload_and_delete` and expect `NotReachable`. The
/// deletion of the same write must not occur.
fn write_that_does_not_pass(s: &mut Setup, draft: &UploadAndDelete, addresses: &dyn AddressCheck) {
    let error = pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        addresses,
        s.feed,
        &draft.diff_id,
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::NotReachable);

    // The feed file is already written, and the upload is on the host.
    // The deletion did not run.
    assert!(live(s).contains("Album (Deluxe)"));
    assert_eq!(
        fs::read(s.dir.path().join(&draft.upload_path)).unwrap(),
        b"one"
    );
    assert!(s.dir.path().join("stray.jpg").exists());
}

#[test]
fn stat_with_a_different_size_stops_the_write_before_the_deletions() {
    let mut s = setup();
    let draft = upload_and_delete(&mut s);
    let inner = LocalFolder::new(s.dir.path(), Some("https://music.example".into()));
    s.hosts.insert(
        draft.connector_id,
        Box::new(CheckedHost {
            inner,
            wrong_size: Some(draft.upload_path.clone()),
            no_read: None,
        }),
    );
    write_that_does_not_pass(&mut s, &draft, &PassAll);
}

#[test]
fn a_public_address_that_fails_the_check_stops_the_write_before_the_deletions() {
    let mut s = setup();
    let draft = upload_and_delete(&mut s);
    let addresses = FailOn {
        failing: vec![draft.upload_url.clone()],
    };
    write_that_does_not_pass(&mut s, &draft, &addresses);
}

#[test]
fn a_feed_address_that_fails_the_check_stops_the_write_before_the_deletions() {
    let mut s = setup();
    let draft = upload_and_delete(&mut s);
    let addresses = FailOn {
        failing: vec!["https://music.example/album.xml".to_owned()],
    };
    write_that_does_not_pass(&mut s, &draft, &addresses);
}

#[test]
fn after_a_write_that_does_not_pass_the_draft_starts_from_the_written_feed() {
    let mut s = setup();
    let draft = upload_and_delete(&mut s);
    let draft_xml = s.collection.draft(s.feed).unwrap().unwrap().draft_xml;
    let addresses = FailOn {
        failing: vec![draft.upload_url.clone()],
    };
    write_that_does_not_pass(&mut s, &draft, &addresses);

    // The base of the draft is the written version. The draft XML stays.
    let kept = s.collection.draft(s.feed).unwrap().unwrap();
    let written = s
        .connector
        .read("album.xml")
        .unwrap()
        .expect("the feed file is on the host");
    assert_eq!(kept.base_version, Some(written.version));
    assert_eq!(kept.base_xml, written.bytes);
    assert_eq!(kept.draft_xml, draft_xml);

    // The draft keeps the deletion and has one confirmation.
    let operations = s.collection.draft_operations(s.feed).unwrap();
    let kinds: Vec<_> = operations
        .iter()
        .map(|o| (o.kind, o.path.as_str()))
        .collect();
    assert_eq!(
        kinds,
        [
            (DraftOperationKind::Delete, "stray.jpg"),
            (DraftOperationKind::Confirm, draft.upload_path.as_str()),
        ]
    );

    // The inventory knows the upload, so the next diff shows a confirm
    // operation and no upload.
    let next = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    let next_kinds: Vec<_> = next.operations.iter().map(|o| o.kind).collect();
    assert_eq!(next_kinds, [OperationKind::Delete, OperationKind::Confirm]);
    assert_ne!(next.id, draft.diff_id);
}

#[test]
fn a_second_write_after_a_write_that_does_not_pass_confirms_again_and_deletes() {
    let mut s = setup();
    let draft = upload_and_delete(&mut s);
    let addresses = FailOn {
        failing: vec![draft.upload_url.clone()],
    };
    write_that_does_not_pass(&mut s, &draft, &addresses);

    // The diff ID from before the failed write is not current.
    let error = pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &draft.diff_id,
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::DiffNotApproved);

    // The new diff, with an address check that passes, does the deletion.
    let next = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &next.id,
    )
    .unwrap();
    assert!(!s.dir.path().join("stray.jpg").exists());
    assert_eq!(
        fs::read(s.dir.path().join(&draft.upload_path)).unwrap(),
        b"one"
    );
    assert!(s.collection.draft(s.feed).unwrap().is_none());
    assert_eq!(s.collection.draft_operations(s.feed).unwrap(), Vec::new());
}

#[test]
fn a_confirmation_compares_the_host_file_with_the_library_file() {
    let mut s = setup();
    let draft = upload_and_delete(&mut s);
    let addresses = FailOn {
        failing: vec![draft.upload_url.clone()],
    };
    write_that_does_not_pass(&mut s, &draft, &addresses);

    // The confirmation keeps the library file of the upload.
    let operations = s.collection.draft_operations(s.feed).unwrap();
    let confirm = operations
        .iter()
        .find(|o| o.kind == DraftOperationKind::Confirm)
        .unwrap();
    assert!(confirm.library_file_id.is_some());

    // The host cuts the upload, and a scan records the new size. The
    // next write compares the host file with the library file, so it
    // does not pass, and the deletion does not occur.
    fs::write(s.dir.path().join(&draft.upload_path), b"on").unwrap();
    inventory::scan(&s.collection, &s.hosts, draft.connector_id).unwrap();
    let next = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    let error = pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &next.id,
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::NotReachable);
    assert!(s.dir.path().join("stray.jpg").exists());
}

#[test]
fn the_confirmation_does_not_read_the_uploaded_file() {
    let mut s = setup();
    let draft = upload_and_delete(&mut s);
    let inner = LocalFolder::new(s.dir.path(), Some("https://music.example".into()));
    s.hosts.insert(
        draft.connector_id,
        Box::new(CheckedHost {
            inner,
            wrong_size: None,
            no_read: Some(draft.upload_path.clone()),
        }),
    );
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &draft.diff_id,
    )
    .unwrap();
    assert!(!s.dir.path().join("stray.jpg").exists());
}

// ---- A move of the feed XML (ADR 0038 items 16 to 20) ----

const MOVE_FEED: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<rss version=\"2.0\" xmlns:podcast=\"https://podcastindex.org/namespace/1.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">
  <channel>
    <title>Album</title>
    <atom:link href=\"https://a.example/album.xml\" rel=\"self\" type=\"application/rss+xml\"/>
    <podcast:guid>917393e3-1b1e-5cef-ace4-edaa54e1f810</podcast:guid>
    <podcast:medium>music</podcast:medium>
    <item>
      <title>Track One</title>
      <guid>track-1</guid>
    </item>
  </channel>
</rss>
";

type Redirects = std::sync::Arc<std::sync::Mutex<Vec<(String, String)>>>;

/// A local folder that can make a redirect. It records each redirect.
struct RedirectingFolder {
    inner: LocalFolder,
    redirects: Redirects,
}

impl Connector for RedirectingFolder {
    fn read(&self, path: &str) -> Result<Option<connector::FileRead>, connector::ConnectorError> {
        self.inner.read(path)
    }
    fn write(
        &self,
        path: &str,
        bytes: &[u8],
        expected: &Expected,
    ) -> Result<connector::Version, connector::ConnectorError> {
        self.inner.write(path, bytes, expected)
    }
    fn public_url(&self, path: &str) -> Option<String> {
        self.inner.public_url(path)
    }
    fn list(&self) -> Result<connector::FileList, connector::ConnectorError> {
        self.inner.list()
    }
    fn stat(&self, path: &str) -> Result<Option<connector::ListedFile>, connector::ConnectorError> {
        self.inner.stat(path)
    }
    fn delete(
        &self,
        path: &str,
        expected: &connector::Stamp,
    ) -> Result<(), connector::ConnectorError> {
        self.inner.delete(path, expected)
    }
    fn can_redirect(&self) -> bool {
        true
    }
    fn set_redirect(&self, path: &str, to_url: &str) -> Result<(), connector::ConnectorError> {
        self.redirects
            .lock()
            .unwrap()
            .push((path.to_owned(), to_url.to_owned()));
        Ok(())
    }
}

struct MoveSetup {
    dir_a: tempfile::TempDir,
    dir_b: tempfile::TempDir,
    collection: Collection,
    hosts: Hosts,
    library: Library,
    feed: FeedId,
    a: ConnectorId,
    b: ConnectorId,
    redirects: Redirects,
}

fn move_setup(redirecting: bool) -> MoveSetup {
    let dir_a = tempfile::tempdir().unwrap();
    let dir_b = tempfile::tempdir().unwrap();
    let folder_a = LocalFolder::new(dir_a.path(), Some("https://a.example".into()));
    let folder_b = LocalFolder::new(dir_b.path(), Some("https://b.example".into()));
    folder_a
        .write("album.xml", MOVE_FEED.as_bytes(), &Expected::Absent)
        .unwrap();
    let mut collection = Collection::open(&dir_a.path().join("collection.sqlite")).unwrap();
    let a = collection
        .add_connector(LocalFolder::KIND, "A", &folder_a.config(), None)
        .unwrap();
    let b = collection
        .add_connector(LocalFolder::KIND, "B", &folder_b.config(), None)
        .unwrap();
    let redirects: Redirects = Redirects::default();
    let reader = connector::from_row(&collection.connector(a).unwrap(), None).unwrap();
    let mut hosts = Hosts::new();
    if redirecting {
        hosts.insert(
            a,
            Box::new(RedirectingFolder {
                inner: folder_a,
                redirects: redirects.clone(),
            }),
        );
    } else {
        hosts.insert(a, Box::new(folder_a));
    }
    hosts.insert(b, Box::new(folder_b));
    let library = Library::open(dir_a.path().join("library")).unwrap();
    let feed = collection
        .add_feed(Some("https://a.example/album.xml"), Some((a, "album.xml")))
        .unwrap();
    pipeline::refresh(&mut collection, reader.as_ref(), feed).unwrap();
    collection.set_feed_folder(feed, "album").unwrap();
    library
        .write_feed_xml("album", "album.xml", MOVE_FEED.as_bytes())
        .unwrap();
    pipeline::start_draft(&collection, reader.as_ref(), feed).unwrap();
    MoveSetup {
        dir_a,
        dir_b,
        collection,
        hosts,
        library,
        feed,
        a,
        b,
        redirects,
    }
}

#[test]
fn a_feed_move_puts_the_feed_on_the_new_host_and_a_pointer_file_on_the_old_host() {
    let mut s = move_setup(false);
    pipeline::set_feed_host(&s.collection, &s.hosts, s.feed, s.b).unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    let moved = diff.feed_move.clone().unwrap();
    assert_eq!(moved.method, pipeline::FeedMoveMethod::Pointer);
    assert_eq!(moved.url.as_deref(), Some("https://b.example/album.xml"));
    assert_eq!(
        moved.from_url.as_deref(),
        Some("https://a.example/album.xml")
    );
    assert!(!diff.announce);

    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();

    let on_b = fs::read_to_string(s.dir_b.path().join("album.xml")).unwrap();
    assert!(on_b.contains("href=\"https://b.example/album.xml\""));
    assert!(!on_b.contains("new-feed-url"));
    let on_a = fs::read_to_string(s.dir_a.path().join("album.xml")).unwrap();
    assert!(on_a.contains("new-feed-url>https://b.example/album.xml</"));
    let row = s.collection.feed(s.feed).unwrap();
    assert_eq!(row.connector_id, Some(s.b));
    assert_eq!(
        row.files_connector_id,
        Some(s.a),
        "ADR 0038 item 16: the files of a moved feed keep their host"
    );
    assert_eq!(row.path.as_deref(), Some("album.xml"));
    assert_eq!(
        row.public_url.as_deref(),
        Some("https://b.example/album.xml")
    );
    assert_eq!(
        row.podcast_guid.as_deref(),
        Some("917393e3-1b1e-5cef-ace4-edaa54e1f810")
    );
    let in_library = fs::read(s.library.path("album", "album.xml")).unwrap();
    assert_eq!(in_library, on_b.as_bytes());
    assert!(s.collection.is_feed_pointer(s.a, "album.xml").unwrap());
    assert!(s.collection.draft(s.feed).unwrap().is_none());
    assert!(s.collection.draft_feed_move(s.feed).unwrap().is_none());
}

#[test]
fn a_host_that_can_redirect_gets_a_redirect_and_keeps_its_file_with_no_pointer() {
    let mut s = move_setup(true);
    pipeline::set_feed_host(&s.collection, &s.hosts, s.feed, s.b).unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert_eq!(
        diff.feed_move.as_ref().unwrap().method,
        pipeline::FeedMoveMethod::Redirect
    );
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();

    assert_eq!(
        *s.redirects.lock().unwrap(),
        [(
            "album.xml".to_owned(),
            "https://b.example/album.xml".to_owned()
        )]
    );
    let on_a = fs::read_to_string(s.dir_a.path().join("album.xml")).unwrap();
    assert_eq!(on_a, MOVE_FEED);
    assert!(!s.collection.is_feed_pointer(s.a, "album.xml").unwrap());
    assert_eq!(s.collection.feed(s.feed).unwrap().connector_id, Some(s.b));
}

#[test]
fn a_move_to_a_host_with_a_file_at_the_path_gives_file_exists() {
    let s = move_setup(false);
    fs::write(s.dir_b.path().join("album.xml"), b"other").unwrap();
    inventory::scan(&s.collection, &s.hosts, s.b).unwrap();
    let error = pipeline::set_feed_host(&s.collection, &s.hosts, s.feed, s.b).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::FileExists);
    assert!(s.collection.draft_feed_move(s.feed).unwrap().is_none());
}

#[test]
fn a_file_put_on_the_new_host_after_the_diff_stops_the_write_with_file_exists() {
    let mut s = move_setup(false);
    pipeline::set_feed_host(&s.collection, &s.hosts, s.feed, s.b).unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    fs::write(s.dir_b.path().join("album.xml"), b"other").unwrap();
    let error = pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::FileExists);
    let on_a = fs::read_to_string(s.dir_a.path().join("album.xml")).unwrap();
    assert_eq!(on_a, MOVE_FEED);
    assert_eq!(s.collection.feed(s.feed).unwrap().connector_id, Some(s.a));
}

#[test]
fn a_write_that_stopped_after_the_new_host_got_the_feed_can_finish_the_move() {
    let mut s = move_setup(false);
    pipeline::set_feed_host(&s.collection, &s.hosts, s.feed, s.b).unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    // A write that stopped after its second step left the draft XML on
    // host B, and host A has no pointer yet.
    let draft = s.collection.draft(s.feed).unwrap().unwrap();
    fs::write(s.dir_b.path().join("album.xml"), &draft.draft_xml).unwrap();

    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();

    let on_a = fs::read_to_string(s.dir_a.path().join("album.xml")).unwrap();
    assert!(on_a.contains("new-feed-url>https://b.example/album.xml</"));
    assert_eq!(s.collection.feed(s.feed).unwrap().connector_id, Some(s.b));
}

#[test]
fn a_deletion_of_the_feed_xml_of_a_feed_is_blocked() {
    let s = move_setup(false);
    s.collection
        .add_draft_operation(s.feed, DraftOperationKind::Delete, None, s.a, "album.xml")
        .unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert!(
        diff.operations
            .iter()
            .all(|op| op.kind != pipeline::OperationKind::Delete)
    );
    assert_eq!(diff.blocked.len(), 1);
    assert_eq!(diff.blocked[0].path, "album.xml");
    assert_eq!(diff.blocked[0].feed_titles.len(), 1);
    assert!(!diff.blocked[0].pointer);
}

#[test]
fn the_host_of_the_feed_removes_the_move_and_puts_the_self_link_back() {
    let s = move_setup(false);
    pipeline::set_feed_host(&s.collection, &s.hosts, s.feed, s.b).unwrap();
    let (feed, _) = pipeline::draft_model(&s.collection, s.feed).unwrap();
    assert_eq!(feed.self_url(), Some("https://b.example/album.xml"));
    pipeline::set_feed_host(&s.collection, &s.hosts, s.feed, s.a).unwrap();
    assert!(s.collection.draft_feed_move(s.feed).unwrap().is_none());
    let draft = s.collection.draft(s.feed).unwrap().unwrap();
    assert_eq!(draft.draft_xml, draft.base_xml);
}

#[test]
fn a_discard_of_the_draft_removes_the_feed_move() {
    let s = move_setup(false);
    pipeline::set_feed_host(&s.collection, &s.hosts, s.feed, s.b).unwrap();
    assert!(pipeline::draft_changed(&s.collection, s.feed).unwrap());
    pipeline::discard_draft(&s.collection, s.feed).unwrap();
    assert!(s.collection.draft_feed_move(s.feed).unwrap().is_none());
}

#[test]
fn a_deletion_of_a_pointer_file_is_blocked() {
    let mut s = move_setup(false);
    pipeline::set_feed_host(&s.collection, &s.hosts, s.feed, s.b).unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();

    let reader = connector::from_row(&s.collection.connector(s.b).unwrap(), None).unwrap();
    pipeline::start_draft(&s.collection, reader.as_ref(), s.feed).unwrap();
    s.collection
        .add_draft_operation(s.feed, DraftOperationKind::Delete, None, s.a, "album.xml")
        .unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert_eq!(diff.operations, Vec::new());
    assert_eq!(diff.blocked.len(), 1);
    assert!(diff.blocked[0].pointer);
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();
    assert!(s.dir_a.path().join("album.xml").exists());
}

// ---- The finish step (ADR 0050 items 7 to 9) ----

/// A connector that wraps a local folder and records each change call as
/// text. It can fail its first `finish`, and it can give kept blobs.
struct Recorder {
    inner: LocalFolder,
    log: Arc<Mutex<Vec<String>>>,
    fail_first_finish: Mutex<bool>,
    blobs: Option<Vec<KeptBlob>>,
}

impl Recorder {
    fn note(&self, text: String) {
        self.log.lock().unwrap().push(text);
    }
}

impl Connector for Recorder {
    fn read(&self, path: &str) -> Result<Option<FileRead>, ConnectorError> {
        self.inner.read(path)
    }
    fn write(
        &self,
        path: &str,
        bytes: &[u8],
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        self.note(format!("write {path}"));
        self.inner.write(path, bytes, expected)
    }
    fn write_from(
        &self,
        path: &str,
        reader: &mut dyn Read,
        length: u64,
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        self.note(format!("write {path}"));
        self.inner.write_from(path, reader, length, expected)
    }
    fn sha256(&self, path: &str) -> Result<Option<String>, ConnectorError> {
        self.inner.sha256(path)
    }
    fn public_url(&self, path: &str) -> Option<String> {
        self.inner.public_url(path)
    }
    fn list(&self) -> Result<FileList, ConnectorError> {
        self.inner.list()
    }
    fn stat(&self, path: &str) -> Result<Option<ListedFile>, ConnectorError> {
        self.inner.stat(path)
    }
    fn delete(&self, path: &str, expected: &Stamp) -> Result<(), ConnectorError> {
        self.note(format!("delete {path}"));
        self.inner.delete(path, expected)
    }
    fn finish(&self) -> Result<(), ConnectorError> {
        let mut fail = self.fail_first_finish.lock().unwrap();
        if *fail {
            *fail = false;
            self.note("finish failed".to_owned());
            return Err(ConnectorError::new(
                ConnectorErrorKind::Io,
                "the test finish fails",
            ));
        }
        self.note("finish".to_owned());
        Ok(())
    }
    fn kept_blobs(&self) -> Option<Vec<KeptBlob>> {
        self.blobs.clone()
    }
}

/// Replace the host of `s` with a [`Recorder`] on the same folder. Gives
/// the log of the recorder.
fn record_host(
    s: &mut Setup,
    fail_first_finish: bool,
    blobs: Option<Vec<KeptBlob>>,
) -> Arc<Mutex<Vec<String>>> {
    let connector_id = s.collection.connectors().unwrap()[0].id;
    let log = Arc::new(Mutex::new(Vec::new()));
    s.hosts.insert(
        connector_id,
        Box::new(Recorder {
            inner: LocalFolder::new(s.dir.path(), Some("https://music.example".into())),
            log: Arc::clone(&log),
            fail_first_finish: Mutex::new(fail_first_finish),
            blobs,
        }),
    );
    log
}

/// Write a draft with one track, and then make a draft with a changed
/// track (new upload) and a deletion of the old file. Gives the approved
/// diff ID, the path of the new file and the path of the old file.
fn draft_with_upload_and_deletion(s: &mut Setup) -> (DiffId, String, String) {
    s.collection.set_feed_folder(s.feed, "album").unwrap();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    let old = import_file(s, "01.mp3", b"one");
    let planned_old = reference_planned(s, old.library_file_id);
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();

    s.collection
        .remove_library_file(old.library_file_id)
        .unwrap();
    let source_dir = tempfile::tempdir().unwrap();
    let new_source = source_dir.path().join("01-new.mp3");
    fs::write(&new_source, b"one (re-recorded)").unwrap();
    let (new_size, new_sha) = s.library.put_file("album", "01.mp3", &new_source).unwrap();
    let new_id = s
        .collection
        .add_library_file(
            s.feed,
            "01.mp3",
            new_size,
            Some(&new_sha),
            "audio/mpeg",
            true,
            None,
            None,
        )
        .unwrap();
    let feed_row = s.collection.feed(s.feed).unwrap();
    let new_file = s.collection.library_file(new_id).unwrap();
    let planned_new =
        inventory::planned_path(&s.collection, &s.hosts, &feed_row, &new_file).unwrap();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    let old_url = planned_old.url.clone().unwrap();
    let new_url = planned_new.url.clone().unwrap();
    pipeline::edit_draft(&s.collection, s.feed, move |f| {
        for item in &mut f.items {
            if let Some(enclosure) = item.value.enclosure.as_mut()
                && enclosure.url.as_deref() == Some(old_url.as_str())
            {
                enclosure.url = Some(new_url.clone());
            }
        }
    })
    .unwrap();
    s.collection
        .add_draft_operation(
            s.feed,
            DraftOperationKind::Delete,
            None,
            planned_old.connector_id,
            &planned_old.path,
        )
        .unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    (diff.id, planned_new.path, planned_old.path)
}

#[test]
fn a_write_calls_finish_after_the_feed_xml_and_again_after_the_deletions() {
    let mut s = setup();
    let (diff_id, new_path, old_path) = draft_with_upload_and_deletion(&mut s);
    let log = record_host(&mut s, false, None);
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff_id,
    )
    .unwrap();
    assert_eq!(
        *log.lock().unwrap(),
        [
            format!("write {new_path}"),
            "write album.xml".to_owned(),
            "finish".to_owned(),
            format!("delete {old_path}"),
            "finish".to_owned(),
        ]
    );
}

#[test]
fn a_host_with_no_deletion_gets_one_finish_only() {
    let mut s = setup();
    let log = record_host(&mut s, false, None);
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    pipeline::edit_draft(&s.collection, s.feed, |f| f.title = Some("New".into())).unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();
    assert_eq!(
        *log.lock().unwrap(),
        ["write album.xml".to_owned(), "finish".to_owned()]
    );
}

#[test]
fn a_failed_finish_gives_an_error_and_keeps_the_draft_and_the_feed_state() {
    let mut s = setup();
    let before = s.collection.feed(s.feed).unwrap().last_version;
    let log = record_host(&mut s, true, None);
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    pipeline::edit_draft(&s.collection, s.feed, |f| f.title = Some("New".into())).unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    let error = pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::Connector);
    assert_eq!(
        *log.lock().unwrap(),
        ["write album.xml".to_owned(), "finish failed".to_owned()]
    );
    assert!(s.collection.draft(s.feed).unwrap().is_some());
    let row = s.collection.feed(s.feed).unwrap();
    assert_eq!(row.last_version, before);
    assert_eq!(row.title.as_deref(), Some("Album"));
}

#[test]
fn a_host_that_gives_kept_blobs_has_them_in_the_collection_after_the_write() {
    let mut s = setup();
    let blobs = vec![KeptBlob {
        path: "album/01.mp3".to_owned(),
        sha256: "ab".repeat(32),
        size: 3,
    }];
    record_host(&mut s, false, Some(blobs.clone()));
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    pipeline::edit_draft(&s.collection, s.feed, |f| f.title = Some("New".into())).unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .unwrap();
    let connector_id = s.collection.connectors().unwrap()[0].id;
    assert_eq!(s.collection.kept_blobs(connector_id).unwrap(), blobs);
}

// ---- A feed on an nsite host (ADR 0048) ----

/// A shared nsite host, so that the draft reads and the write use one view.
struct SharedHost(Arc<NsiteHost>);

impl Connector for SharedHost {
    fn read(&self, path: &str) -> Result<Option<FileRead>, ConnectorError> {
        self.0.read(path)
    }
    fn write(
        &self,
        path: &str,
        bytes: &[u8],
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        self.0.write(path, bytes, expected)
    }
    fn write_from(
        &self,
        path: &str,
        reader: &mut dyn Read,
        length: u64,
        expected: &Expected,
    ) -> Result<Version, ConnectorError> {
        self.0.write_from(path, reader, length, expected)
    }
    fn sha256(&self, path: &str) -> Result<Option<String>, ConnectorError> {
        self.0.sha256(path)
    }
    fn public_url(&self, path: &str) -> Option<String> {
        self.0.public_url(path)
    }
    fn planned_url(&self, path: &str, sha256: Option<&str>) -> Option<String> {
        self.0.planned_url(path, sha256)
    }
    fn plan_events(
        &self,
        changes: &HostChanges,
        plan_time: u64,
    ) -> Result<Vec<PlannedEvent>, ConnectorError> {
        self.0.plan_events(changes, plan_time)
    }
    fn finish(&self) -> Result<(), ConnectorError> {
        self.0.finish()
    }
    fn list(&self) -> Result<FileList, ConnectorError> {
        self.0.list()
    }
    fn stat(&self, path: &str) -> Result<Option<ListedFile>, ConnectorError> {
        self.0.stat(path)
    }
    fn delete(&self, path: &str, expected: &Stamp) -> Result<(), ConnectorError> {
        self.0.delete(path, expected)
    }
}

#[test]
fn a_feed_on_an_nsite_host_writes_an_upload_and_a_deletion_with_two_manifests() {
    let relays = [TestRelay::start(), TestRelay::start()];
    let blossoms = [TestBlossom::start(), TestBlossom::start()];
    let secret_hex = "01".repeat(32);
    let keys = studio_core::nostr_events::parse_secret_key(&secret_hex).unwrap();
    let config = serde_json::json!({
        "public_key": keys.public_key().to_hex(),
        "d": "album",
        "relays": relays.iter().map(TestRelay::url).collect::<Vec<_>>(),
        "servers": blossoms.iter().map(TestBlossom::url).collect::<Vec<_>>(),
        "signing": "vault",
    })
    .to_string();
    let secret = serde_json::json!({ "secret_key": secret_hex })
        .to_string()
        .into_bytes();
    let host =
        Arc::new(NsiteHost::from_config(&config, Some(&secret), Signing::default()).unwrap());
    host.write("album.xml", FEED.as_bytes(), &Expected::Absent)
        .unwrap();
    host.finish().unwrap();

    let dir = tempfile::tempdir().unwrap();
    let mut collection = Collection::open(&dir.path().join("collection.sqlite")).unwrap();
    let connector_id = collection
        .add_connector(NsiteHost::KIND, "Site", &config, None)
        .unwrap();
    let mut hosts = Hosts::new();
    hosts.insert(connector_id, Box::new(SharedHost(Arc::clone(&host))));
    let library = Library::open(dir.path().join("library")).unwrap();
    let feed = collection
        .add_feed(
            host.public_url("album.xml").as_deref(),
            Some((connector_id, "album.xml")),
        )
        .unwrap();
    let connector: Box<dyn Connector> = Box::new(SharedHost(Arc::clone(&host)));
    pipeline::refresh(&mut collection, connector.as_ref(), feed).unwrap();
    let mut s = Setup {
        dir,
        collection,
        connector,
        hosts,
        library,
        feed,
    };

    // The draft writes an upload first. The second draft uploads a changed
    // file and deletes the old file.
    let (diff_id, new_path, old_path) = draft_with_upload_and_deletion(&mut s);
    let before: Vec<usize> = relays.iter().map(TestRelay::received).collect();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff_id,
    )
    .unwrap();
    for (relay, before) in relays.iter().zip(before) {
        assert_eq!(relay.received() - before, 2);
    }
    assert!(blossoms.iter().all(|b| b.deletes().is_empty()));

    // A new connector on the same site sees the new file and not the old.
    let reader = NsiteHost::from_config(&config, Some(&secret), Signing::default()).unwrap();
    assert!(reader.read(&new_path).unwrap().is_some());
    assert!(reader.read(&old_path).unwrap().is_none());
    let feed_xml = reader.read("album.xml").unwrap().unwrap();
    assert!(
        String::from_utf8(feed_xml.bytes)
            .unwrap()
            .contains(&new_path)
    );
}

// ---- The signing plan (ADR 0050 items 10 to 13) ----

/// An nsite site on two test relays and two test servers, with a feed that
/// has a draft.
struct SigningSite {
    relays: [TestRelay; 2],
    blossoms: [TestBlossom; 2],
    config: String,
    secret: Vec<u8>,
    keys: nostr::prelude::Keys,
    // The folder stays while the site exists.
    _dir: tempfile::TempDir,
    collection: Collection,
    library: Library,
    feed: FeedId,
    vault: bool,
}

impl SigningSite {
    /// A host of the site with the events that the user signed.
    fn host(&self, signing: Signing) -> Arc<NsiteHost> {
        let secret = self.vault.then_some(self.secret.as_slice());
        Arc::new(NsiteHost::from_config(&self.config, secret, signing).unwrap())
    }

    /// A host set that has the site as the only host.
    fn hosts(&self, signing: Signing) -> Hosts {
        let mut hosts = Hosts::new();
        hosts.insert(
            self.collection.connectors().unwrap()[0].id,
            Box::new(SharedHost(self.host(signing))),
        );
        hosts
    }

    /// The number of events that the relays got.
    fn received(&self) -> Vec<usize> {
        self.relays.iter().map(TestRelay::received).collect()
    }

    fn write(
        &mut self,
        diff_id: &DiffId,
        plan_time: Option<u64>,
        signed: &[nostr::prelude::Event],
        now: u64,
    ) -> Result<Version, pipeline::PipelineError> {
        let hosts = self.hosts(Signing {
            plan_time,
            signed: signed.to_owned(),
        });
        pipeline::write_draft_signed(
            &mut self.collection,
            &hosts,
            &self.library,
            &PassAll,
            self.feed,
            diff_id,
            signed,
            now,
        )
    }

    /// The events of a diff, signed with the key of the site.
    fn sign_all(&self, diff: &pipeline::Diff) -> Vec<nostr::prelude::Event> {
        diff.events
            .iter()
            .map(|e| {
                studio_core::nostr_events::sign_with_keys(e.event.clone(), &self.keys).unwrap()
            })
            .collect()
    }

    fn diff_at(&self, now: u64) -> pipeline::Diff {
        let hosts = self.hosts(Signing::default());
        pipeline::diff_at(&self.collection, &hosts, &self.library, self.feed, now).unwrap()
    }

    /// The IDs of the events that the first relay keeps.
    fn manifests(&self) -> Vec<nostr::prelude::EventId> {
        self.relays[0].events().iter().map(|e| e.id).collect()
    }

    /// The number of blobs on each server.
    fn blob_counts(&self) -> Vec<usize> {
        self.blossoms.iter().map(|b| b.blobs().len()).collect()
    }
}

fn time_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn signing_site(mode: &str) -> SigningSite {
    let relays = [TestRelay::start(), TestRelay::start()];
    let blossoms = [TestBlossom::start(), TestBlossom::start()];
    let secret_hex = "01".repeat(32);
    let keys = studio_core::nostr_events::parse_secret_key(&secret_hex).unwrap();
    let config_for = |mode: &str| {
        serde_json::json!({
            "public_key": keys.public_key().to_hex(),
            "d": "album",
            "relays": relays.iter().map(TestRelay::url).collect::<Vec<_>>(),
            "servers": blossoms.iter().map(TestBlossom::url).collect::<Vec<_>>(),
            "signing": mode,
        })
        .to_string()
    };
    let secret = serde_json::json!({ "secret_key": secret_hex })
        .to_string()
        .into_bytes();
    // The seed host has the key, so that it can write the first feed XML.
    let seed =
        NsiteHost::from_config(&config_for("vault"), Some(&secret), Signing::default()).unwrap();
    seed.write("album.xml", FEED.as_bytes(), &Expected::Absent)
        .unwrap();
    seed.finish().unwrap();

    let config = config_for(mode);
    let dir = tempfile::tempdir().unwrap();
    let mut collection = Collection::open(&dir.path().join("collection.sqlite")).unwrap();
    let connector_id = collection
        .add_connector(NsiteHost::KIND, "Site", &config, None)
        .unwrap();
    let library = Library::open(dir.path().join("library")).unwrap();
    let feed = collection
        .add_feed(
            seed.public_url("album.xml").as_deref(),
            Some((connector_id, "album.xml")),
        )
        .unwrap();
    let site = SigningSite {
        relays,
        blossoms,
        config,
        secret,
        keys,
        _dir: dir,
        collection,
        library,
        feed,
        vault: mode == "vault",
    };
    let connector = SharedHost(site.host(Signing::default()));
    let mut site = site;
    pipeline::refresh(&mut site.collection, &connector, feed).unwrap();
    pipeline::start_draft(&site.collection, &connector, feed).unwrap();
    pipeline::edit_draft(&site.collection, feed, |f| {
        f.title = Some("Album (Deluxe)".into());
    })
    .unwrap();
    site
}

fn event_ids(diff: &pipeline::Diff) -> Vec<String> {
    diff.events
        .iter()
        .map(|e| e.event.compute_id().to_hex())
        .collect()
}

#[test]
fn a_signer_diff_gives_planned_events_with_the_same_ids_each_time() {
    let site = signing_site("signer");
    let now = time_now();
    let first = site.diff_at(now);
    assert!(!first.events.is_empty());
    assert!(first.events.iter().all(|e| e.user_signs));
    let second = site.diff_at(now + 5);
    assert_eq!(first.id, second.id);
    assert_eq!(event_ids(&first), event_ids(&second));
}

#[test]
fn a_signer_write_with_no_signed_events_changes_nothing_on_a_host() {
    let mut site = signing_site("signer");
    let now = time_now();
    let diff = site.diff_at(now);
    let plan_time = site
        .collection
        .signing_plan(site.feed)
        .unwrap()
        .map(|p| p.1);
    let before = (site.received(), site.blob_counts());
    let error = site.write(&diff.id, plan_time, &[], now).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::SignatureMissing);
    assert_eq!((site.received(), site.blob_counts()), before);
}

#[test]
fn a_signer_write_with_the_signed_events_publishes_the_manifest() {
    let mut site = signing_site("signer");
    let now = time_now();
    let diff = site.diff_at(now);
    let signed = site.sign_all(&diff);
    let plan_time = site
        .collection
        .signing_plan(site.feed)
        .unwrap()
        .map(|p| p.1);
    let manifests = site.manifests();
    let blobs = site.blob_counts();
    site.write(&diff.id, plan_time, &signed, now).unwrap();
    assert_ne!(site.manifests(), manifests);
    assert!(site.blob_counts().iter().zip(blobs).all(|(a, b)| *a > b));
    assert_eq!(site.collection.signing_plan(site.feed).unwrap(), None);
}

#[test]
fn a_planned_event_signed_by_a_different_key_gives_signature_invalid() {
    let mut site = signing_site("signer");
    let now = time_now();
    let diff = site.diff_at(now);
    let other =
        nostr::prelude::Keys::new(nostr::prelude::SecretKey::from_hex(&"02".repeat(32)).unwrap());
    let signed: Vec<_> = diff
        .events
        .iter()
        .map(|e| {
            let unsigned = nostr::prelude::UnsignedEvent::new(
                other.public_key(),
                e.event.created_at,
                e.event.kind,
                e.event.tags.clone(),
                e.event.content.clone(),
            );
            studio_core::nostr_events::sign_with_keys(unsigned, &other).unwrap()
        })
        .collect();
    let plan_time = site
        .collection
        .signing_plan(site.feed)
        .unwrap()
        .map(|p| p.1);
    let before = (site.received(), site.blob_counts());
    let error = site.write(&diff.id, plan_time, &signed, now).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::SignatureInvalid);
    assert_eq!((site.received(), site.blob_counts()), before);
}

#[test]
fn a_signed_event_with_a_wrong_signature_gives_signature_invalid() {
    let mut site = signing_site("signer");
    let now = time_now();
    let diff = site.diff_at(now);
    let mut signed = site.sign_all(&diff);
    // The first event gets the signature of the second event.
    let wrong = signed[1].sig;
    let first = &signed[0];
    signed[0] = nostr::prelude::Event::new(
        first.id,
        first.pubkey,
        first.created_at,
        first.kind,
        first.tags.clone(),
        first.content.clone(),
        wrong,
    );
    let plan_time = site
        .collection
        .signing_plan(site.feed)
        .unwrap()
        .map(|p| p.1);
    let before = (site.received(), site.blob_counts());
    let error = site.write(&diff.id, plan_time, &signed, now).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::SignatureInvalid);
    assert_eq!((site.received(), site.blob_counts()), before);
}

#[test]
fn a_write_at_the_end_of_the_plan_gives_plan_expired() {
    let mut site = signing_site("signer");
    let now = time_now();
    let diff = site.diff_at(now);
    let signed = site.sign_all(&diff);
    let plan_time = site
        .collection
        .signing_plan(site.feed)
        .unwrap()
        .map(|p| p.1)
        .unwrap();
    let before = (site.received(), site.blob_counts());
    let error = site
        .write(&diff.id, None, &signed, plan_time + 3600)
        .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::PlanExpired);
    assert_eq!((site.received(), site.blob_counts()), before);
}

#[test]
fn a_diff_55_minutes_after_the_first_diff_gives_a_new_plan_time() {
    let site = signing_site("signer");
    let now = time_now();
    let first = site.diff_at(now);
    assert_eq!(
        site.collection.signing_plan(site.feed).unwrap(),
        Some((first.id.as_str().to_owned(), now))
    );
    let same = site.diff_at(now + 40 * 60);
    assert_eq!(event_ids(&first), event_ids(&same));
    let later = site.diff_at(now + 55 * 60);
    assert_eq!(first.id, later.id);
    assert_eq!(
        site.collection.signing_plan(site.feed).unwrap(),
        Some((first.id.as_str().to_owned(), now + 55 * 60))
    );
    assert_ne!(event_ids(&first), event_ids(&later));
}

#[test]
fn a_vault_write_needs_no_signed_events() {
    let mut site = signing_site("vault");
    let now = time_now();
    let diff = site.diff_at(now);
    assert!(diff.events.iter().all(|e| !e.user_signs));
    let plan_time = site
        .collection
        .signing_plan(site.feed)
        .unwrap()
        .map(|p| p.1);
    let manifests = site.manifests();
    site.write(&diff.id, plan_time, &[], now).unwrap();
    assert_ne!(site.manifests(), manifests);
}

#[test]
fn a_write_to_a_local_folder_host_needs_no_plan() {
    let mut s = setup();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    pipeline::edit_draft(&s.collection, s.feed, |f| {
        f.title = Some("Album (Deluxe)".into());
    })
    .unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert!(diff.events.is_empty());
    s.collection.remove_signing_plan(s.feed).unwrap();
    pipeline::write_draft_signed(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
        &[],
        time_now(),
    )
    .unwrap();
    assert!(live(&s).contains("Album (Deluxe)"));
}

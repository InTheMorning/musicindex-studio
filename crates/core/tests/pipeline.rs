//! The write pipeline over the local folder connector (ADR 0007, ADR 0015).

use std::fs;

use studio_core::collection::{Collection, ConnectorId, DraftOperationKind, FeedId, LibraryFileId};
use studio_core::connector::{self, Connector, Expected, LocalFolder};
use studio_core::feed::{Enclosure, Entry, Feed, Item};
use studio_core::guid::podcast_guid_for_url;
use studio_core::inventory::{self, Hosts};
use studio_core::library::Library;
use studio_core::pipeline::{self, DiffFile, DiffId, OperationKind, PipelineErrorKind};
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

    let version =
        pipeline::write_draft(&mut s.collection, &s.hosts, &s.library, s.feed, &diff.id).unwrap();
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
    let error = pipeline::write_draft(&mut s.collection, &s.hosts, &s.library, s.feed, &approved)
        .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::DiffNotApproved);
    let error = pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        s.feed,
        &DiffId::new("x"),
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::DiffNotApproved);
    assert_eq!(live(&s), FEED);
    assert!(s.collection.snapshots(s.feed).unwrap().is_empty());
}

#[test]
fn a_file_that_changed_on_the_host_after_the_draft_started_does_not_write() {
    let mut s = setup();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    pipeline::edit_draft(&s.collection, s.feed, |f| f.title = Some("Mine".into())).unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    let theirs = FEED.replace("Track One", "Track One (edited elsewhere)");
    fs::write(s.dir.path().join("album.xml"), &theirs).unwrap();
    let error = pipeline::write_draft(&mut s.collection, &s.hosts, &s.library, s.feed, &diff.id)
        .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::Conflict);
    assert_eq!(live(&s), theirs);
    assert!(s.collection.snapshots(s.feed).unwrap().is_empty());
    assert!(s.collection.draft(s.feed).unwrap().is_some());
}

#[test]
fn a_rollback_is_a_draft_from_a_snapshot_with_its_own_diff() {
    let mut s = setup();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    pipeline::edit_draft(&s.collection, s.feed, |f| f.items.clear()).unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    pipeline::write_draft(&mut s.collection, &s.hosts, &s.library, s.feed, &diff.id).unwrap();
    assert!(!live(&s).contains("track-1"));
    assert!(s.collection.item_guids(s.feed).unwrap().is_empty());

    let snapshot = s.collection.snapshots(s.feed).unwrap()[0].id;
    pipeline::draft_from_snapshot(&s.collection, s.connector.as_ref(), s.feed, snapshot).unwrap();
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    assert!(diff.unified.contains("+      <guid>track-1</guid>"));
    pipeline::write_draft(&mut s.collection, &s.hosts, &s.library, s.feed, &diff.id).unwrap();
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
fn a_new_feed_makes_a_draft_with_no_hosted_file() {
    let (dir, mut collection, connector, connector_id, _hosts, _library) = new_setup();
    let id = pipeline::new_feed(
        &mut collection,
        connector.as_ref(),
        connector_id,
        "my-album.xml",
        "New Album",
        "New Artist",
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
    )
    .unwrap();
    let diff = pipeline::diff(&collection, &hosts, &library, id).unwrap();
    assert!(diff.changed);
    // The live file is absent, so the diff adds the full file and removes
    // nothing.
    assert!(diff.unified.contains("+<rss"));
    assert!(!diff.unified.contains("\n-<"));

    let version = pipeline::write_draft(&mut collection, &hosts, &library, id, &diff.id).unwrap();
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
    )
    .unwrap();
    let diff = pipeline::diff(&collection, &hosts, &library, id).unwrap();
    // A different tool puts a file at the path before Studio writes it.
    fs::write(
        dir.path().join("my-album.xml"),
        b"<rss><channel><title>Other</title></channel></rss>",
    )
    .unwrap();
    let error = pipeline::write_draft(&mut collection, &hosts, &library, id, &diff.id).unwrap_err();
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
    assert!(diff.operations.is_empty());
    assert!(diff.files.is_empty());
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

    pipeline::write_draft(&mut s.collection, &s.hosts, &s.library, s.feed, &diff.id).unwrap();

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
    pipeline::write_draft(&mut s.collection, &s.hosts, &s.library, s.feed, &diff.id).unwrap();
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
            &new_sha,
            "audio/mpeg",
            true,
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

    pipeline::write_draft(&mut s.collection, &s.hosts, &s.library, s.feed, &diff.id).unwrap();

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
    assert!(diff.operations.is_empty());
    assert_eq!(diff.blocked.len(), 1);
    assert_eq!(diff.blocked[0].feed_titles, ["Second Album".to_owned()]);

    pipeline::write_draft(&mut s.collection, &s.hosts, &s.library, s.feed, &diff.id).unwrap();

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

    let error = pipeline::write_draft(&mut s.collection, &s.hosts, &s.library, s.feed, &diff.id)
        .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::Conflict);

    // The feed file is already written, with the title change.
    assert!(live(&s).contains("Album (Deluxe)"));
    assert!(s.dir.path().join("stray.jpg").exists());
}

/// A connector that cannot read one path back, as if a host had not yet
/// made an upload reachable (ADR 0036 item 15).
struct UnreachableAfterWrite {
    inner: LocalFolder,
    blind_to: String,
}

impl Connector for UnreachableAfterWrite {
    fn read(&self, path: &str) -> Result<Option<connector::FileRead>, connector::ConnectorError> {
        if path == self.blind_to {
            return Ok(None);
        }
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
}

#[test]
fn an_address_not_reachable_after_the_upload_stops_the_write_before_the_deletions() {
    let mut s = setup();
    s.collection.set_feed_folder(s.feed, "album").unwrap();
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();

    let imported = import_file(&s, "01.mp3", b"one");
    let planned = reference_planned(&s, imported.library_file_id);
    pipeline::edit_draft(&s.collection, s.feed, |f| {
        f.title = Some("Album (Deluxe)".into());
    })
    .unwrap();

    // A delete operation too, to show that it does not run either.
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

    // Studio cannot read the uploaded file back, as if the host had not
    // yet made it reachable.
    let inner = LocalFolder::new(s.dir.path(), Some("https://music.example".into()));
    s.hosts.insert(
        connector_id,
        Box::new(UnreachableAfterWrite {
            inner,
            blind_to: planned.path.clone(),
        }),
    );

    let error = pipeline::write_draft(&mut s.collection, &s.hosts, &s.library, s.feed, &diff.id)
        .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::NotReachable);

    // The feed file is already written, and the upload is on the host.
    // The deletion did not run.
    assert!(live(&s).contains("Album (Deluxe)"));
    assert_eq!(fs::read(s.dir.path().join(&planned.path)).unwrap(), b"one");
    assert!(s.dir.path().join("stray.jpg").exists());
}

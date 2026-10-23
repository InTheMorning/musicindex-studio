//! The write pipeline over the local folder connector (ADR 0007, ADR 0015).

use std::cell::RefCell;
use std::fs;

use studio_core::collection::{Collection, ConnectorId, DraftFile, FeedId};
use studio_core::connector::{self, Connector, Expected, LocalFolder};
use studio_core::feed::{AlternateEnclosure, Enclosure, Entry, Feed, Item, Source};
use studio_core::guid::podcast_guid_for_url;
use studio_core::imports;
use studio_core::pipeline::{self, DiffFile, DiffId, PipelineErrorKind};
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
    store: imports::Store,
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
    let feed = collection
        .add_feed(
            Some("https://music.example/album.xml"),
            Some((connector_id, "album.xml")),
        )
        .unwrap();
    pipeline::refresh(&mut collection, connector.as_ref(), feed).unwrap();
    let store = imports::Store::new(dir.path().join("imports"));
    Setup {
        dir,
        collection,
        connector,
        store,
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
    let diff = pipeline::diff(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    assert!(diff.changed);
    assert!(diff.unified.contains("-    <title>Album</title>"));
    assert!(diff.unified.contains("+    <title>Album (Deluxe)</title>"));

    let version = pipeline::write_draft(
        &mut s.collection,
        s.connector.as_ref(),
        &s.store,
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
    let approved = pipeline::diff(&s.collection, s.connector.as_ref(), s.feed)
        .unwrap()
        .id;
    pipeline::edit_draft(&s.collection, s.feed, |f| f.title = Some("Two".into())).unwrap();
    let error = pipeline::write_draft(
        &mut s.collection,
        s.connector.as_ref(),
        &s.store,
        s.feed,
        &approved,
    )
    .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::DiffNotApproved);
    let error = pipeline::write_draft(
        &mut s.collection,
        s.connector.as_ref(),
        &s.store,
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
    let diff = pipeline::diff(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    let theirs = FEED.replace("Track One", "Track One (edited elsewhere)");
    fs::write(s.dir.path().join("album.xml"), &theirs).unwrap();
    let error = pipeline::write_draft(
        &mut s.collection,
        s.connector.as_ref(),
        &s.store,
        s.feed,
        &diff.id,
    )
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
    let diff = pipeline::diff(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        s.connector.as_ref(),
        &s.store,
        s.feed,
        &diff.id,
    )
    .unwrap();
    assert!(!live(&s).contains("track-1"));
    assert!(s.collection.item_guids(s.feed).unwrap().is_empty());

    let snapshot = s.collection.snapshots(s.feed).unwrap()[0].id;
    pipeline::draft_from_snapshot(&s.collection, s.connector.as_ref(), s.feed, snapshot).unwrap();
    let diff = pipeline::diff(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    assert!(diff.unified.contains("+      <guid>track-1</guid>"));
    pipeline::write_draft(
        &mut s.collection,
        s.connector.as_ref(),
        &s.store,
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
) {
    let dir = tempfile::tempdir().unwrap();
    let folder = LocalFolder::new(dir.path(), Some("https://music.example".into()));
    let collection = Collection::open(&dir.path().join("collection.sqlite")).unwrap();
    let connector_id = collection
        .add_connector(LocalFolder::KIND, "Home folder", &folder.config(), None)
        .unwrap();
    let connector =
        connector::from_row(&collection.connector(connector_id).unwrap(), None).unwrap();
    (dir, collection, connector, connector_id)
}

#[test]
fn a_new_feed_makes_a_draft_with_no_hosted_file() {
    let (dir, mut collection, connector, connector_id) = new_setup();
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
    let (dir, mut collection, connector, connector_id) = new_setup();
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
    let (dir, mut collection, connector, connector_id) = new_setup();
    let id = pipeline::new_feed(
        &mut collection,
        connector.as_ref(),
        connector_id,
        "my-album.xml",
        "New Album",
        "New Artist",
    )
    .unwrap();
    let diff = pipeline::diff(&collection, connector.as_ref(), id).unwrap();
    assert!(diff.changed);
    // The live file is absent, so the diff adds the full file and removes
    // nothing.
    assert!(diff.unified.contains("+<rss"));
    assert!(!diff.unified.contains("\n-<"));

    let store = imports::Store::new(dir.path().join("imports"));
    let version =
        pipeline::write_draft(&mut collection, connector.as_ref(), &store, id, &diff.id).unwrap();
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
    let (dir, mut collection, connector, connector_id) = new_setup();
    let id = pipeline::new_feed(
        &mut collection,
        connector.as_ref(),
        connector_id,
        "my-album.xml",
        "New Album",
        "New Artist",
    )
    .unwrap();
    let diff = pipeline::diff(&collection, connector.as_ref(), id).unwrap();
    // A different tool puts a file at the path before Studio writes it.
    fs::write(
        dir.path().join("my-album.xml"),
        b"<rss><channel><title>Other</title></channel></rss>",
    )
    .unwrap();
    let store = imports::Store::new(dir.path().join("imports"));
    let error = pipeline::write_draft(&mut collection, connector.as_ref(), &store, id, &diff.id)
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

// ---- Pending files (ADR 0026, ADR 0032 items 10 to 12) ----

/// A connector that records the path of each write, so a test can check
/// their sequence.
struct RecordingConnector {
    inner: LocalFolder,
    writes: RefCell<Vec<String>>,
}

impl RecordingConnector {
    fn new(inner: LocalFolder) -> Self {
        Self {
            inner,
            writes: RefCell::new(Vec::new()),
        }
    }

    /// The path of each write, in sequence.
    fn writes(&self) -> Vec<String> {
        self.writes.borrow().clone()
    }
}

impl Connector for RecordingConnector {
    fn read(&self, path: &str) -> Result<Option<connector::FileRead>, connector::ConnectorError> {
        self.inner.read(path)
    }

    fn write(
        &self,
        path: &str,
        bytes: &[u8],
        expected: &Expected,
    ) -> Result<connector::Version, connector::ConnectorError> {
        self.writes.borrow_mut().push(path.to_owned());
        self.inner.write(path, bytes, expected)
    }

    fn public_url(&self, path: &str) -> Option<String> {
        self.inner.public_url(path)
    }

    fn list(&self) -> Result<connector::FileList, connector::ConnectorError> {
        self.inner.list()
    }

    fn delete(
        &self,
        path: &str,
        expected: &connector::Version,
    ) -> Result<(), connector::ConnectorError> {
        self.inner.delete(path, expected)
    }
}

/// A new album draft, with a connector that records each write and a store
/// for the pending files (ADR 0032 items 6 to 12).
struct FileSetup {
    dir: tempfile::TempDir,
    collection: Collection,
    connector: RecordingConnector,
    store: imports::Store,
    feed: FeedId,
}

fn file_setup() -> FileSetup {
    let dir = tempfile::tempdir().unwrap();
    let folder = LocalFolder::new(dir.path(), Some("https://music.example".into()));
    let mut collection = Collection::open(&dir.path().join("collection.sqlite")).unwrap();
    let connector_id = collection
        .add_connector(LocalFolder::KIND, "Home folder", &folder.config(), None)
        .unwrap();
    let connector = RecordingConnector::new(folder);
    let feed = pipeline::new_feed(
        &mut collection,
        &connector,
        connector_id,
        "album/my-album.xml",
        "New Album",
        "New Artist",
    )
    .unwrap();
    let store = imports::Store::new(dir.path().join("imports"));
    FileSetup {
        dir,
        collection,
        connector,
        store,
        feed,
    }
}

/// Put the bytes of a file into the store of the feed, and keep its row
/// (ADR 0026 item 5).
fn put_pending_file(s: &FileSetup, path: &str, bytes: &[u8]) -> DraftFile {
    let source_dir = tempfile::tempdir().unwrap();
    let source = source_dir.path().join("source");
    fs::write(&source, bytes).unwrap();
    let (sha256, size) = s.store.put_file(s.feed, &source).unwrap();
    let file = DraftFile {
        path: path.to_owned(),
        size,
        media_type: "audio/mpeg".to_owned(),
        sha256,
    };
    s.collection.add_draft_file(s.feed, &file).unwrap();
    file
}

/// Add an item with an enclosure at the address of a pending file, so that
/// the draft refers to it (ADR 0032 item 10).
fn add_referenced_file(s: &FileSetup, path: &str, bytes: &[u8]) -> DraftFile {
    let file = put_pending_file(s, path, bytes);
    let url = s.connector.public_url(path).unwrap();
    let item = Item {
        title: Some(path.to_owned()),
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
    file
}

/// Add an item whose only address of a pending file is the `uri` of a
/// `podcast:source` of an alternate enclosure (ADR 0033 item 7).
fn add_referenced_source_file(s: &FileSetup, path: &str, bytes: &[u8]) -> DraftFile {
    let file = put_pending_file(s, path, bytes);
    let url = s.connector.public_url(path).unwrap();
    let item = Item {
        title: Some(path.to_owned()),
        alternate_enclosures: vec![Entry::new(AlternateEnclosure {
            media_type: Some(file.media_type.clone()),
            sources: vec![Entry::new(Source {
                uri: Some(url),
                ..Source::default()
            })],
            ..AlternateEnclosure::default()
        })],
        ..Item::default()
    };
    pipeline::edit_draft(&s.collection, s.feed, move |f| {
        f.items.push(Entry::new(item));
    })
    .unwrap();
    file
}

#[test]
fn a_pending_file_that_only_a_source_refers_to_is_in_the_diff_and_goes_to_the_host() {
    let mut s = file_setup();
    let file = add_referenced_source_file(&s, "album/my-album/01.flac", b"lossless");

    let diff = pipeline::diff(&s.collection, &s.connector, s.feed).unwrap();
    assert_eq!(
        diff.files,
        [DiffFile {
            path: file.path.clone(),
            size: file.size,
            media_type: file.media_type.clone(),
        }]
    );

    pipeline::write_draft(&mut s.collection, &s.connector, &s.store, s.feed, &diff.id).unwrap();

    assert_eq!(
        s.connector.writes(),
        [
            "album/my-album/01.flac".to_owned(),
            "album/my-album.xml".to_owned(),
        ]
    );
    assert!(s.collection.draft_files(s.feed).unwrap().is_empty());
    assert_eq!(
        fs::read(s.dir.path().join("album/my-album/01.flac")).unwrap(),
        b"lossless"
    );
}

#[test]
fn a_write_puts_each_referenced_file_then_the_feed_file_in_sequence() {
    let mut s = file_setup();
    add_referenced_file(&s, "album/my-album/01.mp3", b"one");
    add_referenced_file(&s, "album/my-album/02.mp3", b"two");

    let diff = pipeline::diff(&s.collection, &s.connector, s.feed).unwrap();
    assert_eq!(diff.files.len(), 2);

    pipeline::write_draft(&mut s.collection, &s.connector, &s.store, s.feed, &diff.id).unwrap();

    assert_eq!(
        s.connector.writes(),
        [
            "album/my-album/01.mp3".to_owned(),
            "album/my-album/02.mp3".to_owned(),
            "album/my-album.xml".to_owned(),
        ]
    );
    assert!(s.collection.draft_files(s.feed).unwrap().is_empty());
    assert_eq!(
        fs::read(s.dir.path().join("album/my-album/01.mp3")).unwrap(),
        b"one"
    );
    assert_eq!(
        fs::read(s.dir.path().join("album/my-album/02.mp3")).unwrap(),
        b"two"
    );
}

#[test]
fn a_file_that_exists_at_a_pending_path_stops_the_write_and_a_retry_puts_the_rest() {
    let mut s = file_setup();
    add_referenced_file(&s, "album/my-album/01.mp3", b"one");
    add_referenced_file(&s, "album/my-album/02.mp3", b"two");

    fs::create_dir_all(s.dir.path().join("album/my-album")).unwrap();
    fs::write(s.dir.path().join("album/my-album/01.mp3"), b"already there").unwrap();

    let diff = pipeline::diff(&s.collection, &s.connector, s.feed).unwrap();
    let error = pipeline::write_draft(&mut s.collection, &s.connector, &s.store, s.feed, &diff.id)
        .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::FileExists);
    assert!(!s.dir.path().join("album/my-album.xml").exists());
    assert_eq!(s.collection.draft_files(s.feed).unwrap().len(), 2);

    // The user removes the track with the conflicting file, and keeps the
    // other track.
    let removed_url = s.connector.public_url("album/my-album/01.mp3").unwrap();
    pipeline::edit_draft(&s.collection, s.feed, move |f| {
        f.items.retain(|i| {
            i.value.enclosure.as_ref().and_then(|e| e.url.as_deref()) != Some(removed_url.as_str())
        });
    })
    .unwrap();
    s.collection
        .remove_draft_file(s.feed, "album/my-album/01.mp3")
        .unwrap();

    let diff = pipeline::diff(&s.collection, &s.connector, s.feed).unwrap();
    assert_eq!(diff.files.len(), 1);
    pipeline::write_draft(&mut s.collection, &s.connector, &s.store, s.feed, &diff.id).unwrap();
    assert_eq!(
        fs::read(s.dir.path().join("album/my-album/02.mp3")).unwrap(),
        b"two"
    );
    assert!(s.dir.path().join("album/my-album.xml").exists());
    assert!(s.collection.draft_files(s.feed).unwrap().is_empty());
}

#[test]
fn an_unreferenced_pending_file_does_not_go_to_the_host_and_the_write_removes_its_copy() {
    let mut s = file_setup();
    let referenced = add_referenced_file(&s, "album/my-album/01.mp3", b"one");
    let unreferenced = put_pending_file(&s, "album/my-album/02.mp3", b"two");

    let diff = pipeline::diff(&s.collection, &s.connector, s.feed).unwrap();
    assert_eq!(
        diff.files,
        [DiffFile {
            path: referenced.path.clone(),
            size: referenced.size,
            media_type: referenced.media_type.clone(),
        }]
    );

    pipeline::write_draft(&mut s.collection, &s.connector, &s.store, s.feed, &diff.id).unwrap();

    assert!(!s.dir.path().join("album/my-album/02.mp3").exists());
    assert!(s.store.open(s.feed, &unreferenced.sha256).is_err());
    assert!(s.collection.draft_files(s.feed).unwrap().is_empty());
}

#[test]
fn discard_draft_removes_the_pending_rows_and_their_copies() {
    let s = file_setup();
    let file = put_pending_file(&s, "album/my-album/01.mp3", b"one");
    assert_eq!(s.collection.draft_files(s.feed).unwrap().len(), 1);

    pipeline::discard_draft(&s.collection, &s.store, s.feed).unwrap();

    assert!(s.collection.draft_files(s.feed).unwrap().is_empty());
    assert!(s.store.open(s.feed, &file.sha256).is_err());
}

/// On 2026-10-23 the review of task 018d found pending files with no copy.
/// Two files with the same bytes share one copy, and the removal of one
/// file removed the copy of the other.
#[test]
fn the_removal_of_a_pending_file_keeps_a_copy_that_another_file_uses() {
    let s = file_setup();
    let first = put_pending_file(&s, "album/my-album/a.mp3", b"same");
    let second = put_pending_file(&s, "album/my-album/b.mp3", b"same");
    assert_eq!(first.sha256, second.sha256);

    pipeline::remove_pending_file(&s.collection, &s.store, s.feed, &first).unwrap();
    assert!(s.store.open(s.feed, &second.sha256).is_ok());

    pipeline::remove_pending_file(&s.collection, &s.store, s.feed, &second).unwrap();
    assert!(s.store.open(s.feed, &second.sha256).is_err());
}

#[test]
fn a_pending_file_at_the_feed_image_address_is_referenced() {
    let s = file_setup();
    let file = put_pending_file(&s, "album/my-album/cover.jpg", b"cover");
    let url = s.connector.public_url("album/my-album/cover.jpg").unwrap();
    pipeline::edit_draft(&s.collection, s.feed, move |f| f.image = Some(url)).unwrap();

    let diff = pipeline::diff(&s.collection, &s.connector, s.feed).unwrap();
    assert_eq!(diff.files.len(), 1);
    assert_eq!(diff.files[0].path, file.path);
}

#[test]
fn the_diff_id_changes_when_a_referenced_file_changes() {
    let s = file_setup();
    add_referenced_file(&s, "album/my-album/01.mp3", b"one");
    let first = pipeline::diff(&s.collection, &s.connector, s.feed)
        .unwrap()
        .id;

    s.collection
        .remove_draft_file(s.feed, "album/my-album/01.mp3")
        .unwrap();
    put_pending_file(&s, "album/my-album/01.mp3", b"one (re-recorded)");
    let second = pipeline::diff(&s.collection, &s.connector, s.feed)
        .unwrap()
        .id;

    assert_ne!(first, second);
}

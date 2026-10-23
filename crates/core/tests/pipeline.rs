//! The write pipeline over the local folder connector (ADR 0007, ADR 0015).

use std::fs;

use studio_core::collection::{Collection, ConnectorId, FeedId};
use studio_core::connector::{self, Connector, Expected, LocalFolder};
use studio_core::feed::Feed;
use studio_core::guid::podcast_guid_for_url;
use studio_core::pipeline::{self, DiffId, PipelineErrorKind};
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
    Setup {
        dir,
        collection,
        connector,
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
    let diff = pipeline::diff(&s.collection, s.feed).unwrap();
    assert!(diff.changed);
    assert!(diff.unified.contains("-    <title>Album</title>"));
    assert!(diff.unified.contains("+    <title>Album (Deluxe)</title>"));

    let version =
        pipeline::write_draft(&mut s.collection, s.connector.as_ref(), s.feed, &diff.id).unwrap();
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
    let approved = pipeline::diff(&s.collection, s.feed).unwrap().id;
    pipeline::edit_draft(&s.collection, s.feed, |f| f.title = Some("Two".into())).unwrap();
    let error = pipeline::write_draft(&mut s.collection, s.connector.as_ref(), s.feed, &approved)
        .unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::DiffNotApproved);
    let error = pipeline::write_draft(
        &mut s.collection,
        s.connector.as_ref(),
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
    let diff = pipeline::diff(&s.collection, s.feed).unwrap();
    let theirs = FEED.replace("Track One", "Track One (edited elsewhere)");
    fs::write(s.dir.path().join("album.xml"), &theirs).unwrap();
    let error = pipeline::write_draft(&mut s.collection, s.connector.as_ref(), s.feed, &diff.id)
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
    let diff = pipeline::diff(&s.collection, s.feed).unwrap();
    pipeline::write_draft(&mut s.collection, s.connector.as_ref(), s.feed, &diff.id).unwrap();
    assert!(!live(&s).contains("track-1"));
    assert!(s.collection.item_guids(s.feed).unwrap().is_empty());

    let snapshot = s.collection.snapshots(s.feed).unwrap()[0].id;
    pipeline::draft_from_snapshot(&s.collection, s.connector.as_ref(), s.feed, snapshot).unwrap();
    let diff = pipeline::diff(&s.collection, s.feed).unwrap();
    assert!(diff.unified.contains("+      <guid>track-1</guid>"));
    pipeline::write_draft(&mut s.collection, s.connector.as_ref(), s.feed, &diff.id).unwrap();
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
    let diff = pipeline::diff(&collection, id).unwrap();
    assert!(diff.changed);
    // The live file is absent, so the diff adds the full file and removes
    // nothing.
    assert!(diff.unified.contains("+<rss"));
    assert!(!diff.unified.contains("\n-<"));

    let version = pipeline::write_draft(&mut collection, connector.as_ref(), id, &diff.id).unwrap();
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
    let diff = pipeline::diff(&collection, id).unwrap();
    // A different tool puts a file at the path before Studio writes it.
    fs::write(
        dir.path().join("my-album.xml"),
        b"<rss><channel><title>Other</title></channel></rss>",
    )
    .unwrap();
    let error =
        pipeline::write_draft(&mut collection, connector.as_ref(), id, &diff.id).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::Conflict);
    assert!(collection.draft(id).unwrap().is_some());
}

//! The write pipeline over the local folder connector (ADR 0007, ADR 0015).

use std::fs;

use studio_core::collection::{Collection, FeedId};
use studio_core::connector::{self, Connector, Expected, LocalFolder};
use studio_core::pipeline::{self, DiffId, PipelineErrorKind};

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
    let s = setup();
    let read_only = s
        .collection
        .add_feed(Some("https://elsewhere.example/feed.xml"), None)
        .unwrap();
    let error = pipeline::start_draft(&s.collection, s.connector.as_ref(), read_only).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::ReadOnly);
}

//! A server that holds no assets (ADR 0105): the removal after a write, the
//! expiry of a pending file, the removal at the start and the temporary
//! file of a copy.

use std::fs;
use std::path::PathBuf;

use studio_core::assets;
use studio_core::collection::{AssetRules, Collection, DraftOperationKind, FeedId, LibraryFileId};
use studio_core::connector::{self, Connector, Expected, LocalFolder};
use studio_core::feed::{Enclosure, Entry, Item};
use studio_core::inventory::{self, Hosts};
use studio_core::library::Library;
use studio_core::pipeline::{self, AddressCheck, PipelineErrorKind};

struct PassAll;

impl AddressCheck for PassAll {
    fn check(&self, _url: &str) -> Result<(), String> {
        Ok(())
    }
}

const FEED: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<rss version=\"2.0\" xmlns:podcast=\"https://podcastindex.org/namespace/1.0\">
  <channel>
    <title>Album</title>
    <podcast:guid>917393e3-1b1e-5cef-ace4-edaa54e1f810</podcast:guid>
    <podcast:medium>music</podcast:medium>
    <item><title>Track One</title><guid>track-1</guid></item>
  </channel>
</rss>
";

const DAY: i64 = 86_400;

struct Setup {
    dir: tempfile::TempDir,
    _library_dir: tempfile::TempDir,
    collection: Collection,
    connector: Box<dyn Connector>,
    hosts: Hosts,
    library: Library,
    feed: FeedId,
    incoming: PathBuf,
}

fn setup(off: bool) -> Setup {
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
    // The library is not under the host folder, so a library file is not a
    // host file.
    let library_dir = tempfile::tempdir().unwrap();
    let library = Library::open(library_dir.path()).unwrap();
    let feed = collection
        .add_feed(
            Some("https://music.example/album.xml"),
            Some((connector_id, "album.xml")),
        )
        .unwrap();
    pipeline::refresh(&mut collection, connector.as_ref(), feed).unwrap();
    collection.set_feed_folder(feed, "album").unwrap();
    pipeline::start_draft(&collection, connector.as_ref(), feed).unwrap();
    let incoming = dir.path().join("incoming");
    if off {
        collection
            .set_asset_rules(
                Some(AssetRules {
                    incoming: incoming.clone(),
                }),
                0,
            )
            .unwrap();
    }
    Setup {
        dir,
        _library_dir: library_dir,
        collection,
        connector,
        hosts,
        library,
        feed,
        incoming,
    }
}

fn import(s: &Setup, name: &str, bytes: &[u8]) -> LibraryFileId {
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
    .library_file_id
}

/// Put an item with the planned address of the file into the draft.
fn reference(s: &Setup, id: LibraryFileId) -> String {
    let row = s.collection.feed(s.feed).unwrap();
    let file = s.collection.library_file(id).unwrap();
    let planned = inventory::planned_path(&s.collection, &s.hosts, &row, &file).unwrap();
    let url = planned.url.clone().unwrap();
    let item = Item {
        title: Some(file.path.clone()),
        enclosure: Some(Enclosure {
            url: Some(url.clone()),
            length: Some(file.size.to_string()),
            media_type: Some(file.media_type),
        }),
        ..Item::default()
    };
    pipeline::edit_draft(&s.collection, s.feed, move |f| {
        f.items.push(Entry::new(item));
    })
    .unwrap();
    url
}

fn write(s: &mut Setup) -> Result<(), pipeline::PipelineError> {
    let diff = pipeline::diff(&s.collection, &s.hosts, &s.library, s.feed).unwrap();
    pipeline::write_draft(
        &mut s.collection,
        &s.hosts,
        &s.library,
        &PassAll,
        s.feed,
        &diff.id,
    )
    .map(|_| ())
}

fn bytes_in_folder(s: &Setup, name: &str) -> bool {
    s.library.path("album", name).exists()
}

#[test]
fn after_a_write_each_sent_asset_has_no_bytes_but_keeps_its_hash_and_address() {
    let mut s = setup(true);
    let id = import(&s, "01.mp3", b"one");
    let url = reference(&s, id);
    let before = s.collection.library_file(id).unwrap();
    write(&mut s).unwrap();

    let row = s.collection.library_file(id).unwrap();
    assert!(
        !row.local,
        "ADR 0105 item 5: a sent asset has local false after the write. Call assets::remove_hosted after the scans."
    );
    assert_eq!(
        row.sha256, before.sha256,
        "ADR 0105 item 5: the row keeps its SHA-256"
    );
    assert_eq!(row.size, 3, "ADR 0105 item 5: the row keeps its size");
    assert_eq!(
        row.source_url.as_deref(),
        Some(url.as_str()),
        "ADR 0105 item 5: the row keeps the address of the copy as source_url"
    );
    assert_eq!(row.removed_at, None);
    assert!(
        !bytes_in_folder(&s, "01.mp3"),
        "ADR 0105 item 5: the bytes of a sent asset are not in the release folder"
    );
    assert_eq!(fs::read(s.dir.path().join("album/01.mp3")).unwrap(), b"one");
}

#[test]
fn with_assets_held_a_write_keeps_the_bytes() {
    let mut s = setup(false);
    let id = import(&s, "01.mp3", b"one");
    reference(&s, id);
    write(&mut s).unwrap();
    assert!(
        s.collection.library_file(id).unwrap().local && bytes_in_folder(&s, "01.mp3"),
        "ADR 0105 item 1: with `on` the server works as before"
    );
}

#[test]
fn a_pending_file_keeps_its_bytes_at_6_days_and_loses_them_at_8_days() {
    let mut s = setup(true);
    let id = import(&s, "01.mp3", b"one");
    let url = reference(&s, id);
    let imported = s.collection.library_file(id).unwrap().imported_at;

    let none = assets::expire_pending(&s.collection, &s.library, imported + 6 * DAY).unwrap();
    assert!(
        none.is_empty() && bytes_in_folder(&s, "01.mp3"),
        "ADR 0105 item 8: a pending file stays for 7 days. Compare with PENDING_SECONDS."
    );
    assert_eq!(s.collection.library_file(id).unwrap().removed_at, None);

    let gone = assets::expire_pending(&s.collection, &s.library, imported + 8 * DAY).unwrap();
    assert_eq!(gone, [id], "ADR 0105 item 10: the pending file expires");
    let row = s.collection.library_file(id).unwrap();
    assert!(!row.local && !bytes_in_folder(&s, "01.mp3"));
    assert_eq!(
        row.removed_at,
        Some(imported + 8 * DAY),
        "ADR 0105 item 10: the row records the time of the removal"
    );

    // The draft keeps the address, and the write refuses it.
    let draft = s.collection.draft(s.feed).unwrap().unwrap();
    let draft = String::from_utf8(draft.draft_xml).unwrap();
    assert!(
        draft.contains(&url),
        "ADR 0105 item 10: the draft keeps the address"
    );
    let error = write(&mut s).unwrap_err();
    assert_eq!(
        error.kind(),
        PipelineErrorKind::FileMissing,
        "ADR 0105 item 10: the write refuses with file_missing until the file is added again"
    );
    assert!(error.to_string().contains("01.mp3"), "{error}");
}

#[test]
fn the_start_removes_an_asset_with_a_host_copy_and_keeps_an_asset_with_none() {
    // The mode `off` begins after both files came in, so neither is pending.
    let mut s = setup(false);
    let hosted = import(&s, "01.mp3", b"one");
    let alone = import(&s, "02.mp3", b"two");
    s.connector
        .write("album/01.mp3", b"one", &Expected::Absent)
        .unwrap();
    let host = s.hosts.ids().next().unwrap();
    inventory::scan(&s.collection, &s.hosts, host).unwrap();
    let later = s.collection.library_file(hosted).unwrap().imported_at + 10;
    s.collection
        .set_asset_rules(
            Some(AssetRules {
                incoming: s.incoming.clone(),
            }),
            later,
        )
        .unwrap();

    let removed = assets::remove_hosted(&s.collection, &s.library, None, &|file| {
        assets::find_copy(&s.collection, &s.hosts, file)
    })
    .unwrap();
    assert_eq!(
        removed,
        [hosted],
        "ADR 0105 item 13: remove the bytes of an asset that a host has with the same SHA-256 only"
    );
    assert!(!bytes_in_folder(&s, "01.mp3"));
    assert!(
        s.collection.library_file(alone).unwrap().local && bytes_in_folder(&s, "02.mp3"),
        "ADR 0105 item 12: an asset that no host has stays"
    );

    // The file from before the mode `off` is not pending, also after 30 days.
    let imported = s.collection.library_file(alone).unwrap().imported_at;
    let expired = assets::expire_pending(&s.collection, &s.library, imported + 30 * DAY).unwrap();
    assert!(
        expired.is_empty() && bytes_in_folder(&s, "02.mp3"),
        "ADR 0105 items 8 and 12: only a file that came in after the mode `off` began expires"
    );
}

/// A second host and a copy draft operation of `file` to it.
fn copy_to_second_host(s: &mut Setup, file: LibraryFileId) -> (tempfile::TempDir, PathBuf) {
    pipeline::start_draft(&s.collection, s.connector.as_ref(), s.feed).unwrap();
    let second_dir = tempfile::tempdir().unwrap();
    let second = LocalFolder::new(second_dir.path(), Some("https://second.example".into()));
    let second_id = s
        .collection
        .add_connector(LocalFolder::KIND, "Second", &second.config(), None)
        .unwrap();
    s.hosts.insert(second_id, Box::new(second));
    s.collection
        .add_draft_operation(
            s.feed,
            DraftOperationKind::Copy,
            Some(file),
            second_id,
            "01.mp3",
        )
        .unwrap();
    let path = second_dir.path().join("01.mp3");
    (second_dir, path)
}

fn incoming_is_empty(s: &Setup) -> bool {
    fs::read_dir(&s.incoming).is_ok_and(|mut entries| entries.next().is_none())
        || !s.incoming.exists()
}

#[test]
fn a_copy_to_another_host_leaves_no_file_in_incoming_also_after_a_failure() {
    let mut s = setup(true);
    let id = import(&s, "01.mp3", b"one");
    reference(&s, id);
    write(&mut s).unwrap();
    assert!(!s.collection.library_file(id).unwrap().local);

    // A different file is on the second host already, so the upload fails.
    let (second_dir, target) = copy_to_second_host(&mut s, id);
    fs::write(&target, b"other").unwrap();
    let error = write(&mut s).unwrap_err();
    assert_eq!(error.kind(), PipelineErrorKind::FileExists, "{error}");
    assert!(
        incoming_is_empty(&s),
        "ADR 0105 item 6: Studio removes the temporary file also after a failure. Keep the guard in put_uploads_and_copies."
    );

    // The same copy with a free path works, and leaves no file.
    fs::remove_file(&target).unwrap();
    write(&mut s).unwrap();
    assert_eq!(fs::read(second_dir.path().join("01.mp3")).unwrap(), b"one");
    assert!(
        incoming_is_empty(&s),
        "ADR 0105 item 6: Studio removes the temporary file after the operation"
    );
}

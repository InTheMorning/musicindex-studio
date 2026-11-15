//! The check for a change from outside Studio (ADR 0150).
//!
//! The check reads the public address of a feed, so it needs no vault. It
//! compares the bytes with the last file that Studio read or wrote. It writes
//! nothing to a host and changes no draft. A change still goes through a new
//! read or the merge of ADR 0129.

use crate::announce::{self, AnnouncementState};
use crate::collection::{Collection, CollectionError, ConnectorId, FeedId, FeedRow, OutsideRow};
use crate::digest::sha256_hex;
use crate::discover::MAX_FEED_BYTES;
use crate::feed::Feed;
use crate::inventory::is_placeholder_address;
use crate::media::plain_address;
use crate::musicindex::MusicIndex;
use crate::web::Web;
use crate::xml::Document;

const SETTING: &str = "check_outside";

/// The podping state for one address announced by the last move.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MoveAnnouncement {
    /// The address that Studio announced.
    pub url: String,
    /// `waiting`, `sent`, `failed`, or `not_sent`.
    pub state: &'static str,
    /// The next send time for a waiting announcement.
    pub send_at: Option<i64>,
    /// The completion time for a sent or failed announcement.
    pub finished_at: Option<i64>,
    /// The last error of a failed announcement.
    pub error: Option<String>,
}

/// The last feed move that Studio completed and its announcement states.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct LastWrittenMove {
    /// The address of the feed before the move.
    pub from_url: Option<String>,
    /// The time of the move, in seconds after the Unix epoch.
    pub moved_at: i64,
    /// The announcement state of the address that a move announces.
    pub announcements: Vec<MoveAnnouncement>,
}

/// Tell if the Music Index address is the source of the last written move.
#[must_use]
pub fn index_is_move_source(index_url: Option<&str>, movement: Option<&LastWrittenMove>) -> bool {
    let (Some(index_url), Some(from_url)) = (
        index_url,
        movement.and_then(|movement| movement.from_url.as_deref()),
    ) else {
        return false;
    };
    plain_address(index_url.trim()) == plain_address(from_url.trim())
}

/// Give the last completed move and its queued announcement states (ADR 0184
/// items 3 and 5).
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn last_written_move(
    collection: &Collection,
    row: &FeedRow,
) -> Result<Option<LastWrittenMove>, CollectionError> {
    let Some(movement) = collection.written_feed_move(row.id)? else {
        return Ok(None);
    };
    let queued = announce::recent(collection, row.id)?;
    let announced_at_move = |url: &str| {
        queued.iter().find(|announcement| {
            announcement.url == url && announcement.created_at >= movement.moved_at
        })
    };
    let mut announcements = Vec::new();
    let url = movement.from_url.as_deref().or(row.public_url.as_deref());
    if let Some(url) = url {
        let status = match announced_at_move(url) {
            Some(announcement) => match announcement.state {
                AnnouncementState::Waiting | AnnouncementState::Sending => MoveAnnouncement {
                    url: url.to_owned(),
                    state: "waiting",
                    send_at: Some(announcement.next_at),
                    finished_at: None,
                    error: None,
                },
                AnnouncementState::Sent => MoveAnnouncement {
                    url: url.to_owned(),
                    state: "sent",
                    send_at: None,
                    finished_at: announcement.finished_at,
                    error: None,
                },
                AnnouncementState::Failed => MoveAnnouncement {
                    url: url.to_owned(),
                    state: "failed",
                    send_at: None,
                    finished_at: announcement.finished_at,
                    error: announcement.last_error.clone(),
                },
            },
            None => MoveAnnouncement {
                url: url.to_owned(),
                state: "not_sent",
                send_at: None,
                finished_at: None,
                error: None,
            },
        };
        announcements.push(status);
    }
    Ok(Some(LastWrittenMove {
        from_url: movement.from_url,
        moved_at: movement.moved_at,
        announcements,
    }))
}

/// What the check found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutsideResult {
    /// The hosted file is the file that Studio knows.
    Same,
    /// The hosted file has different bytes.
    Changed,
    /// The hosted file names a new address in `itunes:new-feed-url`.
    Moved {
        /// The new address.
        url: String,
    },
}

/// True unless the user cleared the check box of ADR 0150 item 10.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn is_on(collection: &Collection) -> Result<bool, CollectionError> {
    Ok(collection
        .setting(SETTING)?
        .and_then(|value| value.as_bool())
        .unwrap_or(true))
}

/// Set the check box of ADR 0150 item 10.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn set_on(collection: &Collection, on: bool) -> Result<(), CollectionError> {
    collection.set_setting(SETTING, &serde_json::Value::Bool(on))
}

/// The address to read, or `None` for a feed with no public address (ADR 0150
/// item 6).
#[must_use]
pub fn address_of(row: &FeedRow) -> Option<String> {
    row.public_url
        .as_deref()
        .filter(|url| !is_placeholder_address(url))
        .map(|url| plain_address(url).into_owned())
}

/// Give the folder address when it differs from the public feed address.
#[must_use]
pub fn folder_address(row: &FeedRow) -> Option<String> {
    let url = row
        .address_differences
        .iter()
        .find(|address| address.source == crate::address::AddressSource::Folder)?
        .url
        .as_str();
    (address_of(row).as_deref() != Some(plain_address(url).as_ref()))
        .then(|| plain_address(url).into_owned())
}

/// Find a connected file host and the path on it that give `url`. The
/// function uses the folder address of each file host, so it opens no file
/// host and needs no vault (ADR 0211 items 2 and 11).
#[must_use]
pub fn host_for_address(
    connectors: &[crate::collection::ConnectorRow],
    url: &str,
) -> Option<(ConnectorId, String)> {
    let target = plain_address(url.trim());
    connectors.iter().find_map(|row| {
        let folder = crate::address::folder_url(row)?;
        let folder = plain_address(folder.trim());
        let path = target
            .strip_prefix(folder.trim_end_matches('/'))?
            .strip_prefix('/')?;
        (!path.is_empty()).then(|| (row.id, path.to_owned()))
    })
}

/// Why Studio refused to follow a move.
#[derive(Debug)]
pub enum FollowMoveError {
    /// The feed has a different `podcast:guid`.
    OtherGuid,
    /// A collection update failed.
    Collection(CollectionError),
    /// The draft move failed.
    Pipeline(crate::pipeline::PipelineError),
}

/// Apply a public read after the network request ends.
///
/// # Errors
///
/// Gives `Collection` or `Pipeline` when an update fails. Gives `OtherGuid`
/// when the feed identity differs.
pub fn follow_move_read(
    collection: &mut Collection,
    row: &FeedRow,
    read: &crate::discover::PublicFeed,
) -> Result<bool, FollowMoveError> {
    if row.podcast_guid != read.feed.podcast_guid {
        return Err(FollowMoveError::OtherGuid);
    }
    let connectors = collection
        .connectors()
        .map_err(FollowMoveError::Collection)?;
    let new_host = host_for_address(&connectors, &read.url);
    let old = row.connector_id.zip(row.path.clone());
    if let Some((connector, path)) = old {
        collection
            .add_feed_pointer(connector, &path, row.id)
            .map_err(FollowMoveError::Collection)?;
    }
    if let Some((connector_id, path)) = &new_host {
        collection
            .set_feed_location(row.id, *connector_id, path, Some(&read.url))
            .map_err(FollowMoveError::Collection)?;
    } else {
        collection
            .clear_feed_location(row.id)
            .map_err(FollowMoveError::Collection)?;
        collection
            .set_address(row.id, crate::address::AddressSource::Read, Some(&read.url))
            .map_err(FollowMoveError::Collection)?;
    }
    collection
        .record_state(row.id, &read.feed, &read.version)
        .map_err(FollowMoveError::Collection)?;
    record_read(collection, row.id, &read.bytes).map_err(FollowMoveError::Collection)?;
    crate::pipeline::move_unedited_draft(collection, row.id, &read.bytes, &read.version)
        .map_err(FollowMoveError::Pipeline)?;
    Ok(new_host.is_none())
}

/// Read the hosted file. A failed read gives `None`, and the earlier result
/// stays (ADR 0150 item 7).
#[must_use]
pub fn read_hosted(web: &Web, url: &str) -> Option<Vec<u8>> {
    let limit = u64::try_from(MAX_FEED_BYTES).unwrap_or(u64::MAX);
    web.get(url, limit).ok().map(|file| file.bytes)
}

/// Record the bytes that Studio read from a host. The result of an earlier
/// check ends.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn record_read(
    collection: &Collection,
    id: FeedId,
    bytes: &[u8],
) -> Result<(), CollectionError> {
    collection.set_outside_check(
        id,
        &OutsideRow {
            seen_sha: Some(sha256_hex(&[bytes])),
            ..OutsideRow::default()
        },
    )
}

/// Record the bytes of a write. A public address can give the file from
/// before for a short time, so those bytes are not a change.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn record_write(
    collection: &Collection,
    id: FeedId,
    bytes: &[u8],
) -> Result<(), CollectionError> {
    let before = collection.outside_check(id)?.unwrap_or_default();
    collection.set_outside_check(
        id,
        &OutsideRow {
            seen_sha: Some(sha256_hex(&[bytes])),
            previous_sha: before.seen_sha,
            ..OutsideRow::default()
        },
    )
}

/// Compare the hosted bytes with the last file that Studio knows, and keep the
/// result. A feed with no record takes these bytes as its start.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn apply_read(
    collection: &Collection,
    row: &FeedRow,
    bytes: &[u8],
) -> Result<OutsideResult, CollectionError> {
    let sha = sha256_hex(&[bytes]);
    let mut check = collection.outside_check(row.id)?.unwrap_or_default();
    let known = check.seen_sha.is_none()
        || check.seen_sha.as_deref() == Some(sha.as_str())
        || check.previous_sha.as_deref() == Some(sha.as_str())
        || check.kept_sha.as_deref() == Some(sha.as_str());
    if check.seen_sha.is_none() {
        check.seen_sha = Some(sha.clone());
    }
    let result = if known {
        OutsideResult::Same
    } else {
        compare(row, bytes)
    };
    check.result_sha = (result != OutsideResult::Same).then_some(sha);
    check.moved_url = match &result {
        OutsideResult::Moved { url } => Some(url.clone()),
        _ => None,
    };
    check.changed = result != OutsideResult::Same;
    collection.set_outside_check(row.id, &check)?;
    Ok(result)
}

/// Store a result for bytes read from the folder address.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn apply_result(
    collection: &Collection,
    id: FeedId,
    bytes: &[u8],
    result: OutsideResult,
) -> Result<OutsideResult, CollectionError> {
    let mut check = collection.outside_check(id)?.unwrap_or_default();
    let sha = sha256_hex(&[bytes]);
    // "Keep This File Host" hides the result until the file changes again
    // (ADR 0150 item 4).
    let result = if check.kept_sha.as_deref() == Some(sha.as_str()) {
        OutsideResult::Same
    } else {
        result
    };
    if check.seen_sha.is_none() {
        check.seen_sha = Some(sha.clone());
    }
    check.result_sha = (result != OutsideResult::Same).then_some(sha);
    check.moved_url = match &result {
        OutsideResult::Moved { url } => Some(url.clone()),
        _ => None,
    };
    check.changed = result != OutsideResult::Same;
    collection.set_outside_check(id, &check)?;
    Ok(result)
}

/// The result of the last check, for a read that failed.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn stored_result(
    collection: &Collection,
    id: FeedId,
) -> Result<OutsideResult, CollectionError> {
    let check = collection.outside_check(id)?.unwrap_or_default();
    Ok(match (check.changed, check.moved_url) {
        (_, Some(url)) => OutsideResult::Moved { url },
        (true, None) => OutsideResult::Changed,
        (false, None) => OutsideResult::Same,
    })
}

/// Give the address in the stored move result.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn stored_move_address(
    collection: &Collection,
    id: FeedId,
) -> Result<Option<String>, CollectionError> {
    Ok(collection.outside_check(id)?.and_then(|row| row.moved_url))
}

/// "Keep This File Host": hide the result until the hosted file changes again
/// (ADR 0150 item 4).
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn keep(collection: &Collection, id: FeedId) -> Result<(), CollectionError> {
    let mut check = collection.outside_check(id)?.unwrap_or_default();
    check.kept_sha = check.result_sha.take();
    check.changed = false;
    check.moved_url = None;
    collection.set_outside_check(id, &check)
}

/// Compare public feed bytes with the feed state.
#[must_use]
pub fn compare(row: &FeedRow, bytes: &[u8]) -> OutsideResult {
    let new_url = Document::parse_bytes(bytes)
        .ok()
        .and_then(|doc| Feed::read(&doc).ok())
        .and_then(|feed| feed.new_feed_url);
    let plain = |text: &str| plain_address(text.trim()).into_owned();
    let Some(new_url) = new_url else {
        return OutsideResult::Changed;
    };
    let is_own = row
        .public_url
        .as_deref()
        .is_some_and(|own| plain(own) == plain(&new_url));
    let known = row
        .new_feed_url
        .as_deref()
        .is_some_and(|old| plain(old) == plain(&new_url));
    if is_own || known {
        OutsideResult::Changed
    } else {
        OutsideResult::Moved { url: new_url }
    }
}

/// Compare a folder file with its public address, allowing a move already
/// recorded as the feed public address.
#[must_use]
pub fn compare_folder(row: &FeedRow, bytes: &[u8], folder_url: &str) -> OutsideResult {
    if let OutsideResult::Moved { url } = compare(row, bytes) {
        return OutsideResult::Moved { url };
    }
    let new_url = Document::parse_bytes(bytes)
        .ok()
        .and_then(|doc| Feed::read(&doc).ok())
        .and_then(|feed| feed.new_feed_url);
    match new_url {
        Some(url) if plain_address(url.trim()) != plain_address(folder_url.trim()) => {
            OutsideResult::Moved { url }
        }
        _ => OutsideResult::Changed,
    }
}

/// The address that musicindex gives for this feed when it is different from
/// the public address (ADR 0150 item 5). No answer and no entry give `None`.
#[must_use]
pub fn index_difference(index: &MusicIndex, row: &FeedRow) -> Option<String> {
    let own = address_of(row)?;
    let guid = row.podcast_guid.as_deref()?;
    let found = index.feed_url(guid).ok()??;
    (plain_address(found.trim()) != own.trim()).then_some(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::LocalFolder;

    const FEED: &str = "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\" xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\"><channel><title>A</title><podcast:guid>g</podcast:guid>ITEM</channel></rss>";

    fn body(extra: &str) -> Vec<u8> {
        FEED.replace("ITEM", extra).into_bytes()
    }

    #[test]
    fn an_index_address_matches_the_plain_source_address_of_the_last_move() {
        let movement = LastWrittenMove {
            from_url: Some("https://old.example/a.xml".to_owned()),
            moved_at: 100,
            announcements: Vec::new(),
        };
        assert!(index_is_move_source(
            Some("https://op3.dev/e/old.example/a.xml"),
            Some(&movement)
        ));
        assert!(!index_is_move_source(
            Some("https://other.example/a.xml"),
            Some(&movement)
        ));
    }

    fn fixture() -> (Collection, FeedRow) {
        let mut collection = Collection::open_in_memory().unwrap();
        let id = collection
            .add_feed(Some("https://r2.example/a.xml"), None)
            .unwrap();
        let row = collection.feed(id).unwrap();
        (collection, row)
    }

    fn linked_feed(
        collection: &Collection,
        root: &std::path::Path,
        name: &str,
        base_url: &str,
    ) -> (ConnectorId, LocalFolder) {
        let connector_id = collection
            .add_connector(
                "local",
                name,
                &serde_json::json!({ "root": root, "base_url": base_url }).to_string(),
                None,
            )
            .unwrap();
        (
            connector_id,
            LocalFolder::new(root, Some(base_url.to_owned())),
        )
    }

    fn move_bytes(guid: &str, new_url: &str) -> Vec<u8> {
        format!(
            "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\" xmlns:itunes=\"http://www.itunes.com/dtds/podcast-1.0.dtd\"><channel><title>Album</title><podcast:guid>{guid}</podcast:guid><itunes:new-feed-url>{new_url}</itunes:new-feed-url></channel></rss>"
        )
        .into_bytes()
    }

    #[test]
    fn follow_move_read_uses_a_connected_host_and_records_the_old_pointer() {
        let server = crate::testing::TestServer::start(Vec::new());
        let mut collection = Collection::open_in_memory().unwrap();
        let old_root = tempfile::tempdir().unwrap();
        let new_root = tempfile::tempdir().unwrap();
        let old_base = server.url("/old");
        let new_base = server.url("/new");
        let new_url = format!("{new_base}/album.xml");
        server.set("/new/album.xml", 200, move_bytes("g-1", &new_url));
        let (old_id, _) = linked_feed(&collection, old_root.path(), "Old", &old_base);
        let (new_id, _) = linked_feed(&collection, new_root.path(), "New", &new_base);
        let id = collection
            .add_feed(
                Some(&format!("{old_base}/album.xml")),
                Some((old_id, "album.xml")),
            )
            .unwrap();
        let read = crate::discover::read_public(&Web::new(), &new_url).unwrap();
        collection
            .record_state(id, &read.feed, &read.version)
            .unwrap();
        let row = collection.feed(id).unwrap();

        let mut collection = collection;
        assert!(
            !follow_move_read(&mut collection, &row, &read).unwrap(),
            "ADR 0211 item 13. Fix: report an editable feed when the new host is connected."
        );
        let followed = collection.feed(id).unwrap();
        assert_eq!(
            followed.connector_id,
            Some(new_id),
            "ADR 0211 item 13. Fix: set the matching connected host."
        );
        assert_eq!(
            followed.path.as_deref(),
            Some("album.xml"),
            "ADR 0211 item 13. Fix: keep the path that gives the new address."
        );
        assert!(
            collection.is_feed_pointer(old_id, "album.xml").unwrap(),
            "ADR 0211 item 13. Fix: record the pointer at the old location."
        );
        assert_eq!(
            stored_result(&collection, id).unwrap(),
            OutsideResult::Same,
            "ADR 0211 item 14. Fix: end the moved result after the follow."
        );
    }

    #[test]
    fn follow_move_read_without_a_host_keeps_an_edited_draft_and_snapshots() {
        let server = crate::testing::TestServer::start(Vec::new());
        let mut collection = Collection::open_in_memory().unwrap();
        let old_root = tempfile::tempdir().unwrap();
        let old_base = server.url("/old");
        let new_url = server.url("/new/album.xml");
        server.set("/new/album.xml", 200, move_bytes("g-1", &new_url));
        let (old_id, _) = linked_feed(&collection, old_root.path(), "Old", &old_base);
        let id = collection
            .add_feed(
                Some(&format!("{old_base}/album.xml")),
                Some((old_id, "album.xml")),
            )
            .unwrap();
        let original = move_bytes("g-1", &new_url);
        let initial_read = crate::discover::read_public(&Web::new(), &new_url).unwrap();
        collection
            .record_state(id, &initial_read.feed, &initial_read.version)
            .unwrap();
        let row = collection.feed(id).unwrap();
        let edited = [original.as_slice(), b"<!-- edit -->"].concat();
        collection
            .save_draft(
                id,
                &crate::collection::DraftRow {
                    base_version: Some(initial_read.version.clone()),
                    base_xml: original.clone(),
                    draft_xml: edited.clone(),
                },
            )
            .unwrap();
        let version = initial_read.version.clone();
        collection.add_snapshot(id, &version, &original).unwrap();
        let before_snapshots = collection.snapshots(id).unwrap();
        let read = initial_read;

        let mut collection = collection;
        assert!(
            follow_move_read(&mut collection, &row, &read).unwrap(),
            "ADR 0211 item 15. Fix: report read-only when no host gives the new address."
        );
        let followed = collection.feed(id).unwrap();
        assert_eq!(
            followed.connector_id, None,
            "ADR 0211 item 15. Fix: clear the old connector for a read-only feed."
        );
        assert_eq!(
            followed.public_url.as_deref(),
            Some(new_url.as_str()),
            "ADR 0211 item 15. Fix: set the public address to the read address."
        );
        let draft = collection.draft(id).unwrap().unwrap();
        assert_eq!(
            draft.draft_xml, edited,
            "ADR 0211 item 15. Fix: keep a draft with edits."
        );
        assert_eq!(
            collection.snapshots(id).unwrap(),
            before_snapshots,
            "ADR 0211 item 15. Fix: keep the feed snapshots."
        );
    }

    #[test]
    fn follow_move_read_refuses_a_different_guid_without_changing_the_feed_row() {
        let server = crate::testing::TestServer::start(Vec::new());
        let mut collection = Collection::open_in_memory().unwrap();
        let old_root = tempfile::tempdir().unwrap();
        let old_base = server.url("/old");
        let new_url = server.url("/new/album.xml");
        let old_read_url = server.url("/old/album.xml");
        server.set("/old/album.xml", 200, move_bytes("g-1", &new_url));
        server.set("/new/album.xml", 200, move_bytes("other", &new_url));
        let (old_id, _) = linked_feed(&collection, old_root.path(), "Old", &old_base);
        let id = collection
            .add_feed(
                Some(&format!("{old_base}/album.xml")),
                Some((old_id, "album.xml")),
            )
            .unwrap();
        let initial = crate::discover::read_public(&Web::new(), &old_read_url).unwrap();
        collection
            .record_state(id, &initial.feed, &initial.version)
            .unwrap();
        let row = collection.feed(id).unwrap();
        let read = crate::discover::read_public(&Web::new(), &new_url).unwrap();
        let before = collection.feed(id).unwrap();

        let error = follow_move_read(&mut collection, &row, &read).unwrap_err();
        assert!(
            matches!(error, FollowMoveError::OtherGuid),
            "ADR 0211 item 12. Fix: refuse a move when podcast:guid differs."
        );
        assert_eq!(
            collection.feed(id).unwrap(),
            before,
            "ADR 0211 item 12. Fix: check podcast:guid before changing the feed row."
        );
    }

    #[test]
    fn a_move_keeps_its_time_and_the_state_of_its_queued_podpings() {
        let (collection, row) = fixture();
        collection
            .record_written_feed_move(row.id, Some("https://old.example/a.xml"), 100)
            .unwrap();
        announce::queue(
            &collection,
            row.id,
            &[
                announce::PlannedAnnouncement {
                    url: "https://r2.example/a.xml".to_owned(),
                    podping: true,
                },
                announce::PlannedAnnouncement {
                    url: "https://old.example/a.xml".to_owned(),
                    podping: true,
                },
            ],
            &announce::Timing::DEFAULT,
            100,
        )
        .unwrap();

        let found = last_written_move(&collection, &row).unwrap().unwrap();
        assert_eq!(found.from_url.as_deref(), Some("https://old.example/a.xml"));
        assert_eq!(found.moved_at, 100);
        assert_eq!(
            found.announcements.len(),
            1,
            "ADR 0189 item 3: show one podping line for the address before the move. Fix: select movement.from_url."
        );
        assert_eq!(found.announcements[0].state, "waiting");
        assert_eq!(found.announcements[0].send_at, Some(130));
        assert_eq!(
            found.announcements[0].url, "https://old.example/a.xml",
            "ADR 0189 item 3: show the address before the move. Fix: select movement.from_url."
        );
    }

    #[test]
    fn the_same_bytes_give_no_result_and_a_first_read_is_the_start() {
        let (collection, row) = fixture();
        assert_eq!(
            apply_read(&collection, &row, &body("")).unwrap(),
            OutsideResult::Same
        );
        assert_eq!(
            apply_read(&collection, &row, &body("")).unwrap(),
            OutsideResult::Same
        );
    }

    #[test]
    fn other_bytes_give_changed_and_a_new_address_gives_moved() {
        let (collection, row) = fixture();
        record_read(&collection, row.id, &body("")).unwrap();
        assert_eq!(
            apply_read(&collection, &row, &body("<description>x</description>")).unwrap(),
            OutsideResult::Changed
        );
        let moved = body("<itunes:new-feed-url>https://other.example/a.xml</itunes:new-feed-url>");
        assert_eq!(
            apply_read(&collection, &row, &moved).unwrap(),
            OutsideResult::Moved {
                url: "https://other.example/a.xml".to_owned()
            },
            "ADR 0150 item 3"
        );
        assert_eq!(
            stored_result(&collection, row.id).unwrap(),
            OutsideResult::Moved {
                url: "https://other.example/a.xml".to_owned()
            },
            "ADR 0150 item 7: a failed read keeps the earlier result."
        );
    }

    #[test]
    fn a_folder_pointer_is_a_move_when_the_public_address_already_moved() {
        let (_, mut row) = fixture();
        row.public_url = Some("https://new.example/a.xml".to_owned());
        let pointer = body("<itunes:new-feed-url>https://new.example/a.xml</itunes:new-feed-url>");
        assert_eq!(
            compare_folder(&row, &pointer, "https://old.example/a.xml"),
            OutsideResult::Moved {
                url: "https://new.example/a.xml".to_owned()
            },
            "ADR 0211 item 3. Fix: compare the pointer with its folder address even when the public address matches."
        );
        assert_eq!(
            compare_folder(&row, &body(""), "https://old.example/a.xml"),
            compare(&row, &body("")),
            "ADR 0211 item 1. Fix: use the public-address result for a feed file at the folder address."
        );
    }

    #[test]
    fn a_tag_with_the_own_address_is_a_change_and_not_a_move() {
        let (collection, row) = fixture();
        record_read(&collection, row.id, &body("")).unwrap();
        let own = body("<itunes:new-feed-url>https://r2.example/a.xml</itunes:new-feed-url>");
        assert_eq!(
            apply_read(&collection, &row, &own).unwrap(),
            OutsideResult::Changed
        );
    }

    #[test]
    fn a_read_ends_the_result() {
        let (collection, row) = fixture();
        record_read(&collection, row.id, &body("")).unwrap();
        apply_read(&collection, &row, &body("<description>x</description>")).unwrap();
        record_read(&collection, row.id, &body("<description>x</description>")).unwrap();
        assert_eq!(
            stored_result(&collection, row.id).unwrap(),
            OutsideResult::Same
        );
        assert_eq!(
            apply_read(&collection, &row, &body("<description>x</description>")).unwrap(),
            OutsideResult::Same
        );
    }

    #[test]
    fn the_file_from_before_a_write_is_not_a_change() {
        let (collection, row) = fixture();
        record_read(&collection, row.id, &body("")).unwrap();
        record_write(&collection, row.id, &body("<description>new</description>")).unwrap();
        assert_eq!(
            apply_read(&collection, &row, &body("")).unwrap(),
            OutsideResult::Same
        );
        assert_eq!(
            apply_read(&collection, &row, &body("<description>new</description>")).unwrap(),
            OutsideResult::Same
        );
    }

    #[test]
    fn keep_hides_the_result_until_the_bytes_change_again() {
        let (collection, row) = fixture();
        record_read(&collection, row.id, &body("")).unwrap();
        let moved = body("<itunes:new-feed-url>https://other.example/a.xml</itunes:new-feed-url>");
        apply_read(&collection, &row, &moved).unwrap();
        keep(&collection, row.id).unwrap();
        assert_eq!(
            stored_result(&collection, row.id).unwrap(),
            OutsideResult::Same
        );
        assert_eq!(
            apply_read(&collection, &row, &moved).unwrap(),
            OutsideResult::Same,
            "ADR 0150 item 4"
        );
        assert_eq!(
            apply_read(&collection, &row, &body("<description>y</description>")).unwrap(),
            OutsideResult::Changed
        );
    }

    #[test]
    fn the_setting_starts_on_and_a_placeholder_has_no_address() {
        let (collection, mut row) = fixture();
        assert!(is_on(&collection).unwrap());
        set_on(&collection, false).unwrap();
        assert!(!is_on(&collection).unwrap());
        assert_eq!(
            address_of(&row).as_deref(),
            Some("https://r2.example/a.xml")
        );
        row.public_url = None;
        assert_eq!(address_of(&row), None, "ADR 0150 item 6");
    }

    #[test]
    fn the_check_changes_no_draft() {
        let (collection, row) = fixture();
        record_read(&collection, row.id, &body("")).unwrap();
        apply_read(&collection, &row, &body("<description>x</description>")).unwrap();
        assert!(
            collection.draft(row.id).unwrap().is_none(),
            "ADR 0150 item 8"
        );
    }
}

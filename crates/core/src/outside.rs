//! The check for a change from outside Studio (ADR 0150).
//!
//! The check reads the public address of a feed, so it needs no vault. It
//! compares the bytes with the last file that Studio read or wrote. It writes
//! nothing to a host and changes no draft. A change still goes through a new
//! read or the merge of ADR 0129.

use crate::announce::{self, AnnouncementState};
use crate::collection::{Collection, CollectionError, FeedId, FeedRow, OutsideRow};
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
    /// The announcement state of each address, new address first.
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
    for url in [row.public_url.as_deref(), movement.from_url.as_deref()]
        .into_iter()
        .flatten()
    {
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

fn compare(row: &FeedRow, bytes: &[u8]) -> OutsideResult {
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
        assert_eq!(found.announcements.len(), 2);
        assert_eq!(found.announcements[0].state, "waiting");
        assert_eq!(found.announcements[0].send_at, Some(130));
        assert_eq!(found.announcements[1].url, "https://old.example/a.xml");
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

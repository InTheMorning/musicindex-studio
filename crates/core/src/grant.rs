//! Show grants (ADR 0054 items 11 to 17).
//!
//! A show grant lets each connected app change the status and the
//! `podcast:liveValue` of one live item of one feed. The approved write of a
//! draft records the grant. Each granted change is then a write with no new
//! diff, through [`crate::pipeline::write_granted`].

use rusqlite::OptionalExtension;
use serde::Serialize;

use crate::collection::{Collection, CollectionError, FeedId};
use crate::feed::{Feed, LiveItem};
use crate::live::LiveStatus;

/// The time in seconds that a grant stays active after the `end` of its
/// live item (ADR 0054 item 14). A live item with no `end` uses its
/// `start`.
pub const GRANT_MARGIN: i64 = 24 * 60 * 60;

/// A change that a show grant covers (ADR 0054 item 13).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrantChange {
    /// The status changes from `pending` to `live`.
    GoLive,
    /// The status changes from `live` to `ended`.
    End,
    /// Set or change `podcast:liveValue`.
    SetLiveValue {
        /// The `uri` attribute.
        uri: String,
        /// The `protocol` attribute.
        protocol: String,
    },
    /// Remove `podcast:liveValue`.
    RemoveLiveValue,
}

impl GrantChange {
    /// The text of the change in the record of a granted write.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GoLive => "go_live",
            Self::End => "end",
            Self::SetLiveValue { .. } => "set_live_value",
            Self::RemoveLiveValue => "remove_live_value",
        }
    }
}

/// One show grant that a write recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShowGrant {
    /// The ID of the grant.
    pub id: i64,
    /// The feed of the live item.
    pub feed_id: FeedId,
    /// The `guid` of the live item.
    pub item_guid: String,
    /// The time of the write that recorded the grant, in seconds since the
    /// Unix epoch.
    pub created_at: i64,
    /// The time when the grant ended, or `None` when it did not end.
    pub ended_at: Option<i64>,
}

/// One granted write (ADR 0054 item 17).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GrantWrite {
    /// The ID of the record.
    pub id: i64,
    /// The grant that covered the write.
    pub grant_id: i64,
    /// The name of the connected app.
    pub app_name: String,
    /// The text of the [`GrantChange`].
    pub change: String,
    /// The time of the write, in seconds since the Unix epoch.
    pub written_at: i64,
}

const GRANT_COLUMNS: &str = "id, feed_id, item_guid, created_at, ended_at";

fn grant_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ShowGrant> {
    Ok(ShowGrant {
        id: row.get(0)?,
        feed_id: FeedId(row.get(1)?),
        item_guid: row.get(2)?,
        created_at: row.get(3)?,
        ended_at: row.get(4)?,
    })
}

/// Record a grant at a write. A grant of the same live item that ended
/// starts again, with no `ended_at`.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn record(
    collection: &Collection,
    feed_id: FeedId,
    item_guid: &str,
    now: i64,
) -> Result<(), CollectionError> {
    collection.db().execute(
        "INSERT INTO show_grants (feed_id, item_guid, created_at, ended_at) VALUES (?1, ?2, ?3, NULL)
         ON CONFLICT (feed_id, item_guid) DO UPDATE SET created_at = ?3, ended_at = NULL",
        rusqlite::params![feed_id.0, item_guid, now],
    )?;
    Ok(())
}

/// The grant with this ID, or `None`.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn grant(collection: &Collection, id: i64) -> Result<Option<ShowGrant>, CollectionError> {
    Ok(collection
        .db()
        .query_row(
            &format!("SELECT {GRANT_COLUMNS} FROM show_grants WHERE id = ?1"),
            [id],
            grant_row,
        )
        .optional()?)
}

/// The grant of one live item, or `None`.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn grant_for(
    collection: &Collection,
    feed_id: FeedId,
    item_guid: &str,
) -> Result<Option<ShowGrant>, CollectionError> {
    Ok(collection
        .db()
        .query_row(
            &format!(
                "SELECT {GRANT_COLUMNS} FROM show_grants WHERE feed_id = ?1 AND item_guid = ?2"
            ),
            rusqlite::params![feed_id.0, item_guid],
            grant_row,
        )
        .optional()?)
}

/// The grants of a feed, the oldest first.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn grants(collection: &Collection, feed_id: FeedId) -> Result<Vec<ShowGrant>, CollectionError> {
    let mut statement = collection.db().prepare(&format!(
        "SELECT {GRANT_COLUMNS} FROM show_grants WHERE feed_id = ?1 ORDER BY id"
    ))?;
    let rows = statement.query_map([feed_id.0], grant_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Each grant of each feed that has no `ended_at`, the oldest first. A
/// grant in this list is not always active: see [`active`].
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn open_grants(collection: &Collection) -> Result<Vec<ShowGrant>, CollectionError> {
    let mut statement = collection.db().prepare(&format!(
        "SELECT {GRANT_COLUMNS} FROM show_grants WHERE ended_at IS NULL ORDER BY id"
    ))?;
    let rows = statement.query_map([], grant_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Set `ended_at` of a grant. A grant that ended already keeps its time.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn end(collection: &Collection, id: i64, now: i64) -> Result<(), CollectionError> {
    collection.db().execute(
        "UPDATE show_grants SET ended_at = ?2 WHERE id = ?1 AND ended_at IS NULL",
        rusqlite::params![id, now],
    )?;
    Ok(())
}

/// Record a granted write (ADR 0054 item 17).
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn add_write(
    collection: &Collection,
    grant_id: i64,
    app_name: &str,
    change: &GrantChange,
    now: i64,
) -> Result<(), CollectionError> {
    collection.db().execute(
        "INSERT INTO grant_writes (grant_id, app_name, change, written_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![grant_id, app_name, change.as_str(), now],
    )?;
    Ok(())
}

/// The granted writes of a grant, the oldest first.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn writes(collection: &Collection, grant_id: i64) -> Result<Vec<GrantWrite>, CollectionError> {
    let mut statement = collection.db().prepare(
        "SELECT id, grant_id, app_name, change, written_at FROM grant_writes
         WHERE grant_id = ?1 ORDER BY id",
    )?;
    let rows = statement.query_map([grant_id], |row| {
        Ok(GrantWrite {
            id: row.get(0)?,
            grant_id: row.get(1)?,
            app_name: row.get(2)?,
            change: row.get(3)?,
            written_at: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The live item of `feed` with this `guid`, or `None`.
#[must_use]
pub fn live_item<'a>(feed: &'a Feed, item_guid: &str) -> Option<&'a LiveItem> {
    feed.live_items
        .iter()
        .map(|entry| &entry.value)
        .find(|live| live.item.guid.as_deref() == Some(item_guid))
}

/// True when `feed` has a live item with this `guid` and the status
/// `pending`. Only such an item can get a grant in the draft (ADR 0054
/// item 11).
#[must_use]
pub fn is_pending_live_item(feed: &Feed, item_guid: &str) -> bool {
    live_item(feed, item_guid).is_some_and(|live| {
        live.status.as_deref().and_then(LiveStatus::parse) == Some(LiveStatus::Pending)
    })
}

/// The grant of a live item when it is active at `now`, in seconds since
/// the Unix epoch (ADR 0054 item 14). `hosted` is the feed that the host
/// gives now.
///
/// A grant is active when all of these conditions are true:
///
/// - The grant has no `ended_at`.
/// - The live item is in `hosted`, and its status is not `ended`.
/// - `now` is less than 24 hours after the `end` of the live item, or
///   after its `start` when it has no `end`.
///
/// A `start` or an `end` that [`parse_time`] cannot read ends the grant.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn active(
    collection: &Collection,
    feed_id: FeedId,
    item_guid: &str,
    hosted: &Feed,
    now: u64,
) -> Result<Option<ShowGrant>, CollectionError> {
    let Some(grant) = grant_for(collection, feed_id, item_guid)? else {
        return Ok(None);
    };
    if grant.ended_at.is_some() {
        return Ok(None);
    }
    let Some(live) = live_item(hosted, item_guid) else {
        return Ok(None);
    };
    if live.status.as_deref().and_then(LiveStatus::parse) == Some(LiveStatus::Ended) {
        return Ok(None);
    }
    let Some(limit) = grant_limit(live) else {
        return Ok(None);
    };
    let now = i64::try_from(now).unwrap_or(i64::MAX);
    Ok((now < limit).then_some(grant))
}

/// The time when the grant of a live item ends: 24 hours after its `end`,
/// or after its `start` when it has no `end`. Gives `None` when a time is
/// blank or [`parse_time`] cannot read it.
#[must_use]
pub fn grant_limit(live: &LiveItem) -> Option<i64> {
    let start = parse_time(live.start.as_deref()?)?;
    // A blank end is the same as no end, because the editor can send an
    // empty text.
    let end = match live.end.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(text) => Some(parse_time(text)?),
    };
    end.unwrap_or(start).checked_add(GRANT_MARGIN)
}

/// Read an ISO 8601 time with a zone, such as `2026-11-01T10:00:00Z` or
/// `2021-09-26T07:30:00.000-0600`. Gives the time in seconds since the
/// Unix epoch.
///
/// The time must have the date, the hours and the minutes. The seconds and
/// a fraction of a second are optional. The zone is `Z`, `+HH:MM`, `+HHMM`
/// or `+HH`, or the same with `-`. A time with no zone gives `None`,
/// because Studio cannot know the zone that the user meant.
#[must_use]
pub fn parse_time(text: &str) -> Option<i64> {
    let text = text.trim();
    let (date, rest) = text.split_once('T')?;
    let mut date_parts = date.split('-');
    let year: i64 = digits(date_parts.next()?, 4)?;
    let month: i64 = digits(date_parts.next()?, 2)?;
    let day: i64 = digits(date_parts.next()?, 2)?;
    if date_parts.next().is_some() || !(1..=12).contains(&month) {
        return None;
    }
    if day < 1 || day > days_in_month(year, month) {
        return None;
    }
    let zone_at = rest.find(['Z', '+', '-'])?;
    let (clock, zone) = rest.split_at(zone_at);
    let mut clock_parts = clock.split(':');
    let hour = digits(clock_parts.next()?, 2)?;
    let minute = digits(clock_parts.next()?, 2)?;
    let second = match clock_parts.next() {
        None => 0,
        Some(part) => {
            let whole = part.split_once('.').map_or(part, |(whole, fraction)| {
                if fraction.is_empty() || !fraction.bytes().all(|b| b.is_ascii_digit()) {
                    // A bad fraction makes the whole value bad.
                    ""
                } else {
                    whole
                }
            });
            digits(whole, 2)?
        }
    };
    if clock_parts.next().is_some() || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let offset = zone_offset(zone)?;
    let days = days_from_civil(year, month, day);
    Some(days * 86_400 + hour * 3600 + minute * 60 + second - offset)
}

/// The value of a text of exactly `count` ASCII digits.
fn digits(text: &str, count: usize) -> Option<i64> {
    if text.len() != count || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// The offset of a zone in seconds east of UTC.
fn zone_offset(zone: &str) -> Option<i64> {
    if zone == "Z" {
        return Some(0);
    }
    let sign = match zone.as_bytes().first()? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let rest = &zone[1..];
    let (hours, minutes) = match rest.len() {
        2 => (digits(rest, 2)?, 0),
        4 => (digits(&rest[..2], 2)?, digits(&rest[2..], 2)?),
        5 if rest.as_bytes()[2] == b':' => (digits(&rest[..2], 2)?, digits(&rest[3..], 2)?),
        _ => return None,
    };
    if hours > 23 || minutes > 59 {
        return None;
    }
    Some(sign * (hours * 3600 + minutes * 60))
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// The number of days from 1970-01-01 to a date of the Gregorian calendar.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_index = (month + 9) % 12;
    let day_of_year = (153 * month_index + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// True when the only differences between `before` and `after` are the
/// status and the `podcast:liveValue` of the live item with this `guid`
/// (ADR 0054 item 13). A status change must also be `pending` to `live`
/// or `live` to `ended`.
///
/// The function compares the two typed models. In copies of the two
/// models, it makes the status and the live value of that live item equal.
/// Then all other parts must be equal. Each other entry also keeps its
/// source element in the compare. Thus a change that the model does not
/// hold also makes the result false.
#[must_use]
pub fn covered(before: &Feed, after: &Feed, item_guid: &str) -> bool {
    let (Some(index), Some(after_index)) = (
        only_position(before, item_guid),
        only_position(after, item_guid),
    ) else {
        return false;
    };
    if index != after_index {
        return false;
    }
    let old = &before.live_items[index];
    let new = &after.live_items[index];
    if old.key() != new.key() || !permitted_status(&old.value, &new.value) {
        return false;
    }
    // The entry of `after` gets the source element of `before`, the model
    // of `after`, and the status and the live value of `before`.
    let mut entry = old.clone();
    entry.value = new.value.clone();
    entry.value.status.clone_from(&old.value.status);
    entry.value.live_value.clone_from(&old.value.live_value);
    let mut changed = after.clone();
    changed.live_items[index] = entry;
    changed == *before
}

/// The position of the one live item with this `guid`. Gives `None` when
/// no live item or more than one live item has it.
fn only_position(feed: &Feed, item_guid: &str) -> Option<usize> {
    let mut found = feed
        .live_items
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.value.item.guid.as_deref() == Some(item_guid))
        .map(|(index, _)| index);
    let first = found.next()?;
    found.next().is_none().then_some(first)
}

/// True when the status does not change, or changes from `pending` to
/// `live` or from `live` to `ended`.
fn permitted_status(before: &LiveItem, after: &LiveItem) -> bool {
    if before.status == after.status {
        return true;
    }
    let parse = |live: &LiveItem| live.status.as_deref().and_then(LiveStatus::parse);
    matches!(
        (parse(before), parse(after)),
        (Some(LiveStatus::Pending), Some(LiveStatus::Live))
            | (Some(LiveStatus::Live), Some(LiveStatus::Ended))
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::{ContentLink, Entry, LiveValue};
    use crate::xml::Document;

    const FEED: &str = "<rss xmlns:podcast=\"https://podcastindex.org/namespace/1.0\"><channel>\
        <title>Album</title><podcast:guid>g-1</podcast:guid>\
        <item><guid>t-1</guid><title>Track</title></item>\
        <podcast:liveItem status=\"pending\" start=\"2026-11-01T10:00:00Z\" end=\"2026-11-01T12:00:00Z\">\
        <title>Show</title><guid>show-1</guid>\
        <podcast:contentLink href=\"https://example.com/show\">Watch</podcast:contentLink>\
        </podcast:liveItem>\
        <podcast:liveItem status=\"pending\" start=\"2026-11-02T10:00:00Z\">\
        <title>Other</title><guid>show-2</guid>\
        </podcast:liveItem>\
        </channel></rss>";

    fn feed() -> Feed {
        Feed::read(&Document::parse(FEED).unwrap()).unwrap()
    }

    /// The feed after a change of the model, read again from its XML, as
    /// a granted write makes it.
    fn changed(change: impl FnOnce(&mut Feed)) -> Feed {
        let mut doc = Document::parse(FEED).unwrap();
        let mut model = Feed::read(&doc).unwrap();
        change(&mut model);
        model.write(&mut doc).unwrap();
        Feed::read(&Document::parse(&doc.write()).unwrap()).unwrap()
    }

    fn show(feed: &mut Feed) -> &mut LiveItem {
        &mut feed.live_items[0].value
    }

    /// A change of the model for a test case.
    type Change = Box<dyn FnOnce(&mut Feed)>;

    const END: i64 = 1_793_534_400; // 2026-11-01T12:00:00Z

    #[test]
    fn a_time_with_a_zone_is_read_and_a_time_with_no_zone_is_not() {
        assert_eq!(parse_time("2026-11-01T12:00:00Z"), Some(END));
        assert_eq!(parse_time("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_time("1970-01-01T00:00Z"), Some(0));
        assert_eq!(parse_time("2026-11-01T12:00:00.000Z"), Some(END));
        assert_eq!(parse_time("2026-11-01T06:00:00-06:00"), Some(END));
        assert_eq!(parse_time("2026-11-01T06:00:00.000-0600"), Some(END));
        assert_eq!(parse_time("2026-11-01T14:00:00+02"), Some(END));
        assert_eq!(parse_time("2024-02-29T00:00:00Z"), Some(1_709_164_800));
        for bad in [
            "2026-11-01T12:00:00",
            "2026-11-01",
            "2026-13-01T12:00:00Z",
            "2026-02-29T12:00:00Z",
            "2026-11-01T24:00:00Z",
            "2026-11-01T12:00:00.Z",
            "2026-11-01T12:00:00+2",
            "soon",
            "",
        ] {
            assert_eq!(parse_time(bad), None, "{bad}");
        }
    }

    #[test]
    fn a_grant_is_active_until_24_hours_after_the_end() {
        let mut collection = Collection::open_in_memory().unwrap();
        let id = collection.add_feed(None, None).unwrap();
        let hosted = feed();
        let at = |now: i64| active(&collection, id, "show-1", &hosted, now.try_into().unwrap());
        assert_eq!(at(END).unwrap(), None, "no grant yet");
        record(&collection, id, "show-1", 5).unwrap();
        assert!(at(END).unwrap().is_some());
        assert!(at(END + GRANT_MARGIN - 1).unwrap().is_some());
        assert_eq!(at(END + GRANT_MARGIN).unwrap(), None);
        assert_eq!(at(END + GRANT_MARGIN + 1).unwrap(), None);
        let grant = grant_for(&collection, id, "show-1").unwrap().unwrap();
        end(&collection, grant.id, 9).unwrap();
        assert_eq!(at(END).unwrap(), None, "an ended grant");
        // A new record starts the grant again.
        record(&collection, id, "show-1", 10).unwrap();
        let again = at(END).unwrap().unwrap();
        assert_eq!(again.id, grant.id);
        assert_eq!(again.ended_at, None);
    }

    #[test]
    fn a_grant_with_no_end_uses_the_start_and_a_bad_time_ends_it() {
        let mut collection = Collection::open_in_memory().unwrap();
        let id = collection.add_feed(None, None).unwrap();
        record(&collection, id, "show-2", 1).unwrap();
        let start = parse_time("2026-11-02T10:00:00Z").unwrap();
        let mut hosted = feed();
        let now = u64::try_from(start + GRANT_MARGIN - 1).unwrap();
        assert!(
            active(&collection, id, "show-2", &hosted, now)
                .unwrap()
                .is_some()
        );
        let later = now + 1;
        assert_eq!(
            active(&collection, id, "show-2", &hosted, later).unwrap(),
            None
        );
        hosted.live_items[1].value.end = Some(" ".into());
        assert!(
            active(&collection, id, "show-2", &hosted, now)
                .unwrap()
                .is_some()
        );
        hosted.live_items[1].value.end = Some("tomorrow".into());
        assert_eq!(
            active(&collection, id, "show-2", &hosted, now).unwrap(),
            None
        );
        hosted.live_items[1].value.end = None;
        hosted.live_items[1].value.start = Some("2026-11-02T10:00:00".into());
        assert_eq!(
            active(&collection, id, "show-2", &hosted, now).unwrap(),
            None
        );
    }

    #[test]
    fn a_grant_ends_when_the_item_leaves_the_feed_or_has_the_status_ended() {
        let mut collection = Collection::open_in_memory().unwrap();
        let id = collection.add_feed(None, None).unwrap();
        record(&collection, id, "show-1", 1).unwrap();
        let now = u64::try_from(END).unwrap();
        let mut hosted = feed();
        show(&mut hosted).status = Some("ended".into());
        assert_eq!(
            active(&collection, id, "show-1", &hosted, now).unwrap(),
            None
        );
        let mut hosted = feed();
        hosted.live_items.remove(0);
        assert_eq!(
            active(&collection, id, "show-1", &hosted, now).unwrap(),
            None
        );
    }

    #[test]
    fn the_status_and_the_live_value_are_covered() {
        let before = feed();
        let after = changed(|f| show(f).status = Some("live".into()));
        assert!(covered(&before, &after, "show-1"));
        let after = changed(|f| {
            show(f).live_value = Some(LiveValue {
                uri: Some("wss://example.com/value".into()),
                protocol: Some("socket.io".into()),
            });
        });
        assert!(covered(&before, &after, "show-1"));
        assert!(covered(&before, &before, "show-1"));
        let live = changed(|f| show(f).status = Some("live".into()));
        let ended = changed(|f| show(f).status = Some("ended".into()));
        assert!(covered(&live, &ended, "show-1"));
    }

    #[test]
    fn each_other_change_is_not_covered() {
        let before = feed();
        let cases: Vec<(&str, Change)> = vec![
            ("feed title", Box::new(|f| f.title = Some("New".into()))),
            (
                "track title",
                Box::new(|f| f.items[0].value.title = Some("New".into())),
            ),
            (
                "show title",
                Box::new(|f| show(f).item.title = Some("New".into())),
            ),
            (
                "show start",
                Box::new(|f| show(f).start = Some("2026-11-01T11:00:00Z".into())),
            ),
            ("show end", Box::new(|f| show(f).end = None)),
            (
                "show guid",
                Box::new(|f| show(f).item.guid = Some("x".into())),
            ),
            (
                "content link",
                Box::new(|f| {
                    show(f).content_links.push(Entry::new(ContentLink {
                        href: Some("https://example.com/b".into()),
                        ..ContentLink::default()
                    }));
                }),
            ),
            (
                "other status",
                Box::new(|f| f.live_items[1].value.status = Some("live".into())),
            ),
            (
                "removed show",
                Box::new(|f| {
                    f.live_items.remove(1);
                }),
            ),
            (
                "pending to ended",
                Box::new(|f| show(f).status = Some("ended".into())),
            ),
            (
                "unknown status",
                Box::new(|f| show(f).status = Some("soon".into())),
            ),
            (
                "new track",
                Box::new(|f| f.items.push(Entry::new(crate::feed::Item::default()))),
            ),
        ];
        for (name, change) in cases {
            let after = changed(change);
            assert!(!covered(&before, &after, "show-1"), "{name}");
        }
        let after = changed(|f| show(f).status = Some("live".into()));
        assert!(!covered(&before, &after, "show-2"), "a different grant");
        assert!(!covered(&before, &after, "missing"), "no such item");
        let live = changed(|f| show(f).status = Some("live".into()));
        assert!(!covered(&live, &before, "show-1"), "live to pending");
    }

    #[test]
    fn a_change_of_the_order_of_the_live_items_is_not_covered() {
        let before = feed();
        let after = changed(|f| f.live_items.swap(0, 1));
        assert!(!covered(&before, &after, "show-1"));
    }

    #[test]
    fn the_writes_of_a_grant_are_kept_in_sequence() {
        let mut collection = Collection::open_in_memory().unwrap();
        let id = collection.add_feed(None, None).unwrap();
        record(&collection, id, "show-1", 1).unwrap();
        let grant = grant_for(&collection, id, "show-1").unwrap().unwrap();
        add_write(&collection, grant.id, "Tool", &GrantChange::GoLive, 2).unwrap();
        add_write(&collection, grant.id, "Tool", &GrantChange::End, 3).unwrap();
        let changes: Vec<String> = writes(&collection, grant.id)
            .unwrap()
            .into_iter()
            .map(|w| w.change)
            .collect();
        assert_eq!(changes, ["go_live", "end"]);
        assert_eq!(
            open_grants(&collection).unwrap(),
            std::slice::from_ref(&grant)
        );
        assert_eq!(
            grants(&collection, id).unwrap(),
            std::slice::from_ref(&grant)
        );
        assert_eq!(super::grant(&collection, grant.id).unwrap(), Some(grant));
    }
}

//! The announcement of a written feed to the Podcast Index hub (ADR 0052).
//!
//! An announcement is one request to the hub. A write puts the planned
//! announcements into a queue in the collection. [`take_due`] gives the
//! next one that is due, and [`send`] sends it. The rules of the queue are
//! here. The collection holds the rows. An announcement has no secret.

use std::fmt;

use serde::Serialize;
use serde_json::Value;

use crate::collection::{AnnouncementAttempt, Collection, CollectionError, FeedId};
use crate::web::Web;

/// The hub of Podcast Index (ADR 0052 item 1).
pub const DEFAULT_HUB: &str = "https://hub.podcastindex.org/pubnotify";

/// The setting that turns the request to the hub on or off.
pub const PUBNOTIFY_SETTING: &str = "announce_pubnotify";

/// The setting that adds `podping=1` to the request.
pub const PODPING_SETTING: &str = "announce_podping";

/// The number of finished announcements that Studio keeps for each feed
/// (ADR 0052 item 19).
const KEEP_FINISHED: i64 = 10;

/// The largest answer of the hub that Studio reads.
const MAX_ANSWER_BYTES: u64 = 64 * 1024;

/// The two global settings of the announcements (ADR 0052 item 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AnnounceSettings {
    /// Tell Podcast Index after each write.
    pub pubnotify: bool,
    /// Send a podping with the request.
    pub podping: bool,
}

impl AnnounceSettings {
    /// Read the settings. A setting that is not set is on.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails or a setting is not a
    /// boolean.
    pub fn load(collection: &Collection) -> Result<Self, CollectionError> {
        Ok(Self {
            pubnotify: flag(collection, PUBNOTIFY_SETTING)?,
            podping: flag(collection, PODPING_SETTING)?,
        })
    }

    /// Keep the settings.
    ///
    /// # Errors
    ///
    /// Gives a [`CollectionError`] when SQLite fails.
    pub fn save(&self, collection: &Collection) -> Result<(), CollectionError> {
        collection.set_setting(PUBNOTIFY_SETTING, &Value::Bool(self.pubnotify))?;
        collection.set_setting(PODPING_SETTING, &Value::Bool(self.podping))
    }
}

fn flag(collection: &Collection, key: &str) -> Result<bool, CollectionError> {
    match collection.setting(key)? {
        None => Ok(true),
        Some(Value::Bool(value)) => Ok(value),
        Some(_) => Err(CollectionError::invalid(format!(
            "the {key} setting is not a boolean"
        ))),
    }
}

/// The times of the queue, in seconds (ADR 0052 items 14, 15 and 18).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    /// The wait between a write and the first attempt.
    pub first_wait: u64,
    /// The wait after the first, second, third and fourth failure.
    pub retry_waits: [u64; 4],
    /// The age after which a waiting announcement is failed at the start.
    pub max_age: u64,
}

impl Timing {
    /// The times that ADR 0052 gives.
    pub const DEFAULT: Self = Self {
        first_wait: 30,
        retry_waits: [60, 120, 240, 480],
        max_age: 86_400,
    };
}

/// An announcement that a write plans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlannedAnnouncement {
    /// The public address of the feed.
    pub url: String,
    /// True when the request has `podping=1`.
    pub podping: bool,
}

/// The reason why a write plans no announcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnounceSkip {
    /// The setting `announce_pubnotify` is off.
    Off,
    /// The write has no public address to announce.
    NoAddress,
}

impl AnnounceSkip {
    /// The text of the reason, for the diff ID.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::NoAddress => "no_address",
        }
    }
}

/// The announcements that a write plans, or the reason for none
/// (ADR 0052 items 7 to 9). It gives one announcement for each address, in
/// the sequence of `addresses`. A second copy of an address goes out too.
#[must_use]
pub fn plan(
    settings: &AnnounceSettings,
    addresses: &[String],
) -> (Vec<PlannedAnnouncement>, Option<AnnounceSkip>) {
    if !settings.pubnotify {
        return (Vec::new(), Some(AnnounceSkip::Off));
    }
    if addresses.is_empty() {
        return (Vec::new(), Some(AnnounceSkip::NoAddress));
    }
    let planned = addresses
        .iter()
        .map(|url| PlannedAnnouncement {
            url: url.clone(),
            podping: settings.podping,
        })
        .collect();
    (planned, None)
}

/// The state of an announcement in the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnouncementState {
    /// The announcement waits for its time.
    Waiting,
    /// Studio sends the announcement now.
    Sending,
    /// The hub accepted the announcement.
    Sent,
    /// The announcement failed, and Studio does not try again.
    Failed,
}

impl AnnouncementState {
    /// The name that the collection keeps.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Sending => "sending",
            Self::Sent => "sent",
            Self::Failed => "failed",
        }
    }

    /// The state for a name that the collection keeps.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "waiting" => Some(Self::Waiting),
            "sending" => Some(Self::Sending),
            "sent" => Some(Self::Sent),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// A row of the queue. All times are seconds after the Unix epoch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Announcement {
    /// The ID of the row.
    pub id: i64,
    /// The feed that the write changed.
    pub feed_id: FeedId,
    /// The address to announce.
    pub url: String,
    /// True when the request has `podping=1`.
    pub podping: bool,
    /// The state of the row.
    pub state: AnnouncementState,
    /// The number of attempts so far.
    pub attempts: u32,
    /// The time of the write, or of the last replacement.
    pub created_at: i64,
    /// The time of the next attempt.
    pub next_at: i64,
    /// The time when the row became `sent` or `failed`.
    pub finished_at: Option<i64>,
    /// The error text of the last failed attempt.
    pub last_error: Option<String>,
}

/// The text of an announcement that Studio did not send before it stopped.
const STOPPED_TEXT: &str = "Studio stopped before it sent the announcement";

fn time(seconds: u64) -> i64 {
    i64::try_from(seconds).unwrap_or(i64::MAX)
}

/// Put the planned announcements of a feed into the queue.
///
/// A `waiting` row for the same address takes the new values and its wait
/// starts again. A `sending` row does not change (ADR 0052 item 16).
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn queue(
    collection: &Collection,
    feed_id: FeedId,
    planned: &[PlannedAnnouncement],
    timing: &Timing,
    now: u64,
) -> Result<(), CollectionError> {
    let created_at = time(now);
    let next_at = time(now.saturating_add(timing.first_wait));
    for item in planned {
        match collection.waiting_announcement(feed_id, &item.url)? {
            Some(id) => collection.restart_announcement(id, item.podping, created_at, next_at)?,
            None => collection.insert_announcement(
                feed_id,
                &item.url,
                item.podping,
                created_at,
                next_at,
            )?,
        }
    }
    Ok(())
}

/// Give the oldest `waiting` announcement that is due, and set it to
/// `sending`.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn take_due(
    collection: &Collection,
    now: u64,
) -> Result<Option<Announcement>, CollectionError> {
    let Some(mut announcement) = collection.oldest_due_announcement(time(now))? else {
        return Ok(None);
    };
    collection.set_announcement_state(announcement.id, AnnouncementState::Sending)?;
    announcement.state = AnnouncementState::Sending;
    Ok(Some(announcement))
}

/// The time of the next `waiting` announcement.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn next_due_at(collection: &Collection) -> Result<Option<u64>, CollectionError> {
    Ok(collection
        .earliest_waiting_announcement()?
        .map(|at| u64::try_from(at).unwrap_or(0)))
}

/// Record that the hub accepted an announcement.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn record_sent(collection: &Collection, id: i64, now: u64) -> Result<(), CollectionError> {
    let row = collection.announcement(id)?;
    collection.record_announcement_attempt(
        id,
        &AnnouncementAttempt {
            state: AnnouncementState::Sent,
            attempts: row.attempts + 1,
            next_at: row.next_at,
            finished_at: Some(time(now)),
            error: None,
        },
    )?;
    collection.trim_announcements(KEEP_FINISHED)
}

/// Record a failed attempt. After the fifth attempt, the announcement is
/// `failed`. If not, it waits for the next retry (ADR 0052 item 15).
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn record_failure(
    collection: &Collection,
    id: i64,
    error: &str,
    timing: &Timing,
    now: u64,
) -> Result<(), CollectionError> {
    let row = collection.announcement(id)?;
    let attempts = row.attempts + 1;
    let retry = usize::try_from(attempts - 1)
        .ok()
        .and_then(|index| timing.retry_waits.get(index));
    let attempt = match retry {
        Some(wait) => AnnouncementAttempt {
            state: AnnouncementState::Waiting,
            attempts,
            next_at: time(now.saturating_add(*wait)),
            finished_at: None,
            error: Some(error),
        },
        None => AnnouncementAttempt {
            state: AnnouncementState::Failed,
            attempts,
            next_at: row.next_at,
            finished_at: Some(time(now)),
            error: Some(error),
        },
    };
    collection.record_announcement_attempt(id, &attempt)?;
    collection.trim_announcements(KEEP_FINISHED)
}

/// Prepare the queue at the start of Studio (ADR 0052 item 18). A `sending`
/// row was interrupted, so it becomes `waiting`. A `waiting` row that is
/// older than the maximum age becomes `failed`.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn expire(collection: &Collection, timing: &Timing, now: u64) -> Result<(), CollectionError> {
    collection.release_sending_announcements()?;
    let oldest = time(now.saturating_sub(timing.max_age));
    collection.fail_old_announcements(oldest, time(now), STOPPED_TEXT)?;
    collection.trim_announcements(KEEP_FINISHED)
}

/// The announcements of a feed, the newest first.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
pub fn recent(
    collection: &Collection,
    feed_id: FeedId,
) -> Result<Vec<Announcement>, CollectionError> {
    collection.announcements(feed_id)
}

/// Set `next_at` to `now` on each `waiting` row, so that a test does not
/// wait for the first wait.
///
/// # Errors
///
/// Gives a [`CollectionError`] when SQLite fails.
#[cfg(any(test, feature = "testing"))]
pub fn make_due(collection: &Collection, now: u64) -> Result<(), CollectionError> {
    collection.make_announcements_due(time(now))
}

/// An error from [`send`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnounceError {
    message: String,
}

impl fmt::Display for AnnounceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AnnounceError {}

/// Send one announcement to a hub (ADR 0052 items 1 to 3).
///
/// The query has `url`, and `podping=1` when the announcement asks for it.
///
/// # Errors
///
/// Gives an [`AnnounceError`] for a network failure, a status that is not
/// 200, or a refusal of the hub.
pub fn send(web: &Web, hub: &str, announcement: &Announcement) -> Result<(), AnnounceError> {
    let mut query = vec![("url", announcement.url.as_str())];
    if announcement.podping {
        query.push(("podping", "1"));
    }
    let file = web
        .get_with_query(hub, &query, MAX_ANSWER_BYTES)
        .map_err(|e| AnnounceError {
            message: e.to_string(),
        })?;
    // A body that is not JSON is a success.
    let Ok(body) = serde_json::from_slice::<Value>(&file.bytes) else {
        return Ok(());
    };
    let refused = match body.get("status") {
        Some(Value::Bool(false)) => true,
        Some(Value::String(text)) => text == "false",
        _ => false,
    };
    if refused {
        let description = body
            .get("description")
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
            .unwrap_or("the hub refused the announcement");
        return Err(AnnounceError {
            message: description.to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{HubRequest, TestHub};

    const NOW: u64 = 1_000_000;

    fn planned(url: &str, podping: bool) -> PlannedAnnouncement {
        PlannedAnnouncement {
            url: url.to_owned(),
            podping,
        }
    }

    fn setup() -> (Collection, FeedId) {
        let mut collection = Collection::open_in_memory().unwrap();
        let feed = collection.add_feed(None, None).unwrap();
        (collection, feed)
    }

    fn queue_one(collection: &Collection, feed: FeedId, url: &str, now: u64) {
        queue(
            collection,
            feed,
            &[planned(url, true)],
            &Timing::DEFAULT,
            now,
        )
        .unwrap();
    }

    fn settings(pubnotify: bool, podping: bool) -> AnnounceSettings {
        AnnounceSettings { pubnotify, podping }
    }

    #[test]
    fn plan_gives_one_announcement_for_each_address_in_sequence() {
        let addresses = vec![
            "https://m.example/b.xml".to_owned(),
            "https://m.example/a.xml".to_owned(),
            "https://m.example/b.xml".to_owned(),
        ];
        let (got, skip) = plan(&settings(true, true), &addresses);
        assert_eq!(skip, None);
        assert_eq!(
            got,
            [
                planned("https://m.example/b.xml", true),
                planned("https://m.example/a.xml", true),
                planned("https://m.example/b.xml", true),
            ]
        );
    }

    #[test]
    fn plan_copies_the_podping_setting() {
        let addresses = vec!["https://m.example/a.xml".to_owned()];
        let (planned, _) = plan(&settings(true, false), &addresses);
        assert!(!planned[0].podping);
    }

    #[test]
    fn plan_gives_off_when_pubnotify_is_off_even_with_no_address() {
        let addresses = vec!["https://m.example/a.xml".to_owned()];
        assert_eq!(
            plan(&settings(false, true), &addresses),
            (Vec::new(), Some(AnnounceSkip::Off))
        );
        assert_eq!(
            plan(&settings(false, true), &[]),
            (Vec::new(), Some(AnnounceSkip::Off))
        );
    }

    #[test]
    fn plan_gives_no_address_when_there_is_no_address() {
        assert_eq!(
            plan(&settings(true, true), &[]),
            (Vec::new(), Some(AnnounceSkip::NoAddress))
        );
    }

    #[test]
    fn the_settings_are_on_when_they_are_not_set() {
        let (collection, _) = setup();
        let settings = AnnounceSettings::load(&collection).unwrap();
        assert!(settings.pubnotify);
        assert!(settings.podping);
    }

    #[test]
    fn a_saved_setting_is_read_again() {
        let (collection, _) = setup();
        AnnounceSettings {
            pubnotify: true,
            podping: false,
        }
        .save(&collection)
        .unwrap();
        let settings = AnnounceSettings::load(&collection).unwrap();
        assert!(settings.pubnotify);
        assert!(!settings.podping);
        AnnounceSettings {
            pubnotify: false,
            podping: true,
        }
        .save(&collection)
        .unwrap();
        let settings = AnnounceSettings::load(&collection).unwrap();
        assert!(!settings.pubnotify);
        assert!(settings.podping);
    }

    #[test]
    fn a_setting_that_is_not_a_boolean_gives_an_error_that_names_it() {
        let (collection, _) = setup();
        collection
            .set_setting(PODPING_SETTING, &serde_json::json!("yes"))
            .unwrap();
        let error = AnnounceSettings::load(&collection).unwrap_err();
        assert!(error.to_string().contains(PODPING_SETTING));
    }

    #[test]
    fn queue_makes_a_waiting_row_that_is_due_after_the_first_wait() {
        let (collection, feed) = setup();
        queue_one(&collection, feed, "https://m.example/a.xml", NOW);
        let rows = recent(&collection, feed).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].state, AnnouncementState::Waiting);
        assert_eq!(rows[0].attempts, 0);
        assert_eq!(rows[0].created_at, 1_000_000);
        assert_eq!(rows[0].next_at, 1_000_030);
        assert_eq!(next_due_at(&collection).unwrap(), Some(1_000_030));
        assert!(take_due(&collection, NOW + 29).unwrap().is_none());
    }

    #[test]
    fn two_queue_calls_for_one_address_within_the_wait_give_one_row() {
        let (collection, feed) = setup();
        queue_one(&collection, feed, "https://m.example/a.xml", NOW);
        queue(
            &collection,
            feed,
            &[planned("https://m.example/a.xml", false)],
            &Timing::DEFAULT,
            NOW + 10,
        )
        .unwrap();
        let rows = recent(&collection, feed).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].created_at, 1_000_010);
        assert_eq!(rows[0].next_at, 1_000_040);
        assert!(!rows[0].podping);
    }

    #[test]
    fn a_replaced_row_loses_the_error_of_an_earlier_attempt() {
        let (collection, feed) = setup();
        queue_one(&collection, feed, "https://m.example/a.xml", NOW);
        let taken = take_due(&collection, NOW + 30).unwrap().unwrap();
        record_failure(
            &collection,
            taken.id,
            "the hub failed",
            &Timing::DEFAULT,
            NOW + 31,
        )
        .unwrap();
        queue_one(&collection, feed, "https://m.example/a.xml", NOW + 40);
        let rows = recent(&collection, feed).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].attempts, 0);
        assert_eq!(rows[0].last_error, None);
    }

    #[test]
    fn a_queue_call_while_the_row_is_sending_gives_a_second_row() {
        let (collection, feed) = setup();
        queue_one(&collection, feed, "https://m.example/a.xml", NOW);
        let taken = take_due(&collection, NOW + 30).unwrap().unwrap();
        assert_eq!(taken.state, AnnouncementState::Sending);
        queue_one(&collection, feed, "https://m.example/a.xml", NOW + 31);
        let rows = recent(&collection, feed).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].state, AnnouncementState::Waiting);
        assert_eq!(rows[1].state, AnnouncementState::Sending);
        assert_eq!(rows[1].id, taken.id);
    }

    #[test]
    fn take_due_gives_the_oldest_due_row_first_and_each_row_one_time() {
        let (collection, feed) = setup();
        queue_one(&collection, feed, "https://m.example/b.xml", NOW + 5);
        queue_one(&collection, feed, "https://m.example/a.xml", NOW);
        let first = take_due(&collection, NOW + 100).unwrap().unwrap();
        let second = take_due(&collection, NOW + 100).unwrap().unwrap();
        assert_eq!(first.url, "https://m.example/a.xml");
        assert_eq!(second.url, "https://m.example/b.xml");
        assert!(take_due(&collection, NOW + 100).unwrap().is_none());
        assert_eq!(next_due_at(&collection).unwrap(), None);
    }

    #[test]
    fn make_due_sets_the_time_of_each_waiting_row_to_now() {
        let (collection, feed) = setup();
        queue_one(&collection, feed, "https://m.example/a.xml", NOW);
        make_due(&collection, NOW).unwrap();
        assert!(take_due(&collection, NOW).unwrap().is_some());
    }

    #[test]
    fn record_sent_finishes_the_row() {
        let (collection, feed) = setup();
        queue_one(&collection, feed, "https://m.example/a.xml", NOW);
        let taken = take_due(&collection, NOW + 30).unwrap().unwrap();
        record_sent(&collection, taken.id, NOW + 31).unwrap();
        let row = &recent(&collection, feed).unwrap()[0];
        assert_eq!(row.state, AnnouncementState::Sent);
        assert_eq!(row.attempts, 1);
        assert_eq!(row.finished_at, Some(1_000_031));
        assert_eq!(row.last_error, None);
    }

    #[test]
    fn five_failures_give_the_retry_times_and_then_the_state_failed() {
        let (collection, feed) = setup();
        queue_one(&collection, feed, "https://m.example/a.xml", NOW);
        let mut now = NOW + 30;
        for wait in [60, 120, 240, 480] {
            let taken = take_due(&collection, now).unwrap().unwrap();
            record_failure(&collection, taken.id, "down", &Timing::DEFAULT, now).unwrap();
            let row = &recent(&collection, feed).unwrap()[0];
            assert_eq!(row.state, AnnouncementState::Waiting);
            assert_eq!(row.next_at, i64::try_from(now + wait).unwrap());
            assert_eq!(row.last_error.as_deref(), Some("down"));
            assert!(take_due(&collection, now + wait - 1).unwrap().is_none());
            now += wait;
        }
        let taken = take_due(&collection, now).unwrap().unwrap();
        record_failure(&collection, taken.id, "down", &Timing::DEFAULT, now).unwrap();
        let row = &recent(&collection, feed).unwrap()[0];
        assert_eq!(row.state, AnnouncementState::Failed);
        assert_eq!(row.attempts, 5);
        assert_eq!(row.finished_at, Some(i64::try_from(now).unwrap()));
        assert!(take_due(&collection, now + 10_000).unwrap().is_none());
    }

    #[test]
    fn expire_fails_a_waiting_row_of_24_hours_and_1_second_and_keeps_a_row_of_23_hours() {
        let (collection, feed) = setup();
        queue_one(&collection, feed, "https://m.example/old.xml", NOW);
        queue_one(
            &collection,
            feed,
            "https://m.example/young.xml",
            NOW + 3_600,
        );
        let now = NOW + 86_401;
        expire(&collection, &Timing::DEFAULT, now).unwrap();
        let rows = recent(&collection, feed).unwrap();
        let find = |name: &str| rows.iter().find(|row| row.url.ends_with(name)).unwrap();
        let old = find("old.xml");
        assert_eq!(old.state, AnnouncementState::Failed);
        assert_eq!(old.last_error.as_deref(), Some(STOPPED_TEXT));
        assert_eq!(old.finished_at, Some(i64::try_from(now).unwrap()));
        assert_eq!(find("young.xml").state, AnnouncementState::Waiting);
    }

    #[test]
    fn expire_uses_the_age_of_24_hours_exactly() {
        let (collection, feed) = setup();
        queue_one(&collection, feed, "https://m.example/a.xml", NOW);
        expire(&collection, &Timing::DEFAULT, NOW + 86_400).unwrap();
        assert_eq!(
            recent(&collection, feed).unwrap()[0].state,
            AnnouncementState::Waiting
        );
        expire(&collection, &Timing::DEFAULT, NOW + 86_401).unwrap();
        assert_eq!(
            recent(&collection, feed).unwrap()[0].state,
            AnnouncementState::Failed
        );
    }

    #[test]
    fn expire_sets_a_sending_row_to_waiting_and_keeps_a_row_of_23_hours() {
        let (collection, feed) = setup();
        queue_one(&collection, feed, "https://m.example/a.xml", NOW);
        take_due(&collection, NOW + 30).unwrap().unwrap();
        expire(&collection, &Timing::DEFAULT, NOW + 23 * 3_600).unwrap();
        assert_eq!(
            recent(&collection, feed).unwrap()[0].state,
            AnnouncementState::Waiting
        );
    }

    #[test]
    fn a_feed_keeps_its_newest_10_finished_rows_and_its_waiting_row() {
        let (collection, feed) = setup();
        for index in 0..12_u64 {
            queue_one(
                &collection,
                feed,
                &format!("https://m.example/{index}.xml"),
                NOW + index,
            );
            let taken = take_due(&collection, NOW + 1_000).unwrap().unwrap();
            record_sent(&collection, taken.id, NOW + 1_000).unwrap();
        }
        queue_one(&collection, feed, "https://m.example/waiting.xml", NOW + 50);
        let rows = recent(&collection, feed).unwrap();
        let finished: Vec<&str> = rows
            .iter()
            .filter(|row| row.state == AnnouncementState::Sent)
            .map(|row| row.url.as_str())
            .collect();
        assert_eq!(finished.len(), 10);
        assert_eq!(finished[0], "https://m.example/11.xml");
        assert_eq!(finished[9], "https://m.example/2.xml");
        assert!(rows.iter().any(|r| r.state == AnnouncementState::Waiting));
    }

    #[test]
    fn the_trim_of_one_feed_leaves_the_rows_of_another_feed() {
        let (mut collection, feed) = setup();
        let other = collection.add_feed(None, None).unwrap();
        for index in 0..11_u64 {
            queue_one(&collection, feed, "https://m.example/a.xml", NOW + index);
            let taken = take_due(&collection, NOW + 1_000).unwrap().unwrap();
            record_sent(&collection, taken.id, NOW + 1_000).unwrap();
        }
        queue_one(&collection, other, "https://m.example/b.xml", NOW);
        let taken = take_due(&collection, NOW + 1_000).unwrap().unwrap();
        record_sent(&collection, taken.id, NOW + 1_000).unwrap();
        assert_eq!(recent(&collection, feed).unwrap().len(), 10);
        assert_eq!(recent(&collection, other).unwrap().len(), 1);
    }

    #[test]
    fn the_removal_of_a_feed_removes_its_announcements() {
        let (mut collection, feed) = setup();
        queue_one(&collection, feed, "https://m.example/a.xml", NOW);
        collection.delete_feed(feed).unwrap();
        assert_eq!(next_due_at(&collection).unwrap(), None);
    }

    fn row(url: &str, podping: bool) -> Announcement {
        Announcement {
            id: 1,
            feed_id: FeedId(1),
            url: url.to_owned(),
            podping,
            state: AnnouncementState::Sending,
            attempts: 0,
            created_at: 0,
            next_at: 0,
            finished_at: None,
            last_error: None,
        }
    }

    #[test]
    fn send_gives_the_address_and_podping_1_to_the_hub() {
        let hub = TestHub::start();
        send(
            &Web::new(),
            &hub.url(),
            &row("https://m.example/a.xml?x=1&y=2", true),
        )
        .unwrap();
        assert_eq!(
            hub.requests(),
            [HubRequest {
                url: Some("https://m.example/a.xml?x=1&y=2".to_owned()),
                podping: Some("1".to_owned()),
            }]
        );
    }

    #[test]
    fn send_gives_no_podping_parameter_when_podping_is_false() {
        let hub = TestHub::start();
        send(
            &Web::new(),
            &hub.url(),
            &row("https://m.example/a.xml", false),
        )
        .unwrap();
        assert_eq!(
            hub.requests(),
            [HubRequest {
                url: Some("https://m.example/a.xml".to_owned()),
                podping: None,
            }]
        );
    }

    #[test]
    fn send_gives_an_error_with_the_status_for_a_500() {
        let hub = TestHub::start();
        hub.fail_next(1);
        let error = send(
            &Web::new(),
            &hub.url(),
            &row("https://m.example/a.xml", true),
        )
        .unwrap_err();
        assert!(error.to_string().contains("500"));
    }

    #[test]
    fn send_gives_the_description_of_a_refusal() {
        let hub = TestHub::start();
        hub.refuse_next(1);
        let error = send(
            &Web::new(),
            &hub.url(),
            &row("https://m.example/a.xml", true),
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "refused by the test hub");
    }

    #[test]
    fn send_gives_an_error_when_the_hub_does_not_listen() {
        let port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        };
        let hub = format!("http://127.0.0.1:{port}/pubnotify");
        let error = send(&Web::new(), &hub, &row("https://m.example/a.xml", true)).unwrap_err();
        assert!(error.to_string().contains(&hub));
    }
}

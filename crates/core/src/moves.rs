//! The old copies of a move and the state of a move that waits (ADR 0148).
//!
//! A write that moves a file deletes no old copy. The collection records
//! each old copy, and [`waiting_moves`] groups the records of the feed. The
//! function decides when a move is ready. The core does not delete a file:
//! the user reviews a deletion and writes it through `pipeline::write_draft`.

use serde::Serialize;

use crate::collection::{ConnectorId, OldCopy};
use crate::media::plain_address;

/// The waiting time of a move with no select: **Move to** on one file
/// (ADR 0148 item 7).
pub const DEFAULT_WAIT_DAYS: u32 = 30;

/// The waiting times that a user can select (ADR 0148 item 6).
pub const WAIT_CHOICES: [u32; 3] = [7, DEFAULT_WAIT_DAYS, 90];

const SECONDS_PER_DAY: i64 = 86_400;

/// The answer of musicindex about the feed address (ADR 0148 item 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexAnswer {
    /// musicindex gives the public address of the feed.
    Follows,
    /// musicindex gives a different address.
    Different,
    /// musicindex gave no answer, or it has no entry for this feed.
    Unknown,
}

impl IndexAnswer {
    /// Compare the address that musicindex gives with the public address
    /// of the feed, with no OP3 prefix (ADR 0076).
    #[must_use]
    pub fn compare(index_url: Option<&str>, public_url: Option<&str>) -> Self {
        match (index_url, public_url) {
            (Some(index), Some(public)) if plain_address(index) == plain_address(public) => {
                Self::Follows
            }
            (Some(_), Some(_)) => Self::Different,
            _ => Self::Unknown,
        }
    }
}

/// The old copies of one feed that one write moved from one file host to
/// one other file host, with one waiting time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WaitingMove {
    /// The record IDs of the old copies.
    pub records: Vec<i64>,
    /// The file host that keeps the old copies.
    pub connector_id: ConnectorId,
    /// The file host that has the new copies. `None` after the removal of
    /// that file host.
    pub to_connector_id: Option<ConnectorId>,
    /// The time of the write, in seconds after the Unix epoch.
    pub moved_at: i64,
    /// The waiting time that the user selected.
    pub wait_days: u32,
    /// The complete days since the write.
    pub days_passed: u32,
    /// The answer of musicindex.
    pub index: IndexAnswer,
    /// True when Studio can offer the deletion.
    pub ready: bool,
}

/// The complete days between `moved_at` and `now`. A clock that is behind
/// the write gives zero.
#[must_use]
pub fn days_since(moved_at: i64, now: i64) -> u32 {
    u32::try_from(now.saturating_sub(moved_at).max(0) / SECONDS_PER_DAY).unwrap_or(u32::MAX)
}

/// True when the days of a move are complete and musicindex does not give a
/// different address (ADR 0148 item 9).
#[must_use]
pub fn is_ready(days_passed: u32, wait_days: u32, index: IndexAnswer) -> bool {
    days_passed >= wait_days && index != IndexAnswer::Different
}

/// Group the old copies of one feed into its moves, the oldest first.
#[must_use]
pub fn waiting_moves(copies: &[OldCopy], now: i64, index: IndexAnswer) -> Vec<WaitingMove> {
    let mut moves: Vec<WaitingMove> = Vec::new();
    for copy in copies {
        let found = moves.iter_mut().find(|m| {
            m.connector_id == copy.connector_id
                && m.to_connector_id == copy.to_connector_id
                && m.moved_at == copy.moved_at
                && m.wait_days == copy.wait_days
        });
        if let Some(found) = found {
            found.records.push(copy.id);
            continue;
        }
        let days_passed = days_since(copy.moved_at, now);
        moves.push(WaitingMove {
            records: vec![copy.id],
            connector_id: copy.connector_id,
            to_connector_id: copy.to_connector_id,
            moved_at: copy.moved_at,
            wait_days: copy.wait_days,
            days_passed,
            index,
            ready: is_ready(days_passed, copy.wait_days, index),
        });
    }
    moves.sort_by_key(|m| m.moved_at);
    moves
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::FeedId;

    fn copy(id: i64, connector: i64, moved_at: i64, wait_days: u32) -> OldCopy {
        OldCopy {
            id,
            feed_id: FeedId(1),
            connector_id: ConnectorId(connector),
            path: format!("album/{id}.mp3"),
            to_connector_id: Some(ConnectorId(9)),
            moved_at,
            wait_days,
        }
    }

    #[test]
    fn a_move_is_not_ready_before_its_days_are_complete() {
        assert!(!is_ready(6, 7, IndexAnswer::Follows));
        assert!(is_ready(7, 7, IndexAnswer::Follows));
    }

    #[test]
    fn a_different_address_of_musicindex_stops_a_ready_move() {
        assert!(!is_ready(90, 7, IndexAnswer::Different));
    }

    #[test]
    fn no_answer_of_musicindex_leaves_only_the_days() {
        assert!(is_ready(7, 7, IndexAnswer::Unknown));
        assert!(!is_ready(1, 7, IndexAnswer::Unknown));
    }

    #[test]
    fn the_comparison_of_musicindex_ignores_an_op3_prefix() {
        let plain = "https://new.example/a.xml";
        let op3 = "https://op3.dev/e,pg=x/new.example/a.xml";
        assert_eq!(
            IndexAnswer::compare(Some(op3), Some(plain)),
            IndexAnswer::Follows
        );
        assert_eq!(
            IndexAnswer::compare(Some("https://old.example/a.xml"), Some(plain)),
            IndexAnswer::Different
        );
        assert_eq!(
            IndexAnswer::compare(None, Some(plain)),
            IndexAnswer::Unknown
        );
    }

    #[test]
    fn the_records_of_one_write_are_one_move() {
        let now = 40 * SECONDS_PER_DAY;
        let moves = waiting_moves(
            &[copy(1, 2, 0, 30), copy(2, 2, 0, 30), copy(3, 3, 0, 30)],
            now,
            IndexAnswer::Unknown,
        );
        assert_eq!(moves.len(), 2);
        assert_eq!(moves[0].records, vec![1, 2]);
        assert_eq!(moves[1].records, vec![3]);
        assert_eq!(moves[0].days_passed, 40);
        assert!(moves[0].ready);
    }

    #[test]
    fn a_clock_behind_the_write_gives_zero_days() {
        assert_eq!(days_since(100, 0), 0);
    }
}

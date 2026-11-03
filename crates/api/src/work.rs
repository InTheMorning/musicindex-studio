//! The count of the work that a shell must not stop without a warning
//! (ADR 0124 items 16 and 17).
//!
//! Work is a write that runs and an upload in parts. The count uses an
//! `Arc` and its own lock, so a reader never waits for the workspace.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// An upload with no new part for this time is not work (ADR 0124 item 17).
pub const UPLOAD_IDLE_SECONDS: u64 = 120;

#[derive(Debug, Default)]
struct Inner {
    writes: AtomicUsize,
    /// The ID of each upload in parts, with the time of its last part in
    /// seconds since the Unix epoch.
    uploads: Mutex<HashMap<String, u64>>,
}

/// A cloneable count of the writes and the uploads that run.
#[derive(Debug, Clone, Default)]
pub struct Work {
    inner: Arc<Inner>,
}

/// A write that runs. The count decreases when the guard drops, also after
/// an error and after a panic.
#[derive(Debug)]
#[must_use = "the write counts only while the guard lives"]
pub struct WriteGuard {
    inner: Arc<Inner>,
}

impl Drop for WriteGuard {
    fn drop(&mut self) {
        self.inner.writes.fetch_sub(1, Ordering::SeqCst);
    }
}

fn wall_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

impl Work {
    /// A new count with no work.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Count a write until the guard drops.
    pub fn write(&self) -> WriteGuard {
        self.inner.writes.fetch_add(1, Ordering::SeqCst);
        WriteGuard {
            inner: Arc::clone(&self.inner),
        }
    }

    /// The number of writes that run.
    #[must_use]
    pub fn writes(&self) -> usize {
        self.inner.writes.load(Ordering::SeqCst)
    }

    /// The number of uploads in parts that got a part in the last 2 minutes.
    #[must_use]
    pub fn uploads(&self) -> usize {
        self.uploads_at(wall_seconds())
    }

    /// The number of uploads in parts at the time `now`, in seconds since
    /// the Unix epoch. An upload with no new part for 2 minutes is not in
    /// the count, and the call removes it from the list.
    #[must_use]
    pub fn uploads_at(&self, now: u64) -> usize {
        let Ok(mut uploads) = self.inner.uploads.lock() else {
            return 0;
        };
        uploads.retain(|_, last| now.saturating_sub(*last) < UPLOAD_IDLE_SECONDS);
        uploads.len()
    }

    /// Record a part of the upload `id` at the time of the computer.
    pub(crate) fn touch_upload(&self, id: &str) {
        self.touch_upload_at(id, wall_seconds());
    }

    /// Record a part of the upload `id` at the time `now`.
    pub(crate) fn touch_upload_at(&self, id: &str, now: u64) {
        if let Ok(mut uploads) = self.inner.uploads.lock() {
            uploads.insert(id.to_owned(), now);
        }
    }

    /// Remove the upload `id`: it finished or the user stopped it.
    pub(crate) fn end_upload(&self, id: &str) {
        if let Ok(mut uploads) = self.inner.uploads.lock() {
            uploads.remove(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_write_guard_counts_until_it_drops() {
        let work = Work::new();
        assert_eq!(work.writes(), 0);
        let first = work.write();
        let second = work.clone().write();
        assert_eq!(work.writes(), 2, "ADR 0124 item 17: each write must count.");
        drop(first);
        assert_eq!(work.writes(), 1);
        drop(second);
        assert_eq!(
            work.writes(),
            0,
            "ADR 0124 item 17: a drop must end the count."
        );
    }

    #[test]
    fn a_write_that_fails_ends_its_count() {
        let work = Work::new();
        let run = |work: &Work| -> Result<(), &'static str> {
            let _guard = work.write();
            Err("the write failed")
        };
        assert!(run(&work).is_err());
        assert_eq!(
            work.writes(),
            0,
            "ADR 0124 item 17: an error must end the count."
        );
    }

    #[test]
    fn a_write_that_panics_ends_its_count() {
        let work = Work::new();
        let inner = work.clone();
        let result = std::thread::spawn(move || {
            let _guard = inner.write();
            panic!("the write panics");
        })
        .join();
        assert!(result.is_err());
        assert_eq!(
            work.writes(),
            0,
            "ADR 0124 item 17: a panic must end the count."
        );
    }

    #[test]
    fn an_upload_counts_from_its_first_part_to_its_finish() {
        let work = Work::new();
        assert_eq!(work.uploads_at(1000), 0);
        work.touch_upload_at("a", 1000);
        assert_eq!(work.uploads_at(1000), 1);
        work.touch_upload_at("a", 1050);
        work.touch_upload_at("b", 1050);
        assert_eq!(work.uploads_at(1060), 2);
        work.end_upload("a");
        assert_eq!(
            work.uploads_at(1060),
            1,
            "ADR 0124 item 17: a finished or deleted upload must leave the count."
        );
        work.end_upload("b");
        assert_eq!(work.uploads_at(1060), 0);
    }

    #[test]
    fn an_upload_with_no_part_for_two_minutes_is_not_work() {
        let work = Work::new();
        work.touch_upload_at("a", 1000);
        assert_eq!(work.uploads_at(1000 + UPLOAD_IDLE_SECONDS - 1), 1);
        assert_eq!(
            work.uploads_at(1000 + UPLOAD_IDLE_SECONDS),
            0,
            "ADR 0124 item 17: an upload with no part for 2 minutes must not count."
        );
        work.touch_upload_at("a", 1000 + UPLOAD_IDLE_SECONDS + 5);
        assert_eq!(work.uploads_at(1000 + UPLOAD_IDLE_SECONDS + 6), 1);
    }
}

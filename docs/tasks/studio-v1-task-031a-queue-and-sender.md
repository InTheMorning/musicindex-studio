# Studio v1 Task 031a: The Test Hub, The Queue And The Sender

Parent: [task 031](studio-v1-task-031-announce.md). Decisions: [ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md) items 1 to 3, 5, 6 and 13 to 19, and [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md).
Sequence: the first packet of task 031.

## Goal

The core has a local test hub, the two settings and the announcement queue with its retry rules. It also has a function that sends one announcement to a hub. Only the tests call them in this packet.

## Files

Inspect: `crates/core/src/lib.rs`, `crates/core/src/collection.rs` (`MIGRATIONS`, `setting`, `set_setting`, the migration tests), `crates/core/src/web.rs` (`Web::get_with_query`), `crates/core/src/testing.rs`, `crates/core/src/testing/fake_s3.rs` (the form of a test server), `crates/core/src/download.rs` (a setting with a default).

Change: `crates/core/src/lib.rs`, `crates/core/src/collection.rs`, `crates/core/src/testing.rs`, and the new files `crates/core/src/announce.rs` and `crates/core/src/testing/hub.rs`.

Do not change: `pipeline.rs`, the API crate, the desktop crate, the UI.

## Architecture

These choices are fixed. Do not change them.

### The Test Hub

- `testing::TestHub` is a small HTTP server in the test process, in the form of `fake_s3.rs`. It needs no new crate.
- `TestHub::start()` listens on `127.0.0.1` with a free port. `url()` gives `http://127.0.0.1:<port>/pubnotify`.
- Each `GET /pubnotify` adds a `HubRequest { url: Option<String>, podping: Option<String> }` with the decoded query values. `requests()` gives them in sequence.
- By default, it answers 200 with `{"status":"true","description":"Feed marked for immediate update."}`.
- `fail_next(n)` makes the next `n` requests answer 500. `refuse_next(n)` makes the next `n` requests answer 200 with `{"status":"false","description":"refused by the test hub"}`.
- The server stops when the value is dropped, as the other test servers do.

### The Settings

In `announce.rs`:

```rust
pub const DEFAULT_HUB: &str = "https://hub.podcastindex.org/pubnotify";

pub struct AnnounceSettings { pub pubnotify: bool, pub podping: bool }
impl AnnounceSettings {
    pub fn load(collection: &Collection) -> Result<Self, CollectionError>;
    pub fn save(&self, collection: &Collection) -> Result<(), CollectionError>;
}
```

- The keys are `announce_pubnotify` and `announce_podping`, JSON booleans. A missing key is `true`.

### The Queue Table

- A new migration at the end of `MIGRATIONS` makes this table:

  ```sql
  CREATE TABLE announcements (
      id INTEGER PRIMARY KEY,
      feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
      url TEXT NOT NULL,
      podping INTEGER NOT NULL,
      state TEXT NOT NULL,
      attempts INTEGER NOT NULL DEFAULT 0,
      created_at INTEGER NOT NULL,
      next_at INTEGER NOT NULL,
      finished_at INTEGER,
      last_error TEXT
  );
  ```

- `state` is `waiting`, `sending`, `sent` or `failed`. All times are seconds after the Unix epoch.
- If the removal of a feed fails because of this table, delete the rows of the feed in the function that removes the feed. Do not change how other tables are removed.
- `collection.rs` gets the storage functions only. Each one does one SQL step. `announce.rs` holds the rules and the constants.

### The Rules

In `announce.rs`:

```rust
pub struct Timing { pub first_wait: u64, pub retry_waits: [u64; 4], pub max_age: u64 }
impl Timing { pub const DEFAULT: Self = Self { first_wait: 30, retry_waits: [60, 120, 240, 480], max_age: 86_400 }; }

pub struct PlannedAnnouncement { pub url: String, pub podping: bool }   // serde::Serialize
pub struct Announcement { pub id: i64, pub feed_id: FeedId, pub url: String, pub podping: bool,
    pub state: AnnouncementState, pub attempts: u32, pub created_at: i64, pub next_at: i64,
    pub finished_at: Option<i64>, pub last_error: Option<String> }        // serde::Serialize
pub enum AnnouncementState { Waiting, Sending, Sent, Failed }            // snake_case in serde

pub fn queue(collection, feed_id, planned: &[PlannedAnnouncement], timing: &Timing, now: u64) -> Result<(), CollectionError>;
pub fn take_due(collection, now: u64) -> Result<Option<Announcement>, CollectionError>;
pub fn next_due_at(collection) -> Result<Option<u64>, CollectionError>;
pub fn record_sent(collection, id: i64, now: u64) -> Result<(), CollectionError>;
pub fn record_failure(collection, id: i64, error: &str, timing: &Timing, now: u64) -> Result<(), CollectionError>;
pub fn expire(collection, timing: &Timing, now: u64) -> Result<(), CollectionError>;
pub fn recent(collection, feed_id) -> Result<Vec<Announcement>, CollectionError>;
```

- `queue`: for each planned announcement, a `waiting` row of the same feed and the same `url` gets the new `podping`, `created_at = now`, `next_at = now + first_wait` and `attempts = 0`. With no such row, a new `waiting` row gets these values. A `sending` row does not change (ADR 0052 item 16).
- `take_due`: the oldest `waiting` row with `next_at <= now` becomes `sending`, and the function gives it.
- `next_due_at`: the smallest `next_at` of the `waiting` rows.
- `record_sent`: the row becomes `sent`, with `finished_at = now`, `attempts + 1` and no `last_error`.
- `record_failure`: `attempts + 1` and `last_error = error`. With 5 attempts or more, the row becomes `failed` with `finished_at = now`. If not, it becomes `waiting` with `next_at = now + retry_waits[attempts - 1]`.
- `expire`: each `sending` row becomes `waiting`, because a stop of Studio interrupted it. Then each `waiting` row with `created_at + max_age < now` becomes `failed`, with `finished_at = now` and `last_error` "Studio stopped before it sent the announcement".
- After `record_sent`, `record_failure` and `expire`, delete the finished rows of the feed after its newest 10 (ADR 0052 item 19).
- `recent`: the rows of the feed, the newest first, by `created_at` and then `id`.
- `make_due(collection, now: u64)` sets `next_at = now` on each `waiting` row. It is `#[cfg(any(test, feature = "testing"))]`, so that an API test does not wait 30 seconds.

### The Sender

```rust
pub fn send(web: &Web, hub: &str, announcement: &Announcement) -> Result<(), AnnounceError>;
```

- It calls `web.get_with_query(hub, query, 64 * 1024)`. The query is `url`, and `podping=1` only when `announcement.podping` is true.
- A `WebError` gives an `AnnounceError` with the text of the `WebError`.
- A body that is JSON with `status` `false` or `"false"` gives an `AnnounceError` with its `description`, or "the hub refused the announcement". A body that is not JSON is a success.
- `AnnounceError` has `Display` and `std::error::Error`. Its text never contains a secret, because an announcement has none.

## Steps

1. Add `TestHub`, with a test of each answer.
2. Add the migration, with a migration test from the previous schema version, as the existing migration tests do.
3. Add the settings, with a test of the defaults and a test of a saved value.
4. Add the queue functions and the rules, with tests that give `now` directly.
5. Add `send`, with tests against `TestHub`: a success, `podping=1` present and absent, a 500, a refusal, and a hub that does not listen.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate. No async code.
- The core has no HTTP server code outside `testing` ([ADR 0002](../adr/0002-rust-core-json-api-browser-ui.md)). `TestHub` is test code, as the other test servers are.
- No test sends a request to a host outside the computer. No test uses `DEFAULT_HUB`.

## Acceptance Criteria

Mechanical:

- A queue test: two `queue` calls for the same address within the wait give one row, with the second `created_at`.
- A queue test: a `queue` call while the row is `sending` gives a second row.
- A retry test: five `record_failure` calls give `next_at` values of `now + 60`, `+ 120`, `+ 240` and `+ 480`, and then the state `failed`.
- An expiry test: a `waiting` row of 24 hours and 1 second becomes `failed`. A row of 23 hours stays `waiting`. A `sending` row becomes `waiting`.
- A trim test: a feed with 12 finished rows keeps its newest 10. A `waiting` row stays.
- A sender test: `TestHub` gets `url` and `podping=1`. With `podping` false, it gets no `podping` parameter.
- A sender test: a 500 and a refusal each give an error with a useful text.
- The migration test passes from the previous schema version.
- The three commit checks of Rust pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

- Stop and report when the removal of a feed needs a change to other tables.
- Stop and report when `Web::get_with_query` cannot read a 500 answer as a `WebError` with its status.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-031a-queue-and-sender.md
- docs/adr/0052-announcements-through-the-podcast-index-hub.md
- The files that the Files section names.

Goal:
- Add `testing::TestHub`, the two announcement settings, the `announcements` table with its queue rules, and `announce::send` in the core. Nothing outside the tests calls them yet.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. No async code.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.
- No test sends a request to a host outside the computer.

Do not touch:
- `crates/core/src/pipeline.rs`, the API crate, the desktop crate, the UI.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

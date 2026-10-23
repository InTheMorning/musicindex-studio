# Studio v1 Task 013a: The Recipient Store And The Musicindex Routes

Parent: [task 013](studio-v1-task-013-recipient-picker.md). Decision: [ADR 0027](../adr/0027-the-recipient-picker.md) items 2 to 6.
Sequence: the first packet of task 013. It changes only the core.

## Goal

Keep the recipients of each feed and the saved recipients in the collection. Read the payment routes of a feed and of a track from musicindex.

## Files

Inspect: `crates/core/src/collection.rs`, `crates/core/src/feed.rs`, `crates/core/src/feed/value.rs`, `crates/core/src/feed/elements.rs` (`Recipient`), `crates/core/src/musicindex.rs`, `crates/core/src/testing.rs`, and `../musicindex/api.json` (`RouteResponse`, `FeedResponse`, `TrackResponse`, `TrackSummary`).

Change: `collection.rs`, `feed.rs` and `musicindex.rs`.

Do not change: `crates/api`, the UI, `pipeline.rs`, the existing migrations 0 and 1, and the vault.

## Architecture

These choices are fixed. Do not change them.

- The identity of a recipient is its address, its custom key and its custom value. In the database, a missing custom key or custom value is the text `''`, so that the identity can be a primary key.
- Migration 2 adds two tables. `SCHEMA_VERSION` becomes 3.

  ```sql
  CREATE TABLE feed_recipients (
      feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
      address TEXT NOT NULL,
      custom_key TEXT NOT NULL DEFAULT '',
      custom_value TEXT NOT NULL DEFAULT '',
      name TEXT,
      recipient_type TEXT,
      seen_at INTEGER NOT NULL,
      PRIMARY KEY (feed_id, address, custom_key, custom_value)
  );
  CREATE TABLE saved_recipients (
      id INTEGER PRIMARY KEY,
      address TEXT NOT NULL,
      custom_key TEXT NOT NULL DEFAULT '',
      custom_value TEXT NOT NULL DEFAULT '',
      name TEXT,
      recipient_type TEXT,
      saved_at INTEGER NOT NULL,
      UNIQUE (address, custom_key, custom_value)
  );
  ```

- `Feed::recipients(&self) -> Vec<&Recipient>` in `feed.rs` gives the recipients of each value block of the channel, of each item and of each live item. It does not include the recipients of a value time split. A recipient with no address, or with an address of only spaces, is not included.
- `record_state` replaces the `feed_recipients` rows of the feed in its transaction, with `seen_at = now()`. A recipient that occurs two times in one feed gives one row. The last occurrence gives the name and the type.
- New public types and methods on `Collection`:

  ```rust
  pub struct RecipientIdentity { pub address: String, pub custom_key: Option<String>, pub custom_value: Option<String> }
  pub struct KnownRecipient {
      pub name: Option<String>, pub recipient_type: Option<String>,
      pub address: String, pub custom_key: Option<String>, pub custom_value: Option<String>,
      pub feeds: usize,            // the number of feeds that have it
      pub saved_id: Option<i64>,   // set when the user saved it
  }
  pub fn known_recipients(&self) -> Result<Vec<KnownRecipient>, CollectionError>;
  pub fn save_recipient(&self, recipient: &Recipient) -> Result<i64, CollectionError>;
  pub fn remove_saved_recipient(&self, id: i64) -> Result<(), CollectionError>;
  ```

  `KnownRecipient` and `RecipientIdentity` derive `Debug`, `Clone`, `PartialEq`, `Eq` and `Serialize`. In the API, `''` shows as `null`.
- `known_recipients` gives each identity one time, from the two tables. The name and the type come from the row with the highest `seen_at` or `saved_at`. The sequence is by name, then by address. The sequence of names ignores capital letters.
- `save_recipient` needs an address with one or more characters. With no address, it gives a `CollectionError` of the new kind `Invalid`. When the identity is saved, it changes the name, the type and `saved_at` of that row, and gives its `id`.
- `remove_saved_recipient` with an unknown `id` gives `NotFound`.
- In `musicindex.rs`, two new methods on `MusicIndex`:

  ```rust
  pub struct IndexTrack { pub track_guid: String, pub title: String }
  pub struct IndexRoutes { pub title: String, pub recipients: Vec<Recipient>, pub tracks: Vec<IndexTrack> }
  pub fn feed_routes(&self, feed_guid: &str) -> Result<Option<IndexRoutes>, WebError>;
  pub fn track_routes(&self, feed_guid: &str, track_guid: &str) -> Result<Option<IndexRoutes>, WebError>;
  ```

  `feed_routes` calls `GET {base}/v1/feeds/{guid}` with `include=payment_routes,tracks` and `limit=200`. `track_routes` calls `GET {base}/v1/feeds/{guid}/tracks/{track_guid}` with `include=payment_routes`, and gives no tracks. Filter each GUID as `feed_url` does. A 404 gives `None`.
- One route becomes one `Recipient`: `recipient_name` to `name`, `route_type` to `recipient_type`, `address`, `split` as decimal text, `custom_key`, `custom_value`, and `fee` as `Some("true")` only when it is `true`.

## Steps

1. Add `Feed::recipients` and its unit test.
2. Add migration 2, the new error kind, the types and the three methods. Change `record_state`.
3. Add `feed_routes`, `track_routes` and their serde types.
4. Add the tests of the acceptance criteria.

## Constraints

- Keep each existing test. Do not change the migrations 0 and 1.
- A recipient address is public. The tables contain no secret (ADR 0027 consequences).
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- A database at schema version 2 opens, gets the two tables, and keeps its feeds. Use the method of `the_address_migration_gives_each_old_address_its_source`.
- After `record_state` of a feed with a channel recipient and item recipients, `known_recipients` gives each identity one time, with `feeds` equal to 1.
- Two feeds with the same identity give one entry, with `feeds` equal to 2. The name of the most recent read is in the entry.
- After `record_state` of the feed with no recipients, a recipient of only that feed goes out of the list. After `delete_feed`, the recipients of that feed go out of the list.
- A saved recipient stays in the list when no feed has it. After `remove_saved_recipient`, it goes out of the list.
- `feed_routes` and `track_routes` give the routes from a `TestServer`, with `split` as text and `fee` as `"true"` or `None`. `feed_routes` gives the tracks. A 404 gives `None`.
- The test `the_database_file_contains_no_connector_secret` also records a feed with recipients and saves a recipient before it reads the file. The test must not fail.
- The three commit checks pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report when:

- A read or a write changes the recipient rows in a function that is not `record_state`.
- A corpus feed gives a recipient with an address but an unexpected structure, so that `Feed::recipients` cannot get it.
- The musicindex response is not the shape that `api.json` gives.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-013a-recipient-store.md
- docs/adr/0027-the-recipient-picker.md
- crates/core/src/collection.rs
- crates/core/src/feed.rs
- crates/core/src/feed/value.rs
- crates/core/src/feed/elements.rs
- crates/core/src/musicindex.rs
- crates/core/src/testing.rs

Goal:
- The collection keeps the recipients of each feed and the saved recipients. The musicindex client reads the payment routes of a feed and of a track.

Constraints:
- Follow the Architecture section of the task file fully: the SQL, the types, the signatures and the conversion.
- Keep each existing test and migrations 0 and 1.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- crates/api, the UI, pipeline.rs, the vault.
- Do not commit.

Acceptance criteria:
- The criteria of the task file.

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

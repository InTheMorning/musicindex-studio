# Studio v1 Task 031c: The Worker And The Routes

Parent: [task 031](studio-v1-task-031-announce.md). Decisions: [ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md) items 1, 5, 6, 14 to 18 and 20, and [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md).
Sequence: the third packet of task 031. It depends on 031b.

## Goal

A worker thread in `studio-api` sends each due announcement to the hub. The API gives the settings and the announcements of a feed. The desktop shell can select a different hub.

## Files

Inspect: `crates/core/src/announce.rs`, `crates/api/src/downloads.rs` (the form of a worker with a weak workspace reference), `crates/api/src/workspace.rs` (`Workspace`, `open`, `with_musicindex`, `resume_downloads`), `crates/api/src/lib.rs` (the routes), `crates/api/src/routes.rs` (`write_feed`, `get_library`, `put_library_limits`), `crates/api/src/tests.rs`, `crates/desktop/src/main.rs`.

Change: `crates/api/src/lib.rs`, `crates/api/src/routes.rs`, `crates/api/src/workspace.rs`, `crates/api/src/tests.rs`, `crates/desktop/src/main.rs`, and the new file `crates/api/src/announcer.rs`.

Do not change: the core, the UI.

## Architecture

These choices are fixed. Do not change them.

### The Workspace

- `Workspace` gets `hub: String`, with `announce::DEFAULT_HUB` at first, and `announce_timing: Timing`, with `Timing::DEFAULT` at first. It also gets `announcer: Arc<Announcer>`.
- `with_hub(self, url: &str) -> Self` sets the hub. It is public, as `with_musicindex` is.
- `with_announce_timing(self, timing: Timing) -> Self` is `#[cfg(test)]`. A test uses it for a short `retry_waits`.
- `resume_announcements(&self)` calls `announce::expire` with `announce_timing` and the time now, and then starts the worker. The desktop shell calls it after `resume_downloads`.

### The Worker

- `announcer.rs` has one worker thread. It keeps a weak reference to the workspace, as the download queue does. When the workspace is gone, the thread ends.
- The loop:
  1. Lock the workspace. Call `announce::take_due` with the time now. Copy `web` and `hub`. Unlock.
  2. With an announcement, call `announce::send` with no lock on the workspace.
  3. Lock the workspace. Call `record_sent` or `record_failure` with `announce_timing`. Unlock. Go to step 1.
  4. With no due announcement, wait on a `Condvar` until `next_due_at`, or for a maximum of one second.
- `Announcer::wake()` wakes the worker. The write route calls it after a write with no error.
- The time now comes from one function, so that the tests can read the same clock.
- An API test does not wait 30 seconds. After the write, it locks the workspace, calls `announce::make_due` and calls `Announcer::wake()`. The core tests examine the first wait.

### The Routes

- `GET /api/v1/settings/announcements` gives `{"pubnotify": true, "podping": true}`.
- `PUT /api/v1/settings/announcements` with the same body saves the two settings, and gives them. A body with a missing field gives 422.
- `GET /api/v1/feeds/{id}/announcements` gives `announce::recent` of the feed as a JSON list. A feed that does not exist gives 404, as the other feed routes do.
- These routes need no vault.

### The Desktop Shell

- `--podcastindex-hub URL` selects the hub, as `--musicindex URL` selects the musicindex API. The help text in the file header names it.

## Steps

1. Add the workspace fields and `with_hub`.
2. Add `announcer.rs` and the call to `wake` in `write_feed`.
3. Add `resume_announcements` and the call in the desktop shell.
4. Add the three routes.
5. Add the desktop flag.
6. Add the API tests of the acceptance criteria against `TestHub`.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate.
- The worker never holds the workspace lock during a request.
- A failure of the worker does not change a write.
- No test sends a request to a host outside the computer. Each API test sets the hub to a `TestHub`.

## Acceptance Criteria

Mechanical:

- An API test: a write of a feed with a public address. Within 5 seconds, `TestHub` gets one request with the feed address and `podping=1`. `GET /feeds/{id}/announcements` then gives one `sent` entry.
- An API test: `PUT /settings/announcements` with `pubnotify` false, then a write. After 2 seconds, `TestHub` has no request, and the feed has no announcement.
- An API test: `PUT` with `podping` false, then a write. The request has no `podping` parameter.
- An API test: `TestHub::fail_next(1)` and a short `retry_waits`. The feed shows `waiting` with `attempts` 1 and the error text, and then `sent`.
- An API test: `GET /settings/announcements` gives the defaults on a new workspace.
- An API test: `resume_announcements` marks a `waiting` row older than 24 hours as `failed`.
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

- Stop and report when a test needs a sleep of more than 5 seconds.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-031c-worker-and-routes.md
- docs/adr/0052-announcements-through-the-podcast-index-hub.md
- The files that the Files section names.

Goal:
- A worker thread in studio-api sends each due announcement to the hub. Add the settings routes, the announcements route of a feed, and the `--podcastindex-hub` flag of the desktop shell.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.
- The worker never holds the workspace lock during a request.

Do not touch:
- The core, the UI.
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

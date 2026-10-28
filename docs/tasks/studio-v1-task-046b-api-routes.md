# Studio v1 Task 046b: The API Routes For A Read-Only Draft

Parent: [task 046](studio-v1-task-046-read-only-editor.md). Decisions: [ADR 0067](../adr/0067-a-read-only-feed-opens-in-the-editor.md) items 3 to 7 and 12, [ADR 0041](../adr/0041-publisher-feeds-and-two-way-links.md) item 5 and [ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md).
Sequence: the second packet of task 046. It depends on 046a.

Status: Ready.

## Goal

The draft routes work for a feed with no connector. The host route records the host choice of a read-only draft. The write route writes a moved read-only draft. A folder match keeps the draft of a feed that it links.

## Files

Inspect: `crates/api/src/routes.rs` (`writer`, `ensure_draft`, `check_mediums`, `get_draft`, `put_draft`, `delete_draft`, `draft_json`, `refresh_feed`, `set_feed_host`, `write_feed`, `match_connector`, `give_linked_feed_folder`, `set_files_host`), `crates/api/src/lib.rs` (the routes), `crates/api/src/tests.rs` (the tests of `/feed-host`, `/write` and `/connectors/{id}/match`), the new functions of packet 046a in `crates/core/src/pipeline.rs`.

Change: `crates/api/src/routes.rs`, `crates/api/src/tests.rs`. Change `crates/api/src/lib.rs` only for the new route of this packet.

Do not change:

- `crates/core`. Packet 046a gives each core function.
- The routes of `/app/v1/` and the granted writes.
- The link routes of ADR 0041. They refuse a read-only feed, as now.
- The UI.

## Architecture

These choices are fixed. Do not change them.

- `ensure_draft`: for a feed with no connector, call `pipeline::start_public_draft` with the library of the workspace. A feed with a connector keeps the call of `writer` and `start_draft`.
- `writer` does not change. Each route that writes to a host for a feed with a connector keeps it.
- Each route that only reads or changes the draft works for a feed with no connector: `get_draft`, `put_draft`, `delete_draft`, the diff route and each route that edits the model. Replace each call of `writer` in these routes with `ensure_draft`, or with no check when the route needs no host.
- `set_feed_host` with a body `{ "connector_id": null }` calls `pipeline::clear_feed_host` for a feed with no connector. A feed with a connector refuses `null` with 422, as an unknown body does now.
- `write_feed`: for a feed with no connector, call `pipeline::write_public_draft` with `w.web`. The response is the same as for each other write.
- `match_connector`: after `set_feed_connector`, call `pipeline::rebase_linked_draft` with the connector. An error goes into the log, as `give_linked_feed_folder` does, and the link stays.
- `check_mediums` does not change. A link of a publisher feed and a read-only release stays refused (ADR 0041 item 5, ADR 0067 item 12).
- The draft response gives `"has_host": false` for a feed with no connector and no draft feed move, and `true` in each other case.

## Steps

1. Change `ensure_draft` and the draft routes.
2. Change `set_feed_host` and `write_feed`.
3. Change `match_connector`.
4. Add `has_host` to the draft response.
5. Add the API tests.
6. Run the four checks.

## Constraints

- In local mode, each `/api/` request needs the access token and a loopback `Host` (ADR 0017). On the server, each `/api/` request needs the session cookie and `X-Studio-CSRF` (ADR 0055). The new behavior keeps each check.
- The write route reads the public address only through `w.web`, so that the network policy of the server applies (ADR 0057).
- No secret is in a log or a response (ADR 0003).
- Each code comment and doc comment is in ASD-STE100 Simplified Technical English, as the files are.
- Apply the `rust-dev` skill.

## Acceptance Criteria

Mechanical, each one a test in `crates/api/src/tests.rs`:

- `GET /api/v1/feeds/{id}/draft` of a read-only feed with a release folder gives the model and `has_host` false.
- `PUT /api/v1/feeds/{id}/draft` of a read-only feed saves an edit, and the diff route gives the unified diff.
- `POST /api/v1/feeds/{id}/write` of a read-only draft with no host gives 409 with the code `read_only`.
- `PUT /api/v1/feeds/{id}/feed-host` with a local folder connector gives a diff with a feed move and the method `none`. Then `has_host` is true.
- `PUT /api/v1/feeds/{id}/feed-host` with `null` removes the move.
- A write of the moved draft gives 200, and the feed row then has the connector and the path.
- A folder match that links a read-only feed with a draft keeps the draft.
- A link of a publisher feed and a read-only release stays refused with 409.
- The server shell has the network policy `Public`. A read-only draft has a loopback address. Its write gives the error of the network policy.
- The four commit checks pass.

Visual: none in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a draft route needs a connector for a reason that this packet does not name.
- Stop and report when the server shell has no test helper for the network policy `Public`.
- Stop and report when a core function of packet 046a is missing or has a different signature.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-046b-api-routes.md
- docs/tasks/studio-v1-task-046a-core-draft-with-no-host.md
- docs/adr/0067-a-read-only-feed-opens-in-the-editor.md
- The files that the Files section names.

Goal:
- The draft routes, the host route and the write route work for a feed with no connector.
- A folder match keeps the draft of a feed that it links.

Constraints:
- Follow the Architecture section of the task file fully.
- Read the public address only through `w.web`.
- Apply the rust-dev skill. Write each comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- `crates/core`, the routes of `/app/v1/`, the granted writes, the link routes of ADR 0041 and the UI.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

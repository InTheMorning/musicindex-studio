# Studio v1 Task 013b: The Recipient Routes

Parent: [task 013](studio-v1-task-013-recipient-picker.md). Decision: [ADR 0027](../adr/0027-the-recipient-picker.md) items 2 to 6.
Sequence: after [task 013a](studio-v1-task-013a-recipient-store.md).

## Goal

Give the UI the recipient list, and the save and the removal of a saved recipient. Give the UI the payment routes of musicindex.

## Files

Inspect: `crates/api/src/lib.rs` (`local_router`), `crates/api/src/routes.rs` (`list_feeds`, `add_feed`, `search`, `blocking`), `crates/api/src/error.rs`, `crates/api/src/tests.rs` (`app`, `app_with_index`, `call`, `web_feeds`, `search_gives_feed_hits_from_musicindex`), and the core API of task 013a in `crates/core/src/collection.rs` and `crates/core/src/musicindex.rs`.

Change: `lib.rs`, `routes.rs`, `error.rs` and `tests.rs`.

Do not change: `crates/core`, the UI, and the existing routes.

## Architecture

These choices are fixed. Do not change them.

| Method and path | Handler | Result |
| --- | --- | --- |
| `GET /api/v1/recipients` | `list_recipients` | 200, the array of `known_recipients` |
| `POST /api/v1/recipients/saved` | `save_recipient` | 201, `{ "id": n }` |
| `DELETE /api/v1/recipients/saved/{id}` | `remove_saved_recipient` | 204 |
| `GET /api/v1/musicindex/feeds/{guid}/recipients` | `index_feed_recipients` | 200, `{ title, recipients, tracks }` |
| `GET /api/v1/musicindex/feeds/{guid}/tracks/{track}/recipients` | `index_track_recipients` | 200, `{ title, recipients, tracks: [] }` |

- The body of `POST /recipients/saved` is a recipient in the field names of the UI model: `name`, `recipient_type`, `address`, `split`, `custom_key`, `custom_value`, `fee`. The route deserializes it as the core `Recipient`. Studio does not keep the split and the fee of a saved recipient.
- The collection error kind `Invalid` maps to 422 with the code `invalid_recipient`.
- A musicindex 404 (`None`) gives 404 with the code `not_found`. A network error gives the existing `network` code.
- The musicindex routes run the request in `blocking`, as `search` does.

## Steps

1. Add the five routes to `local_router`.
2. Add the handlers and the error mapping.
3. Add the tests of the acceptance criteria. For the musicindex routes, set `/v1/feeds/b-1` and `/v1/feeds/b-1/tracks/t-1` on a `TestServer` with `payment_routes` and `tracks`.

## Constraints

- Each route needs the access token, as the other `/api/` routes do ([ADR 0017](../adr/0017-local-api-access.md)). Do not let a route operate with no token.
- No response contains a secret. The test `no_get_response_contains_a_secret` must include `GET /api/v1/recipients`.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

## Acceptance Criteria

Mechanical:

- After `set_up` with a feed that has recipients, `GET /api/v1/recipients` gives each recipient one time.
- `POST /api/v1/recipients/saved` gives 201 and an `id`. The recipient is then in the list with that `saved_id`. With no address, it gives 422 and `invalid_recipient`.
- `DELETE /api/v1/recipients/saved/{id}` gives 204, and an unknown `id` gives 404.
- `GET /api/v1/musicindex/feeds/b-1/recipients` gives the routes and the tracks from the test server. An unknown GUID gives 404.
- `GET /api/v1/musicindex/feeds/b-1/tracks/t-1/recipients` gives the routes of the track.
- A request with no token to `GET /api/v1/recipients` gives 401.
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

Stop and report when a route needs a change to `crates/core`. Stop and report when the router does not accept a path with `{id}`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-013b-recipient-routes.md
- docs/adr/0027-the-recipient-picker.md
- crates/api/src/lib.rs
- crates/api/src/routes.rs
- crates/api/src/error.rs
- crates/api/src/tests.rs
- crates/core/src/collection.rs (the recipient methods)
- crates/core/src/musicindex.rs

Goal:
- Five API routes: the recipient list, save, remove, and the payment routes of a musicindex feed and track.

Constraints:
- Follow the Architecture table and rules of the task file fully.
- Each route needs the access token. No response contains a secret.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- crates/core, the UI, the existing routes.
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

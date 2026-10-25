# Studio v1 Task 025b: The Publisher Routes

Parent: [task 025](studio-v1-task-025-publisher-feeds.md). Decisions: [ADR 0041](../adr/0041-publisher-feeds-and-two-way-links.md) items 1, 2, 4, 5, 8 and 11, [ADR 0017](../adr/0017-local-api-access.md).
Sequence: the second packet of task 025. It depends on 025a.

## Goal

The local API makes a publisher feed with its role, adds and removes a link, and changes the role. Each route starts the drafts that the core needs.

## Files

Inspect: `crates/api/src/lib.rs` (the router), `crates/api/src/routes.rs` (`new_album_feed`, `NewAlbumFeed`, `writer`, `use_new_file` for the start of a draft, `blocking`), `crates/api/src/error.rs` (the `From` conversions), `crates/api/src/tests.rs` (`set_up`, `call`, `a_new_release_is_made_as_a_draft_with_no_hosted_file_and_then_written`), `crates/core/src/publisher.rs` from packet 025a.

Change: `lib.rs`, `routes.rs`, `error.rs`, `tests.rs`.

Do not change: `crates/core/`, the UI, `crates/desktop/`, `crates/server/`.

## Architecture

These choices are fixed. Do not change them.

### The New Feed

`POST /api/v1/feeds/new` takes two more fields in its body:

- `medium`: `"music"` or `"publisher"`. The default is `"music"`. Each other value gives 422 `invalid_feed`.
- `role`: `"artist"` or `"label"`. It is necessary for a publisher feed. A publisher feed with no `role` gives 422 `invalid_feed`. A music feed ignores it.

The route calls `pipeline::new_feed` with the medium, and then `Collection::set_publisher_role` for a publisher feed. The error text of an empty field names the name, not the album title, for a publisher feed. The response is the feed row, with `publisher_role`.

### A Draft For Each Feed Of A Change

A new helper `ensure_draft(w, id)` starts the draft of a feed with `pipeline::start_draft` and `writer` when the feed has no draft. It does nothing for a feed with a draft.

### The Link Routes

| Route | Steps |
| --- | --- |
| `POST /api/v1/feeds/{id}/links` with `{ "release": <feed id> }` | `ensure_draft` of the publisher feed and of the release, then `publisher::link` |
| `DELETE /api/v1/feeds/{id}/links/{release}` | `ensure_draft` of the two feeds, then `publisher::unlink` |
| `PUT /api/v1/feeds/{id}/role` with `{ "role": "artist" }` | `ensure_draft` of the publisher feed, then of each feed of `publisher::linked_releases`, then `publisher::set_role` |

`{id}` is always the publisher feed. Each route gives 200 with this body:

```json
{ "changed": [ { "id": 3, "title": "Check Album" } ] }
```

`changed` lists each feed with a draft that the route changed, `{id}` too, in the sequence of the IDs. Each page leaves out its own feed. The title is the title of the feed row.

### The Errors

| Kind of `PublisherError` | Status and code |
| --- | --- |
| `NotPublisher` | 422 `not_publisher` |
| `NotRelease` | 422 `not_release` |
| `ReadOnly` | 409 `read_only`, as a draft route of a read-only feed gives it now |
| `NoRole` | 409 `role_required` |
| `NoGuid` | 422 `no_guid` |
| `NoAddress` | 422 `no_public_address` |
| `Pipeline` | The conversion of `PipelineError` that the API has now |

## Steps

1. Add `medium` and `role` to the new feed route, with tests.
2. Add `ensure_draft` and the three routes, with tests.
3. Add the error conversion.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate. Each route uses `blocking`, as the other routes do.
- Each new route is under `/api/v1/`, so it needs the access token ([ADR 0017](../adr/0017-local-api-access.md)).

## Acceptance Criteria

Mechanical:

- `POST /feeds/new` with `medium` `publisher` and `role` `label` gives a feed row with the medium `publisher` and `publisher_role` `label`. Its draft has `podcast:medium` `publisher`.
- `POST /feeds/new` with `medium` `publisher` and no `role` gives 422 `invalid_feed`.
- A link between a new publisher feed and a new release gives `changed` with the release. The draft of each feed has the item of ADR 0041 items 6 and 7, with `rel` `label`.
- A link to a written release with no draft starts its draft, and the draft has the link.
- `DELETE /feeds/{id}/links/{release}` takes the link out of the two drafts.
- `PUT /feeds/{id}/role` with `artist` changes `rel` in the two drafts, and `changed` lists the release.
- A link from a music feed as `{id}` gives 422 `not_publisher`. A link to a read-only feed gives 409 `read_only`.
- A request with no token to each new route gives 401.
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

Stop and report when a route needs a change of `crates/core/`.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-025b-publisher-routes.md
- docs/adr/0041-publisher-feeds-and-two-way-links.md
- The files that the Files section names.

Goal:
- The local API makes a publisher feed with its role, adds and removes a link, and changes the role. Each route starts the drafts that the core needs.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. Each new route is under /api/v1/.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- crates/core/, the UI, crates/desktop/, crates/server/.
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

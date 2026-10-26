# Studio v1 Task 033b: The Live Item Page

Parent: [task 033](studio-v1-task-033-live-items.md). Decisions: [ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md) items 1 to 5, [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md), [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md), [ADR 0037](../adr/0037-button-roles.md) and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
Sequence: the second packet of task 033. It depends on 033a.

## Goal

The user adds a live show to an album feed, or makes a show feed. The live item page edits the show. "Go live" and "End the show" change the draft and open the review sheet.

## Files

Inspect: `crates/core/src/live.rs`, `crates/api/src/routes.rs` (`new_album_feed`, the draft routes), `crates/api/src/error.rs`, `crates/api/ui/editor.js` (`AlbumPage`, the track page, `ReviewSheet`, the item value editor), `crates/api/ui/app.js` (`AddFeeds`, the view routing), `crates/api/ui/shell.js` (`leftPanel`), `crates/api/ui/playlist.js` (the form of a model module), `crates/api/src/ui.rs`.

Change: `crates/api/src/routes.rs`, `crates/api/src/lib.rs`, `crates/api/src/tests.rs`, `crates/api/ui/editor.js`, `crates/api/ui/app.js`, `crates/api/ui/shell.js`, `crates/api/ui-tests/shell.test.mjs`, the new files `crates/api/ui/live.js` and `crates/api/ui-tests/live.test.mjs`, `crates/api/src/ui.rs` (one entry), `crates/api/ui/styles.css` (only when a class is necessary).

Do not change: the core other than a small helper with its test, the connected apps and the grants (033d to 033f).

## Architecture

These choices are fixed. Do not change them.

### The Routes

- `POST /api/v1/feeds/{id}/live-items` with `{"title", "start"}` adds `live::new_live_item` to the draft model of the feed, and gives the draft. An empty title or start gives 422 `invalid_live_item`. A feed that is not `music` gives 422 `invalid_live_item`.
- `POST /api/v1/feeds/{id}/live-items/{guid}/status` with `{"status": "live"}` or `{"status": "ended"}` calls `live::set_status` on the draft model, and gives the draft. A refused change gives 409 `bad_status`. The route does not write.
- A show feed is a new album feed. The new feed form gets the kind "Show feed". It sends `medium` `music`. The only difference in the UI is the title of the form.

### The Draft Model

An entry of `live_items` in the draft model has `status`, `start`, `end`, `item` (the fields of an item, such as `title`, `guid`, `description`, `image`, `enclosure` and `values`), `live_value` (`uri` and `protocol`, or `null`), `content_links` (each with `href` and `label`) and the entry key `_cid`. 033a added `live_value` and `content_links`. A new live item has no content link, so its page shows the problem until the user adds one.

### The Model Module

`live.js` has no Preact and no browser state. It exports:

- `statusLabel(status)` gives "Planned", "Live now" or "Ended".
- `nextAction(status)` gives `{ label: 'Go live', status: 'live' }`, `{ label: 'End the show', status: 'ended' }` or `null`.
- `showSummary(liveItem, timeText)` gives one summary line: the status label and the start time, for example "Planned · starts 10/26/2026, 8:00 PM".
- `liveProblems(liveItem)` gives the texts of the problems of ADR 0054 item 4, the same texts as `live::problems` of the core.

### The Pages

- The outline of an album feed and of a show feed gets a "Live shows" entry after the tracks. It has each live item as a summary line, and "Add a live show".
- The live item page edits the title, the description, the image, `start` and `end`. It also edits the stream enclosure (address and type), `podcast:contentLink` (address and text), and the item value routes with the value editor of a track. It shows `podcast:liveValue` (address and protocol) as two fields.
- The page shows the status and the button of `nextAction` with the primary role. The button calls the status route, loads the draft, and opens the review sheet.
- The page shows each text of `liveProblems` with the `--error` token.

## Steps

1. Add the two routes, with API tests.
2. Add `live.js` and its Node tests.
3. Add the outline entries and the live item page.
4. Add "Show feed" to the new feed form.
5. Do the visual check of the acceptance criteria.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the files are.
- The UI loads nothing from other servers, and it has no build step.
- Each control has the target size of its profile. Each function works from the keyboard.

## Acceptance Criteria

Mechanical:

- An API test adds a live item, writes it, sets `live`, writes, sets `ended`, and writes. The hosted file has each status in sequence and the same `guid`.
- An API test gives 409 `bad_status` for `ended` to `live`.
- The Node tests examine each function of `live.js` and the outline entries.
- The four commit checks pass.

Visual:

- The planning model checks with `tools/drive-ui.mjs`: the outline with "Live shows", the live item page with a problem, the review sheet after "Go live", and the "Show feed" form. Each one in the dark theme, in the light theme and at 390 px.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the track page cannot give its fields to the live item page without a large change.
- Stop and report when the draft model of a live item has no `item` part.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-033b-live-item-page.md
- docs/adr/0054-live-items-connected-apps-and-show-grants.md
- The files that the Files section names.

Goal:
- Add the routes that add a live item and change its status in the draft.
- Add the model module `live.js` with Node tests, and the "Live shows" outline entry.
- Add the live item page with "Go live" and "End the show".
- Add the "Show feed" kind of the new feed form.

Constraints:
- Follow the Architecture section of the task file fully.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.
- The status routes change the draft only. They never write.

Do not touch:
- The core, other than a small helper with its test. The connected apps and the grants.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file. The planning model does the visual check.

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

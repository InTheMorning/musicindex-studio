# Studio v1 Task 025d: The Publisher Feeds Of A Release

Parent: [task 025](studio-v1-task-025-publisher-feeds.md). Decisions: [ADR 0041](../adr/0041-publisher-feeds-and-two-way-links.md) items 5, 7 to 9 and 13, [ADR 0037](../adr/0037-button-roles.md).
Sequence: the last packet of task 025. It depends on 025c.

## Goal

The album page of a release shows its publisher feeds in their sequence. The user adds, removes and orders them there.

## Files

Inspect: `crates/api/ui/editor.js` (`AlbumPage`, `AlbumEditor`, `PublisherPage` from packet 025c and its helpers for the match of a feed and for the notice of a changed draft), `crates/api/ui/app.js` (`Shell`, `EditableFeed`), `crates/api/ui/styles.css`.

Change: `editor.js`, and `app.js` only to give `AlbumPage` the feed list and the load of the feed list.

Do not change: `crates/core/`, `crates/api/src/`, `values.js`, `draft.js` and the track page. Change the publisher page only to move a helper of 025c, so that the two pages use it.

## Architecture

These choices are fixed. Do not change them.

### The Section

`AlbumPage` gets a section "Publisher feeds" after "Album details", with one group:

- One row for each entry of `model.publisher.remote_items`, in model sequence. Each row shows the title, or the `feed_url` when it has no title. Its second line gives the role from `rel`, and "Not in the collection" for a row with no match. The first row also gives "Primary" (ADR 0041 item 9).
- Each row has "Move up" and "Move down" with the normal role, and "Remove" with the destructive role.
- Below the rows is a `select` of the candidate publisher feeds with the action button "Add the Publisher Feed". With no row, the group shows "No publisher feeds yet.".

### The Rules

- A row matches a feed of the feed list with the rule of packet 025c. A candidate is a feed with the medium `publisher`, a connector, and no matching row.
- "Move up" and "Move down" change the sequence of `model.publisher.remote_items` only. They change no other feed (ADR 0041 item 9).
- "Remove" of a matched row calls `DELETE /feeds/{publisher}/links/{this feed}`. "Remove" of a row with no match takes the entry out of the model. When the list then has no entry, `model.publisher` becomes `null` (ADR 0041 item 13).
- "Add the Publisher Feed" calls `POST /feeds/{publisher}/links` with this feed as `release`.
- Each call does the four steps of "A Change Of Two Drafts" in packet 025c. The two pages use one helper for these steps.
- For `role_required`, the text is "Open the publisher feed and select its role first."

## Steps

1. Put the match helper and the change helper of 025c together in `editor.js`. Skip this step when 025c did it.
2. Add the section and its calls.
3. Run the `drive-ui.mjs` check below, and give the screenshots to the planning model.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No build step and no new UI file ([ADR 0016](../adr/0016-ui-without-a-build-step.md)).
- Each new button label is in title case (musicindex ADR 0023 R-CASE-1).
- All functions work from the keyboard.

## Acceptance Criteria

Mechanical:

- The UI tests of `crates/api/src/tests.rs` pass.
- The three commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`:

- The album page of a release with no publisher feed shows "No publisher feeds yet." and the candidates.
- "Add the Publisher Feed" adds an artist feed and then a label feed. The first row shows "Primary". The notice names each publisher feed.
- "Move down" on the first row makes the label feed the primary one, and the review of the release shows the new sequence in `podcast:publisher`.
- "Remove" takes one publisher feed out, and the notice names it.
- The page in the dark theme and in the light theme.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run Studio with a data folder in `.studio-dev/`, as `AGENTS.md` says. Drive the UI with `tools/drive-ui.mjs`. When Node.js is not on the PATH, run the script with Bun.

## Escalation

Stop and report when a step needs a change of `crates/api/src/` or of the core.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-025d-release-publishers.md
- docs/tasks/studio-v1-task-025c-publisher-page.md
- docs/adr/0041-publisher-feeds-and-two-way-links.md
- The files that the Files section names.

Goal:
- The album page of a release shows its publisher feeds in their sequence. The user adds, removes and orders them there.

Constraints:
- Follow the Architecture section of the task file fully.
- No build step and no new UI file. All functions work from the keyboard.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- crates/core/, crates/api/src/, values.js, draft.js, the track page.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file. Give the screenshots of the visual criteria to the planning model.

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

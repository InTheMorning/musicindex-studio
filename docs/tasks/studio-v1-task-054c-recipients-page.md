# Studio v1 Task 054c: The Recipients Page And The Review Of A Change Set

Parent: [task 054](studio-v1-task-054-recipient-change-sets.md). Decision: [ADR 0091](../adr/0091-a-change-set-edits-one-recipient-in-many-feeds.md) items 1 to 4, 8, 9 and 21 to 24.

Status: Complete, with its screenshot visual check. The Recipients page also lists the open change sets, so that the user can go back to a review. The check did not show a feed with `read_only_no_host`. An API test of 054b examines it.

## Goal

The user opens Recipients, opens one recipient, selects an operation and its feeds, examines the review, and writes each feed.

## Files

Inspect: `crates/api/ui/shell.js` (`leftPanel`), `crates/api/ui/app.js` (`LeftPanelFooter`, `RootPage`, the views, `Bar`, `useSheet`), `crates/api/ui/editor.js` (`ReviewSheet`, `DiffView`, the signer flow), `crates/api/ui/signing.js`, `crates/api/ui/values.js`, `crates/api/ui/pages.js`, `crates/api/ui-tests/shell.test.mjs`.

Change: a new model module `crates/api/ui/recipients.js`, `crates/api/src/ui.rs` (it serves `recipients.js` as it serves `op3.js`), `crates/api/ui/shell.js`, `crates/api/ui/app.js`, `crates/api/ui/pages.js`, `crates/api/ui/styles.css`, and the Node tests.

Do not change: the core, the API, the ADRs.

## Architecture

These choices are fixed. Do not change them.

1. `leftPanel` gives the bottom entries `recipients`, `hosts`, `settings`, then `lock` or `unlock`. The `recipients` entry has the icon `wallet` of task 056a. The views are `recipients`, `recipient` and `change-set`.
2. The Recipients page lists each recipient of `GET /recipients` in one summary line: the name, the type, and "N feeds" or "Saved, not used" (ADR 0028). The list is A to Z by name.
3. The page of a recipient gets its feeds from `recipients/uses`. Each feed is one row with a checkbox, its title and its places, for example "Channel · 12 tracks". Each checkbox starts selected.
4. The page has four action buttons: "Change address", "Rename", "Remove" and "Add to a level". Each one opens a sheet. "Remove" is destructive (ADR 0037).
5. "Add to a level" has a select menu of the publisher feeds, and a field "Percentage" from 1 to 99. The sheet then shows the feeds of `feeds/{id}/level` with checkboxes.
6. The submit of a sheet sends `POST /change-sets`, then opens the review of the change set.
7. The review lists each feed with its outcome. A changed feed opens its diff in a closed `details` entry, with `DiffView`. A feed that Studio cannot write shows its cause. One entry is open at a time (ADR 0028).
8. "Write N feeds" writes each feed one at a time: `GET /feeds/{id}/diff`, the signer flow of `ReviewSheet` when the diff has events to sign, then `POST /feeds/{id}/write`. Each feed shows "Waiting", "Writing", "Written" or the error text. A failure does not stop the next feed.
9. "Discard" sends `DELETE /change-sets/{id}`. A change set with a written feed asks for a confirmation first.
10. `recipients.js` gives the row text, the places text, the outcome text, the cause text and the write status. It has no Preact and no browser state (ADR 0042).

## Steps

1. Write `recipients.js` with Node tests.
2. Write the pages, the sheets and the review in `app.js`.
3. Take the screenshots, and run `$edges()`.
4. Run the four commit checks.

## Constraints

- No feed rule in the UI (ADR 0065). The UI does not calculate a split, and does not find a place.
- A text names a connector "file host" (ADR 0092). The word test of task 056b examines the new texts.
- Each labeled button has a role (ADR 0037).
- Each control has the target size of its profile.
- Each UI text is in ASD-STE100 Simplified Technical English.

## Acceptance Criteria

Mechanical:

- A Node test: `leftPanel` gives `recipients`, `hosts`, `settings`, then `lock`.
- A Node test: `recipients.js` gives the summary line of a used recipient and of a saved recipient that no feed uses.
- A Node test: the places text of a feed with the channel and 12 tracks, and of a feed with only live items.
- A Node test: the outcome text of each cause of task 054a.
- A Node test: the write status after one failure keeps the next feed "Waiting".
- The four commit checks pass.

Visual, with `tools/drive-ui.mjs`, in the dark theme, in the light theme and at 390 px:

- The Recipients page, the page of a recipient, and each sheet.
- The review of a change set with a changed feed, a feed with no change, and a read-only feed.
- The review during a write with one feed that fails.
- `$edges()` finds no content at the edge of a group.

## Escalation

- Stop and report when the signer flow of `ReviewSheet` cannot run for a feed outside its sheet. Do not copy it. Move the shared part into a function that both use.

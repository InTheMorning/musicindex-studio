# Studio V1 Task 102: Update From Linked Pages

Status: Waits for task 101.

Owner of the rules: [ADR 0170](../adr/0170-update-people-from-their-linked-pages.md). It uses ADR 0159 items 8, 9 and 13 to 15, ADR 0116 and ADR 0132.

## Goal

A user applies all suggestions of a person with one button. A user updates many people from their linked pages with a review sheet.

## Packets

- 102a: the core and the route give the changes of a batch for many people (ADR 0170 items 2, 3 and 6).
- 102b: **Use All Suggestions** on the page of a person (items 1 to 3).
- 102c: **Update from Linked Pages** and its review sheet on the People page (items 4 to 6).

## Files To Inspect

- `crates/core/src/people.rs`: `page_suggestions`, `PageSuggestion`, `PageWallet` and the save of a person (search for the function that the route `/people/{id}/save` calls).
- `crates/api/src/people.rs`: the route `page_suggestions` (around line 150) and `save_person`.
- `crates/api/src/lib.rs`: the routes of `/people` (around lines 237 to 295).
- `crates/api/ui/people.js`: `pageSuggestions` (around line 1054), `pendingStatus`, `saveBody`, `pendingKeys`, `peopleSections`.
- `crates/api/ui/peoplepages.js`: the page of a person (`fromPages`, `pageRow`, around line 846), the People page with its folds `suggestions` and `hidden` (around lines 130 to 190), `PickFold`, and `useLeaveGuard` (line 823).
- `crates/api/ui/confirms.js`: the form of a sheet with a footer.
- `crates/api/ui-tests/people.test.mjs` and `crates/api/src/tests.rs` (around line 6756).

## Constraints

- No feed rule in the UI (ADR 0065).
- UI files are in `crates/api/ui/*.js` (Preact + htm). `node --check` misses template errors, so check a `.mjs` copy of each changed UI file.
- The change of a person changes no feed (ADR 0170 item 6).
- **Use All Suggestions** makes pending changes only. The user saves or discards them (ADR 0116).
- The core decides which values change for each person. The UI has no copy of the test of a matching wallet.
- The collection must not hold a lookup of a suggestion that the user did not add (ADR 0097 item 3).

## Acceptance Criteria

Mechanical, with Rust tests:

- The batch gives the changes for two people. Each change names the person and the values that change.
- The batch leaves out a wallet that does not match (ADR 0159 item 13).
- The batch leaves out a suggestion that the user hid.
- The batch saves only the selected people, and a person that is not selected does not change.
- The batch changes no feed and no change set.

Mechanical, with Node tests:

- The model of the page of a person gives **Use All Suggestions** only when the person has a suggestion. It turns each suggestion into a pending change.
- The model of the People page gives **Update from Linked Pages** only when one or more people have suggestions.
- The model of the review sheet lists each person with the values that change. Each person starts selected.

Visual, by the agent, with screenshots by `tools/drive-ui.mjs` at 1100 px and at 390 px with touch, in the dark and the light theme. Run `$edges()` and `$overflow()` with each screenshot:

- The button on the page of a person, before and after a click.
- The button on the People page.
- The review sheet with the people, with one person cleared.
- The People page after **Update**.

The screenshots show the button and the review to the operator before the commit (ADR 0170).

Human, by the operator: a batch update of people that link to real pages.

## Test Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation Triggers

- The batch needs a new route. Give it the shape of the other People routes, and name it in the report.
- Music Index refuses the reads of many pages in a short time. ADR 0170 item 5 reads them only when the user selects the button.
- Task 101 is not complete, so the matching wallets are not yet separate from the others.

# Studio V1 Task 103: Search Sheet Keeps Its Field Still

Status: Ready.

Owner of the rules: [ADR 0168](../adr/0168-a-search-sheet-keeps-its-field-still.md). Evidence: finding 7 of the [rc.21 findings](../reviews/2026-11-10-rc21-human-test-findings.md).

## Goal

The field of a search sheet does not move when the user types, when the field list opens or closes, or when the results change. The text "no result" shows only after a finished search.

## Files To Inspect

- `crates/api/ui/pickers.js`: `FindOneSheet` (around line 250), `SearchResults` (around line 223) and `useFieldList`.
- `crates/api/ui/peoplepages.js`: `FindSheet` (around line 545). It is the sheet "Link to an Artist or Label Page" and "Find on Music Index: NAME".
- `crates/api/ui/values.js`: the sheet "Copy from a Feed" (around line 403).
- `crates/api/ui/editor.js`: the sheet "Find a Nostr profile" (around line 714), and the playlist picker (line 2618).
- `crates/api/ui/fieldlist.js`: the model of the field list.
- `crates/api/ui/styles.css`: `dialog.sheet.search-sheet[open]` (around line 754), `.sheet`, the media block at 760 px (around line 798), and the field list near line 1568.
- `crates/api/ui-tests/fieldlist.test.mjs`, `people.test.mjs` and `live.test.mjs` for the style of the model tests.

## Constraints

- No feed rule in the UI (ADR 0065).
- UI files are in `crates/api/ui/*.js` (Preact + htm). `node --check` misses template errors, so check a `.mjs` copy of each changed UI file.
- All four sheets of ADR 0168 item 4 follow items 1 to 3: **Find on Music Index**, **Link to an Artist or Label Page**, **Copy from a Feed** and **Find a Nostr profile**.
- The class `search-sheet` already sets a full height. Find out which sheet or width still moves, and fix that cause. Do not add a second mechanism.
- This task does not close the field list after Enter. Task 107 owns finding 1.

## Acceptance Criteria

Mechanical, with a browser drive: open each sheet of ADR 0168 item 4 at 390 px with touch. Read the top of the field before the list opens, after it opens, after two letters and after the results. The four values are the same.

Mechanical, with a Node test: the model gives no "no result" text while a search for the current text runs. It gives the text after a finished search with no result.

Visual, by the agent, with screenshots by `tools/drive-ui.mjs` at 1100 px and at 390 px with touch, in the dark and the light theme. Run `$edges()` and `$overflow()` with each screenshot:

- Each of the four sheets in each step of the drive above.
- The results scroll inside the sheet, below the field.

Human, by the operator: the same steps on the iPhone with the keyboard open.

## Test Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation Triggers

- A sheet has a form above its search, and a full height leaves no room for the results at 390 px with the keyboard.
- The browser moves the sheet when the keyboard opens, and CSS cannot hold the field still.

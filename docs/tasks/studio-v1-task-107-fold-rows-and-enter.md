# Studio V1 Task 107: Fold Rows And Enter In The Field List

Status: Ready.

Owner of the rules: [ADR 0171](../adr/0171-a-closed-fold-is-a-row-and-enter-closes-the-field-list.md), which applies musicindex ADR 0060 R-FOLD-1 and R-FOLD-2 and musicindex ADR 0052 R-QUERY-7 item 4.

## Goal

A closed fold looks like a row that the user can open. A search that Enter starts closes the field list, so the first result is visible.

## Files To Inspect

- `crates/api/ui/pickers.js`: `PickFold`, `foldStartsOpen`, `SearchResults`.
- Each `details class="disclosure"` in `crates/api/ui/app.js`, `editor.js` and `peoplepages.js`. Use `grep -n 'details class="disclosure' crates/api/ui/*.js`.
- `crates/api/ui/styles.css`: `.disclosure`, `.pick-fold`, `.pick-fold-title`, `.pick-fold-count`.
- `crates/api/ui/fieldlist.js` and the key handler of the field list in `pickers.js`.
- `../musicindex/docs/adr/0060-*.md` and `../musicindex/docs/adr/0052-*.md`. Read them only. Do not change them.

## Constraints

- No feed rule in the UI (ADR 0065).
- One fold style in `styles.css` (ADR 0171 item 3). Do not add a style for one fold.
- The rows of ADR 0171 item 2 keep their own form.
- `node --check` misses template errors. Check a `.mjs` copy of each changed UI file.

## Packets

- 107a: the fold header of ADR 0171 items 1 to 3.
- 107b: Enter closes the field list (ADR 0171 items 4 to 6).

## Acceptance Criteria

Mechanical:

- A Node test of `PickFold` gives a header with the title, the count and the open condition.
- A Node test of the field list model gives a closed list after Enter with no selected row. The next letter opens the list again.
- A browser drive reads each fold header of ADR 0171 item 1 on the People page, a feed page and the sheet "Add a Credit". The text has the body size and no capitals. The height is the target of musicindex ADR 0024 R-TARGET-1 or more.
- A browser drive types `artist:` and two letters in the search field of the Welcome page, then presses Enter. The list closes, and the first result is not covered.

Visual, by the agent: the folds closed and open, and the search after Enter. Take screenshots at 1100 px and at 390 px with touch, in the dark and the light theme, with `$edges()` and `$overflow()`.

Human, by the operator: the folds and a search on the iPhone.

## Test Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation Triggers

- A fold that is not in the list of ADR 0171 item 2 is also a list item that opens in place.
- The site or v4vmm uses a fold style that does not agree with musicindex ADR 0060.

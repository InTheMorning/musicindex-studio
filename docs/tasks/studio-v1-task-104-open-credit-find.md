# Studio V1 Task 104: Find On Music Index For An Open Credit

Status: Complete, with its screenshot visual check on 2026-11-11. The operator accepted the screenshots of **Replace**. The screenshots used a fixed answer in place of Music Index. The human test passed on 2026-11-12, in the release build of local `main`.

Owner of the rules: [ADR 0169](../adr/0169-an-open-credit-can-find-on-music-index.md), and [ADR 0145](../adr/0145-each-sheet-that-adds-a-value-has-one-layout.md) items 1, 3, 5 and 8. ADR 0169 changes item 8 of ADR 0145 for an open credit.

## Goal

An existing credit gets the row **Find on Music Index**. A selected page fills the empty fields, and offers **Replace** and **Add** for the other values.

## Packets

- 104a: the fill model: the four results of ADR 0169 items 2 to 4, in `crates/api/ui/people.js`.
- 104b: the row, **Replace** and **Add** on the open credit, and the pending state until the user confirms (items 1 and 5).

## Files To Inspect

- `crates/api/ui/editor.js`: `CreditSheet` and `CreditForm` (around lines 160 to 215), the credit rows with `EntryRow` (around line 129 and line 225 to 270), `NostrRow` and `BLANK_PERSON`.
- `crates/api/ui/pickers.js`: `PickerLayout` (line 186), `FindOneSheet` (line 250), `SearchResults` and `PickFillRow`.
- `crates/api/ui/people.js`: `creditChoices` (around line 1658), `withPersonValues`, `withPersonKey` and `fillPatch`.
- `crates/api/ui/peoplepages.js`: the credit names of a person, `'credit-names'` (around line 368), `fillPatch` and `fillSource`.
- `crates/api/ui-tests/people.test.mjs`: the tests of `creditChoices` and the fill.

## Constraints

- No feed rule in the UI (ADR 0065).
- UI files are in `crates/api/ui/*.js` (Preact + htm). `node --check` misses template errors, so check a `.mjs` copy of each changed UI file.
- The row uses `PickerLayout` and the heading "From Music Index" (ADR 0145 item 1). The search is the one search of ADR 0145 item 3.
- The sheet of the search follows ADR 0168. Task 103 is complete first.
- A field with a value keeps its value until the user selects **Replace**.
- **Cancel** keeps the credit as it was (ADR 0169 item 5).

## Acceptance Criteria

Mechanical, with Node tests of the fill model:

- Each empty field gets the value of the page.
- Each field with a value keeps it.
- Each different value gives a **Replace** choice, with the value of the page.
- A field that holds a list gives an **Add** choice. **Add** keeps the values that the list has.
- A value that agrees with the field gives no choice.
- **Cancel** gives back the credit as it was.

Visual, by the agent, with screenshots by `tools/drive-ui.mjs` at 1100 px and at 390 px with touch, in the dark and the light theme. Run `$edges()` and `$overflow()` with each screenshot:

- An open credit with the row **Find on Music Index**.
- The credit after a page is selected, with a fill, a **Replace** and an **Add**.
- The credit after **Cancel**.

Human, by the operator: an existing credit with no picture gets the picture of a real page.

## Test Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation Triggers

- ADR 0169 item 1 puts the row in the open credit row (`EntryRow`) of the Credits page. A credit has no field that holds a list, so **Replace** shows and **Add** does not (ADR 0169 item 4).
- Show the screenshots of the open credit row with **Replace** to the operator before the commit. ADR 0169 has no mockup.
- The credit names of the page of a person are not in ADR 0169. Do not change them.

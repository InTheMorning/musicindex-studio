# Studio v1 Task 062: Field Text On Touch

Decision: musicindex ADR 0024 R-TYPE-1 (shared rule), Body on Touch. musicindex ADR 0029 R-SCALE-1 item 3 keeps the Touch table in a web app.

Status: Complete, with its screenshot visual check on 2026-10-31.

## Goal

iOS Safari does not zoom the page when the user taps a field.

## Context

- iOS Safari zooms the page when a field with text smaller than 16 px gets the focus. The page stays zoomed after the keyboard closes.
- On 2026-10-31 the operator found the zoom on a phone. `$fieldText()` then found fields of 15 px in each sheet of the Recipients page. Each field had `font: inherit` from the body of 15 px.
- The Touch table of musicindex ADR 0024 R-TYPE-1 gives Body 17 px. The search field of musicindex.org has 16 px for the same cause. Its body stays 15 px. Thus this task changes only the fields, and the body of Studio stays 15 px.

## Files

Change: `crates/api/ui/styles.css`, `crates/api/ui-tests/field-text.test.mjs` (new), `tools/drive-ui.mjs`.

## Architecture

1. The last rule of `styles.css` is `:root :is(input, select, textarea) { font-size: 17px; }` in `@media (pointer: coarse)`. Its specificity is the same as `.field input`. It is the last rule, thus it replaces the text dimension of each rule before it.
2. `field-text.test.mjs` fails in three conditions:
   - The rule is not the last rule.
   - Its text is smaller than 16 px.
   - A rule that sets the font of a field has a higher specificity.
3. `tools/drive-ui.mjs` gets `$fieldText()` and `$overflow()`. `$fieldText()` gives each field that shows text smaller than 16 px. `$overflow()` gives each element that goes across the right edge of the window. The visual check of a UI packet runs the two helpers at 390 px with touch.

## Acceptance Criteria

Mechanical:

- The tests of Architecture item 2 pass. Each fails on its defect: text of 15 px, a rule after the touch rule, and the rule `.sheet .field input { font: inherit; }`.
- The four commit checks pass.

Visual, with `tools/drive-ui.mjs` at 390 px with touch:

- `$fieldText()` and `$overflow()` are empty on these pages: Recipients, a recipient page with its four operation sheets, File hosts, Releases, an album page and a track page.
- The fields with text of 17 px fit in their groups. `$edges()` is empty.

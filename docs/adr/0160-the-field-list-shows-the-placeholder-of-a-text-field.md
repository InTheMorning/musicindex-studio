# ADR 0160: The Field List Shows The Placeholder Of A Text Field

- Status: Accepted
- Date: 2026-11-08
- Owner: operator
- Accepted: 2026-11-08 by the operator, through the orchestrator session.
- Uses: musicindex ADR 0058 R-QUERY-11 and R-QUERY-12, musicindex ADR 0052 R-QUERY-7, [ADR 0154](0154-the-search-fields-have-a-field-list.md), [ADR 0151](0151-the-search-fields-read-the-shared-query-text.md).

## Context

musicindex ADR 0058 removes the names of artists and releases from the examples of the text fields. The crate `musicindex-search` at `e84bab5` gives the example `artist:"artist name"` and the placeholder `artist name` for `artist:`. It does the same for `title:`, `release:` and `playlist:`. On musicindex.org, a row of the field list shows that example in the placeholder style, and puts only the token into the field. The help of the site has the click with the selected placeholder, and the examples from the index.

Studio has the field list of ADR 0154, and it has no help.

## Decision

1. Studio uses the crate `musicindex-search` at `e84bab5`. The core gives the placeholder of each field row, or none.
2. A row of a text field shows its values text, then its example in the color of the placeholder of a search field (R-QUERY-11 item 2). A row of each other field shows its example as before.
3. A click or Enter on a row puts only the token into the field, as the row of the site does. The three apps keep one behavior of the field list (musicindex ADR 0049 and ADR 0052). A change of that behavior needs a musicindex record first.
4. Studio has no help surface. Thus R-QUERY-11 item 3 and R-QUERY-12 do not apply in Studio, and the field list has no rows of examples from the index.

## How This Decision Is Checked

Mechanical:

- `studio-core`: the row of `artist:` gives its example and its placeholder. It puts in only the token. The row of `kind:` has no placeholder.
- `crates/api/ui-tests/fieldlist.test.mjs`: the model gives the example of a text field as its placeholder. The text to put in is the token.

Visual, by the agent: the field list with a text field row, in the light and the dark theme. The agent examines it at 1280 px and at 390 px with touch.

Visual result: passed on 2026-11-08 ([visual checks of 2026-11-08](../reviews/2026-11-08-visual-checks.md)).

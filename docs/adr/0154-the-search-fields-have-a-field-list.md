# ADR 0154: The Search Fields Have A Field List

- Status: Accepted
- Date: 2026-11-07
- Owner: operator
- Accepted: 2026-11-07 by the operator, through the orchestrator session.
- Uses: musicindex ADR 0052 R-QUERY-7 and R-QUERY-8, musicindex ADR 0049, [ADR 0151](0151-the-search-fields-read-the-shared-query-text.md), [ADR 0016](0016-ui-without-a-build-step.md).

## Context

Studio has five search fields that read the query text of musicindex ADR 0049 (ADR 0151). None of them shows the fields that a person can type. musicindex ADR 0052 R-QUERY-7 gives a field list, and the crate `musicindex-search` gives its rows.

## Decision

1. Each of the five search fields of ADR 0151 has the field list of R-QUERY-7: "Find on Music Index", "Copy from a Feed on Music Index", the search of the first visit, the search of "Add tracks" and the search of the page of a person.
2. Studio has no Library in the sense of musicindex ADR 0049. The core asks the crate for the rows with `Library::Absent`, so the list has no row of `is:`, `playlist:`, `has:` or `in:` (R-QUERY-7 item 3).
3. The core gives the rows through the API route `GET /musicindex/fields`. The UI has no parser (ADR 0151) and loads no external script (ADR 0016). The route takes the text and the cursor in UTF-16 code units, and gives each range in UTF-16 code units. The UI puts the text of a row into the range that the core gives.
4. After `artist:`, the route `GET /musicindex/artist-names` gives the artist names of Music Index that start with the typed text. The UI asks for them after the pause of the search. `release:` gets no name rows (R-QUERY-7 item 2).
5. The search of the page of a person finds only artist and label feeds. Its list leaves out each field that gives no request for these feeds.
6. The list opens on a click in the field or a typed key, not on the focus alone (R-QUERY-7 item 4). A field that opens with a start name thus shows no list until the person acts.
7. The list opens with no selected row. Up and Down move the selected row. Tab or Enter puts the selected row into the text. Escape closes the list.
8. Enter with no selected row does the action that the field had before this record: it runs the search or sends the form. A typed key or a move of the cursor clears the selection. This is the rule of v4vmm ADR 0087 Decision 3.
9. The list follows the ARIA pattern of a combobox with a list box (R-QUERY-7 item 5).
10. The help "How to Search" of R-QUERY-10 is not part of this record.

## How This Decision Is Checked

Mechanical:

- `studio-core`: the rows for a text and a UTF-16 cursor, the ranges in UTF-16, no Library row, and the filter of item 5.
- `studio-api`: the route gives the rows, and needs the access token.
- `crates/api/ui-tests/`: the model puts a row into the text at its range. It gives the selection and the result of each key of items 7 and 8.

Visual, by the agent at 1280 px and at 390 px with touch:

- Each of the five fields with its list open.
- A field with a start name and no list.
- The error row of a field.

Visual result: passed on 2026-11-08 ([visual checks of 2026-11-08](../reviews/2026-11-08-visual-checks.md)).

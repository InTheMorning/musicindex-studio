# ADR 0151: The Search Fields Read The Shared Query Text

- Status: Accepted
- Date: 2026-11-07
- Owner: operator
- Accepted: 2026-11-07 by the operator, with five fields and a plain start text.
- Technical choices: item 6 is a choice of the planning model.
- Source: musicindex ADR 0049, section "What this record leaves open", item 1. `research_notes/search/shared-query-text.md` in `../musicindex-research` gives the research. `../musicindex-lib/docs/plans/search-move-phase-plan.md` Phase 4 gives the sequence.

## Context

musicindex ADR 0049 gives one query text for the three apps, and it leaves open which Studio search fields read that text.

Studio has these search fields that send a request to musicindex on 2026-11-07:

| Field | Location | Route | Start text |
|---|---|---|---|
| The search of each sheet that adds a value | `FindOneForm` of `crates/api/ui/pickers.js` | `/musicindex/search` | A form value in four sheets |
| The search of the recipient picker | `PickerSheet` of `crates/api/ui/values.js` | `/musicindex/search` | None |
| "Or search musicindex by name" of the first run | `FindFeeds` of `crates/api/ui/app.js` | `/search` | None |
| "Search musicindex" of the playlist sheet | `crates/api/ui/editor.js` | `/search` | None |
| "Find on musicindex" of the page of a person | `FindForm` of `crates/api/ui/peoplepages.js` | `/people/{id}/musicindex` | The stored name |

A person types the text in each field. Two of them can also start with a stored name or a form value, and that text can look like a field. musicindex ADR 0049 gives `in:flux` as an error.

## Decision

1. Each field of the table reads the query text of musicindex ADR 0049.
2. The rules of that text, and of a field that an app cannot apply, are in musicindex ADR 0049. This record does not restate them.
3. A start text is plain words until the person edits the field. A name with a colon does not become a field, and it gives no field error. When the person edits the field, the text reads as a query text. This also applies when the edit gives the start text again.
4. The UI holds no parser. The core reads the text with the crate `musicindex-search`, and it gives the UI the result (ADR 0002, ADR 0065, musicindex ADR 0049 R-QUERY-4).
5. A field error goes to the UI as a message, and no request goes to the index. Each field shows the message.
6. A query with no index request for a different cause gives a note and no rows. The note does not use the error style.
7. A response from the index that cannot decode gives an error, not an empty list.
8. A field keeps the relevance sequence of the index, and it sends no sort.

## Alternatives Considered

- A field with a stored start text stays plain for each text. Not chosen by the operator: a person who types `kind:single` there would get a search for those words.
- A start text reads as a query text at once. Not chosen by the operator: a stored name with a field word would open the sheet with a field error.
- Only the two picker fields read the text. Not chosen by the operator: a person types the text in each field.
- A parser in the UI. Not chosen: the core sends each index request, and ADR 0065 keeps each feed rule out of the UI.

## Consequences

- The same text can give a different request before and after an edit, when it is a start text.
- The search of the page of a person asks only for artist and label feeds. The crate sends no such request for a filter that those feeds cannot take, so that search can give no rows for a valid field.

## How This Decision Is Checked

Mechanical, in the tests of `studio-api` and `studio-core`:

- A `kind:single` text gives the request of musicindex ADR 0049 R-QUERY-6 at a fake index.
- A field error and a Library field each give their result at the route.
- A plain start text gives no field.
- A response that cannot decode gives an error.

Visual, by the operator:

- The recipient picker, with a result and with a field error.
- The first-run search with a field.

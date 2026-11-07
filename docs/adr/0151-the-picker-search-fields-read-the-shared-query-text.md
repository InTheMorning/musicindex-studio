# ADR 0151: The Picker Search Fields Read The Shared Query Text

- Status: Proposed
- Date: 2026-11-07
- Owner: operator
- Source: musicindex ADR 0049, section "What this record leaves open", item 1. `research_notes/search/shared-query-text.md` in `../musicindex-research` gives the recommendation. `../musicindex-lib/docs/plans/search-move-phase-plan.md` Phase 4 gives the sequence.

This record is a proposal. It makes no rule until the operator accepts it. No code holds it yet.

## Context

musicindex ADR 0049 gives one query text for the three apps, and it leaves open which Studio search fields read that text. Each Studio field now sends plain text.

Studio has these search fields that send a request to musicindex on 2026-11-07:

| Field | Location | Route |
|---|---|---|
| The search of each sheet that adds a value | `FindOneForm` of `crates/api/ui/pickers.js` | `/musicindex/search` |
| The search of the recipient picker | `PickerSheet` of `crates/api/ui/values.js` | `/musicindex/search` |
| "Or search musicindex by name" of the first run | `FindFeeds` of `crates/api/ui/app.js` | `/search` |
| "Search musicindex" of the playlist sheet | `crates/api/ui/editor.js` | `/search` |
| "Find on musicindex" of the page of a person | `crates/api/ui/peoplepages.js` | `/people/{id}/musicindex` |

The route `/musicindex/search` gives its text to two core functions: `lookup::search` and `MusicIndex::search`. Both send the text as `q`.

## Decision

1. The two fields that call `/musicindex/search` read the query text of musicindex ADR 0049.
2. The rules of that text, and of a field that an app cannot apply, are in musicindex ADR 0049. This record does not restate them.
3. The UI holds no parser. The core applies the text, and it gives the UI the result (ADR 0002, ADR 0065, musicindex ADR 0049 R-QUERY-4).
4. The other three fields of the table send plain text.

## Alternatives Considered

- Each field reads the text. The research names the first run and the page of a person as candidates. This proposal leaves them out, because the research recommends the two picker fields, and a later record can add a field.
- A parser in the UI. Not proposed: the core sends each index request, and ADR 0065 keeps each feed rule out of the UI.

## Consequences

- One text can have a different result in a picker and in a field of the first run, until a record adds that field. This is the condition that musicindex ADR 0049 calls one text with two meanings, for those fields of Studio.
- A user gets a field error in the two pickers only.

## Follow-Up

The crate `musicindex-search` does not exist on 2026-11-07. These parts of Phase 4 wait for it, and each needs its own task:

- The core takes the output of the shared crate.
- A field error goes to the UI as a message.
- A decode error gives an error, not an empty list. `MusicIndex::search` now gives an empty list for a response that it cannot decode.

The first item of Phase 4 is complete. On 2026-11-07, `routes::search` got the form with no lock, and `/musicindex/search` already had it. Tests of both routes hold the workspace lock during a search.

## How This Decision Is Checked

Phase 4 of the plan names these checks. They start with the follow-up work.

Mechanical:

- A `kind:single` text to the route gives the request of musicindex ADR 0049 R-QUERY-6 at a fake index.

Visual:

- The recipient picker, with a result and with a field error.

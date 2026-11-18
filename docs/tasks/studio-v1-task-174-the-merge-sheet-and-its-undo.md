# Studio v1 Task 174: The Merge Sheet And Its Undo

Decision: [ADR 0217](../adr/0217-a-new-artist-sees-the-result-of-a-write-the-payment-types-and-an-undo-of-a-merge.md) items 12 to 17. [ADR 0203](../adr/0203-people.md) items 31, 32 and 52.

Status: Open.

## Problem

The merge sheet selects the first other person at the start (`peoplepages.js:1278`). Rows of a credit name can have the same text, because a row tells no feed (`people.js:2199`). A merge deletes the other person, and no step undoes it.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Merge in the core | extend | crates/core/src/people.rs | `merge_people` near line 3311 does the merge in one transaction. It also writes the undo data in that transaction. |
| Credit name uses | extend | crates/core/src/people.rs | `credit_name_uses` near line 2535 gives the feed count and the roles. Add the title of the feed when the count is 1. |
| Collection schema | extend | crates/core/src/collection.rs | `MIGRATIONS` and `SCHEMA_VERSION` near line 763. Add one step for the undo table. |
| Person routes | extend | crates/api/src/people.rs | `merge_person` near line 555. Add the undo route. `person_json` near line 63 gives the credit name uses for the merge sheet. |
| Merge sheet | extend | crates/api/ui/peoplepages.js | `MergeForm` near line 1272. |
| Merge rows | extend | crates/api/ui/people.js | `mergeLinks` near line 2194. |
| Notice | reuse | crates/api/ui/app.js | The notice form near line 4899, with the role `status`. |

## Changes

1. Core: a table `person_merge_undo` keeps one row: the ID of the kept person, the ID of the other person, and the data that puts back the two persons. The data holds each row of the two persons in the tables that `merge_people` changes or deletes, as they were before the merge. `merge_people` replaces the row in its transaction.
2. Core: `undo_merge(collection, kept_id)` puts back the other person with its ID, and each row of the two persons, in one transaction. It refuses when no undo row is there, or when its kept ID differs.
3. Core: each function that changes a person deletes the undo row when the person is the kept person or the other person of that row. Find these functions in `people.rs` and in `change_set.rs`. A change of a different person keeps the row.
4. API: `POST /people/{id}/merge/undo`. A refusal gives the text "Studio cannot undo this merge, because a person changed after it."
5. API: the merge sheet gets the credit name uses of the selected person.
6. UI: the menu "Person to merge" starts on "Select a person". The links and the result show after a selection. **Merge the People** is disabled until a selection.
7. UI: a row of a credit name has the line "Credit name · {feed title} · {role}" for one feed, or "Credit name · {n} feeds" for more feeds. A name with no feed has "Credit name".
8. UI: after a merge, the page of the person shows "{other name} is merged into {name}." with **Undo**. The notice stays until the user leaves the page. **Undo** calls the route and then loads the page again.

## Files

- Change: `crates/core/src/people.rs`, `crates/core/src/collection.rs`, `crates/core/src/change_set.rs` if item 3 needs it, `crates/api/src/people.rs`, `crates/api/src/lib.rs`, `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`.
- Change tests: the tests in `crates/core/src/people.rs`, `crates/api/ui-tests/people.test.mjs`, and an API test of the undo route.

## Checks

Mechanical:

- Core: merge two persons with wallets, keys, credit names, values, pages, members, change sets and dismissals, then undo. Each row of the two persons is the same as before the merge, with the same IDs.
- Core: after a merge, a change of one of the two persons makes the undo refuse. A change of a third person keeps the undo.
- Core: a second merge replaces the undo data of the first merge.
- Core: an upgrade from the schema before this task opens a collection with the new table.
- UI: the merge form starts with no selected person, and the line of a credit name of change 7.
- `tools/gate.sh` passes.

Visual, by the main session with screenshots (`.studio-dev/journeys/j07.json`, data `base`):

- The merge sheet before and after a selection, at 1280 px and 390 px.
- The notice with **Undo** after a merge, and the two persons after the undo.

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When the ADR does not answer a question, stop and report.

# Studio v1 Task 112: The User Controls Each Collection

Decision: [ADR 0176](../adr/0176-the-user-controls-each-collection.md).

Status: Completed on 2026-11-11. The four commit checks are green. The visual check ran at 1280 px and 390 px. The human gate passed on 2026-11-12, in the release build of local `main`.

## Goal

The user can rename, replace and delete each collection that is not open, "Main collection" included. The export is in the section "Collections", adjacent to the import.

## Constraints

- Each rule is in the core or the API. The UI model gives the actions, and the UI shows them (ADR 0065, ADR 0042).
- A delete or a replace of the open collection is refused in the core, not only hidden in the UI.
- A replace reads the file into a new folder first. A read that is not good changes nothing.
- No test sends a request outside the computer (ADR 0045).
- Use only the current tokens and the current classes. A new marker or a new color needs a shared musicindex ADR first.

## Files To Read First

- ADR 0176, ADR 0149 and ADR 0107 items 3 to 5 and 9 to 13.
- `crates/core/src/collections.rs`: `delete`, `prepare_with_steps`, `checked_name`.
- `crates/core/src/account_archive.rs`: `write_archive`, `check_archive`.
- `crates/api/src/collections.rs` and `crates/api/src/import.rs`.
- `crates/api/ui/collectionlist.js`, `crates/api/ui/pages.js` and `CollectionsPanel` and `MovePanel` in `crates/api/ui/app.js`.

## Packets

Do the packets in this sequence. Each packet is one change, with the four commit checks.

### 112a: The Core And The API

ADR 0176 items 1 to 9.

- Remove the protection of `main` in `delete`. Refuse a delete of the open collection.
- A start with no `main` entry in the list opens the open entry and makes no `collections/main/`.
- Rename changes the list file only.
- Replace: read the file into a new folder with the import checks, then write the list file with the new folder, then remove the previous folder (ADR 0176 item 8). Refuse the open collection.
- The routes of rename, replace and an export of a collection that is not open.

### 112b: The Data Tab

ADR 0176 items 2, 3, 6, 7, 10, 11 and 12.

- The UI model gives the actions of each row. Add a Node test.
- The rows, the sheets of rename, replace and delete, and "Export This Collection…" in "Collections".
- "Move Studio" shows only in a shell with no list of collections.
- The mark of the left panel shows when the list has more than one collection.

## Acceptance Criteria

Mechanical:

- The mechanical criteria of ADR 0176.
- The four commit checks.

Visual, by the planning model:

- The visual criteria of ADR 0176.

Human, by the operator. This gate stays open until it runs:

- The human criteria of ADR 0176.

## Stop And Ask When

- The export of a collection that is not open needs its vault to be open.

## Final Report

For each packet, give the diff, the results of the four commit checks and the screenshots of packet 112b. Report each gate that did not run as open.

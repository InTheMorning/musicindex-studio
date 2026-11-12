# Studio v1 Task 113: The Files Page Has One Way To Move

Decision: [ADR 0177](../adr/0177-the-files-page-has-one-way-to-move.md).

Status: Completed on 2026-11-11. The four commit checks are green. The visual check ran at 1280 px and 390 px. The human gate passed on 2026-11-12, with a move of each value of "What moves" between two real file hosts.

## Goal

The Files page of a release has the section "File hosts" with text rows and "Move This Release…" as the only move. The section "On this computer" has one library row.

## Constraints

- Each rule is in the core or the API. The UI model gives the rows and the values, and the UI shows them (ADR 0065, ADR 0042).
- Use only the current tokens and the current classes. A new marker or a new color needs a shared musicindex ADR first.
- No test sends a request outside the computer (ADR 0045).

## Files To Read First

- ADR 0177, ADR 0148 items 1 to 6 and 15, ADR 0167 and ADR 0036 item 8.
- The Files page in `crates/api/ui/editor.js`, with `MoveReleaseSheet`, `setFeedHost` and `setDefaultHost`.
- `libraryFolderRow` in `crates/api/ui/feedfacts.js` and its test `crates/api/ui-tests/libraryfolder.test.mjs`.
- The route of the release move in `crates/api/src/` and its plan in `crates/core/src/moves.rs`.

## Packets

### 113a: What Moves

ADR 0177 items 3 and 4.

- The plan of a release move takes the value of "What moves". Add an API test for each value.
- A move of the release files changes the default file host of the release.

### 113b: The Page

ADR 0177 items 1, 2 and 5 to 10.

- Remove the two select menus. The UI model gives the two text rows and the library row. Add a Node test.
- A start for the library row is in the scratchpad patch of the planning model. At 390 px, its buttons covered the path in the visual check. Item 9 corrects that.
- Make the screenshots of the visual criteria. Examine each screenshot. `$edges()` and `$overflow()` did not find the defect at 390 px.

## Acceptance Criteria

Mechanical:

- The mechanical criteria of ADR 0177.
- The four commit checks.

Visual, by the planning model:

- The visual criteria of ADR 0177.

Human, by the operator. This gate stays open until it runs:

- The human criteria of ADR 0177.

## Stop And Ask When

- A value of "What moves" needs a plan that ADR 0148 does not give.

## Final Report

For each packet, give the diff, the results of the four commit checks and the screenshots. Report each gate that did not run as open.

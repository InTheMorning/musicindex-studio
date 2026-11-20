# Studio v1 Task 200: Delete A Leftover Copy Of A Feed

Decision: [ADR 0228](../adr/0228-undo-in-the-bar-a-notice-that-stays-and-the-rc28-check-findings.md) items 20 to 22.

Status: Done.

## Problem

Finding 9 of the [rc.28 human check](../reviews/2026-11-20-rc28-human-check.md). Music Index holds the GUID of the feed at a different address. `read_removal` reads the GUID only and gives `NotYet`, so the delete of the leftover file never becomes available.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Removal read | extend | crates/core/src/feed_retire.rs | `read_removal` near line 136 ignores the `IndexFeed` of `IndexPresence::Held`. `retire_state` near line 174 sets `removal_done`. `ensure_index_removed` near line 153 refuses the delete draft. |
| Index record | reuse | crates/core/src/musicindex.rs | `IndexFeed` near line 535. Its field `url` is the address. |
| Address comparison | reuse | crates/core/src/feed_check.rs | `same_but_spelling` near line 555. The index check uses it near feed_check/index.rs:99. |
| Move source | reuse | crates/core/src/outside.rs | `index_is_move_source` near line 122. |
| Routes | extend | crates/api/src/feed_retire.rs | The retire state near line 22 and the delete draft near line 69. Each calls `read_removal`. |
| Sheet | extend | crates/api/ui/feeddelete.js | `DELETE_CHOICES` near line 5 and `deleteChoices` near line 30. |

## Changes

1. `read_removal` also takes the public address of this file and the last move of the feed. When Music Index holds the GUID at an address that is not equal and not `same_but_spelling`, and the address is not the move source of `index_is_move_source`, the answer is a new variant, for example `LeftoverCopy { index_url }`.
2. `retire_state` gives `removal_done: true` for that variant. `ensure_index_removed` accepts it. Each other answer is refused as before.
3. The sheet shows "Music Index reads this feed from {address}. This file is a leftover copy." above the rows.

## Checks

Mechanical:

- Core tests: a GUID at a different address gives the new variant and `removal_done: true`. The same address, a spelling difference and the source address of a Studio move each give `NotYet`.
- API test: the delete draft passes for the leftover copy and refuses the other cases.
- UI test: the sheet text of change 3.
- `bash tools/gate.sh` passes.

Visual (the main session): none. The human check deletes the file of finding 9 on the real file host.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.

## Result

- The core gives `IndexRemoval::LeftoverCopy` when the record of Music Index has an address that is not the public address of this file. The core compares the two addresses after `plain_address` and `same_but_spelling`. The source address of the last move by Studio gives `NotYet` (item 21).
- `retire_state` gives `removal_done: true` for a leftover copy, and `ensure_index_removed` accepts it. Each other answer stays refused.
- The first sheet and step 2 of the retire show "Music Index reads this feed from {address}. This file is a leftover copy."
- Core tests, route tests and UI tests cover a different address, the same address and the source address of a move. `bash tools/gate.sh` passed.
- No walk shows the sheet with a leftover copy. The walk needs an index stub that gives a record at a different address. The human check of ADR 0228 (the leftover copy of finding 9 on the real file host) stays open.

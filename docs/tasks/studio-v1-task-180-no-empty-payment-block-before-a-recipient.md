# Studio v1 Task 180: No Empty Payment Block Before A Recipient

Decision: none. This task corrects a defect.

Status: Open.

Operator choice of 2026-11-19: correct the defect first, and then make the mockups for O8 and O6. The operator agreed to this order in the review of the open observations.

## Problem

On the Value Routes page of a feed with no recipients, **Add recipients** puts an empty payment block in the draft (`values.js:539`). **Add a Recipient** on the Check page and on the release checklist does the same (`values.js:529`). The toolbar then shows "Saved, not written" before the user saves a recipient. When the user closes the form with no recipient, the empty block can stay in the draft (inference, not tested).

On a track page, **Give This Track Its Own Recipients** puts an empty block in the draft when the album has no recipients (`values.js:553`).

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Album payments | extend | crates/api/ui/values.js | `AlbumPayments` near line 527. Show the form of a new recipient without a change of the draft. |
| Recipient list and form | reuse | crates/api/ui/values.js | `Recipients` near line 133 and its `save`. The first save gives the block with its first recipient. |
| Payment block | reuse | crates/api/ui/values.js | `Payments` near line 512 and `BLANK_BLOCK` near line 503. |
| Track payments | extend | crates/api/ui/values.js | `TrackPayments` near line 550. The same correction for a track with no album recipients. |

## Changes

1. UI: **Add recipients** and the `openAdd` path do not change the draft. They show the payment group with a block that is not in the model, and open the form "Add a Recipient". Use `BLANK_BLOCK` for that block.
2. UI: **Add the Recipient** writes the block with its first recipient to the draft in one change.
3. UI: when the user closes the form with no recipient, the draft does not change, and the page shows "This feed has no recipients." again.
4. UI: on a track page, when the album has no recipients, **Give This Track Its Own Recipients** follows changes 1 to 3. When the album has recipients, the button does not change.
5. Do not change the core. Do not change what Studio writes for a block that the user empties with **Remove**. Report what the core does with a block with no recipients, with the file and line.

## Files

- Change: `crates/api/ui/values.js`.
- Change tests: `crates/api/ui-tests/values.test.mjs`, or the UI test file that already tests `AlbumPayments`. Report which file.

## Checks

Mechanical:

- UI: a test shows that the open path of change 1 calls no `update` before a save.
- UI: a test shows that the first save gives one block with one recipient.
- UI: the same two tests for change 4.
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.
- Check each changed `.js` file: copy it to a `.mjs` file and run `tools/node --check` on the copy.

Visual, by the main session with screenshots at 1280 px and 390 px:

- **Add recipients** on a feed with no recipients: the toolbar keeps "No changes" while the form is open.
- Close the form: the toolbar keeps "No changes".
- Save a recipient: the toolbar shows "Saved, not written", and the row shows.

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not write a new interface text.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.

# Studio v1 Task 182: Review Shows The Payment Details And No Placeholder

Decision: none. This task changes the words of two rows of the Simplified view of Review (ADR 0114 items 1 to 5). It is a choice on one module.

Status: Done.

Operator choice of 2026-11-19, through the questionnaire, from drawings: Review shows one row for each recipient of a new or removed payment block. The row has the name, the address, the type and the percentage. The operator selected "One line per recipient".

Operator choice of 2026-11-19, through the questionnaire, from drawings: the row of a placeholder GUID shows "Set when the feed has a public address", not the placeholder. The Detailed view keeps the XML line. The operator selected "Plain words".

## Problem

Observation O6 of the [analysis of two journeys](../research/2026-11-18-user-journeys/analysis.md).

- When the user adds the first recipient, the Simplified view shows the group "Payments" and one row "Payments / Added" (screenshot `docs/research/2026-11-18-user-journeys/shots/j06-06-review.png`). The artist cannot check the address or the share before the write. Cause: `brief` in `review.js` finds no name in a payment block, because the block has its recipients in `recipients`.
- For a feed with no public address, the row "Podcast GUID" shows `none → pending-60384c23-…` (screenshot `j02-05-review.png`). The placeholder is an internal value (ADR 0122 item 5, `has_placeholder_guid` in `crates/core/src/pipeline.rs:1021`).

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Row of one change | extend | crates/api/ui/review.js | `changeRow` near line 238. Give the rows of a payment block, and the placeholder text. |
| Rows of the field view | extend | crates/api/ui/review.js | `fieldRows` near line 293. One change of a whole block gives one row for each recipient. |
| Percentages of a block | reuse | crates/api/ui/recipients.js | `percentages` near line 34, `percentText` near line 226, `isFee` near line 28. |
| Type words | reuse | crates/api/ui/recipients.js | `WALLET_TYPES` near line 453. |
| Short node key | reuse | crates/api/ui/nostrkey.js | `shortKey`. `values.js` near line 191 uses it for a node address. |
| Field row view | reuse | crates/api/ui/editor.js | `FieldRow` near line 1544. No change is necessary when each recipient is one row. |

## Changes

1. UI: a change that adds or removes a whole payment block gives one row for each recipient of the block. A payment block is a value in the group `values` (and `live_value`, when its value has the same form) with an array `recipients`.
   - Label: `Added: {name}` or `Removed: {name}`. A recipient with no name uses "No name".
   - Text: `{address} · {type} · {percentage}`. The address of a node is the short key (`shortKey`), as on the Value Routes page. The type is the menu text of `WALLET_TYPES`. The percentage is `percentText` of `percentages` of the recipients of the block. A fee recipient adds ` fee` after the percentage, as on the Value Routes page.
   - A block with no recipients keeps the row of today.
2. UI: a change that adds or removes one recipient in a block that stays (the change has `entry`) gets the text `{verb}: {address} · {type} · share {split}`. The percentage is not known from one change, so the row gives the share. The label stays the name of the recipient.
3. UI: a string value at the key `podcast_guid` or `feed_guid` that starts with `pending-` is a placeholder. When the new value of a change is a placeholder, the text of the row is "Set when the feed has a public address". When the old value is a placeholder, it shows as "none". Do not change the Detailed view.
4. Do not change the core or the diff route.

## Files

- Change: `crates/api/ui/review.js`.
- Change tests: `crates/api/ui-tests/review.test.mjs`.

## Checks

Mechanical:

- UI: `fieldRows` for an added block with two recipients (one LNURL address, one node address with a share of 10 of 100) gives two rows with the labels, addresses, types and percentages of change 1.
- UI: a removed block gives `Removed: {name}` rows. A block with no recipients gives the row of today.
- UI: an added recipient in a block that stays gives the text of change 2.
- UI: `changeRow` for `podcast_guid` from none to `pending-…` gives "Set when the feed has a public address". From `pending-…` to a real GUID, it gives `none → {guid}`.
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.
- Check each changed `.js` file: copy it to a `.mjs` file and run `tools/node --check` on the copy.

Visual, by the main session with screenshots at 1280 px and 390 px:

- Review after the first recipient of "Loose Single": the group "Payments" with one row for the recipient.
- Review of a new release with no public address: the row "Podcast GUID" with the text of change 3.

## Result

Done on 2026-11-19. A ChatGPT run made the change. The main session corrected it:

- The run found a payment block by the group kind `field`. The core gives `values` the kind `list` (`crates/core/src/review.rs:595`), and a track the kind `track`. So a real Review showed no change. The code now finds a block and a recipient by the form of the value.
- The placeholder row read "none → Set when …". It now reads only the sentence of change 3.
- One `recipientText` replaces two copies of the address and type code.
- A recipient with no name has the title of its row on the Value Routes page (`recipientTitle`), not "No name".

Results:

- Mechanical: 857 UI tests and `bash tools/gate.sh` pass. The syntax check of `review.js` passes.
- Visual, from screenshots at 1280 px and 390 px (walk `v182a.json`): the group "Payments" shows "Added: artist@getalby.com" and "artist@getalby.com · LNURL Address · 100.0 %".
- Visual, from screenshots at 1280 px and 390 px (walk `v182b.json`, a new release with no public address): the row "Podcast GUID" shows "Set when the feed has a public address".

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Use the exact texts of this task. Do not write a new interface text.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.

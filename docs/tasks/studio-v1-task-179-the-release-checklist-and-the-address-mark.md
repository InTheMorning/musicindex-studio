# Studio v1 Task 179: The Release Checklist And The Address Mark

Decision: [ADR 0219](../adr/0219-a-release-checklist-and-a-check-of-the-address-type.md) items 6 to 8 and 10.

Status: Open. It needs task 177.

## Problem

The findings of task 177 show only on the Check tab. A new artist sees the release page first.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Release page header | extend | crates/api/ui/app.js | `EditableFeed` near line 2243. The checklist goes in `feed-head`, after `OutsideNotices`. `check` comes from `useFeedCheck`. |
| Findings of the draft | reuse | crates/api/ui/app.js | `useFeedCheck` near line 2064. The checklist reads its findings. It calculates no condition again. |
| Finding buttons | reuse | crates/api/ui/feedcheck.js | `checkRows` and the actions of task 177. The checklist buttons use the same actions as the Check page. |
| Checklist view | extend | crates/api/ui/feedcheck.js | Add one exported function that gives the checklist view from the findings: the title, the four rows, the marks and the buttons. |
| Recipient rows | extend | crates/api/ui/values.js | The rows of the payment page. Add the warning mark of ADR 0219 item 10. |
| Warning style | reuse | crates/api/ui/styles.css | Use the tokens and the classes of an existing notice and warning mark. Add a rule only when no class fits. |

## Changes

1. UI: a function gives the checklist view of ADR 0219 items 6 and 7 from the findings. With none of the four kinds, it gives no view. Else it gives the title "This release needs {n} things" or "This release needs 1 thing". It also gives the rows "Tracks", "Cover image", "Audio file of each track" and "Payment recipient", in this sequence. Each row has a check mark, or a cross and a button. With more than one `no_audio_file`, the row reads "{n} tracks have no audio file", and **Open the Track** opens the first one.
2. UI: the release page shows the checklist under its header (item 6). The buttons do what the same buttons on the Check page do.
3. UI: a recipient row on the payment page shows a warning mark when its address has a `wrong_address_type` finding (item 10). The mark has an accessible name, for example "Wrong address type".

## Files

- Change: `crates/api/ui/feedcheck.js`, `crates/api/ui/app.js`, `crates/api/ui/values.js`, and `crates/api/ui/styles.css` only if needed.
- Change tests: `crates/api/ui-tests/feedcheck.test.mjs`.

## Checks

Mechanical:

- UI: the checklist view gives no view with none of the four kinds.
- UI: the title for 1 item and for 2 items, and the four rows in sequence with their marks and buttons.
- UI: the row text for two tracks with no audio file.
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.
- Check each changed `.js` file: copy it to a `.mjs` file and run `tools/node --check` on the copy.

Visual, by the main session with screenshots:

- The release page with the checklist at 1280 px and 390 px, and each button.
- The payment page with the mark.

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Use the exact texts of ADR 0219. Do not write a new interface text.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When the ADR does not answer a question, stop and report.

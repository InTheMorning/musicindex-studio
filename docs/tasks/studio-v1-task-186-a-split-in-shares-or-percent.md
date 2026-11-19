# Studio v1 Task 186: A Split In Shares Or Percent

Decision: [ADR 0223](../adr/0223-a-split-in-shares-or-percent-a-release-search-and-one-tap-on-a-phone.md) items 1 to 4.

Status: Done.

## Problem

Observation O16 of the [user journeys](../research/2026-11-18-user-journeys/observations.md). After "Add the Recipient", the split field is empty and the bar shows 100.0 %. The recipient form has only shares, and the user must calculate a percentage.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Percentages of a block | reuse | crates/api/ui/recipients.js | `percentages` near line 34 and `percentText` near line 226. |
| Scale of splits | new | crates/api/ui/recipients.js | Searched crates/api/ui and ~/build/musicindex-lib. The core has `scale` in `crates/core/src/change_set.rs` near line 478, but the UI cannot call Rust for each field change. Write `scaleSplits` in JS with the same rule. Test it with the same cases. |
| Text field with hint | reuse | crates/api/ui/editor.js | `TextField` near line 74. |
| Field and select menu on one line | reuse | crates/api/ui/styles.css | The class `inline`, as in `app.js` near line 1636. |
| Recipient form | extend | crates/api/ui/values.js | `Recipients` near line 133. The split field is near line 194. |

## Changes

1. `recipients.js`: add `scaleSplits(splits, total)`. It is the rule of the core function `scale`: each split becomes `split * total / sum` with integer division, and the remainder goes to the first biggest split. A sum of zero gives the splits with no change.
2. `recipients.js`: add `setPercent(recipients, index, percent)`. It gives the recipient at `index` the split `percent`. It scales the splits of each other recipient, each fee included, to `100 - percent` with `scaleSplits`. It returns new recipient objects, and it keeps each other field. A split that is not a number stops the change: the function returns `null`.
3. `values.js`: the open recipient has the field "Split" and, on the same line, a select menu with the label "Split as" and the options "Shares" and "Percent". The menu starts at "Shares". Keep the choice in the state of `Recipients`, by `_cid`. Do not save it.
4. `values.js`: below the field, one line with the class `hint` gives the other value:
    - "Shares": "{percentText} of this block", for example "25.0 % of this block".
    - "Percent": "{shares} shares of {total}", for example "25 shares of 100".
    Keep the hint of ADR 0222 item 7, "A recipient gets its shares divided by the total shares of the block.", as the `hint` of the field.
5. `values.js`: with "Percent", the field shows the percentage of the recipient, rounded to a whole number. The user types a whole number from 1 to 99. On the `change` event of the field (the focus leaves, or Enter), call `setPercent` and `onChange`. A value that is not a whole number from 1 to 99 shows the field error "Type a whole number from 1 to 99." and changes nothing.
6. `values.js`: an empty split shows the line "Give a split." with the class of a field error, in the place of the line of change 4. The `Meter` and the share text of the row show "—" for that recipient, also with one recipient. Use `null` for the percent of a recipient with an empty split, in the display only. Do not change `percentages`.
7. `values.js`: with one recipient that has a split, "Percent" shows 100, the field is read-only, and the line is "One recipient gets each payment.".
8. Do not change the core, the API or the feed format. The feed always holds shares.

## Files

- Change: `crates/api/ui/recipients.js`, `crates/api/ui/values.js`, `crates/api/ui/styles.css` only when the class `inline` does not fit.
- New test: `crates/api/ui-tests/split-mode.test.mjs`.

## Checks

Mechanical:

- `split-mode.test.mjs`:
    - `scaleSplits` gives these results, the same as the core `scale`: `[50, 50]` to 75 gives `[38, 37]`. `[1, 2, 3]` to 60 gives `[10, 20, 30]`. `[0, 0]` to 10 gives `[0, 0]`. `[3, 3, 3]` to 10 gives `[4, 3, 3]`.
    - `setPercent` on 3 recipients with splits 50, 30 and 20 and the percent 25 for the first: the first split is 25, the total is 100, and `percentages` gives 25 for the first.
    - `setPercent` with a fee: the fee scales also, and the total is 100.
    - `setPercent` with a split "abc" returns `null`.
    - The source of `values.js` has "Split as", "of this block", "Give a split." and "One recipient gets each payment.".
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.
- Check each changed `.js` file: copy it to a `.mjs` file and run `tools/node --check` on the copy.

Visual, by the main session with screenshots at 1280 px and 390 px:

- An open recipient with "Shares", with "Percent", and with an empty split.

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Use the exact texts of this task. Do not write a new interface text.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.

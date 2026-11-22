# Studio v1 Task 235: The Review Panels For Addresses And People

Decision: [ADR 0245](../adr/0245-review-repairs-short-addresses-and-offers-the-people-values.md) items 6 to 9, 11 to 13. Run it after tasks 233 and 234.

Status: Done 2026-11-22. The gate passed (2061 Rust tests, 987 Node tests). The main session checked three scenes with screenshots:

- From the base data, a move of 3 artists to "Doerfel-Verse Hosting" gives the panel "1 feed has addresses that are not full" with "Bloodshot Lies - The Album". The write is disabled and "Repair these addresses first." shows. After **Add https:// in 1 Feed**, the panel goes and the write is enabled.
- With the person "The Doerfels", the same review gives "8 feeds differ from People" and **Use The Doerfels's Values in 8 Feeds**. The sheet "Correct the Feeds: The Doerfels" has **Apply to the Drafts**. After the apply, the panels go and the diff shows the corrections.
- The person page refuses the website `www.doerfelverse.com`. The field shows "www.doerfelverse.com has no https://. Studio cannot save it." and **Add https://**. After the repair, the save works.

The fit checks at 390 px and 761 px are clean, and the console has no error.

Corrections of the main session:

- The address panel shows the button below the title. Each feed shows its title one time, and then its rows.
- The People panel shows each feed as a closed row with the summary of the check. The row holds the differences, as in the check of a person.
- The sheet reads the person from the answer of the API directly.
- **Apply to the Drafts** is on the footer of the correct form, not on the footer of the operation form.
- The field error of the person page uses the words of the feed finding, not the field name of the API. The status line gives "Studio could not save. See Websites." and not a second copy of the error.

Open points:

- An address with no `short_address` finding still shows its model path in the list of the review of one feed.
- The correct sheet opens as a dialog in the review dialog.
- The write is not disabled while the findings load.
- The button label for a name that ends in "s" is "The Doerfels's".

## Problem

The review of one feed and the review of a change set show no repair for an address that is not full. They show no difference from People. The person page saves an address that is not full with no message.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Refusal panel | extend | crates/api/ui/editor.js | The panel of ADR 0218 is near lines 2333 to 2338. The write line is near 2398, and the disabled write button is near 2407. They give the pattern of a panel that stops the write. |
| Refusal texts | extend | crates/api/ui/feedcheck.js | `pageRefusalView` gives the texts of the panel. The view model of the two new panels goes in the same file. |
| Change set review | extend | crates/api/ui/app.js | `ChangeSetPage` near line 3347 loads each diff and writes each feed. |
| Correct sheet | extend | crates/api/ui/peoplepages.js | `CorrectSheet` and `CorrectForm` with `feedIds` and `onlyIds`. |
| Person fields | extend | crates/api/ui/people.js | The field list near line 882 and the save body. |

## Changes

1. Read ADR 0245 and ADR 0218 items 1 to 5 first. Then read the panel of `editor.js`, `pageRefusalView`, `ChangeSetPage`, `CorrectSheet`, `CorrectForm` and the routes of tasks 233 and 234.
2. In `feedcheck.js`, add two view models:
   - The address panel (item 6) reads the `short_address` findings of the feeds of a review. It gives the title, the rows and the button label. A row has the feed title, the place, the address and the new address. It also gives the write line "Repair these addresses first."
   - The People panel (item 11) reads the answer of the People route of task 234. It gives the title and a group for each person with its findings. The button label is **Use {name}'s Values in {n} Feeds** or **Use {name}'s Values in 1 Feed**.
3. In the review of one feed (`editor.js`) and in `ChangeSetPage` (`app.js`), show the address panel at the top. Its button applies the repair of task 233 to each listed finding, then the review reads the diffs again. While the panel shows, the write button is disabled and the write line shows, as in the panel of ADR 0218.
4. Show the People panel under the address panel. Its button opens `CorrectSheet` for that person, with only those feeds. In this use, the primary button reads **Apply to the Drafts** and calls the route of task 234 step 5. Then the user goes back to the same review, which reads the diffs again. The People panel does not disable the write button.
5. Item 8: a refused write shows the places that the `bad_address` error gives, not model paths. An address with no `short_address` finding shows its place and **Open the {release word}** in the review.
6. Item 9: the person page shows the error of the save under the field, with the error color. When the error gives a repair text, the field also has the button **Add https://**, which puts the repair text in the field.
7. Tests in `crates/api/ui-tests/`: the two view models of step 2 with one feed and with two feeds, including each text of ADR 0245 items 6, 7 and 11. The save error of step 6 gives the button text and the repair text.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0245 and its item.
- The UI calculates no address rule and no People difference. It reads the findings of the core (ADR 0065, ADR 0203 item 70).
- Use the tokens and the components that the review uses now. Do not add a color or a dimension.
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.
- If a step needs a design decision that this task does not give, stop and report it.

## Acceptance Criteria

Mechanical:

- The tests of step 7 pass.
- `tools/gate.sh` passes.

Visual (the main session checks with screenshots of the tutorial test data):

- The move review shows the address panel with "Bloodshot Lies - The Album", and **Write 37 Feeds** is disabled. After **Add https:// in 1 Feed**, the panel goes and the write button works.
- With a person "The Doerfels" with the information address `https://www.doerfelverse.com/`, the move review shows the People panel. After **Apply to the Drafts**, the panel goes.
- The person page refuses `www.doerfelverse.com` and offers **Add https://**.

## Report

Give the changed files, the new tests, the result of `tools/gate.sh` and `git status --short`.

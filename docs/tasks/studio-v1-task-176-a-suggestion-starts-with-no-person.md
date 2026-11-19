# Studio v1 Task 176: A Suggestion Starts With No Person

Decision: [ADR 0218](../adr/0218-a-page-link-in-review-and-a-suggestion-starts-with-no-person.md) items 10 to 14. [ADR 0203](../adr/0203-people.md) items 38 to 42.

Status: Open.

## Problem

The sheet "Check: {name}" starts on "A new person" when no person has the name of the suggestion (`people.js:2074`). The sheet "Add to a Person: {name}" of a payee always starts on "A new person" (`people.js:2103`). The fastest path makes an extra person.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Check sheet state | extend | crates/api/ui/people.js | `checkSheetStart`, `checkSheetChoices`, `checkSheetPerson`, `checkSheetLabel` and `checkSheetReady` near line 2074. |
| Payee sheet state | extend | crates/api/ui/people.js | `walletToPersonStart`, `walletToPersonLabel` and `walletToPersonReady` near line 2103. |
| Check sheet | extend | crates/api/ui/peoplepages.js | `CheckSuggestionForm` near line 305. |
| Payee sheet | extend | crates/api/ui/peoplepages.js | `WalletPersonForm` near line 1639. |
| Suggestions fold | extend | crates/api/ui/peoplepages.js | The note of `people-suggestions` near line 211. |
| Select menu | reuse | crates/api/ui/app.js | `ChoiceSelect` near line 1313 with its `placeholder`, as the merge sheet uses it. |

## Changes

1. UI: a new value for "Select a person" (for example the empty text). `checkSheetStart` gives it when the suggestion has no `person_id`. With a `person_id`, the start does not change.
2. UI: the menu choices are "Select a person", "A new person", then each person A to Z. Use the placeholder of `ChoiceSelect` for "Select a person".
3. UI: with "Select a person", the sheet shows the menu and **Hide This Suggestion** only. The primary button reads **Add the Items** and is disabled.
4. UI: the payee sheet starts on "Select a person". With no selection, its primary button reads **Add the Wallet** and is disabled.
5. UI: the note of the fold "Suggestions from your feeds" is "Each name is a credit in your feeds. Add it to a person that you have. Make a new person only for a person that is not in the list."
6. Each function of the state handles the new value. A selection of "A new person" works as today.

## Files

- Change: `crates/api/ui/people.js`, `crates/api/ui/peoplepages.js`.
- Change tests: `crates/api/ui-tests/people.test.mjs`.

## Checks

Mechanical:

- UI: `checkSheetStart` gives "Select a person" with no `person_id`, and the person with a `person_id`.
- UI: the choices of change 2, and the label and the ready state of change 3.
- UI: the start, the label and the ready state of the payee sheet of change 4.
- `tools/gate.sh` passes.

Visual, by the main session with screenshots (`.studio-dev/journeys/j07.json` as a base, data `base`):

- The sheet "Check: {name}" on "Select a person", and after the selection of "A new person", at 1280 px and 390 px.
- The note of the fold.

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When the ADR does not answer a question, stop and report.

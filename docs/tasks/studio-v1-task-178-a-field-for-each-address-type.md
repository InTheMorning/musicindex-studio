# Studio v1 Task 178: A Field For Each Address Type

Decision: [ADR 0219](../adr/0219-a-release-checklist-and-a-check-of-the-address-type.md) items 11 to 13. [ADR 0217](../adr/0217-a-new-artist-sees-the-result-of-a-write-the-payment-types-and-an-undo-of-a-merge.md) items 7 to 11.

Status: Done.

## Problem

Each form with the menu "Type" of an address has one field "Address" for the two types. The node fields "Custom key" and "Account" show for a node address. The operator asked for a different labeled field for each type.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Address check | reuse | crates/api/ui/recipients.js | `addressProblem` near line 459 and `WALLET_TYPES` near line 453. |
| Fields of a recipient | extend | crates/api/ui/recipients.js | `recipientFields` and `recipientFromFields`. Keep the text of each type field in its own value, and give one `address` at the save. |
| Recipient form | extend | crates/api/ui/values.js | `RecipientForm` near line 227. The fields near line 262. |
| Wallet forms of a person | extend | crates/api/ui/peoplepages.js | The wallet fields near line 554 and the wallet form near line 1559 (`walletTypeChange`). |
| Wallet field list | extend | crates/api/ui/people.js | The field list near line 886. |
| Type correction | extend | crates/api/ui/peoplepages.js | `UseTypeButton` near line 52. It must move the text to the field of the other type. |
| Type panel | reuse | crates/api/ui/peoplepages.js | `TypePanel` near line 42. |

## Changes

1. UI: with "LNURL Address", the form shows one field with the label "LNURL address". With "Node Address", it shows "Node address", "Custom key (optional)" and "Account (optional)". The menu "Type" and the panel under it stay.
2. UI: each field keeps its text when the user changes the menu. The save uses the fields of the selected type only.
3. UI: each address field has its own format check through `addressProblem`. When the text matches the other type, the field tells the reason and has **Use LNURL Address** or **Use Node Address**. The button moves the text to the field of the other type and changes the menu.
4. UI: the hint of each address field is the hint of its type today.
5. Apply changes 1 to 4 to each form with the menu "Type" of an address: the form "Add a Recipient" and the wallet forms of a person.
6. A recipient that exists opens with its address in the field of its type.

## Files

- Change: `crates/api/ui/recipients.js`, `crates/api/ui/values.js`, `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`.
- Change tests: `crates/api/ui-tests/recipients.test.mjs`, `crates/api/ui-tests/people.test.mjs`.

## Checks

Mechanical:

- UI: the field list of each type of change 1.
- UI: change 2. The text of each field stays after a change of the menu.
- UI: the save gives only the address of the selected type. An LNURL address gets no custom key and no account.
- UI: change 3. The correction moves the text and changes the type.
- UI: change 6.
- `tools/node --test crates/api/ui-tests/*.test.mjs` and `bash tools/gate.sh` pass.
- Check each changed `.js` file: copy it to a `.mjs` file and run `tools/node --check` on the copy.

Visual, by the main session with screenshots: the form "Add a Recipient" with each type, and the wallet form of a person, at 1280 px and 390 px.

## Result

Done on 2026-11-19. A ChatGPT run made the change. The main session corrected it: one move function `takeOtherType`, one `addressField`, one hint table `ADDRESS_HINTS`, and the custom pair of a wallet that starts on LNURL.

- Mechanical: 849 UI tests and `bash tools/gate.sh` pass. The syntax check of each changed `.js` file passes.
- Visual, from screenshots at 1280 px and 390 px: with "LNURL Address", the form "Add a Recipient" shows only "LNURL address". With "Node Address", it shows "Node address", "Custom key (optional)" and "Account (optional)".

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Use the exact texts of ADR 0219 and ADR 0217. Do not write a new interface text.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When the ADR does not answer a question, stop and report.

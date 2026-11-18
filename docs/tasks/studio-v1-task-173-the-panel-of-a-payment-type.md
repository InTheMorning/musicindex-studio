# Studio v1 Task 173: The Panel Of A Payment Type

Decision: [ADR 0217](../adr/0217-a-new-artist-sees-the-result-of-a-write-the-payment-types-and-an-undo-of-a-merge.md) items 7 to 11. [ADR 0138](../adr/0138-the-names-of-a-wallet-address-and-the-correction-of-a-wrong-key.md) item 3 gives the words.

Status: Complete. The screenshot check passed on 2026-11-18.

## Problem

The form "Add a Recipient" starts with "Node Address" and gives only a format hint. It accepted `artist@getalby.com` with the type "Node Address" (observation O2). A new artist does not know which type to select.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Type list | extend | crates/api/ui/recipients.js | `WALLET_TYPES` near line 453. Put "LNURL Address" first. Add the panel texts in the same file. |
| Type menu of the recipient form | extend | crates/api/ui/values.js | `TYPES` near line 64, `Choice` near line 89, `RecipientForm` near line 229, the default near line 511. |
| Type menu of a wallet operation | extend | crates/api/ui/peoplepages.js | The menu near line 1521 uses `WALLET_TYPES`. |
| Type menu of "Add a Wallet" | extend | crates/api/ui/people.js | The separate labels near line 853. Use `WALLET_TYPES`, do not keep a copy. |
| LNURL address syntax | reuse | crates/core/src/keysend.rs | `lookup_url` near line 123 gives the syntax. Write the same rule in the UI check, with a test that uses the same cases. |
| Icon | reuse | crates/api/ui/app.js | `Icon`. Verify that `at-sign` and `zap` exist in the icon set. If one is missing, add it in the form of the other icons. |

## Changes

1. One component `TypePanel` shows the panel of ADR 0217 item 8 for a type value. Each form with a type menu shows it under the menu.
2. `WALLET_TYPES` has "LNURL Address" first. Each type menu uses this list.
3. A new recipient in "Add a Recipient" starts with `lnaddress`. A recipient that exists keeps its type. The change set sheets keep their start type (ADR 0207 items 43 and 47).
4. A pure function `addressProblem(type, address)` gives null, or a reason and the other type when the address matches it. A node address is 66 hexadecimal characters. An LNURL address follows `keysend::lookup_url`. The form disables its save button and shows the reason. When the other type matches, the field has **Use LNURL Address** or **Use Node Address**, and the button changes the type.

## Files

- Change: `crates/api/ui/recipients.js`, `crates/api/ui/values.js`, `crates/api/ui/peoplepages.js`, `crates/api/ui/people.js`, `crates/api/ui/styles.css` if a style is necessary.
- Change tests: `crates/api/ui-tests/recipients.test.mjs`, `crates/api/ui-tests/people.test.mjs`, and the tests that examine the order of the type labels.

## Checks

Mechanical:

- A test examines the panel texts of item 8, the order of `WALLET_TYPES` and the start type of a new recipient.
- A test of `addressProblem`: a 66-character hex key is a node address. `name@getalby.com` is an LNURL address. `name@getalby.com` with the type node gives the reason and the other type. An empty text, `a@b@c`, `a @b.com` and `a@b/c` are refused.
- `tools/gate.sh` passes.

Visual, by the main session with screenshots:

- "Add a Recipient" with each type, at 1280 px and 390 px.
- The refusal of `artist@getalby.com` with the type "Node Address", and **Use LNURL Address**.

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When the ADR does not answer a question, stop and report.

## Result

- The delegate made the change. The main session corrected these faults:
  - The refusal was a line below the field group, not in the Address field. After the correction, it is
    the error of the Address field. One component, `UseTypeButton` in `peoplepages.js`, gives the button **Use LNURL Address**
    or **Use Node Address** in the three forms.
  - The reason "This is a Node Address." used the menu form. The correct text is "This is a Node address."
    (ADR 0138 item 3).
  - `WalletForm` examined the address in each type of sheet. The sheet "use_lnaddress" has the type of the
    wallet and an LNURL address, so its button became disabled with no reason. After the correction, the check
    applies only to the sheets with the type menu: "change_address" and "add".
- Screenshots in `.studio-dev/journeys/runs/v173a/shots/` and `v173b/shots/`, 1280 px and 390 px:
  - The form "Add a Recipient" starts with "LNURL Address" and its panel.
  - A node address with the type "LNURL Address" gives the reason and **Use Node Address**. The button
    "Add the Recipient" is disabled. **Use Node Address** changes the type and the panel.
  - The form "Add a Wallet" of a person page gives the same panel, reason and button.
- The main session ran `tools/gate.sh`. It passed with 845 UI tests.

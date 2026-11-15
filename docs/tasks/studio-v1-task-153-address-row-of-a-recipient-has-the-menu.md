# Studio v1 Task 153: The Address Row Of A Recipient Has The Menu

Decision: [ADR 0207](../adr/0207-recipients-change-sets-and-wallets.md) item 60. Items 53 to 59 give the menu of an address row in People.

Status: Complete on 2026-11-15, with the screenshot visual check. The visual gate for the operator is open.

## Result

- The delegate was `gpt-6-luna` with medium effort. It made the shared row `crates/api/ui/addressrow.js`.
- Corrections of the main session: the recipient row gives the icon component to the menu control. The accessible name of the menu button uses the menu label, because a recipient can have no address. An error of **Add to People** shows above the row. One property that was given two times and one import with no use are removed.
- The entry **Add to People** leaves the menu after the save. The text "Added" for 2 seconds does not show in this row.
- The gate passed on the tree with this change.
- Screenshots of the main session, at 1280 px and at 390 px, in the light theme and the dark theme: the tab **Value Routes** of a release and the payments of a track. Each address row has one menu button, and **Remove** is a destructive button below the row. The menu has **Change…**, **Copy** and **Add to People**. **Change…** opens the sheet "Change the Address". After **Add to People**, the menu has only **Change…** and **Copy**. Escape closes the menu, and the focus stays on its button. `$edges()` and `$overflow()` are empty, and the fit check has no finding.
- The script cannot read the clipboard. The entry **Copy** ran with no error, and the clipboard content is not checked.
- The address row of a wallet in People shows its line, "Default", the feed count and the menu with **Edit** and **Copy**, as before.
- One step changed the split and opened the menu. The two changes stayed.
- Observation, not from this task: a step that changes the split and the check box "Fee, paid first" in the same event cycle keeps only the last change. The function `change` of `Recipients` reads the list of the last render. A person cannot make two changes in one event cycle.
- Observation, not from this task: for some seconds after **Read Again**, the recipient rows open and close with no input. This task does not change the code of that state. This is an inference from the diff, not from a run of the code before the change.

Source: [walk notes of 2026-11-15](../reviews/2026-11-15-walk-notes.md), Note 15.

## Goal

The address row of a recipient of a feed uses the address row of People. The row has one menu button with **Change…**, **Copy** and **Add to People**. The button **Remove** stays a button.

## Operator Choices

- Operator choice (2026-11-15), from drawings: "Menu, Remove stays a button".
- Operator choice (2026-11-15): the row uses the code of the People row. It is not a copy of that code.
- The menu entry has the label **Add to People**, which is the label of the button today.
- A recipient of a feed has no default address, so the menu has no **Make Default**.

## Code Locations

- `crates/api/ui/values.js`, `Recipients`, lines 106 to 193. The address row has the text button **Change…**. `SaveRecipientButton` has the label **Add to People**. The recipient has the button **Remove**.
- `crates/api/ui/peoplepages.js`, `AddressRow`, lines 410 to 420: the address row of a wallet in People, with `MenuControl`.
- `crates/api/ui/menu.js`, `MenuControl`: the menu button and its menu. Escape closes the menu.
- `crates/api/ui/people.js`, `addressMenu` and `memberLine`: the model of the menu items of a wallet address.
- `crates/api/ui/styles.css`, `.address-row` and `.row-menu*`.
- `crates/api/ui-tests/people.test.mjs`, the tests of `addressMenu`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Address row | extend | crates/api/ui/peoplepages.js | `AddressRow` moves to a module that `values.js` and `peoplepages.js` import. It gets the menu items as a property. |
| Menu button and menu | reuse | crates/api/ui/menu.js | `MenuControl` gives the two forms of the button (ADR 0183 item 1). |
| Menu model | extend | crates/api/ui/people.js | A new function gives the items of a recipient address, next to `addressMenu`. |
| Row styles | reuse | crates/api/ui/styles.css | `.address-row` and `.row-menu*` give the form. |

## Do Not Touch

- `docs/adr/`, `AGENTS.md`
- The menu items of a wallet address in People
- `crates/api/ui/assets/tokens.json`
- The API routes

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a report. Write each comment in Simplified Technical English.
- Do not commit. Leave the change in the working tree and report it. The main session reads the diff, runs the gate and makes the commit.
- Do not change a file in `docs/adr/`, `AGENTS.md` or a sibling repository.
- Do not change `crates/api/ui/assets/tokens.json`. Add no color, no dimension and no new CSS class when a class of the Base components table fits.
- Do not make a second address row component. If `Recipients` cannot use `AddressRow`, stop and report the cause.
- For a file in `crates/api/ui/`, copy it to a `.mjs` file in `$TMPDIR` and run `tools/node --check` on the copy. Plain `node --check` on the `.js` file does not find a template error.
- Stop and report when a step needs a rule that this file does not give.

## Implementation Steps

1. Move `AddressRow` from `peoplepages.js` to a module that the two pages can import. Use `menu.js` if the import graph permits it. If not, make `crates/api/ui/addressrow.js` and say so in the report.
2. Give `AddressRow` its menu items and its item handler as properties. `peoplepages.js` passes the result of `addressMenu`, and the People row does not change for the user.
3. In `people.js`, add `export function recipientAddressMenu(recipient)`. It gives the items `change` (**Change…**), `copy` (**Copy**, only with an address) and `save` (**Add to People**, only when `SaveRecipientButton` shows today).
4. In `Recipients`, replace the address line and its text buttons with `AddressRow` and the items of `recipientAddressMenu`. Each item calls the function that its button calls today.
5. Keep the button **Remove** below the row, with its class and its confirmation.
6. Remove each style rule and each component that only the old buttons used.

## Acceptance Criteria

Mechanical:

1. A Node test of `recipientAddressMenu`: three items for a recipient with an address that People does not hold, no `save` item for an address that People holds, and no `copy` item with no address. No item has the label "Make Default". Each failure message starts with "ADR 0207 item 60" and gives the fix.
2. A Node test reads `values.js` and `peoplepages.js`: the two files import the same `AddressRow`, and `values.js` defines no address row of its own. The failure message names ADR 0207 item 60 and the rule "extend the component, do not copy it".
3. The tests of `addressMenu` pass with no change of their assertions.
4. `tools/gate.sh` passes.

Visual, by the main session, with screenshots. This gate is open until the main session records it:

1. The tab **Value Routes** of a release and the payments of a track, at 1280 px and at 390 px, in Light and in Dark: each address row has one menu button, and **Remove** is a button.
2. The menu opens, each of the three entries works, and Escape closes the menu.
3. The address row of a wallet in People looks the same as before the change.
4. Two controls change in one step of the screenshot tool, and the two changes stay.

## Test Commands

- `tools/node --test crates/api/ui-tests/people.test.mjs`
- `tools/gate.sh`

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/studio-v1-task-153-address-row-of-a-recipient-has-the-menu.md` (this file), each section
- `docs/adr/0207-recipients-change-sets-and-wallets.md`, items 53 to 60
- `crates/api/ui/values.js`
- `crates/api/ui/peoplepages.js`
- `crates/api/ui/menu.js`
- `crates/api/ui/people.js`
- `crates/api/ui-tests/people.test.mjs`

Goal:
- The address row of a recipient of a feed uses the address row of People, with one menu button for **Change…**, **Copy** and **Add to People**. **Remove** stays a button.

Constraints:
- The sections "Operator Choices", "Base components" and "Rules For The Delegate" of this file.

Do not touch:
- `docs/adr/`, `AGENTS.md`
- The menu items of a wallet address in People
- `crates/api/ui/assets/tokens.json`
- The API routes

Acceptance criteria:
- Each mechanical criterion of the section "Acceptance Criteria".

Test commands:
- `tools/node --test crates/api/ui-tests/people.test.mjs`
- `tools/gate.sh`

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

# Studio v1 Task 057: Confirmation Sheets

Decision: musicindex ADR 0023 R-CONFIRM-1 and ADR 0020 R-ACT-3 (shared rules). Also [ADR 0037](../adr/0037-button-roles.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md) and musicindex ADR 0023 R-CASE-1.

Status: Complete, with its screenshot visual check on 2026-10-30. The confirmation shows above the review sheet, and the close of the sheet keeps the draft.

## Goal

Each confirmation of the UI is a sheet of Studio, not the browser box `confirm()`. Each sheet names the result in clear words.

## Context

- musicindex ADR 0023 R-CONFIRM-1 says that a confirmation is a sheet of the app, and never `confirm()`. Its table of 2026-10-24 gave Studio "Differs: 8 `confirm()` calls".
- On 2026-10-30, Studio had 13 calls. The operator found the box "Discard the draft?" on `0.1.0-rc.5`. The box showed "studio.musicindex.org says", the buttons "OK" and "Cancel", and a text that was not clear.

## Files

Inspect: `crates/api/ui/app.js` (each `confirm(`), `crates/api/ui/editor.js` (each `confirm(`, `useSheet`, the sheet markup of `AddLiveSheet`), `crates/api/ui/values.js` (`confirm(`), `crates/api/ui/recipients.js` (`DISCARD_ASK`), `crates/api/ui/styles.css` (`.sheet`, `.sheet.small`), `crates/api/src/ui.rs` (the list of UI files), `crates/api/ui-tests/secrets.test.mjs` (an example of a guard test).

Change: `crates/api/ui/confirms.js` (new), `crates/api/ui/app.js`, `crates/api/ui/editor.js`, `crates/api/ui/values.js`, `crates/api/ui/recipients.js`, `crates/api/ui/styles.css` (only when the sheet needs a rule), `crates/api/src/ui.rs`, `crates/api/ui-tests/confirms.test.mjs` (new).

Do not change: the core, the API, the ADRs, the label of a button that opens a confirmation.

## Architecture

These choices are fixed. Do not change them.

1. `crates/api/ui/confirms.js` is a model module with no Preact and no browser state (ADR 0042). It exports one object `CONFIRMS`. Each entry is a function that gives `{ title, text, action, role }`. `role` is `destructive` or `primary`. The texts are in the table below, word for word.
2. `app.js` exports `askConfirm(entry)`. It gives a `Promise<boolean>`. One component `ConfirmHost` in the root of `App` shows the sheet. The sheet is a `dialog` with `showModal()`, so it shows above an open sheet, for example the review sheet.
3. The sheet has the class `sheet small`. The header holds the title as `h2`. The body holds the text as a paragraph. The footer holds "Cancel" (`class="button"`), then the action button with its role (ADR 0037). The action button is last (R-ACT-3 item 2).
4. The focus starts on "Cancel". Escape, "Cancel" and the close button give `false`. The action button gives `true`. The sheet closes in each case.
5. Each of the 13 calls of `confirm()` becomes `await askConfirm(CONFIRMS.<name>(...))`. The code after the call does not change. `DISCARD_ASK` of `recipients.js` goes, and its Node test uses `CONFIRMS.discardChangeSet`.
6. `ui.rs` serves `confirms.js` as it serves `op3.js`.
7. `confirms.test.mjs` has these tests:
   - No file of `crates/api/ui/` has `confirm(` other than `askConfirm(`. The message: `musicindex ADR 0023 R-CONFIRM-1: <file> uses the browser box confirm(). Use askConfirm of app.js with an entry of confirms.js.`
   - Each entry gives a title that ends with "?", a text, an action in title case (musicindex ADR 0023 R-CASE-1), and the role `destructive` or `primary`.

## Texts

Each title names the result. Each action names the result in title case. The second button of each sheet is "Cancel".

| Entry | Call now | Title | Text | Action | Role |
| --- | --- | --- | --- | --- | --- |
| `removeFeed(name)` | `app.js` and `editor.js`, "Remove … from your collection?" | Remove “{name}” from your collection? | The feed file on its file host does not change. You can add the feed again later. | Remove from Collection | destructive |
| `removeApp(name)` | `app.js`, "Remove the app …?" | Remove the app “{name}”? | Its token stops working. The app can no longer change your live shows. | Remove the App | destructive |
| `newAppToken(name)` | `app.js`, "Connect the app … again?" | Make a new token for “{name}”? | The old token stops working. Give the new token to the app. | Make a New Token | destructive |
| `removeFileHost(name)` | `app.js`, "Remove the file host …?" | Remove the file host “{name}”? | Studio forgets this file host. The files on it do not change. | Remove the File Host | destructive |
| `discardChangeSet()` | `app.js`, `DISCARD_ASK` | Discard this change set? | Each feed that you did not write gets back its draft from before the change. A feed that you wrote does not change. | Discard the Change Set | destructive |
| `discardDraft()` | `editor.js`, "Discard the draft?" | Discard the draft? | Studio deletes each change that you did not write. The feed file on its file host does not change. | Discard the Draft | destructive |
| `startAgain()` | `editor.js`, "Start again from the hosted file?" | Start again from the hosted file? | Studio deletes each change in the draft. Then it starts a new draft from the feed file on its file host. | Start Again | destructive |
| `replaceDraft()` | `editor.js`, "Replace the draft with this earlier version?" | Replace the draft with this version? | Studio deletes each change in the draft. Then it starts a new draft from this earlier version. | Replace the Draft | destructive |
| `removeTrackCredits()` | `editor.js`, "Remove the credits of this track?" | Remove the credits of this track? | The track then uses the credits of the album. | Remove the Credits | destructive |
| `stopGrant()` | `editor.js`, "Stop the grant?" | Stop the grant? | Connected apps can no longer change this show. | Stop the Grant | destructive |
| `downloadFirst(size)` | `editor.js`, `askFirst` | Download this file first? | Studio downloads the file{ (size)} first. The copy goes into the draft now, and the write waits for the download. | Download the File | primary |
| `removeRecipients(noun)` | `values.js`, "Remove the recipients of this …?" | Remove the recipients of this {noun}? | The {noun} then uses the recipients of the album. | Remove the Recipients | destructive |

`{ (size)}` is the text ` (12.3 MB)` when the row gives a size, and nothing when it does not. Use `fileSize` as the call does now.

## Steps

1. Write `confirms.js` and `confirms.test.mjs`.
2. Write `askConfirm` and `ConfirmHost` in `app.js`, and add `confirms.js` to `ui.rs`.
3. Change the 13 calls.
4. Check the syntax of each changed UI file with a `.mjs` copy.
5. Run the four commit checks.

## Constraints

- No feed rule in the UI (ADR 0065).
- Each control has the target size of its profile.
- The sheet loads nothing from other servers (ADR 0016).
- Each UI text, comment and test message is in ASD-STE100 Simplified Technical English. The texts of the table are fixed.

## Acceptance Criteria

Mechanical:

- The two tests of item 7 pass.
- The Node tests that gave an old text use the new entry.
- The four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`, in the dark theme, in the light theme and at 390 px:

- "Discard the draft?" above the open review sheet.
- "Remove the file host …?" on the File hosts page.
- Escape closes the sheet, and the draft does not change.
- `$edges()` finds no content at the edge of a group.

## Escalation

- Stop and report when a call of `confirm()` is in a function that cannot wait for a promise.
- Stop and report when `showModal()` of the confirmation does not show above an open sheet.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-057-confirmation-sheets.md
- docs/adr/0037-button-roles.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- Each confirmation is a sheet of Studio with the texts of the table.

Constraints:
- Follow the Architecture section and the Texts table of the task file fully.

Do not touch:
- The core, the API, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

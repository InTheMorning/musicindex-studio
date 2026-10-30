# Studio v1 Task 058: Title Case On Buttons

Decision: musicindex ADR 0023 R-CASE-1 (shared rule). Also [ADR 0037](../adr/0037-button-roles.md) and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).

Status: Not started. It starts after the commit of [task 057](studio-v1-task-057-confirmation-sheets.md), because the two tasks change the same files.

## Goal

Each button label of the UI uses title case. Each other text keeps sentence case.

## Context

- musicindex ADR 0023 R-CASE-1 item 1 gives title case to some items. These are a button, a menu item, a menu title, a tab, a segment, a search scope and a window title. Item 2 gives sentence case to each other text. Examples are a heading, a section name, a sidebar item, a field label, a list row, a message and help text.
- Its table of 2026-10-24 gave Studio "Differs: sentence case on buttons".
- On 2026-10-30, a count found about 89 button labels in sentence case in `app.js`, `editor.js` and `values.js`. Tasks 056b and 057 already use title case, for example "Add a File Host" and "Discard the Draft". Thus Studio shows the two forms together, for example "Discard the draft" in the review sheet and "Discard the Draft" in its confirmation.

## Files

Inspect: each file in `crates/api/ui/`, `crates/api/ui-tests/words.test.mjs`, `crates/api/ui-tests/*.test.mjs` (the tests that give a button label), `docs/user-guide/` (the labels in bold).

Change: `crates/api/ui/*.js` (only the text of a label), `crates/api/ui/casing.js` (new), `crates/api/ui-tests/casing.test.mjs` (new) and `crates/api/src/ui.rs` (it serves `casing.js`). Also change the Node tests, the pages of `docs/user-guide/` and `README.md` that give a changed label.

Do not change: the core, the API, the ADRs, a heading, a field label, a list row, a sidebar item, a message, a hint, a sheet title, an `aria-label` that is not a visible label, the value of a select option.

## Architecture

These choices are fixed. Do not change them.

1. A button label is the visible text of an element with the class `button`, with each role of ADR 0037. This includes the busy text, for example "Making the Vault…", and a label that a ternary gives.
2. A `button` element with the class `link-row` or `row` is a list row or a sidebar item. It keeps sentence case, for example "＋ Add a person" and "File hosts".
3. The title case is the title style of the Apple HIG:
   - Each word starts with a capital letter.
   - These words are in lowercase, except as the first word or the last word: `a`, `an`, `the`, `and`, `but`, `for`, `nor`, `or`, `so`, `yet`, `as`, `at`, `by`, `from`, `in`, `into`, `of`, `off`, `on`, `onto`, `over`, `to`, `up`, `via`, `with`.
   - A word of the list that is part of the verb starts with a capital letter. The list of these pairs is `Sign In`, `Sign Up`, `Back Up`, `Set Up`, `Log In`, `Look Up`.
   - A name keeps its form, for example "nsite", "MSP 2.0", "Amber", "R2" and "npub".
4. `crates/api/ui/casing.js` is a model module with no Preact (ADR 0042). It exports `titleCase(text)` with the rules of item 3, and `isTitleCase(text)`. It is for the test. The UI does not call it, and each label stays a literal text in its template.
5. `casing.test.mjs` finds each button label of item 1 in the templates of `crates/api/ui/*.js`, and calls `isTitleCase` for it. A list in the test gives each name of item 3, and each text that the search finds by error. The failure message: `musicindex ADR 0023 R-CASE-1: the button label "<label>" in <file> is not in title case. Write "<titleCase(label)>".`

## Steps

1. Write `casing.js` and `casing.test.mjs`. Run the test, and keep its list of findings.
2. Change each label of the list. Change the Node tests that give a changed label.
3. Change the user guide pages and `README.md` that give a changed label in bold.
4. Check the syntax of each changed UI file with a `.mjs` copy.
5. Run the four commit checks.

## Constraints

- Change only the capital letters of a label. Do not change a word. Thus the word test of task 056b continues to pass with no change.
- No feed rule in the UI (ADR 0065).
- Each comment and test message is in ASD-STE100 Simplified Technical English.

## Acceptance Criteria

Mechanical:

- A Node test: `titleCase` gives "Discard the Draft", "Sign In with Amber or a Bunker", "Yes, I Have Feeds", "Move to This File Host" and "Add to a Level".
- A Node test: `isTitleCase` refuses "Discard the draft" and "Add The Folder".
- The test of Architecture item 5 passes, and it fails on the label "Discard the draft".
- The four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`, in the dark theme and at 390 px:

- The first-run pages, the File hosts page with its add form, an album page with its review sheet, and the Recipients page.
- A button with a longer label keeps its target size, and does not go past the edge of its group. `$edges()` finds nothing.

## Escalation

- Stop and report when a label does not fit its place after the change.
- Stop and report when the search of item 5 finds a label that a rule of item 3 cannot decide.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-058-title-case-buttons.md
- docs/adr/0037-button-roles.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- Each button label uses title case, and a test finds each label that does not.

Constraints:
- Follow the Architecture section of the task file fully.
- Change only the capital letters of a label.

Do not touch:
- The core, the API, the ADRs, a text that is not a button label.
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

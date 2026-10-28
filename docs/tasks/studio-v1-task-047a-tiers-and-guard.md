# Studio v1 Task 047a: The Tiers And The Guard

Parent: [task 047](studio-v1-task-047-settings-page-and-tiers.md). Decision: [ADR 0071](../adr/0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) items 1 to 8, 15 and the item 9 row.

Status: Complete - 2026-10-28.

## Goal

Each section in `crates/api/ui/pages.js` has a tier, and a Node test stops a section that comes before a section of a lower tier. The left panel model gives the row "Settings".

## Files

Inspect: `crates/api/ui/pages.js`, `crates/api/ui-tests/pages.test.mjs`, `crates/api/ui/shell.js` (`leftPanel`), `crates/api/ui-tests/shell.test.mjs`, `crates/api/ui/editor.js` (the five calls of `pageSequence` and `showSections`).

Change: `crates/api/ui/pages.js`, `crates/api/ui-tests/pages.test.mjs`, `crates/api/ui/shell.js`, `crates/api/ui-tests/shell.test.mjs`.

Do not change: `crates/api/ui/app.js`, `crates/api/ui/editor.js`, `crates/api/ui/styles.css`, the Rust crates, the ADRs.

## Architecture

These choices are fixed. Do not change them.

- `pages.js` exports `TIERS = ['identity', 'content', 'details', 'setup', 'limits']`.
- Each section object gets the key `tier`, with the value of the table in the parent task. The sequence of the sections, their fields and their rows does not change.
- Two new pages:
  - `hosts`: `{ id: 'cards', tier: 'identity' }`, `{ id: 'add', tier: 'content' }`.
  - `studio-settings`: `account`, `apps`, `announcements` (each `setup`), then `library` (`limits`).
- `pageSequence(name, options = {})`. For `studio-settings`, `options.server` false leaves out `account`. The other pages ignore `options`.
- `editor.js` reads only `id`, `fields` and `rows`. The new key `tier` must not change what it shows. Do not change `editor.js`.
- `leftPanel` gives `bottom` as Hosts, then `{ id: 'settings', label: 'Settings', current: view.name === 'settings' }`, then Lock Studio or Unlock Studio. Settings shows also with no open vault.

## Steps

1. Add `TIERS`, the tiers and the two pages to `pages.js`. Change its header comment to name ADR 0071 with ADR 0068.
2. In `pages.test.mjs`, delete the `EXPECTED` lists and the test that compares each page with them (ADR 0071 item 15). Keep the test of the new copy and the test of `ChannelFields`. Change their messages from "Change the list in crates/api/ui/pages.js and the ADR together." to name ADR 0071, as below.
3. Add the tier tests of ADR 0071 item 6. Examine each page that `pageSequence` knows, with `{ server: true }` and `{ server: false }`:
   - Each section has a tier in `TIERS`.
   - The tier index does not go down from one section to the next.
   - The message is: `ADR 0071 item 6: on the page ${page}, the section ${id} (${tier}) comes after a section of the tier ${previous}. Give the section its tier of ADR 0071 item 1, and put it after each section of a lower tier in crates/api/ui/pages.js.`
   - A test that a section with a tier not in `TIERS` fails with a message that names ADR 0071 item 1.
4. Add a test that a list with a lower tier after a higher tier fails the check. Put the check in an exported function `tierError(page, sections)` of `pages.js`, which gives the message or `null`, so that the test can give it a bad list.
5. Add tests: `studio-settings` has `account` only with `server: true`. `hosts` starts with `cards`.
6. In `shell.test.mjs`, change the expected `bottom` lists to have `settings` after `hosts`, and add a test that `settings` is current on the view `{ name: 'settings' }`.
7. Run the four commit checks.

## Constraints

- No feed rule in the UI (ADR 0065). This packet adds no function that changes the model.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- `pages.js` has no Preact and no browser state (ADR 0042).

## Acceptance Criteria

Mechanical:

- The tier test passes for each page, and `tierError` gives a message for a bad list.
- No test compares a page with a fixed list of ADR 0068.
- The shell tests give Hosts, Settings and Lock Studio in that sequence.
- The four commit checks pass.

## Test Commands

```sh
tools/node --test crates/api/ui-tests/*.test.mjs
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

- Stop and report when a section of the current pages has no tier in the table of the parent task.
- Stop and report when `editor.js` shows a different page after the change.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md
- docs/tasks/studio-v1-task-047-settings-page-and-tiers.md
- docs/tasks/studio-v1-task-047a-tiers-and-guard.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- A tier for each section in pages.js, the tier guard test, the pages hosts and studio-settings, and the row Settings in the left panel model.

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).

Do not touch:
- app.js, editor.js, styles.css, the Rust crates, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- The commands of the Test Commands section.

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

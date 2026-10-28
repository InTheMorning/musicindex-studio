# Studio v1 Task 047b: The Settings Page And The Hosts Page

Parent: [task 047](studio-v1-task-047-settings-page-and-tiers.md). Decision: [ADR 0071](../adr/0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) items 5 and 9 to 14.

Status: Complete - 2026-10-28.

## Goal

The row "Settings" opens a Settings page with Account, Connected apps, Announcements and Library. The Hosts page shows the host cards, then "Add a host". The two pages show their sections from `pageSequence`.

## Files

Inspect: `crates/api/ui/app.js` (`Hosts`, `LibraryPanel`, `AnnouncePanel`, `ConnectedAppsPanel`, `AccountPanel`, `LeftPanelFooter`, `RootPage`, the view switch in `Shell`), `crates/api/ui/pages.js` after 047a, `crates/api/ui/announce.js` line 18, `crates/api/ui-tests/announce.test.mjs`, `crates/api/ui/editor.js` line 1162, the user guide pages below.

Change: `crates/api/ui/app.js`, `crates/api/ui/announce.js`, `crates/api/ui-tests/announce.test.mjs`, `crates/api/ui/editor.js` (the text of line 1162 only), `docs/user-guide/hosts.md`, `docs/user-guide/account.md`, `docs/user-guide/announcements.md`, `docs/user-guide/connected-apps.md`, `docs/user-guide/find-your-way.md`, `docs/user-guide/first-run.md`, `docs/user-guide/use-on-a-phone.md`.

Do not change: `pages.js`, `shell.js`, `styles.css` (unless a visual defect needs it, and then say so), the Rust crates, the ADRs.

## Architecture

These choices are fixed. Do not change them.

- A new component `SettingsPage` in `app.js`. Its bar title is "Settings". It maps each section ID of `pageSequence('studio-settings', { server: serverMode })` to a panel: `account` to `AccountPanel`, `apps` to `ConnectedAppsPanel`, `announcements` to `AnnouncePanel`, `library` to `LibraryPanel`. The panels do not change.
- The view `{ name: 'settings' }` shows `SettingsPage` in the view switch of `Shell`.
- `LeftPanelFooter` shows the row `settings` as a `button class="row"` with `aria-current`, and no type dot. It opens `{ name: 'settings' }`. `RootPage` gets it through the same footer.
- `Hosts` maps the section IDs of `pageSequence('hosts')`: `cards` to the host cards, `add` to the form or the button "＋ Add a host". The four settings panels go from `Hosts`. The error notice and the result notices of a new host stay above the host cards (ADR 0071 item 13). With no host, the page shows only the form "Connect your first host", as before the change.
- The text "No announcement: the announcements are off in Hosts." changes to "No announcement: the announcements are off in Settings." The text "No app is connected. Connect an app in Hosts." changes to "No app is connected. Connect an app in Settings."
- The comment of `AccountPanel` names the Settings page.
- The user guide pages name Settings for Account, Connected apps, Announcements and Library. `hosts.md` loses the section "Other settings on the Hosts page" and gets one line that points to Settings. `find-your-way.md` and `use-on-a-phone.md` list Settings between Hosts and Lock Studio.

## Steps

1. Change `app.js`, `announce.js`, its test and the text in `editor.js`.
2. Change the user guide pages.
3. Run the four commit checks, and `ste_lint.py --check --no-heuristics` on each changed guide page.
4. The planning model does the visual check of the parent task. Do not run it.

## Constraints

- No feed rule in the UI (ADR 0065). This packet adds no function that changes the model.
- Each comment, each UI text and each guide page is in ASD-STE100 Simplified Technical English.
- Each labeled button has its role of ADR 0037.

## Acceptance Criteria

Mechanical:

- The announce test expects the text with "Settings".
- No UI text tells the user to go to Hosts for a setting: `grep -n "in Hosts" crates/api/ui/announce.js crates/api/ui/editor.js` finds no announcement text and no connected app text.
- The four commit checks pass.

Visual, by the planning model: the criteria of the parent task.

## Test Commands

```sh
tools/node --test crates/api/ui-tests/*.test.mjs
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 ~/.agents/skills/asd-ste100/scripts/ste_lint.py --check --no-heuristics docs/user-guide/hosts.md docs/user-guide/account.md docs/user-guide/announcements.md docs/user-guide/connected-apps.md docs/user-guide/find-your-way.md docs/user-guide/first-run.md docs/user-guide/use-on-a-phone.md
```

## Escalation

- Stop and report when a settings panel needs a prop that only the Hosts page has.
- Stop and report when the server shell shows a different set of panels than the desktop shell, other than Account.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md
- docs/tasks/studio-v1-task-047-settings-page-and-tiers.md
- docs/tasks/studio-v1-task-047b-settings-and-hosts-pages.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- The Settings page, the Hosts page with the host cards first, the two UI texts, and the user guide pages.

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).

Do not touch:
- pages.js, shell.js, the Rust crates, the ADRs.
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

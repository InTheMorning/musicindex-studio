# Studio v1 Task 165: The Studio Version In Settings

Decision: ADR 0166 items 1 and 2 (the Settings tabs). This task adds one read-only section, so it has no new decision.

Operator choice, 2026-11-17: the version shows in a row at the bottom of Settings > General, in the server and in the desktop app. The operator selected it from drawings, over a line below the left panel and a line in the account menu.

Status: Complete on 2026-11-17, with its screenshot visual check.

## Problem

The UI does not show the version of Studio. On the server, only `GET /api/v1/status` gives the version, and it needs a session and the CSRF header. A user and the operator cannot see the version without the developer tools. The [human check walk of 2026-11-14](../reviews/2026-11-14-human-check-walk.md), server step 1, records this fault.

## Work

1. In `crates/api/ui/pages.js`, add the section `{ id: 'version', tier: 'setup', tab: 'general' }` to `studio-settings`, as the last section of the General tab. It is in the desktop app and on the server.
2. In `crates/api/ui/app.js`, add `VersionPanel` to the map `panels` of `SettingsPage`. It shows one label and value row: the label "Studio" and the value of `version` from `GET /status`, in the mono style. The panel has no control and no draft value.
3. Use the status that `Studio` loads, if `SettingsPage` can receive it. If not, read `/status` one time in the panel, as `OutsidePanel` reads its setting. Show nothing while the value is not known.
4. Use the existing label and value row of the UI (for example the `<dl>` rows of the feed facts near `app.js` line 2036). Do not make a new row style.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Settings sections | extend | crates/api/ui/pages.js | One more section in `studio-settings`. |
| Settings panels | extend | crates/api/ui/app.js | One more entry in the map `panels`. |
| Status route | reuse | crates/api/src/routes.rs | `status` gives `version` from `CARGO_PKG_VERSION`. |
| Label and value rows | reuse | crates/api/ui/app.js | The `<dl>` rows of the feed facts. |

## Do Not Touch

- `crates/api/src/` and each Rust file. The route gives the version.
- `crates/api/ui/settingsdraft.js`. The version is not a setting.
- `tools/sheets.json`.

## Acceptance Criteria

Mechanical:

1. `crates/api/ui-tests/pages.test.mjs`: `settingsTabs` gives `version` as the last section of General, for the desktop (`server: false`) and for the server (`server: true`).
2. The tier test of `pages.test.mjs` passes, so the section sequence obeys ADR 0071.
3. `tools/gate.sh` passes.

Visual, by the main session:

1. Settings > General at 1280 px Light and at 390 px Dark. The last row is "Studio" with the version, in the mono style. `$edges()` gives no result.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/studio-v1-task-165-studio-version-in-settings.md
- crates/api/ui/pages.js (`studio-settings`, `settingsTabs`, `tierError`)
- crates/api/ui/app.js (`Studio`, `SettingsPage`, `OutsidePanel`, the `<dl>` rows near line 2036)
- crates/api/ui-tests/pages.test.mjs
- /home/citizen/.agents/skills/asd-ste100/SKILL.md, for code comments

Goal:
- Settings > General ends with a read-only row "Studio" with the version from `GET /status`.

Constraints:
- Reuse the existing row markup and styles. Add no CSS unless the row needs it, and then name the cause in your report.
- Write code comments in Simplified Technical English.
- Do not commit. Keep the change in the working tree. The main session reviews the diff and commits.

Do not touch:
- crates/api/src/, each Rust file, crates/api/ui/settingsdraft.js and tools/sheets.json.

Acceptance criteria:
- Mechanical criteria 1 to 3 of the task file.

Test commands:
- tools/gate.sh

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

## Result

- Mechanical 1 to 3: met. The section `version` is the last section of General for the desktop and for the server.
- Visual, by the main session: met. At 1280 px Light and 390 px Dark, the last section of General is "Version" with the row "Studio 0.1.0-rc.25" in the mono style. `$edges()`, `$overflow()` and the fit check give no result.
- The panel uses the status that `Studio` loads. It sends no request of its own.

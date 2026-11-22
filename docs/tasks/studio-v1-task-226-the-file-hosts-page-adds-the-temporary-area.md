# Studio v1 Task 226: The File Hosts Page Adds The Temporary Area

Decision: [ADR 0122](../adr/0122-a-new-user-starts-with-a-default-file-host.md) items 3 and 9.

Operator choice (2026-11-21): on a server, the form "Add a file host" of the File Hosts page also offers the temporary area, as step 2 of the Welcome sheet does. On the desktop, the same form offers "Folder on this computer". No decision record is necessary. The route and the limit of one temporary area exist (ADR 0122 item 9), and only one form changes.

Status: Done. The main session checked the visual criteria with screenshots of a server-mode Studio on 2026-11-21, in the dark theme at 1280 px. On File Hosts, "Temporary area (7 days)" was the first type, with its hint. After the add, the type was not in the menu. Step 2 of the Welcome sheet made the page on the temporary area.

Operator choice (2026-11-21), defect correction: the match route after an add gave 422 for the temporary area. `apply_folder_address` made the connector with no temporary folder. After the correction, it gets the host environment of the workspace, and the test `the_match_route_accepts_the_temporary_area` covers it.

## Problem

On a server, `AddHostForm` in `crates/api/ui/app.js` offers S3, SFTP, nsite and Blossom. The temporary area is only in step 2 of the Welcome sheet (`HOST_ROWS` in `crates/api/ui/welcome.js`). A user who removes the temporary area, or who does not use the Welcome sheet, cannot add it from File Hosts.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Add host form | extend | crates/api/ui/app.js | `AddHostForm` gets the type `temporary` on a server. |
| Start sheet | extend | crates/api/ui/app.js | `StartSheet` uses the `temporary` branch of `AddHostForm` and drops its own `addTemporary` button. |
| Host step text | reuse | crates/api/ui/welcome.js | `HOST_STEP.temporary` gives the line "Kept for 7 days. Pick a real file host to keep your pages." |
| Connector route | reuse | crates/api/src/routes.rs | `POST /connectors` with `{ kind: 'temporary' }` adds the area. No change. |

## Changes

1. `AddHostForm` gets the prop `connectors`. On a server with no connector of the kind `temporary`, the first value of "Type of file host" is "Temporary area (7 days)". It is also the start type.
2. The form of the type `temporary` shows `HOST_STEP.temporary` as a hint. Its one button sends `POST /connectors` with `{ kind: 'temporary' }` and calls `onAdded`. The button label is `submitLabel`, or "Add the Temporary Area".
3. The File Hosts page and the Welcome sheet give `connectors` to `AddHostForm`.
4. `StartSheet`: the row "Try it here (temporary)" opens `AddHostForm` with `start="temporary"` and `submitLabel="Make the Page"`, as each other row does. Remove `addTemporary`.
5. Put the list of the types in a pure function, for example `hostTypeOptions(serverMode, connectors)` in `welcome.js`. Give it a Node test: the desktop gives "Folder on this computer" first; a server with no temporary area gives "Temporary area (7 days)" first; a server with a temporary area does not give it.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0122 and an item.
- Button labels use title case.
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy.
- The agent does not commit and does not stage.

## Acceptance Criteria

Mechanical:

- The Node test of step 5 passes.
- `tools/gate.sh` passes.

Visual (the main session checks with screenshots of a server-mode Studio):

- File Hosts, "＋ Add a File Host": the temporary area is the first type, with its hint, when the account has none.
- Step 2 of the Welcome sheet still makes the page on the temporary area.

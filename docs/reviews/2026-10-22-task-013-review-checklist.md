# Task 013 Review Checklist

Reviewed work: the diff of each packet of [task 013](../tasks/studio-v1-task-013-recipient-picker.md), against [ADR 0027](../adr/0027-the-recipient-picker.md).
The planning model uses this list after each packet, before the commit of that packet.

## Each Packet

- The diff changes only the files that the packet names.
- The three commit checks pass.
- Each new code comment is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the packet does not tell the coding model to do.
- For a UI packet: the `drive-ui.mjs` check passes, and the planning model examines its screenshots.

## Invariants

- The picker does not remove or replace a recipient of the editor (ADR 0027).
- The list has each recipient, by address and custom pair, one time (ADR 0027).
- No table, log or API response contains a secret ([ADR 0003](../adr/0003-user-unlocked-secret-vault.md)).
- Each `/api/` route needs the access token ([ADR 0017](../adr/0017-local-api-access.md)).
- A schema change is a new migration. Migrations 0 and 1 do not change ([ADR 0014](../adr/0014-collection-in-sqlite.md)).
- The UI uses three background levels ([ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md)).

## Packet Results

| Packet | Result | Required corrections |
| --- | --- | --- |
| [013a](../tasks/studio-v1-task-013a-recipient-store.md) | Pass after two corrections | The packet named a type `RecipientIdentity` that no method used. The planning model removed it. `save_recipient` accepted an address of only spaces, which `Feed::recipients` ignores. It now refuses it. |
| [013b](../tasks/studio-v1-task-013b-recipient-routes.md) | Pass after one correction | The coding agent found that `track_routes` of task 013a filtered the track GUID to hex digits. An item GUID can be any text, so a real GUID could change. The planning model encodes the track GUID as one path segment. The tests use track GUIDs with a space and a `/`. |
| [013c](../tasks/studio-v1-task-013c-picker-sheet.md) | Pass | The planning model examined the sheet in both themes. `$small()` of `tools/drive-ui.mjs` counted a checkbox in a label of 44 px. The planning model changed the helper to accept that case. |

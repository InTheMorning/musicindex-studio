# Task 036 Review Checklist

- Date: 2026-10-26
- Reviewed: each packet of [task 036](../tasks/studio-v1-task-036-grant-vaults.md)
- Decisions: [ADR 0058](../adr/0058-grant-vaults-and-connected-apps-on-the-server.md), [ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md), [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) and [ADR 0013](../adr/0013-vault-format-and-cryptography.md)

The planning model uses this list after each packet, before its commit.

## Each Packet

- The diff changes only the files that the packet names.
- The four commit checks pass.
- Each new code comment and each new UI text is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the packet does not tell the coding model to do.
- For a UI change: the `drive-ui.mjs` check passes, and the planning model examines its screenshots in the dark theme and in the light theme.

## Manual Checks

- 036a: a granted write uses only the grant vault, never the user vault. `sync_grant_vaults` removes the vault of each grant that is not active, also while the user vault is locked. `write_granted` and `covered` do not change.
- 036a: no app token and no grant vault key is in a file, a row or a log as plain text.
- 036b: an app reaches only the workspace of its user. `/app/v1/` does not accept the session cookie, and `/api/` does not accept an app token.

## Invariants

- A granted write uses only the secrets of the grant vault ([ADR 0058](../adr/0058-grant-vaults-and-connected-apps-on-the-server.md)).
- A grant vault exists only while its grant is active ([ADR 0058](../adr/0058-grant-vaults-and-connected-apps-on-the-server.md)).
- A connected app never gets a secret of the vault ([ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md)).
- A write to a host occurs only through `pipeline::write_draft` ([ADR 0007](../adr/0007-draft-diff-write-and-announce.md), [ADR 0015](../adr/0015-connector-interface-and-write-pipeline.md)).
- No test sends a request to a host outside the computer ([ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md)).

## Results

| Work | Result | Required corrections |
| --- | --- | --- |

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
| [036a](../tasks/studio-v1-task-036a-grant-vault.md) | Pass after correction | `GET /app/v1/shows` read the hosts through the user vault, so an app could not list its shows while the vault was locked. The planning model asked for one shared function `grant_hosts`, which the list and the granted write use. A show whose grant vault does not open with the token is not in the list, and a new test covers this with a second app. A grant on a host with no secret gets no grant vault. The end of a grant removes the grant vault file at once, and the record in the user vault goes at the next unlock. No route changes a host secret, so a test changes the secret in the user vault directly. The planning model removed `add_app_without_key`, which only the harness used. In the manual checks, `grant_vault::open` takes the wrapper of the app of the token, and `hosts_with_grant_vault` reads only the grant vault, also when the user vault is open. `app_check` keeps the token in a type that clears its memory, and a test finds no token and no grant vault key in the data folder. The `drive-ui.mjs` check used a temporary harness. It passes in the dark theme, in the light theme and at 390 px with touch, with the disabled grant check box for "My signer" and "Connect again" for an app with no key. |
| [036b](../tasks/studio-v1-task-036b-apps-on-the-server.md) | Pass | No correction of the code. The hook is a builder `Workspace::with_app_hook`, because the handle comes after the router. `add_app` and `remove_app` only call it. `core/src/apps.rs` gives the token hashes, because the database connection belongs to the core. `Workspace::with_address_check` is public only with the `testing` feature, and only the dev dependencies of the server use it. One earlier assertion changed: `/app/v1/whoami` with a cookie now gives 401, not 404. In the manual checks, the outer layer skips the `Origin` rule only for `/app/v1/`, and a test shows that `/api/` still refuses a POST with no `Origin`. An unknown token counts for the client address only. An app of one user cannot see or change a show of a different user. A blocked address also gets 429 for a correct token, as the sign-in limits do. Users behind one proxy with no `--client-address-header` share one counter, and the runbook of task 037 must state this. Task 036 is complete. |

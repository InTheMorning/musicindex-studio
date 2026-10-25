# Studio v1 Task 029e: The SFTP Host Form

Parent: [task 029](studio-v1-task-029-sftp-connector.md). Decisions: [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) item 4, [ADR 0047](../adr/0047-the-sftp-connector.md), [ADR 0037](../adr/0037-button-roles.md), [ADR 0010](../adr/0010-studio-uses-the-afterglow-direction.md) and [ADR 0018](../adr/0018-passphrase-strength-and-error-color.md).
Sequence: the fifth packet of task 029. It depends on 029d.

## Goal

The type choice of the Hosts page gets "SFTP server". The SFTP form asks for the server, gets the fingerprint of its host key, and shows it. The host is added only after the user accepts the fingerprint. The host card of an SFTP host shows its facts and its fingerprint, and lets the user accept a new host key.

## Files

Inspect: `crates/api/ui/app.js` (`AddHostForm`, `S3HostForm`, `HostCard`, `Field`, `ErrorNotice`, `splitError`, `useAction`, `CreateVault`, `ShowRecoveryKey`), `crates/api/ui/styles.css`, `crates/api/src/routes.rs` (the fingerprint route and the `Sftp` body of 029d).

Change: `crates/api/ui/app.js`. Change `styles.css` only when a style is missing.

Do not change: the Rust code, the other UI files.

## Architecture

These choices are fixed. Do not change them.

- The type choice gets a third radio input, "SFTP server", after "S3 or R2 bucket".
- The SFTP form has two steps.
- Step 1, the server. The fields, in this sequence:

  | Field | Label | Notes |
  | --- | --- | --- |
  | name | Name (optional) | An empty name takes the host name. |
  | host | Server | Required. Placeholder `sftp.example.com`. |
  | port | Port | Default `22`. Type `number`. |
  | user | User name | Required. |
  | root | Folder on the server | Required. Placeholder `/home/me/public_html/feeds`. Hint: "The folder that holds your feed files. It is often not the first folder after the login." |
  | base_url | Public address of the folder (optional) | Type `url`. Hint: "The web address that publishes this folder." |
  | login | Login | A `fieldset` with three radio inputs: "Private key", "Password" and "ssh-agent". Default "Private key". |
  | private_key | Private key | For "Private key" only. A `textarea`, `autocomplete="off"`, `spellcheck="false"`. Hint: "Paste the text of the key file, such as id_ed25519. Studio keeps it in your vault." |
  | passphrase | Passphrase of the key (optional) | For "Private key" only. Type `password`. |
  | password | Password | For "Password" only. Type `password`. Hint: "Studio keeps it in your vault." |

  "ssh-agent" shows the hint "Studio uses the keys of the ssh-agent on this computer, and keeps no secret."
- The button of step 1 has the role primary and the text "Connect", or "Connecting…" while the request runs. It sends `POST /connectors/fingerprint`.
- Step 2, the fingerprint. The form shows "Studio connected to <host>. Make sure that this is the fingerprint of the server:" and the fingerprint in the `mono` style. The buttons are "Accept and add the server" (primary) and "Back" (normal). "Accept and add the server" sends `POST /connectors` with the `Sftp` body and the fingerprint as `host_key`.
- Errors use the same displays as the S3 form:
  - `invalid_address` and `invalid_config` show at the fields, and `folder_not_found` shows at the folder field.
  - `no_conditions` shows as an error notice with the title "Studio cannot write to this server safely".
  - `host_unauthorized` shows "The server refused the login." `host_unavailable` shows "Studio cannot connect to the server."
  - `vault_required` opens `CreateVault`, as the S3 form does.
- Each field uses a functional state update, as the S3 form does after packet 028d.
- `HostCard` of an `sftp` host has the eyebrow "SFTP server". Its facts are "Server" (`user@host:port`), "Folder", "Login" ("Private key", "Password" or "ssh-agent"), "Host key" (the fingerprint, `mono`) and "Public address". It never shows a secret.
- An operation on the host can give `host_unauthorized` with "the host key changed" in the message. The card then shows a notice with the new fingerprint and a button "Accept the new key" (action). The button sends `PUT /connectors/{id}` with that fingerprint as `host_key`. A 409 `host_key_mismatch` shows "The server gives a different key now. Connect again to see it."
- "Edit" changes the name and the public address, as for S3.
- Each control has the target size of its profile. All functions work from the keyboard.

## Steps

1. Add the third type and step 1 of the form.
2. Add step 2 and the add request, with the error displays and the vault flow.
3. Change `HostCard` for the kind `sftp`, with the new host key notice.
4. The planning model checks the form with `tools/drive-ui.mjs`. The coding model does not need to run it.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No build step, and nothing loads from other servers (ADR 0016).

## Acceptance Criteria

Mechanical:

- The four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs` against a local `TestSshd`, in the dark theme and in the light theme:

- The SFTP form shows step 1, and after "Connect" it shows the fingerprint of the server.
- After "Accept and add the server", the host card shows its facts and the fingerprint, and no secret.
- After a change of the host key, the card shows the new key notice, and "Accept the new key" removes it.
- At 390 px, the form is one column, and no control is smaller than the touch target.

Human tests, open until the operator can use a real SFTP host:

- A login with a password, and a login with ssh-agent, on a real server.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

Stop and report when the form needs a change of the Rust code.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-029e-sftp-host-form.md
- docs/adr/0047-the-sftp-connector.md
- docs/adr/0037-button-roles.md
- The files that the Files section names.

Goal:
- The Hosts page gets the host type "SFTP server". Its form connects, shows the host key fingerprint, and adds the host only after the user accepts it. The host card shows the fingerprint and lets the user accept a new host key.

Constraints:
- Follow the Architecture section of the task file fully.
- No build step, and nothing loads from other servers.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The Rust code, the other UI files.
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

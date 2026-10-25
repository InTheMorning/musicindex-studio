# Studio v1 Task 028d: The S3 Host Form

Parent: [task 028](studio-v1-task-028-s3-connector.md). Decisions: [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) item 4, [ADR 0046](../adr/0046-the-s3-and-r2-connector.md), [ADR 0037](../adr/0037-button-roles.md), [ADR 0010](../adr/0010-studio-uses-the-afterglow-direction.md) and [ADR 0018](../adr/0018-passphrase-strength-and-error-color.md).
Sequence: the fourth packet of task 028. It depends on 028c.

## Goal

On the Hosts page, the user selects the type of a new host: a folder on this computer, or an S3 or R2 bucket. The S3 form sends `POST /connectors` with the S3 body of 028c. It shows each error of the route, and it asks for a vault when the vault does not exist, as the MSP import does. The host card of an S3 host shows its facts.

## Files

Inspect: `crates/api/ui/app.js` (`AddHostForm`, `Hosts`, `HostCard`, `MspImport` and its `vault_required` flow, `Field`, `ErrorNotice`, `splitError`, `useAction`), `crates/api/ui/styles.css`, `crates/api/src/routes.rs` (the S3 body of 028c), `tools/drive-ui.mjs`.

Change: `crates/api/ui/app.js`. Change `styles.css` only when a style is missing.

Do not change: the Rust code, the other UI files.

## Architecture

These choices are fixed. Do not change them.

- `AddHostForm` starts with a choice of two types, as two radio inputs in a `fieldset` with the legend "Type of host": "Folder on this computer" and "S3 or R2 bucket". "Folder on this computer" is selected first. The local folder fields do not change.
- The S3 fields, in this sequence:

  | Field | Label | Notes |
  | --- | --- | --- |
  | name | Name (optional) | An empty name takes the bucket name. |
  | endpoint | Endpoint address | Required. Placeholder `https://<account>.r2.cloudflarestorage.com`. Hint: "The S3 API address that your provider gives." |
  | region | Region | Default `auto`. Hint: "R2 uses auto. AWS uses the region of the bucket, such as us-east-1." |
  | bucket | Bucket | Required. |
  | prefix | Folder in the bucket (optional) | Placeholder `feeds/`. |
  | url_style | Address style | A `select` with "Path style" (`path`) and "Virtual host style" (`virtual`). Default `path`. |
  | base_url | Public address of the bucket (optional) | Type `url`. Hint: "The custom domain that publishes the bucket. R2 needs a custom domain for a public feed." |
  | access_key_id | Access key ID | Required. `autocomplete="off"`. |
  | secret_access_key | Secret access key | Required. Type `password`. `autocomplete="off"`. Hint: "Studio keeps this key in your vault." |

- The submit button has the role primary and the text "Add the bucket", or "Testing the bucket…" while the request runs.
- Errors: `invalid_address` shows at the endpoint field or the public address field, from the error message. `no_conditions` shows as an error notice with the title "Studio cannot write to this provider safely" and the message of the route. `host_unauthorized` shows "The provider refused the access key." `host_unavailable` shows "Studio cannot connect to the endpoint." Each error uses the `--error` token (ADR 0018).
- `vault_required` opens `CreateVault`, then `ShowRecoveryKey` when it gives a key, and then sends the form again, as the MSP import does. `vault_locked` shows "Unlock the vault first."
- After the host is added, the page calls the match route, as for a folder (`added` in `Hosts`).
- `HostCard` of an `s3` host has the eyebrow "S3 or R2 bucket". Its facts are "Endpoint", "Bucket", "Folder in the bucket" (or "—") and "Public address". "Edit" changes the name and the public address, as for a folder. The card never shows a key.
- Each control has the target size of its profile (musicindex ADR 0024 R-TARGET-1). All functions work from the keyboard.

## Steps

1. Add the type choice and the S3 fields.
2. Add the error display and the vault flow.
3. Change `HostCard` for the kind `s3`.
4. The planning model checks the form with `tools/drive-ui.mjs`. The coding model does not need to run it.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No build step, and nothing loads from other servers (ADR 0016).
- A UI text is in plain English, with the words of the table.

## Acceptance Criteria

Mechanical:

- The four commit checks pass.
- No UI file contains the text of a test secret.

Visual, for the planning model with `tools/drive-ui.mjs`, in the dark theme and in the light theme:

- The Hosts page shows the type choice, and the S3 form after "S3 or R2 bucket".
- An S3 host that the planning model adds against a local `FakeS3` shows as a host card with its facts and no key.
- The `no_conditions` error shows its title and its message.
- At 390 px, the form is one column, and no control is smaller than the touch target.

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
- docs/tasks/studio-v1-task-028d-s3-host-form.md
- docs/adr/0046-the-s3-and-r2-connector.md
- docs/adr/0037-button-roles.md
- The files that the Files section names.

Goal:
- The Hosts page lets the user select "S3 or R2 bucket" and add an S3 host with the fields of the task file. The host card of an S3 host shows its facts and no key.

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

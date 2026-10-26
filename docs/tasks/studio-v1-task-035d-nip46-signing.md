# Studio v1 Task 035d: "My Signer" Through NIP-46

Parent: [task 035](studio-v1-task-035-phone.md). Decisions: [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md) items 17 to 19, [ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md) items 10 to 13, and [ADR 0044](../adr/0044-the-nsite-signer.md) item 4.
Sequence: the fourth packet of task 035. It depends on 035c.

## Goal

With no NIP-07 signer in the browser, the review sheet signs the planned events of an nsite or Blossom write through NIP-46. This works in both shells. On a server, the signer of a NIP-46 sign-in is ready for it with no second connection.

## Files

Inspect: `crates/api/ui/signing.js` (`signAll`, `eventsToSign`), `crates/api/ui/editor.js` (the review sheet near line 930), `crates/api/ui-tests/signing.test.mjs`, `crates/api/src/routes.rs` (`write_feed`), `crates/api/src/workspace.rs`, `crates/core/src/nip46.rs` (035c), `crates/server/src/nip46_sessions.rs` (035c).

Change: `crates/api/src/workspace.rs`, `crates/api/src/routes.rs`, `crates/api/src/lib.rs`, `crates/api/src/tests.rs`, `crates/api/ui/signing.js`, `crates/api/ui/editor.js`, `crates/api/ui/app.js`, `crates/api/ui-tests/signing.test.mjs`, `crates/server/src/auth.rs`, `crates/server/src/tests.rs`.

Do not change: the write route and the pipeline. The signed events go to the write route as now.

## Architecture

These choices are fixed. Do not change them.

### The API, In Both Shells

- `Workspace` gets `remote_signer: Option<RemoteSigner>`. `WorkspaceHandle` gets `set_remote_signer(RemoteSigner)`.
- `POST /api/v1/signer/connect` with `{}` or `{"bunker"}`, and `GET /api/v1/signer`, behave as the connection routes of 035c. The connected signer goes into the workspace. `DELETE /api/v1/signer` removes it.
- `GET /api/v1/signer` gives `{"state": "none"|"pending"|"connected", "pubkey"}`.
- `POST /api/v1/signer/sign` with `{"event"}` asks the remote signer to sign one unsigned event. It refuses an event of a kind other than 35128 and 24242 with 422 `kind_not_permitted`. It refuses an event whose `pubkey` is not the public key of the signer with 409 `signer_key_differs`. It gives the signed event.
- The route does not run under the lock of the workspace while it waits for the signer.

### The Server

- After a NIP-46 sign-in, the server gives the connected signer to the workspace of the user with `set_remote_signer`. A sign-out removes it.

### The UI

- `signing.js` gets `remoteSigner(sign)`: an object with `signEvent(event)` that calls `sign(event)`. `signAll` does not change.
- The review sheet uses `window.nostr` when the browser has it. Otherwise, it gets `GET /api/v1/signer`. With a connected signer, it signs through `remoteSigner` and `POST /api/v1/signer/sign`. With no signer, it shows "Connect a signer" with the action role, which opens the NIP-46 connection of 035c in the sheet.
- If the signer key is not the key of the host, the sheet shows the text of `signer_key_differs`. It then asks for no signature.
- The signer step of the nsite and Blossom host forms gets the same choice: NIP-07, or a NIP-46 signer.

## Steps

1. Add the remote signer to the workspace and the four routes, with API tests that use `TestSigner`.
2. Give the signer of a NIP-46 sign-in to the workspace in the server.
3. Add `remoteSigner` with its Node tests, and the review sheet and host form changes.
4. Do the visual check.

## Constraints

- Each code comment and each UI text is in ASD-STE100 Simplified Technical English, as the files are.
- No new crate.
- The UI loads nothing from other servers. The relay connections are in the API.

## Acceptance Criteria

Mechanical:

- An API test connects `TestSigner` through `POST /api/v1/signer/connect` with its `bunker://` address. It signs the planned events of an nsite write on the test relay and the test Blossom server, and writes. The site has the new manifest.
- An API test refuses an event of kind 1 with 422 `kind_not_permitted`, and an event of a different key with 409 `signer_key_differs`.
- A server test: after a NIP-46 sign-in, `GET /api/v1/signer` gives `connected` with no second connection.
- The Node tests examine `remoteSigner` with `signAll`.
- The four commit checks pass.

Visual:

- The planning model checks the review sheet of an nsite write in three states: a connected NIP-46 signer, "Connect a signer", and a signer of a different key. Each one in the dark theme, in the light theme and at 390 px.

Human test, in task 037:

- An nsite write from an Android phone with Amber.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when `TestSigner` needs more than the plan window of ADR 0050 item 10 for many events.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-035d-nip46-signing.md
- docs/adr/0056-server-sign-in-methods-and-vault-wrappers.md
- docs/adr/0050-nsite-and-blossom-signer-environment-and-plan.md
- The files that the Files section names.

Goal:
- Add a remote signer to the workspace, with the routes to connect it and to sign one planned event.
- Let the review sheet and the host forms sign through it when the browser has no NIP-07 signer.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate. The write route and the pipeline do not change.
- Write each code comment and each UI text in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- The write route and the pipeline.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file. The planning model does the visual check.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

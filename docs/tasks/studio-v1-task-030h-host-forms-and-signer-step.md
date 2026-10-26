# Studio v1 Task 030h: The Host Forms And The Signer Step

Parent: [task 030](studio-v1-task-030-nsite-and-blossom.md). Decisions: [ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md) items 1 to 3 and 10 to 12, [ADR 0048](../adr/0048-the-nsite-and-blossom-connectors.md) items 1 to 5, [ADR 0044](../adr/0044-the-nsite-signer.md) items 1, 3 and 6, [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) item 4, [ADR 0037](../adr/0037-button-roles.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md), [ADR 0016](../adr/0016-ui-without-a-build-step.md).
Sequence: the eighth packet of task 030. It depends on 030g.

## Goal

The type choice of the Hosts page gets "nsite site" and "Blossom server". Each form gets the key from a NIP-07 signer or from a pasted private key. It reads the lists of the key and adds the host. The host card shows the facts of the host and the state of the relays of a site. The Review sheet asks the signer to sign the planned events before a write.

## Files

Inspect: `crates/api/ui/app.js` (`AddHostForm`, `SftpHostForm`, `HostCard`, `Field`, `ErrorNotice`, `splitError`, `useAction`, `CreateVault`, `ShowRecoveryKey`), `crates/api/ui/editor.js` (`ReviewSheet`), `crates/api/ui/styles.css`, `crates/api/ui-tests/shell.test.mjs` (the form of a model test), `crates/api/src/routes.rs` (the routes of 030f and 030g).

Change: `crates/api/ui/app.js`, `crates/api/ui/editor.js`. Add `crates/api/ui/signing.js` and `crates/api/ui-tests/signing.test.mjs`. Change `styles.css` only when a style is missing.

Do not change: the Rust code, the other UI files.

## Architecture

These choices are fixed. Do not change them.

### The Type Choice

- The type choice gets two more radio inputs after "SFTP server": "nsite site" and "Blossom server".

### The Key Step (Both Forms)

- A `fieldset` "Signing" with two radio inputs: "My signer" (the default) and "Key in the vault".
- "My signer": a button "Use my signer" (role action). It calls `window.nostr.getPublicKey()`. With no `window.nostr`, the form shows an error notice: "Studio found no NIP-07 signer in this browser. Install a signer extension, or select Key in the vault."
- "Key in the vault": a field "Private key of the site" of type `password`, with `autocomplete="off"`. Hint: "Paste the nsec of the key. Studio keeps it in your vault."
- When the form has a key, it sends `POST /connectors/nostr-lists` with that key. It shows the `npub` in the `mono` style, and fills the relays and the servers from the result. A key with no lists gives empty fields and the hint "The key has no list. Type the addresses."

### The nsite Form

The fields after the key step, in this sequence:

| Field | Label | Notes |
| --- | --- | --- |
| name | Name (optional) | An empty name takes `nsite <d>`. |
| feed_id | Release (optional) | A `select` of the feeds. It gives the site name of a new site. |
| d | Site name | Placeholder "From the release". Hint: "1 to 13 characters: a to z, 0 to 9 and -. A site that exists keeps its name." |
| relays | Relays | A `textarea` with one address on each line. |
| servers | Blossom servers | A `textarea` with one address on each line. |
| gateway | Gateway | Default `nsite.lol`. |
| title | Title of the site (optional) | |

- The button has the role primary and the text "Add the site", or "Adding…" while the request runs.

### The Blossom Form

- The fields after the key step: "Name (optional)" (an empty name takes the host name of the first server) and "Blossom servers" (a `textarea`).
- Hint of the form: "A Blossom host keeps media only. The feed XML needs a different host."
- The button has the role primary and the text "Add the Blossom host".

### The Errors Of Both Forms

- `invalid_config` shows at the field that it names. When the message names no field, it shows as an error notice.
- `invalid_address` shows at the servers field.
- `key_mismatch` shows at the private key field: "This private key is not the key of the public key."
- `host_unavailable` shows "Studio cannot connect to the relays or the servers."
- `vault_required` opens `CreateVault`, as the S3 form does. `vault_locked` shows "Unlock the vault first."
- Each field uses a functional state update, as the S3 form does.

### The Host Card

- An nsite host has the eyebrow "nsite site". Its facts: "Public key" (the `npub`, `mono`), "Site name", "Address" (the gateway address of the site, `mono`), "Relays", "Blossom servers", "Signing" ("My signer" or "Key in the vault").
- A Blossom host has the eyebrow "Blossom server". Its facts: "Public key", "Blossom servers" and "Signing".
- The card shows no secret key.
- An nsite card has a button "Check the relays" (role normal). It sends `GET /connectors/{id}/site`. With `consistent` true, the card shows "Each relay has the newest manifest." With `consistent` false, it shows a notice: "The relays do not agree yet. Studio shows the site as inconsistent until each relay has the newest manifest." It lists each relay with a different manifest or an error.
- "Edit" changes the name, and for these kinds also the relays, the servers and the gateway, with `PUT /connectors/{id}`.

### The Signer Step Of The Review Sheet

- `signing.js` is a model module with no Preact and no browser state ([ADR 0042](../adr/0042-ui-models-with-node-tests.md)):
  - `eventsToSign(diff)` gives the events of `diff.events` with `user_signs`, in their sequence.
  - `async signAll(events, signer, onProgress)` calls `signer.signEvent(event)` for each event in sequence, and calls `onProgress(done, total)`. A signed event with a different ID gives an error with the code `changed_event`. A signer that refuses gives an error with the code `signer_refused`.
- In `ReviewSheet`, when `eventsToSign(diff)` is not empty:
  - The sheet shows a summary line: "This write signs <n> events with your signer."
  - "Write" first checks `window.nostr`. With none, the sheet shows "Studio found no NIP-07 signer in this browser. This write needs your signer." and writes nothing.
  - While `signAll` runs, the button shows "Signing <done> of <total>…".
  - Then the sheet sends `POST /feeds/{id}/write` with `diff_id` and `signed_events`.
  - `changed_event` shows "Your signer changed an event. Studio wrote nothing." `signer_refused` shows "Your signer did not sign. Studio wrote nothing."
  - `plan_expired` loads the diff again and shows "The plan of this write expired. Examine the changes again, then write."
  - `signature_required` and `signature_invalid` show their messages in the error notice.
- A write with no events to sign does not change.

## Steps

1. Add `signing.js` and its Node tests: `eventsToSign`, a signer that works, a signer that changes an event, and a signer that refuses.
2. Add the two types, the key step and the two forms.
3. Change `HostCard` for the two kinds.
4. Add the signer step to `ReviewSheet`.
5. The planning model checks the forms, the card and the signer step with `tools/drive-ui.mjs`. The coding model does not need to run it.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No build step, and nothing loads from other servers ([ADR 0016](../adr/0016-ui-without-a-build-step.md)).
- No secret key is in the browser storage, the address or a console message.
- Each control has the target size of its profile. All functions work from the keyboard.

## Acceptance Criteria

Mechanical:

- The Node tests of `signing.js` pass.
- The four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs` against local test servers, in the dark theme and in the light theme:

- The nsite form fills the relays and the servers from the lists of the key, and adds the site. The card shows its facts and no secret key.
- The Blossom form adds a host, and the card shows its facts.
- "Check the relays" shows the state of a site with a relay that did not get the newest manifest.
- The Review sheet shows the summary line and the signer step, and the notice when the browser has no signer.
- At 390 px, each form is one column, and no control is smaller than the touch target.

Human tests, open until the operator can use a test key, relays and a Blossom server:

- The operator writes a release to an nsite site with "My signer" in a browser extension.
- A podcast app plays a track from the gateway address.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

Stop and report when the forms or the signer step need a change of the Rust code.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-030h-host-forms-and-signer-step.md
- docs/adr/0050-nsite-and-blossom-signer-environment-and-plan.md
- docs/adr/0037-button-roles.md
- docs/adr/0042-ui-models-with-node-tests.md
- The files that the Files section names.

Goal:
- The Hosts page gets the host types "nsite site" and "Blossom server", with a NIP-07 or vault key step and the lists of the key. The host card shows the facts and the relay state. The Review sheet signs the planned events with window.nostr before a write.

Constraints:
- Follow the Architecture section of the task file fully: the fields, labels, hints, button roles and texts, and the error displays.
- The signing logic is in the model module signing.js, with Node tests.
- No build step, and nothing loads from other servers. No secret key in the browser storage.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.
- Do not run tools/drive-ui.mjs. The planning model does the visual check.

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

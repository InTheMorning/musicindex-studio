# Studio v1 Phase Plan

Date: 2026-10-22.
Status: draft. The ADRs in `docs/adr/` own each rule. This plan only puts the work in sequence.

## Goal

An artist can do these steps on the desktop or on a public Studio server:

1. Add the feeds of the artist to a collection.
2. Connect each feed to the host that keeps it.
3. Edit a feed, examine a diff and write the change to the host.
4. Move files from one host to a different host.
5. Announce the change to indexes.

## Non-Goals

- Studio as a host of files. Studio connects to hosts ([ADR 0009](../adr/0009-studio-connects-to-hosts-and-moves-files.md)).
- MSP 2.0 hosting, Wavlake, Fountain and Castopod connectors ([ADR 0006](../adr/0006-connectors-and-the-v1-set.md)).
- Video feeds.
- Proof tokens in feeds ([ADR 0005](../adr/0005-collection-and-first-run-discovery.md)).

## Current State

Phase 0 to Phase 4 are complete, except the visual check of task 009. Phase 1 has one open item.

- The lossless XML tree and the typed model exist ([ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md)).
- The corpus has 23 published feeds and 2 fixtures. Each feed gives the same bytes after a read and a write.
- Open item: the corpus has no Podtunes feed. The research gives no Podtunes feed URL. The operator can supply one.

- The vault exists in `studio-core` ([ADR 0013](../adr/0013-vault-format-and-cryptography.md)). A passphrase, a recovery key or a vault key from a Nostr signer opens it.
- The collection is in SQLite ([ADR 0014](../adr/0014-collection-in-sqlite.md)). A test finds no connector secret in the database file.
- The local folder connector and the write pipeline exist ([ADR 0015](../adr/0015-connector-interface-and-write-pipeline.md)).
- The local JSON API, the UI shell, the settings screen and the first-run wizard exist ([ADR 0016](../adr/0016-ui-without-a-build-step.md), [ADR 0017](../adr/0017-local-api-access.md)). A test finds no secret in API responses.
- The visual checks of task 008 pass. The [review](../reviews/2026-10-22-phase-4-visual-check.md) records the results.
- The one-question wizard of [ADR 0020](../adr/0020-first-run-asks-one-question.md) exists: discovery from addresses and musicindex, read-only feeds, a folder match, and the MSP 2.0 import with the vault on demand. Its visual check is open.
- Risk for Phase 9: discovery reads each address that a user gives. The server shell must refuse addresses in private networks.

Phase 5 (album editor) is next.

## Target State

- A Cargo workspace with a core crate, an API crate, a server binary and a Tauri desktop app.
- One browser UI in the Afterglow direction ([ADR 0010](../adr/0010-studio-uses-the-afterglow-direction.md)).

## Phases

Each phase ends with a commit that passes all tests.
A phase gets task files in `docs/tasks/` before work starts.

### Phase 0: Workspace

- Make the Cargo workspace with empty `core`, `api`, `server` and `desktop` members.
- Add the format, lint and test commands, and CI.
- Mechanical check: `cargo fmt --check`, `cargo clippy` and `cargo test` pass.

### Phase 1: Feed Model, Reader And Writer

- Model the four feed types, value blocks and value time splits ([ADR 0008](../adr/0008-feed-types-and-feed-identity.md)).
- Tasks: [002](../tasks/studio-v1-task-002-lossless-xml-tree.md), [003](../tasks/studio-v1-task-003-feed-model.md) and [004](../tasks/studio-v1-task-004-remaining-feed-elements.md) are complete.
- Keep each element that the core does not model.
- Mechanical check: a corpus of published feeds gives the same meaning after a read and a write with no edit. The corpus includes MSP 2.0, Podtunes, Wavlake and Sovereign Feeds output.
- Mechanical check: a read and write keeps the feed GUID and each item GUID.

### Phase 2: Vault

- Make the encrypted vault format, with a passphrase unlock and a NIP-44 unlock ([ADR 0003](../adr/0003-user-unlocked-secret-vault.md)).
- Task: [005](../tasks/studio-v1-task-005-vault.md) is complete.
- Mechanical check: a test opens a vault with each unlock method and fails with an incorrect key.
- Mechanical check: a test finds no secret in a log, an API response or the collection database.

### Phase 3: Collection And Local Connector

- Make the collection store and the connector interface ([ADR 0006](../adr/0006-connectors-and-the-v1-set.md)).
- Make the local folder connector first, because it needs no network.
- Make the write pipeline: draft, diff, version check and snapshot ([ADR 0007](../adr/0007-draft-diff-write-and-announce.md)).
- Mechanical check: a write stops with a conflict when the file changed after the read.
- Task: [006](../tasks/studio-v1-task-006-collection-connector-pipeline.md) is complete.

### Phase 4: API And UI Shell

- Give the core functions as a JSON API ([ADR 0002](../adr/0002-rust-core-json-api-browser-ui.md)).
- Select the UI build method in a new ADR. [ADR 0016](../adr/0016-ui-without-a-build-step.md) selects it.
- Tasks: [007](../tasks/studio-v1-task-007-local-api.md) and [008](../tasks/studio-v1-task-008-ui-shell-and-wizard.md) are complete. [009](../tasks/studio-v1-task-009-more-discovery-methods.md) is complete, except its visual check.
- Make the UI shell, the configuration screen and the first-run wizard.
- Mechanical check: API contract tests for each route.
- Visual check: the operator compares the shell with the musicindex.org baseline.

### Phase 5: Album Editor

- Make the album editor: channel, tracks, value recipients and value time splits.
- Make the diff view and the snapshot list with rollback.
- Visual check: the operator edits a published album feed completely.

### Phase 6: Remaining Connectors

- SFTP, S3 and R2, Git pages, Blossom and nsite.
- Mechanical check: each connector passes the shared connector test set against a local test server.

### Phase 7: Announce

- Send a podping and a Podcast Index pubnotify after each write, with a setting for each.
- Mechanical check: a test confirms the announcement after a write and no announcement when the setting is off.

### Phase 8: Other Feed Types

- Publisher, musicL and live item editors.

### Phase 9: Server Shell

- Refuse discovery addresses in private networks, so that a user cannot make the server read them.
- Sign-in with a Nostr signer and with email and passphrase ([ADR 0004](../adr/0004-server-sign-in-with-nostr-or-email.md)).
- Mechanical check: a session cannot read or write the feeds of a different user.
- Mechanical check: the server rejects a used sign-in event.

### Phase 10: Desktop Shell

- Package the Tauri app for Linux, macOS and Windows.
- Visual check: the operator installs and uses the app on each platform.

### Phase 11: File Moves And The Self-Hosted File Host

- Move some or all files of a user from one host to a different host ([ADR 0009](../adr/0009-studio-connects-to-hosts-and-moves-files.md)).
- Update each feed that refers to a moved file, through the write pipeline.
- Make the optional self-hosted file host and its connector. The form of this host waits for the answer to the open question in ADR 0009.
- Mechanical check: a move between two local test hosts keeps the feed GUID and the item GUIDs.
- Mechanical check: a move does not remove a file from the previous host before the new URLs are reachable.

## Risks

- Wavlake hosts most of the index and has no known write path. Many artists cannot use Studio to write their primary feeds.
- A NIP-44 unlock needs signer support. Some signers do not have it.
- A defect in the round trip can damage a feed that listeners use. Phase 1 has the largest test corpus for this reason.

## Test Strategy

- The core has unit tests and corpus tests. They need no network.
- Each connector has tests against a local test server.
- The API has contract tests.
- The operator does the visual checks. A visual check that has not occurred is an open gate.

## Rollback Strategy

- Each write keeps a snapshot. The user rolls back a feed from the snapshot list.
- Each phase is one or more commits. A failed phase is reverted.

## Open Questions


- The UI build method (Phase 4).
- The podping send method: a Hive account of Studio, or a hivepinger service (ADR 0007).
- The snapshot retention limit (ADR 0007).
- The vault key derivation function, the encryption algorithm and a recovery method (ADR 0003).
- The questions in ADR 0009.
- The order of the four feed types after the album editor.

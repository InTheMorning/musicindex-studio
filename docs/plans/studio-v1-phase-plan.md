# Studio v1 Phase Plan

Date: 2026-10-22.
Status: draft. The ADRs in `docs/adr/` own each rule. This plan only puts the work in sequence.

## Goal

An artist can do these steps on the desktop or on a public Studio server:

1. Add the feeds of the artist to a collection.
2. Connect each feed to the host that keeps it.
3. Edit a feed, examine a diff and write the change to the host.
4. Make a new release from music files, and publisher feeds for labels and artists.
5. Move files from one host to a different host.
6. Announce the change to indexes.

## Non-Goals

- Studio as a host of files. Studio connects to hosts ([ADR 0009](../adr/0009-studio-connects-to-hosts-and-moves-files.md)).
- MSP 2.0 hosting, Wavlake, Fountain and Castopod connectors ([ADR 0006](../adr/0006-connectors-and-the-v1-set.md)).
- Video feeds.
- Proof tokens in feeds ([ADR 0005](../adr/0005-collection-and-first-run-discovery.md)).

## Current State

Phase 0 to Phase 4 are complete. Phase 1 has one open item.

- The lossless XML tree and the typed model exist ([ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md)).
- The corpus has 23 published feeds and 2 fixtures. Each feed gives the same bytes after a read and a write.
- Open item: the corpus has no Podtunes feed. The research gives no Podtunes feed URL. The operator can supply one.

- The vault exists in `studio-core` ([ADR 0013](../adr/0013-vault-format-and-cryptography.md)). A passphrase, a recovery key or a vault key from a Nostr signer opens it.
- The collection is in SQLite ([ADR 0014](../adr/0014-collection-in-sqlite.md)). A test finds no connector secret in the database file.
- The local folder connector and the write pipeline exist ([ADR 0015](../adr/0015-connector-interface-and-write-pipeline.md)).
- The local JSON API, the UI shell, the settings screen and the first-run wizard exist ([ADR 0016](../adr/0016-ui-without-a-build-step.md), [ADR 0017](../adr/0017-local-api-access.md)). A test finds no secret in API responses.
- The visual checks of task 008 pass. The [review](../reviews/2026-10-22-phase-4-visual-check.md) records the results.
- The one-question wizard of [ADR 0020](../adr/0020-first-run-asks-one-question.md) exists: discovery from addresses and musicindex, read-only feeds, a folder match, and the MSP 2.0 import with the vault on demand. Its [visual check](../reviews/2026-10-22-task-009-visual-check.md) passes.
- The public address of a feed follows [ADR 0021](../adr/0021-public-address-of-a-feed.md). The collection keeps the address of each source and shows each difference. A folder match can find the folder address from the linked feeds.
- Risk for the Server Shell phase: discovery reads each address that a user gives. The server shell must refuse addresses in private networks.

Phase 5 is in progress. [ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md) gives the method of an edit. [ADR 0023](../adr/0023-web-addresses-and-audio-file-facts.md) gives the address rule and the facts of a new audio file. [ADR 0024](../adr/0024-track-numbers-follow-the-track-sequence.md) sets the track numbers from the track sequence. [Task 010](../tasks/studio-v1-task-010-draft-api.md) (the draft API) and [task 011](../tasks/studio-v1-task-011-album-editor.md) (the album editor UI) are complete. [Task 012](../tasks/studio-v1-task-012-value-editor.md) (the value editor) is complete. The visual checks of [task 011](../reviews/2026-10-22-task-011-visual-check.md) and [task 012](../reviews/2026-10-22-task-012-visual-check.md) pass. Next: [task 013](../tasks/studio-v1-task-013-recipient-picker.md) (the recipient picker).

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
- Tasks: [007](../tasks/studio-v1-task-007-local-api.md) and [008](../tasks/studio-v1-task-008-ui-shell-and-wizard.md) are complete. [009](../tasks/studio-v1-task-009-more-discovery-methods.md) is complete.
- Make the UI shell, the configuration screen and the first-run wizard.
- Mechanical check: API contract tests for each route.
- Visual check: the operator compares the shell with the musicindex.org baseline.

### Phase 5: Album Editor

- Make the album editor: channel, tracks, value recipients and value time splits.
- Tasks: [010](../tasks/studio-v1-task-010-draft-api.md), [011](../tasks/studio-v1-task-011-album-editor.md) and [012](../tasks/studio-v1-task-012-value-editor.md) are complete. [013](../tasks/studio-v1-task-013-recipient-picker.md) (the recipient picker, [ADR 0027](../adr/0027-the-recipient-picker.md)) is open.
- Make the diff view and the snapshot list with rollback.
- Visual check: the operator edits a published album feed completely.

### Phase 6: New Releases

- Show the art of the album and of each track in the editor ([ADR 0025](../adr/0025-studio-serves-cover-art.md)).
- Make a new album feed in a connected folder. Its feed GUID comes from its address ([ADR 0008](../adr/0008-feed-types-and-feed-identity.md)).
- Import audio files and art into the draft. The approved write puts them on the host ([ADR 0026](../adr/0026-import-music-files-through-the-draft.md)).
- Mechanical check: an import of tagged test files gives one track for each file, with its title, track number, duration, size and type.
- Mechanical check: an approved write puts each file on the host before the feed file, and does not replace a file that exists.
- Mechanical check: the UI loads no image from a server other than Studio.
- Visual check: the operator makes a new release from a folder of music files and writes it.

### Phase 7: Publisher Feeds

- Make and edit publisher feeds for labels and artists, with a `podcast:remoteItem` for each release ([ADR 0008](../adr/0008-feed-types-and-feed-identity.md)).
- Decide in an ADR how a release refers to its publisher feed, and what Studio changes in the release feeds.
- Visual check: the operator makes a label feed and an artist feed for the releases in the collection.

### Phase 8: Remaining Connectors

- SFTP, S3 and R2, Git pages, Blossom and nsite.
- Mechanical check: each connector passes the shared connector test set against a local test server.

### Phase 9: Announce

- Send a podping and a Podcast Index pubnotify after each write, with a setting for each.
- Mechanical check: a test confirms the announcement after a write and no announcement when the setting is off.

### Phase 10: Other Feed Types

- MusicL and live item editors.

### Phase 11: Server Shell

- Refuse discovery addresses, audio addresses and image addresses in private networks, so that a user cannot make the server read them.
- Sign-in with a Nostr signer and with email and passphrase ([ADR 0004](../adr/0004-server-sign-in-with-nostr-or-email.md)).
- Mechanical check: a session cannot read or write the feeds of a different user.
- Mechanical check: the server rejects a used sign-in event.

### Phase 12: Desktop Shell

- Package the Tauri app for Linux, macOS and Windows.
- Visual check: the operator installs and uses the app on each platform.

### Phase 13: File Moves And The Self-Hosted File Host

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
- The snapshot retention limit (ADR 0007). ADR 0022 keeps each snapshot until a subsequent ADR sets a limit.
- The vault key derivation function, the encryption algorithm and a recovery method (ADR 0003).
- The questions in ADR 0009.
- The sequence of the musicL and live item editors (Phase 10).
- Later work that the operator noted on 2026-10-22, with no phase yet:
  - v4vmm sends value time splits to Studio. The album editor does not edit them (task 012).
  - A check that a payment route works. It probably needs a connected Lightning node. Podcast Index marks some routes with a warning, and its method is not known.

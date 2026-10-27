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
- The Git pages connector. It comes after v1 ([ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md)).
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

Phase 5 is complete. [ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md) gives the method of an edit. [ADR 0023](../adr/0023-web-addresses-and-audio-file-facts.md) gives the address rule and the facts of a new audio file. [ADR 0024](../adr/0024-track-numbers-follow-the-track-sequence.md) sets the track numbers from the track sequence. [Task 010](../tasks/studio-v1-task-010-draft-api.md) (the draft API) and [task 011](../tasks/studio-v1-task-011-album-editor.md) (the album editor UI) are complete. [Task 012](../tasks/studio-v1-task-012-value-editor.md) (the value editor) is complete. The visual checks of [task 011](../reviews/2026-10-22-task-011-visual-check.md) and [task 012](../reviews/2026-10-22-task-012-visual-check.md) pass. [Task 014](../tasks/studio-v1-task-014-editor-layout.md) (the editor layout of [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md)) is complete, and its [visual check](../reviews/2026-10-22-task-014-visual-check.md) passes. [Task 013](../tasks/studio-v1-task-013-recipient-picker.md) (the recipient picker) is complete, and its [visual check](../reviews/2026-10-23-task-013-visual-check.md) passes. [Task 015](../tasks/studio-v1-task-015-hosted-position.md) ([ADR 0031](../adr/0031-a-new-element-takes-its-hosted-position.md)) puts a new element at its hosted position. The [visual check of Phase 5](../reviews/2026-10-23-phase-5-visual-check.md) passes. Phase 6 (new releases) is complete, with tasks 016 to 020. The [visual check of Phase 6](../reviews/2026-10-23-phase-6-visual-check.md) passes, with the checks of tasks 019, 020 and 024. Task 024 closes its six open items. Phase 7 (the release library) is complete. Milestone 7a (task 021) is complete, and its [visual check](../reviews/2026-10-23-phase-7a-visual-check.md) passes with ten open items. [Task 022](../tasks/studio-v1-task-022-ui-pass-after-7a.md) ([ADR 0037](../adr/0037-button-roles.md)) corrects them, and its [visual check](../reviews/2026-10-24-task-022-visual-check.md) passes with five open items. Milestone 7b ([task 023](../tasks/studio-v1-task-023-release-library-7b.md), [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md)) is complete, and its [visual check](../reviews/2026-10-24-phase-7b-visual-check.md) passes with seven open items. Phase 8 (publisher feeds) is complete, with [task 025](../tasks/studio-v1-task-025-publisher-feeds.md), and its [visual check](../reviews/2026-10-25-phase-8-visual-check.md) passes with four open items. Task 026 (the shell of ADR 0039) is complete. The tasks of Phase 9 (remaining connectors) are complete, with their human tests open. Phase 10 (announce) is complete, with [task 031](../tasks/studio-v1-task-031-announce.md). Its human test with a real feed is open. Phase 11 (other feed types) has two milestones. Milestone 11a (playlists, [task 032](../tasks/studio-v1-task-032-playlists.md)) is complete, and its visual check passes. Milestone 11b (live items, connected apps and show grants, [task 033](../tasks/studio-v1-task-033-live-items.md)) is complete, and its visual checks pass. Its human test with a real host and the other app is open. Phase 12 (the server shell) is planned, with tasks 034 to 037.

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
- Tasks: [010](../tasks/studio-v1-task-010-draft-api.md), [011](../tasks/studio-v1-task-011-album-editor.md) and [012](../tasks/studio-v1-task-012-value-editor.md), [013](../tasks/studio-v1-task-013-recipient-picker.md) (the recipient picker, [ADR 0027](../adr/0027-the-recipient-picker.md) and [ADR 0030](../adr/0030-an-added-recipient-gets-no-split.md)) and [014](../tasks/studio-v1-task-014-editor-layout.md) (the editor layout, [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md)) and [015](../tasks/studio-v1-task-015-hosted-position.md) (the hosted position, [ADR 0031](../adr/0031-a-new-element-takes-its-hosted-position.md)) are complete. The [visual check](../reviews/2026-10-23-phase-5-visual-check.md) passes.
- Make the diff view and the snapshot list with rollback.
- Visual check: the operator edits a published album feed completely.

### Phase 6: New Releases

- Decisions: [ADR 0025](../adr/0025-studio-serves-cover-art.md), [ADR 0026](../adr/0026-import-music-files-through-the-draft.md), [ADR 0032](../adr/0032-new-releases-formats-paths-and-new-feeds.md) [ADR 0033](../adr/0033-a-lossless-import-gives-an-mp3-enclosure.md), [ADR 0034](../adr/0034-an-import-reuses-the-same-file-on-the-host.md) and [ADR 0040](../adr/0040-replace-the-audio-of-a-track.md).
- Tasks: [016](../tasks/studio-v1-task-016-cover-art.md) (cover art), [017](../tasks/studio-v1-task-017-new-album-feed.md) (a new album feed) and [018](../tasks/studio-v1-task-018-import-music-files.md) (the import, in four packets) are complete. The [visual check](../reviews/2026-10-23-phase-6-visual-check.md) passes. [Task 019](../tasks/studio-v1-task-019-lossless-import.md) (an MP3 enclosure for a lossless import, in four packets) is complete, and its check passes. [Task 020](../tasks/studio-v1-task-020-reuse-the-file-on-the-host.md) (an import uses the same file on the host) is complete, and its check passes. The [review checklist](../reviews/2026-10-23-phase-6-review-checklist.md) applies to each.
- [Task 024](../tasks/studio-v1-task-024-phase-6-open-items.md) corrects the six open items of the visual check, and its check passes.
- Show the art of the album and of each track in the editor ([ADR 0025](../adr/0025-studio-serves-cover-art.md)).
- Make a new album feed in a connected folder. Its feed GUID comes from its address ([ADR 0008](../adr/0008-feed-types-and-feed-identity.md)).
- Import audio files and art into the draft. The approved write puts them on the host ([ADR 0026](../adr/0026-import-music-files-through-the-draft.md)).
- Mechanical check: an import of tagged test files gives one track for each file, with its title, track number, duration, size and type.
- Mechanical check: an approved write puts each file on the host before the feed file, and does not replace a file that exists.
- Mechanical check: the UI loads no image from a server other than Studio.
- Visual check: the operator makes a new release from a folder of music files and writes it.
- Visual check: the operator imports a WAV file and a FLAC file, and writes the release with an MP3 enclosure and two alternative enclosures for each.

### Phase 7: The Release Library

- Decisions: [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md), [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) and [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md).
- Milestone 7a: [task 021](../tasks/studio-v1-task-021-release-library-7a.md), in seven packets. The [review checklist](../reviews/2026-10-23-phase-7-review-checklist.md) applies to each. Milestone 7b: [task 023](../tasks/studio-v1-task-023-release-library-7b.md), in eight packets: published feeds with optional downloads, the folder scan, and the move of the feed XML. Each milestone has its own visual check.
- Studio records each file on each host. The user can reference, dereference, copy, move and delete each file on each host.
- The user selects a library folder. The library has one folder for each release and each publisher feed, with its feed XML, images and audio files.
- Add a published feed: Studio keeps its XML in the library and offers to download its images and audio files. Each download is optional.
- Each release has a default host for its files and a host for its XML. Each file can go to a different host.
- The diff lists each upload, replacement and deletion on each host. The write puts the uploads, then the feed XML, then the deletions.
- Studio compares the library folder with its records, and shows each difference.
- Move the current collection, its connected folders and its pending files into the library at the first start.
- A move of the feed XML sets a redirect, or leaves a feed with `itunes:new-feed-url` at the address before the move. The announcement of the new address waits for Phase 10 (ADR 0038 item 21).
- Mechanical check: a release with files on two local test hosts writes a feed with the correct address for each file.
- Mechanical check: a changed audio file gets a new name on the host, and the write deletes the file from before only after the feed is written.
- Mechanical check: Studio does not delete a file on a host while a feed of the library refers to its address.
- Visual check: the operator adds a published feed, puts its files on two hosts, changes one audio file, and writes the release.

### Phase 8: Publisher Feeds

- Decision: [ADR 0041](../adr/0041-publisher-feeds-and-two-way-links.md). It decides how a release refers to its publisher feeds, and what Studio changes in the release feeds.
- Task: [025](../tasks/studio-v1-task-025-publisher-feeds.md), in four packets, is complete. The [review checklist](../reviews/2026-10-25-phase-8-review-checklist.md) applies to each. The [visual check](../reviews/2026-10-25-phase-8-visual-check.md) passes.
- Make and edit publisher feeds for labels and artists, with a `podcast:remoteItem` for each release ([ADR 0008](../adr/0008-feed-types-and-feed-identity.md)).
- Each link goes in the two directions, with the same `rel` on the two sides.
- Visual check: the operator makes a label feed and an artist feed for the releases in the collection.

### The Shell Before Phase 9

- Decisions: [ADR 0039](../adr/0039-the-shell-releases-and-the-outline.md) and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
- Task: [026](../tasks/studio-v1-task-026-shell.md), in five packets, is complete. The [review checklist](../reviews/2026-10-25-task-026-review-checklist.md) records each packet and the operator check in Firefox.
- Studio starts on Releases. The left panel holds the destinations, or the outline of the open feed. The bar has the back control.
- Mechanical check: the Node tests examine the models of the left panel, of Releases and of the navigation.
- Visual check: the visual criteria of ADR 0039.

### Phase 9: Remaining Connectors

- Decisions: [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md), [ADR 0044](../adr/0044-the-nsite-signer.md), [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md), [ADR 0046](../adr/0046-the-s3-and-r2-connector.md), [ADR 0047](../adr/0047-the-sftp-connector.md), [ADR 0048](../adr/0048-the-nsite-and-blossom-connectors.md), [ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md) and [ADR 0051](../adr/0051-the-core-signs-nostr-events-with-secp256k1.md).
- Sequence: the interface change, then S3 and R2, then SFTP, then nsite and Blossom.
- Task: [027](../tasks/studio-v1-task-027-connector-interface.md) (the interface change), in three packets, is complete. The [review checklist](../reviews/2026-10-25-task-027-review-checklist.md) records each packet.
- Task: [028](../tasks/studio-v1-task-028-s3-connector.md) (S3 and R2), in four packets, is complete. The [review checklist](../reviews/2026-10-25-task-028-review-checklist.md) records each packet. Its human test with a real bucket is open.
- Task: [029](../tasks/studio-v1-task-029-sftp-connector.md) (SFTP), in five packets, is complete. The [review checklist](../reviews/2026-10-25-task-029-review-checklist.md) records each packet. Its human tests of the password login and the ssh-agent login on a real server are open.
- Task: [030](../tasks/studio-v1-task-030-nsite-and-blossom.md) (nsite and Blossom), in eight packets, is complete. The [review checklist](../reviews/2026-10-25-task-030-review-checklist.md) records each packet. Its human tests with a browser signer, relays and a podcast app are open.
- Human tests against real hosts stay open until the operator can use a real host of each type ([ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) item 5).
- Mechanical check: each connector passes the test module of the trait against a local test server.
- Mechanical check: a test confirms that Studio does no deletion when a new file does not pass the confirmation.
- Open research items. Each one needs a live host, a container or a different computer:
  - R2: the conditional DeleteObject and the conditional CompleteMultipartUpload. Also Garage, B2, Wasabi, Spaces, Ceph and Versity.
  - Hosts and gateways: the `Content-Type` of a feed, and `Range` for audio.
  - nsite feeds: do podcast apps and Podcast Index accept a feed address on an nsite gateway?
  - nsite manifests: the event size limit of the relays, with a manifest of 500 paths.
  - NIP-46 as a second "My signer" method ([ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md) item 1).
  - `ssh2`: a build on Windows and macOS.
  - Docker tests with MinIO and Garage. Docker did not operate in the research session.

### Phase 10: Announce

- Decision: [ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md). Studio sends one request to the Podcast Index hub, with `podping=1`. It needs no key and no Hive account.
- Task: [031](../tasks/studio-v1-task-031-announce.md), in four packets, is complete. The [review checklist](../reviews/2026-10-25-task-031-review-checklist.md) records each packet and the visual check. The human test is open.
- Send a podping and a Podcast Index pubnotify after each write, with a setting for each. A feed move announces the new address and the address before the move (ADR 0035 item 21, ADR 0038 item 21).
- The diff shows each planned announcement. A queue sends it 30 seconds after the write, and tries again after a failure. The feed page shows the last announcement.
- Mechanical check: a test confirms the announcement after a write and no announcement when the setting is off.
- Visual check: the review sheet, the feed facts and the Hosts panel of task 031d.
- Human test: after a write of a real feed, a podping with the feed address appears, and stophammer crawls the feed. Only the PodSovereign source shows the `podping=1` parameter.

### Phase 11: Other Feed Types

- Decisions: [ADR 0053](../adr/0053-playlists-are-musicl-feeds-with-remote-items.md) (playlists) and [ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md) (live items, connected apps and show grants). ADR 0054 amends the invariant of ADR 0007.
- Milestone 11a: [task 032](../tasks/studio-v1-task-032-playlists.md), in three packets. The [review checklist](../reviews/2026-10-26-task-032-review-checklist.md) applies to each. The user makes and edits a `musicL` playlist, with tracks from a search of musicindex and from the releases of the collection.
- Milestone 11b: [task 033](../tasks/studio-v1-task-033-live-items.md), in six packets. The [review checklist](../reviews/2026-10-26-task-033-review-checklist.md) applies to each. The user adds a live show to an album feed or a show feed. The user starts and ends it through the review sheet, and can add a recording. A connected app such as v4vmm can start and end a show and set its live value under a show grant.
- Mechanical check: a written playlist has its remote items in sequence and no `<item>`.
- Mechanical check: a test client with an app token starts and ends a granted show, and Studio refuses each change that the grant does not cover.
- Visual check: the visual criteria of tasks 032 and 033.
- Human test: a test show with v4vmm or the live publisher, after that repository builds its side of the contract of ADR 0054.

### Phase 12: Server Shell

- Goal of the operator: an artist changes a feed from a phone, away from home, through a public Studio server or a personal one.
- Decisions: [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) (user folders, accounts, sessions and the vault window), [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md) (sign-in methods and vault wrappers), [ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md) (the private-network guard), [ADR 0058](../adr/0058-grant-vaults-and-connected-apps-on-the-server.md) (grant vaults and connected apps on the server) and [ADR 0059](../adr/0059-uploads-in-parts-and-the-home-screen-app.md) (uploads in parts and the home screen app). The [authentication research](../research/2026-10-26-phase-12-auth-research.md) gives the options.
- Milestone 12a: [task 034](../tasks/studio-v1-task-034-server-core.md), in six packets. The server serves the API to many users, with email and NIP-07 sign-in, long sessions, a short vault window and the private-network guard. It is complete, with its visual check. The review checklist records the result.
- Milestone 12b: [task 035](../tasks/studio-v1-task-035-phone.md), in seven packets. Passkeys with PRF, the sign-in with the recovery key, NIP-46 sign-in and signing, uploads in parts and the home screen app. It is complete, with its visual check. The human tests with real phones are open in task 037.
- Milestone 12c: [task 036](../tasks/studio-v1-task-036-grant-vaults.md), in two packets. Grant vaults in both shells, and connected apps on the server. It is complete, with its visual check.
- Milestone 12d: [task 037](../tasks/studio-v1-task-037-runbook-and-human-tests.md). The runbook, the deployment on `studio.musicindex.org` and the human tests with real phones. The [runbook](../runbooks/studio-server.md) exists. The deployment and the [human tests](../reviews/2026-10-26-phase-12-human-tests.md) are open.
- Mechanical check: a session cannot read or write the feeds of a different user.
- Mechanical check: the server rejects a used sign-in event and a used passkey challenge.
- Mechanical check: with the policy `Public`, Studio refuses each address in a private network, also after a redirect.
- Mechanical check: a granted write works while the user vault is locked, and only with the token of the app.
- Visual check: the visual criteria of tasks 034, 035 and 036.
- Human test: the tests of task 037.

### The Editor Files And Covers

- The operator found three defects on 2026-10-27. A feed in the DeMu form showed no cover. A description showed blank space. No field could take an image or an audio file from the computer, the release or a host.
- Decisions: [ADR 0060](../adr/0060-the-album-cover-keeps-the-form-of-its-feed.md) (the cover keeps the form of its feed), [ADR 0061](../adr/0061-a-text-value-has-no-space-at-its-ends.md) (no space at the ends of a text value) and [ADR 0062](../adr/0062-a-file-picker-for-each-file-field.md) (a file picker for each file field, and the audio player).
- [Task 038](../tasks/studio-v1-task-038-cover-and-text-space.md), one packet, and [task 039](../tasks/studio-v1-task-039-file-picker.md), three packets, are complete. Their screenshot visual checks and their human tests passed. The [review checklist](../reviews/2026-10-27-tasks-038-039-review-checklist.md) records each packet.
- The test feed of the operator was hidden from apps, and Studio did not show it. [ADR 0063](../adr/0063-one-switch-hides-a-feed-from-apps.md) gives one switch for it. [Task 040](../tasks/studio-v1-task-040-hide-from-apps.md) is implemented. Its screenshot visual check passed, and its human test is open.
- Mechanical check: the mechanical criteria of tasks 038, 039 and 040.
- Visual check: the visual criteria of tasks 038, 039 and 040.

### Phase 13: Desktop Shell

- Package the Tauri app for Linux, macOS and Windows.
- Visual check: the operator installs and uses the app on each platform.

### Phase 14: File Moves And The Self-Hosted File Host

- Move some or all files of a user from one host to a different host ([ADR 0009](../adr/0009-studio-connects-to-hosts-and-moves-files.md)). A move is a change of placement ([ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md) item 10).
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
- The snapshot retention limit (ADR 0007). ADR 0022 keeps each snapshot until a subsequent ADR sets a limit.
- The vault key derivation function, the encryption algorithm and a recovery method (ADR 0003).
- The questions in ADR 0009.
- Later work that the operator noted on 2026-10-22, with no phase yet:
  - v4vmm sends value time splits to Studio. The album editor does not edit them (task 012).
  - A check that a payment route works. It probably needs a connected Lightning node. Podcast Index marks some routes with a warning, and its method is not known.

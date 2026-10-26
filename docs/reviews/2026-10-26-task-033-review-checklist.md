# Task 033 Review Checklist

- Date: 2026-10-26
- Reviewed: each packet of [task 033](../tasks/studio-v1-task-033-live-items.md)
- Decisions: [ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md), [ADR 0007](../adr/0007-draft-diff-write-and-announce.md), [ADR 0017](../adr/0017-local-api-access.md) and [ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md)

The planning model uses this list after each packet, before its commit.

## Each Packet

- The diff changes only the files that the packet names.
- The four commit checks pass.
- Each new code comment and each new UI text is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the packet does not tell the coding model to do.
- For a UI change: the `drive-ui.mjs` check passes, and the planning model examines its screenshots in the dark theme and in the light theme.

## Manual Checks

- 033a: the corpus tests give the same bytes. The live item check runs before any change on a host.
- 033b: the status routes change the draft only, and never write.
- 033c: the recording uses the import path of a track, with no second import path.
- 033d: no app token is in a log, an error message, the database file or a response other than the POST response. The `/api/` layer is unchanged for the access token.
- 033e: `write_granted` writes only through `write_draft_signed`. On each error path, the feed keeps no draft that the user did not make. A draft of the user never goes to a host through a grant.
- 033e: `covered` refuses each change other than the status and the live value of the granted item.
- 033f: the text logic is in `live.js`, not in the Preact components.

## Invariants

- No write occurs without a diff that the user has seen, or a change that a show grant of the user covers ([ADR 0007](../adr/0007-draft-diff-write-and-announce.md), [ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md)).
- A write to a host occurs only through `pipeline::write_draft` ([ADR 0007](../adr/0007-draft-diff-write-and-announce.md), [ADR 0015](../adr/0015-connector-interface-and-write-pipeline.md)).
- A connected app never gets a secret of the vault ([ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md)).
- The feed GUID and the item GUIDs do not change without a user action ([ADR 0008](../adr/0008-feed-types-and-feed-identity.md)).
- No test sends a request to a host outside the computer ([ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md)).

## Results

| Work | Result | Required corrections |
| --- | --- | --- |
| [033a](../tasks/studio-v1-task-033a-live-item-rules.md) | Pass | The core did not model `podcast:contentLink`, so the coding model stopped. The planning model added `ContentLink` and `content_links` of a live item to the packet. The live item of the corpus feed `behindthesch3m3s-music-pixel-palace.xml` has no content link. ADR 0054 item 4 refuses a write of it, and a test records the one problem. Its read and its round trip do not change. The planning model made a blank `start` a problem, because the editor can send an empty text. In the manual checks, the corpus tests give the same bytes, and the live item check runs after the address check and before the first read of the host. |
| [033b](../tasks/studio-v1-task-033b-live-item-page.md) | Open | |
| [033c](../tasks/studio-v1-task-033c-recording.md) | Open | |
| [033d](../tasks/studio-v1-task-033d-connected-apps.md) | Open | |
| [033e](../tasks/studio-v1-task-033e-show-grants.md) | Open | |
| [033f](../tasks/studio-v1-task-033f-grants-ui.md) | Open | |

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
| [033b](../tasks/studio-v1-task-033b-live-item-page.md) | Pass | The content links were open forms, not entry rows. The planning model made each link an entry row with a summary line (ADR 0028). The status line and the problem texts had no padding in their group, and each problem text was an alert at each load. The planning model gave them the padding of a group row and removed the alert role. The value routes of a show said "track", so `TrackPayments` now takes the noun "show". The add route refused a feed with no `podcast:medium`, which the UI shows as an album. It now accepts that feed. In the manual checks, the status route changes only the draft. The `drive-ui.mjs` check passes in the dark theme, in the light theme and at 390 px. It shows the outline with "Live shows", the add sheet, the live item page with the problem "This show has no content link.", the review sheet with the status `live` after "Go live", the page after the change, and the show feed form. No live item can be removed yet, other than through Versions or a discard of the draft. |
| [033c](../tasks/studio-v1-task-033c-recording.md) | Pass | `pipeline::feed_addresses` gave the addresses of the tracks only. Thus a write did not copy the recording to the host. The coding model stopped, and the planning model added the items of the live items to `feed_addresses` (ADR 0054 item 6). The API test now asserts the file on the host. The done text of a first recording spoke of a file from before, and the recording showed as the stream address. The planning model gave a first recording its own text, showed the fields of the recording in a group, and hid the stream of an ended show. In the manual checks, the recording uses the import path of a track, through `ReplaceAudio`, with no second import path. The `drive-ui.mjs` check passes in the dark theme and in the light theme. It writes a show live and ended, then shows "Add the recording", the page after an MP3 import, and the review sheet with the upload and the enclosure of the live item. |
| [033d](../tasks/studio-v1-task-033d-connected-apps.md) | Pass | The outer layer of `access.rs` does the loopback `Host` check of `/app/v1/`, and it asks for the access token only on `/api/`. Thus `access.rs` did not change, and the app layer checks only the app token. Tests refuse the access token and a removed token on `/app/v1/whoami`, an app token on `/api/v1/status`, and a host that is not loopback on `/app/v1/`. In the manual checks, the token is only in the POST response and in the state of the panel. The collection keeps its SHA-256, and a test finds no token text in the database file. `ConnectedApp` has no hash. The token field had a border and no fill, so the planning model gave it the fill of a field. The `drive-ui.mjs` check of the Connected apps panel passes in the dark theme, in the light theme and at 390 px, with the token after "Make the token" and the list after "Done". |
| [033e](../tasks/studio-v1-task-033e-show-grants.md) | Pass | No correction of the code. The collection keeps no facts of live items, so `grant::active` also takes the hosted feed. `covered` also refuses a status step that ADR 0054 item 13 does not permit, and `write_granted` refuses a diff with a host operation or a feed move. A test seam `write_granted_edit` exists only in test builds. In the manual checks, `write_granted` writes only through `write_draft_signed`, with the ID of the diff of `diff_at`. Each error after step 3 removes the draft of the grant. A draft of the user with other bytes, a host operation, a feed move or a draft grant gives `DraftOpen`, and the draft does not change. `covered` compares the model that Studio reads from the new draft bytes. The app routes run the write under the lock of the workspace. `DELETE /feeds/{id}/grants/{guid}` removes a draft grant, so the planning model gave the end of a grant in packet 033f a different path. Packet 033f also gets the limit of a grant and the values of `change`. |
| [033f](../tasks/studio-v1-task-033f-grants-ui.md) | Pass | No correction of the code. In the manual checks, the text logic is in `live.js`, and the review sheet takes the new diff from the response of the grant route, with no second request. A diff with only a grant can be written. The `drive-ui.mjs` check used a connected app of the harness, and `curl` with its token as the other app. It passes in the dark theme, in the light theme and at 390 px. It shows the check box in the review sheet, the live item page with an active grant and two granted writes ("Went live" and "Set the live value"), and the page after "Stop the grant". After the stop, the app gets 409. The live item page does not load the grant list again after a write of the other app, until the page opens again. |

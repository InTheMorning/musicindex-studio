# Studio v1 Task 032: Playlists

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 11, milestone 11a.
Decisions: [ADR 0053](../adr/0053-playlists-are-musicl-feeds-with-remote-items.md), [ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md), [ADR 0032](../adr/0032-new-releases-formats-paths-and-new-feeds.md) and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).

## Goal

A user can make a playlist and edit its channel fields and value routes. The user adds tracks from a search of musicindex or from the releases of the collection. The user can change the sequence of the tracks and remove a track. Studio writes the playlist as a `musicL` feed with one `podcast:remoteItem` for each track.

## Packets

The planning model examines each packet with the [review checklist](../reviews/2026-10-26-task-032-review-checklist.md) before its commit. The packets go in this sequence:

1. [032a: the playlist feed and the track sources in the API](studio-v1-task-032a-playlist-api.md).
2. [032b: the playlist models](studio-v1-task-032b-playlist-models.md). It depends on 032a.
3. [032c: the playlist page and the picker](studio-v1-task-032c-playlist-page.md). It depends on 032b.

## Acceptance Criteria

Mechanical:

- An API test makes a new `musicL` feed, adds two remote items through the draft, and writes it. The written feed has the two remote items in sequence and no `<item>`.
- An API test gives the tracks of the collection and the tracks of an indexed feed for the picker.
- The Node tests give the rules of the picker and of the track rows.
- The corpus tests of the three `musicL` feeds still pass.
- The four commit checks pass, in CI too.

Visual:

- A new playlist, the playlist page, the picker with its two sources, and the track rows. Each one in the dark theme, in the light theme and at 390 px. The planning model checks them with `tools/drive-ui.mjs`.

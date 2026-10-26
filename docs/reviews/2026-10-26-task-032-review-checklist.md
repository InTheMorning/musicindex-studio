# Task 032 Review Checklist

- Date: 2026-10-26
- Reviewed: each packet of [task 032](../tasks/studio-v1-task-032-playlists.md)
- Decisions: [ADR 0053](../adr/0053-playlists-are-musicl-feeds-with-remote-items.md), [ADR 0022](../adr/0022-the-editor-sends-the-model-with-entry-keys.md) and [ADR 0042](../adr/0042-ui-models-with-node-tests.md)

The planning model uses this list after each packet, before its commit.

## Each Packet

- The diff changes only the files that the packet names.
- The four commit checks pass.
- Each new code comment and each new UI text is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the packet does not tell the coding model to do.
- For a UI change: the `drive-ui.mjs` check passes, and the planning model examines its screenshots in the dark theme and in the light theme.

## Manual Checks

- 032a: no test sends a request to a host outside the computer. The musicindex test uses the local `TestServer`.
- 032a: the written playlist has no `<item>`, and the corpus tests give the same bytes.
- 032b: each function of `playlist.js` keeps the entry keys of the model.
- 032c: the text logic is in `playlist.js`, not in the Preact components. The picker loads no image from a server other than Studio.

## Invariants

- A playlist has no `<item>` ([ADR 0053](../adr/0053-playlists-are-musicl-feeds-with-remote-items.md)).
- A change of a playlist does not change a different feed ([ADR 0053](../adr/0053-playlists-are-musicl-feeds-with-remote-items.md)).
- A read and a write with no edit give the same bytes ([ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md)).
- The UI loads nothing from other servers ([ADR 0016](../adr/0016-ui-without-a-build-step.md)).

## Results

| Work | Result | Required corrections |
| --- | --- | --- |
| [032a](../tasks/studio-v1-task-032a-playlist-api.md) | Pass | No correction of the code. In the manual checks, the musicindex test uses the local `TestServer`, and the written playlist has no `<item>`. The tracks route sends two requests to musicindex, one for the tracks and one for the address. The remote items of the draft model have the names `feed_guid`, `item_guid`, `feed_url`, `medium` and `title`, and the entry key `_cid`. Thus the planning model changed the names in packet 032b. A feed gets its item rows at its first read, so `own_tracks` has no tracks of a feed that Studio did not read. |
| [032b](../tasks/studio-v1-task-032b-playlist-models.md) | Pass | No correction of the code. Each function of `playlist.js` keeps the entry key `_cid`, and the module imports nothing. The old shell test said that a `musicL` feed is not editable. The coding model changed it, because ADR 0053 item 1 reverses that rule. The outline of a playlist gives the pages `album`, `tracks`, `recipients` and `files`. Thus the planning model changed packet 032c, so that each outline entry opens its own page. |
| [032c](../tasks/studio-v1-task-032c-playlist-page.md) | Open | |

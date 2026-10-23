# Phase 6 Review Checklist

Reviewed work: the diff of each task and packet of Phase 6 (tasks [016](../tasks/studio-v1-task-016-cover-art.md), [017](../tasks/studio-v1-task-017-new-album-feed.md) and [018](../tasks/studio-v1-task-018-import-music-files.md)), against [ADR 0025](../adr/0025-studio-serves-cover-art.md), [ADR 0026](../adr/0026-import-music-files-through-the-draft.md) and [ADR 0032](../adr/0032-new-releases-formats-paths-and-new-feeds.md).
The planning model uses this list after each task or packet, before its commit.

## Each Task Or Packet

- The diff changes only the files that the task names.
- The three commit checks pass.
- Each new code comment is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the task does not tell the coding model to do.
- For a UI change: the `drive-ui.mjs` check passes, and the planning model examines its screenshots.

## Invariants

- No file and no feed file goes to a host without a diff that the user saw ([ADR 0007](../adr/0007-draft-diff-write-and-announce.md)).
- An import does not replace a file on the host (ADR 0026).
- No WAV file goes to a host, and each track from a lossless import has an MP3 enclosure ([ADR 0033](../adr/0033-a-lossless-import-gives-an-mp3-enclosure.md)).
- The UI loads nothing from other servers, and the page policy does not change ([ADR 0016](../adr/0016-ui-without-a-build-step.md)).
- Each `/api/` route needs the access token ([ADR 0017](../adr/0017-local-api-access.md)).
- A schema change is a new migration ([ADR 0014](../adr/0014-collection-in-sqlite.md)).
- A read and a write with no edit give the same bytes ([ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md)).

## Results

| Work | Result | Required corrections |
| --- | --- | --- |
| [016](../tasks/studio-v1-task-016-cover-art.md) | Pass after one correction | The image route held the workspace lock during the read from the host. A slow host of art then stopped each other request, such as a save of the draft, for up to 20 s. The planning model moved the read out of the lock. |
| [017](../tasks/studio-v1-task-017-new-album-feed.md) | Pass after two corrections | The template gave a new feed file on one line, with no XML declaration, so each later diff showed the full file. The planning model gave the template the declaration and indented channel elements, and added a test. The first-run text for a new user said that a later version makes new feeds. The planning model changed it. |
| [018a](../tasks/studio-v1-task-018a-pending-files.md) | Pass | The planning model examined the sequence of the write: the diff ID, the addresses and the version of the feed file come before the first file goes to the host. The coding agent added `Store::root()` for task 018c. |
| [018b](../tasks/studio-v1-task-018b-tags-and-conversion.md) | Pass | `lofty` 0.25.4 does not build with no default features. The coding agent kept the feature `id3v2_compression_support`, which adds only `flate2` with its pure Rust back end. The planning model accepted this deviation. |
| [018c](../tasks/studio-v1-task-018c-upload-route.md) | Pass after one correction | The name of an upload went to the core as the browser gave it. The connector refuses `..`, but a name with a folder part put the file in a subfolder of the release folder. The planning model made the route keep only the last part of the name, refuse a name that is no file name with 422 `invalid_name`, and added a test. The coding agent added 400 `upload_failed` for a body that the route cannot read. |
| [018d](../tasks/studio-v1-task-018d-import-sheet.md) | Pass after three corrections | (1) Two pending files with the same bytes share one copy, and the removal of one file removed the copy of the other (a defect of 018a). The copy now goes only with its last file, and a test covers it. (2) A discard of the draft of a new release left a feed with no draft and no hosted file, which Studio could not edit. The route now refuses it with 409 `no_hosted_file`. (3) The native file field showed "Choose Files No file chosen". A button of the app now opens it. |
| [019a](../tasks/studio-v1-task-019a-alternate-enclosure-model.md) | Pass after one correction | The task told the coding agent to check the `uri` of each `podcast:source` as a web address. The namespace allows `ipfs://` there, and the check would stop the write of a feed with an IPFS source. The planning model removed the check and corrected the task. The coding agent added an `after` rule in `sync.rs` for the position after `enclosure`. The planning model accepted it, because the other callers pass no rule. |
| [019b](../tasks/studio-v1-task-019b-mp3-encoder.md) | Pass after one correction | The encoder kept two more copies of all samples in memory, so a long WAV file needed approximately three times its sample memory. The planning model changed the encoder to take 65536 sample frames at a time, and added a test with a 3-second file. The coding agent found that `mp3lame-encoder` writes no VBR header into a buffer with no capacity, and corrected it. With no VBR header, a reader gives a wrong duration. The planning model accepted that correction. The upload test of 018c failed until 019c, so 019b and 019c have one commit. |
| [019c](../tasks/studio-v1-task-019c-upload-two-files.md) | Pass | The import occurs before the lock of the workspace. A FLAC upload with no public address now gives 422 after the encode, not before it. The planning model accepted this. |
| [019d](../tasks/studio-v1-task-019d-import-sheet-and-track-page.md) | Pass | The planning model examined the screenshots of the sheet and of the track page. The coding agent used `untagged.mp3`, because `tagged.mp3` has the same release path as the MP3 file of `tagged.wav`. |

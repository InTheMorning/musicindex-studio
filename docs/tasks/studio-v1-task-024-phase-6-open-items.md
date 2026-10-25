# Studio v1 Task 024: The Open Items Of Phase 6

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 6.
Decisions: [ADR 0040](../adr/0040-replace-the-audio-of-a-track.md), [ADR 0025](../adr/0025-studio-serves-cover-art.md), [ADR 0037](../adr/0037-button-roles.md).
Source: the six open items of the [Phase 6 visual check](../reviews/2026-10-23-phase-6-visual-check.md).

## Goal

The user can replace the audio of a track and keep its GUID. The editor shows an imported image before the write. The screens of a new user give a clear next step.

## Changes

1. Replace the audio of a track ([ADR 0040](../adr/0040-replace-the-audio-of-a-track.md)):
   - `pipeline::import_replacement_into_library` gives the new file the hash name when a different file has its name in the release folder.
   - The upload route accepts `replace=true` in its query.
   - The operation `delete_unused` adds a deletion for each copy at a given address that the draft does not use.
   - The track page has "Replace the Audio File".
2. The image route gives a library image from the release folder before the write puts it on its host ([ADR 0025](../adr/0025-studio-serves-cover-art.md) item 2).
3. The row of a converted WAV file lists the MP3 file and the FLAC file by name. Task 019 made this change. This task adds no change for it.
4. "Add feeds" with no host tells the user to connect a host, with the action button "Connect a Host".
5. The MSP 2.0 import uses a hidden file field with the action button "Choose Backup Files", and lists the selected file names.
6. The header of a release with no hosted file has no "Read again".

## Acceptance Criteria

Mechanical:

- A test in `crates/api/src/tests.rs` imports a WAV file with `replace=true` after a write of `tagged.mp3`. The MP3 file gets the hash name, and the item GUID does not change. Two calls of `delete_unused` give one deletion of `album/tagged.mp3`.
- A test in `crates/api/src/tests.rs` gets an imported PNG file through the image route before the write.

Visual:

- The operator replaces the audio of a written track with a WAV file. The track page keeps the title and the GUID, and shows the MP3 enclosure and the two alternative enclosures. The review lists the deletion of the file from before. After the write, the host has the new files and does not have the file from before.
- The import sheet shows the art preview of an imported image before the write.
- The row of a converted WAV file shows the name of the FLAC file.
- "Add feeds" with no host shows "Connect a Host", and the button opens Hosts.
- The MSP 2.0 import shows "Choose Backup Files" and the names of the selected files.
- A new release with no hosted file shows no "Read again".

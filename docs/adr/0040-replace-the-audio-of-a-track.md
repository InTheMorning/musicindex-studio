# ADR 0040: Replace The Audio Of A Track

- Status: Accepted
- Date: 2026-10-25
- Owner: operator, with the technical choices of the planning model

## Context

The [Phase 6 visual check](../reviews/2026-10-23-phase-6-visual-check.md) has this open item. A removed track and a new import of the same file give a new item GUID. Listener apps then show a new track. The user has no other method to change the audio of a track.

[ADR 0008](0008-feed-types-and-feed-identity.md) keeps each item GUID. An import does not replace a file in the release folder ([ADR 0034](0034-an-import-reuses-the-same-file-on-the-host.md), [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) item 5). A new master of a song frequently has the same file name as the file from before.

On 2026-10-25 the operator selected a button on the track page.

## Decision

1. The track page has the action button "Replace the Audio File" ([ADR 0037](0037-button-roles.md)). It opens a file field for one audio file.
2. Studio imports the file as the import sheet does ([ADR 0026](0026-import-music-files-through-the-draft.md), [ADR 0033](0033-a-lossless-import-gives-an-mp3-enclosure.md), [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) item 5). A WAV file and a FLAC file give an MP3 file and a FLAC file.
3. The release folder can have a different file with the same name. Then the replacement gives the new file the hash name of [ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) item 16, for example `01 Song.3f9a2c1d.mp3`. The import sheet keeps its refusal with 409 `file_exists`.
4. The track keeps its item GUID, its title, its track number and each other value. Studio changes only these values of the track:
   - the enclosure: its address, its length and its media type.
   - each alternative enclosure with an audio media type. The import gives the new set: none for an MP3 file, and the MP3 file and the FLAC file of [ADR 0033](0033-a-lossless-import-gives-an-mp3-enclosure.md) for a lossless file. An alternative enclosure with a different media type stays.
   - the duration, from the new file.
5. The replacement does not use the title or the track number from the tags of the new file.
6. After the replacement, a copy on a host of a file from before can have no address in the draft. Each such copy gets a deletion in the draft. The write does the deletion after the feed, as [ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) item 15 says. "Undo" in the review sheet keeps the copy.
7. The file from before stays in the release folder. The Files page shows it.

## Alternatives Considered

- A match in the import sheet: an imported file with the title of a track replaces the audio of that track. Not chosen by the operator: a match by title is not reliable, and the user cannot see it before the add.
- Write the new bytes over the file from before in the release folder. Rejected: the library then loses the master from before, and the host keeps the copy from before at the same name until the write.
- Keep each copy from before on its host. Rejected: the hosts then keep files that no feed uses, and ADR 0035 item 16 deletes the file from before.

## Consequences

- The upload route gets a mark for a replacement. The import sheet does not send it.
- When the user discards the draft after a replacement, the new file stays in the release folder. The feed does not refer to it. The Files page shows it.

## Invariants

- A replacement does not change the item GUID ([ADR 0008](0008-feed-types-and-feed-identity.md)).
- A replacement does not write over a file in the release folder or on a host.

# ADR 0034: An Import Reuses The Same File On The Host

- Status: Accepted. [ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) replaces item 3 when Phase 7 implements it.
- Date: 2026-10-23
- Owner: operator

## Context

[ADR 0026](0026-import-music-files-through-the-draft.md) does not let an import replace a file on the host. The upload route gives 409 `file_exists` when a file is at the release path.

On 2026-10-23 the operator did the check of [task 019](../tasks/studio-v1-task-019-lossless-import.md). The operator wrote a lossless track, and then imported the same FLAC file again. Studio refused the import, because the MP3 file and the FLAC file were on the host. The files on the host had the same bytes as the files of the import. The user could not add the track again with its alternative enclosures.

## Decision

1. When a file is at the release path of an imported file, Studio compares the SHA-256 of the file on the host with the SHA-256 of the imported file.
2. When the two are the same, the import uses the file on the host. Studio adds no pending file for that path, and the write puts no file there. The upload result marks the file with `on_host: true`.
3. When the two are different, the upload gives 409 `file_exists`, as before. Studio does not replace a file on the host.
4. Studio compares each file of a lossless import separately. One file can be on the host, and the other can be a pending file.
5. The import sheet shows "On the host already" for a file that the import uses from the host. "Cancel" and "Remove" do not call the removal of a pending file for such a file.

## Alternatives Considered

- Keep the refusal. Not chosen by the operator: the user cannot add a track again with its lossless file.
- Offer a free name, such as `name (2).mp3`. Not chosen by the operator: the host then keeps two copies of the same bytes.

## Consequences

- To compare, Studio reads the file on the host. For a remote host in a subsequent phase, this is a download of the file.
- LAME gives the same MP3 bytes for the same source and settings. Thus a second import of the same lossless file also uses its MP3 file from the host. If the bytes are different, the upload gives 409 `file_exists`.

## Invariants

- An import does not replace a file on the host (ADR 0026).
- A file that the import uses from the host has the same SHA-256 as the imported file.

# ADR 0026: Import Music Files Through The Draft

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

On 2026-10-22 the operator asked for an import of music files for a new release. Without it, the user types the address, the size and the type of each track ([ADR 0023](0023-web-addresses-and-audio-file-facts.md)).

Studio connects to hosts and does not host files ([ADR 0009](0009-studio-connects-to-hosts-and-moves-files.md)). A write to a host occurs only through the write pipeline, with the ID of a diff that the user approved ([ADR 0007](0007-draft-diff-write-and-announce.md), [ADR 0015](0015-connector-interface-and-write-pipeline.md)).

## Decision

### The Import

1. The user selects audio files and image files on the computer. Studio makes one track for each audio file.
2. Studio reads the title, the track number and the duration of each track from the tags of its file. It reads the size and the media type from the file. The user can change each value.
3. The tracks get the sequence of their track numbers from the tags. A file with no track number goes after the files with a number. The track numbers then follow [ADR 0024](0024-track-numbers-follow-the-track-sequence.md).
4. An image file can become the art of the album or of a track.

### The Files Go To The Host With The Write

5. The import does not write to the host. Studio keeps each imported file in its data folder until the write, and the draft records the files.
6. The diff lists each file that the write puts on the host, with its path and its size.
7. The approved write first puts each file on the host through the connector of the feed, and then writes the feed file. A file goes to the host only when no file exists at its path. When a file exists at the path, the write stops before the feed file changes.
8. After the write, Studio removes its copy of each file. A removed draft also removes the copies.
9. The address of each file is the public address of the folder, with the path of the file added ([ADR 0021](0021-public-address-of-a-feed.md)). The import needs a folder with a public address.

## Alternatives Considered

- The user puts the files on the host with a different tool, and Studio reads only the tags. Rejected by the operator: the user then does the copy and types the folder address by hand.
- Copy each file to the host when the user selects it. Rejected: a write to the host then occurs without an approved diff.

## Consequences

- Studio needs a tag reader for each supported audio format. The task of this decision selects the formats and the library.
- The data folder holds the files of a release until the write. A large release uses that space for that time.
- A write can stop after it put some files on the host and before the feed file. The files on the host then have no track that refers to them. A new write puts the remaining files.
- An import to a remote host waits for its connector.

## Invariants

- No file goes to a host without a diff that the user approved.
- An import does not replace a file on the host.

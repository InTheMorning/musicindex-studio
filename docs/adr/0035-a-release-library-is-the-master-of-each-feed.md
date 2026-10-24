# ADR 0035: A Release Library Is The Master Of Each Feed

- Status: Accepted. Phase 7 of the [Studio v1 phase plan](../plans/studio-v1-phase-plan.md) implements it. Until then, the rules that this ADR replaces stay in force. [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) gives the records, the host inventory and the host operations. [ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md) gives milestone 7b, and item 21 waits for Phase 10.
- Date: 2026-10-23
- Owner: operator

## Context

Studio edits a hosted feed on its host. The feed file and its imported files are on one connected folder, and the release folder is on that host ([ADR 0026](0026-import-music-files-through-the-draft.md), [ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md) item 4). Studio keeps an imported file only until the write. An import does not replace a file on the host ([ADR 0034](0034-an-import-reuses-the-same-file-on-the-host.md)).

On 2026-10-23 the operator gave a different model. Studio keeps a folder structure with one folder for each release and each publisher feed. Each folder has the feed XML, the images and the audio files. This structure is the master that the user sees. Hosts keep copies of the files. The number of hosts has no limit, and each file of a feed can be on a different host.

Studio uploads, replaces and deletes files on each host, and writes feeds with the correct addresses.

The operator answered twelve questions in three rounds. This ADR records the answers.

## Decision

### The Library

1. In the desktop shell, the user selects a library folder, for example `~/Music/Studio`. The library has one folder for each release and each publisher feed. Each folder has the feed XML, the images and the audio files of that feed.
2. The library is the master. A host has copies of library files.
3. The collection records each library file: its path in the library, its size, its SHA-256, and each host that has it, with the host path and the public address.
4. The Server Shell phase decides where the server keeps the library of each user.

### Feeds That Are Published

5. When the user adds a published feed, Studio makes a new release folder with the feed XML. It offers to download each image and audio file that the feed refers to.
6. Each download is optional, and the user can skip it at each time. A file that is not in the library stays a remote address, and Studio shows it.
7. Studio downloads a file only when an operation needs its bytes, for example a move to a different host or a new name. It asks the user first.
8. Downloads occur in the background, two files at a time, with progress for each feed. The editor opens immediately. After a stop, Studio continues each file from the point where it stopped. It compares the size of each file with the enclosure length, and shows a difference.

### The Feed XML

9. The library XML holds the public address of each file. The library XML and the published feed file have the same bytes. [ADR 0012](0012-lossless-tree-with-a-typed-model.md) does not change.
10. When a file goes to a different host, Studio changes its address in the library XML through the draft and the diff.

### Hosts And Placement

11. The user configures as many hosts as necessary through connectors ([ADR 0006](0006-connectors-and-the-v1-set.md)).
12. Each release and each publisher feed has a default host for its files, and a host for its feed XML. The user can put each file on a different host.
13. On a host, the path of a file is the folder of its release, then the path of the file in the release folder.

### Writes To Hosts

14. Each upload, replacement and deletion on a host is in the diff. It occurs only with the approved write ([ADR 0007](0007-draft-diff-write-and-announce.md)).
15. The write puts the uploads first, then the feed XML, then the deletions. A deletion occurs only after the new feed is written and its new addresses are reachable ([ADR 0009](0009-studio-connects-to-hosts-and-moves-files.md)).
16. Studio does not write over a file on a host. When a library file changes, Studio uploads it with a new name. The new name has the first 8 hexadecimal characters of the SHA-256 of the file before the extension, for example `01 Song.3f9a2c1d.mp3`. The first upload of a file keeps its name. It changes the address in the feed. After the write of the feed, it deletes the file that was there before.
17. Studio deletes a file on a host only when no feed of the library refers to its address.

### A Move Of The Feed XML

18. When the feed XML goes to a different host, the feed keeps its `podcast:guid` ([ADR 0008](0008-feed-types-and-feed-identity.md)). Studio writes the feed to the new host with its new self link.
19. When the first host can make an HTTP 301 redirect, Studio sets a redirect from the address before the move to the new address. Each connector tells if it can make a redirect.
20. When it cannot, Studio writes a last version of the feed at the address before the move, with `itunes:new-feed-url`. Studio keeps that file, and item 17 does not apply to it.
21. After the write, Studio tells the musicindex API and Podcast Index the new address. All the steps are in one diff.

### The Move Into The Library

22. At the first start with the library, Studio asks for the library folder. Each connected folder becomes a host. Each feed gets a release folder as in items 5 to 8. Each file stays on its host, so no feed changes. Each pending file goes into its release folder.

### Changes Outside Studio

23. At the start, and when the user asks, Studio compares the library folder with its records. It shows each difference, for example a new audio file or a missing file. Studio changes nothing without a decision of the user.

### Rules That This ADR Replaces

When Phase 7 implements this ADR, these rules change:

- [ADR 0009](0009-studio-connects-to-hosts-and-moves-files.md) item 1: Studio does not give files to the public. It keeps a library on the computer of the user, and it connects to hosts.
- [ADR 0026](0026-import-music-files-through-the-draft.md) items 5, 8 and 9: an imported file goes into the release folder and stays there. Its address comes from its host (items 12 and 13).
- [ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md) items 4 and 10 to 12: the pending files become library files with no host.
- [ADR 0034](0034-an-import-reuses-the-same-file-on-the-host.md) item 3: a different file at the host path gets a new name (item 16), and the upload does not give 409.

## Alternatives Considered

- The library in the Studio data folder. Not chosen by the operator: the user must see the folder and make backups of it.
- Decide at this time that the server shell keeps a library for each user. Not chosen by the operator: the Server Shell phase decides it.
- Download each file of a published feed before the editor opens. Not chosen by the operator: the user must be able to skip downloads, and a large feed takes minutes.
- A version number in a new name, such as `01 Song-2.mp3`, or a folder for each hash. Not chosen by the operator.
- Always a pointer file at the old feed address, with no redirect. Not chosen by the operator: an app that does not read `itunes:new-feed-url` keeps the old address.
- Move each feed into the library when the user opens it, or start with an empty library. Not chosen by the operator.
- Each file chooses its host with no default, or one host for each release. Not chosen by the operator.
- A library XML that refers to library paths, and a build of the published file at each write. Not chosen by the operator: the published file is then not the same as the library file.
- Replace a changed file at the same address. Not chosen by the operator: an app or a CDN can keep the bytes from before, and the length in the feed then does not agree.
- Keep the hosts the same as the library with no approval of each file operation. Not chosen by the operator: ADR 0007 requires a diff before each write.
- Studio owns the folder, and a change by a different program is an error. Not chosen by the operator.
- Start after Phase 7 (publisher feeds), or with the file moves. Not chosen by the operator: publisher feeds also live in the library.

## Consequences

- The library needs disk space for each audio file that the user downloads.
- A move of a file to a different host becomes a change of its placement. The file moves of [ADR 0009](0009-studio-connects-to-hosts-and-moves-files.md) item 4 use this method.
- A feed on a host that Studio cannot write, such as Wavlake, can go to a different host. That is a move of the feed file, with `podcast:previousUrl` ([ADR 0008](0008-feed-types-and-feed-identity.md)).

## Invariants

- No upload, replacement or deletion on a host occurs without a diff that the user saw.
- Studio does not write over a file on a host.
- Studio does not delete a file on a host while a feed of the library refers to its address.
- The library XML and the published feed file have the same bytes.

## Open Questions

- The download limits: the maximum size of one file, and the disk space that Studio keeps free.

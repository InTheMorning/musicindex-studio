# ADR 0035: A Release Library Is The Master Of Each Feed

- Status: Accepted. Phase 7 of the [Studio v1 phase plan](../plans/studio-v1-phase-plan.md) implements it. Until then, the rules that this ADR replaces stay in force.
- Date: 2026-10-23
- Owner: operator

## Context

Studio edits a hosted feed on its host. The feed file and its imported files are on one connected folder, and the release folder is on that host ([ADR 0026](0026-import-music-files-through-the-draft.md), [ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md) item 4). Studio keeps an imported file only until the write. An import does not replace a file on the host ([ADR 0034](0034-an-import-reuses-the-same-file-on-the-host.md)).

On 2026-10-23 the operator gave a different model. Studio keeps a folder structure with one folder for each release and each publisher feed. Each folder has the feed XML, the images and the audio files. This structure is the master that the user sees. Hosts keep copies of the files. The number of hosts has no limit, and each file of a feed can be on a different host.

Studio uploads, replaces and deletes files on each host, and writes feeds with the correct addresses.

The operator answered eight questions in two rounds. This ADR records the answers.

## Decision

### The Library

1. In the desktop shell, the user selects a library folder, for example `~/Music/Studio`. The library has one folder for each release and each publisher feed. Each folder has the feed XML, the images and the audio files of that feed.
2. The library is the master. A host has copies of library files.
3. The collection records each library file: its path in the library, its size, its SHA-256, and each host that has it, with the host path and the public address.
4. The Server Shell phase decides where the server keeps the library of each user.

### Feeds That Are Published

5. When the user adds a published feed, Studio makes a new release folder. It downloads into it the feed XML, and each image and audio file that the feed refers to.
6. A file that Studio cannot download stays a remote address. Studio shows each such file.

### The Feed XML

7. The library XML holds the public address of each file. The library XML and the published feed file have the same bytes. [ADR 0012](0012-lossless-tree-with-a-typed-model.md) does not change.
8. When a file goes to a different host, Studio changes its address in the library XML through the draft and the diff.

### Hosts And Placement

9. The user configures as many hosts as necessary through connectors ([ADR 0006](0006-connectors-and-the-v1-set.md)).
10. Each release and each publisher feed has a default host for its files, and a host for its feed XML. The user can put each file on a different host.
11. On a host, the path of a file is the folder of its release, then the path of the file in the release folder.

### Writes To Hosts

12. Each upload, replacement and deletion on a host is in the diff. It occurs only with the approved write ([ADR 0007](0007-draft-diff-write-and-announce.md)).
13. The write puts the uploads first, then the feed XML, then the deletions. A deletion occurs only after the new feed is written and its new addresses are reachable ([ADR 0009](0009-studio-connects-to-hosts-and-moves-files.md)).
14. Studio does not write over a file on a host. When a library file changes, Studio uploads it with a new name that has a short content hash. It changes the address in the feed. After the write of the feed, it deletes the file that was there before.
15. Studio deletes a file on a host only when no feed of the library refers to its address.

### Changes Outside Studio

16. At the start, and when the user asks, Studio compares the library folder with its records. It shows each difference, for example a new audio file or a missing file. Studio changes nothing without a decision of the user.

### Rules That This ADR Replaces

When Phase 7 implements this ADR, these rules change:

- [ADR 0009](0009-studio-connects-to-hosts-and-moves-files.md) item 1: Studio does not give files to the public. It keeps a library on the computer of the user, and it connects to hosts.
- [ADR 0026](0026-import-music-files-through-the-draft.md) items 5, 8 and 9: an imported file goes into the release folder and stays there. Its address comes from its host (items 10 and 11).
- [ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md) items 4 and 10 to 12: the pending files become library files with no host.
- [ADR 0034](0034-an-import-reuses-the-same-file-on-the-host.md) item 3: a different file at the host path gets a new name (item 14), and the upload does not give 409.

## Alternatives Considered

- The library in the Studio data folder. Not chosen by the operator: the user must see the folder and make backups of it.
- Decide at this time that the server shell keeps a library for each user. Not chosen by the operator: the Server Shell phase decides it.
- Keep the XML of a published feed and refer to its remote files. Not chosen by the operator: the library must be complete, so that each file can go to each host.
- Each file chooses its host with no default, or one host for each release. Not chosen by the operator.
- A library XML that refers to library paths, and a build of the published file at each write. Not chosen by the operator: the published file is then not the same as the library file.
- Replace a changed file at the same address. Not chosen by the operator: an app or a CDN can keep the bytes from before, and the length in the feed then does not agree.
- Keep the hosts the same as the library with no approval of each file operation. Not chosen by the operator: ADR 0007 requires a diff before each write.
- Studio owns the folder, and a change by a different program is an error. Not chosen by the operator.
- Start after Phase 7 (publisher feeds), or with the file moves. Not chosen by the operator: publisher feeds also live in the library.

## Consequences

- The library needs disk space for each audio file of each feed.
- The download of a large catalog takes time. Studio must show its progress, and continue after a stop.
- A move of a file to a different host becomes a change of its placement. The file moves of [ADR 0009](0009-studio-connects-to-hosts-and-moves-files.md) item 4 use this method.
- A feed on a host that Studio cannot write, such as Wavlake, can go to a different host. That is a move of the feed file, with `podcast:previousUrl` ([ADR 0008](0008-feed-types-and-feed-identity.md)).

## Invariants

- No upload, replacement or deletion on a host occurs without a diff that the user saw.
- Studio does not write over a file on a host.
- Studio does not delete a file on a host while a feed of the library refers to its address.
- The library XML and the published feed file have the same bytes.

## Open Questions

- The format of the content hash in a new name, for example `01 Song.3f9a2c.mp3`.
- How Studio moves the current collection, its connected folders and its pending files into the library.
- How the feed XML of a release relates to the feed address of [ADR 0021](0021-public-address-of-a-feed.md) when the XML host changes.
- How Studio downloads a large catalog: its limits, the sequence, and the continuation after a stop.

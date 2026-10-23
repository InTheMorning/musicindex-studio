# ADR 0036: Library Records, The Host Inventory And Host Operations

- Status: Accepted
- Date: 2026-10-23
- Owner: operator, with the technical choices of the planning model

## Context

[ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) makes a library folder the master of each feed. It leaves the records, the host operations and the download limits to Phase 7.

On 2026-10-23 the operator answered four questions about Phase 7:

- The user types the library path in the browser UI until the Desktop Shell phase. The default is `~/musicindex/studio`.
- Studio notices and records each file on each host. The user can reference, dereference, move, copy and delete each file on each host.
- The download limits are a user setting.
- Phase 7 has two milestones, 7a and 7b, each with its own visual check.

The planning model added the technical choices below.

## Decision

### The Library Folder

1. At the first start with the library, a sheet proposes `~/musicindex/studio`. The user can type a different path. Studio makes the folder, writes a test file, and removes it. Studio keeps the path in a new `settings` table.
2. A release folder has the name of the feed file with no extension, as in [ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md) item 4. When that name is in use, Studio adds ` 2`, ` 3` and so on. The feed XML is in the release folder with its file name, for example `my-album/my-album.xml`.
3. After each write, and after each read of the hosted file, Studio puts the hosted feed XML into the release folder. The draft stays in the collection ([ADR 0014](0014-collection-in-sqlite.md)).

### The Records

4. A migration adds these records:
   - `settings`: one row for each setting, as a key and a JSON value.
   - `feeds.folder`: the release folder. `feeds.files_connector_id`: the default host of the files. The existing `connector_id` and `path` stay the host and the path of the feed XML.
   - `library_files`: each file of a release folder, with its path in the folder, its size, its SHA-256 and its media type. It also has the address that the file came from. `local` is true when the bytes are in the library.
   - `host_files`: the inventory. Each file that Studio saw on each host, with its path, its size, its version, and its SHA-256 when Studio read it. A row refers to the library file with the same SHA-256, when there is one.
   - `draft_operations`: each host operation that the user started in the draft: a copy of a library file to a host, or a deletion of a host file.
5. `library_files` and the release folder replace the pending files: the `draft_files` table and the store of [ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md) items 10 to 12. An imported file goes into its release folder immediately.

### The Inventory

6. Studio lists each host with `Connector::list` at the start, after each write, and when the user asks. The list gives the size and a stamp of each file with no read of the file. The stamp is a change marker, for example the size and the modification time, or an `ETag`. Studio records each file in `host_files`. It reads the SHA-256 of a host file only when the size agrees with a library file. A row keeps its SHA-256 while its stamp does not change.
7. A feed refers to a host file when the public address of the host file is an address in the feed: an enclosure, a `podcast:source` or an image.

### Addresses And Placement

8. The host of a library file is its own host, when the user selected one, or the default host of its release.
9. The address of a library file on its host is the public address of `<release folder>/<path>`. When the inventory has a different file at that path, the address uses the hash name of [ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) item 16.
10. The draft refers to the address. An address that has no host file with the same SHA-256 is an upload in the diff.

### User Operations On Files

11. The release page has a "Files" page. It shows each library file and each copy of it on each host. It also shows each host file of the release folder that is not a copy of a library file.
12. The user can do these operations. Each one changes the draft or adds a draft operation, and the write does it:
    - Reference: put the address of a copy in the feed, and remove the current address.
    - Dereference: remove an address from the feed.
    - Copy: put a library file on a different host, with no change of the feed.
    - Move: copy, reference the new copy, and delete the copy from before.
    - Delete: remove a copy from a host.
13. Studio refuses a deletion of a host file while a feed of the library refers to its address. The diff names each feed that refers to it.

### The Write

14. The diff lists the feed XML change, each upload and copy with its host and its size, and each deletion with its host.
15. The write puts the uploads and the copies first, then the feed XML, then the deletions. Before the deletions, Studio reads each new address through its connector, to confirm that the file is there ([ADR 0009](0009-studio-connects-to-hosts-and-moves-files.md)).
16. `Connector` gets `stat(path)`, which gives the size and the stamp of one file, and `delete(path, stamp)`. A deletion uses the stamp from the inventory, so Studio does not delete a file that changed after the scan.

### Download Limits

17. The settings `download_max_file_bytes` and `download_min_free_bytes` have the default 2 GiB each. Milestone 7b uses them.

### Milestones

18. Milestone 7a: the library folder, the records, the inventory, placement, the user operations, the write, and the move of the current collection.
19. Milestone 7b: published feeds with optional downloads, the folder scan, and the move of the feed XML.

## Alternatives Considered

- A fixed library path until the Desktop Shell phase. Not chosen by the operator.
- Delete only the files that Studio uploaded. Not chosen by the operator: Studio records each file on each host, and the user can delete each one.
- Fixed download limits, or no limits. Not chosen by the operator.
- One phase with one visual check. Not chosen by the operator.
- Keep the pending files store, and add the library beside it. Rejected: two copies of each imported file, and two sets of records.
- A deletion with no inventory version. Rejected: Studio could delete a file that a different program changed after the scan.

## Consequences

- The import route writes into the release folder, not into the store.
- Each write updates the inventory of the hosts that it touched.
- The list of a large host takes time. [ADR 0019](0019-select-a-feed-from-the-host.md) limits the depth and the number of files of a list.

## Invariants

- The library XML has the same bytes as the hosted feed file after each write.
- No host operation occurs without a diff that the user saw.
- Studio does not delete a host file while a feed of the library refers to its address.
- A deletion uses the stamp from the inventory.
- A scan reads no file. A test enforces it.

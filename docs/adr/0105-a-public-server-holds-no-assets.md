# ADR 0105: A Public Server Holds No Assets

- Status: Accepted
- Date: 2026-11-01
- Owner: operator
- Accepted: 2026-11-01 by the operator, with items 10, 15, 17 and 18 as written.
- Changes, for a server with `--hold-assets off`: [ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) item 2 (the library is the master), [ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md) items 6 to 10 and items 13 to 15 (downloads and the folder scan), and [ADR 0055](0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) item 21 (the storage limit).
- Uses: [ADR 0007](0007-draft-diff-write-and-announce.md) (no write without a diff), [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) item 15 (the read of each new address), [ADR 0059](0059-uploads-in-parts-and-the-home-screen-app.md) item 4 (an upload expires) and [ADR 0062](0062-a-file-picker-for-each-file-field.md) item 29 (media tickets).

## Context

- The library of ADR 0035 is the master of each feed. On the server, ADR 0055 item 3 puts the library of each user in the user folder. Each audio file and each image of a user thus stays on the disk of the server.
- On 2026-11-01, the operator decided that `studio.musicindex.org` must not hold the audio of users. The operator also decided that a public server holds no assets by default.
- The reasons are the storage cost, the backups and the content that an operator holds. The Linode Backup Service does not copy a Block Storage volume, and the Studio data is on a volume. A copy of only the databases is small, and the root disk can hold it.
- On 2026-11-01, the production volume held 3.0 MiB. The four libraries held 1.4 MiB, with no audio. The change thus has almost no files to remove.
- Most operations need the bytes of a file only for a short time. The audio player reads a byte range from the host and keeps no file (`crates/api/src/media.rs`). The image route reads each image from its address and keeps it in memory (`routes::get_image`).
- One operation needs the bytes for longer. ADR 0007 and ADR 0035 item 14 permit no upload to a host before the user sees the diff. A new file must thus wait on the server until the write.
- On 2026-11-01, the operator selected four answers. The rule covers audio and images. A new file waits on the server with an expiry. Studio removes a file only when a host has a copy. The operator can change the option. On 2026-11-01, the operator said that the mockups of `.studio-dev/mockup-105/site/` "look good".

## Decision

### The Option

1. The server has the option `--hold-assets off|on`. The default is `off`. With `on`, the server works as before this ADR. A personal server, where the operator is the artist, can use `on`.
2. The desktop does not change. Its library is on the computer of the user, and it is the master.
3. An asset is an audio file or an image of a release folder. The feed XML in the release folder is not an asset, and it stays (ADR 0036 item 3).

### The Host Is The Master

4. With `off`, the host of a file is its master. A library file with `local` false, a SHA-256 and a copy on a host is the usual state of a written file.
5. After a write, Studio reads each new address (ADR 0036 item 15). Then it removes the local bytes of each asset that the write sent. The row keeps its SHA-256, its size and the address of its copy as `source_url`, and it gets `local` false.
6. A copy or a move to a different host reads the file from its source host through the connector. The bytes stay in a temporary file in `incoming/` during the operation. Studio removes the temporary file after the operation, also after a failure.
7. A lossless import (ADR 0033) makes its files in `incoming/`. The new files are then pending files (item 8).

### Pending Files

8. A pending file is an asset that an upload, an import or a replacement (ADR 0040) gave, and that no write has sent. It stays on the server until the write that sends it. It stays for a maximum of 7 days from the import.
9. The Files page shows the status "Waits for the write" and the date of the removal for each pending file. The sheet "Add tracks" tells the user that the server keeps the files only until the write, for a maximum of 7 days.
10. At the end of the 7 days, Studio removes the bytes. The draft keeps the address of the file. The Files page shows "Removed from this server" with "Add the File Again". The write refuses the draft with 409 `file_missing` until the user adds the file again. Studio does not change the draft by itself.
11. Studio examines the pending files at the start and each hour, as it does for uploads (ADR 0059 item 4).

### Files That No Host Has

12. A local asset that no host has, with the same SHA-256, stays on the server. The Files page shows "On this server only" with an error notice. The user can write the draft, or download the file.
13. Studio examines each local asset when the option changes from `on` to `off`, and at each start with `off`. It removes the local bytes of each asset that a host has with the same SHA-256. The inventory of ADR 0036 items 6 and 7 gives the copies. Each other asset stays, as item 12 says.

### Downloads

14. With `off`, the server has no "Download All", no "Download" into the library, and no "Scan the Library Folder". ADR 0038 items 6 to 10 and items 13 to 15 do not apply.
15. Each file of the Files page has "Download to This Device". The UI gets a media ticket (ADR 0062 item 29) for the file, and the browser saves the response. The server reads the local bytes or the copy on its host, and it sends them with no copy on the disk.
16. To keep a full local library, the user uses Studio desktop with "Download All". The user guide says this.

### The Interface

17. With `off`, the Files page starts with the notice "This server keeps no audio and no images." Settings has the section "Files on this server", in the tier `limits` (ADR 0071). It gives the rule for audio and images, the rule for the feed XML, drafts and versions, and the space in use.
18. At 760 px or less, the status of a file is on a second line of its row, below the name. The status is the most important information of the row.
19. The review sheet tells the user that Studio removes its copies of the sent files after the write.

### Limits

20. The storage limit of ADR 0055 item 21 counts the pending files, the files that no host has, the temporary files and the databases.

## Alternatives Considered

- A staging path on the host, with the upload before the diff. Not chosen by the operator. It writes to a host with no diff that the user saw, so it needs a change of ADR 0007. It also leaves files that no feed uses on the hosts.
- No option, and no asset on a server at all. Not chosen by the operator. A personal server would then have no library.
- Audio only, with images kept. Not chosen by the operator. One rule for each asset is simpler for the user.
- A removal of each local asset at the change. Not chosen by the operator. Studio would then remove the only copy of a file that no host has.
- The removal of the track from the draft at the expiry. Rejected: Studio would change a draft when the user did not tell it to.

## Consequences

- The data folder of a server holds the databases, the feed XML, the vault files and a small number of pending files. The backup of the data is small, and the root disk can hold a copy each night.
- A server user has no master copy on the server. When a host loses a file, the copies of the user are the only copies.
- A move between hosts reads the file from the source host each time. This is slower, and an S3 or R2 host can charge for the read.
- The core gets the removal after a write, the expiry, and the removal at the start. The API gets the download route of item 15. The UI gets the states of the Files page, the notice, the Settings section and the narrow status line.
- A new task gives the work, after this ADR is accepted.

## Acceptance Criteria

Mechanical:

- A core test: after a write, each sent asset has `local` false, its SHA-256 and its `source_url`, and its bytes are not in the release folder.
- A core test: a pending file older than 7 days has no bytes, the draft keeps its address, and the write refuses with `file_missing`.
- A core test: at a start with `off`, Studio removes the bytes of an asset with a host copy of the same SHA-256. An asset with no host copy keeps its bytes.
- A core test: a move between hosts with `off` leaves no file in `incoming/`, also after a failure of the upload.
- An API test: with `off`, the download routes of ADR 0038 and the folder scan give 403 `not_on_server`.
- An API test: the route of item 15 sends the bytes of a file from its host, and writes no file.
- A Node test: the Files page model gives the status of each state of the mockups.
- With `on`, the tests of ADR 0035, ADR 0036 and ADR 0038 pass with no change.

Visual: the seven screens of the mockups, in the two themes, at 1100 px and at 390 px with touch.

Human, on `studio.musicindex.org`: the operator adds a track from a phone, writes the draft, and sees that the server has no copy of the audio. The operator then saves the file with "Download to This Device".

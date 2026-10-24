# ADR 0038: Published Feeds, Downloads, The Folder Scan And Feed Moves

- Status: Accepted
- Date: 2026-10-24
- Owner: operator, with the technical choices of the planning model

## Context

Milestone 7b of the [Studio v1 phase plan](../plans/studio-v1-phase-plan.md) implements [ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) items 5 to 8, 18 to 21 and 23. It uses the records of [ADR 0036](0036-library-records-host-inventory-and-host-operations.md).

After milestone 7a, a library file always has its bytes in the library. A published feed has only its feed XML in its release folder. Its audio files and images are on their hosts, and the collection has no record of them.

On 2026-10-24 the operator answered four questions:

- The announcement of a new feed address (ADR 0035 item 21) waits for Phase 10.
- The Files page offers the downloads. Nothing starts without a selection of the user.
- For a changed file in the library folder, Studio offers "Use the new file" and "Restore".
- A copy or a move of a file that is not in the library asks first. Studio then downloads the file, and the copy goes into the draft.

The planning model added the technical choices below.

## Decision

### Remote Library Files

1. Each address of a feed with a release folder is a library file: each enclosure, each alternative enclosure source and each image. Each address that has no library file gets a row with `local` false and no SHA-256. The `source_url` of the row is the address.
2. The path of a remote file in its release folder is the last segment of its address, with the percent encoding decoded. When that path is in use, Studio adds ` 2`, ` 3` and so on before the extension. The size is the enclosure length when the feed gives one, otherwise 0. The media type is the enclosure type, otherwise the type of the extension.
3. Studio records the remote files at these times: when the user adds a published feed, after each read of the hosted file and after each write. At the start, it records them for each feed with a release folder. A remote row whose address is no longer in the hosted feed goes out, because it has no bytes.
4. A published feed gets its release folder and its feed XML when the user adds it, as in ADR 0035 item 5. This applies to a read-only feed and to a feed that a folder match links.
5. A copy of a remote file is each host file with the `source_url` as its public address. When Studio knows the SHA-256, each host file with the same SHA-256 is also a copy.

### Downloads

6. The Files page shows the number and the total size of the files that are not in the library, with "Download all". Each remote file has its own "Download". No download starts without a selection of the user.
7. Studio downloads two files at a time for all feeds. The Files page shows the progress of each file, with "Stop".
8. When a connected host has a copy of the file, Studio reads it through the connector. Otherwise it uses HTTP. An HTTP download writes into a hidden partial file in the release folder. After a stop or a restart, it continues from the end of the partial file with a range request.
9. The collection records each download that is not complete. At the start, Studio continues each one.
10. Studio refuses a download when the size is more than `download_max_file_bytes`, or when the free space after the download is less than `download_min_free_bytes` ([ADR 0036](0036-library-records-host-inventory-and-host-operations.md) item 17). The error names the setting.
11. After a download, the row gets `local` true, its size and its SHA-256. When the size is different from the enclosure length in the feed, the Files page shows the two values. Studio does not change the feed for it.
12. A copy or a move of a remote file asks the user first, with the size of the download. Studio then adds the draft operation and starts the download. The diff marks the operation as "Waiting for the download", and the write refuses the draft until each download of the draft is complete.

### The Folder Scan

13. A library file records a stamp: its size and its modification time. The folder scan compares each release folder with the records, with no read of a file whose stamp did not change.
14. The scan gives these differences:
    - New: a file in a release folder with no row. The feed XML, hidden files and partial files are not differences.
    - Missing: a row with `local` true and no file.
    - Changed: a file whose stamp and SHA-256 are different from its row.
15. Studio scans at the start and when the user asks. It changes nothing without a decision of the user. The actions are:
    - New: "Add to the library" gives a library file that the feed does not use.
    - Missing: "Download again" when a copy or a `source_url` is known. "Forget the file" sets `local` false, or removes the row when Studio knows no copy and no address.
    - Changed: "Use the new file" records the new size, stamp and SHA-256. Each address of the file in the draft changes to its planned address, which is the hash name of ADR 0035 item 16 when a different file is at the clean path. Each copy of the bytes from before that the draft referred to gets a deletion in the draft. "Restore" downloads the recorded bytes from a copy, and puts them back into the release folder.

### A Move Of The Feed XML

16. The Files page has "Host of the feed XML". A different host puts a feed move into the draft. The feed XML keeps its path on the new host, so the files of the release keep their paths too ([ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) item 13).
17. `Connector` gets `can_redirect()` and `set_redirect(path, to_url)`. A connector that cannot make an HTTP 301 redirect gives false. The local folder connector cannot.
18. The diff shows the new address, and the method at the address from before: a redirect, or a pointer file with `itunes:new-feed-url`. It also says that Studio does not tell the musicindex API and Podcast Index yet.
19. The write puts the uploads and the copies, then the feed XML on the new host with its new self link. Then it sets the redirect, or writes the pointer file at the address from before. Then it records the new host, path and address of the feed, then it does the deletions.
20. The collection records each pointer file. Studio refuses a deletion of a pointer file, as ADR 0035 item 20 says.

### Announcement

21. The announcement of ADR 0035 item 21 waits for Phase 10, which adds it to the same diff. Until then, ADR 0035 item 21 is not in force.

## Alternatives Considered

- Announce the new address in milestone 7b. Not chosen by the operator: Phase 10 owns announcements.
- A sheet after the user adds a feed, or downloads that start at once. Not chosen by the operator.
- Show a changed file with no action, or with "Use the new file" only. Not chosen by the operator.
- Disable Copy and Move until the user downloads the file. Not chosen by the operator.
- An empty SHA-256 text for a remote file. Rejected: an empty text can agree with a different empty text, and a missing value cannot.
- Downloads that run only while the page is open. Rejected: ADR 0035 item 8 says that the downloads continue after a stop.
- A read of each library file at each scan. Rejected: a library of many audio files takes minutes to read.

## Consequences

- The SHA-256 of a library file becomes optional, and each use of it handles a remote file.
- The API keeps a download queue with two workers.
- Each connector of Phase 9 tells if it can make a redirect.

## Invariants

- No download starts without a selection of the user.
- The write refuses a draft with an operation that waits for a download.
- Studio does not delete a pointer file.
- The scan reads only a file whose stamp changed.
- No upload, copy, redirect, pointer file or deletion occurs without a diff that the user saw.

# Phase 7 Review Checklist

- Date: 2026-10-23
- Reviewed: each packet of [task 021](../tasks/studio-v1-task-021-release-library-7a.md) and of [task 023](../tasks/studio-v1-task-023-release-library-7b.md)
- Decisions: [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md), [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) and [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md)

The planning model uses this list after each packet, before its commit.

## Each Packet

- The diff changes only the files that the packet names.
- The three commit checks pass.
- Each new code comment is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the packet does not tell the coding model to do.
- For a UI change: the `drive-ui.mjs` check passes, and the planning model examines its screenshots.

## Invariants

- No upload, copy or deletion on a host occurs without a diff that the user saw ([ADR 0007](../adr/0007-draft-diff-write-and-announce.md)).
- Studio does not write over a file on a host ([ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md) item 16).
- Studio does not delete a host file while a feed of the library refers to its address (ADR 0035 item 17).
- A deletion uses the stamp from the inventory (ADR 0036 item 16).
- The library XML has the same bytes as the hosted feed file after each write (ADR 0035 item 9).
- A read and a write with no edit give the same bytes ([ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md)).
- Each `/api/` route needs the access token ([ADR 0017](../adr/0017-local-api-access.md)).
- A schema change is a new migration ([ADR 0014](../adr/0014-collection-in-sqlite.md)).
- No download starts without a selection of the user (ADR 0038 item 6).
- The write refuses a draft with an operation that waits for a download (ADR 0038 item 12).
- Studio does not delete a pointer file (ADR 0038 item 20).
- The folder scan reads only a file whose stamp changed (ADR 0038 item 13).

## Results

| Work | Result | Required corrections |
| --- | --- | --- |
| [021a](../tasks/studio-v1-task-021a-records-and-library-folder.md) | Pass after two corrections | (1) `put_host_file` cleared the SHA-256 at each scan, also when the version did not change, so task 021b would read each file again. It now keeps the SHA-256 for the same version, and a test covers it. (2) `put_file` and `write_feed_xml` did not check the folder name, so a folder `..` went out of the library. The folder name now has the check of a file name, and a test covers it. |
| [021b](../tasks/studio-v1-task-021b-inventory-and-addresses.md) | Pass after one correction | `Connector::list` gave only paths, so the coding agent read each file in the scan to get its size and version. On a large host, each scan then downloads each audio file. The planning model changed the connector interface: the list gives the size and a stamp of each file, `stat` gives them for one file, and `delete` checks the stamp. ADR 0036 items 6 and 16 record it, and a test refuses each read in a scan. |
| [021c](../tasks/studio-v1-task-021c-library-import-and-diff.md) | Pass after one correction | The diff gave an upload to the host of a library file also when the draft referred only to a copy on a different host. After a reference to a copy on host B, Studio then put a second copy on host A. Only the planned URL now gives an upload. The planning model replaced the test, whose comment said that a review had approved the rule from before. 021c changes the signature of `diff`, so 021c and 021d have one commit. |
| [021d](../tasks/studio-v1-task-021d-write-with-host-operations.md) | Pass after one return to the agent | The first result did not change the import route. An imported file then went into the pending store, which the new write does not upload, so the feed referred to an address with no file. The agent changed the import route to `import_into_library`, the removal of a file to its library ID, and `Hosts` to leave out a host that needs the locked vault. It restored the test of ADR 0034. The planning model accepted two choices of the agent: a feed gets its release folder at its first import, and `vault_locked` keeps its status 423. |
| [021e](../tasks/studio-v1-task-021e-move-into-the-library.md) | Pass after one return to the agent | (1) A feed got its folder in the collection before its move, so a failed move was not tried again. The folder is now recorded after the last step, and a test tries a failed feed again. (2) `Diff.files` was always empty, so the review sheet showed no file. It now has the uploads until task 021g. (3) `diff` kept a connector parameter with no use. The agent removed it. The planning model accepted that `into_library` drops `draft_files` after a run with no error, not in an automatic migration, because a migration would drop the rows before the move. |
| [021f](../tasks/studio-v1-task-021f-library-api.md) | Pass after one correction | The diff did not give the ID of a draft operation, so the UI could not remove one, and the test read the ID from the database. The planning model added `operation_id` to each copy, deletion and blocked deletion of the diff, and the test now reads it from the diff. The planning model accepted that a `copy` operation gives an operation of the kind `copy`, not `upload`. |
| [021g](../tasks/studio-v1-task-021g-library-ui.md) | Pass after three corrections in the API | The planning model examined the screenshots of the Files page and of the review sheet. (1) A copy or a deletion with no XML change showed "No changes" after a reload, because only the browser knew it. `draft_changed` now counts the draft operations. (2) "Reference" replaced only the planned address of the file, so a feed that referred to a copy on a third host did not change. It now replaces each address of the file. (3) A reference to a host with no copy of the file gave the feed an address with no file. The reference now adds a copy. A new API test covers (1) and (3). The coding agent kept the Files page in `editor.js`, because a new UI file needs a change of `ui.rs`. |
| [022a](../tasks/studio-v1-task-022a-api-marks-and-host-address.md) | Pass | The coding agent added `Collection::set_connector_name`, because the route changes the name of a host and no function did it. The planning model accepted it. |
| [022b](../tasks/studio-v1-task-022b-button-roles-and-files-page.md) | Pass after two corrections | The usage limit stopped the coding agent once, and the agent continued with its context. The planning model examined the screenshots in the dark and the light theme. (1) "Host of this file" compared the resolved host with the default host, because the files route gave no own host. The route now gives `own_connector_id`, and a test covers it. (2) The review sheet took back an operation with a red "Remove", and the Files page with a normal "Undo". Both are now "Undo" with the normal role. |
| [023a](../tasks/studio-v1-task-023a-remote-file-records.md) | Pass after one correction | The coding agent turned the foreign keys off while the migrations run, because the drop of the old `library_files` table deleted the draft operations that refer to it. A test covers it. The planning model accepted it. The planning model added the length of an alternative enclosure to the size of a remote file, because packet 023d compares the size with the length in the feed. |
| [023b](../tasks/studio-v1-task-023b-published-feeds-in-the-library.md) | Pass after one correction | The planning model examined the screenshot of the Files page. `length_in_feed` read only the enclosure length, but packet 023a gives a remote alternative source the length of its alternative enclosure. The route now reads both. The planning model accepted four choices of the coding agent: `Found` keeps the bytes of the feed, a read-only feed does not reserve its folder name before its fetch, a failed folder step does not stop the add of a feed, and a helper gives a linked feed its folder. "Host of this file" stays for a remote file until packet 023d. |
| [023c](../tasks/studio-v1-task-023c-download-queue.md) | Pass after three corrections | (1) `finish` read the SHA-256 of the file while the worker held the workspace lock, so a large download stopped each other request for some seconds. `download::hash_part` now reads it with no lock, and `finish` only renames and records. (2) The files route gave `unknown_host` for a downloaded file of a feed with no host. A file with no connected host now has no planned address, and an API test covers it. (3) With no draft, the files route compared with no feed, so each file of a read-only feed was "Not in use". The route now compares with the hosted feed in the release folder. The planning model accepted these choices of the coding agent: a test server for range requests, `digest::sha256_file`, a POST that queues nothing when one file is over a limit, and a download client with no global timeout. Open item: ureq has no idle timeout for a body, so a connection that stops in a body can keep a worker until the connection closes. |

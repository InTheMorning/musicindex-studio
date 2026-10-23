# Phase 7 Review Checklist

- Date: 2026-10-23
- Reviewed: each packet of [task 021](../tasks/studio-v1-task-021-release-library-7a.md) and of milestone 7b
- Decisions: [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md) and [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md)

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

## Results

| Work | Result | Required corrections |
| --- | --- | --- |
| [021a](../tasks/studio-v1-task-021a-records-and-library-folder.md) | Pass after two corrections | (1) `put_host_file` cleared the SHA-256 at each scan, also when the version did not change, so task 021b would read each file again. It now keeps the SHA-256 for the same version, and a test covers it. (2) `put_file` and `write_feed_xml` did not check the folder name, so a folder `..` went out of the library. The folder name now has the check of a file name, and a test covers it. |
| [021b](../tasks/studio-v1-task-021b-inventory-and-addresses.md) | Pass after one correction | `Connector::list` gave only paths, so the coding agent read each file in the scan to get its size and version. On a large host, each scan then downloads each audio file. The planning model changed the connector interface: the list gives the size and a stamp of each file, `stat` gives them for one file, and `delete` checks the stamp. ADR 0036 items 6 and 16 record it, and a test refuses each read in a scan. |
| [021c](../tasks/studio-v1-task-021c-library-import-and-diff.md) | Pass after one correction | The diff gave an upload to the host of a library file also when the draft referred only to a copy on a different host. After a reference to a copy on host B, Studio then put a second copy on host A. Only the planned URL now gives an upload. The planning model replaced the test, whose comment said that a review had approved the rule from before. 021c changes the signature of `diff`, so 021c and 021d have one commit. |
| [021d](../tasks/studio-v1-task-021d-write-with-host-operations.md) | Pass after one return to the agent | The first result did not change the import route. An imported file then went into the pending store, which the new write does not upload, so the feed referred to an address with no file. The agent changed the import route to `import_into_library`, the removal of a file to its library ID, and `Hosts` to leave out a host that needs the locked vault. It restored the test of ADR 0034. The planning model accepted two choices of the agent: a feed gets its release folder at its first import, and `vault_locked` keeps its status 423. |
| [021e](../tasks/studio-v1-task-021e-move-into-the-library.md) | Pass after one return to the agent | (1) A feed got its folder in the collection before its move, so a failed move was not tried again. The folder is now recorded after the last step, and a test tries a failed feed again. (2) `Diff.files` was always empty, so the review sheet showed no file. It now has the uploads until task 021g. (3) `diff` kept a connector parameter with no use. The agent removed it. The planning model accepted that `into_library` drops `draft_files` after a run with no error, not in an automatic migration, because a migration would drop the rows before the move. |

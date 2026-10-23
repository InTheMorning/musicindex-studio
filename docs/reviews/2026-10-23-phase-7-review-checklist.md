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

# ADR 0179: A Release Keeps One Library Record For Each File

- Status: Accepted
- Date: 2026-11-12
- Owner: operator
- Accepted: 2026-11-12 by the operator, through the questionnaire of the human checks.
- Technical choices: items 1, 2, 4 and 6 are choices of the planning model.
- Changes: [ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md) item 1. A host that Studio cannot read no longer gives a remote row.
- Uses: [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) items 7 and 12, [ADR 0148](0148-a-move-keeps-the-old-files-until-the-user-reviews-the-deletion.md) items 2 to 5, [ADR 0177](0177-the-files-page-has-one-way-to-move.md) item 3.
- Source: finding 9 of the [human checks of 2026-11-11](../reviews/2026-11-11-local-human-test-findings.md).

## Context

On 2026-11-12, a move of a test release back to its first file host showed duplicate uploads with " 2" names in Review. The release had two library records for each of its four media files, with the same SHA-256.

`record_remote_files` adds a remote row for each feed address that it does not know. It knows an address from the planned address and from the copies of each local library file. It ignored an error in these reads, and it then treated the address as a new address. At 11:24:46 UTC a read of the inventory of `heycitizen` failed. The function then added a second record for each `heycitizen` address, with a free name.

The move plan also made an upload of one file to a host that had a copy of it. The copy had the same SHA-256 and was at the target path. The write gives `FileExists` for such an upload.

## Decision

### No Record From A Failed Read

1. Studio records the remote files of a feed. It reads the planned address and the copies of each local file. When one of these reads fails, Studio adds no remote row for that feed in that pass. It writes the error to the log. The next record time of ADR 0038 item 3 tries again.
2. A feed address that is not on a connected file host still gets a remote row, as ADR 0038 item 1 gives.

### One Record For One File

3. Two library records of one release with the same SHA-256 and the same size are one file. Studio keeps the older record and removes the newer record. It also removes the file of the newer record from the release folder when that file has the same SHA-256.
4. Studio does this at each record time of ADR 0038 item 3, and at the start. It does not do this while a draft operation names the newer record. The Files page then shows the two records, as before.

### A Move Uses A Copy On The New Host

5. When the new host of a move has a copy of a file (ADR 0036 item 7), the move makes no upload of that file. The draft uses the address of that copy (ADR 0036 item 12). The deletion choice of ADR 0148 item 5 applies to the old copies of the file.
6. Review shows this file in the row of the files that are on the new host.

## Alternatives Considered

- Fix only the cause, and let the user remove the duplicate records on the Files page. Not chosen: the operator said that Studio must do this work. The user must not clean up after a defect of Studio.
- Merge records with a draft operation, and change the operation and the draft addresses. Not chosen: the change of a draft that the user has not examined is a larger risk than a short wait. The user can discard the draft.

## Consequences

- A collection that has duplicate records gets one record for each file again, with no work of the user.
- A remote row can come one record time later when a host does not answer.
- A move to a host with a copy sends fewer bytes.

## Acceptance Criteria

Mechanical:

- A Rust test: `record_remote_files` adds no row when the copies read of a local file fails. It adds the row at the next call when the read succeeds.
- A Rust test: two records with the same SHA-256 and the same size become one. The older record stays. The file of the newer record is not in the release folder. With a draft operation on the newer record, the two records stay.
- A Rust test: a move plan to a host with a copy of a file gives no copy operation for that file. The plan counts the file on the target. It gives a hold operation for each old copy.

Visual, by the planning model:

- None. The Review rows are the current rows.

Human, by the operator. This gate stays open until it runs:

- In the desktop app, discard the draft of "HeyCitizen's Test Album" and restart. The **Files** page shows no " 2" names. Then move "The feed file and the release files" to `bastone`. Each media file has a copy on `bastone`, so Review shows no upload of a media file and no spare copy. The feed uses the address of each copy on `bastone`.

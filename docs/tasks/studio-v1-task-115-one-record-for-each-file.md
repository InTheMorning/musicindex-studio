# Studio v1 Task 115: A Release Keeps One Library Record For Each File

Decision: [ADR 0179](../adr/0179-a-release-keeps-one-library-record-for-each-file.md).

Status: Complete on 2026-11-12. The four commit checks are green. A move to a host with a copy of a file adds a copy operation at the path of that copy. The write makes no host operation of it, and it removes the record of an old copy at that path (ADR 0148 item 13). The human gate passed on 2026-11-12: the merge removed the duplicate records, and the move back to `bastone` sent no upload.

## Goal

Studio adds no duplicate library record when a read of a file host fails. Studio merges the duplicate records that a collection has. A move uses a copy of a file that is on the new host, with no upload.

## Constraints

- Each rule is in the core or the API. The UI shows the rows that the API gives (ADR 0065, ADR 0042).
- No test sends a request outside the computer (ADR 0045). Use the test connectors.
- Keep ADR 0038 item 1 for an address that is not on a connected file host.
- Do not change a draft operation or a draft address in the merge (ADR 0179 item 4).

## Files To Read First

- ADR 0179, and ADR 0038 items 1 to 5.
- `record_remote_files` in `crates/core/src/remote.rs`.
- `copies` and `host_sha` in `crates/core/src/inventory.rs`.
- `release_move_plan`, `apply_release_move` and `reference_url` in `crates/api/src/routes.rs`.
- `record_remote_files_with` in `crates/api/src/workspace.rs`, and the start of the API, for item 4.

## Packets

### 115a: No Record From A Failed Read

ADR 0179 items 1 and 2.

- In `record_remote_files`, give up the pass when `inventory::planned_path` or `inventory::copies` fails for a local file. Do not stop on `NoHost`: a file with no host has no planned address. Write the error to the log in the caller.
- Change the doc comment of the function.
- Add the Rust test of the ADR. A test connector that fails a read gives the failure.

### 115b: The Merge Of Duplicate Records

ADR 0179 items 3 and 4.

- Find each pair of records in one release with the same SHA-256 and the same size. Keep the record with the lower `added_at`, then the lower `id`.
- Skip the pair when a draft operation names the newer record.
- Remove the newer record.
- Remove the file of the newer record from the release folder. Do this only when the file has the SHA-256 of the record.
- Run the merge at each record time of ADR 0038 item 3 and at the start.
- Add the Rust test of the ADR.

### 115c: A Move Uses A Copy On The New Host

ADR 0179 items 5 and 6.

- In `release_move_plan`, a file with a copy on the target gives no `FileMove`. When the feed uses an old copy of the file, the plan puts a reference to the copy on the target in the draft. The plan also keeps the hold operations of the old copies when the user selected the deletion.
- Count the file in `on_target`.
- Add the Rust test of the ADR.

## Acceptance Criteria

Mechanical: the criteria of ADR 0179.

Visual: none.

Human, by the operator. This gate stays open until it runs:

- The human criterion of ADR 0179.

## Stop And Ask When

- A rule of ADR 0038 or ADR 0148 conflicts with ADR 0179.
- The merge must change a draft to complete.

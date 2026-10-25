# Task 028 Review Checklist

- Date: 2026-10-25
- Reviewed: each packet of [task 028](../tasks/studio-v1-task-028-s3-connector.md)
- Decisions: [ADR 0046](../adr/0046-the-s3-and-r2-connector.md), [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) and [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md)

The planning model uses this list after each packet, before its commit.

## Each Packet

- The diff changes only the files that the packet names.
- The four commit checks pass.
- Each new code comment is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the packet does not tell the coding model to do.
- For a UI change: the `drive-ui.mjs` check passes, and the planning model examines its screenshots in the dark theme and in the light theme.

## Manual Checks

- 028a: the fake server gives 412 for each condition of the operations table. A test that sends a real request examines each one.
- 028b: `write_from` reads the reader in parts, and an aborted multipart upload leaves no open upload.
- 028b and 028c: no access key is in a log, an error message, an API response, the collection row or the `Debug` text.
- 028c: a failed provider test changes neither the vault nor the collection.

## Invariants

- The connector writes only with a condition on the host ([ADR 0046](../adr/0046-the-s3-and-r2-connector.md)).
- A write to a host occurs only through `pipeline::write_draft` ([ADR 0007](../adr/0007-draft-diff-write-and-announce.md)).
- No test sends a request to a host outside the computer ([ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md)).
- The core has no async runtime ([ADR 0015](../adr/0015-connector-interface-and-write-pipeline.md)).

## Results

| Work | Result | Required corrections |
| --- | --- | --- |
| [028a](../tasks/studio-v1-task-028a-fake-s3-server.md) | Pass | The planning model replaced the MD5 constants from `sin()` with the fixed table of RFC 1321. The precision of `sin` can be different on a different platform. The planning model changed `allow` to `expect` with a reason. The manual check passes: a test with a real request examines each condition that gives 412. |
| [028b](../tasks/studio-v1-task-028b-s3-connector.md) | Pass | The planning model removed each address from the text of a connection error. A presigned address holds the access key ID. A new test examines this. The planning model accepted these choices of the coding agent. A 404 on a write with `If-Match` gives `Conflict`. A file of 64 MiB or less goes into memory for one PUT, so the memory has a limit of 64 MiB for each upload. |
| [028c](../tasks/studio-v1-task-028c-provider-test-and-route.md) | Pass | No correction. The planning model accepted these choices of the coding agent. `apply_folder_address` gets the vault, because the match route makes the connector of an S3 host. The API crate gets `uuid`, which the core already uses. When the vault save or the new row fails, the route removes the secret from the vault again. The manual check passes: a failed provider test changes neither the vault file nor the collection. |
| 028d | Open | |

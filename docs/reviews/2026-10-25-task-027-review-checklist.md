# Task 027 Review Checklist

- Date: 2026-10-25
- Reviewed: each packet of [task 027](../tasks/studio-v1-task-027-connector-interface.md)
- Decisions: [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md), [ADR 0015](../adr/0015-connector-interface-and-write-pipeline.md) and [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md)

The planning model uses this list after each packet, before its commit.

## Each Packet

- The diff changes only the files that the packet names.
- The four commit checks pass.
- Each new code comment is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the packet does not tell the coding model to do.
- For a UI change: the `drive-ui.mjs` check passes, and the planning model examines its screenshots in the dark theme and in the light theme.

## Manual Checks

- 027a: no error message, log line or API response contains a secret from the vault ([ADR 0003](../adr/0003-user-unlocked-secret-vault.md)).
- 027a: `put_uploads_and_copies` does not read a full library file into memory.
- 027b: each check of `suite.rs` uses only the functions of the `Connector` trait. A test cannot find where a check uses a different type, so the planning model reads each check.
- 027c: `confirm_new_addresses` calls no `read`, and no deletion occurs before it passes.

## Invariants

- A write to a host occurs only through `pipeline::write_draft`, with the ID of the diff that the user approved ([ADR 0007](../adr/0007-draft-diff-write-and-announce.md)).
- A connector gets its secrets from the vault only ([ADR 0006](../adr/0006-connectors-and-the-v1-set.md)).
- A read and a write with no edit give the same bytes. The corpus tests pass ([ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md)).

## Results

| Work | Result | Required corrections |
| --- | --- | --- |
| [027a](../tasks/studio-v1-task-027a-trait-changes.md) | Pass | The planning model removed two copies of the hexadecimal text code in `digest.rs`. Both functions now use `finish_hex`. The planning model accepted two choices of the coding agent. `ConnectorError::new` is now public, so that the API test can make an error. A vault read error in `secret_of` gives `InvalidConfig`, with a message that holds no secret. |
| [027b](../tasks/studio-v1-task-027b-trait-test-module.md) | Pass | The planning model removed an assertion that `write_from` and `write` give the same version. The packet asks only for the same bytes, and an S3 multipart upload gives a different ETag for the same bytes. The planning model changed `allow` to `expect` with a reason for `clippy::missing_panics_doc`. The manual check passes: each check uses only the trait, and the suite reads no file. LocalFolder gives no SHA-256 and can make no redirect, so two checks assert nothing for it. |
| 027c | Open | |

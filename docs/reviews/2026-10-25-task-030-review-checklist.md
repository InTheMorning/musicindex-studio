# Task 030 Review Checklist

- Date: 2026-10-25
- Reviewed: each packet of [task 030](../tasks/studio-v1-task-030-nsite-and-blossom.md)
- Decisions: [ADR 0048](../adr/0048-the-nsite-and-blossom-connectors.md), [ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md), [ADR 0044](../adr/0044-the-nsite-signer.md), [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) and [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md)

The planning model uses this list after each packet, before its commit.

## Each Packet

- The diff changes only the files that the packet names.
- The four commit checks pass.
- Each new code comment is in Simplified Technical English and agrees with the code.
- The diff has no cleanup that the packet does not tell the coding model to do.
- For a UI change: the `drive-ui.mjs` check passes, and the planning model examines its screenshots in the dark theme and in the light theme. A step that reads the page after a click has its wait on the same step as the read.

## Manual Checks

- 030a: `cargo tree -e features -p studio-core` shows no `nip44`, `nip46`, `rand` or `os-rng` feature of `nostr`. `cargo tree -d -p studio-core` shows one version of `secp256k1`. `sign_with_keys` is the only code that signs with a key. The licenses of the new crates are MIT, Apache-2.0, ISC, BSD or CC0. No thread of a test server stays after the tests.
- 030b: the relay client drops each event with a bad ID or a bad signature. No error message contains a secret key.
- 030c: the local folder, S3 and SFTP tests pass with no change of their assertions. `finish` comes after the feed XML and before `record_state`.
- 030d: Studio sends no `DELETE` for a blob that a different record of the host names. No temporary spool file stays after a write.
- 030e: a `Conflict` in `finish` publishes nothing. A delete sends no `DELETE` to a Blossom server.
- 030f: with "My signer", a write with a missing or wrong event sends no request to a test server.
- 030d, 030e and 030g: no secret key is in a log, an error message, an API response, the collection row or the `Debug` text.
- 030g: a failed connection changes neither the vault nor the collection. The lookup route sends requests only to the lookup relays.
- 030h: the UI loads nothing from other servers. The signer step works from the keyboard. No secret key is in the browser storage.

## Invariants

- Studio publishes no event that a key other than the key of the host signed ([ADR 0044](../adr/0044-the-nsite-signer.md)).
- A write with "My signer" changes nothing on a host before it has each planned event with a correct signature ([ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md)).
- Studio does not delete a blob that a manifest or a kept record of the host refers to ([ADR 0048](../adr/0048-the-nsite-and-blossom-connectors.md), [ADR 0050](../adr/0050-nsite-and-blossom-signer-environment-and-plan.md)).
- A write to a host occurs only through `pipeline::write_draft` ([ADR 0007](../adr/0007-draft-diff-write-and-announce.md)).
- No test sends a request to a host outside the computer ([ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md)).
- The core has no async runtime ([ADR 0015](../adr/0015-connector-interface-and-write-pipeline.md)).

## Results

| Work | Result | Required corrections |
| --- | --- | --- |
| 030a | Pass, 2026-10-25 | The unit test of `sign_with_keys` compares the event IDs (the planning model added it). |
| 030b | Pass, 2026-10-25 | None. `Reads::into_newest` gives the `Unavailable` rule of `newest`. |
| 030c | Pass, 2026-10-25 | None. When a first `finish` fails on one host, a host before it in the sequence keeps its new blob records, because its `finish` published them. |
| 030d | Pass, 2026-10-25 | `finish` removed the planned events before the deletions. The coding model removed that `finish` and added a test. Open: a write over an existing path does not delete the old blob from the servers. |
| 030e | Pass, 2026-10-25 | The planning model removed a second `Expected` check from `write`. A plan with deletions and no writes has only the manifest after the deletions. Open: one path with no blob on a server makes `list` fail. A `HEAD` answer with no `Content-Length` looks like a missing blob. |
| 030f | Pass, 2026-10-25 | The planning model removed a semicolon from the `PlanExpired` message. A refused write reads the relays in `plan_events`, but it sends no event and no blob. The API crate gets `nostr` as a dev dependency for its tests. |
| 030g | Open | |
| 030h | Open | |

# ADR 0007: Draft, Diff, Write And Announce

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

An incorrect feed reaches listeners and indexes quickly.
A hosted file can change outside Studio, for example by a different tool or by hand.
Stophammer reads podpings. It has no route that sends a podping for an editor.

## Decision

1. The user edits a draft. A draft does not change the hosted file.
2. Before a write, Studio shows a diff between the draft and the live file.
3. Studio writes only if the live file has not changed since it was read ([ADR 0006](0006-connectors-and-the-v1-set.md)). If it changed, Studio shows the conflict and does not write.
4. Before each write, Studio keeps a snapshot of the live file. The user can roll back to a snapshot.
5. After a write, Studio sends a podping and a Podcast Index pubnotify. A setting turns each one off.

## Alternatives Considered

- Each save writes the live file. Rejected: an error reaches listeners with no review.
- Manual announce buttons only. Rejected: artists forget to announce, and indexes then show old data.

## Consequences

- Studio needs a podping sender. The method (its own Hive account, or a hivepinger service) is not decided.
- Snapshots use storage. A retention limit is necessary.

## Invariants

- No write occurs without a diff that the user has seen.
- A snapshot is not changed after it is made.

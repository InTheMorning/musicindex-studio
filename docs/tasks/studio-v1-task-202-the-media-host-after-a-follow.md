# Studio v1 Task 202: The Media Host After A Follow

Decision: [ADR 0229](../adr/0229-the-media-host-after-a-follow-a-podping-again-and-empty-folders.md) items 1 to 3.

Status: Open.

## Problem

Finding 1 of the [rc.29 human check](../reviews/2026-11-20-rc29-human-check.md). **Follow the Move** and **Use This File Host** call `set_feed_location`. That function keeps the old release files host (`files_connector_id`). Thus the tab **Files** names a host that does not hold the media files.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Follow | extend | crates/core/src/outside.rs | `follow_move_read` near line 291 calls `set_feed_location` near line 312. |
| Use This File Host | extend | crates/api/src/routes.rs | `outside_use_host` near line 2128 calls `set_feed_location` near line 2149. It has the public address, not a read feed. |
| Host of an address | reuse | crates/core/src/outside.rs | `host_for_address` gives the connector and the path of an address. |
| Release files host | reuse | crates/core/src/collection.rs | `set_files_connector` near line 2996. |
| Feed model | reuse | crates/core/src | The parsed feed of `PublicFeed` (`read.feed`) has the items with their enclosure addresses. Find the field names in the feed model. |

## Changes

1. Add one core function, for example `media_host(connectors, feed) -> Option<ConnectorId>`. It gives the connector when `host_for_address` gives the same connector for each enclosure address of the feed. It gives `None` for a feed with no enclosure, or when an address has no host or a different host.
2. `follow_move_read`: after `set_feed_location`, when the function of change 1 gives a connector, call `set_files_connector` with it (item 1). With `None`, do not change the release files host (item 2).
3. `outside_use_host`: use the last read of the feed that the collection keeps (the feed of the last version or state), and apply the same rule. If the route cannot get the feed with no network read, keep the release files host and report this in the Result section.
4. Nothing writes to a file host (item 3).

## Checks

Mechanical:

- Core test: a follow to a host that gives each media address sets the release files host to that host.
- Core test: a follow when one media address is on no connected host keeps the release files host.
- `bash tools/gate.sh` passes.

Visual: none. The main session examines the tab **Files** with the desktop drive.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.

## Result

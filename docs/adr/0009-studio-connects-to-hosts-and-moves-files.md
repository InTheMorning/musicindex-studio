# ADR 0009: Studio Connects To Hosts And Moves Files Between Them

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

MSP 2.0 hosts feeds on its own Vercel Blob store. MSP 2.0 hosting is not a v1 connector ([ADR 0006](0006-connectors-and-the-v1-set.md)).
The mission of the project is self-hosting.
An artist can have feeds, audio and art on more than one host.

## Decision

1. Studio does not host files. It connects to hosts through connectors.
2. One optional connector is a basic self-hosted file host. Studio sets it up and configures it for the user. It is a convenience. The user can select a different host.
3. Each host is a module. No part of Studio depends on one host.
4. The user can move some or all files from one host to a different host. This includes feed files, audio and art.

## Alternatives Considered

- Studio hosts files for its users, as MSP 2.0 does. Rejected: Studio connects to hosts and does not become one.

## Consequences

- A connector writes media files and feed files.
- A move of a media file changes its URL. Studio updates each feed that refers to that file, through the write pipeline ([ADR 0007](0007-draft-diff-write-and-announce.md)).
- A move of a feed file writes `podcast:previousUrl` and `itunes:new-feed-url` ([ADR 0008](0008-feed-types-and-feed-identity.md)). Studio gives a redirect from the previous URL when the previous host can do it.
- The self-hosted file host obeys the same connector interface as each other host.

## Invariants

- A move does not remove a file from the previous host before the new feed is written and the new URLs are reachable.
- The feed GUID and the item GUIDs do not change in a move.

## Open Questions

- Does the self-hosted file host run in the desktop and server shells, or as a separate program?
- Which host types can give a redirect from a previous URL?
- Does the user remove files from the previous host after a move, or does Studio do it as an option?

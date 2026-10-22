# ADR 0008: Feed Types And Feed Identity

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

MSP 2.0 and Podtunes damage feed identity.
Podtunes makes a new feed GUID when the URL changes and keeps item GUIDs only in the browser.
MSP 2.0 gives a new feed a UUIDv4 GUID and makes new GUIDs on import.
MSP 2.0 does not model `podcast:valueTimeSplit`.

## Decision

1. The v1 feed types are:
   - album and single (`podcast:medium` music).
   - publisher feeds (`podcast:medium` publisher) with `remoteItem` and the `rel` role set from Podcast Namespace PR #793.
   - playlists (`podcast:medium` musicL).
   - live items (`podcast:liveItem`).
2. A new feed gets a UUIDv5 `podcast:guid` from its URL, as the namespace specifies. After that, the GUID does not change, also when the URL changes.
3. The collection keeps each item GUID. A rebuild or an import does not make new item GUIDs.
4. The core models `podcast:value` at channel and item level, with `customKey`, `customValue` and `fee` on each recipient. It also models `podcast:valueTimeSplit`.
5. The reader keeps each element that the core does not model. The writer writes it back. The core does not both model an element and pass it through.
6. A host move writes `podcast:previousUrl` and `itunes:new-feed-url`. Stophammer follows a move only through these elements, `atom:link rel="self"` or a redirect.

## Alternatives Considered

- Album feeds only in v1. Rejected by the operator.

## Consequences

- Each feed type needs its own editor screens.

## Invariants

- A read and a write with no edit give a feed with the same meaning as the source.
- The feed GUID and the item GUIDs do not change without a user action.

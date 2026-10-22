# ADR 0021: The Public Address Of A Feed

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

The collection keeps one public address for each feed ([ADR 0005](0005-collection-and-first-run-discovery.md)).
Discovery gives the address where Studio read a feed ([ADR 0020](0020-first-run-asks-one-question.md)). A local folder connector has an optional public address, and Studio adds the file path to it ([ADR 0019](0019-select-a-feed-from-the-host.md)).

On 2026-10-22 the operator connected a folder that holds four feeds. Three feeds had addresses from discovery. The fourth feed came from the folder, and it had no address, because the folder had no public address.
Studio does not compare these addresses. Two sources can give different addresses for one feed, and Studio does not show this.

Subsequent phases need a correct address for each feed. Phase 5 compares the local file with the live file. Phase 7 announces each write by the feed address.

## Decision

### Sources

1. A feed has one public address in the collection. Studio takes it from the first source in this list that gives one:
   1. The `href` of the first `atom:link` with `rel="self"` in the feed.
   2. The address where Studio read the feed: a pasted address, a search result, a publisher link or an MSP 2.0 backup.
   3. The public address of the folder connector, with the file path added.
2. The collection records which source gave the address.
3. When two sources give different addresses, Studio keeps the address of the first source. The collection records the difference. The UI shows it on the feed and on the host card.
4. Studio does not change a feed file because of a difference. The user decides.

### The Public Address Of A Folder

5. After a folder match, Studio examines each linked feed with a public address. When the address ends with the file path of the feed, the start of the address is a candidate. The candidate is a possible folder address.
6. When all candidates are the same, and the folder has no public address, Studio sets the folder address to that candidate. The UI tells the user that Studio found the address from the feeds.
7. When the candidates are different, Studio sets no folder address. The host card lists each feed with an address that does not agree with the folder.
8. Studio does not replace a folder address that the user typed.
9. A feed with no address from source 1 or 2 then gets its address from source 3.

## Alternatives Considered

- Use only the address where Studio read the feed. Rejected: a feed that Studio found in a folder has no such address.
- Use only the folder address. Rejected: a folder that is not organized in the same structure as the server gives incorrect addresses.
- Select one address silently when sources are different. Rejected: an incorrect address makes the announcements of Phase 7 fail without a sign.

## Consequences

- The collection schema gets the address source and the recorded difference. This is a new migration ([ADR 0014](0014-collection-in-sqlite.md)).
- A feed without `atom:link rel="self"` depends on discovery or on the folder address.

## Invariants

- Studio does not select silently between two different addresses for one feed.
- An inferred folder address does not replace an address that the user typed.

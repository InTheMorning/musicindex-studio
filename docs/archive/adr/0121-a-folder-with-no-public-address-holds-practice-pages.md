> Archived: superseded by [ADR 0122](../../adr/0122-a-new-user-starts-with-a-default-file-host.md) on 2026-11-02. ADR 0122 keeps the temporary GUID and the change of the links, and removes practice pages.

# ADR 0121: A Folder With No Public Address Holds Practice Pages

- Status: Superseded by ADR 0122
- Date: 2026-11-02
- Owner: operator
- Accepted: 2026-11-02 by the operator, from the choice "Practice mode" after the visual check of task 083.
- Amended: 2026-11-02 by the planning model, from the review of packet 085a: item 1 applies to a local folder only.
- Changes: [ADR 0032](../../adr/0032-new-releases-formats-paths-and-new-feeds.md) items 6 to 9, for a file host with no public address. [ADR 0119](../../adr/0119-a-new-user-starts-as-a-musician-or-a-label.md) item 6, the row "My own server".
- Uses: [ADR 0008](../../adr/0008-feed-types-and-feed-identity.md) for feed identity. [ADR 0007](../../adr/0007-draft-diff-write-and-announce.md) for the write. [ADR 0038](../../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) for a move after a write. [ADR 0041](../../adr/0041-publisher-feeds-and-two-way-links.md) items 6, 7 and 12 for the links.

## Context

- ADR 0119 item 6 offers "A folder on this computer, to try Studio first". The visual check of task 083 on 2026-11-02 found that a folder with no public address cannot make a page. The core gives `no_public_address`.
- The GUID of a new feed is the UUIDv5 of its public address (ADR 0032, the Podcasting 2.0 rule for `podcast:guid`). After Studio writes the feed, its GUID does not change (ADR 0008).
- A write ends with a read of the new file from its public address. A file host with no public address thus cannot finish a write.
- A feed move of ADR 0038 needs a first write, because it moves a hosted file and leaves a redirect.
- The artist page and its albums name each other by `feedGuid` and `feedUrl` (ADR 0041 item 12).
- The operator selected practice mode: the user tries everything in a folder, nothing is public, and a later step publishes the pages for real.

## Decision

1. A feed is a practice page when its file host is a local folder (the kind `local`) and gives no public address for its path. Studio does not keep a flag: the core finds the state from the file host. A feed on a different kind of file host with no public address gets the error `no_public_address`, as before: S3, SFTP and nsite are servers that must have an address, and a Blossom host holds media only (ADR 0048 item 2).
2. The core makes a practice page as a draft, as it makes each new feed. Its GUID is temporary: the UUIDv5 of `studio-practice://{connector ID}/{path}`. Its draft has no `self` link.
3. Studio writes no practice page. The core refuses `write_draft` for a practice page with the error `practice_page`, before it changes a file. Studio announces nothing for a practice page.
4. A link to a practice page has the temporary GUID in `feedGuid`, and no `feedUrl`.
5. "Publish for Real" acts on each practice page of one file host together. The user selects a file host with a public address (a new one or a connected one). Then the core, in one transaction:
    - Moves each practice page to that file host, with the same path. A path that exists on that file host stops the operation, and the core changes nothing.
    - Gives each page its real GUID, the UUIDv5 of its new public address, and its `self` link.
    - Changes each `feedGuid` and `feedUrl` in the practice pages that named a temporary GUID of the operation.
    - Keeps the release folder and the library files of each page.
6. After "Publish for Real", each page is a draft on the new file host. The first write goes through Review (ADR 0007).
7. A practice page shows the badge "Practice" in Releases, in the left panel and in the toolbar. In the toolbar, "Publish for Real" is in the place of "Review".
8. The Welcome step 2 row "My own server" opens the folder form with the public address as an optional field. Its hint is "Leave it empty to practice. Nothing is public until you publish for real."

## Alternatives Considered

- Ask for the public address. Not chosen by the operator: a new user often has no web server.
- Put "Nostr (Blossom)" back as the first row. Not chosen by the operator.
- A random GUID that stays after the publish. Not chosen: the Podcasting 2.0 rule wants the UUIDv5 of the feed address.
- A move of one practice page at a time. Not chosen: the links between the pages use the temporary GUIDs.

## Consequences

- The core makes a feed on a file host with no public address, and refuses its write.
- The core gets the operation "Publish for Real", with a route of the API.
- A feed row of the API tells that the feed is a practice page.
- The UI shows the badge, the toolbar button and the sheet of the operation.
- A task file gives the work.

## Non-goals

- A preview of a practice page in a player.
- A change of the move of ADR 0038 for a written feed.
- A practice page on a server shell with a file host that has a public address.

## Acceptance Criteria

Mechanical:

- A core test: a new feed on a folder with no public address has the temporary GUID of item 2 and no `self` link.
- A core test: `write_draft` of a practice page gives `practice_page` and changes no file.
- A core test: "Publish for Real" of an artist page and an album that link to each other gives each its real GUID and `self` link. It also changes the two links. A path that exists on the target stops it with no change.
- An API test: the feed row tells a practice page, and the route of "Publish for Real" moves the pages.
- A Node test: the badge, the toolbar button and the texts of items 7 and 8.
- The four commit checks pass.

Visual: a new user selects "I make music", leaves the public address empty, adds an album, and selects "Publish for Real". The check uses 1100 px, and 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human: the operator tries Studio in a local folder, then publishes the pages for real on a file host.

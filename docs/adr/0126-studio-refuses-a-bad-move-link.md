# ADR 0126: Studio Refuses A Bad Move Link

- Status: Accepted
- Date: 2026-11-04
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-04 by the operator. The operator accepted items 5, 7, 12 and 26 as written.
- Changed by: [ADR 0127](0127-a-changed-feed-gets-a-new-build-date.md). Items 8 and 26: Review and a granted write can send the time request of that record. That request reads no feed.
- Source: [research request on a feed check with repairs](../plans/research-feed-check-request.md), item F0. Research note `research_notes/aggregation/itunes-new-feed-url.md` in `../musicindex-research`.
- Changes: [ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md) item 19 (the sequence) and item 20 (a pointer file can become the feed). [ADR 0043](0043-phase-9-connectors-and-the-interface-change.md) item 9, for a move that stage 2 refuses. [ADR 0057](0057-the-private-network-guard-of-the-server.md) item 1, for the read of this record on the desktop.
- Uses: [ADR 0021](0021-public-address-of-a-feed.md), [ADR 0054](0054-live-items-connected-apps-and-show-grants.md) and [ADR 0065](0065-the-core-applies-each-feed-rule.md).

## Context

The research request records this requirement of the operator of 2026-11-04. A legitimate move always works. A loop or another problematic link fails, and Studio detects and refuses it. The operator also agreed to a refusal when Studio cannot read the destination. A failed read of the index gives a notice.

The review of the request found these facts at commit `4279546`:

- The write puts the feed on the new host, writes the pointer, and then checks the public address (`pipeline.rs:3436`, `pipeline.rs:3482`). The check reads a status and not a body. Studio can thus write a pointer to an address that fails.
- A move back to an earlier host fails. A probe test gave `FileExists` from `set_feed_host`, because the pointer file is at the earlier host.
- `write_to_new_host` continues a stopped move only with the same bytes (`pipeline.rs:4076`).
- A feed that names itself is the normal form. Each of the 6 feeds with the tag in the Stophammer backup names its own `feed_url`. In the sample of the research note, 559 of 621 tags name the exact URL of the feed.
- A self link can be stale, and the public address comes first from the read (ADR 0021 item 1 and its evidence). A correct feed behind a `301` can have a self link that differs from the public address.
- No v1 connector implements `set_redirect`. Each move of a connected feed thus uses a pointer file.
- The Stophammer API has no lookup of a feed by URL at commit `d807f8b`.
- A granted write changes no link (ADR 0054 items 13 and 15).

## Decision

### The Names Of A Body

1. A body names the URL in `itunes:new-feed-url` and the URL in the `href` of the first `atom:link` with `rel="self"`.
2. The core compares two URLs as exact text, as Stophammer does today. A different spelling is a different URL.
3. The URL of a read body is the address after each redirect. A chain visits the requested address and that address.
4. The URL of a draft is the address that the write gives: the destination of the move, otherwise the public address. A draft on a file host with no public address has no URL. A body that names only its own URL, or names nothing, is the end of a chain.

### What The Core Follows

5. The core follows the tag of a draft when the tag names a different URL, also when the draft does not change the tag. The core follows the self link of a draft when it names a different URL and the draft changes it from the base. This narrows the requirement of the operator, and the operator accepted it. A self link that the draft does not change gets item 6, and no refusal.
6. Review gives a notice for a self link that names a different URL and that the draft does not change. The core does not read that URL. ADR 0021 items 3 and 5 stay in force for that difference.
7. The core reads each URL with `NetPolicy::Public`, in both shells. A move to a private address thus fails on the desktop too. The test exception of ADR 0057 item 7 stays the only exception.
8. A write with no followed URL and no move sends no new request.

### The Refusal

9. The core refuses the write in each of these conditions:

| # | Condition | Stage |
|---|---|---|
| 1 | The chain comes back to a URL that it visited, or has more than 10 hops | 1 |
| 2 | A followed URL or the destination does not answer with an RSS feed, or the read fails | 1 and 2 |
| 3 | The body has a `podcast:guid` different from the draft, or has none when the draft has one | 1 and 2 |
| 4 | A body names two different URLs, other than its own URL | 1 |
| 5 | The index holds the destination or a followed URL as the source of a different GUID | 1 |
| 6 | The draft changes the self link, and the self link differs from the URL of the draft | 1 |
| 7 | A followed URL or the destination is not a global address (item 7) | 1 |

10. The refusal gives the condition and each hop with its URL and its result. Studio has no override.
11. A failed read of the index gives a notice and no refusal.
12. Condition 5 does not run until Stophammer has a lookup of a feed by URL. When that route exists, a stale row can stop a correct move until the index operator acts, because Studio has no override. The text of the refusal says this.

### The Two Stages

13. Stage 1 runs before any file changes on a host. It runs when Review opens, and again at the start of the write, because a destination can change after Review. For a move, stage 1 reads the destination only for condition 7, because the destination has no feed yet.
14. Stage 2 runs for each move: after the feed XML is on the new host, and before the redirect, the pointer file, the snapshot and the new location of the feed. This also applies to a move from a read-only feed and from an expired temporary area, which write no pointer.
15. Stage 2 reads the destination. It passes when the body is an RSS feed, with the `podcast:guid` of the draft and the names of the draft. It makes a maximum of 3 reads, with 10 seconds between them, because a host or a CDN can serve an earlier result.
16. The sequence of ADR 0038 item 19 becomes:
    1. The uploads and the copies.
    2. The feed XML on the new host.
    3. Stage 2.
    4. The redirect or the pointer file.
    5. The new location of the feed.
    6. The check of ADR 0043 items 7 and 8.
    7. The deletions.

### After A Refusal

17. After a refusal in stage 1, no file changes. The draft does not change.
18. After a refusal in stage 2, the address before the move does not change. Studio writes no pointer and does no deletion, and the feed keeps its location. The draft keeps its move. The uploads and the copies get a confirmation, as in ADR 0043 item 9.
19. The collection records the feed XML that stage 2 refused as a placed file of the feed. The next write replaces a placed file when the file on the host has the `podcast:guid` of the draft. Otherwise the write gives `FileExists`. `set_feed_host` accepts a placed file of the feed. A placed file is not a pointer file, so the user can delete it.
20. The core gives two repairs for one user action, through the draft and the diff: "Use the Public Address" sets the self link to the URL of the draft, and "Remove the Tag" removes `itunes:new-feed-url`.

### A Return To An Earlier Host

21. `set_feed_host` accepts a host with a pointer file of this feed at the path. The write replaces that file when it has the `podcast:guid` of the draft. Otherwise the write gives `FileExists`. The diff says that the pointer file becomes the feed.
22. After that write, the path is no longer a pointer file in the collection. The host from before gets its own pointer file (ADR 0038 item 19).
23. No v1 connector can set a redirect. A connector that adds it needs a record that gives the removal of a redirect.

### Other Writes

24. A published feed with a bad link stays readable. Studio refuses only a write, and an import or a read gives no refusal.
25. A change set applies this record to each feed. A refused feed keeps its draft, and the refusal does not stop the other feeds.
26. A granted write makes no read and gets no refusal from this record, because it changes no link (ADR 0054 items 13 and 15). The write that records the grant is a normal write. The research request asks for the same check on a granted write. This record does not do that, so that a host failure or a stale link does not stop a live show.
27. A feed with no connector gets the same check when it gets a write.

## Alternatives Considered

- One check before the write. Rejected: the new host has no feed at that time, so each move fails.
- A read after the pointer. Rejected: the bad link is then on the old host.
- A refusal for each self link that differs from the public address. Not chosen by the operator: it refuses a correct feed behind a `301`, and each feed with a stale self link until the user repairs it.
- An override for the user. Not chosen: the operator asked for a refusal.
- The network policy of the shell for each read. Not chosen by the operator: the desktop policy `Open` permits each address, so condition 7 has no effect there.
- A delete of the file on the new host after stage 2 refuses it. Rejected: the file can be a help for the next try, and a delete is a host operation that the user did not see.
- The tag in the new feed. Not decided here: the research did not test other apps, and Stophammer accepts the form of ADR 0038.

## Consequences

- A host failure stops a write with a followed URL until the host answers.
- A move can take 20 seconds more when the destination is slow to serve the new file.
- A refused move leaves files on the new host until the next write or a deletion by the user.
- Studio needs a body read through the network policy in `write_draft`. The pipeline tests need a test double that serves the bodies.
- Condition 5 needs a route in Stophammer. That change needs a request and a decision in that repository.
- A tool other than Studio can still write a loop. The Stophammer request covers this.

## Invariants

- No pointer file and no redirect occurs before the destination passes stage 2.
- A refusal in stage 1 changes no file on a host.
- The feed keeps its `podcast:guid` and each item GUID (ADR 0008).
- Each repair goes through the draft and the diff (ADR 0007).
- The rules of this record are in the core (ADR 0065).

## Non-goals

- The feed check, the build date and the index view: items F1 to F6 of the request.
- An update of an older pointer after a second move.
- A lookup route in Stophammer.
- The form of the refusal in Review. A mockup comes before the UI packet of the task.

## Acceptance Criteria

Mechanical:

- Core tests give each condition of item 9, except condition 5, with a local test server. Each test proves that stage 1 changes no file.
- A core test: a draft with a tag that names its own URL passes, with no read.
- A core test: chains with an end pass (scenarios S13 and S16). A loop of two URLs and of three URLs fails (scenarios S3 and S12).
- A core test: a self link that the draft does not change gives a notice and no read. The same self link, when the draft changes it, is refused.
- A core test: a destination that fails stage 2 gets no pointer. The feed keeps its location, and the draft keeps its move.
- A core test: after that refusal, a repair changes the bytes, and the next write completes the move.
- A core test: a slow destination passes on the second read.
- A core test: a move from A to B and then back to A completes. A has the feed, B has a pointer that names A, and A is not a pointer file in the collection.
- A core test: a file at a pointer path with a different `podcast:guid` gives `FileExists`, and the file does not change.
- A core test: a granted write makes no read.
- A core test: each repair gives a diff with only its one change.

Visual, by the planning model:

- Review shows a refusal with each hop, and the notice of item 6, at 1280 px and at 390 px with touch.

Human, by the operator:

- A move between two real hosts, and a return. A second app reads the feed at the new address after each move.

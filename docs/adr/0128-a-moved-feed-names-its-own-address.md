# ADR 0128: A Moved Feed Names Its Own Address

- Status: Accepted
- Date: 2026-11-04
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-04 by the operator. The operator selected the tag at each move, and a warning at each return.
- Changed: on 2026-11-05 the operator added item 12, so that a person who reads the file finds the tag first. This changes [ADR 0012](0012-lossless-tree-with-a-typed-model.md) item 3 for this tag.
- Source: the handoff note of the Stophammer session of 2026-11-04, `.idea/studio/stophammer-feed-contract-handoff.md` in `../stophammer`. Stophammer ADR 0083 owns the rule of the index.
- Changes: [ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md) item 19, for the feed on the new host. [ADR 0126](0126-studio-refuses-a-bad-move-link.md) "Alternatives Considered", which left the tag in the new feed open.
- Changed by: [ADR 0190](0190-review-gives-the-return-wait-of-music-index.md). Items 7, 8 and 10: the warning gives the time of the return wait from Music Index.
- Uses: [ADR 0007](0007-draft-diff-write-and-announce.md), [ADR 0052](0052-announcements-through-the-podcast-index-hub.md) and [ADR 0065](0065-the-core-applies-each-feed-rule.md).

## Context

Stophammer ADR 0083 is Accepted, with its code on Stophammer `main` at commit `61e8762`. No release candidate of Stophammer holds it yet. For a return to a URL that a record left, it needs a tag at each end. The body at that URL must have an `itunes:new-feed-url` that names its own URL. A self link cannot start a return.

The examination of Studio at commit `1283bc7` found these facts:

- `pipeline.rs` sets the tag only in `pointer_bytes`. `set_feed_host` sets only the self link, and only when the feed has one.
- After a move from A to B and back to A, the feed at A has no tag. Under ADR 0083, the index would refuse that return, and the record would stay at B. Later writes at A would not reach the index.
- The test `a_move_from_a_to_b_and_back_to_a_completes` examines only the self link, so it passes.
- ADR 0126 already passes a draft with a tag that names its own URL, with no read.
- ADR 0083 section 5 gives a return a wait that grows, and the first return has no wait. Studio cannot read the return count from the index.

On 2026-11-04, a one-time check ran in copies of both repositories. It sent the bytes that Studio writes (commit `375ae55`) through the Stophammer parser and `POST /ingest/feed` (commit `61e8762`). Each test passed.

A move, a first return and a later write reach the index. A second return waits for the cooldown, and it completes at a crawl after the wait. The same return with the tag removed stays at B with the refusal of ADR 0083. The check ran no real host, no real crawler and no deployed node.

## Decision

### The Tag

1. Each move gives the draft an `itunes:new-feed-url` with the URL of the destination, in the exact text of `FeedMove::url`. This also applies to a feed with no self link. It applies to a move from a read-only feed and from an expired temporary area.
2. The diff shows the tag as each other change (ADR 0007).
3. A tag that the draft had before the move gets the new value. The diff shows that change, and stage 1 of ADR 0126 then has no URL to follow.
4. Undo of a move puts back the tag of the base, or removes the tag when the base has none.
5. The pointer file keeps its form. It names the destination, as ADR 0038 gives.
6. A feed that never moved gets no tag from this record. A later edit keeps a tag that names the address of the feed. A copy keeps no tag (`copy_feed`).

### The Warning

7. A return shows a warning in Review, below the move row. The text says that this is a return to an earlier address. It says that the index can wait before it follows a return, and that the wait grows with each return.
8. The warning does not stop the write, and it is not an error. It gives no time, because Stophammer ADR 0083 owns the times, and Studio cannot read the count.
9. `FeedMove::replaces_pointer` from the core selects the warning. The UI calculates no condition (ADR 0065).

### The Index

10. Stophammer owns how the index reads a feed. Studio cites ADR 0083 and does not restate its rule as a rule of Studio.
11. If the rule of ADR 0083 changes, this record changes in the same commit as the code.

### The Position Of The Tag

12. When Studio adds the tag, the tag goes directly after `<channel>`. A tag that the feed already has stays in its position, and a tag of the hosted file gets its hosted position ([ADR 0031](0031-a-new-element-takes-its-hosted-position.md)).

## Alternatives Considered

- A tag in each new feed. Not chosen by the operator: the index needs nothing for a feed that never moved. A feed with no address has no value for the tag. Each copy by hand also carries a tag that names the earlier address.
- A tag only at a return. Rejected: the code then needs a separate form for a return. A tag at each move is also the form that Apple describes for a new feed.
- A text with the time of the wait. Rejected: the time can change, and Studio does not know the count.
- No warning until the index view of F3. Not chosen by the operator: a user can think that the move failed.
- A refusal of a second return. Rejected: the wait is in the index, and it ends.

## Consequences

- The diff of each move has one more line.
- A wait ends only at a later crawl of the destination. A Studio write announces each address one time, so a return in a wait can need a later crawl or a new write. The research did not examine how often the crawler reads such a feed.
- A tag stays in a feed after its move. A copy of that file by hand then names the earlier address. Stophammer ADR 0083 section 2 item 3 examines this case.

## Invariants

- After a move, the feed on the new host has a tag that names its own URL.
- The pointer file names the destination.
- The feed GUID and each item GUID do not change (ADR 0008).
- A warning never stops a write.

## Non-goals

- The index view, and a display of the stored source URL: F3 of the research request.
- A lookup of a feed by URL: ADR 0126 condition 5.
- An update of an older pointer after a second move.

## Acceptance Criteria

Mechanical:

- A core test: after a move, the feed on the new host has a tag with the exact text of its URL. The failure message names Stophammer ADR 0083 section 2 and this record.
- A core test: the same after a return, and after two returns in sequence.
- A core test: the same for a feed with no self link, and for a draft with a different tag before the move.
- A core test: Undo of a move gives the tag of the base, and it gives no tag when the base has none.
- A core test: the diff of a move shows the tag.
- A core test: a new tag is the first child of the channel, and a tag of the base stays in its position.
- A Node test: the model module gives the warning only when `replaces_pointer` is true.

Visual, by the planning model:

- Review shows the warning below the move row of a return, at 1280 px and at 390 px with touch. A move that is not a return has no warning.

Human, by the operator, after a release of Stophammer holds ADR 0083:

- A move and a return between two real file hosts. After each step, `GET /v1/feeds/{guid}` of the index gives the new address.

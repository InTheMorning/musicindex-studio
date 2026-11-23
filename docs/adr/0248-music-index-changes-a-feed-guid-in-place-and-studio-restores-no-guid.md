# ADR 0248: Music Index Changes A Feed GUID In Place, And Studio Restores No GUID

- Status: Accepted
- Date: 2026-11-22
- Owner: operator
- Accepted: 2026-11-22 by the orchestrator session, for the operator. The orchestrator approved the deletion of the items below after the deploy of stophammer 0.22.0-rc.1 started.
- Changes: [ADR 0247](0247-studio-restores-a-changed-feed-guid-and-corrects-a-link-with-an-old-guid.md) items 1 to 6, 10 and 11. This record deletes them. Items 7 to 9 stay current.
- Changes: [ADR 0175](0175-the-check-shows-what-music-index-holds.md), the fact of its Context on the decision of a GUID change and its invariant on a GUID change. Items 4 and 11 of ADR 0175 apply again with no change.
- Uses: [ADR 0008](0008-feed-types-and-feed-identity.md) item 2, [ADR 0122](0122-a-new-user-starts-with-a-default-file-host.md) item 8, [ADR 0131](0131-a-feed-can-be-retired-from-the-index.md) item 6. Stophammer ADR 0106 §3 and §4 own the behavior of the index. This record cites them and does not restate them as rules of Studio.
- Source: the report of the main session to the orchestrator on stophammer ADR 0106, 2026-11-22. The research note `musicindex-research/research_notes/aggregation/podcastindex-guid-behavior.md` §6.
- Bar: opposite choice by mistake. Without this record, an agent can build **Restore the Original GUID** of ADR 0247 item 4. Music Index then changes the GUID of the feed a second time, and Podcast Index keeps the new GUID.

## Context

ADR 0247 items 1 to 6 used a fact of Stophammer ADR 0052 §4 and §5. Music Index kept the original GUID of a feed and a pending GUID change with no decision. Thus Studio restored the original GUID in the draft.

Stophammer ADR 0106 (accepted 2026-11-22, the deploy of 0.22.0-rc.1 started on that date) replaces that behavior:

- A feed that declares a new channel GUID at the same URL changes its record in place. No operator step occurs (Stophammer ADR 0106 §1 and §2).
- A request for the old GUID gives 404 with `superseded_by`, which is the new GUID. This applies to `GET /v1/feeds/{old_guid}` and to `GET /v1/feeds/{old_guid}/tracks/{track_guid}`. The track GUID does not change (Stophammer ADR 0106 §3).
- The index deletes the pending rows. `GET /v1/guid-changes` gives an empty list (Stophammer ADR 0106 §4).

On 2026-11-22 Podcast Index gave the new GUID for each of the four Elijah Lied addresses of ADR 0247 (research note §6). After the crawler reads each feed again, Music Index holds the same new GUID.

These facts of the code on 2026-11-22 apply:

- No code of Studio reads `GET /v1/guid-changes` or a GUID change event.
- `MusicIndex::feed_presence` in `crates/core/src/musicindex.rs` reads `superseded_by` and gives `IndexPresence::Superseded`. The retire (`crates/core/src/feed_retire.rs`) and the outside check (`crates/api/src/routes.rs`) use it. Each other read of the index treats each 404 as a feed that the index does not hold.
- No code of ADR 0247 is built.
- In the tutorial data, the four releases have the new GUIDs, because the feed files have the new GUIDs. No live row of that collection holds an old GUID.

## Decision

1. Items 1 to 6, 10 and 11 of ADR 0247 are deleted. Studio has no finding `guid_changed` and no finding `release_guids_changed`. Studio sends no request to `GET /v1/feeds/by-url` for these findings.
2. Studio changes a feed GUID at one time only: a placeholder GUID gets the real GUID (ADR 0122 item 8). Studio has no repair that puts back an earlier GUID of a feed.
3. Items 7 to 9 of ADR 0247 stay current. A link with an old GUID gets the finding `link_guid_old`, and the move corrects the link.
4. ADR 0175 items 4 and 11 apply again as their text gives. A feed that the index does not hold gives the line "Music Index does not hold this feed yet."
5. Studio does not change a GUID in the index. Under Stophammer ADR 0106, the index changes a GUID in place when the feed declares a new GUID at the same URL.

## Open Questions

- The check does not follow `superseded_by`. A draft with an old GUID then gets the line of ADR 0175 item 4 for a feed that the index does not hold. A finding that names the new GUID needs a new ADR that changes ADR 0175 item 1. On 2026-11-22 no draft with an old GUID is known, so no change occurs now.
- Four reads do not follow `superseded_by`: the payment routes, the track routes, the publisher values and the links of a feed. A stored old GUID then gives "not in Music Index". Examples are the GUID of a person page and a link to a feed outside the collection. A change needs a new ADR that changes ADR 0207 item 6 and ADR 0159 item 3. On 2026-11-22 no stored old GUID is known, so no change occurs now.
- The main session examined only the tutorial data. The data of the operator and the data of the hosting user of the tutorial are not examined. It is not known if a draft or a link there holds one of the four old GUIDs.

## Alternatives Considered

- Keep ADR 0247 items 1 to 6 for a future index that keeps a pending change. Rejected, because no reader of the code remains, and dead decisions are deleted.
- Follow `superseded_by` in each read now. Not chosen, because no stored old GUID is known.

## Checks

Mechanical: no code changes, so no test changes. The ADR link guard of `tools/gate.sh` checks the link lines of this record, ADR 0247 and ADR 0175.

Visual: none.

## Consequences

- The tutorial needs no GUID repair for the four Elijah Lied releases after the crawler reads them again.
- ADR 0247 now gives only the link with an old GUID and its correction by the move.

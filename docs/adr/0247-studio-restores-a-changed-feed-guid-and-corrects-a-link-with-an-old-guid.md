# ADR 0247: Studio Restores A Changed Feed GUID And Corrects A Link With An Old GUID

- Status: Accepted
- Date: 2026-11-22
- Owner: operator
- Accepted: 2026-11-22 by the operator, from drawings in the questionnaire. The move corrects the link, and the tab **Issues** shows each link with an old GUID with a repair. For a changed feed GUID, the tab shows one row for each feed and one row for all on the page that links the feeds.
- Technical choices: the main session selected the reads of items 1 and 2 and the condition of item 3. It also selected the level, the texts and the path of the repair.
- Changed by: [ADR 0248](0248-music-index-changes-a-feed-guid-in-place-and-studio-restores-no-guid.md) items 1 to 5, on 2026-11-22. Items 1 to 6, 10 and 11 are deleted. Music Index changes a feed GUID in place (Stophammer ADR 0106), so Studio restores no GUID. Items 7 to 9 stay current. This record no longer changes ADR 0175.
- Uses: [ADR 0008](0008-feed-types-and-feed-identity.md) item 2, [ADR 0122](0122-a-new-user-starts-with-a-default-file-host.md) item 8, [ADR 0174](0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md), [ADR 0255](0255-publisher-feeds-announcements-and-show-grants.md) item 12, [ADR 0243](0243-a-move-is-a-change-set-and-a-level-gathers-its-pages.md). Stophammer ADR 0052 sections 4 and 5 own the pending GUID change. This record cites them and does not restate them as rules of Studio.
- Source: [the findings of the tutorial for a hosting user](../reviews/2026-11-22-tj-tutorial-findings.md), finding 13.
- Bar: crosses modules. The reads, the check, the repair, the move and the UI change. Without this record, an agent can correct the links to the changed GUID, and Music Index then keeps the old record.

## Context

The artist feed of Elijah Lied links six releases. The link to "This Is It" has the GUID `11f41bfe-7cbe-5795-85c3-6deaccac20aa`. The file `thisisit.xml` has the GUID `24e9311b-73a9-5f48-b593-81148712cb14`. Music Index holds that address as `24e9311b…` and has no record of `11f41bfe…`. The `podcast:podroll` of "all in a day" has the same old GUID.

On 2026-09-11, a publisher tool gave four Elijah Lied releases new channel GUIDs. On 2026-09-25 the publisher confirmed that the tool made them in error (Stophammer ADR 0052 section 2). On 2026-11-22, `GET /v1/feeds/by-url` gave the original GUID for each of the four addresses:

| Release | GUID in the file | GUID that Music Index holds |
| --- | --- | --- |
| Kill The MF Beat | `2b05d37c-fac5-5ae3-a5ad-3a9662038487` | `01128ba4-19c4-53d4-b209-d4a7ece8b889` |
| Through The Mic | `ff59d578-2f7f-56e4-b38d-b0c0f3ab799c` | `d6fe0d1c-bd8d-552b-ac66-cce620391810` |
| all in a day | `25f3e971-453c-507a-97e8-800f38f56415` | `6fde038d-3585-5d28-93e5-aabe48fd7fa6` |
| M0mma | `64645682-dbe0-5bc0-b52b-9f662d0b38d9` | `902ae9db-c74b-5fd4-bcf1-d206c19479b4` |

Each record has a `pending_guid_change` with the new GUID and no decision. Music Index keeps the record and its content while the change is pending. Thus Music Index does not take a write of these feeds. When the address declares the original GUID again, Music Index deletes the pending change (Stophammer ADR 0052 section 4).

These facts of the code on 2026-11-22 apply:

- The check reads `GET /v1/feeds/{guid}` with the GUID of the draft. For the four releases, the answer is 404, and the check says "Music Index does not hold this feed yet." This text is not correct.
- No code reads `GET /v1/feeds/by-url` or `pending_guid_change`.
- `names` in `crates/core/src/publisher.rs` matches a link by its `feedGuid` when it has one. A link with an unknown `feedGuid` and a correct `feedUrl` names no feed. Thus "Move Artists Here…" gives the link to "This Is It" no `rel`.
- The editor refuses a change of the feed GUID (`keep_identity` in `crates/core/src/pipeline.rs`). `assign_real_guids` changes a feed GUID and each link that names it, in the drafts of the collection (ADR 0122 item 8).

The check of Podcast Index is not possible from Studio, because its API needs a key. It is not known which GUID Podcast Index holds.

## Decision

### The Changed Feed GUID

Items 1 to 6 are deleted by [ADR 0248](0248-music-index-changes-a-feed-guid-in-place-and-studio-restores-no-guid.md).

### The Link With An Old GUID

7. A `podcast:remoteItem` in a draft of the collection has an old GUID when these conditions are true:
    - Its `feedGuid` names no feed of the collection.
    - Its `feedUrl` names a feed of the collection, with the spelling rule of ADR 0174 item 12.
    - That feed has a different `podcast:guid`.

   The check finds such a link with no request.
8. The finding `link_guid_old` has the level Warning. Its title is "A link to {title} has an old GUID". Its reason gives the old GUID and the GUID of the feed. Its button is **Use the GUID of {title}**. The repair sets the `feedGuid` of each such link of the draft that names that feed. The repair changes only the draft of this feed.
9. Before the move of ADR 0243 sets the roles, the move corrects each link of item 7 in the drafts that it changes. Thus the match of ADR 0255 item 12 then finds the release, and the release gets its role.

### Facts That Give No Finding

Items 10 and 11 are deleted by [ADR 0248](0248-music-index-changes-a-feed-guid-in-place-and-studio-restores-no-guid.md).

## Alternatives Considered

- Correct the links to the changed GUID. Rejected, because Music Index keeps the original record, and a feed GUID must not change (ADR 0008 item 2).
- Ask the operator of Music Index to approve the change. Rejected, because the publisher confirmed that the change is an error. Also, the user of Studio cannot decide a GUID change.
- A row for each feed only, or a row on the linking page only. The operator selected both rows.
- Match a link by its `feedUrl` when its `feedGuid` names no feed, in ADR 0255 item 12. Not chosen: the move corrects the link first, so the match rule does not change.

## Checks

Mechanical:

- A test of item 7: a link with an unknown `feedGuid` and the `feedUrl` of a collection feed gives `link_guid_old`. A link with a known `feedGuid` gives no finding. The repair sets the `feedGuid`.
- A test of the move: "Move Artists Here…" with a page that has a link of item 7 gives the link the GUID of the release and the `rel` of the role.
- A Node test of the rows in `crates/api/ui/feedcheck.js`: the label of item 8.

Visual, by the main session with screenshots of the tutorial data (`.studio-dev/v-230/data-tut`) and scripted index answers:

- The tab **Issues** of Elijah Lied shows the row of item 8 for "This Is It".

## Consequences

- An old link in a `podcast:podroll` or a channel link gets a finding with a repair.

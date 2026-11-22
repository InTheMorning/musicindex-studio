# ADR 0247: Studio Restores A Changed Feed GUID And Corrects A Link With An Old GUID

- Status: Accepted
- Date: 2026-11-22
- Owner: operator
- Accepted: 2026-11-22 by the operator, from drawings in the questionnaire. The move corrects the link, and the tab **Issues** shows each link with an old GUID with a repair. For a changed feed GUID, the tab shows one row for each feed and one row for all on the page that links the feeds.
- Technical choices: the main session selected the reads of items 1 and 2 and the condition of item 3. It also selected the level, the texts and the path of the repair.
- Changes: [ADR 0175](0175-the-check-shows-what-music-index-holds.md) items 4 and 11, and its invariant on a GUID change. A pending GUID change of the feed gives a finding.
- Uses: [ADR 0008](0008-feed-types-and-feed-identity.md) item 2, [ADR 0122](0122-a-new-user-starts-with-a-default-file-host.md) item 8, [ADR 0174](0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md), [ADR 0205](0205-publisher-feeds-announcements-and-show-grants.md) item 12, [ADR 0243](0243-a-move-is-a-change-set-and-a-level-gathers-its-pages.md). Stophammer ADR 0052 sections 4 and 5 own the pending GUID change. This record cites them and does not restate them as rules of Studio.
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

### The Reads

1. When `GET /v1/feeds/{guid}` gives 404 for a feed with a public address, the reads of ADR 0175 item 1 also send `GET /v1/feeds/by-url` with that address. When the answer gives a different GUID, the reads send `GET /v1/feeds/{that guid}` and keep its `pending_guid_change`. The facts of the index then give the original GUID. They give it only when the `new_guid` of the pending change is the GUID of the draft, and the decision is not `approve`.
2. The reads of a feed with channel release links (ADR 0246 item 1) send `GET /v1/feeds/by-url` for the `feedUrl` of each release link. They keep the original GUID of each linked release that item 1 gives. The reads send these requests only when the user opens the check and when the user selects **Check Again** (ADR 0175 item 1).

### The Changed Feed GUID

3. The finding `guid_changed` has the level Warning. Its title is "The GUID of this feed changed". Its reason is "Music Index holds this address with the GUID {original}. A feed GUID must not change. Music Index does not take your writes until the feed has this GUID again." The line "Music Index does not hold this feed yet." does not show with this finding.
4. The button of the finding is **Restore the Original GUID**. The repair sets the `podcast:guid` of the draft to the original GUID and changes its OP3 prefixes.

   In the same step, it changes each `feedGuid` that names the changed GUID in the drafts of the collection to the original GUID. A written feed with no draft that names the changed GUID gets a draft from its file, as in ADR 0122 item 8. Item GUIDs do not change. The user writes the changed drafts through Review (ADR 0007).
5. The page that links the feeds shows one finding `release_guids_changed` with the level Warning. Its title is "{n} releases changed their GUID". The finding lists each release with its changed GUID and its original GUID. Its button is **Restore the {n} Original GUIDs**, or **Restore the Original GUID** for one release. The button does the repair of item 4 for each listed release in one step.
6. A read-only feed shows the findings of items 3 and 5 without the button (ADR 0174 item 20).

### The Link With An Old GUID

7. A `podcast:remoteItem` in a draft of the collection has an old GUID when these conditions are true:
    - Its `feedGuid` names no feed of the collection.
    - Its `feedUrl` names a feed of the collection, with the spelling rule of ADR 0174 item 12.
    - That feed has a different `podcast:guid`.

   The check finds such a link with no request.
8. The finding `link_guid_old` has the level Warning. Its title is "A link to {title} has an old GUID". Its reason gives the old GUID and the GUID of the feed. Its button is **Use the GUID of {title}**. The repair sets the `feedGuid` of each such link of the draft that names that feed. The repair changes only the draft of this feed.
9. Before the move of ADR 0243 sets the roles, the move corrects each link of item 7 in the drafts that it changes. Thus the match of ADR 0205 item 12 then finds the release, and the release gets its role.

### Facts That Give No Finding

10. A pending GUID change that does not agree with item 1 gives no finding. Examples are a change with the decision `approve`, and a change of a different new GUID. A copy with the same recipients and a closed copy also give no finding (ADR 0175 item 11).
11. Studio does not decide a GUID change in Music Index. The repair of item 4 changes only the feed of the user. Music Index then deletes the pending change itself.

## Alternatives Considered

- Correct the links to the changed GUID. Rejected, because Music Index keeps the original record, and a feed GUID must not change (ADR 0008 item 2).
- Ask the operator of Music Index to approve the change. Rejected, because the publisher confirmed that the change is an error. Also, the user of Studio cannot decide a GUID change.
- A row for each feed only, or a row on the linking page only. The operator selected both rows.
- Match a link by its `feedUrl` when its `feedGuid` names no feed, in ADR 0205 item 12. Not chosen: the move corrects the link first, so the match rule does not change.

## Checks

Mechanical:

- `crates/core` tests of the reads with `ScriptedIndexReads`: a 404 on the GUID, `by-url` with a different GUID, and a pending change to the GUID of the draft give the original GUID. A pending change with `approve`, or with a different new GUID, gives no original GUID. A feed with no public address sends no `by-url` request.
- A test of `feed_check::check`: the original GUID gives `guid_changed` with the level Warning and no line of ADR 0175 item 4.
- A test of the repair: **Restore the Original GUID** sets the GUID of the draft and the OP3 prefixes. It changes the `feedGuid` of a link in another draft and in a written feed with no draft. The item GUIDs do not change.
- A test of the publisher reads and check: two linked releases with a pending change give one `release_guids_changed` with two releases. Its repair changes both feeds and the links of the page.
- A test of item 7: a link with an unknown `feedGuid` and the `feedUrl` of a collection feed gives `link_guid_old`. A link with a known `feedGuid` gives no finding. The repair sets the `feedGuid`.
- A test of the move: "Move Artists Here…" with a page that has a link of item 7 gives the link the GUID of the release and the `rel` of the role.
- A Node test of the rows in `crates/api/ui/feedcheck.js`: the labels of items 4, 5 and 8, the singular label, and no button on a read-only feed.

Visual, by the main session with screenshots of the tutorial data (`.studio-dev/v-230/data-tut`) and scripted index answers:

- The tab **Issues** of "Kill The MF Beat" shows the row of item 3 with its button.
- The tab **Issues** of Elijah Lied shows the row of item 5 with the four releases, and the row of item 8 for "This Is It".
- After **Restore the 4 Original GUIDs**, Review shows the GUID change of each release and the changed links of the artist page.

## Consequences

- Music Index takes the writes of a feed that a tool changed in error again, after one repair and one write.
- The check sends one more request for a feed that Music Index does not hold, and one request for each release link of a linking page.
- An old link in a `podcast:podroll` or a channel link gets a finding with a repair.
- The tool that changed the GUIDs can change them again at its next write. The tutorial tells the user this.

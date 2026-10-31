# ADR 0091: A Change Set Edits One Recipient In Many Feeds

- Status: Accepted
- Date: 2026-10-30
- Owner: operator
- Accepted: 2026-10-30 by the operator.
- Changed by: [ADR 0095](0095-a-lightning-address-and-its-node-key-are-one-recipient.md). Items 2 to 8 and 12: an operation acts on each address of one wallet, and the page of a recipient has a fifth operation.
- Changes: [ADR 0039](0039-the-shell-releases-and-the-outline.md) items 1, 5 and 11, for the destinations at the bottom of the left panel. [ADR 0030](0030-an-added-recipient-gets-no-split.md), for a recipient that a change set adds.
- Uses: [ADR 0027](0027-the-recipient-picker.md) for the identity of a recipient, [ADR 0088](0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) for the levels, and [ADR 0007](0007-draft-diff-write-and-announce.md) for the review and the write.

## Context

- A large collection has many albums, playlists and publisher feeds. Some releases have a two-way link to a publisher feed, some have a one-way link, and some have no publisher feed.
- One person is frequently a recipient in many feeds. When that person has a new wallet or a new node, the user must change the recipient in each feed and each track by hand.
- ADR 0027 identifies a recipient by its address, its custom key and its custom value. The table `feed_recipients` records each feed that has each recipient. `Feed::recipients` gives the recipients of the channel, of each item and of each live item. It does not give a recipient of a value time split.
- A track value block replaces the channel value block for that track. A change of the channel value block only can thus leave the previous recipient on each track that has its own value block.
- The collection keeps one draft for each feed (`drafts`). A write needs the ID of a diff that the user saw (ADR 0007).
- In Podcasting 2.0, `split` is a number of shares, not a percentage. A recipient with `fee="true"` takes its split as a percentage of the total first. Many feeds that MSP 2.0 made have non-fee splits with a total of 100.
- ADR 0030 gives no split to an added recipient. The user then changes the splits in each feed. With many feeds, this is not practical.
- The operator selected this decision from three questionnaires on 2026-10-30.

## Decision

### The Recipients Page

1. The left panel gets the destination "Recipients" at its bottom, before File hosts. The sequence at the bottom is Recipients, File hosts, Settings, then Lock Studio. The names and the icons of the bottom group are in [ADR 0092](0092-file-hosts-and-the-destination-icons.md). This changes ADR 0039 items 1, 5 and 11.
2. The Recipients page lists each recipient one time, with the identity of ADR 0027. A row shows the name, the type, and the number of feeds that use the recipient. A saved recipient that no feed uses shows the text "Saved, not used". The number comes from the same model of each feed as the list of item 3: the draft of the feed, or else its library copy. Thus the two pages give the same number. The planning model added this rule on 2026-10-30, after the visual check of task 054c.
3. A row opens the page of the recipient. That page lists each feed that uses the recipient, with its places: "Channel", the number of tracks, and the number of live items. Each feed has a checkbox, and each checkbox starts selected.
4. The page of a recipient has four operations: Change address, Rename, Remove, and Add to a level.

### The Operations And Their Scope

5. Change address replaces the address, the custom key, the custom value and the type. Each split and each other attribute stays the same.
6. Rename replaces the `name` attribute. Each other attribute stays the same.
7. Remove deletes the recipient from each value block of its places.
8. The scope of Change address, Rename and Remove is each selected feed of item 3.
9. Add to a level adds the recipient to each value block of each feed of a level. A level is a publisher feed and each feed below it, through `child_ids` (ADR 0088). The user selects the level, then the core lists its feeds with checkboxes, and each checkbox starts selected. The user gives a percentage from 1 to 99.
10. An operation changes each value block of the channel, of each item and of each live item. It does not change a value time split.
11. A publisher feed is in a scope as each other feed. Studio copies no recipient from a publisher feed to its releases, or from a release to its publisher feed ([ADR 0075](0075-the-publisher-page-has-payments-and-credits.md) item 4).
12. A change of the address or of the name of a saved recipient also changes the saved recipient. The core makes this change with the change set, and keeps the saved recipient from before the change. A change set in which no feed changed does not change the saved recipient.

### The Splits

13. Add to a level sets the split of the new recipient to the percentage p. The core then scales the other non-fee splits of the block, so that their total is 100 − p. Each recipient keeps its relative share. The core rounds each split to an integer, and gives the remainder to the largest split.
14. Remove scales the other non-fee splits of the block, so that their total is the non-fee total before the change. Each recipient keeps its relative share. A block with a non-fee total of 100 keeps a total of 100. The core rounds as item 13 says.
15. A fee split does not change.
16. These calculations are feed rules. They are in the core ([ADR 0065](0065-the-core-applies-each-feed-rule.md)).

### The Change Set

17. The core applies an operation to each feed of the scope as one change set. For each feed, the result is one of these outcomes:
    - **Changed.** The core put the change in the draft of the feed.
    - **Not changed.** The core gives the cause. For example, the block has the recipient before the change, or the block has the new address.
    - **Not changed** also applies when the feed has no value block, or when each split of the block is a fee.
    - **Not changed** also applies when the feed has a change of an open change set that is not written. Thus a discard of one change set cannot put back a draft over the change of a different change set. The planning model added this rule on 2026-10-30, at the review of task 054a.
18. The core puts the change in the open draft of the feed, if the feed has one. If the feed has no draft, the core makes a draft.
19. The collection keeps each change set until each changed feed is written or the user discards the change set. The change set keeps the draft of each feed before the change. Thus the core deletes a change set after the write of its last changed feed. A change set is closed when it has no changed feed that is not written. The core deletes each closed change set at the start of the next change set. The planning model added these three sentences on 2026-10-30, before `0.1.0-rc.5`.
20. "Discard" on a change set puts back the draft of each feed from before the change. A feed that had no draft then has no draft. A feed written after the change does not change. When the user discarded the draft of a feed after the change, that feed also does not change. The planning model added this rule on 2026-10-30, at the review of task 054a. Discard also puts back the saved recipient of item 12. The operator selected this rule on 2026-10-30.

### The Review And The Write

21. The review of a change set lists each feed with its outcome. Each changed feed opens its diff. The diff of a feed with an earlier draft also shows the earlier edits.
22. The review also lists each changed feed that Studio cannot write now, with the cause. For example, a read-only feed has no host ([ADR 0067](0067-a-read-only-feed-opens-in-the-editor.md)), or the vault is locked.
23. "Write" writes each feed that the user can write, one at a time, through `pipeline::write_draft` with the ID of the diff of that feed. The review shows the status of each feed during the write.
24. When the write of one feed fails, Studio does not stop the write of the other feeds. A conflict of ADR 0007 item 3 stops only that feed. The draft of a feed that is not written stays, and the change set stays open.
25. Each write announces its feed as each other write does ([ADR 0052](0052-announcements-through-the-podcast-index-hub.md)).
26. A show grant does not cover a change set. A change set needs the vault of the user.

## Alternatives Considered

- Select many feeds in a list, then apply one edit. Not selected by the operator for the first version. With it, a search for each place that pays a person is not easy. It can use the change set of this record later.
- A release takes the value block of its publisher feed. Not selected. Podcasting 2.0 has no inheritance of a value block, and ADR 0075 item 4 does not let Studio copy one.
- Change only the channel value block. Not selected by the operator. A track value block then keeps the previous recipient.
- Skip a feed with an open draft. Not selected by the operator. The user then must write each open draft before a change set.
- One fixed split for an added recipient, or a split of 0 (ADR 0030). Not selected by the operator. A fixed split changes the total of each block, and a split of 0 needs an edit of each feed.
- Keep the split values, and add a split of S × p / (100 − p). Not selected by the operator. The total of the block is then not 100.
- A scope of link groups: two-way, one-way, or not linked. Not selected for the first version.
- The Recipients page as a destination below Releases, or as a row on the Settings page. Not selected by the operator.

## Consequences

- The user changes one recipient in many feeds with one review.
- The collection schema gets a table for the change sets and the draft of each feed before the change. This is a new migration ([ADR 0014](0014-collection-in-sqlite.md)).
- A change set with many feeds makes many writes and many announcements.
- An added recipient gets a split. ADR 0030 continues to apply to the recipient picker of the editor.

## Non-Goals

- Batch edits of other fields, for example OP3 or the switch of ADR 0063.
- A recipient of a value time split.
- A repair of the one-way links.

## Invariants

- An operation does not change a recipient outside its places, or a feed outside its scope.
- An operation does not change a fee split.
- Each write of a change set uses the ID of a diff that the review showed.
- Discard puts back the draft of each feed from before the change, byte for byte.

## Verification

Mechanical:

- Core tests for items 5 to 7, 10 and 13 to 15. The test feed has a channel block, an item block, a live item block and a value time split.
- Core tests for items 17 to 20: an open draft gets the change, an outcome gives its cause, and Discard puts back each draft byte for byte.
- A core test: Add to a level gives each feed of the level, through `child_ids`.
- A core test: the write of a change set writes the other feeds after a conflict in one feed.
- A core test: Discard puts back the saved recipient of item 12.
- An API test: the number of feeds of a recipient in the list of item 2 is the number of feeds of item 3, also for a feed that has only a draft.
- A Node test: the bottom of the left panel gives Recipients, File hosts, Settings, then Lock Studio.
- The UI has no copy of the rules of items 10 and 13 to 15. A test cannot tell a feed rule from an edit aid. Thus the review of each UI packet examines each new UI function that changes the model (ADR 0065).

Visual:

- The Recipients page, the page of a recipient, each operation sheet and the review of a change set. In the two themes, at desktop width and at 390 px.
- The review during a write, with one feed that fails.
- `$edges()` of `tools/drive-ui.mjs` finds no content at the edge of a group.

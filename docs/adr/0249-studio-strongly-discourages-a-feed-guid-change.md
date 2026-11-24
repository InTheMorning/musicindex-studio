# ADR 0249: Studio Strongly Discourages A Feed GUID Change

- Status: Accepted
- Date: 2026-11-22
- Owner: operator
- Accepted: 2026-11-22 by the operator, from drawings in the questionnaire. The write stops until the user types the title. An outside change gets a Warning with **Put Back** and **Keep the New GUID**. The move with **Replace the File**, a new feed at a used address, the add by address and the answer `superseded_by` also tell the user.
- Technical choices: the main session selected the known GUID of item 1 and the position of the guard in the write. It also selected the refusal of a grant write, the texts, and the behavior when the read of item 9 fails.
- Changes: [ADR 0248](0248-music-index-changes-a-feed-guid-in-place-and-studio-restores-no-guid.md) item 2. Studio puts back an earlier GUID when the user selects **Put Back** for an outside change (item 6). Studio makes no restore on its own.
- Changes: [ADR 0175](0175-the-check-shows-what-music-index-holds.md) item 4. An answer 404 with `superseded_by` gives the finding of item 11, not the line "Music Index does not hold this feed yet."
- Uses: [ADR 0008](0008-feed-types-and-feed-identity.md) item 2, [ADR 0122](0122-a-new-user-starts-with-a-default-file-host.md) item 8, [ADR 0174](0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md), [ADR 0255](0255-publisher-feeds-announcements-and-show-grants.md), [ADR 0247](0247-studio-restores-a-changed-feed-guid-and-corrects-a-link-with-an-old-guid.md) items 7 to 9. Stophammer ADR 0106 owns the behavior of the index for a GUID change, and Stophammer ADR 0088 owns `GET /v1/feeds/by-url`. This record cites them and does not restate them as rules of Studio.
- Source: the request of the operator on 2026-11-22 that Studio strongly discourages a GUID change. The search of the code paths by ChatGPT, examined by the main session.
- Bar: crosses modules (collection, pipeline, check, move, add, UI). Without this record, an agent can add a path that writes a different GUID with only an ordinary line in Review.

## Context

A feed GUID identifies a feed. Other music feeds name it in their value blocks and links. Under Stophammer ADR 0106, Music Index changes a GUID in place, and a request for the old GUID gives 404 with `superseded_by`. A payment route never follows `superseded_by` (Stophammer ADR 0106, Invariants). Thus each GUID change makes the value blocks of other feeds stale. On 2026-09-11 a publisher tool changed four Elijah Lied GUIDs in error (ADR 0247, Context).

These facts of the code on 2026-11-22 apply. The main session examined each one.

- The typed editor refuses a change of an existing GUID (`keep_identity` in `crates/core/src/pipeline.rs`).
- The write compares no GUID. A draft with a different GUID gets an ordinary line in Review and writes.
- The merge of a changed hosted file (`crates/core/src/merge.rs`, `merge_draft`) treats `podcast_guid` as an ordinary field.
- **Versions → Put in the Draft** (`draft_from_snapshot`) puts the XML of an earlier version in the draft with no identity check.
- The Discard of a change set restores the saved draft bytes.
- `record_state` in `crates/core/src/collection.rs` sets `feeds.podcast_guid` to the GUID of each read. A read after an outside change thus takes the new GUID with no message.
- A move with **Replace the File** writes the moved feed over a different feed. Review says "This move replaces a different file." and gives no GUID.
- The add by address skips an address of the collection with a different GUID, with no message.
- The check treats 404 with `superseded_by` as a feed that the index does not hold.
- A grant write (ADR 0255) starts from the hosted file.

## Decision

### The Known GUID

1. Each feed of the collection has a known GUID. It is the GUID that Studio made for the feed, or the GUID of the first read. A read of the hosted file does not change it. Only these events change it: a placeholder gets the real GUID (ADR 0122 item 8), a confirmed write of item 4, and **Keep the New GUID** of item 6. A collection that exists before this record takes the current `feeds.podcast_guid` as the known GUID.

### The Write Stops

2. A write is a GUID change when the `podcast:guid` of the draft is not the known GUID. A placeholder GUID is not a GUID change. The rule applies at the write in the core, so it covers each path: the merge, a restored version, the Discard of a change set, an import and each future path.
3. Review of a GUID change shows a panel with the level Stop:
    - Title: "This write changes the GUID".
    - The two GUIDs: "Known: {known} → New: {new}".
    - The consequences: "Payments from other music feeds that name {known} stop." and "Apps can show the feed as a new show."
    - The button **Keep the Old GUID**. It sets the `podcast:guid` of the draft to the known GUID.
    - The field "Type {title} to change the GUID". The write button stays off until the text of the field, with no white space at each end, is equal to the feed title. A feed with no title uses the word `CHANGE`.
4. The write request carries the confirmation. The core refuses a GUID change with no confirmation, so a request that does not come from the panel also stops. After a confirmed write, the known GUID is the new GUID.
5. A grant write (ADR 0255) has no user to confirm. The core refuses a grant write that is a GUID change.

### An Outside Change

6. When a read gives a GUID that is not the known GUID, the tab **Issues** of the feed shows the finding `guid_changed_outside`:
    - Level: Warning.
    - Title: "The GUID changed outside Studio".
    - Reason: "Studio knows this feed as {known}. The hosted file now has {new}. Payments from other music feeds that name {known} stop. A second change has the same effect for feeds that name {new}."
    - The button **Put Back {known}** sets the `podcast:guid` of the draft to the known GUID. The draft then is not a GUID change, and the write needs no confirmation.
    - The button **Keep the New GUID** sets the known GUID to the new GUID. The finding then goes.
    - The GUIDs in the button labels show the first 8 characters. The reason shows each GUID in full.
7. Until the user selects a button, a write of the draft with the new GUID is a GUID change, and item 3 applies.

### The Other Paths

8. A move with **Replace the File** gives a panel with the level Stop in the move review. It says: "The address served the GUID {old}. It then serves {new}. Payments that name {old} stop." The write needs the typed title of item 3.
9. Before the first write of a feed that Studio has never written, Studio reads `GET /v1/feeds/by-url` with the public address. When a `feed_url` match has a different `feed_guid`, Review shows a panel with the level Stop: "Music Index has a different feed at this address", with that GUID. The write needs the typed title of item 3. A failed read gives the line "Studio could not ask Music Index about this address." and does not stop the write.
10. The add by address gives a Notice for each address of the collection with a different GUID: "{address} is in the collection already with a different GUID." Studio adds nothing for that address, as before.
11. When `GET /v1/feeds/{guid}` gives 404 with `superseded_by`, the check gives the finding `index_guid_changed` in place of the line of ADR 0175 item 4:
    - Level: Warning.
    - Title: "Music Index changed the GUID of this feed".
    - Reason: "Music Index now holds this feed as {new}. This draft has {guid}."
    - The finding has no button.

## Open Questions

- Other reads that do not follow `superseded_by` stay as ADR 0248 gives (Open Questions).
- The item GUIDs are out of scope. `keep_identity` keeps them in the editor. A merge or a restored version can still change them.

## Alternatives Considered

- Refuse each GUID change, with no way to confirm. Not chosen by the operator: a real need stays possible, but only with a deliberate step.
- A Warning with one click to write. Not chosen by the operator: too weak.
- For an outside change, a Warning with no restore, as ADR 0248 item 2 gives. Not chosen by the operator: the user decides, and the reason states the cost of a second change.
- Compare the draft with the hosted file in place of a known GUID. Rejected, because a read after an outside change makes the hosted file the reference, and the change then gives no stop.

## Checks

Mechanical:

- A core test for each path of item 2: a merge, a restored version and the Discard of a change set that give a different GUID stop the write with no confirmation, and write with it. The known GUID then is the new GUID.
- A core test of item 1: a read with a new GUID does not change the known GUID, and gives `guid_changed_outside`. **Put Back** and **Keep the New GUID** give the results of item 6.
- A core test of item 5: a grant write with a different GUID is refused.
- A test of item 8 and item 9 with scripted index answers, and a test of item 10 for the add by address.
- A test of item 11 with a scripted answer 404 with `superseded_by`.
- Node tests of the panel model: the write stays off until the typed text is equal to the title.

Visual, by the main session with screenshots:

- The Stop panel of item 3 in Review.
- The Warning of item 6 on the tab **Issues**.
- The Stop panel of item 8 in the move review.

## Consequences

- Each path that can change a feed GUID now stops at the write. A new path needs no guard of its own.
- The tutorial data of Elijah Lied has the new GUIDs as known GUIDs, because those collections exist before this record (item 1).

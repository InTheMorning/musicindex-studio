# Research Request: A Feed Check With Repairs

## Status

Received - 2026-11-04. This request comes from the research repository at
`../musicindex-research`. That repository has no Git remote, and it is private.
Its index is `AGENTS.md`. The note is
`research_notes/studio/improving-the-state-of-feeds.md` in that repository. The
scenario numbers S7 to S23 come from
`research_notes/aggregation/itunes-new-feed-url.md` in that repository.

This document binds nothing in Studio. Studio records each decision in its own
ADR. [ADR 0126](../adr/0126-studio-refuses-a-bad-move-link.md) gives the decision
for F0. It differs from this request in items 5, 7, 12 and 26.

The research examined Studio at commit `a1b8220` and the Stophammer backup
`stophammer-20261031T234835Z.db`. It read a part of the ADRs and of
`crates/core`. A search can miss a check with an unusual name, so examine each
gap below before a decision.

Studio corrected this request on 2026-11-04, after a review against the code at
commit `4279546`. The text marks each result of that review as **Studio review**.
The research note has the same corrections.

## Purpose

| # | Change | Why |
|---|---|---|
| F0 | Refuse a write with a loop or another bad move link | The operator requirement of 2026-11-04 |
| F1 | One feed check in the core, with a repair through the draft | The search found no general check of a feed |
| F2 | Write `lastBuildDate` | `crates/` has no code for this element |
| F3 | Show the index view of a feed | A publisher cannot see what the index holds |
| F4 | Repair many feeds in one change set | One tool or one label can have the same gap in many feeds |
| F5 | A public read-only check | It reaches publishers on other tools |
| F6 | Publish the check rules | Other tool authors can correct their output |

The research recommends F0 to F3 first.

## F0: Studio Refuses A Bad Move Link

On 2026-11-04 the operator stated this requirement: a legitimate move always
works. A loop or another problematic link fails, and Studio detects and refuses
it.

The core reads each URL that a write names. It follows each named URL to the
end, with a limit of 10 hops. A body that names only its own URL, or names
nothing, is the end of a chain. The core refuses the write in these conditions:

| # | Condition | Example |
|---|---|---|
| 1 | The chain comes back to a URL that it visited, other than a body that names its own URL | A names B and B names A |
| 2 | The destination does not answer with a feed | HTTP 404, or an HTML page |
| 3 | The destination gives a different `podcast:guid`, or no GUID | A different album |
| 4 | The tag and the self link of one body name two different URLs, other than the URL of that body | Scenario S15 |
| 5 | The index holds the destination as the source of a different GUID | Scenario S21 |
| 6 | The self link differs from the public address, outside a move | A copied feed |
| 7 | The address is not a public web address | A private address |

**Studio review:**

- **Condition 1.** In the backup, each of the 6 feeds with the tag names its own
  `feed_url`. In the sample of the tag note, 559 of 621 tags name the exact URL
  of the feed. A self-reference is thus the normal form, and it is not a loop.
- **The time of the read.** Before a write, the new host has no feed. A read at
  that time refuses each move through condition 2. The read of the destination
  must occur after the feed is on the new host, and before the pointer.
- **The sequence today.** Studio writes the pointer (`pipeline.rs:3436`) and then
  checks the address (`pipeline.rs:3482`). That check reads a status and not a
  body. Studio can thus write a pointer to an address that fails.
- **A repair after a refusal.** `write_to_new_host` continues only with the same
  bytes (`pipeline.rs:4076`). A repair changes the bytes, so the next write gets
  `FileExists`.
- **Condition 5.** The index API has no lookup of a feed by URL at Stophammer
  commit `d807f8b`. This condition needs a new route.
- **Condition 6.** The public address is the address after each redirect
  (ADR 0021 item 1). A correct feed behind a `301` can thus have a different self
  link. ADR 0021 item 5 says that the user decides on a difference.
- **Condition 7.** The existing network policy does not give this condition on
  the desktop. `NetPolicy::Open` permits each address.

The refusal gives the reason and the URL of each hop. The write stays in the
draft, and the check offers a repair when one exists.

These are not bad links, and Studio must let them pass:

- A move to a new host. ADR 0038 item 19 puts the feed on the new host first, so
  the destination can answer before Studio writes the pointer.
- A return to an earlier host. The pointer at the earlier host becomes the feed.
  Stophammer scenario S23 shows that the index follows this.
- A chain that has an end. Stophammer scenarios S13 and S16 follow it with the
  proposed rule.

**Studio review:** a return to an earlier host fails today. A probe test made a
move from A to B and then a move back to A. `set_feed_host` gave `FileExists`,
because the pointer file is at A. ADR 0038 item 20 also refuses a deletion of a
pointer file. This needs a code change.

A published feed that already has a bad link stays readable. The refusal applies
when the user writes it. The check shows the repair.

### Questions For The ADR

- **A failed read.** The operator agreed on 2026-11-04 to a refusal when the
  destination cannot be read, because a legitimate destination answers. A brief
  host failure thus stops the write until the host answers. A failed read of the
  index gives a notice and not a refusal, so that an index failure does not stop
  each write.
- **An override.** The research recommends none, because the operator asked for a
  refusal. **Studio review:** with no override, a stale row in the index stops a
  correct move until the index operator acts. The tag note lists this as a limit
  of "always".
- **A feed that Studio does not host.** A connected app or a read-only feed goes
  through the same check when it gets a write.

  **Studio review:** a granted write can occur during a live show, and the user may not be present. ADR 0054 item 16
  refuses a granted change while the feed has a draft with changes. Thus a repair
  through the draft also stops the grant. ADR 0054 items 13 and 15 permit no
  change of a link in a granted write.

The Stophammer request proposes that the node refuses a loop from any tool.
Today, scenario S3 gives 4 moves in the research probe. Studio refuses earlier
and gives the reason, which the node does not give to a publisher.

## The Limit Of The Effect

Studio can change only feeds that its users control. Of the 10,649 feeds in the
backup, 9,344 are on `wavlake.com`. Thus the direct reach is at most 1,305
feeds. The research has no data on the number of Studio users, so it does not
estimate the effect.

## The Measurements

These counts use the 1,305 feeds and 5,172 tracks outside `wavlake.com`. They
come from what Stophammer stored, so a value that the parser or the node does
not keep can look absent. The self link count can be too high for this reason.

| Measure | Count | Share |
|---|---|---|
| No `atom:link rel="self"` row | 901 feeds | 69.0% |
| `podcast:locked` absent | 974 feeds | 74.6% |
| Music feed with no publisher link | 574 of 1,139 | 50.4% |
| No track number | 1,712 tracks | 33.1% |
| No `lastBuildDate` | 370 feeds | 28.4% |
| No language | 310 feeds | 23.8% |
| GUID seen at more than one URL | 229 feeds | 17.5% |
| Enclosure with no size | 209 tracks | 4.0% |
| `http` enclosure | 140 tracks | 2.7% |
| No duration | 78 tracks | 1.5% |
| Image with `http` | 17 feeds | 1.3% |
| Public copy with a different body | 10 feeds | 0.8% |
| No `pubDate` | 32 tracks | 0.6% |

**Studio review:** a recount in the backup gives each number of this table. The
901 feeds have no `self_feed` row in `source_entity_links`. The column
`declared_self_url` is empty for 905 feeds, because 4 feeds with a row have no
value in the column.

These measures are not defects, and a check must not report them as errors:

- `split` is "The number of shares" in the namespace. A sum other than 100 is
  correct.
- A type other than `audio/mpeg` can be correct.
- A track with no image can use the image of the album.
- An absent optional field, such as `podcast:locked`, is a choice.

## F1: The Feed Check

The proposal is one core function that takes a feed and gives a list of
findings. Each finding has a level, a reason, and a repair that writes to the
draft when one exists. ADR 0065 makes the core the place for a feed rule. The
corpus can give the test feeds.

**Studio review:** `address.rs` already records each address that differs from
the public address, with the self link and the tag. The UI shows it
(`app.js:611`, ADR 0021 items 3 to 5). F1 starts from this record.

The check does not repair a feed without the user. ADR 0008 keeps the meaning of
a feed through a read and a write with no edit. The diff of ADR 0007 stays the
gate.

The first findings can be these:

- No self link, or a self link that differs from the known address, when F0 does
  not refuse it.
- An `http` enclosure or image when the `https` address serves the same file.
- An enclosure with no size or a wrong size, and a track with no duration.
- A tag or a self link with a different spelling of the address.
- A move that does not confirm: the tag names a different URL, and the body at
  that URL names a third URL.
- A GUID that differs between the local feed and the index.

A finding that needs a choice, such as the lock or the language, gives a
question and not a repair. The conditions of F0 block a write. The research
recommends that each other finding is a notice. One condition has one level, so
F1 gives only a notice where F0 does not refuse.

### A Self Link In Each Feed

A self link helps when it is correct. It states the address of the feed, and it
confirms a move to that address. A wrong self link hinders the goal, because it
can move a record in Stophammer. Scenario S8 shows the same move through the tag
with a different spelling.

**Observation:** the backup has five records with a self link that names a
different URL, and one of these URLs is the source of a different record. The
research did not examine why.

The proposal has these parts:

1. A new feed always gets a self link when its public address is known. Studio
   already gives no link to a feed with no public address.
2. An existing feed gets an offer through the draft and the diff. ADR 0008 and
   ADR 0021 do not allow a silent change to a file.
3. The self link has the exact spelling of the public address.
4. After a write, Studio reads the public address and confirms that it serves
   this feed. **Studio review:** Studio requests the public address today, but it
   reads only the status (ADR 0043 item 8). ADR 0126 adds the read of the body
   for a move. F1 can extend it to each write.
5. A copy of a feed has a self link to its own address. **Studio review:**
   `copy_feed` does this today, clears the tag, and removes the self link when
   the copy has no public address. No change is necessary for a copy.

## F2: The Build Date

**Observation:** outside the test feeds, `crates/` has no mention of
`lastBuildDate`.

**Inference:** the reader keeps the old value through each edit, so an edited
feed can give an old build time. A new feed of Studio has no value, so ADR 0053
does not protect it. This was not tested.

Stophammer ADR 0053 uses this value to reject an older copy, and an equal value
passes. Thus an edit is not rejected today. A roll back can be rejected when a
different tool had changed the date. This also was not tested.

The proposal writes a new `lastBuildDate` for each write, also for a roll back.
Test the result against the ADR 0053 rule.

**Studio review:** the ADR for F2 must decide these points:

- **A durable rule.** "The writer changes only the elements that the user
  changed" (ADR 0008, ADR 0012) does not permit a new date in each write. The ADR
  must change this rule.
- **The time of the stamp.** The write must put the approved bytes on the host
  (ADR 0007), and the retry of a move compares bytes. The core must thus put the
  date in the draft before the diff, and the diff shows it.
- **The clock.** A clock that is ahead stores a future date. ADR 0053 then rejects
  each later write with an earlier date. A date never goes behind the date in the
  feed.

## F3: The Index View

`musicindex.rs` reads feeds and routes. The search found no use of
`GET /v1/feeds/{guid}/copies`, `GET /v1/guid-changes` or
`pending_guid_change`.

**Studio review:** ADR 0052 items 4 and 8 tell the index about a move through a
podping. Studio announces the new address and the address before the move. ADR 0038 item 18
describes the time before Phase 10.

The proposal shows these differences on the page of a feed:

- The index holds a different URL. Studio gives the ways to correct this: a tag
  with a confirmed destination, a `301`, or a request to the operator.
- A public copy or a pending GUID change exists.
- The index has not seen the last write. **Studio review:**
  `GET /v1/feeds/{guid}` gives `last_build_date`, so F2 makes this comparison
  possible.

This item also serves two feeds that name themselves with one GUID. The node
keeps the first URL and gives `source_conflict` only to the crawler. Studio is
where the publisher edits the feed.

## Feed Moves And The Stophammer Request

The research sent a request to Stophammer:
`docs/plans/research-feed-move-confirmation-request.md` in that repository. It
proposes that the node follows a chain of named URLs to its end and moves nothing
on a loop. Stophammer has not decided.

**Observation:** the form of ADR 0038 passes that rule. A pipeline test shows the
tag on the old host, and no tag with the new self link on the new host.

Studio can examine two points:

1. **A second move.** ADR 0038 keeps each pointer file. After a move from A to B
   and a move from B to C, the pointer at A names B and the pointer at B names C.
   The proposed rule follows this chain, but each extra hop needs a crawl. Studio
   can update each older pointer to name the last address, so that the chain has
   one hop. This needs the older host to stay connected, and the diff must show
   the change (ADR 0038). Without the proposed rule, the chain moves through B.
   This was not tested in Studio.
2. **The new feed.** Apple says to use the tag "in your new feed to point to the
   new URL". Studio writes no tag on the new host, and Stophammer accepts this.
   The research did not test other apps. Examine if the new feed should also
   name itself.

## F4 To F6

- **F4.** ADR 0091 gives a change set for one recipient. The proposal extends the
  form to the repairs of F1. In the backup, two hosts have 131 and 105 music feeds
  with no publisher link.
- **F5.** ADR 0067 opens a read-only feed in the editor. A public page on the
  server can run F1 for any feed URL. The research did not design its fetch
  limits, and it recommends the app first.
- **F6.** Publish the findings as a plain list with the namespace source of each.
  Do this after F1 is stable.

## What The Research Does Not Propose

- A repair at import without the user.
- A public score of a publisher. Many absent fields are optional, so a score can
  name a correct feed as bad.
- A step in which Studio resolves a duplicate in Podcast Index or Stophammer.
  Studio can report it, and the owner of that index decides.

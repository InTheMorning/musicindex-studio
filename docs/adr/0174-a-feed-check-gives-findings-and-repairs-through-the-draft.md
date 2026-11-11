# ADR 0174: A Feed Check Gives Findings And Repairs Through The Draft

- Status: Accepted
- Date: 2026-11-11
- Owner: operator
- Proposed: 2026-11-11 by the planning model, on the operator decision of 2026-11-11 that lifts the stop on F1. The orchestrator session gave that decision.
- Accepted: 2026-11-11 by the operator. The operator confirmed item 17 on 2026-11-11 on the mockup of packet 109c in `.studio-dev/task-109/out/`. On 2026-11-11 the operator amended item 19: a read-only feed does not show the text of Form A.
- Technical choices: items 4, 9, 13, 15 and 18 are choices of the planning model.
- Source: [research request on a feed check with repairs](../plans/research-feed-check-request.md), item F1 and the section "A Self Link In Each Feed". Research note `research_notes/studio/improving-the-state-of-feeds.md` in `../musicindex-research`, recommendation 1.
- Uses: [ADR 0007](0007-draft-diff-write-and-announce.md), [ADR 0008](0008-feed-types-and-feed-identity.md), [ADR 0021](0021-public-address-of-a-feed.md), [ADR 0023](0023-web-addresses-and-audio-file-facts.md), [ADR 0065](0065-the-core-applies-each-feed-rule.md), [ADR 0126](0126-studio-refuses-a-bad-move-link.md), [ADR 0130](0130-the-facts-of-an-audio-file-stay-locked.md), [ADR 0135](0135-the-panel-holds-the-content-and-the-page-has-tabs.md).

## Context

The research found no general check of a feed in Studio. `review.rs` describes the changes of a draft, and it does not examine the result. `address.rs` records each address that differs from the public address, and the UI shows it (ADR 0021 items 3 to 5). ADR 0126 refuses a bad move link at the write. Its item 6 gives a notice in Review for a self link that names a different URL.

The Stophammer backup of 2026-10-31 gives these counts for the 1,305 feeds outside `wavlake.com`. A Studio review counted them again.

| Measure | Count |
|---|---|
| No `atom:link rel="self"` | 901 feeds |
| No language | 310 feeds |
| Enclosure with no size | 209 tracks |
| `http` enclosure | 140 tracks |
| No duration | 78 tracks |
| Image with `http` | 17 feeds |

On 2026-11-04 the operator stopped F1 until Stophammer had its findings on a move. Stophammer ADR 0083 is Accepted, and Stophammer release 0.18.1 holds it. On 2026-11-11 the operator lifted the stop.

These facts of the code on 2026-11-11 apply:

- `pipeline::new_feed` gives a new feed with a public address its GUID, but no self link. A feed with a placeholder GUID gets a self link when it gets its public address (ADR 0122 item 8, `apply_feed_change`).
- `copy_feed` gives the copy a self link to its own address, or removes the link when the copy has no public address.
- `move_link::Repair::UsePublicAddress` sets the self link of a draft to the URL of the draft, through the draft and the diff.
- ADR 0023 item 5 reads the size and the type of an audio file with a `HEAD` request.

On 2026-11-04 the operator selected Form A of the mockups in `.studio-dev/mockup-feedcheck/`. In Form A, an outline row "Feed check" with a count opens its own page. ADR 0135 then moved the pages of a feed from the outline to tabs. The outline now holds only the head, the tracks and "Add tracks".

## Decision

### The Check

1. The core module `feed_check` has one function. It takes a feed, its public address and the facts of its reads, and it gives a list of findings. The function sends no request. It applies to each feed that Studio opens in the editor, also a read-only feed (ADR 0067).
2. Each finding has a kind, a level, a title, a reason and the entry that it is about. It has a repair when one exists.
3. The levels are **Notice** and **Question**. A notice has a repair or a button that opens the field. A question needs a value from the user, and it has no repair.
4. The core gathers the facts in a different step from the check. The local facts come from the draft and the collection. The network facts come from reads with `NetPolicy::Public` (items 13 and 14).
5. A finding never stops a write. One condition has one level. When ADR 0126 refuses a write for a link, the check gives no finding for that link, and Review shows the refusal.
6. The check changes no feed. A repair changes the draft only when the user selects its button. Each repair goes through the draft and the diff (ADR 0007), and the user reviews it before the write. A read and a write with no edit keep the bytes of the source (ADR 0008).

### The Findings

7. The first set of findings is this table. A new finding needs a change of this record.

| Kind | Level | Condition | Repair |
|---|---|---|---|
| `no_self_link` | Notice | The feed has a public address and no self link. | **Add the Link** (item 10) |
| `self_link_differs` | Notice | The self link differs from the public address, and the draft does not change it. | **Use the Public Address** (item 10) |
| `self_link_spelling` | Notice | The self link or the tag differs from the public address only in its spelling (item 12). | **Use the Public Address** (item 10). The repair keeps the tag, because ADR 0128 gives a moved feed a tag that names its own address. |
| `http_file` | Notice | An enclosure or an image has an `http` address, and the `https` address gives the same file (item 14). | **Use the https Address** |
| `no_file_size` | Notice | An enclosure has no `length`, or `length` is 0. | **Read the Size** (item 15) |
| `wrong_file_size` | Notice | The host gives a `Content-Length` that differs from `length`. | **Read the Size** (item 15) |
| `no_duration` | Notice | A track has no `itunes:duration`. | **Open the Track**. The track page shows the notice of ADR 0130 item 3. |
| `move_not_confirmed` | Notice | The tag names a URL, and the body at that URL names a third URL (item 13). | None. The finding gives each URL of the chain. |
| `no_language` | Question | The feed has no `language`. | **Select a Language** opens the field. |

8. These conditions give no finding, because the research shows that they are not defects:
    - A split sum other than 100. `split` is "The number of shares".
    - A media type other than `audio/mpeg`.
    - A track with no image.
    - An absent optional element, such as `podcast:locked`.
9. A GUID that differs between the local feed and the index gives no finding in this record. It needs a lookup of a feed by URL in Stophammer, as ADR 0126 condition 5 does. It belongs to the index view, F3 of the request.

### The Self Link

10. **Add the Link** and **Use the Public Address** first read the public address. **Use the Public Address** sets each link of its finding to the exact text of the public address. The repair changes the draft only when the body there is an RSS feed with the `podcast:guid` of the draft. Otherwise the repair changes nothing and gives the reason. A wrong self link can move a record in Stophammer, so Studio does not write a link that it did not confirm.
11. A new feed with a public address gets a self link when `pipeline::new_feed` makes it. The link has the exact text of the public address. A feed with a placeholder GUID keeps the rule of ADR 0122 item 8. A copy keeps the rule of `copy_feed`.
12. The self link has the exact spelling of the public address. The core compares two URLs as exact text (ADR 0126 item 2). Two URLs differ only in spelling when they are equal after these changes: lowercase the scheme and the host, remove the default port, and give an empty path the path `/`.

### The Reads

13. `move_not_confirmed` reads the URL of the tag with the chain reader of ADR 0126, with its limit of 10 hops. The finding shows when the chain has an end at a URL that is not the URL of the tag.
14. `http_file` reads the `https` address of the file. The two addresses give the same file when the two `Content-Length` values are equal and the first 64 KiB of each are equal. When a server does not answer a range request, the check gives no finding for that file.
15. **Read the Size** sends the `HEAD` request of ADR 0023 item 5, and it puts `Content-Length` into `length`. The value goes into the draft as a locked fact (ADR 0130).
16. The local findings come again at each change of the draft. The reads run when the user opens the check, and when the user selects **Check Again**. A failed read gives no finding and a notice: "Studio could not read this address."

### The Page

17. Form A goes into the tabs of ADR 0135. The page of an album, an artist, a label or a playlist gets the tab **Check** before **Settings**. The tab label has the count of the findings, for example "Check 5". With no finding, the label is "Check", with no count. At 760 px or less, the select menu of the tabs gives the same label (ADR 0135 item 6).
18. The count of the tab is the count of the local findings and of the last reads in this session of Studio. The API keeps the last read facts of each feed in memory until the feed changes or Studio stops.
19. The page has the text of Form A first: "Studio does not change the feed until you select a button. Each change goes to the draft, and you review it before the write." Then each finding is one row with its level, its title, its reason and one button. A read-only feed has no button (item 20), so its page does not show this text.
20. A read-only feed shows its findings with no button. A connected app or a show grant can wait for a draft: a granted change stops while the feed has a draft with changes (ADR 0054 item 16). When the feed has a grant, the page says this above the rows.
21. A page with no finding shows "Studio found no problem in this feed." and **Check Again**.

## Alternatives Considered

- Each finding blocks the write. Not chosen: the research recommends a notice, and the diff stays the gate. ADR 0126 gives the only refusals.
- A repair without the user at import. Not chosen: ADR 0008 keeps the meaning of the source, and the research advises against it.
- A question for `podcast:locked`. Not chosen: an absent lock is a choice, and the research names it as not a defect.
- An outline row, as in Form A. Not chosen: since ADR 0135, the outline holds only the head, the tracks and "Add tracks".
- A read of each address at each open of a feed. Not chosen: a feed with many tracks would send many requests at each open.

## Consequences

- The check of a person (ADR 0111) stays a different check. Both use the same form of a finding row.
- A repair through the draft stops a granted change until the write. The page says this.
- F4 of the request can extend the change set of ADR 0091 to these repairs. F5 can run this check on a public page. Each needs its own record.
- A new finding changes item 7 of this record and its tests.

## Acceptance Criteria

Mechanical, in the tests of `studio-core`, `studio-api` and the Node tests of the UI models:

- A core test for each kind of item 7: a feed with the condition gives the finding with its level and its repair, and a feed without the condition gives none.
- A core test for each condition of item 8: the condition gives no finding.
- A core test: the check of each corpus feed sends no request and changes no byte of the feed.
- A core test: **Add the Link** with a body that has a different `podcast:guid` changes no draft and gives the reason.
- A core test: a new feed with a public address has a self link with the exact text of that address. A new feed with no public address has no self link.
- A core test: the spelling rule of item 12 for each of its three changes, and two URLs with a different path are not a spelling difference.
- A core test: when ADR 0126 refuses a link of the draft, the check gives no finding for that link.
- A core test: `http_file` gives no finding when the server does not answer a range request.
- An API test: the route of the check gives the findings, and the route of a repair changes the draft and gives the new findings.
- An API test: a read-only feed gives its findings with no repair.
- A Node test: the tab label has the count, and the model of the page gives one row for each finding.
- The four commit checks.

Visual, by the planning model with `tools/drive-ui.mjs`:

- The tab **Check** with its count at 1280 px and at 390 px.
- The page with the five rows of Form A, and the page with no finding.
- `$edges()` gives no result. At 390 px, `$fieldText()` and `$overflow()` give no result.

Human, by the operator. This gate stays open until it runs:

- In the AppImage, a repair of a real feed goes to Review, and the write puts it on a real file host.

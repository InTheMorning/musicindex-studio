# Findings From The Tutorial For A Hosting User

On 2026-11-22, the main session made a tutorial for a person who hosts many artists on one DreamHost site. The tutorial changes the payees of about 45 feeds to LNURL addresses, and moves three artist pages below a new hosting page. The main session ran each step on a test copy of the public feeds (`.studio-dev/tj-tutorial`, ignored). No feed was written to a server.

This file records the possible improvements. It states no rule. Each fix needs its own decision or task.

## Findings

1. **A page move leaves many drafts, and no control writes them together.** The move of The Doerfels gave 21 release drafts and one hosting page draft. The move sheet has no write button (`crates/api/ui/editor.js:2805`). Only a change set has a write button for many feeds (`crates/api/ui/app.js:3375`). Review of a release writes its linked pages only before the first write of the release (`crates/api/src/routes.rs:4509`). Thus the user opens Review for each of about 38 feeds.

   Proposal: a page move makes a change set, as a payee batch does, with **Write N Feeds** and **Discard**.

2. **The move error does not name the feed.** When one release is read-only, the move sheet shows "Studio cannot change this feed. It is read-only." The move of 21 releases does not tell which release is read-only. Proposal: name each read-only release, and give the button that connects its file host.

3. **The folder scan misses feeds at a large folder depth or in a large folder.** A file host list goes to a depth of 4 folders and stops at 1000 files (`crates/core/src/connector.rs:78` and `:80`). A WordPress site keeps its feeds in `wp-content/uploads/2023/08/`, which is below that depth. Three feeds stayed read-only until a second file host used the folder `wp-content/uploads`. A DreamHost site folder with media files can also have more than 1000 files. The File hosts page tells the user nothing about the feeds that it did not find. The hint for a truncated list (`crates/api/ui/app.js:1821`) is on a different page. Proposal: after a match, name the feeds that stay read-only, and tell when the scan stopped at a limit.

4. **An artist feed with no role has no Move To….** The three artist feeds have no `rel` role, so Studio shows them under "No role" with the type "Feed". **Move To…** shows only for the role artist or label (`crates/api/ui/app.js:2595`). The user must find the Role field in the page form first. Proposal: show **Move To…** on a publisher feed with no role, and set the role Artist with the move.

5. **The move sheet shows "Changes 21 releases:" with no lines.** When Label and Hosting are both "None", the sheet shows a colon and an empty list. Proposal: show "Select a label or a hosting page." until a value changes.

6. **The batch sheet selects feeds that do not change.** "Use the LNURL Address" for Middle Season selected 8 feeds. Review had 6 feeds, because 2 feeds already paid the address (outcome `no_node_recipient`). Proposal: the sheet tells "2 feeds pay this address already" and does not count them in **Review in N Feeds**.

7. **A URL breaks inside a word.** The fit check found these (musicindex ADR 0065 R-FIT-3): at 390 px, `https://www.doerfelverse.com/` on the File hosts card; at 761 px, `https://sirtjthewrathful.com/tracks/thoughtitwasreal.xml` on a release page. Proposal: give each URL element the attribute `data-fit-allow-break`, as the check message tells.

8. **The hosting form shows "Artist or label" as its type.** The sheet "Make a new hosting page" has Type "Artist or label" and Role "Hosting". The two fields seem to disagree. Proposal: name the type "Publisher page", or hide the field when the role is set.

## Findings Of The Second Walk

The main session ran the tutorial again after commit `a0ed27c`, in `.studio-dev/v-230` with the data copy `data-tut`. In that copy, the three artist pages have no role, the collection has no hosting page, and the folders of sirtjthewrathful.com are not connected.

9. **The role of a page does not go into an artist feed if its links have no medium.** "Move Artists Here…" gave the three pages the role artist, but the change set holds only The Doerfels and Middle Season. `set_roles` in `crates/core/src/publisher.rs` puts the `rel` only on a `podcast:remoteItem` with a release medium, such as `medium="music"`. The six links of `elijahliedpubfeed.xml` have no `medium`, so the file of Elijah Lied gets no role, but its linked releases change. Proposal: when a link has no medium, use the medium of the feed in the collection that the link names.
10. **The feed check does not find an address that is not full.** The release "Bloodshot Lies - The Album" has credits with the information address `www.doerfelverse.com`, with no `https://`. The Check tab shows no finding. Studio finds the fault only at the write, and refuses that feed with the code `bad_address`. Proposal: a finding with the repair "Use the https address" for each such field.
11. **The refusal of a bad address names model paths.** The write message is "these address fields are not full http or https addresses: items/0/persons/0/href …". A user does not know what `items/0/persons/0/href` is. Proposal: name the track, the credit and the field, for example "Track 1, credit Ben Doerfel: Information address".
12. **The review of a role change repeats one line.** The review of The Doerfels shows "Remote items: 18 entries get the relation artist", then the line "Relation: none → artist" for each entry. Proposal: show the count line only.

The write of the new hosting page gave `not_reachable` (404) in the test, because the test does not put the file on the public site. This is a limit of the test, not a finding.

## Open

- [ADR 0243](../adr/0243-a-move-is-a-change-set-and-a-level-gathers-its-pages.md) gives findings 1, 2, 3, 4 and 6. Tasks 229 to 232 build ADR 0243 and findings 5, 7 and 8. They are committed, the last in `9b1a300`.
- [ADR 0245](../adr/0245-review-repairs-short-addresses-and-offers-the-people-values.md) gives findings 10 and 11. Each review repairs an address with no `https://` before the write, and offers the values of People. No task builds it now.
- Findings 9 and 12 have no decision now. The tutorial tells the user how to correct the fault of finding 10 by hand.

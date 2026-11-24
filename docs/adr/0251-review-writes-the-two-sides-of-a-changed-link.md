# ADR 0251: Review Writes The Two Sides Of A Changed Link

- Status: Accepted
- Date: 2026-11-23
- Owner: operator
- Accepted: 2026-11-23 by the operator, from drawings in the questionnaire. The check box of a linked feed is on and locked. The rule applies from the two sides of a link: the review of a page and the review of a release.
- Changed: 2026-11-23, items 10 and 11 added after observation 2 of [the desktop check walk of 2026-11-23](../reviews/2026-11-23-desktop-check-walk.md). The operator selected the Stop in Review from drawings in the questionnaire. Item 9 gets the field `refusal`.
- Changed: 2026-11-23, after the check of task 243, item 10 gets the choice "New artist page…". Items 12 and 13 are new. The operator asked for the row of the new page with a check box, and for a choice that makes an artist page in the menu. Item 9 gets the field `via`.
- Technical choices: the main session selected the condition of item 2 and the removed link in item 3. It also selected the order of item 7, the API field of item 9 and the texts.
- Changes: [ADR 0231](0231-a-new-release-writes-its-artist-page-and-the-files-of-a-restored-release.md) item 7. The rows are not only in Review of a new release.
- Changes: [ADR 0234](0234-a-new-release-is-written-with-its-pages-and-a-one-sided-link-is-reported.md) item 3. The locked check box is not only for the first write of a new release.
- Changes: [ADR 0218](0218-a-page-link-in-review-and-a-suggestion-starts-with-no-person.md) items 3 and 6. The menu of the refusal panel always shows, and its last choice makes an artist page.
- Uses: [ADR 0231](0231-a-new-release-writes-its-artist-page-and-the-files-of-a-restored-release.md) items 8 and 9, one write button and the stop after a failed write. [ADR 0234](0234-a-new-release-is-written-with-its-pages-and-a-one-sided-link-is-reported.md) items 5 to 8, the report of a one-sided link. [ADR 0255](0255-publisher-feeds-announcements-and-show-grants.md) items 6 and 7, a link changes the two drafts. [ADR 0007](0007-draft-diff-write-and-announce.md), the write of a draft.
- Source: [the UI findings of 2026-11-23](../reviews/2026-11-23-ui-findings.md), finding 9.
- Bar: crosses modules. Item 2 is in the API, and items 4 to 8 are in the UI. Without this record, an agent can keep the rows only for a new release, and a link of two written feeds then needs two reviews.

## Context

- A link of a page and a release changes the two drafts (`publisher::link` and `publisher::unlink` in `crates/core/src/publisher.rs`). The page gets a `podcast:remoteItem` for the release. The release gets the page in `podcast:publisher`.
- `also_write_pages` in `crates/api/src/routes.rs` gives the rows only for a release that was never written. It gives no row in Review of a page.
- An artist page adds an existing release in the section **Releases**. Review of the artist page shows only the artist page. The change "Artist or label: Added" stays in the draft of the release. The user must open the release and write it in a second review.
- The same occurs from the release. **Artist or label** on a written release names a written artist page. Review of the release writes only the release, and the artist page needs a second review.
- When only one side is written, apps show a link on one side only. ADR 0234 item 5 reports this for a release that names a page. It gives no report for a page that lists a release that does not name the page.

## Decision

### The Linked Feeds Of A Review

1. Review of a feed shows a row for each linked feed of item 2. The page of a link and the release of a link both get the rows. A new release keeps its rows (ADR 0231 item 7).
2. A feed B is a linked feed of the review of feed A when these conditions are true:
   - A is a page and B is a release, or A is a release and B is a page.
   - B has a draft.
   - The link between A and B in the draft of B is not the same as in the written file of B. The written file is the file of ADR 0234 item 7. A feed with no written file has no link in it.
3. Thus a new link and a removed link both give a row. A draft of B with other changes and no change to this link gives no row.

### The Rows

4. Each row has the check box "Also write {title}". The check box is on and locked, as in ADR 0234 item 3. Review lists each change of the draft of B under the row, also the changes that are not the link.
5. The text under a row tells the change of the link:

   | Review of | Row | Link | Text |
   | --- | --- | --- | --- |
   | a release | an artist page | added | "Adds the link to {release} on the artist page." |
   | a release | a label or hosting page | added | "Adds the link to {release} on {page}." |
   | a page | a release | added | "Adds {page} as the artist or label of {release}." |
   | a release | an artist page | removed | "Removes the link to {release} from the artist page." |
   | a release | a label or hosting page | removed | "Removes the link to {release} from {page}." |
   | a page | a release | removed | "Removes {page} from the artist or label of {release}." |

6. The text under the rows is "A link is written on its two feeds." It replaces the text of ADR 0234 item 3.

### The Order Of The Writes

7. One press of **Write to the File Host** writes each release first and each page after it. In Review of a page, Studio writes the release rows, then the page. In Review of a release, Studio writes the release, then the page rows. Each feed keeps its own diff, its own signed events and its own write (ADR 0007).
8. ADR 0231 item 9 applies to each write. When a write fails, Studio writes no feed after it, and Review names the feed of the error. The feeds before it stay written. A release is first, so a failed page write gives the one-sided link of ADR 0234 item 5, with its report and its repair.

### The API

9. The diff route gives `also_write` for each review, with the feeds of item 2. Each entry has `id`, `title`, `side` (`page` or `release`), `artist`, `change` (`added` or `removed`), `refusal` and `via`. `refusal` is the page refusal code of the draft of B (`no_artist_page` or `no_publisher_page`, ADR 0218), or `null`. `via` is the `id` of the release of item 12, or `null`. The UI uses `side` for the order of item 7 and `artist` and `change` for the text of item 5.

### A Linked Release With No Page

10. A row with a `refusal` stops the review before the write, because the write of B fails (ADR 0256 item 18). The usual cause is the removal of the only artist page of a release from that page. Review shows a Stop panel above the changes, one for each such row:

    - The title is "{release} has no other artist page." for `no_artist_page`, and "{release} has no other artist, label or hosting page." for `no_publisher_page`.
    - The line is "A release needs an artist page." or "A release needs a page."
    - The select menu **Artist page** (or **Page**) lists the pages of the collection that the release can link, except A. Its last choice is "New artist page…". **Link and Continue** links the selected page to the release and loads the diff again.
    - With "New artist page…" selected, **Link and Continue** opens the form that makes an artist page, with an empty name. After the page is made, Studio links it to the release and opens Review of A again.
    - **Keep the Release** links A to the release again and loads the diff again.

11. While a Stop panel of item 10 shows, **Write to the File Host** is off, and the line above it is "Link a page to {release} first." Review of a release has no such panel, because a page has no page refusal.

### The New Page Of A Linked Release

12. In Review of a page A, each page C whose link to a release row B changed is also a row, except A. The usual cause is **Link and Continue** of item 10. The entry of C has `side` `page` and `via` the `id` of B. Its check box is on and locked (item 4), and Studio writes it after B (item 7). The text under the row is "Adds the link to {B} on {C}." or "Removes the link to {B} from {C}."

### A New Artist Page From The Refusal Panel

13. The refusal panel of ADR 0218 in Review of a release always has the menu. Its last choice is "New artist page…". With that choice selected, **Link and Continue** opens the form of ADR 0218 item 6. The button **Make an Artist Page…** is removed.

## Item Map

| Earlier item | Change by this ADR |
| --- | --- |
| ADR 0231 item 7 | Items 1 and 2: the rows are in Review of each feed with a linked feed, not only of a new release. |
| ADR 0234 item 3 | Items 4 and 6: the check box is on and locked for each linked feed. The text under the rows changes. |
| ADR 0218 item 3 | Item 13: the menu has the last choice "New artist page…". |
| ADR 0218 item 6 | Item 13: the menu always shows. The choice "New artist page…" replaces the button **Make an Artist Page…**. |

## Consequences

- One review writes the two sides of a new link and of a removed link, from the page and from the release.
- The draft of a linked feed goes out complete. An edit of a release that the user did not finish goes out with the link. Review shows it under the row.
- The draft of a linked feed can have a link to a third feed. Review writes that link as it is. Review of a page offers the third page (item 12). Review of a release does not offer a third feed. The report of ADR 0234 item 5 shows a one-sided link that results.
- A page that lists a release that does not name the page still has no report. Item 7 makes this result less frequent, because the release is written first.

## Alternatives Considered

- The check box is on at the start, and the user can turn it off. Not chosen by the operator: then a link goes out on one side, with no report.
- The check box is locked only when the draft of B has only the link. Not chosen by the operator: one more rule to learn.
- The rule applies only to Review of a page. Not chosen by the operator: the second review then stays for a link made from the release.
- The rows for each linked feed with a draft change, as in ADR 0231 item 7. Not chosen: an unrelated draft of an artist page then goes out with each release write.

## Checks

Mechanical:

- API test: the diff of a written page gives `also_write` with a written release whose draft adds the link, with `side` `release` and `change` `added`.
- API test: the diff of a written release gives `also_write` with a written artist page whose draft adds the link, and with `change` `removed` after an unlink.
- API test: a page draft with a change that is not the link gives no row in the review of its release.
- API test: a new release keeps its rows.
- UI test (`crates/api/ui-tests/`): `alsoRows` gives each row locked, with the text of item 5 for each line of the table.
- UI test: the write order gives the release rows before the page, and the page rows after the release.
- UI test: a failed write of a release row writes no page, and names the release.
- API test: after the removal of the only artist page of a written release from the page, the diff of the page gives the release with `refusal` `no_artist_page`.
- UI test: the view model of Review gives the Stop panel of item 10 and the write off for a row with a `refusal`.
- API test: after the link of a second artist page to the release of item 10, the diff of A gives the second page with `via` the `id` of the release.
- UI test: `alsoRows` gives the row of item 12 locked, with its text.
- UI test: the refusal panel of item 13 and the Stop panel of item 10 give the choice "New artist page…" with and without other pages.

Visual, by the main session with screenshots at 1100 px and 390 px:

- Review of an artist page with one linked written release.
- Review of a written release with its artist page.

Visual, by the main session: the Stop panel of item 10 in Review of an artist page at 1100 px and at 390 px. The row of item 12 after **Link and Continue**, and a write of A, B and C.

Human check: the next desktop check adds an existing release on an artist page. It writes the two feeds in one review and reads the two written files.

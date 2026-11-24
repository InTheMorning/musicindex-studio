# ADR 0251: Review Writes The Two Sides Of A Changed Link

- Status: Accepted
- Date: 2026-11-23
- Owner: operator
- Accepted: 2026-11-23 by the operator, from drawings in the questionnaire. The check box of a linked feed is on and locked. The rule applies from the two sides of a link: the review of a page and the review of a release.
- Changed: 2026-11-23, items 10 and 11 added after observation 2 of [the desktop check walk of 2026-11-23](../reviews/2026-11-23-desktop-check-walk.md). The operator selected the Stop in Review from drawings in the questionnaire. Item 9 gets the field `refusal`.
- Technical choices: the main session selected the condition of item 2 and the removed link in item 3. It also selected the order of item 7, the API field of item 9 and the texts.
- Changes: [ADR 0231](0231-a-new-release-writes-its-artist-page-and-the-files-of-a-restored-release.md) item 7. The rows are not only in Review of a new release.
- Changes: [ADR 0234](0234-a-new-release-is-written-with-its-pages-and-a-one-sided-link-is-reported.md) item 3. The locked check box is not only for the first write of a new release.
- Uses: [ADR 0231](0231-a-new-release-writes-its-artist-page-and-the-files-of-a-restored-release.md) items 8 and 9, one write button and the stop after a failed write. [ADR 0234](0234-a-new-release-is-written-with-its-pages-and-a-one-sided-link-is-reported.md) items 5 to 8, the report of a one-sided link. [ADR 0205](0205-publisher-feeds-announcements-and-show-grants.md) items 6 and 7, a link changes the two drafts. [ADR 0007](0007-draft-diff-write-and-announce.md), the write of a draft.
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

9. The diff route gives `also_write` for each review, with the feeds of item 2. Each entry has `id`, `title`, `side` (`page` or `release`), `artist`, `change` (`added` or `removed`) and `refusal`. `refusal` is the page refusal code of the draft of B (`no_artist_page` or `no_publisher_page`, ADR 0218), or `null`. The UI uses `side` for the order of item 7 and `artist` and `change` for the text of item 5.

### A Linked Release With No Page

10. A row with a `refusal` stops the review before the write, because the write of B fails (ADR 0213 item 18). The usual cause is the removal of the only artist page of a release from that page. Review shows a Stop panel above the changes, one for each such row:

    - The title is "{release} has no other artist page." for `no_artist_page`, and "{release} has no other artist, label or hosting page." for `no_publisher_page`.
    - The line is "A release needs an artist page." or "A release needs a page."
    - The select menu **Artist page** (or **Page**) lists the pages of the collection that the release can link, except A. **Link and Continue** links the selected page to the release and loads the diff again.
    - **Keep the Release** links A to the release again and loads the diff again.
    - With no page in the menu, the panel has only **Keep the Release**.

11. While a Stop panel of item 10 shows, **Write to the File Host** is off, and the line above it is "Link a page to {release} first." Review of a release has no such panel, because a page has no page refusal.

## Item Map

| Earlier item | Change by this ADR |
| --- | --- |
| ADR 0231 item 7 | Items 1 and 2: the rows are in Review of each feed with a linked feed, not only of a new release. |
| ADR 0234 item 3 | Items 4 and 6: the check box is on and locked for each linked feed. The text under the rows changes. |

## Consequences

- One review writes the two sides of a new link and of a removed link, from the page and from the release.
- The draft of a linked feed goes out complete. An edit of a release that the user did not finish goes out with the link. Review shows it under the row.
- The draft of a linked feed can have a link to a third feed. Review writes that link as it is and does not offer the third feed. The report of ADR 0234 item 5 shows a one-sided link that results.
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

Visual, by the main session with screenshots at 1100 px and 390 px:

- Review of an artist page with one linked written release.
- Review of a written release with its artist page.

Visual, by the main session: the Stop panel of item 10 in Review of an artist page at 1100 px and at 390 px.

Human check: the next desktop check adds an existing release on an artist page. It writes the two feeds in one review and reads the two written files.

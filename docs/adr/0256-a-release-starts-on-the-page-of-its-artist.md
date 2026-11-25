# ADR 0256: A Release Starts On The Page Of Its Artist

- Status: Accepted
- Date: 2026-11-24
- Owner: operator
- Accepted: 2026-11-24 by the operator. The operator accepted the consolidation of each changed decision (musicindex-research `df7b823`). This ADR makes no new decision. It restates each live rule of ADR 0213 with each later change applied.
- Accepted in ADR 0213: 2026-11-14 by the operator, through the questionnaire. The operator rejected a block that shows the levels as a tree. The operator selected "Remove the row" for the row "Album" and "Studio keeps it and shows it" for the release kind. For a release with no artist page, the operator selected "A finding on the tab Check" and also a stop in Review.
- Technical choices from ADR 0213: the planning model selected items 3 to 6, 8, 10 to 16, 19 to 22 and 24 to 27.
- Supersedes: [ADR 0213](../archive/adr/0213-a-release-starts-on-the-page-of-its-artist.md), A Release Starts On The Page Of Its Artist.
- Revised: 2026-11-15 by operator decision, from the [walk notes of 2026-11-15](../reviews/2026-11-15-walk-notes.md). Item 13 moves the menu "Kind" from the tab **Settings** to the first tab. Item 16 gives the hint of that menu.
- Uses: [ADR 0014](0014-collection-in-sqlite.md) for the collection. [ADR 0065](0065-the-core-applies-each-feed-rule.md) for the feed rules. [ADR 0174](0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md) for the findings of the tab **Check**. [ADR 0205](../archive/adr/0205-publisher-feeds-announcements-and-show-grants.md) items 5 to 8 for the links of a publisher feed. [ADR 0260](0260-the-welcome-page-and-the-first-steps.md) item 8 for the first-step rows. [ADR 0007](0007-draft-diff-write-and-announce.md) for the review.
- Source: design item 7 of the [human check walk of 2026-11-14](../reviews/2026-11-14-human-check-walk.md).
- Restates: the changes of ADR 0213 by [ADR 0218](0218-a-page-link-in-review-and-a-suggestion-starts-with-no-person.md) items 1 to 9, [ADR 0223](0223-a-split-in-shares-or-percent-a-release-search-and-one-tap-on-a-phone.md) items 11 and 12 and [ADR 0261](0261-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 1 to 13, and the revision of 2026-11-15.
- Changed by: [ADR 0257](0257-a-release-has-one-artist-one-label-and-one-publisher-in-its-fields.md) items 16a, 17 and 18, on 2026-11-24. Item 8: one label and one publisher. Items 9 and 19: a release has the fields Artist, Label and Publisher in place of the section "Linked artist and label pages".
- Changed by: [ADR 0263](0263-overview-is-the-start-page-and-each-page-keeps-the-sequence-of-its-feed.md) item 1, on 2026-11-25. A release starts from **＋ Add** on Overview or on an artist page (item 2).

## Context

The operator found that the flow to make a feed does not show how the feeds are in each other. The levels are publisher feeds: a hosting page, a label page and an artist page. A hosting page and a label page are optional. A release must have an artist page.

Today a user can make a release from the page "Add a Feed" with no link to an artist page. The left panel then shows it in the group "Not linked".

The form has no release kind. The operator wants a select menu with album, EP and single. The Podcasting 2.0 namespace has no tag for the kind, so the feed file cannot hold it.

## Decision

### Where A Release Starts

1. A release starts from **＋ Add** on Releases or on an artist page. On Releases, **＋ Add** opens a menu with Release, Artist, Label, Publisher, Show and Playlist, in that order (ADR 0254 item 44). On an artist page, Release gets that artist as its parent. The artist page also keeps the add row **＋ Add a Release** of ADR 0164, and the first-step row of ADR 0260 item 8.
2. The head of a label page and of a publisher page has the **＋ Add** menu of ADR 0254 item 17, with no Release. The page "Add a Feed" has no option that makes a release.
3. The menu "Type" of the form of ADR 0212 item 6 has Artist or label, Show and Playlist.
4. The page of a label and the page of a publisher feed have no add row for a release. The list view of a group of ADR 0164 item 6 has the action only for an artist.
5. The first-step row "Add a release" of a label page opens the list of the artist pages of the collection. A selection opens the form on the page of that artist. With no artist page, the row opens the form that makes an artist page.
6. The form of a release has no menu "Type" and no "Start from" field. Its title, its parent card, the card with no parent, the row "Find on Music Index" and its fields "Title", "Kind", "Artist name", "File host" and "Feed file" follow ADR 0261 items 8 to 12. **Change…** of the parent card lists only the artist pages (ADR 0231 item 6). "Artist name" starts with the name of the parent. The form follows ADR 0212 items 8 to 12 for the file host and feed file.

### The Pages Of The New Release

7. The new release gets the link to its artist page (ADR 0164 item 3). The form shows no hosting page and no label page.
8. The new release also gets the hosting pages and the label pages of the newest other release of that artist. Each link has the same roles as on that release. An artist with no other release gives only the artist link.
9. The user changes the links of a release in its section "Linked artist and label pages", as before.

### The Release Kind

10. A release has a kind: album, EP or single. Studio keeps the kind in the collection. The feed file does not hold it, and no app gets it.
11. A release with no kept kind, no track and no live item has the kind "release" and the label "Release". For each other release with no kept kind, the count of its tracks gives the kind: 1 to 3 tracks give a single, 4 to 6 give an EP, and each other count gives an album. The core gives this rule.
12. The menu "Kind" has four values: "From the tracks", "Album", "EP" and "Single". The form starts with "From the tracks". That value adds the kind that it gives, for example "From the tracks (Single)". With no track, it is "From the tracks (no tracks yet)" (ADR 0223 item 12).
13. The first tab of a release has the same menu, between "Title" and "Artist". The tab **Settings** has no menu "Kind". A change saves at once. It is not a part of the draft, and it needs no write.
14. The type label of a release shows the kind, in each place that showed "Album": the tile, the row, the head of the feed page and the left panel.
15. A playlist and a publisher feed have no kind. A feed with live shows and no track keeps the label "Album" from item 11.
16. A hint at the menu of the form says "Studio shows this kind. The feed file has no tag for it." A hint at the menu of the first tab says "Saves at once. The feed file has no tag for the kind."

### A Feed With No Page

17. A release must have a link to an artist page. A playlist must have a link to one publisher feed or more. A feed with live shows and no track has the same rule as a playlist. A publisher feed is a hosting page, a label page or an artist page.
18. The tab **Check** gives a finding for a feed that does not obey item 17. For a release, the text is "This release has no artist page." For a playlist or a show feed, the text is "This feed has no artist, label or publisher page." The finding has the line "Studio cannot write it before the link." The level and the button **Link a Page** stay.
19. The finding has the button **Link a Page**. It opens the section "Linked artist and label pages" of the feed. The refusal in Review has the page link itself, with the texts, the menu and **Link and Continue** of ADR 0218 items 1 to 6. The last menu value "New artist page…" makes an artist page, links it and opens Review again (ADR 0251 item 13).
20. The core gives the finding from the draft. It reads no network.
21. Review also stops the write of a feed that does not obey item 17. The diff gives the reason, and the route of the write refuses it. Review shows the refusal text and the page link from item 19. While the refusal shows, the write button is disabled. The line "Link an artist page first." or "Link a page first." shows above the footer.
22. These writes have no stop:
    - A write that deletes the feed file.
    - A write of a feed that is hidden from apps.
    - A write of a change set.
    - A write that a connected app makes under a show grant.
23. A read-only feed gets the finding with no button, as each finding of a read-only feed (ADR 0174).

### What Does Not Change

24. A feed in the collection with no artist page stays as it is. Studio changes no feed because of this ADR.
25. The route that makes a feed still can make an album, because the form of item 6 uses it. The route gets the kind and the publisher feed of the new release.
26. The form has no copy of a release. The copy of ADR 0077, its route field `from` and its core code go (ADR 0261 item 13).
27. The Welcome page and the first-step row of a musician do not change.

## Item Map

Each item in this ADR restates the item with the same number in ADR 0213.

| Item | Source of the change |
| --- | --- |
| 1 | ADR 0230 items 1 to 13 |
| 2 | ADR 0230 items 1 to 13 |
| 5 | ADR 0230 items 1 to 13 |
| 6 | ADR 0230 items 1 to 13 |
| 11 | ADR 0223 items 11 and 12 |
| 18 | ADR 0218 items 1 to 9 |
| 19 | ADR 0218 items 1 to 9 |
| 21 | ADR 0218 items 1 to 9 |

## Alternatives Considered

- A block "Where it goes" that shows a hosting page, a label and an artist as a tree with select menus. Rejected by the operator.
- The row "Album" stays and asks for the artist first. Not chosen by the operator: a release then has two start points.
- The kind comes only from the count of the tracks, with no menu. Not chosen: a single with four versions then shows as an EP.
- Wait for a tag of the namespace for the kind. Not chosen: no date exists for it.
- Only a finding, with no stop in Review. Not chosen by the operator.
- A show feed and a playlist also need an artist page. Not chosen by the operator: a playlist can have the tracks of many artists.

## Consequences

- A release with no kept kind can change its label when the user adds or removes tracks. A release in the collection with 1 to 6 tracks shows "Single" or "EP" after this change.
- The user cannot write a release with no artist page from Review, also for a small correction. The repair is one link.
- A different Studio does not get the kind from the feed.
- A label makes a release through the page of an artist.
- The collection gets one column, and its schema version changes.

## How This Decision Is Checked

Mechanical:

- A Node test proves the rows of "Make a new feed", the values of "Type" and the fields of the release form.
- A Node test proves that only an artist page gives the add action.
- A core test proves the kind from the count of the tracks, and that a kept kind comes first.
- An API test proves that a new release keeps its kind, gets the artist link, and gets the hosting and label links of the newest other release of the artist.
- A core test proves the two findings of item 18, and no finding for a feed that obeys item 17.
- An API test proves that the write route refuses a release with no artist page, and does not refuse each write of item 22.
- A Node test proves the type label for each kind.
- The five commit checks pass.

Visual, by the planning model:

- The page "Add a Feed", the release form from an artist page, the first tab of a release with the menu, a tile of each kind, the finding on the tab **Check**, and the refusal in Review, at 1280 px light and 390 px dark, with the fit checks.

Human, by the operator. This gate is open:

- In the desktop app, make a release from the page of an artist, with the kind EP. The tile shows "EP", and the left panel shows the release below the artist.
- Open a release of the group "Not linked", change its title and open Review. Review refuses the write and offers **Link a Page**.

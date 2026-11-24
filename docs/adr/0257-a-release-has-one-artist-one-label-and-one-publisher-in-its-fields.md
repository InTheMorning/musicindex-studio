# ADR 0257: A Release Has One Artist, One Label And One Publisher In Its Fields

- Status: Accepted
- Date: 2026-11-24
- Owner: operator
- Accepted: 2026-11-24 by the operator, through the questionnaire and from mockups (`.studio-dev/design-brief/prototype/release-fields.html` version 4 and `release-artists.html`, which Git ignores). The operator asked for Artist, Label and Publisher on a release page, as the tags of a music file. The operator selected a select menu for each field and a release alias. The operator stated that a release has one publisher.
- Operator choice, 2026-11-24: after the mockup of the three conditions of two artists, the operator refused more than one linked artist page for a release. Two or more linked artists wait for a protocol of coordination requests between Studios. The operator then selected one label page for each release, for the same cause.
- Technical choices: the main session selected items 10, 11, 12 and 14 and the texts.
- Technical choices, detail: Item 10 is the refusal. Item 11 is the rule of a file with two pages for one role. Item 12 is the sequence of `podcast:publisher`. Item 14 removes "Also on another label…". The operator did not receive them as a question.
- Bar: crosses modules and repositories. It changes the core links, the release page, the Catalog sheets and the feed files that HitIt and Music Index read. Without this record, an agent can keep the chips of two artists or the sheet "Artist and label…".
- Changes: [ADR 0254](0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) items 20, 23, 24, 26 to 33 and the non-goal of a drag of a release row. The fields replace the sheet "Artist and label…", and a release has one label and one publisher.
- Changes: [ADR 0255](0255-publisher-feeds-announcements-and-show-grants.md) items 9 and 14. A release has one link for each of the roles `artist`, `label` and `hosting`, and Studio sets their sequence. A field change also writes `itunes:author` (item 3).
- Changes: [ADR 0253](0253-the-roles-of-a-feed-and-of-each-link.md) item 22. A new role of a feed does not go on a link where it gives a release a second page of that role (item 10a).
- Changes: [ADR 0256](0256-a-release-starts-on-the-page-of-its-artist.md) items 8, 9 and 19. A new release gets one label and one publisher. A release has the fields in place of the section "Linked artist and label pages".
- Changes: [ADR 0218](0218-a-page-link-in-review-and-a-suggestion-starts-with-no-person.md) items 3, 4 and 6. The refusal panel of a release uses the pages of the field Artist, its link sets that field, and a new artist page gets the artist text of the release.
- Changes: [ADR 0146](0146-link-words-already-marks-and-credit-edits.md) item 1. The section "Linked artist and label pages" is for a playlist and a person only.
- Uses: [ADR 0253](0253-the-roles-of-a-feed-and-of-each-link.md) items 1, 15 and 21 for the level roles, the rank and the roles of a link. [ADR 0251](0251-review-writes-the-two-sides-of-a-changed-link.md) for the write of the two sides of a link. [ADR 0254](0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) items 34 to 37 for the change set, the notice and Undo.
- Source: the app research of 2026-11-24, ChatGPT run `20261124T172536Z-10`. The main session examined the cited lines of Partytime and Podverse.

## Context

- A release page shows the links to its pages in the section "Linked artist and label pages" (ADR 0146 item 1). ADR 0254 items 29 to 33 add the sheet "Artist and label…". A musician looks for these values where a music file has its tags: Artist, Label and Publisher.
- The first tab of a release has the text field "Artist". It writes `itunes:author` (`crates/api/ui/editor.js`, `ChannelFields`).
- Each app that the research examined shows one artist line, from `itunes:author`: Podverse, AntennaPod, boostmebitch, MSP 2.0, Stophammer and HitIt.
- The Podcasting 2.0 tag `podcast:publisher` has the count "Single". It must contain exactly one `podcast:remoteItem`. Partytime (`src/parser/phase/phase-7.ts` line 78), and thus Podverse, reads only the first item. MSP 2.0 reads only the first item. musicindex ADR 0033 and ADR 0097 R-ROLE-7 name each artist page and each label page of a release in `podcast:publisher`. Each link has a `rel`.
- A link changes the drafts of the two feeds (`publisher::link` in `crates/core/src/publisher.rs`). Thus a Studio links a release only to a page of its own collection. A second artist with a page in a different Studio cannot get the release in the catalog of that page.

## Decision

### The Fields Of A Release

1. The first tab of a release has the heading "Credits" and three select menus, in this sequence: **Artist**, **Label** and **Publisher**. They replace the text field "Artist" in the same place, below "Kind" (ADR 0256 item 13). A playlist and a show feed keep their fields.
2. **Artist** lists the writable artist pages of the collection, then "New Artist Page…". A release needs an artist page (ADR 0256 item 17), so the menu has no empty value. A release with no artist page shows the placeholder "Select an artist page", and the finding of ADR 0256 item 18 stays. The hint is "One artist page. Put a second artist in the release alias."
3. Below **Artist**, the closed section "Release alias" has one text field.
   - With an empty alias, Studio writes the title of the artist page to `itunes:author`.
   - With an alias, Studio writes the alias to `itunes:author`. The release stays in the catalog of its artist page.
   - The hint is "The artist name of this release in podcast apps and music files: "{text}". The release stays in the catalog of the artist page."
   - When the `itunes:author` of the draft is not the title of the artist page, the section opens with that text as the alias.
4. **Label** lists the writable label pages, then "Independent (no label)", then "New Label Page…".
5. **Publisher** lists the writable publisher pages, then "Not published", then "New Publisher Page…". The value that follows is the publisher page directly above the label in the tree (ADR 0253 item 15). With no label, it is the publisher of the newest other independent release of the artist, or else the publisher page directly above the artist page. When two or more publisher pages qualify, the first from A to Z is the value. With no such publisher, it is "Not published".
   - When the value is the value that follows, the hint is "Follows the label." or "Follows the artist page.", then "A release has one publisher. A music file calls the label "Publisher". Here the label goes in Label."
   - When the value is a different one, the hint is "You selected the publisher." with the button **Follow the Label Again** or **Follow the Artist Page Again**. The button sets the value that follows.
   - A change of **Artist** or **Label** also sets the value that follows. This occurs only when **Publisher** had the value that followed before the change.
6. "New Artist Page…", "New Label Page…" and "New Publisher Page…" open the form of a publisher page with that role. After the make, the field selects the new page.
7. One page can be the value of two fields, for example a feed with the roles `artist hosting` in **Artist** and **Publisher**. Then the release has one link to the page, and its `rel` has the role of each field that names the page (musicindex ADR 0097 R-ROLE-7 item 4).
8. A change of a field removes the link of that role and adds the link to the selected page. Each change goes into the change set of the Catalog (ADR 0254 item 34). The notice of ADR 0254 item 35 says "{page} is now the {artist, label or publisher} of {title}.", "{title} is independent." or "{title} is not published.", then "Not written yet.". Review writes the two sides of each link (ADR 0251).

### One Page For Each Role

9. A release has at most one link with the role `artist`, one with the role `label` and one with the role `hosting`. A link with the role `curator` does not change.
10. The core refuses a link that gives a release a second page of the role `artist`, `label` or `hosting`. The error code is `role_taken`. The text is "{release} already has the {artist, label or publisher} page {page}. Change it in the fields of the release." Each route that links a page to a release gets this refusal. Examples are the add of a release on a page, the refusal panel of ADR 0218 and the sheets of ADR 0254. A field change of item 8 and a change of ADR 0254 item 26 remove the old link first. Thus they do not get the refusal.
10a. A change of the roles of a feed (ADR 0253 item 22) does not add a role to a link when the release of that link has a different page with that role. The link keeps its other roles. The page of the feed says "{N} releases keep their {role} page." with the names in a fold.
11. A file from a different app or from an older Studio can have two or more pages for one role. The field shows the first page in the sequence of `podcast:publisher`, and the hint "Also linked in the file: {names}." with the button **Remove the Others**. The button removes the other links of that role, as one change of item 8. Studio does not stop the write of such a file.
12. Studio writes the items of `podcast:publisher` in this sequence: the publisher, the label, then the artist page. This is the rank of ADR 0253 item 15. An app that reads only the first item gets the publisher. The user does not change the sequence. This changes ADR 0255 item 9.

### Two Or More Artists Wait

13. A release has one linked artist page and one linked label page until a protocol lets one Studio ask a different Studio to list a release. That protocol is a separate decision. Until then, a second artist goes in the release alias, for example "Elm Street Duo feat. Mara Quinn".

### The Changes Of ADR 0254

14. The action "Also on another label…" is removed from items 23, 24, 26 and 27. An artist with releases on two labels uses "Change label…" with only some releases selected (item 28). In item 20, the action of the hint is "Change label…" or "Change publisher…".
15. In item 26, "Sign to a label…" and "Change label…" add a link to the selected label and to the one publisher that follows it (item 5), not to each publisher above it.
16. Items 29 to 33 are replaced. The menu **⋯** of a release row has no "Artist and label…". The sheet "Artist and label of {title}" is removed. The fields of items 1 to 8 change the artist, the label and the publisher of a release. The non-goal "The sheet "Artist and label…" changes a release" becomes "The fields of a release change it (ADR 0257)".

### The Changes Of ADR 0256, ADR 0218 And ADR 0146

16a. ADR 0256 item 8: the new release gets the label link and the publisher link of the newest other release of that artist. Each one is the first of its role in the sequence of `podcast:publisher` of that release.
17. ADR 0256 item 9: the user changes the links of a release in its fields **Artist**, **Label** and **Publisher**.
18. ADR 0256 item 19: for a release, **Link a Page** opens the first tab and puts the focus on **Artist**. For a playlist and a show feed, it opens the section "Linked artist and label pages", as before.
19. ADR 0218 item 3: for a release, the menu of the refusal panel has the pages of **Artist** (item 2). ADR 0218 item 4: for a release, **Link and Continue** sets the value of **Artist**. ADR 0218 item 6: the new artist page gets the `itunes:author` text of the release as its name. The refusal panel of a playlist and of a show feed does not change.
20. ADR 0146 item 1: the control "Link to an artist or label page" and the heading "Linked artist and label pages" apply to a playlist, a show feed and a person. A release has the fields of items 1 to 8.

## Item Map

| Earlier item | Change by this ADR |
| --- | --- |
| ADR 0254 items 20, 23, 24, 27 | Item 14: no "Also on another label…". |
| ADR 0254 item 26 | Items 14 and 15: no "Also on another label…", and one publisher. |
| ADR 0254 items 29 to 33 | Items 1 to 8 and 16: the fields replace the sheet. |
| ADR 0254 item 31 | Item 5: one publisher follows the label or the artist page. |
| ADR 0255 item 9 | Items 9 and 12: one page for each role, and a fixed sequence. |
| ADR 0255 item 14 | Item 3: a field change also writes `itunes:author`. |
| ADR 0253 item 22 | Item 10a. |
| ADR 0256 item 8 | Item 16a. |
| ADR 0256 items 9 and 19 | Items 17 and 18. |
| ADR 0218 items 3, 4 and 6 | Item 19. |
| ADR 0146 item 1 | Item 20. |

## Alternatives Considered

- Chips for two or more artist pages and labels (mockup version 3). Not chosen by the operator: a Studio can write only the pages of its own collection. The chips give a second catalog only when one Studio holds each artist page.
- One `podcast:publisher` item for each release, as the specification says. Not chosen: musicindex ADR 0033 and ADR 0097 name each artist page and each label page in `podcast:publisher`. The conflict goes to musicindex.
- The artist page first in `podcast:publisher`. Not chosen: an app that reads only the first item then shows the artist as the publisher.
- Keep the sheet "Artist and label…" next to the fields. Not chosen: two places for one change.

## Consequences

- A musician sets the artist, the label and the publisher where a music file has its tags.
- A release with two artists shows them in the alias. The second artist does not get the release in the catalog of a page.
- A file with two pages for one role still loads, and the user can remove the others.
- An artist with releases on two labels needs one change for each label.

## Non-goals

- The protocol between two Studios for a request to list a release.
- A new name of an artist page in the `itunes:author` of its releases. A release with no alias keeps its written text until a change of its fields.
- The section of a playlist and of a person.

## Open Points

- Closed on 2026-11-24 by the orchestrator for musicindex. R-ROLE-7 of musicindex ADR 0097 keeps one remoteItem with `rel` for each linked page (Podcast Namespace PR #793, musicindex ADR 0067 item 87). R-ROLE-7 adds the sequence of item 12 as a rule for a writer. The website session gives the commit. The rule of one page for each role (item 10) is a Studio rule only, because a reader can read a feed from a different tool with two labels.
- Known limit: MSP 2.0 writes the `podcast:publisher` block again when a user saves a feed, and it keeps only one link. Stophammer reads the publisher title from the first remoteItem with `medium="publisher"`. Thus the sequence of item 12 also gives the hosting page to Stophammer.
- The rank of curator (musicindex ADR 0097 open point 1) does not change item 12, because a release has no curator link in `podcast:publisher` now.

## How This Decision Is Checked

Mechanical (the code tasks of this ADR add these tests):

- A core test: a second link of the role `artist`, `label` or `hosting` to a release gives `role_taken` (item 10). A link of the role `curator` does not.
- A core test: a field change replaces the link of its role in one change, and Undo puts back the old link (item 8).
- A core test: one page in **Artist** and **Publisher** gives one link with `rel="artist hosting"` (item 7).
- A core test: the written `podcast:publisher` has the sequence publisher, label, artist (item 12).
- A core test: an empty alias writes the title of the artist page to `itunes:author`, and an alias writes the alias (item 3).
- A Node test of the view model of the fields: the values of each menu, the value that follows of item 5, the hints, and the hint of item 11 for a file with two artist pages.
- A Node test: the menu of a release row has no "Artist and label…", and the actions of a page have no "Also on another label…" (items 14 and 16).

Visual (the main session with screenshots, then the operator):

- The first tab of a release at the desktop width and at 390 px, with the alias open.
- A change of **Label** with **Publisher** that follows, then Undo.

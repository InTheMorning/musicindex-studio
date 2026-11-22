# ADR 0240: Hosting Is A First Role, And A Stray Release Moves To Its Pages

- Status: Accepted
- Date: 2026-11-21
- Owner: operator
- Accepted: 2026-11-21 by the operator. In the chat, the operator gave three rules. A hosting page makes labels and artists, not releases. A release cannot start without an artist. A stray release needs a path into an artist page, a label page or a hosting page. The operator asked for Hosting as the third role in the menus, with the long form "I'm hosting for other artists". The operator then selected the panel of items 5 to 8 and the sheet of items 12 to 19 from drawings in the questionnaire.
- Revised: 2026-11-21. Items 20 to 25 add the move of an artist page and a label page. The operator asked in the chat that releases, artists and labels can move, not only a stray release. The operator then selected from drawings in the questionnaire: a move links the releases of the page, and Move To… is in the page head and in the menu of a panel row.
- Technical choices: the main session selected the texts of items 2 and 3 and the menu of a page head in item 4. It also selected the start values of item 15 and the route of item 18. The operator did not receive them as a question.
- Bar: crosses modules. It changes the add menu, the Welcome page and the left panel. It also changes the Not linked list, a release page and a route. Without this record, an agent can keep **＋ Add release** on a hosting page.
- Changes: [ADR 0230](0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 1 and 2 (items 1 and 4).
- Changes: [ADR 0221](0221-the-welcome-page-and-the-first-steps.md) items 2, 4 and 8 (items 2 and 3).
- Changes: [ADR 0232](0232-each-group-below-an-open-feed-has-an-add-row.md) items 2, 3 and 4 (items 5 to 8).
- Changes: [ADR 0239](0239-the-panel-holds-the-content-and-the-page-has-tabs.md) item 4 (item 7).
- Uses: [ADR 0082](0082-a-publisher-feed-has-a-set-of-roles.md) and [ADR 0083](0083-hosting-is-a-first-class-role.md) for the roles. [ADR 0205](0205-publisher-feeds-announcements-and-show-grants.md) for the two-way link of a page and a release. [ADR 0215](0215-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) item 2 for the rule "below". [ADR 0238](0238-a-found-release-gives-the-page-of-each-role.md) item 4 for the meaning of the hosting role.
- Changed by: [ADR 0243](0243-a-move-is-a-change-set-and-a-level-gathers-its-pages.md) items 1 to 6, on 2026-11-22. Items 18 and 19: a move refuses with the names of the feeds, and it makes a change set.

## Context

On 2026-11-21, the operator made a hosting page "Wrathful Test". Its left panel showed only the group Releases and **＋ Add release**. The hosting role is a person who hosts the feeds of other artists and labels (ADR 0238 item 4), so the panel gave an incorrect action.

The level of a hosting feed (`feedLevel` in `crates/api/ui/shell.js`) has the groups Labels and Artists with their add rows. The page of the same feed uses the outline (`outlineItems`), and the outline of each publisher feed has only Releases and **＋ Add release** (ADR 0239 item 4).

The add menu has Artist and Label, but no Hosting. A user makes a hosting page only with the menu "Role" of the Artist or Label form.

The release form cannot make a release without an artist page (ADR 0230 items 9 and 10). An import can make a release with no link. Such a release shows in "Not linked". To link it, the user opens the release and selects one page at a time in the section "Linked artist and label pages". The section does not say that a release needs an artist.

## Decision

### Hosting In The Menus

1. The menu **＋ Add** of the page Releases has these values, in this sequence: Release, Artist, Label, Hosting, Show, Playlist. Hosting opens the form of a publisher page with the role Hosting as its start value.
2. The Welcome page has a third row in its first group, after "I run a label": "I'm hosting for other artists", with the line "Start with your hosting page. The artists and labels that you host go on it."
3. "I'm hosting for other artists" opens the sheet of the first steps with the steps "Your name", "Where it lives" and "Your first artist". The core makes the publisher feed with the role `hosting`. The page of the new feed has the rows "Add an artist", "Add a label" and "Find an artist that is online".
4. The menu **＋ Add** of a page head has no Release on a label page or a hosting page. On a hosting page, Artist and Label make a page below that hosting page. On a label page, Artist makes a page below that label page. A menu value of a role with the same rank or a higher rank is not in the menu of a page head.

### The Groups Of A Page

5. A level shows these groups, also when a group has no rows:
   - A hosting feed: Labels with **＋ Add label**, and Artists with **＋ Add artist**. It has no group Releases.
   - A label feed: Artists with **＋ Add artist**, then Releases with no add row.
   - An artist feed: Releases with **＋ Add release**.
   - A group of a different role heading shows only when it has rows, and it has no add row.
6. **＋ Add release** shows only on an artist feed. A release starts only on the page of its artist.
7. The outline of a publisher feed holds its head and the groups of item 5 for its role. The outline of a playlist holds only its head.
8. At 760 px or less, the level page and the outline page show the same groups and add rows.

### A Stray Release Moves To Its Pages

9. A stray release is a release in "Not linked": it has no link to an artist page, a label page or a hosting page of the collection.
10. Each row of "Not linked" has the button **Move To…**.
11. The section "Linked artist and label pages" of a release has the button **Move To…**. With no linked artist page, the section says "A release needs an artist page."
12. **Move To…** opens the sheet "Move {title}". It has three select menus: Artist, Label and Hosting.
13. Artist is necessary. Its values are the writable artist pages of the collection. Label and Hosting are optional, and their first value is "None". Their values are the writable label pages and hosting pages.
14. The last value of each menu is "New {Artist, Label or Hosting} Page…". It opens the form of a publisher page with that role. After the make, the sheet selects the new page.
15. When the selected artist page is below a label page or a hosting page (ADR 0215 item 2), the Label menu and the Hosting menu start with that page. On a release page, each menu starts with the linked page of that role.
16. **Move** links the release to the selected artist page, and to the selected label page and hosting page. Each link changes the drafts of the two feeds, as ADR 0205 gives for a link. Then the artist page is below the label page and the hosting page, because they share the release (ADR 0215 item 2).
17. On a release page, **Move** also removes the link to each other page of the collection with the role artist, label or hosting. A link to a page that is not in the collection stays.
18. One route does each link and each removal of item 16 and item 17. When one link fails, the route changes no draft.
19. After **Move**, the release shows on the level of its artist page, and the sheet closes. The writes go through Review (ADR 0007).

### An Artist Page Or A Label Page Moves

20. An artist page and a label page also have **Move To…**. It is in the head of the page, and in the menu **⋯** of its row in a level and in an outline. Each release row of a level and of an outline also has the menu **⋯** with **Move To…**. The button **⋯** has the accessible name "More for {name}". A hosting page has no **Move To…**, because no role has a higher rank.
21. The sheet "Move {name}" of an artist page has the menus Label and Hosting. The sheet of a label page has only the menu Hosting. The menus follow items 13 and 14. Each menu starts with the page of that role that the moved page is below now (ADR 0215 item 2), or with "None".
22. The releases of a page are the releases of the collection that link to that page. The sheet shows "Changes {N} releases:" and one line for each page that the releases get or lose, for example "+ Night Owl − Old Label".
23. **Move** applies item 16 and item 17 to each release of the page, only for the roles of the sheet. The link of each release to the moved page stays. On a label page, the links of a release to its artist pages also stay.
24. **Move** also removes each kept link (ADR 0232 item 9) from the moved page to a page of the roles of the sheet. It adds a kept link to each selected page. Thus a page with no releases also moves.
25. Item 18 and item 19 apply to the move of a page. After **Move**, the page shows on the level of its new upper page.

## Item Map

| Earlier item | Change by this ADR |
| --- | --- |
| ADR 0230 item 1 | Item 1: the menu gets Hosting after Label. |
| ADR 0230 item 2 | Item 4: the menu of a label page or a hosting page has no Release, and its new pages go below the page. |
| ADR 0221 item 2 | Item 2: the third row of the first group. |
| ADR 0221 items 4 and 8 | Item 3: the steps and the first rows of a hosting page. |
| ADR 0232 item 2 | Item 5: a hosting feed has no group Releases, and the Releases of a label feed have no add row. |
| ADR 0232 items 3 and 4 | Items 6 and 7: the outline shows the groups of the level, and only an artist feed adds a release. |
| ADR 0239 item 4 | Item 7: the outline of a publisher feed shows the groups of its role. |

## Alternatives Considered

- The hosting feed lists the releases of each of its artists, with no add row. Not chosen: the panel gets long, and the artists already hold the releases.
- **Move To…** asks only for the artist page, and the label and the hosting come from the place of that artist. Not chosen: a stray release can belong to a label page or a hosting page that the artist page is not below yet.
- The move of a page changes only the kept link, and the release links stay. Not chosen: Music Index and HitIt read only the feed files, so they do not see the new place.
- Check boxes in "Not linked", and one **Move To…** for many releases. Not chosen now: one sheet for each release is the first step.

## Consequences

- A hosting page shows only the pages that it hosts. Its releases are on the levels of its artists.
- The move of an artist page or a label page changes the feed file of each of its releases and of each page that a release gets or loses. Review shows each of these drafts.
- A release on a label page with no artist is still possible from an import. It shows in the group Releases of the label, and **Move To…** gives it an artist.

## How This Decision Is Checked

Mechanical:

- `crates/api/ui-tests/addfeeds.test.mjs`: `addMenuEntries` gives the values of item 1, and the values of item 4 for a label page and a hosting page.
- `crates/api/ui-tests/welcome.test.mjs`: the Welcome model gives the row of item 2 and the steps of item 3.
- `crates/api/ui-tests/shell.test.mjs`: the level and the outline of a hosting, a label and an artist feed give the groups and the add rows of items 5 to 7.
- A Node test of the model of the sheet: the menus, the values "None" and "New … Page…", and the start values of item 15.
- A Node test of the model of the sheet of a page: the menus of item 21 and the change lines of item 22.
- Rust tests in `crates/api/src/tests.rs`: the move of an artist page links each of its releases to the new label page and unlinks the old label page, keeps the artist links, and moves the kept link of item 24.
- Rust tests in `crates/api/src/tests.rs`: the route of item 18 links the three pages, removes the other links of item 17, keeps a link to a page that is not in the collection, and changes no draft when one link fails.

Visual (a person, or the main session with screenshots):

- The panel of a hosting page and of a label page, in the dark theme, at 1280 px and at 390 px.
- The Welcome page with three rows.
- The sheet "Move {title}" from "Not linked" and from a release page.
- The sheet "Move {name}" of an artist page from its head and from the menu **⋯** of its panel row.

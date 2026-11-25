# ADR 0254: The Catalog Is A Tree, And Each Change Uses Music Words

- Status: Accepted
- Date: 2026-11-24
- Owner: operator
- Accepted: 2026-11-24 by the operator, after the clickable prototype of the structure (`.studio-dev/design-brief/prototype/structure.html`, which Git ignores). The operator accepted the music words of each action. In the questionnaire, the operator selected the panel name "Catalog" and drag and drop on the desktop only. The operator also selected one row for a feed that publishes its own music, and the Add sheet with three groups.
- Technical choices: the main session selected the texts of the sheets and of the notice. It also selected the start selection of item 28 and the third menu of the release sheet in item 30. The operator did not receive them as a question.
- Bar: crosses modules. It changes the left panel, the page of a publisher feed, the sheets, the core routes of a move and the change set. Without this record, an agent can keep the levels of ADR 0215 or the sheet **Move To…** of ADR 0240.
- Operator choice, 2026-11-24: one record replaces ADR 0085, ADR 0181, ADR 0215, ADR 0232 and ADR 0240. The tree changes most of their rules. ADR 0085 and ADR 0232 had three "Changed by" lines. ADR 0181 gave the rules of a level only.
- Operator choice, 2026-11-25: in Review, a page that is only a publisher shows its name with no "Now" and no "After" (item 36). It has no page above it, so it has no place.
- Supersedes: [ADR 0085](../archive/adr/0085-each-artist-and-label-is-a-destination.md), Each Artist And Label Is A Destination.
- Supersedes: [ADR 0181](../archive/adr/0181-a-level-shows-the-state-of-its-own-feed.md), A Level Shows The State Of Its Own Feed.
- Supersedes: [ADR 0215](../archive/adr/0215-the-left-panel-shows-the-levels-of-the-publisher-feeds.md), The Left Panel Shows The Levels Of The Publisher Feeds.
- Supersedes: [ADR 0232](../archive/adr/0232-each-group-below-an-open-feed-has-an-add-row.md), Each Group Below An Open Feed Has An Add Row.
- Supersedes: [ADR 0240](../archive/adr/0240-hosting-is-a-first-role-and-a-stray-release-moves-to-its-pages.md), Hosting Is A First Role, And A Stray Release Moves To Its Pages.
- Changed by: [ADR 0257](0257-a-release-has-one-artist-one-label-and-one-publisher-in-its-fields.md) items 1 to 16, on 2026-11-24. Items 20, 23, 24, 26 to 33: the fields of a release replace the sheet "Artist and label…", a release has one label and one publisher, and "Also on another label…" is removed.
- Changes: [ADR 0243](0243-a-move-is-a-change-set-and-a-level-gathers-its-pages.md) items 1, 3, 5 and 7 to 12. The changes of the tree go into one change set (item 34). Review does not open after each change (item 35). The Add sheet replaces the menus of the add rows (items 18 to 21).
- Changes: [ADR 0206](0206-the-shell-and-the-editor-pages.md) items 6 to 8. A publisher feed has no outline. The first row of an outline is "‹ Catalog" (item 7).
- Changes: [ADR 0090](0090-studio-names-the-feeds-that-it-does-not-edit.md) item 2. "No artist yet" replaces "Not linked", and the heading "No role" is a heading of the top rows (items 2 and 3).
- Changes: [ADR 0165](0165-the-left-panel-marks-the-state-of-each-feed.md) item 3. The marks of a tree row are in item 6a.
- Changes: [ADR 0221](0221-the-welcome-page-and-the-first-steps.md) item 8. The page of a new label has no row "Find an artist that is online" (item 46).
- Changes: [ADR 0239](0239-the-panel-holds-the-content-and-the-page-has-tabs.md) item 4. The panel shows the tree for a publisher feed. Only a release and a playlist have an outline (item 7).
- Uses: [ADR 0253](0253-the-roles-of-a-feed-and-of-each-link.md) items 7, 11, 15, 16 and 20 to 23 for the roles, the levels and the word "Publisher". [ADR 0207](0207-recipients-change-sets-and-wallets.md) items 24 to 33 for a change set. [ADR 0227](0227-undo-and-redo.md) for Undo.
- Uses: [ADR 0233](0233-each-add-form-finds-on-music-index-and-copies-the-found-feed.md) items 2 to 14 for a search on Music Index. [ADR 0230](0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 1 to 5 for the menu **＋ Add** and the forms.
- Uses: [ADR 0089](0089-the-left-panel-shows-the-art-of-each-publisher-feed.md) for the art of a row, and [ADR 0165](0165-the-left-panel-marks-the-state-of-each-feed.md) items 1, 2 and 4 for the marks. [ADR 0090](0090-studio-names-the-feeds-that-it-does-not-edit.md) item 2 for "Other feeds". [ADR 0065](0065-the-core-applies-each-feed-rule.md) for the rules that the core applies.
- Uses: Stophammer ADR 0069 §"A direct link between a label feed and an artist feed" (rejected).
- Source: observations 6 to 9, 11 and 14 to 19 of the [desktop walk of 2026-11-23](../reviews/2026-11-23-desktop-check-walk.md). The design brief and its two reviews are in `.studio-dev/design-brief/`. The roles are in the [note of 2026-11-24](../notes/2026-11-24-roles-levels-and-words.md).

## Context

- ADR 0215 showed the publisher feeds as levels. A selection replaced the panel with the next level, so the user saw one level at a time. In the walk of 2026-11-23, the user could not see where an artist was and where it could go (observation 18).
- ADR 0240 gave one sheet **Move To…** with three menus: Artist, Label and Hosting. The words "move" and "place" did not tell the user what the change did to the music (observations 6 to 9).
- A move did not show before its write. The user did not see what changed (topic E, observation 19).
- ADR 0232 and ADR 0243 gave an add row with a menu of two values: a new page or "Move … Here…". The user had to know which value to select before seeing the pages.
- ADR 0253 gives each link its own roles and four level roles. The screen word of the role `hosting` is "Publisher".
- ADR 0085 gave one flat list of publisher feeds. ADR 0181 gave the state of the feed of a level.
- ADR 0085, ADR 0181, ADR 0215, ADR 0232 and ADR 0240 are in `docs/archive/adr/` for research, with their context, alternatives and evidence.

## Decision

### The Catalog Panel

1. The left panel has the name "Catalog". The name is its heading and its accessible name. The panel shows the structure of the music: the publishers, the labels, the curators and the artists. A release shows on the page of each feed that it links to (item 13).
2. The panel shows, in this sequence: the destination Releases, the tree of item 3, the row "No artist yet" and the row "Other feeds". Releases lists each album and each playlist of the collection. A publisher feed opens from its row in the tree, not from Releases. "No artist yet" shows only when a release has no link to an artist page of the collection. It has the sorts of Releases. "Other feeds" shows when a feed that is not a release or a publisher feed exists (ADR 0090 item 2).
3. The tree holds the publisher feeds. Its top rows are the feeds with no feed above them (ADR 0253 item 15, and item 40 of this ADR). The top rows are in the role headings of ADR 0253 item 16, A to Z in each heading. A publisher feed with no role is in the heading "No role" (ADR 0090 item 2).
4. The core gives the feeds directly below and directly above each feed in the feed list, as `child_ids` and `parent_ids`. Feed B is directly below feed A when B is below A, and no feed C is below A with B below C. A feed can be directly below more than one feed. The UI shows these lists and has no copy of the rule (ADR 0065).
4a. The core gives the links of each feed of the collection, as `release_ids` and `publisher_ids`. A link counts when one of the two feeds names the other. A publisher feed gives first the releases of its channel, in that sequence, then the releases that only name it. A release gives first the feeds of its `podcast:publisher`, in that sequence, then the feeds that only list it.
5. A row with feeds directly below it has a button before its title. The button has the accessible name "Open {name}" or "Close {name}", and it gives `aria-expanded`. Below an open row, the tree shows the feeds directly below it, in the rank of ADR 0253 item 15, then A to Z. Each depth adds an indent of 16 px. A feed below two feeds shows below each one.
6. A row shows the art and the dot of the feed (ADR 0089 items 1 to 4), its title, and the number of its releases. A row of a feed with the role `hosting` shows the number of the pages below it. A row of a feed with a different level role shows the number of its releases. A feed with the two kinds of roles shows the two numbers. The row ends with the button **⋯**, with the accessible name "More for {name}".
6a. A row has the marks of ADR 0165 items 1, 2 and 4. A tree row covers its feed and each release that it shows, also when the row is open. It does not cover the rows below it, because these rows have their own marks. "No artist yet" and "Other feeds" cover the feeds that they hold. This changes ADR 0165 item 3.
6b. A feed with two or more level roles is one row, at the rank of its first level role (ADR 0253 item 15). For example, a feed with the roles `artist hosting` is at the artist rank. Its page shows the groups of item 13 for each of its level roles, with their add rows, from the highest rank. Its menu **⋯** has the actions of item 23 for its rank.
7. A selection of a row opens the page of the feed in the main pane, and the row shows as current (`aria-current`). The tree keeps its open rows. A release or a playlist that opens shows its outline in the panel (ADR 0239 items 1 and 4). The first row of the outline is "‹ Catalog". It shows the tree again, with the same open rows. A publisher feed has no outline.
8. A row of a feed that is directly below two or more feeds has a second line in the tree. Below a label, the line names the other labels: "Also on {names}". When no other label exists, the line names the other publishers: "Also published by {names}".
9. The note "Also on" of a release on a page names only the feeds that are not above or below the feed of the page. An example is "Also on Some Label".
10. At 760 px or less, the panel is a page of the stack. A selection of a row opens the page of the feed, and its back row "‹ Catalog" shows the tree again. The phone has no drag and drop. The menu **⋯** gives each change.

### The Page Of A Publisher Feed

11. The head of the page shows the art, the title, the type label (ADR 0253 item 7) and the place of the feed in music words. Examples are "On Northwind Records, published by Riverside", "Independent, published by Riverside" and "Not published". The head has the button **⋯** of item 23 and the button "Open the feed".
11a. The head also shows the state of the feed of the page only, with the text and the marks of a release tile. Examples are "No changes" and "Saved, not written". A release with a draft does not change this state.
12. Above the head, a path names each feed above the feed of the page on one line, with " › " between two names. Each name opens its page. A feed with no feed above it shows "Top level".
12a. When the feed is directly below two or more feeds, the path follows the tree row that the user selected. When the page opens from a different place, the path follows the feed directly above it with the highest rank, then A to Z. The page of the path is "the page above" in items 28 and 39.
13. The page shows these groups, also when a group has no rows:
    - A feed at the publisher rank: "Labels and artists published here", with the add rows **＋ Publish a label** and **＋ Publish an artist**. It has no group Releases.
    - A feed at the label rank: "Artists on this label", with the add row **＋ Sign an artist**. Then the group Releases, with no add row.
    - A feed at the curator rank: the group Releases, with no add row.
    - A feed at the artist rank: the group Releases, with the add row **＋ Add release**.
    - A group of a different role heading shows only when it has rows, and it has no add row.
13a. The group Releases has the sorts "Feed order", the default, then "Newest" and "A to Z". "Feed order" is the sequence of item 4a. The page keeps its sort in the navigation record of the session. The address keeps only the sort and the view of Releases.
14. An empty group says "Nothing published here yet.", "No artists on this label yet." or "No releases on this page."
15. **＋ Add release** opens the form of a release with that artist as the parent (ADR 0230 item 3). It shows only on an artist page, so a release starts only on the page of its artist. The add row **＋ Add a Release** in the body of an artist page stays (ADR 0230 item 5).
16. Each row of a group shows the dot, the name, the type label and the number of releases or pages. It has the button **⋯** of item 23. Each release row has the button **⋯** of item 29.
17. The menu **＋ Add** of a page head has no Release on a label page or on a publisher page. On a publisher page, Artist and Label make a page below that publisher page. On a label page, Artist makes a page below that label page. A menu value of a role with the same rank or a higher rank is not in the menu of a page head.

### The Add Sheet

18. The add rows **＋ Sign an artist**, **＋ Publish an artist** and **＋ Publish a label** open the Add sheet. Its title is "Sign an artist to {label}", "Publish an artist on {publisher}" or "Publish a label on {publisher}". Its first field is "Search", with the placeholder "Name".
19. Below the field, the sheet has three groups, in this sequence:
    - "In your collection": a row with a check box for each writable page of the collection with the role of the sheet or with no role. A page that is directly below the page of the sheet now has no row. A row shows the name, the type label or "No role", the number of releases, and the place in music words. No row starts selected. The search field filters the rows.
    - "On Music Index": the search of ADR 0233 items 2 and 4 for the kind of the role. A selection adds the feed as ADR 0233 items 6 to 14 give, and the new page goes below the page of the sheet (item 40).
    - "New": the row **＋ Make a new {artist or label} page**, with the typed name in quotes when the field has text. It opens the form of a publisher page with the role of the sheet and the typed name (ADR 0230). The new page goes below the page of the sheet (item 40).
20. When a selected row has no role, the sheet says "A page with no role gets the role Artist." or "A page with no role gets the role Label." When a row is selected, the sheet says: "All releases of the selected {artists or labels} then come out on {label}." On a publisher page, it says "… are then published by {publisher}." It adds: "For only some releases, use {action} in the ⋯ menu of the {artist or label}." The action is "Also on another label…" or "Change publisher…".
21. The button is **Sign** on a label page and **Publish** on a publisher page. With no selected row, the button is not available. The button gives each selected page with no role the role of the sheet (ADR 0253 items 5 and 10). Then it applies item 26 to each selected page, with all its releases and the page of the sheet as the selected page.

### The Actions Of A Page Use Music Words

22. A change of a page asks one question. The words tell what happens to the music: an artist is on a label or is independent, and a page is published by a publisher.
23. The menu **⋯** of a page has these actions, then "Open":
    - An artist on a label: "Change label…", "Also on another label…" and "Go independent…".
    - An artist with no label: "Sign to a label…", then "Change publisher…", or "Choose a publisher…" when no publisher is above it.
    - A label: "Change publisher…", or "Choose a publisher…" when no publisher is above it.
    - A publisher and a curator: no action. The rank of a publisher is the highest.
24. Each action opens a sheet with one question:
    - "Sign to a label…" and "Change label…": "Which label releases {name}?", with the button **Change Label**.
    - "Also on another label…": "Which other label releases {name}?", with the button **Add Label**.
    - "Go independent…": "Make {name} independent", with no list of pages, and the button **Go Independent**.
    - "Change publisher…" and "Choose a publisher…": "Who publishes {name}?", with the button **Change Publisher**.
25. A sheet of item 24 shows these parts. The sheet "Go independent…" has no list of pages and no hint to select a page.
    - "Now", with the place of the page in music words.
    - A search field and a list of the pages of the role of the question. One row can be selected. The list of a publisher question starts with the row "Not published". A row shows the dot, the name, and the place of a label or the number of pages of a publisher. The page above the page now has the mark "now". The sheet "Also on another label…" does not list the labels that the page is on now.
    - The releases of the page, each with a check box. The list title is "Releases on the new label", "Releases that come out on the other label", "Releases with no label after the change" or "Releases that change publisher".
    - The result: "After: {name} is {place}." and "Changes {N} releases." Each release that is not selected is named: "{titles} keeps its label." or "{titles} keep their publisher."
    - The hint "Select a label." or "Select a publisher." until a row is selected. The hint "Nothing changes. Select a different one or a release." when the change gives the same links. Then the button is not available.
26. The change applies to each selected release of the page. Each action has its own links to remove and to add:
    - "Sign to a label…" and "Change label…" remove the links of the release to each label page and each publisher page. They add a link to the selected label and to each publisher above it.
    - "Also on another label…" removes no link. It adds a link to the selected label and to each publisher above it.
    - "Go independent…" removes the links of the release to each label page. It keeps the links to the publisher pages and adds no link.
    - "Change publisher…" and "Choose a publisher…" remove the links of the release to each publisher page. They add a link to the selected publisher. "Not published" adds no link. The links to the label pages stay.
    - A link to a curator page stays in each action.
    - The link of the release to the page stays. On a label page, the links of a release to its artist pages also stay.
    - Each new link gets only the level role of the selected page that the action names: `label` for a label, `hosting` for a publisher. The link gets no other role of ADR 0253 item 21 (musicindex ADR 0097 R-ROLE-7 item 4). Review writes the two sides of each link (ADR 0251).
    - A release that is not selected keeps its links.
27. The change also applies to the kept links (item 40) of the page, so a page with no releases also changes. Four actions replace each kept link to a page of the role of the question: "Sign to a label…", "Change label…", "Change publisher…" and "Choose a publisher…". The new kept link goes to the selected page. "Not published" removes each kept link to a publisher page. "Also on another label…" adds a kept link to the selected page and keeps the other kept links. "Go independent…" removes each kept link to a label page.
28. The start selection of the releases: the releases that link to the page above (item 12a). In the tree, the page above is the parent of the row where the user opened the menu. When no release links to that feed, each release of the page is selected. "Also on another label…" starts with no selected release.

### The Artist And The Label Of A Release

29. Each release row has the menu **⋯** with "Artist and label…". The section "Linked artist and label pages" of a release page has the button **Artist and Label…**. With no linked artist page, the section says "A release needs an artist page."
30. "Artist and label…" opens the sheet "Artist and label of {title}". It shows "Now", then three select menus:
    - "Who made it?": the writable artist pages. This menu is necessary.
    - "Which label released it?": first "Independent (no label)", then the writable label pages.
    - "Published by": first the publisher of item 31, then the other writable publisher pages, then "Not published".
    - The last value of each menu is "New Artist Page…", "New Label Page…" or "New Publisher Page…". It opens the form of a publisher page with that role. After the make, the sheet selects the new page.
31. The first value of the menu "Published by" is the publishers that follow from the other two menus, as one value: "As the {label or artist}: {names}". With a label, these are the publishers above the label. With no label, these are the publishers of the other independent releases of the artist, or else the publishers above the artist. This value links the release to each of these publishers. The menu starts with this value. A different value links the release to one publisher, with the role `hosting` only (item 26).
32. The result says "After: {title} is by {artist}, {on label or independent}, {published by names or not published}." With no change, it says "Nothing changes.", and the button **Change** is not available.
33. **Change** links the release to the selected pages. It removes the link to each other page of the collection with the role artist, label or hosting. A link to a page that is not in the collection stays. One route does each link and each removal. When one link fails, the route changes no draft. After **Change**, the sheet closes, the notice of item 35 shows, and the page that was open stays open. The release shows in the group Releases of its new artist page.

### Each Change Waits For Review

34. No change of the Catalog writes a file. Each change of items 21, 26, 27, 33 and 39 goes into one open change set (ADR 0207 items 24 to 33), until Write or Discard. This change set is the change set of the Catalog. This changes ADR 0243 item 1, where each move made its own change set. ADR 0243 items 2 and 6 apply to each change.
34a. ADR 0243 item 5 applies only to a different open change set. When a feed of a change is in a different open change set, Studio does not do the change. The sheet gives the text and the button of ADR 0243 item 5. This changes ADR 0243 item 5.
35. After a change, the sheet closes, and a notice shows "{name} is now {place}. Not written yet." with the buttons **Undo** and **Review** and a close button. Review does not open by itself. This changes ADR 0243 item 3.
36. While the change set is open, the top of the panel says "{N} changes not written", with the button **Review**. Review has the group "What changes" first. It shows each changed page and each changed release with "Now: {place}" and "After: {place}". A page that is only a publisher shows only its name. Then it shows the group "Feeds that Studio writes", and the buttons **Discard** and **Write**.
37. **Undo** puts back the drafts and the kept links of the change set before the last change (ADR 0227). Discard puts back each draft and each kept link of the change set (ADR 0243 item 2).

### Drag And Drop On The Desktop

38. Wider than 760 px, a tree row of an artist or of a label can move with drag and drop. A row of a higher level rank (ADR 0253 item 15) takes the drop. During a drag, each row that takes the drop shows the drop color, and the top of the tree shows the area "Drop here: no label and no publisher".
39. A drop on a row does the change "Change label…" or "Change publisher…" of item 26, with the dragged page and the row of the drop as the selected page:
    - The page above is the parent of the dragged row in the tree.
    - When the start selection of item 28 holds each release of the page, the change occurs at once, and the notice of item 35 shows.
    - When the page has releases that do not link to the page above, the sheet of item 24 opens. The dropped row is the selected page, and the user selects the releases.
    - A drop on the top area does "Go independent…" and then "Not published" (items 26 and 27).
    The menu **⋯** gives each change of a drag, so the keyboard and the phone have each change.

### A Link That Only Studio Keeps

40. When the user makes an artist page or a label page from a page of a higher rank, or selects a page with no releases in a sheet, Studio keeps a link in the collection: the page is below the page of the sheet. Studio writes nothing in a feed file for this link (Stophammer ADR 0069). The core adds each kept link to the rule "below" of ADR 0253 item 15.
41. A kept link goes when Studio links a release to the two feeds, and the link of the upper feed gives a level role of a higher rank. Then the shared release puts the lower feed below the upper feed (ADR 0253 item 15). It also goes when the user removes one of the two feeds from the collection. The undo of that remove gives the link back (ADR 0227).
42. Music Index, HitIt and the other apps read only the feed files. They show the new artist below the label only after a shared release.

### A New Feed Opens In The Tree

43. A new publisher feed opens with its row selected in the tree, and each row above it is open. A new release, show or playlist opens its page. When the user made it from the page of a publisher feed, "‹ Catalog" shows the tree with the row of that feed selected. A new release or playlist also shows its outline (item 7).

### The Menu And The Welcome Page

44. The menu **＋ Add** of the page Releases has these values, in this sequence: Release, Artist, Label, Publisher, Show, Playlist (ADR 0253 item 12). Publisher opens the form of a publisher page with the role Publisher as its start value.
45. The Welcome page has a third row in its first group, after "I run a label": "I publish music for other artists", with the line "Start with your publisher page. The artists and labels that you publish go on it." The row opens the sheet of the first steps with the steps "Your name", "Where it lives" and "Your first artist". The core makes the publisher feed with the role `hosting`. The page of the new feed has the rows "Add an artist" and "Add a label".
46. The page of a new label has the rows "Add an artist" and "Add a release". The row "Find an artist that is online" goes. This changes ADR 0221 item 8.

## Item Map

| Superseded item | ADR 0254 item | Change |
| --- | --- | --- |
| ADR 0085 items 1 and 3 | 2, 3, 6 | The tree in place of one flat list. "No artist yet" in place of "Not linked". |
| ADR 0085 item 2 | 4a | None. |
| ADR 0085 items 4 and 8 | 11, 13a | The head of a publisher page, and the sorts of its group Releases. |
| ADR 0085 item 5 | 9 | The note names only the feeds that are not above or below. |
| ADR 0085 items 6 and 7 | 2, 10 | None. |
| ADR 0181 items 1 and 5 | 6a | A tree row covers its feed and its releases, also when it is open. |
| ADR 0181 items 2 to 4 | 11a | The head of the page shows the state of its feed. The tile goes. |
| ADR 0215 items 1 and 1a | (ADR 0253 item 15) | ADR 0253 owns the rank. |
| ADR 0215 item 2, ADR 0232 item 10 | 4, 40 | ADR 0253 item 15 owns the rule "below". The lists `child_ids` and `parent_ids` do not change. |
| ADR 0215 item 3 | 2, 3 | A tree in place of level 0. "Not linked" becomes "No artist yet". |
| ADR 0215 item 4 | 3 | The headings of ADR 0253 item 16 are for the top rows only. |
| ADR 0215 item 5 | 5 to 7, 11 to 13a | A row opens below its parent. The page shows the head and the groups. The releases keep the feed order. |
| ADR 0215 item 6, ADR 0232 item 8 | 7, 43 | "‹ Catalog" in place of the name of a level. |
| ADR 0215 item 7, ADR 0232 item 7, ADR 0240 item 8 | 10 | The tree is one page of the stack. |
| ADR 0215 item 8 | 9 | None. |
| ADR 0232 items 1 to 3, ADR 0240 items 5 and 7 | 7, 13, 14 | The add rows are on the page, not in the panel. A curator has its own group. A publisher feed has no outline. |
| ADR 0232 items 4 and 6, ADR 0240 item 6 | 15 | None. |
| ADR 0232 item 5 | 18 to 21 | The Add sheet. |
| ADR 0232 items 9, 11 and 12 | 40 to 42 | A page with no releases that a sheet selects also gets a kept link. |
| ADR 0240 items 1 to 3 | 44, 45 | "Publisher" (ADR 0253 item 12). The row "Find an artist that is online" goes. |
| ADR 0240 item 4 | 17 | "Publisher" in place of "Hosting". |
| ADR 0240 items 9 to 19 | 2, 29 to 33 | "Artist and label…" with three menus. "Published by" follows the label. A release with a label and no artist is in "No artist yet". |
| ADR 0240 items 20 to 25 | 22 to 28 | One question for each action, in music words. |
| (new) | 1, 6b, 8, 12, 12a, 34 to 39 | The name "Catalog", a feed with two level roles, the path, one change set with Undo, and drag and drop. |

## Alternatives Considered

- Keep the levels of ADR 0215 and add a path. Not chosen: the user still sees one level at a time.
- Keep one sheet **Move To…** with three menus. Not chosen by the operator: the music words tell the user what happens.
- Drag and drop on the phone. Not chosen by the operator: a long press on a phone conflicts with the scroll, and the menu **⋯** gives each change.
- Show a feed with the roles `artist` and `hosting` as a publisher row with an artist row below it. Not chosen by the operator (ADR 0253).
- An add row in the tree below each open row. Not chosen: the page of the feed has the add rows, and the tree stays short.
- The other alternatives are in the archived records.

## Consequences

- The user sees the full structure in one panel, and opens the rows that matter.
- A change shows in the tree and in Review before the write. Undo removes one change at a time.
- The core move routes take a list of releases and a destination path. The routes of ADR 0240 change.
- A curator page has no action and no add row until a record designs it.

## Non-goals

- The design of a curator page and its playlists.
- A drag of a release row. The sheet "Artist and label…" changes a release.

## Open Points

- The rank of curator relative to label (musicindex ADR 0097 open point 1).
- The place of the menu **＋ Add** stays on the page Releases (ADR 0230 item 1). The prototype shows it in the head of the panel.

## How This Decision Is Checked

Mechanical (the code tasks of this ADR add these tests):

- `crates/core/tests/publisher.rs`: `the_levels_follow_the_rank_and_the_shared_releases` gives `child_ids` and `parent_ids` (item 4). `a_kept_link_places_a_new_feed_below_its_label` gives a kept link (item 40). These tests exist. The code tasks change them to the rank of ADR 0253 item 15.
- A core test of the change of item 26: a selected release loses its label link and gets the links to the selected label and its publisher. A release that is not selected keeps its links. The roles of each new link follow ADR 0253 item 21.
- A core test of the release change of item 33, with "Published by" different from the label.
- A core test that the changes of items 21, 26 and 33 go into one change set (item 34). The test also shows that Undo and Discard put back the drafts and the kept links (item 37).
- `crates/api/ui-tests/shell.test.mjs`: the view model of the tree gives the top rows in their headings and the rows below an open row (items 3 to 5). It also gives the counts, the marks and the second line "Also on" (items 6 to 8). The menu of a page gives the actions of item 23 for each rank.
- The current Node tests of the levels and the panel check the replaced rules. The code tasks change or delete them in the same commit as the new code.
- A Node test of the text of each sheet of item 24 and of the Add sheet (items 18 to 20).

Visual (the main session with screenshots, then the operator):

- The tree at the desktop width and at 390 px, with an open publisher, a label and an artist on two labels.
- A drag of an artist onto a label, with the drop color and the notice.
- Review with "What changes" after two changes.

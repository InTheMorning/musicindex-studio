# ADR 0243: A Move Is A Change Set, And A Level Gathers Its Pages

- Status: Accepted
- Date: 2026-11-22
- Owner: operator
- Accepted: 2026-11-22 by the operator. The operator asked the main session to correct the findings of the tutorial for a hosting user before the tutorial. The operator selected three options from drawings in the questionnaire. A move makes one change set and opens its review. A hosting page moves the artists of the collection below it in one sheet. The folder search goes deeper and reports the feeds that it did not find.
- Technical choices: the main session selected the texts and the start values of item 9. It also selected the depth of 6 and the limit of 5000 files in item 13, and items 6, 7 and 16.
- Bar: crosses modules. It changes the move route, the change set of the core, the panel add rows, the File hosts page and the wallet sheets. Without this record, an agent can keep a move as many separate drafts.
- Changes: [ADR 0240](../archive/adr/0240-hosting-is-a-first-role-and-a-stray-release-moves-to-its-pages.md) items 18 and 19. A move refuses with the names of the feeds, and it makes a change set.
- Changes: [ADR 0207](0207-recipients-change-sets-and-wallets.md) items 10 and 27. The sheet of Use the LNURL Address lists only the feeds that change, and Discard of a move puts back the kept links.
- Changes: [ADR 0232](../archive/adr/0232-each-group-below-an-open-feed-has-an-add-row.md) item 5. On a hosting level and on a label level, an add row opens a menu.
- Changes: [ADR 0019](0019-select-a-feed-from-the-host.md) item 3. The depth and the file limit of a list.
- Changed by: [ADR 0254](0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) items 18 to 21 and 34 to 36, on 2026-11-24. Items 1 and 3: the changes of the Catalog go into one change set, and Review does not open after each change. Item 5: it applies only to a different open change set (ADR 0254 item 34a). Items 7 to 12: the Add sheet replaces the menus of the add rows.
- Uses: [ADR 0253](0253-the-roles-of-a-feed-and-of-each-link.md) for the roles. [ADR 0253](0253-the-roles-of-a-feed-and-of-each-link.md) item 15 for the rule "below". [ADR 0222](0222-errors-give-a-step-terms-get-a-hint-and-the-first-run-has-two-stages.md) for the step of an error.
- Source: [the findings of the tutorial for a hosting user](../reviews/2026-11-22-tj-tutorial-findings.md), findings 1, 2, 3, 4 and 6.

## Context

The tutorial for a hosting user moves three artist pages below a new hosting page. The test of the tutorial found these faults:

- The move of one artist page with 21 releases gave 22 drafts. No control wrote them together, so the user opened Review 22 times.
- The three artist feeds had no role. **Move To…** shows only on a page with the role artist or label. Thus the user set the role in the page form first, three times.
- When one release was read-only, the move sheet said "It is read-only" and did not name the release.
- The folder search stops at a depth of 4 folders and at 1000 files. Feeds in `wp-content/uploads/2023/08/` stayed read-only, and the File hosts page did not tell the user.
- The sheet of Use the LNURL Address selected 8 feeds, and the review had 6, because 2 feeds paid the address already.

A change set already holds a payee operation (ADR 0207) and a move of files (ADR 0148). It has one review, **Write {N} Feeds** and **Discard**.

## Decision

### A Move Makes A Change Set

1. Each move makes one change set (ADR 0207 items 24 to 33). These are the moves: a release (ADR 0254 items 29 to 33), an artist page or a label page (ADR 0254 items 22 to 28), and the pages of item 11. The change set holds each feed whose draft the move changes, with the outcome "changed".
2. Before the first link, the core keeps the draft of each feed of item 1. It also keeps each kept link (ADR 0254 item 40) that the move adds or removes. Discard puts back each draft and each kept link. A page that the user made from the sheet (ADR 0254 item 30) stays after Discard.
3. After **Move**, the sheet closes and Studio opens the review of the change set (ADR 0207 item 28). The review has the title "Move {name} to {page}". When the move has two or more pages, the title is "Move {N} artists to {page}" or "Move {N} labels to {page}".
4. "Changes to write" of People also shows the change set, with the title of item 3.
5. When a feed of the move is in an open change set, the move changes no draft. The sheet says "{name} has changes to write. Write or discard them first." for each such feed, and its button opens that change set.
6. When a feed of the move is read-only, the move changes no draft. The sheet says "Studio cannot change {name}. It is read-only." for each such feed. The sheet has the button **Add a File Host**, which opens File hosts and the sheet **＋ Add a File Host**.

### A Level Gathers Its Pages

7. On a hosting level, **＋ Add artist** opens a menu with "New Artist Page…" and "Move Artists Here…". **＋ Add label** opens a menu with "New Label Page…" and "Move Labels Here…". On a label level, **＋ Add artist** opens the same menu as on a hosting level. A "New … Page…" value opens the form of ADR 0232 item 5. **＋ Add artist** on an artist level does not exist (ADR 0240 item 5).
8. "Move Artists Here…" opens the sheet "Move artists to {page}". It lists each writable publisher page of the collection with the role artist or with no role, that is not below the page of the level now (ADR 0215 item 2). "Move Labels Here…" lists the label pages and the pages with no role in the same way. Each row has a check box, the name, the number of releases, and the role or "No role".
9. Each check box starts selected for a page with no role, and not selected for a page with a role. The button is **Move {N} Artists** or **Move {N} Labels**, with the number of selected rows. With no selected row, the button is not available.
10. Below the list, the sheet says "A page with no role gets the role Artist." or "A page with no role gets the role Label." when a selected row has no role.
11. **Move** gives each selected page with no role the role of the sheet (ADR 0253 items 5 and 10). The core changes the publisher feed and each linked release, as the Role field of the page form does. Then the core applies ADR 0254 items 26 and 27 to each selected page, with the page of the level as the selected page of its role. The links of the other roles do not change.
12. Item 1 to item 6 apply to the move of item 11. One change set holds the feeds of all selected pages.

### A Folder Search Reports What It Did Not Find

13. A list goes to a depth of 6 folders below the root of the connector, and stops at 5000 files. A hidden folder stays out of the list.
14. After **Connect**, the File hosts page shows each read-only feed of the collection that the match did not find, when the public address of that feed has the same host as the public address of the folder. The section has the heading "{N} feeds stay read-only. Studio did not find them in this folder.", one row for each feed with its title and its address, and the button **Add a File Host for Them**.
15. When the list stopped at the file limit, the page also says "Studio stopped at 5000 files. Connect a smaller folder, for example the folder that holds your feeds."

### The Sheet Of Use The LNURL Address Lists Only The Feeds That Change

16. For each feed of a wallet, the core tells if the feed has a `node` recipient of that wallet. The sheet of Use the LNURL Address lists only these feeds. When other feeds of the wallet have no `node` recipient, the sheet says "{N} feeds pay this address already." below the list. **Review in {N} Feeds** counts only the listed and selected feeds.

## Item Map

| Earlier item | Change by this ADR |
| --- | --- |
| ADR 0240 item 18 | Items 5 and 6: the refusal names each feed and gives the next step. |
| ADR 0240 item 19 | Items 1 to 4: the writes go through the review of a change set. |
| ADR 0207 item 10 | Item 16: the sheet of Use the LNURL Address lists only the feeds that change. |
| ADR 0207 item 27 | Item 2: Discard of a move also puts back the kept links. |
| ADR 0232 item 5 | Item 7: on a hosting level and a label level, an add row opens a menu. |
| ADR 0019 item 3 | Item 13: a depth of 6 folders and 5000 files. |

## Alternatives Considered

- The move keeps the drafts, and the review of the moved page gets "Also write the 21 changed releases". Not chosen by the operator: Discard is for each feed, and People does not list the move.
- **Move To…** on a page with no role, with a Role menu in the sheet. Not chosen by the operator: the user moves each artist in a different sheet.
- The search limits stay, and only the report of items 14 and 15 is new. Not chosen by the operator: a WordPress site keeps its feeds 5 folders down.
- A move refuses a read-only release and moves the other releases. Not chosen: the page would list a release that does not link the page.

## Consequences

- A move of a page with many releases has one review and one write.
- A page that the user made from the sheet stays after Discard. The user removes it from the collection when it is not necessary.
- A deeper list takes more time on a slow SFTP server.
- A user who has open payee changes on a feed must write or discard them before a move of that feed.

## How This Decision Is Checked

Mechanical:

- `crates/api/src/tests.rs`: a move of an artist page gives one change set with each release and each page that changes. Discard of that change set puts back each draft byte for byte and each kept link. A move with a read-only release, and a move with a feed in an open change set, change no draft and give each name (items 1, 2, 5 and 6).
- `crates/api/src/tests.rs`: the move of item 11 with two pages with no role gives each page the role artist, links each release to the hosting page, and gives one change set.
- Node tests: the add rows of a hosting level and a label level give the menus of item 7. The model of the sheet of item 8 gives the rows, the start values of item 9, the button text and the note of item 10.
- `crates/core/src/connector/local.rs` and `sftp.rs` tests: a list finds a file 6 folders down and stops at 5000 files (item 13).
- `crates/api/src/tests.rs`: the match gives the read-only feeds of item 14 and the stop of item 15.
- A Node test of the sheet of Use the LNURL Address: it lists only the feeds with a `node` recipient and gives the note of item 16.

Visual (a person, or the main session with screenshots):

- The review that opens after **Move**, the sheet of item 8, the refusal of item 6, and the report of item 14, in the dark theme at 1280 px and at 390 px.

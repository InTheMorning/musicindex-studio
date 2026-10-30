# ADR 0039: The Shell, Releases And The Outline Of A Feed

- Status: Accepted
- Date: 2026-10-25
- Owner: operator
- Replaces: [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) items 5, 6 and 15, and its situational invariant "The sidebar tree has two levels".
- Amended: 2026-10-25 by operator decision. Item 1 pins Hosts and Lock Studio to the bottom of the left panel. They always show, also with the outline of item 5 (musicindex ADR 0027 R-SHELL-1 item 4).
- Amended: 2026-10-25 by operator decision. Items 10 and 11 put the left panel behind a menu button at narrow width. The menu button is at the top left and always visible. No tab bar shows.
- Amended: 2026-10-25 by operator decision. Item 2a gives the sort and the view of Releases, as musicindex ADR 0027 R-LIST-1 item 6 asks. [ADR 0042](0042-ui-models-with-node-tests.md) gives the method of the mechanical criteria.
- Amended: 2026-10-25 by operator decision, after the check of the touch back control in Firefox. Item 10a gives the bar of a feed two rows at narrow width.
- Amended: 2026-10-25 by operator decision. musicindex ADR 0030 replaces the menu button and the drawer. Items 10, 10a, 11 and 11a now make Studio one stack at narrow width, with the root page and the outline page.
- Amended: 2026-10-26 by operator decision, after the visual check of task 034f. On a server with a locked vault, "Unlock Studio" takes the place of "Lock Studio" in item 1. It opens the unlock sheet of [ADR 0056](0056-server-sign-in-methods-and-vault-wrappers.md).
- Amended: 2026-10-27 by operator decision. [ADR 0069](0069-the-outline-head-opens-the-feed-page.md) changes items 6 and 7. The outline head opens the first page, and the outline has no row for it. The link shows "‹ Releases".
- Amended: 2026-10-25 by operator decision. musicindex ADR 0031 makes the Touch back control of item 9 a circular icon button with no word.
- Amended: 2026-10-25 by operator decision. Items 11 and 11a give the root page and the outline page the color of the left panel.
- Amended: 2026-10-28 by operator decision. [ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) items 9 and 10 add the destination "Settings" between Hosts and Lock Studio in items 1, 5 and 11.
- Amended: 2026-10-28 by operator decision. [ADR 0072](0072-add-a-feed-opens-from-releases.md) removes Add a feed from items 1, 5 and 11. It opens from a button on Releases.
- Amended: 2026-10-28 by operator decision. [ADR 0073](0073-the-add-pages-ask-one-question-at-a-time.md) item 11 replaces "Add a track" and "Import files" in item 6 with one row "Add tracks".
- Amended: 2026-10-30 by operator decision. [ADR 0085](0085-each-artist-and-label-is-a-destination.md) adds a destination below Releases for each publisher feed in items 1 and 11. Releases no longer lists the publisher feeds (item 2).
- Amended: 2026-10-30 by operator decision. [ADR 0091](0091-a-change-set-edits-one-recipient-in-many-feeds.md) item 1 adds the destination "Recipients" before Hosts in items 1, 5 and 11.
- Amended: 2026-10-30 by operator decision. [ADR 0092](0092-file-hosts-and-the-destination-icons.md) renames Hosts to "File hosts" in items 1, 5 and 11, and gives each fixed destination an icon. No destination row shows a type dot.

## Context

musicindex ADR 0027 R-SHELL-1 gives the shell of musicindex.org, Studio and v4vmm: a stack with two exceptions. Studio is the case of exception 1, an editor of one item with many parts. musicindex ADR 0028 R-PAGE-1 gives the sequence of the sections of an item page. Studio follows these records. It does not change them.

On 2026-10-24 the research examined Studio at commit `24980a3` (`docs/plans/research-ui-shell-request.md`). It found these facts:

- The sidebar holds a tree of the feeds and their pages ([ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) items 5 and 6).
- With no feed open, the editor area says "Select a feed". It is a placeholder.
- The tree has Credits and links before Recipients and before the tracks.

The operator checked the Studio mockup of the research repository (`research_notes/ui-design/mockups/studio.html`) and lifted the hold on this ADR on 2026-10-25.

## Decision

### The Destinations

1. With no feed open, the left panel holds the destinations. Releases is at the top. Add a feed opens from Releases ([ADR 0072](0072-add-a-feed-opens-from-releases.md)). Hosts, Settings and Lock Studio are at the bottom of the panel, and stay there at each height of the window. On a server with a locked vault, "Unlock Studio" is in the place of "Lock Studio".
2. Releases is a list page at the full width of the content area. Each row shows the cover, the type label, the host and the draft state, for example "Saved, not written".
2a. Releases has the sort and the view of musicindex ADR 0027 R-LIST-1:
   - The sorts are "Newest", the default, and "A to Z" with a letter bar. "Newest" gives the feed that joined the collection last first.
   - The views are rows, the default, and tiles.
   - A sort or a view that is not the default is in the query of the address, as `sort=az` and `view=tiles`.

3. Releases is the start page. An empty collection starts the first-run wizard of [ADR 0020](0020-first-run-asks-one-question.md).
4. No page is a placeholder. "Select a feed" goes.

### The Outline Of An Open Feed

5. While a feed is open, the left panel shows the outline of that feed in place of Releases. Hosts, Settings and Lock Studio stay at the bottom (item 1). The left panel is one panel at any time.
6. The outline shows a link "Releases", the cover and the title of the feed, then these items in this sequence: Album, each track by its number and title, "Add tracks" ([ADR 0073](0073-the-add-pages-ask-one-question-at-a-time.md) item 11), Value routes, Credits and links, Files.
7. The link "Releases" opens the Releases page at the same scroll position, with the row of the feed marked. It is a destination link, not a back control (musicindex ADR 0027 R-NAV-2 item 5).
8. Each item of the outline opens one page of the editor. [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) items 7 to 9 stay.

### Back

9. The editor bar has the back control of musicindex ADR 0027 R-NAV-2 at its leading edge: the icons `‹ ›` on Desktop, and the circular icon button `‹` named "Back" on Touch (musicindex ADR 0031 R-NAV-3).

### Narrow Width

10. At 760 px or less, Studio is one stack (musicindex ADR 0030 R-SHELL-2). It has no menu button and no drawer. The back control of item 9 is the first control of the bar.

10a. At 760 px or less, the bar of a feed has two rows. The first row has the back control and the title. The second row has the draft status, then "Versions" and "Review". No part of the bar is hidden.

11. The root page "Studio" shows the destinations of item 1: Releases, with Hosts, Settings and Lock Studio at its bottom. It has no back control. Releases stays the start page (item 3), and the root page is below it in the stack. The root page has the color of the left panel, and its bar has the color of the other bars.

11a. A selection of a feed on Releases opens the outline page of the feed: the items of item 6 as a page. A selection on the outline page opens its part. Back from a part opens the outline page, and Back from the outline page opens Releases with the row of the feed marked. The outline page has the color of the left panel, as the root page has.

### Value Routes

12. The Value routes page is at most 560 px wide. A share segment and a percentage show a tooltip at once, with the name, the percentage and the raw share (musicindex ADR 0020 R-VALUE-1 item 3).

## Alternatives Considered

- Keep the tree in the sidebar. Rejected by the operator: the tree is a second place of work, and Studio opens on a placeholder.
- A stack with no outline. Rejected: the person then goes back to move between the tracks of one album.
- A row "‹ Releases" as the only way back. The research mockup draws it. Rejected: musicindex ADR 0027 R-NAV-2 gives the back control in the bar on each profile. The link stays as a destination link.

## Consequences

- The sidebar tree of ADR 0029 goes, and the outline replaces it.
- The tests that examine the tree change with the build of this ADR.
- The draft status, "Versions" and "Review" stay in the editor bar ([ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) items 1 to 4).
- Open: a search of the collection (O-9). No decision covers it.

## Verification

Mechanical, at the owning layer, in the model modules of [ADR 0042](0042-ui-models-with-node-tests.md):

- With no feed open, the UI model of the left panel gives the destinations of item 1, and no text "Select a feed".
- With a feed open, the outline model gives its items in the sequence of item 6.
- After the link "Releases", the Releases page gives the marked feed row.
- The Releases model gives the rows in the sequence of each sort, and the letter groups of "A to Z".
- The navigation model disables Forward when no subsequent page exists (musicindex ADR 0027 R-NAV-2 item 4).
- At narrow width, the navigation model gives the sequence of item 11a, and the root page below Releases (item 11).

Visual, for the operator:

- Studio starts on Releases, and a feed opens over it with its outline.
- Hosts, Settings and Lock Studio are at the bottom of the left panel in a short window and in a tall window.
- At 390 px, no page has a menu button. Back on Releases opens the root page, and a feed opens on its outline page.

## Invariants

Situational, until a decision replaces this layout:

- The left panel holds Releases, or the outline of the open feed. Hosts, Settings and Lock Studio always show at its bottom. On a server with a locked vault, "Unlock Studio" is in the place of "Lock Studio". At 760 px or less, the root page and the outline page hold the same content.

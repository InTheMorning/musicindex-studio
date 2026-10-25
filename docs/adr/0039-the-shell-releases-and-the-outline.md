# ADR 0039: The Shell, Releases And The Outline Of A Feed

- Status: Accepted
- Date: 2026-10-25
- Owner: operator
- Replaces: [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) items 5, 6 and 15, and its situational invariant "The sidebar tree has two levels".
- Amended: 2026-10-25 by operator decision. Item 1 pins Hosts and Lock Studio to the bottom of the left panel.

## Context

musicindex ADR 0027 R-SHELL-1 gives the shell of musicindex.org, Studio and v4vmm: a stack with two exceptions. Studio is the case of exception 1, an editor of one item with many parts. musicindex ADR 0028 R-PAGE-1 gives the sequence of the sections of an item page. Studio follows these records. It does not change them.

On 2026-10-24 the research examined Studio at commit `563cce6` (`docs/plans/research-ui-shell-request.md`). It found these facts:

- The sidebar holds a tree of the feeds and their pages ([ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) items 5 and 6).
- With no feed open, the editor area says "Select a feed". It is a placeholder.
- The tree has Credits and links before Recipients and before the tracks.

The operator checked the Studio mockup of the research repository (`research_notes/ui-design/mockups/studio.html`) and lifted the hold on this ADR on 2026-10-25.

## Decision

### The Destinations

1. With no feed open, the left panel holds the destinations. Releases and Add a feed are at the top. Hosts and Lock Studio are at the bottom of the panel, and stay there at each height of the window.
2. Releases is a list page at the full width of the content area. Each row shows the cover, the type label, the host and the draft state, for example "Saved, not written".
3. Releases is the start page. An empty collection starts the first-run wizard of [ADR 0020](0020-first-run-asks-one-question.md).
4. No page is a placeholder. "Select a feed" goes.

### The Outline Of An Open Feed

5. While a feed is open, the left panel shows the outline of that feed in place of the destinations. The left panel is one panel at any time.
6. The outline shows a link "Releases", the cover and the title of the feed, then these items in this sequence: Album, each track by its number and title, "Add a track", "Import files", Value routes, Credits and links, Files.
7. The link "Releases" opens the Releases page at the same scroll position, with the row of the feed marked. It is a destination link, not a back control (musicindex ADR 0027 R-NAV-2 item 5).
8. Each item of the outline opens one page of the editor. [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) items 7 to 9 stay.

### Back

9. The editor bar has the back control of musicindex ADR 0027 R-NAV-2 at its leading edge: the icons `‹ ›` on Desktop, and `‹ Back` on Touch.

### Narrow Width

10. At 760 px or less, the destinations are in a tab bar when no feed is open.
11. At 760 px or less, the outline of an open feed is a drawer. A button in the editor bar opens it.

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
- Open: the drawer against a tab bar for the outline at narrow width (open question O-8 of the Studio journeys in the research repository).
- Open: a search of the collection (O-9). No decision covers it.

## Verification

Mechanical, at the owning layer:

- With no feed open, the UI model of the left panel gives the four destinations, and no text "Select a feed".
- With a feed open, the outline model gives its items in the sequence of item 6.
- After the link "Releases", the Releases page gives the marked feed row.

Visual, for the operator:

- Studio starts on Releases, and a feed opens over it with its outline.
- Hosts and Lock Studio are at the bottom of the left panel in a short window and in a tall window.
- At 390 px, the outline opens as a drawer from the editor bar.

## Invariants

Situational, until a decision replaces this layout:

- The left panel holds the destinations, or the outline of the open feed. It never holds both.

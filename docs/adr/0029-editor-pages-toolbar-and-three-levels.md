# ADR 0029: Editor Pages, The Toolbar And Three Background Levels

- Status: Accepted
- Date: 2026-10-22
- Owner: operator
- Changed by: musicindex ADR 0023 R-WORD-2, on 2026-10-25. The page "Recipients" of item 5 is "Value routes". [ADR 0010](0010-studio-uses-the-afterglow-direction.md#rules-that-musicindex-replaces) names the change.

## Context

On 2026-10-22 the operator examined the album editor after [task 012](../tasks/studio-v1-task-012-value-editor.md). The operator reported three problems:

- The draft status, "Examine the changes", the write and the earlier versions are at the end of a page of about 2,800 px. The operator scrolled to the end for each diff or version.
- No control goes directly to a track. The operator asked for a tree in the sidebar.
- The operator cannot easily read the many shades of gray. The page has six backgrounds: the page, the sidebar, the panel, the entry, the open track and the field. Most levels also have a border. The tracks use a border and the recipients use a fill, at the same level.

On a phone, the sidebar takes about 45 % of the screen above the editor. The hero and the feed facts take one full screen before the first field.

The Apple guidance gives these points:

- Grouped content uses three background levels: the page, a group, and an item in a group (`systemGroupedBackground`, `secondarySystemGroupedBackground` and `tertiarySystemGroupedBackground`).
- A sidebar uses disclosure controls and shows "no more than two levels of hierarchy".
- The frequent commands and the primary action of a toolbar "remain visible at all window sizes".
- At compact width, a split view puts the sidebar behind a control.

The operator answered two rounds of questions and examined a mockup at desktop and phone width. The operator accepted each part of the mockup. The mockup is in commit `d1e6fe2`, at `docs/notes/2026-10-22-editor-layout-mockup.html`. Task 014 removed it after its visual check.

## Decision

### The Toolbar

1. The bar at the top of the editor stays in view when the page scrolls. It shows the draft status, a "Versions" button and a "Review" button at all times.
2. "Review" opens a sheet with the diff. The sheet has "Discard the draft" and "Write to the host". The write rule of [ADR 0007](0007-draft-diff-write-and-announce.md) does not change.
3. "Versions" opens a sheet with the snapshots and "Put in the draft".
4. The editor page has no Changes panel and no Earlier versions panel.

### The Tree And The Pages

5. In the sidebar, a feed row opens and closes. Below the open feed, the tree shows these pages: Album, Credits and links, Recipients, a "Tracks" label, each track by its number and title, and "Add a track".
6. The tree has two levels: the feed, and its pages.
7. Each item of the tree opens one page of the editor. A track page shows one track, with buttons that go to the track before it and to the track after it.
8. "Add a track" opens a sheet with the fields of [ADR 0023](0023-web-addresses-and-audio-file-facts.md) items 6 and 7.
9. The recipients of a track are a section of the track page. They are not in a disclosure group.

### The Header

10. The editor page of a feed has a compact header: the cover, the type, the title, the feed address and "Read again". The feed facts are a collapsed row.

### Background Levels

11. The editor uses three background levels:
    - level 1: the page.
    - level 2: a group, which holds the rows of one section.
    - level 3: an open entry in a group.
12. Each level changes the fill only. A border does not identify a level. Thin separators divide the rows of a group.
13. A text field has a fill and no border. A field is not a level.
14. Studio adds three tokens for the levels. The dark values are the values of `--bg`, `--surface` and `--surface2`. The light values are `#f2f2f7`, `#ffffff` and `#f2f2f7`. The tokens copied from `search.html` do not change ([ADR 0010](0010-studio-uses-the-afterglow-direction.md)).

### Phone Width

15. At 760 px or less, the sidebar is a drawer behind a menu button in the toolbar. The editor uses the full width.
16. At 760 px or less, a sheet comes up from the bottom of the screen. Each label goes above its field.

## Alternatives Considered

- A bar at the bottom of the editor. Rejected by the operator: it covers content, and the top bar exists.
- The draft controls in the sidebar. Rejected: on a phone they are behind the menu button.
- One long page, and the tree scrolls to the track. Rejected by the operator: the page stays long.
- Only the tracks in the tree. Rejected by the operator: the album parts then stay far down the page.
- The sidebar above the editor on a phone, as before. Rejected by the operator: the tree makes it longer.
- The large hero on the editor page. Rejected by the operator: it takes one full screen on a phone.

## Consequences

- This ADR changes ADR 0010 item 1 for the editor page: the editor page has a compact header and not the detail hero. ADR 0010 has a note that names this ADR.
- The summary lines of [ADR 0028](0028-summary-lines-and-one-open-entry.md) stay. A track is a page, not an entry that opens.
- The recipient picker of [task 013](../tasks/studio-v1-task-013-recipient-picker.md) goes into the Recipients page and the recipients section of a track page. Task 014 comes first, so that the picker is made one time.
- A test examines `styles.css`. Each background must use an allowed token.

## Invariants

Durable:

- The editor uses at most three background levels. A level changes the fill only.

Situational, until a decision replaces this layout:

- The user can see the draft status, "Versions" and "Review" at all scroll positions and at all widths.
- The sidebar tree has two levels.

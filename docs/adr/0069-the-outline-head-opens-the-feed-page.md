# ADR 0069: The Outline Head Opens The Feed Page

- Status: Accepted
- Date: 2026-10-27
- Owner: operator
- Accepted: 2026-10-27 by the operator.
- Changes: [ADR 0039](0039-the-shell-releases-and-the-outline.md) items 6 and 7.

## Context

[ADR 0039](0039-the-shell-releases-and-the-outline.md) item 6 gives the outline of an open feed. It shows a link "Releases", the cover and the title of the feed, then a row for the first page. That row has the name "Album", "Playlist", "Artist", "Label" or "Feed". The rows of the tracks follow it. The head with the cover and the title is not a control.

On 2026-10-27 the operator examined the outline of an album, and found these defects:

- The function of the row "Album" is not easy to see. The head above it names the album, but the user cannot select the head.
- The link "Releases" does not show that it opens the list of all releases, one level higher.

musicindex ADR 0028 R-PAGE-1 gives the outline of an editor in this sequence: the item, its tracks, the value routes, then the credits. A head that opens the page of the item agrees with this rule. musicindex ADR 0027 R-NAV-2 item 5 says that a destination link in the outline is not a back control. musicindex.org and v4vmm have no outline (musicindex ADR 0027 R-SHELL-1 item 4).

## Decision

### The Head

1. The head of the outline is one button: the cover, the title and the type label of the feed. It opens the first page of the feed: the album page, the playlist page, the publisher page, or the page of a read-only feed.
2. The outline has no row for the first page. The rows "Album", "Playlist", "Artist", "Label" and "Feed" go.
3. When the first page is open, the head has the selected fill and `aria-current="page"`, as each other row of the outline has.
4. The accessible name of the head is the title of the feed, then its type label.
5. The head is the same in the left panel and on the outline page at 760 px or less ([ADR 0039](0039-the-shell-releases-and-the-outline.md) item 11a).

### The Link To Releases

6. The link shows "‹ Releases": the back chevron icon of the bar, then the word "Releases". Its accessible name is "Releases".
7. The link stays a destination link. It opens the Releases page at the same scroll position, with the row of the feed marked, as ADR 0039 item 7 says. It does not go back in the history (musicindex ADR 0027 R-NAV-2 item 5).
8. The link shows only in the left panel at a wide width. At 760 px or less, the back control of the bar does this function (ADR 0039 item 11a).

## Alternatives Considered

- Keep the row "Album". Not chosen by the operator: the head and the row then name the same page in two places.
- An icon of a stack of covers before "All releases". Not chosen by the operator: it shows that the link goes up less clearly.
- A muted line "Releases ›" in the head, as a breadcrumb. Not chosen by the operator: the head then has two targets.

## Consequences

- The outline model gives the head as the first item of the outline, with its `current` value. It gives no row for the first page.
- The chevron of the link to Releases and the back control of the bar look almost the same. The link has a word, and the back control has no word. The visual check examines this.
- The read-only feed of [ADR 0067](0067-a-read-only-feed-opens-in-the-editor.md) loses its row "Feed" too.

## Invariants

- The outline has no row that names the first page of the feed.
- The link to Releases opens Releases. It does not go back in the history.

## Verification

Mechanical:

- A Node test examines the outline model of an album, a playlist, a publisher feed and a read-only feed. Each gives the head first, with no row for the first page.
- A Node test: with the first page open, the head is current, and no other item is current.
- A Node test: the link to Releases opens the Releases view, not the history entry before it.

Visual:

- At desktop width, the head of an album shows the selected fill when the album page is open.
- At desktop width, "‹ Releases" shows its chevron. The link and the back control of the bar look different.
- At 390 px, the outline page shows the head as a button, with no row "Album".

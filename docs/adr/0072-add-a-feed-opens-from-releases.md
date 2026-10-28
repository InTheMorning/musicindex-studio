# ADR 0072: Add A Feed Opens From Releases

- Status: Accepted
- Date: 2026-10-28
- Owner: operator
- Accepted: 2026-10-28 by the operator.
- Changes: [ADR 0039](0039-the-shell-releases-and-the-outline.md) items 1, 5 and 11, its verification and its invariant.

## Context

[ADR 0039](0039-the-shell-releases-and-the-outline.md) item 1 puts Releases and Add a feed at the top of the left panel. Item 11 puts the same two rows on the root page at narrow width.

The other destinations are pages for a set of objects: Releases, Hosts and Settings. Add a feed is a procedure that changes the collection of Releases. The Hosts page gives its procedure in the page: "＋ Add a host" ([ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) item 12).

On 2026-10-28 the operator examined Studio on a phone. Studio opens on Releases. To add a feed, the user goes back to the root page first. The operator asked if Add a feed must be in Releases. The operator selected this decision.

musicindex ADR 0027 R-SHELL-1 item 1 puts the destinations in the sidebar. It does not make a procedure a destination. This ADR changes no shared rule.

## Decision

1. The top of the left panel holds Releases only. Hosts, Settings and Lock Studio stay at the bottom (ADR 0039 item 1). The root page at narrow width holds the same rows (ADR 0039 item 11).
2. When the collection has a feed, the head of the Releases page shows the button "＋ Add a feed", with the role `action` ([ADR 0037](0037-button-roles.md)). It is after the sort and the view controls.
3. An empty collection keeps its primary button "Add a Feed" in the empty page. The head then shows no second button.
4. The button opens the page "Add feeds". The page and its sections do not change.
5. The Add feeds page is a page of the Releases destination:
   - The row Releases is current while the Add feeds page is open.
   - At narrow width, the stack is the root page, Releases, then Add feeds. Back from Add feeds opens Releases.
6. After a feed is added, Studio opens that feed, as before.

## Alternatives Considered

- Keep Add a feed as a destination. Not selected: a procedure then has the position of a destination, and a phone user goes back to the root page to add a feed.
- Show the row and the button. Rejected: two paths go to one page, and the row stays a procedure in the list of destinations.
- Put the button in the bar of Releases. Not recommended: musicindex ADR 0027 R-TOOLBAR-1 item 1 gives the bar the back control and the title. The head of the list holds the list controls.

## Consequences

- The left panel has one row at its top.
- The user guide pages that say "Select **Add a feed** in the left panel" change to the button on Releases.
- The welcome text of the first run names "Add a feed". It stays correct, because the page and the button keep that name.

## Invariants

Situational, until a decision replaces this layout:

- The top of the left panel and of the root page holds only Releases.
- Add feeds opens from Releases, and Back from it at narrow width opens Releases.

## Non-Goals

- No change to the Add feeds page, its sections or its forms.
- No change to the outline of an open feed.
- No change to the shared shell rules of musicindex ADR 0027.

## Verification

Mechanical, in the model modules of [ADR 0042](0042-ui-models-with-node-tests.md):

- With no feed open, `leftPanel` gives `top` as Releases only, and `bottom` as before.
- `leftPanel` gives Releases as current on the view `add-feed`.
- `narrowStack({ name: 'add-feed' })` gives the root page, Releases, then Add feeds.

Visual:

- At desktop width and at 390 px, in the two themes, Releases shows "＋ Add a feed" in its head. The button opens Add feeds, and Back opens Releases.
- The empty Releases page shows only its primary button.
- `$edges()` of `tools/drive-ui.mjs` finds no content at the edge of a group.

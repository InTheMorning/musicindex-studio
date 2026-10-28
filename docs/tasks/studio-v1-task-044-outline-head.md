# Studio v1 Task 044: The Outline Head

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The Editor Files And Covers".
Decisions: [ADR 0069](../adr/0069-the-outline-head-opens-the-feed-page.md), [ADR 0039](../adr/0039-the-shell-releases-and-the-outline.md) items 6, 7 and 11a, and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).
Sequence: one packet. It is the first task of tasks 044 to 046.

Status: Complete - 2026-10-27. The four commit checks pass. The screenshot visual check passed, and the [review checklist](../reviews/2026-10-27-tasks-044-046-review-checklist.md) records it.

## Goal

The head of the outline opens the first page of the feed, and the outline has no row for that page. The link to Releases shows "‹ Releases". It opens Releases, and it does not go back in the history.

## Files

Inspect: `crates/api/ui/shell.js` (`outlineItems`, `leftPanel`, `isCurrentPage`, `pushNav`, `backNav`, `setScroll`), `crates/api/ui/app.js` (`Outline`, `LeftPanel`, `showReleases`, `go`, `withScroll`, `Chevron`, the scroll `useEffect` after "After a change of view"), `crates/api/ui/styles.css` (`.row`, `.node`, `.outline-feed`, `.outline-cover`), `crates/api/ui-tests/shell.test.mjs`.

Change: `crates/api/ui/shell.js`, `crates/api/ui/app.js`, `crates/api/ui/styles.css`, `crates/api/ui-tests/shell.test.mjs`.

Do not change:

- `crates/api/ui/editor.js` and each other UI module.
- `backNav`, `forwardNav` and the browser history mirror of `app.js`.
- The rows of the outline other than the first-page row.
- Each Rust file.

## Architecture

These choices are fixed. Do not change them.

### The Model

- `outlineItems` gives no item for the first page. Remove the items with the IDs `album`, `publisher`, `playlist` and `feed`. Keep each other item and its sequence.
- `leftPanel` gives `feed.page` as `'album'` and `feed.current` as `isCurrentPage(view, 'album')` in the outline result.
- `leftPanel` gives `feed.label`: the title, a comma and a space, then the type label. For example `Disciples by Design, Album`.
- Add `export function releasesView(nav, feedId)` to `shell.js`. It gives `{ name: 'releases', marked: feedId }`. When an entry with the name `releases` is at or before `nav.index`, the result also gets the `scroll` of the nearest such entry. The function does not change `nav`.

### The Outline

- In `Outline`, the head `div.outline-feed` becomes `button.outline-feed` with `type="button"`, `aria-current` from `currentAttr(panel.feed)` and `aria-label` from `panel.feed.label`. When the feed has the change mark, the label gets `, changes in the library folder` at its end.
- A click on the head calls `go({ name: 'feed', id: feed.id, page: 'album' })`. When the head is current, the click does nothing.
- The row "Releases" shows `<${Chevron} direction="back" />` and then the word "Releases". The chevron is `aria-hidden`, so the accessible name is "Releases".
- `showReleases(feedId)` calls `go(releasesView(withScroll(), feedId))`. Remove the call of `back()` from `showReleases`.

### The Styles

- `button.outline-feed` has no border, no background, the font of the page and `text-align: left`. Its width is 100 %, its minimum height is `var(--target)`, and its corner radius is the radius of `.node`.
- Its hover fill is the hover fill of `.node`. Its `aria-current="page"` style is the style of `.node[aria-current="page"]`. Use one selector for both. Do not write the values two times.
- The focus ring is the focus ring that each other button has.
- The chevron of the Releases row has the size of the chevron of the bar, and a gap of 6 px before the word.

## Steps

1. Change `outlineItems` and `leftPanel`. Add `releasesView`.
2. Change the Node tests.
3. Change `Outline`, `showReleases` and the styles.
4. Run the four checks.
5. Make the screenshots of the visual check with `tools/drive-ui.mjs`, and run `$edges()`.

## Constraints

- No feed rule in the UI (ADR 0065). This packet changes no model of a feed.
- Each code comment is in ASD-STE100 Simplified Technical English, as the files are. Each comment that names ADR 0039 item 6 also names ADR 0069.
- The UI loads nothing from other servers (ADR 0016).
- All functions work from the keyboard. The head is reachable with Tab, and Enter and Space open the page.

## Acceptance Criteria

Mechanical:

- A Node test for each feed type: an album, a playlist, a publisher feed and a read-only feed. The outline items have no ID `album`, `publisher`, `playlist` or `feed`.
- A Node test: the labels of the narrow outline of the test album are `First`, `Untitled track`, `Add a track`, `Import files`, `Live shows`, `Add a live show`, `Value routes`, `Credits and links`, `Feed settings` and `Files`, in this sequence.
- A Node test: with the view `{ name: 'feed', id: 1 }`, `feed.current` is true, and no item is current. With a track page open, `feed.current` is false.
- A Node test: `feed.label` is the title, a comma and a space, then the type label.
- A Node test: `releasesView` gives the `scroll` of the nearest Releases entry before the index, and no `scroll` when no Releases entry is before it. The `nav` that the test gives does not change.
- The four commit checks pass.

Visual, for the agent with `tools/drive-ui.mjs`, in the dark theme and the light theme:

- At desktop width, the head of an album has the selected fill when the album page is open, and no fill on a track page.
- At desktop width, "‹ Releases" shows its chevron. The link and the back control of the bar look different.
- At 390 px, the outline page shows the head as a button, and no row "Album".
- Tab reaches the head, and the focus ring shows. Enter opens the album page.
- `$edges()` finds no content at the edge of a group.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a different module reads the item with the ID `album` of the outline.
- Stop and report when Releases does not get its scroll position again without `back()`.
- Stop and report when the head is smaller than the target size in the narrow outline page.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-044-outline-head.md
- docs/adr/0069-the-outline-head-opens-the-feed-page.md
- docs/adr/0039-the-shell-releases-and-the-outline.md
- The files that the Files section names.

Goal:
- The outline head is a button that opens the first page of the feed. The outline has no row for that page.
- The link to Releases shows "‹ Releases" and opens Releases with no history back.

Constraints:
- Follow the Architecture section of the task file fully.
- Use the selectors of `.node` for the head. Do not write the values two times.
- Write each code comment in ASD-STE100 Simplified Technical English, as the files do.

Do not touch:
- `crates/api/ui/editor.js`, `backNav`, `forwardNav`, the history mirror and each Rust file.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

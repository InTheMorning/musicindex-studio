# ADR 0225: The First Tab Of A Release Is Release

- Status: Accepted
- Date: 2026-11-19
- Owner: operator
- Accepted: 2026-11-19 by the operator, in the human check of the AppImage of `98d8bfa`. The operator gave the two labels.
- Bar: opposite choice by mistake. The first tab used the kind of the release, and the first section used "Album". An agent can make the same choice again.
- Changes: [ADR 0135](../archive/adr/0135-the-panel-holds-the-content-and-the-page-has-tabs.md) item 2. The first tab of an album is "Release", not "Album" (item 1).
- Uses: [ADR 0256](0256-a-release-starts-on-the-page-of-its-artist.md) items 13 to 15 and [ADR 0223](0223-a-split-in-shares-or-percent-a-release-search-and-one-tap-on-a-phone.md) item 11, for the kind that Studio shows.

## Context

The first tab of a release showed the kind of the release, for example "Single". The first section of that tab had the heading "Album" for each kind. The head of the page also shows the kind above the title. Thus the tab and the heading gave two different names for the same release.

## Decision

1. The first tab of a feed with the medium `music` is "Release" for each kind. The first tab of an artist, a label and a playlist does not change.
2. The heading of the first section of that tab is the kind that Studio shows: "Release", "Album", "EP" or "Single" (`feedKind` in `shell.js`).

## Alternatives Considered

- Keep the kind in the tab and "Album" in the heading. The operator rejected it, because the tab name changes with the kind and the heading is incorrect for a single.

## Consequences

- The tab name is the same for each release. The menu "Kind" and the heading show the kind.

## How This Decision Is Checked

- Mechanical: `crates/api/ui-tests/shell.test.mjs`, the test "the page of a feed gives the tabs of its type", checks item 1.
- Visual: a screenshot of the first tab of a release shows the tab "Release" and the heading of its kind (item 2). The heading is in the render layer, so no model test reaches it.

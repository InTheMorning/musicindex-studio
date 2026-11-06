# ADR 0134: A UI Scale And Larger Release Tiles

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Uses: musicindex ADR 0041 R-SCALE-4, and the UI scale of v4vmm that musicindex ADR 0039 names
- Source: [findings of local use, finding 15](../reviews/2026-11-06-local-use-findings.md)

## Context

A browser lets a user zoom a page, and the desktop app has no such control. v4vmm has a "UI scale" setting in five levels. The Apple accessibility guidance asks an app to let people adjust the size of text, through its own control when necessary.

The Apple guidance on collections gives no tile size. Studio had a minimum of 160 px, and no shared rule owns this value.

## Decision

1. The Settings page has a select menu "UI scale", with the levels Extra small, Small, Medium, Large and Extra large. Medium is the default.
2. The factors are those of v4vmm: 0.85, 0.92, 1.0, 1.12 and 1.25.
3. The level applies with a mouse, in each shell. With touch, Studio shows no menu and applies no factor.
4. The browser keeps the level, as it keeps the theme. The level is not account data, and it is not in an export.
5. The level scales the whole page through CSS, so the app needs no Tauri command (ADR 0124).
6. A release tile has a minimum width of 200 px, at each level before its factor. This value belongs to Studio.

## Alternatives Considered

- The zoom of the Tauri window. Not chosen: it needs a command for the page, and ADR 0124 gives the page no Tauri command.
- A factor of 2.0, the ideal of the Apple guidance. Not chosen: the operator asked for the levels of v4vmm.

## How This Decision Is Checked

Mechanical: `crates/api/ui-tests/uiscale.test.mjs` examines the levels and the factors. `crates/api/ui-tests/pages.test.mjs` examines the section.

Visual: the planning model looks at each level in Chromium and in the Tauri window, including the height of the shell. The operator judges the result on the real app.

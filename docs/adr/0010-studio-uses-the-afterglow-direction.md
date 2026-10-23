# ADR 0010: Studio Uses The Afterglow Visual Direction

- Status: Accepted
- Date: 2026-10-22
- Owner: operator
- Changed by: [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md). The editor page of a feed has a compact header, not the detail hero.

## Context

The public site `musicindex.org` uses the "Afterglow" visual direction (musicindex ADR 0003).
Its entity colors are in musicindex ADR 0012.
These ADRs apply only to `search.html`. Studio can adopt them only by its own decision.

## Decision

1. Studio adopts the full Afterglow direction:
   - the one-color logo mark, the brand assets and the self-hosted Figtree font.
   - the dark and light themes and the blue accent.
   - the sidebar layout and the detail hero with glow and grain.
   - the entity colors and their rules.
2. The source of the tokens is `../musicindex/search.html` and musicindex ADR 0012.

## Consequences

- Studio adds editor screens that the public site does not have. These screens use the same tokens.

## Invariants

- An entity color is not a general accent color.
- Red as an entity color means only "live now". Errors use their own token ([ADR 0018](0018-passphrase-strength-and-error-color.md)).
- Color is not the only indication of a type.
- Each control is a minimum of 44 px high. All functions work from the keyboard.

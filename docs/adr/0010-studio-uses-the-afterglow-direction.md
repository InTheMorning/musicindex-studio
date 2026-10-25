# ADR 0010: Studio Uses The Afterglow Visual Direction

- Status: Accepted
- Date: 2026-10-22
- Owner: operator
- Changed by: [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md). The editor page of a feed has a compact header, not the detail hero.
- Changed by: musicindex ADRs 0020 to 0029, on 2026-10-25 by operator decision. They own the interface rules that musicindex.org, Studio and v4vmm share. [Rules that musicindex replaces](#rules-that-musicindex-replaces) names each part that they replace in this record, ADR 0029 and ADR 0037.

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

## Rules That musicindex Replaces

The records are in `../musicindex/docs/adr/`. Their numbers are musicindex numbers, not Studio numbers. musicindex owns each rule below. A change to one needs a superseding record in `../musicindex`, not a change here (musicindex ADR 0020, Consequences).

| Part of a Studio record | Replaced by | What stays here |
| --- | --- | --- |
| This record, Context: musicindex ADRs apply only to `search.html`, and Studio adopts them only by its own decision | musicindex ADR 0020: ADRs 0020 to 0029 apply to Studio directly | musicindex ADR 0003 and ADR 0012 apply to Studio through Decision 1 |
| This record, Decision 1: the Figtree font for all text | musicindex ADR 0024 R-TYPE-1 item 4: the system font on an Apple device, and Figtree on each other system | Figtree on each system that is not an Apple device |
| This record, Decisions 1 and 2: the brand assets, the font and the tokens come from `search.html` | musicindex ADR 0022: `../musicindex-assets` holds the brand files, the fonts, the Lucide icons and `tokens/tokens.json`. Studio keeps copies and does not change them | musicindex ADR 0012 gives the entity colors |
| This record, Invariants: each control is a minimum of 44 px high | musicindex ADR 0024 R-PROFILE-1 and ADR 0029 R-SCALE-3: a mouse selects Desktop. Desktop has a default of 32 px and a minimum of 24 px. Touch has a default of 44 px and a minimum of 28 px (ADR 0024 R-TARGET-1) | All functions work from the keyboard |
| [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) item 5: the page name "Recipients" | musicindex ADR 0023 R-WORD-2: the page is "Value routes", with the groups "Recipients" and "Fees" | None. [ADR 0039](0039-the-shell-releases-and-the-outline.md) replaces the rest of item 5 |
| [ADR 0037](0037-button-roles.md) item 1: labels in sentence case, for example "Write to the host" and "Discard the draft" | musicindex ADR 0023 R-CASE-1: a button is title case, for example "Write to the Host" and "Discard the Draft" | The four roles, their look, and one primary button in each page or sheet |

The other rules of musicindex ADRs 0020 to 0029 add to the Studio records. They replace no part of them. ADR 0018 gives the error color that musicindex ADR 0020 R-TOKEN-1 uses as the danger color.

## Consequences

- Studio adds editor screens that the public site does not have. These screens use the same tokens.

## Invariants

- An entity color is not a general accent color.
- Red as an entity color means only "live now". Errors use their own token ([ADR 0018](0018-passphrase-strength-and-error-color.md)).
- Color is not the only indication of a type.
- All functions work from the keyboard. musicindex ADR 0029 R-SCALE-3 and ADR 0024 R-TARGET-1 give the size of each control.

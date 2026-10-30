# ADR 0089: The Left Panel Shows The Art Of Each Publisher Feed

- Status: Accepted
- Date: 2026-10-30
- Owner: operator
- Accepted: 2026-10-30 by the operator.
- Changes: [ADR 0085](0085-each-artist-and-label-is-a-destination.md) item 1 and [ADR 0088](0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) item 5, which give a row of a publisher feed a dot before its title.
- Uses: musicindex ADR 0038 R-DOT-3 (a dot on art), musicindex ADR 0021 R-ART-2 and R-ART-3 item 2 (the art of an artist or a label) and musicindex ADR 0032 R-DOT-1.

## Context

- The rows of the publisher feeds in the left panel showed a dot and a title, with no art. On 2026-10-30 the operator asked for the art of each feed, with the dot on the art.
- musicindex ADR 0021 R-ART-3 item 2 gives the art of an artist or a label: its own image, then a mosaic of the covers of its albums, then the placeholder. musicindex.org shows the mosaic in the logo shape. Studio had no mosaic.
- musicindex ADR 0038 R-DOT-3 gives the position of a dot on art for the three apps.

## Decision

1. Each row of a publisher feed in the left panel shows the art of the feed at 28 px, and then its title. This applies to the rows of level 0 and to the feeds below each level. It also applies to the root page and the pages of the levels at narrow width.
2. The art is the cover of the feed. A feed with no cover shows a mosaic of the different covers of its releases. One cover shows as it is. Two or more covers show as a mosaic of 2 by 2 in the logo shape, as on musicindex.org. A feed with no cover and no release cover shows the logo shape in the color of its first role.
3. The type dot is on the bottom right corner of the art, as R-DOT-3 says. The ring has the color of the left panel. On the current row, it has the color of the bottom of the row fill. The row shows no other dot.
4. The head of a level shows the art of item 2 at 40 px, with no dot. The type label gives the type.
5. The row "Not linked" has no art. A space of 28 px keeps its title in line with the other titles.

## Alternatives Considered

- The dot on the top left corner. Not chosen by the operator.
- No ring. Not chosen by the operator.

## Consequences

- The left panel loads the covers of the publisher feeds and of their releases through the `/images` address of the API.
- A row of 28 px art fits in the 32 px target of a mouse and the 44 px target of touch.

## Invariants

- `crates/api/ui-tests/shell.test.mjs` has the test "a publisher feed shows its cover, then the mosaic of its releases, then the logo shape". The position of the dot is a visual check.

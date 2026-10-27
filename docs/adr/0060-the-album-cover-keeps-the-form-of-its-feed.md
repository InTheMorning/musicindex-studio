# ADR 0060: The Album Cover Keeps The Form Of Its Feed

- Status: Accepted
- Date: 2026-10-27
- Owner: operator

## Context

On 2026-10-27 the operator opened the feed `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml` in the album editor. The field "Cover image address" was empty. The channel has an RSS `<image>` with a `<url>`, but no `itunes:image`. The same feed showed blank space in its description. [ADR 0061](0061-a-text-value-has-no-space-at-its-ends.md) decides that defect.

[ADR 0025](0025-studio-serves-cover-art.md) item 1 shows the `itunes:image` of the album. It does not name the RSS `<image>`. The core does not model `<image>`, so the tree keeps it and the editor does not show it ([ADR 0012](0012-lossless-tree-with-a-typed-model.md) item 4).

The DeMu feed template is the base form of a self-hosted music feed (`de-mu/demu-feed-template`, commit `4e2a1dc` of 2025-08-27). In its channel, the album cover is an RSS `<image>` with four children: `url`, `title`, `link` and `description`. The channel has no `itunes:image`. Each item has an `itunes:image` for the art of its track. The comment of that tag says: "If the tag below is not present, the channel level's image will be displayed instead".

MSP 2.0 follows the DeMu template (`src/utils/xmlParser.ts` and `src/utils/xmlGenerator.ts` at commit `09f3cf9`). It has one cover field. It reads `<image><url>` first and `itunes:image` second. On each save it writes the same address to both tags, and it writes the album cover into each item that has no art.

The two tags have different sources. RSS 2.0 defines `<image>` as a small channel logo of at most 144 × 400 px. Apple defines `itunes:image` as square cover art of 1400 to 3000 px. Only `itunes:image` can be on an item. Most podcast apps show `itunes:image` first. Many music feeds put the full cover in `<image><url>`.

## Decision

### The Cover Of A Channel

1. The core models the channel RSS `<image>` with its four children `url`, `title`, `link` and `description`. The editor changes only `url`. The writer keeps the other children as the source has them. The `url` is an address field of [ADR 0023](0023-web-addresses-and-audio-file-facts.md) item 1. The host operations of [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) item 7 change it as they change `itunes:image`.
2. The editor has one cover field for each channel. The field shows the `href` of the channel `itunes:image`. When the channel has no `itunes:image`, the field shows the `<url>` of the channel `<image>`.
3. An edit of the cover field keeps the form of the feed. The writer changes each of the two tags that the feed has:
   - A feed with only `<image>` keeps only `<image>`. The writer changes its `<url>` and adds no `itunes:image`. This is the DeMu form.
   - A feed with only `itunes:image` keeps only `itunes:image`. The writer adds no `<image>`.
   - A feed with both tags gets the new address in both.
   - A feed with neither tag gets an `itunes:image`, as before this ADR.
4. When both tags exist and their addresses are different, the editor shows the address of `<image><url>` below the field, with the action "Use this address". The writer changes no tag until the user edits the field or uses the action. After an edit or the action, both tags have the same address.
5. When the user clears the field, the writer removes each cover tag of the channel. It removes the full `<image>` element, because RSS 2.0 requires `url` in `<image>`.
6. A publisher feed and a playlist feed follow items 1 to 5 for their channel.

### The Cover Of An Item

7. The writer never writes the album cover into an item. An item with no `itunes:image` shows the album cover of item 2 in the editor. This replaces "the album image" in ADR 0025 item 1.

## Alternatives Considered

- Add a channel `itunes:image` when a feed has only `<image>`. Rejected by the operator on 2026-10-27: the feed then leaves the DeMu form.
- Read `<image><url>` first, as MSP 2.0 does. Rejected: most apps show `itunes:image`, so the field would then show a cover that most apps do not show.
- Write the same address to both tags on each save, as MSP 2.0 does. Rejected: a save with no edit then changes a feed with two different covers. ADR 0012 forbids a change to an element that the user did not change.
- Write the album cover into each item with no art, as MSP 2.0 does. Rejected: the feed then holds a value that its author did not write. The DeMu template says that an app shows the channel image for such an item.

## Consequences

- A feed made from the DeMu template shows its cover in the editor, and keeps its form after an edit.
- The diff of a cover edit in a feed with both tags shows two changed elements.
- ADR 0025 item 1 is amended: the cover of the album comes from item 2 of this ADR.

## Verification

Mechanical:

- A core test reads a copy of the DeMu `feed.xml` and of `feed-with-comments.xml` in `crates/core/tests/corpus`, with their entries in `manifest.json`. A read and a write with no edit give the same bytes.
- A core test reads a channel with only `<image>`. The cover is the address of `<image><url>`. An edit changes only `<url>`, and the document has no `itunes:image`.
- A core test reads a channel with only `itunes:image`. An edit changes only its `href`, and the document has no `<image>`.
- A core test reads a channel with both tags and the same address. An edit changes both.
- A core test reads a channel with both tags and different addresses. The model gives both addresses. A write with no edit gives the same bytes.
- A core test clears the cover of a channel with both tags. The document has no `<image>` and no `itunes:image`.
- A core test writes an item with no `itunes:image`. The item has no `itunes:image` after the write.
- The Node tests examine the UI model: the field value for each of the four forms of item 3, and the action "Use this address" only when the two addresses are different.

Visual:

- The album editor with the feed of the operator. The field "Cover image address" shows `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.png`, and the editor shows that image. In the dark theme, in the light theme and at 390 px.
- The album editor with a feed that has two different cover addresses. The second address and the action "Use this address" show below the field.

## Invariants

- The writer adds no cover tag that the feed did not have, apart from `itunes:image` in a feed with neither tag.
- The writer does not write a cover into an item.

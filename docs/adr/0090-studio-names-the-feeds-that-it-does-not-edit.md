# ADR 0090: Studio Names The Feeds That It Does Not Edit

- Status: Accepted
- Date: 2026-10-30
- Owner: operator
- Accepted: 2026-10-30 by the operator.
- Changes: [ADR 0085](0085-each-artist-and-label-is-a-destination.md) items 3 and 6, and [ADR 0088](0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) item 4.
- Uses: [ADR 0008](0008-feed-types-and-feed-identity.md) (the feed types), [ADR 0025](0025-studio-serves-cover-art.md) (the image limit of 10 MB).

## Context

- On 2026-10-30 the operator opened the feed "Into The Doerfel-Verse" and asked why it showed as it did. The feed has `podcast:medium` `podcast`. Studio edits albums, playlists and publisher feeds, so it showed only the facts. The page did not tell the cause.
- Its cover is a GIF of 13,138,651 bytes. The `/images` address of the API refuses an image above 10 MB with the code `too_large`. The page showed the logo shape, and it did not tell the cause. The hero of this page also did not show the cover of the feed.
- ADR 0085 item 6 says that Releases lists each album and each playlist. The list removed only the publisher feeds, so the podcast feed was in Releases and in "Not linked".

## Decision

1. The page of a feed that the editor does not open gives its type and the cause: "This feed has the type “podcast”. Studio edits albums, playlists, and artist and label feeds, so it shows only the facts of this feed."
2. Releases and "Not linked" list only albums and playlists. After "Not linked", the destination "Other feeds" lists each other feed that is not a publisher feed. It shows only when such a feed exists. A publisher feed with no role is in the heading "No role" of its level.
3. The hero of that page shows the cover of the feed. When the `/images` address refuses a cover with `too_large`, the head of the page says: "The cover is larger than 10 MB, so Studio does not show it." The box of the cover gives the same text as its title.

## Alternatives Considered

- A higher image limit. Not chosen: the limit of ADR 0025 protects the server shell.
- Refuse a feed of a different type when the user adds it. Not chosen by the operator.

## Consequences

- A podcast feed stays in the collection. The user opens it, reads it again or removes it from "Other feeds".

## Invariants

- `crates/api/ui-tests/shell.test.mjs` tests items 1 and 2. `crates/api/ui-tests/cover.test.mjs` tests the text of item 3.

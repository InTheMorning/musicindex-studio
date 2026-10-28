# ADR 0070: A Cover Opens The Artwork Sheet

- Status: Accepted
- Date: 2026-10-27
- Owner: operator
- Accepted: 2026-10-27 by the operator.
- Changes: [ADR 0064](0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) item 10, for the position of the list.

## Context

The editor shows a cover in the header of the album page, the playlist page, the publisher page and the track page. The track page also shows the header of its album. A cover does nothing when the user clicks it.

The artwork of one item is in two places:

- The album cover is a field of the album page. The "More artwork" list of the album (`podcast:image`) is on the page "Credits and links" (ADR 0064 item 10).
- The track image and the "More artwork" list of a track are in "More" on the track page.

On 2026-10-27 the operator said that a click on a cover must give the artwork settings. No shared musicindex rule gives a function to a cover. musicindex.org and v4vmm only show covers. musicindex ADR 0021 gives the look of art (R-ART-1 to R-ART-5). This ADR does not change it.

## Decision

### The Cover Is A Button

1. Each cover in the header of an editor page is a button. A click, Enter or Space opens the Artwork sheet of the item of that cover. On the track page, the album cover opens the sheet of the album, and the track cover opens the sheet of the track.
2. The accessible name of the button is "Artwork of" and the title of the item. Its tooltip is "Change the artwork".
3. The cover shows the hover style and the focus style that each other button has, and a pointer cursor. It gets no badge and no new marker.
4. The cover in the outline head does not open the sheet. It opens the first page of the feed ([ADR 0069](0069-the-outline-head-opens-the-feed-page.md)).

### The Artwork Sheet

5. The sheet has the title "Artwork" and the title of the item. It holds these parts in this sequence:
   1. A large preview of the image of the item, in the shape of musicindex ADR 0021 R-ART-4.
   2. The image field of the item, with its file picker ([ADR 0062](0062-a-file-picker-for-each-file-field.md)): the cover image address of a feed, or the track image address of a track.
   3. For a feed, the note about a different RSS image address, with "Use this address" ([ADR 0060](0060-the-album-cover-keeps-the-form-of-its-feed.md)).
   4. For an album and a track, the "More artwork" list of ADR 0064 item 10.
6. A track with no image of its own shows the album cover in the preview, with the text "Uses the album cover". When the track has its own image, the sheet has the button "Use the Album Cover". It removes the track image from the draft.
7. Each change in the sheet goes into the draft immediately, as a change of a field on a page does. The sheet has one button "Done", which closes it. Escape also closes it.

### The Artwork Row

8. The fields of item 5 are only in the sheet. The cover field, the track image field and the two "More artwork" lists leave their pages.
9. In their position, each page has one summary row "Artwork" ([ADR 0028](0028-summary-lines-and-one-open-entry.md)). The row opens the same sheet. Its summary gives the image and the number of other images, for example "Cover · 2 more images" or "Uses the album cover".
10. [ADR 0068](0068-the-sequence-of-the-sections-and-fields-of-an-editor-page.md) gives the position of the row on each page.

## Alternatives Considered

- A click that scrolls to the image field and focuses it. Not chosen by the operator: "More artwork" then stays on a different page.
- An "Artwork" page in the outline, with all the images of the release. Not chosen by the operator: the outline gets one more item.
- The image fields on the pages and also in the sheet. Not chosen by the operator: two places can then edit one field.
- A pencil badge on the cover. Not chosen: it is a new UI marker, and no shared musicindex ADR owns it.

## Consequences

- The page "Credits and links" loses "More artwork".
- "More" on the track page loses "Track image" and "More artwork".
- The sheet uses the same dialog as the other sheets (`useSheet` in `crates/api/ui/editor.js`).

## Invariants

- Each artwork field of an item is only in the Artwork sheet of that item.
- A change in the sheet goes only into the draft. No write occurs without a diff that the user saw ([ADR 0007](0007-draft-diff-write-and-announce.md)).

## Verification

Mechanical:

- A Node test examines the page model of ADR 0068. No page gives a cover field, a track image field or a "More artwork" list. Each page of item 9 gives the row "Artwork".
- A Node test examines the summary of item 9 from the artwork model. It uses a feed with no other image, a feed with two other images, and a track with no image.
- A Node test: "Use the Album Cover" gives a track with no image.

Visual:

- A click on each cover of the album page and the track page opens the sheet of the correct item.
- Tab reaches each cover, and Enter opens the sheet. The focus ring shows on the cover.
- The sheet shows its parts in the sequence of item 5 at desktop width and at 390 px.

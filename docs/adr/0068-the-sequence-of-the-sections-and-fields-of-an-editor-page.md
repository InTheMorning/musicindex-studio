# ADR 0068: The Sequence Of The Sections And Fields Of An Editor Page

- Status: Accepted
- Date: 2026-10-27
- Owner: operator
- Accepted: 2026-10-27 by the operator.
- Amended: 2026-10-28 by operator decision. [ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) item 15 replaces item 11 and the mechanical check. The sequences of items 2 to 9 are the start values of the model module. The module now owns the sequence, and the tier test of ADR 0071 item 6 is the guard.

## Context

[ADR 0028](0028-summary-lines-and-one-open-entry.md) item 5 says that the control that the user uses most comes first in an open group. musicindex ADR 0028 R-PAGE-1 gives the sequence of the sections of an item page. For the Studio editor, R-PAGE-1 names only the outline, and the outline follows it ([ADR 0039](0039-the-shell-releases-and-the-outline.md) item 6). No test examines the sequence of an editor page.

On 2026-10-27 the operator said that the sequence of an editor page has no logic. Rare settings come before the fields that the user expects. The agent compared the pages in `crates/api/ui/editor.js` with these rules:

- The album page shows the album fields, then Credits, Feed settings, Value routes and Feed facts, then Artists and labels. The tracks are last.
- On the album page, the playlist page and the publisher page, the cover comes after the description.
- The track page shows the publication date and the duration before the audio file. The duration comes from the audio file. The track image is in "More".

The Studio journeys in `../musicindex-research` (`research_notes/ui-design/journeys/studio/`) count the steps of each task. They give no data on how frequently a user changes each field. The operator selected the sequences below on 2026-10-27.

## Decision

### The Principle

1. A page shows the fields that identify the item first, then the content of the item, then its money and its credits. The settings that the user sets one time come next. The technical facts are last.

### The Album Page

2. The album page has these sections in this sequence:
   1. "Album": Title, Artist, Artwork, Description, Website, Hide from apps, Explicit. "Artwork" is the row of [ADR 0070](0070-a-cover-opens-the-artwork-sheet.md) item 9.
   2. "Tracks": each track, "Add a track" and "Import files".
   3. "Album details": the rows Value routes, then Credits and links.
   4. "Artists and labels".
   5. "Feed": the rows Feed settings, then Feed facts.
3. "Explicit" stays after "Hide from apps" in the first section, as [ADR 0064](0064-the-editor-edits-the-fields-that-msp-2-0-edits.md) item 7 says.
4. "Language" moves to the page "Feed settings". This changes ADR 0064 item 7 and [ADR 0053](0053-playlists-are-musicl-feeds-with-remote-items.md) item 3. The field is on the settings page, not on the first page.

### The Playlist Page And The Publisher Page

5. On the playlist page and the publisher page, the first section has the field sequence of the "Album" section of item 2. The second field has the name "Author" or "Name".
6. The publisher page has these sections in this sequence: the first section, "Releases", "Role", then a section with the rows Feed settings and Feed facts.
7. The playlist page has the first section, then a section with the rows Feed settings and Feed facts. Its tracks stay on their own page ([ADR 0053](0053-playlists-are-musicl-feeds-with-remote-items.md) item 4).

### The Feed Settings Page

8. The page "Feed settings" has these sections in this sequence: "Language", "Keywords", "Owner", "Lock against import", "Editors".

### The Track Page

9. The track page has these sections in this sequence:
   1. "Track": Title, the audio file with its duration and its alternative enclosures, Artwork, Description.
   2. "Credits".
   3. "Lyrics".
   4. The payments of the track ([ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) item 9).
   5. "More": Publication date, Explicit, Identity.
10. The duration field stays, and the user can change it. The publication date keeps the hint of [ADR 0024](0024-track-numbers-follow-the-track-sequence.md) item 5.

### The Check

11. A model module gives the sections and the fields of each page of this ADR, in their sequence. Each page shows its sections and fields from that list.
12. The outline does not change. It keeps the sequence of ADR 0039 item 6 and musicindex ADR 0028 R-PAGE-1.

## Alternatives Considered

- The tracks first on the album page, as R-PAGE-1 gives for an item page. Not chosen by the operator: the fields that identify the release come first in an editor.
- Keep the sections, and change only the field sequence in each group. Not chosen by the operator: the tracks then stay after the technical facts.
- The MSP 2.0 track editor: title, audio, duration, date, number, description, image, lyrics, explicit. Not chosen by the operator: it shows each field at one level, with no "More".
- Move "Explicit" and "Hide from apps" to "Feed settings" too. Not chosen by the operator: ADR 0063 and ADR 0064 item 7 keep them on the first page.
- A field sequence in the page code only, with no test. Rejected: the rule of ADR 0028 item 5 had no test, and the pages did not follow it.

## Consequences

- The album page, the playlist page, the publisher page, the settings page and the track page change their sequence.
- The page code reads its sections and fields from a model module ([ADR 0042](0042-ui-models-with-node-tests.md)).
- The playlist page and the publisher page get "Language" only on "Feed settings".

## Invariants

- The sections of each page show in the sequence of the model module, and their tiers do not go down (ADR 0071).
- The outline keeps its sequence.

## Verification

Mechanical:

- The tier test of ADR 0071 item 6 replaces the comparison with the lists of items 2 and 5 to 9.

Visual:

- The album page, the publisher page, the playlist page, the settings page and a track page show the sequence of this ADR. Examine them at desktop width and at 390 px. The `$edges()` check of `tools/drive-ui.mjs` finds no content at the edge of a group.

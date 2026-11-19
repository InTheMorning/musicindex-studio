# ADR 0212: The Add Pages

- Status: Accepted
- Date: 2026-11-14
- Owner: operator
- Accepted: 2026-11-14. The operator accepted the consolidation of each changed decision (musicindex-research `df7b823`), and selected the consolidation of ADR 0073 through the questionnaire. This ADR makes no new decision. It restates each live rule of the ADR that it supersedes, with each change from a subsequent ADR.
- Supersedes: [ADR 0073](../archive/adr/0073-the-add-pages-ask-one-question-at-a-time.md), The Add Pages Ask One Question At A Time.
- Uses: [ADR 0023](0023-web-addresses-and-audio-file-facts.md) for the facts of an audio address. [ADR 0028](0028-summary-lines-and-one-open-entry.md) for a closed row with a summary. [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) for the groups and the rows. [ADR 0042](0042-ui-models-with-node-tests.md) for the model modules. [ADR 0065](0065-the-core-applies-each-feed-rule.md) for the feed rules.
- Uses: these current ADRs own a part of this area. This ADR names each one at its item. [ADR 0077](0077-a-new-album-can-start-from-an-album.md) (the menu "Start from"), [ADR 0092](0092-file-hosts-and-the-destination-icons.md) (the words "file host"), [ADR 0221](0221-the-welcome-page-and-the-first-steps.md) (the Welcome page and the first steps), [ADR 0130](0130-the-facts-of-an-audio-file-stay-locked.md) (the row "Properties") and [ADR 0164](0164-the-releases-of-a-publisher-feed-have-an-add-row.md) (the add row of a publisher feed and the title of the sheet).
- Changed by: [ADR 0213](0213-a-release-starts-on-the-page-of-its-artist.md) items 2 and 3: the page "Add a Feed" and the menu "Type" have no album.

## Context

- ADR 0073 replaced seven forms on the Add feeds page with two groups of rows. It gave one form to make each new feed, and it gave the outline one row "Add tracks".
- ADR 0077, ADR 0119 and ADR 0130 changed ADR 0073. ADR 0073 had three "Changed by" lines. Global `CLAUDE.md` §Consolidate A Changed Decision permits three, and a subsequent change needs this consolidation first.
- ADR 0092 changed the word "host" to "file host" in each UI text. ADR 0164 gave the sheet of the form its title.
- The current ADRs in the Uses lines own their rules. This ADR restates the result at each item, and names the owner.
- The archived ADR 0073 keeps the measurements of 2026-10-28 and the alternatives that the operator did not select.

## Decision

### The Add Feeds Page

1. The Add feeds page holds two groups of rows, and no form:
    - "Make a new feed": Album, Artist or label, Show, Playlist.
    - "Add a feed you have": By address or name, From a file host folder, From MSP 2.0 backups ([ADR 0092](0092-file-hosts-and-the-destination-icons.md) item 2).
2. Each row shows its title, one line of text, and `›`. A group holds rows, as [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) item 11 says.
3. A row opens a sheet with the form of that row. ADR 0073 did not change the forms of "Add a feed you have", and this ADR does not change them.
4. With no connected file host, "Make a new feed" shows a hint and the button "Connect a File Host", and not its rows. The row "From a file host folder" does not show.
5. A user with no feed starts on the Welcome page, not on this page ([ADR 0221](0221-the-welcome-page-and-the-first-steps.md)). The rows of this page use the word "feed", and "Artist or label" is one row (ADR 0221 item 10).

### One Form Makes A New Feed

6. One form makes each new feed. Its first field is "Type", a select menu with Album, Artist or label, Show and Playlist. The row of item 1 sets the start value of "Type".
7. The form shows the fields of its type:
    - Album: "Album title" and "Artist".
    - Artist or label: "Name" and "Role".
    - Show: "Show feed title" and "Artist".
    - Playlist: "Playlist name" and "Author".
8. When "Type" is Album and the collection has an album, the form shows the select menu "Start from", after "Type". The first value is "Empty album" ([ADR 0077](0077-a-new-album-can-start-from-an-album.md) items 1 to 3). ADR 0077 owns the copy.
9. "File host" shows only when more than one file host is connected.
10. "Feed file" gets a start value from the title. Studio writes the title in lower case and removes the accents. Each sequence of characters other than `a` to `z` and `0` to `9` becomes one hyphen, with no hyphen at the two ends. Then `.xml` follows.
11. The form shows "Feed file" as a closed row, with the start value as the summary ([ADR 0028](0028-summary-lines-and-one-open-entry.md)). The row opens to show the field. After the user changes the field, the title does not change it again.
12. The core checks the path. When the file exists, the row opens and shows the field with the error.
13. The start value of item 10 is an edit aid of [ADR 0065](0065-the-core-applies-each-feed-rule.md). It is not a feed rule. A model module gives it, and a Node test examines it.
14. The title of the sheet names the page that the form makes, and the add row of a publisher feed opens the same form ([ADR 0164](0164-the-releases-of-a-publisher-feed-have-an-add-row.md) items 1 to 3 and 7). ADR 0164 owns these rules.

### Add Tracks

15. The outline and the Tracks section have one row "Add tracks".
16. "Add tracks" opens the sheet "Add tracks". Its first control is "＋ Choose Music and Image Files". The import of the files does not change.
17. While the sheet has no file, it shows the link "The audio is already online". The link opens the sheet "Add a track from an address", with the fields Title and Audio file address. That sheet has the link "Choose Files from This Computer".
18. The facts of the audio file are in a closed row "Properties", as on the track page. [ADR 0130](0130-the-facts-of-an-audio-file-stay-locked.md) owns the row:
    - Its summary is the summary of the track page: the size, the type and the duration, for example "3.8 MB · MP3 · 2:57". A file with no duration field gives the size and the type (ADR 0130 item 1).
    - An open "Properties" shows each value as text, then the button "Advanced" (ADR 0130 item 2).
    - The row opens when Studio does not find a fact (ADR 0130 item 4).
    - The user changes a value only after "Advanced", its confirmation sheet and "Edit Values" (ADR 0130 item 5).

## The Item Map

| ADR 0073 | This ADR |
| --- | --- |
| Item 1 | Item 1 |
| Item 2 | Item 2 |
| Item 3 | Item 3 |
| Item 4 | Item 4 |
| Item 5 | Item 6 |
| Item 6 | Item 7 |
| Item 7 | Item 9 |
| Item 8 | Items 10 and 11 |
| Item 9 | Item 12 |
| Item 10 | Item 13 |
| Item 11 | Item 15 |
| Item 12 | Item 16 |
| Item 13 | Item 17 |
| Item 14 | Item 18 |

Items 5, 8 and 14 restate the rules of ADR 0119, ADR 0077 and ADR 0164 for this area.

## Consequences

- An agent reads one current ADR for the add pages.
- ADR 0073 is in the archive. It keeps its number, its file and its content.

## Invariants

Situational, until a decision replaces these pages:

- The Add feeds page holds rows, and no form.
- One form makes each new feed.
- The outline has one row to add tracks.

## Non-Goals

- No change to the import of files, to the search of feeds, or to the MSP 2.0 import.
- No change to the route `/feeds/new` or to `/media/facts`.
- No change to "Add a live show" or to the "Add a track" row of a playlist.

## How This Decision Is Checked

Mechanical, in the model modules of [ADR 0042](0042-ui-models-with-node-tests.md):

- The model of the Add feeds page gives the two groups of item 1. With no file host, it leaves out the rows of item 4.
- The model of the form gives the fields of item 7 for each type.
- The start value of "Feed file" follows item 10.
- The outline model gives one row with the action `add-tracks`, and no row `add-track` or `import`.

Visual. The screenshot checks of task 049 passed for ADR 0073. This ADR adds no rule, so it adds no check.

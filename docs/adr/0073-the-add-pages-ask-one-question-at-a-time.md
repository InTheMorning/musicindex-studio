# ADR 0073: The Add Pages Ask One Question At A Time

- Status: Accepted
- Date: 2026-10-28
- Owner: operator
- Accepted: 2026-10-28 by the operator.
- Changes: [ADR 0023](0023-web-addresses-and-audio-file-facts.md) item 6, [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) item 8, [ADR 0039](0039-the-shell-releases-and-the-outline.md) item 6 and [ADR 0068](0068-the-sequence-of-the-sections-and-fields-of-an-editor-page.md) item 2.
- Amended: 2026-10-28 by operator decision. [ADR 0077](0077-a-new-album-can-start-from-an-album.md) adds the select menu "Start from" to the form of items 5 and 6, for the type Album.

## Context

On 2026-10-28 the operator said that the pages to add a feed and to add a track are too much for a user. The planning model examined them at 1100 px with `tools/drive-ui.mjs`:

- The Add feeds page has seven forms, one after the other. The page is 2942 px high.
- Four forms make a new feed: a release, an artist feed or a label feed, a show feed and a playlist. Three forms add a feed that exists: by address or name, from a connected folder, and from MSP 2.0 backup files. The user reads all seven forms to find one.
- Each form that makes a feed asks for "Feed file" before the title. The file name is a technical fact.
- The outline and the Tracks section have two rows: "Add a track" and "Import files". "Add a track" opens a sheet that asks for the size in bytes and the media type. "Import files" reads these facts from the file.
- The core finds the size and the type of an audio address ([ADR 0023](0023-web-addresses-and-audio-file-facts.md) item 5, the route `/media/facts`). The sheet shows the two fields before the user gives the address.

The operator selected each decision of this ADR from a questionnaire.

## Decision

### The Add Feeds Page

1. The Add feeds page holds two groups of rows, and no form:
   - "Make a new feed": Album, Artist or label, Show, Playlist.
   - "Add a feed you have": By address or name, From a host folder, From MSP 2.0 backups.
2. Each row shows its title, one line of text, and `›`. A group holds rows, as [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) item 11 says.
3. A row opens a sheet with the form of that row. The forms of "Add a feed you have" do not change.
4. With no connected host, "Make a new feed" shows the hint and the button "Connect a Host", and not its rows. The row "From a host folder" does not show.

### One Form Makes A New Feed

5. One form makes each new feed. Its first field is "Type", a select menu with Album, Artist or label, Show and Playlist. The row of item 1 sets the start value of "Type".
6. The form shows the fields of its type:
   - Album: "Album title" and "Artist".
   - Artist or label: "Name" and "Role".
   - Show: "Show feed title" and "Artist".
   - Playlist: "Playlist name" and "Author".
7. "Host" shows only when more than one host is connected.
8. "Feed file" gets a start value from the title. Studio writes the title in lower case and removes the accents. Each sequence of characters other than `a` to `z` and `0` to `9` becomes one hyphen, with no hyphen at the two ends. Then `.xml` follows.
   - The form shows it in a closed row "Feed file", with the start value as the summary ([ADR 0028](0028-summary-lines-and-one-open-entry.md)). The row opens to show the field. After the user changes the field, the title does not change it again.
9. The core checks the path, as before. When the file exists, the row opens and shows the field with the error.
10. The start value of item 8 is an edit aid of [ADR 0065](0065-the-core-applies-each-feed-rule.md). It is not a feed rule. A model module gives it, and a Node test examines it.

### Add Tracks

11. The outline and the Tracks section have one row "Add tracks". It replaces "Add a track" and "Import files".
12. "Add tracks" opens the sheet "Add tracks". Its first control is "Choose music and image files". The import of the files does not change.
13. While the sheet has no file, it shows the link "The audio is already online". The link opens the sheet "Add a track from an address", with the fields Title and Audio file address. That sheet has the link "Choose files from this computer".
14. The size and the type are in a closed row "Properties", as on the track page. Its summary is the summary of the track page, for example "3.8 MB · MP3". The row opens when Studio does not find the two facts. The user can open it and change the two facts, as ADR 0023 item 6 says. This changes how ADR 0023 item 6 shows them.

## Alternatives Considered

- An accordion: each row opens its form below it, one at a time ([ADR 0028](0028-summary-lines-and-one-open-entry.md)). Not selected: on a phone, an open form pushes the other rows down.
- Keep the seven forms, and only hide the technical fields. Not selected: the user continues to read seven forms.
- Keep four forms to make a feed. Not selected: the four forms repeat the same fields.
- Show "Host" and "Feed file" after the title. Not selected: the user continues to see a file name that Studio can give.
- A new core request for the size and the type. Not necessary: the core has it (ADR 0023 item 5).

## Consequences

- The Add feeds page fits in one window at desktop width.
- The left panel and the album page lose one add row.
- The user guide pages about new feeds and about tracks change.
- A Node test examines the start value of "Feed file" and the fields of each type.

## Invariants

Situational, until a decision replaces these pages:

- The Add feeds page holds rows, and no form.
- One form makes each new feed.
- The outline has one row to add tracks.

## Non-Goals

- No change to the import of files, to the search of feeds, or to the MSP 2.0 import.
- No change to the route `/feeds/new` or to `/media/facts`.
- No change to "Add a live show" or to the "Add a track" row of a playlist.

## Verification

Mechanical, in the model modules of [ADR 0042](0042-ui-models-with-node-tests.md):

- The model of the Add feeds page gives the two groups of item 1, and leaves out the rows of item 4 with no host.
- The model of the form gives the fields of item 6 for each type.
- The start value of "Feed file" follows item 8.
- The outline model gives one row with the action `add-tracks`, and no row `add-track` or `import`.

Visual:

- At desktop width and at 390 px, in the two themes, the Add feeds page shows the rows only. Each row opens its sheet.
- The form to make a feed shows Type, the two fields of the type, and the line of the feed file.
- "Add tracks" opens the file sheet. The link opens the address sheet, and the link of that sheet opens the file sheet.
- `$edges()` of `tools/drive-ui.mjs` finds no content at the edge of a group.

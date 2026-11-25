# ADR 0258: The Shell And The Editor Pages

- Status: Accepted
- Date: 2026-11-25
- Owner: operator
- Accepted: 2026-11-25. The global decision on ADR overuse (musicindex-research `df7b823`) sets one current ADR for an area after three changes. This ADR makes no new decision. It restates each live rule of the ADRs that it supersedes, with each subsequent change.
- Supersedes: [ADR 0206](../archive/adr/0206-the-shell-and-the-editor-pages.md), The Shell And The Editor Pages.
- Changed by: [ADR 0263](0263-overview-is-the-start-page-and-each-page-keeps-the-sequence-of-its-feed.md) items 1 to 4, 13 and 14, on 2026-11-25. Overview replaces Releases in the panel and as the start page. Overview has no sort and no view, and item 7 gives the full row (items 1 to 8). The narrow stack starts on Overview, and the button "‹ Overview" of a feed page has the name "‹ Sections" (item 18).
- Uses: musicindex ADR 0080 R-SHELL-1, R-NAV-2 and R-LIST-1. musicindex ADR 0028 R-PAGE-1, musicindex ADR 0080 R-SHELL-2 and musicindex ADR 0080 R-NAV-3. [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) for the editor pages and the bar. [ADR 0042](0042-ui-models-with-node-tests.md) for the model modules.
- Uses: [ADR 0069](0069-the-outline-head-opens-the-feed-page.md) (the outline head), [ADR 0070](0070-a-cover-opens-the-artwork-sheet.md) (the Artwork sheet), [ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) (Settings and the tiers), [ADR 0072](0072-add-a-feed-opens-from-releases.md) (Add a feed).
- Uses: [ADR 0212](0212-the-add-pages.md) (Add tracks), [ADR 0074](0074-nostr-keys-for-artists-labels-and-contributors.md) (the Nostr keys), [ADR 0076](0076-an-op3-switch-on-feed-settings.md) (OP3), [ADR 0080](0080-the-season-of-a-track-is-an-experimental-disc-setting.md) (the disc).
- Uses: [ADR 0085](../archive/adr/0085-each-artist-and-label-is-a-destination.md) (the earlier publisher destinations), [ADR 0254](0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) (the Catalog), [ADR 0090](0090-studio-names-the-feeds-that-it-does-not-edit.md) (Other feeds), [ADR 0092](0092-file-hosts-and-the-destination-icons.md) (File hosts and the icons).
- Uses: [ADR 0112](0112-a-credit-role-comes-from-a-list.md) (the credit roles), [ADR 0260](0260-the-welcome-page-and-the-first-steps.md) (the Welcome page), [ADR 0130](0130-the-facts-of-an-audio-file-stay-locked.md) (the audio facts), [ADR 0239](0239-the-panel-holds-the-content-and-the-page-has-tabs.md) (the tabs).
- Uses: [ADR 0155](0155-the-sort-menu-names-both-directions.md) (the sort directions), [ADR 0203](0203-people.md) (People).

## Context

- ADR 0206 consolidated the shell, the MSP 2.0 fields and the start sequences of the editor pages.
- musicindex ADR 0080 changed items 10 to 12. ADR 0223 changed the narrow layout, and ADR 0254 changed the Catalog and the outline.
- ADR 0226 and ADR 0228 changed the narrow layout of ADR 0223. This ADR keeps those changes.
- `crates/api/ui/pages.js` owns the current page sequences. Items 31 to 38 keep the recorded start values and do not override that module.
- The archived ADR 0206 keeps its earlier context, alternatives and item map. The Uses lines keep the owners of the other rules.

## Decision

### The Destinations

1. With no feed open, the left panel holds the destinations:
   - The top group holds Releases, then the tree of the Catalog, then "No artist yet", then "Other feeds" ([ADR 0254](0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) items 2 and 3). "No artist yet" shows only when a release has no link to an artist page of the collection. "Other feeds" shows only when a feed that is not a release or a publisher feed exists ([ADR 0090](0090-studio-names-the-feeds-that-it-does-not-edit.md) item 2).
   - Add a feed opens from a button on Releases ([ADR 0072](0072-add-a-feed-opens-from-releases.md) items 1 and 2).
   - The bottom group holds People, File hosts, Settings and Lock Studio, in this sequence ([ADR 0203](0203-people.md) item 36, [ADR 0092](0092-file-hosts-and-the-destination-icons.md) item 1, [ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) item 9). On the desktop with the list of collections, the collection row follows Settings ([ADR 0242](0242-the-collection-row-of-the-footer-opens-switch-collection.md) items 1 and 6). On a server, the row "Sign Out" follows "Lock Studio" or "Unlock Studio" ([ADR 0241](0241-sign-out-and-switch-collection-are-rows-of-the-panel-footer.md) items 1 and 8). The bottom group stays at the bottom of the panel at each height of the window.
   - On a server with a locked vault, "Unlock Studio" is in the place of "Lock Studio". It opens the unlock sheet of [ADR 0056](0056-server-sign-in-methods-and-vault-wrappers.md).
   - Each fixed destination shows its icon, and no destination row shows a type dot (ADR 0092 items 5 and 9).
2. Releases is a list page at the full width of the content area. Each row shows the cover, the type label, the host and the draft state, for example "Saved, not written". Releases lists each album and each playlist. A publisher feed opens from its row in the Catalog, not from a row of Releases (ADR 0254 item 2).
3. Releases has the sort and the view of musicindex ADR 0080 R-LIST-1:
   - The sorts are "Newest", the default, and "A to Z" with a letter bar. "Newest" gives the feed that joined the collection last first. The menu also names the reversed direction of each sort: "Oldest" and "Z to A" ([ADR 0155](0155-the-sort-menu-names-both-directions.md) item 1).
   - The views are rows, the default, and tiles.
   - A sort, a direction or a view that is not the default is in the query of the address, as `sort=az`, `reversed=1` and `view=tiles` (ADR 0155 item 4).
4. Releases is the start page. A user with no feed in the collection sees the page "Welcome to Studio" in the place of the empty Releases page ([ADR 0260](0260-the-welcome-page-and-the-first-steps.md) item 1).
5. No page is a placeholder. The text "Select a feed" does not show.

### The Outline Of An Open Feed

6. While a release or a playlist is open, the left panel shows its outline in place of the Catalog tree. The bottom group stays at the bottom (item 1). The left panel is one panel at each time. A publisher feed has no outline (ADR 0254 item 7).

7. The outline of an album starts with "‹ Catalog" and the head of the feed (ADR 0254 item 7). Then it shows each track by its number and title, and "Add tracks" (ADR 0239 item 1, ADR 0212 item 15). The head opens the first tab of the feed. The outline has no row for the first page (ADR 0069 items 1 and 2, ADR 0239 item 3). The outline of a playlist holds only its head after "‹ Catalog" (ADR 0239 item 4, ADR 0254 item 7). Value routes, credits, files and the settings of a feed are tabs of its page (ADR 0239 item 2).

8. The first row of the outline is "‹ Catalog". It shows the tree again, with the same open rows (ADR 0254 item 7). The back link to a level goes.

9. Each item of the outline opens one page of the editor. [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) items 7 to 9 stay.

### Back

10. The editor bar has the back control of musicindex ADR 0080 R-NAV-2 at its leading edge. In a narrow window, it follows the home button of item 11. The controls are the icons `‹ ›` on Desktop, and the circular icon button `‹` named "Back" on Touch (musicindex ADR 0080 R-NAV-3).

### Narrow Width

11. At 760 px or less, Studio is one stack (musicindex ADR 0080 R-SHELL-2). It has no menu button and no drawer. musicindex ADR 0070 gives the website the same stack. On each page other than the root page, the bar starts with the home button, then the back control of item 10. Forward with a mouse and the title button follow (R-SHELL-2 item 6). The home button is the musicindex mark, and its accessible name is "Destinations" (R-TITLE-2). 

    The left edge of the mark is on the content margin, and its target extends into the margin (operator selection of 2026-11-16, relayed by the orchestrator session). The title button is the title area.

    The two buttons add the root page as a new history entry after the current entry, with `history.pushState` (R-TITLE-1 item 4). Back on that root page opens the page that the user left, and Forward has no page after the action. 

    The root page and a wide window have no home button, and the back control is the first control of the bar. Studio shows no edge indicator.
12. At 760 px or less, the bar of a feed has two rows. The first row has the home button, the back control and the title button. The second row has the draft status, then "Versions" and "Review". No part of the bar is hidden.
13. The root page "Studio" shows the destinations of item 1. It has no back control. Releases stays the start page (item 4), and the root page is below it in the stack. People, File hosts and Settings stay at the bottom, in the two layouts (ADR 0226 items 7 and 8). The root page ends with the collection row, then "Lock Studio" or "Unlock Studio", then "Sign Out" on a server (ADR 0242 item 3). The root page has the color of the left panel, and its bar has the color of the other bars.

14. At narrow width, a tap on a release or a feed opens its feed page (ADR 0223 item 8). The button "‹ Overview" is at the left of the select menu of the tabs. It opens the outline page (ADR 0228 items 11 and 12). 

    The stack is Releases, the feed page, the outline page and a part. Back goes one level down in that stack. The outline page has the color of the left panel. A publisher feed keeps the tree page and has no outline (ADR 0254 items 7 and 10).

### Value Routes

15. The Value routes page is at most 560 px wide. A share segment and a percentage show a tooltip immediately, with the name, the percentage and the raw share (musicindex ADR 0080 R-VALUE-1 item 3).

### The MSP 2.0 Fields In The Core

16. The core models these elements: `itunes:explicit` of the channel and of each item, `itunes:owner` with `itunes:name` and `itunes:email`, `itunes:keywords`, `managingEditor`, `webMaster`, and `podcast:season` with its `name`. A read and a write with no edit give the same bytes.
17. Each new element takes its hosted position ([ADR 0031](0031-a-new-element-takes-its-hosted-position.md)).
18. Studio writes no field that the user did not set. A feed with no `itunes:explicit` gets none from Studio. A new track gets no `podcast:season`.
19. A switch keeps the form of the feed. The UI sends `true` or `false`. At the draft save, the core compares the sent value with the draft value. It does the same for the GUIDs and the track numbers. When the draft has `yes` or `no`, the core writes `yes` or `no`, with the same capital letter. In each other condition, the core writes `true` or `false`, as Apple specifies.
20. Studio keeps a text value as the user types it, with no space at its ends ([ADR 0061](0061-a-text-value-has-no-space-at-its-ends.md)). Studio does not refuse a `managingEditor`, a `webMaster` or an owner email that is not an email address.

### The Channel Fields

21. The album page, the playlist page and the publisher page get these fields, as [ADR 0063](0063-one-switch-hides-a-feed-from-apps.md) item 3 does for its switch:
    - "Explicit": one switch for `itunes:explicit`.
    - "Owner": the owner name and the owner email.
    - "Keywords": one text field.
    - "Lock against import": one switch for `podcast:locked`, and the owner email of the lock.
    - "Managing editor" and "Webmaster": one text field each.
22. "Explicit" is in the first section of the page, after "Hide from apps". Apple requires this tag. The other fields of item 21 are on the page "Feed settings", with "Language" (item 34). The Settings tab of the feed opens that page (ADR 0239 items 2 and 7). The user does not change these fields in each visit ([ADR 0028](0028-summary-lines-and-one-open-entry.md)). The page "Feed settings" of an album also has the switch "OP3 statistics" ([ADR 0076](0076-an-op3-switch-on-feed-settings.md) item 5).

### The Track Fields

23. The track page gets these fields:
    - "Explicit": one switch for the item `itunes:explicit`.
    - "Credits": a selection between "Use the album credits" and "Credits for this track". The second selection shows the People control of the album. When the user selects it, the list starts with a copy of the album credits, because the item list replaces the channel list.

      The copy keeps the Nostr key of each person ([ADR 0074](0074-nostr-keys-for-artists-labels-and-contributors.md) item 26).

      A change back to "Use the album credits" removes each item `podcast:person`, after a confirmation. The role of a credit is a select menu from the role list, then "Other role…" ([ADR 0112](0112-a-credit-role-comes-from-a-list.md) items 2 and 3).
    - "Lyrics": a list of `podcast:transcript` entries. Each entry has the address, the type, the language and a "Captions" check box for `rel="captions"`. The address field uses the file picker ([ADR 0062](0062-a-file-picker-for-each-file-field.md)).
    - "Season": a number field and a name field, in the "Identity" row. They show as the experimental entry "Disc", with the fields "Disc number" and "Disc name". The fields write `podcast:season` and its `name` ([ADR 0080](0080-the-season-of-a-track-is-an-experimental-disc-setting.md) items 1 to 4).
24. The file picker gets a third kind of file: a lyrics file. The extensions are `.srt`, `.vtt`, `.txt`, `.json` and `.html`. The three sources of ADR 0062 item 2 and the drop of ADR 0062 item 20 accept this kind in the lyrics field. An upload of a lyrics file goes to the release folder, as an image does. This extends ADR 0062 items 1 and 3.

### More Artwork

25. The album and each track have a list of `podcast:image` entries. Each entry has the address, the description for accessibility (`alt`), the `purpose` tokens and the dimensions. The address field uses the file picker. The list is in the Artwork sheet of the item, not on its page ([ADR 0070](0070-a-cover-opens-the-artwork-sheet.md) items 5 and 8).
26. When the user selects an image, Studio reads its width, its height and its type from the file. Studio gives the ratio, and selects a near ratio from a list, such as `16/9`. The user can change each value.
27. The `purpose` field offers the tokens of the specification as check boxes: `artwork`, `social`, `canvas`, `banner`, `publisher`, `circular` and `poster`. Studio keeps each other token of the feed.

### Not In This Decision

28. `itunes:category`: deferred by the operator on 2026-10-27.
29. The fields of a live item.

### The Principle Of A Page

30. A page shows the fields that identify the item first, then the content of the item, then its money and its credits. The settings that the user sets one time come next. The technical facts are last. The tiers of ADR 0071 item 1 apply this principle to each section.

### The Start Values Of The Page Sequences

31. The start values of the album page were these sections in this sequence. ADR 0239 item 7 changes them: the first page has no row for Value Routes, Credits or Settings, and it keeps the Feed facts. `pages.js` owns the current sequence (item 40).
    1. "Album": Title, Artist, Artwork, Description, Website, Hide from apps, Explicit. "Artwork" is the row of ADR 0070 item 9.
    2. "Tracks": each track, then "Add tracks" (ADR 0212 item 15).
    3. "Album details": the rows Value routes, then Credits and links.
    4. "Artists and labels".
    5. "Feed": the rows Feed settings, then Feed facts.
32. "Explicit" stays after "Hide from apps" in the first section (item 22).
33. On the playlist page and the publisher page, the first section has the field sequence of the "Album" section of item 31. The second field has the name "Author" or "Name".
34. "Language" is on the page "Feed settings", not on the first page. This applies to the album, the playlist and the publisher feed.
35. The start values of the publisher page were the first section, "Releases", "Role", then a section with the rows Feed settings and Feed facts.
36. The start values of the playlist page were the first section, then a section with the rows Feed settings and Feed facts. Its tracks stay on their own tab ([ADR 0053](0053-playlists-are-musicl-feeds-with-remote-items.md) item 4, ADR 0239 item 2).
37. The start values of the page "Feed settings" were these sections in this sequence: "Language", "Keywords", "Owner", "Lock against import", "Editors".
38. The start values of the track page were these sections in this sequence:
    1. "Track": Title, the audio file with its duration and its alternative enclosures, Artwork, Description.
    2. "Credits".
    3. "Lyrics".
    4. The payments of the track (ADR 0029 item 9).
    5. "More": Publication date, Explicit, Identity.
39. The duration is in "Properties" of the audio file. It is text until the user selects "Advanced", and then the user can change it (ADR 0130 items 1, 2 and 5). The publication date keeps the hint of [ADR 0024](0024-track-numbers-follow-the-track-sequence.md) item 5.

### The Module And The Check

40. `crates/api/ui/pages.js` gives the sections of each page, in their sequence, each with its tier. Each page shows its sections from that list. The module owns the sequence, and the tier test of ADR 0071 item 6 is the guard (ADR 0071 items 5 and 15).
41. The outline keeps the sequence of item 7 and musicindex ADR 0028 R-PAGE-1. A page sequence does not change the outline.

## Consequences

- The Catalog tree shows the publisher feeds. An open release or playlist shows its outline (ADR 0254 item 7).
- The draft status, "Versions" and "Review" stay in the editor bar (ADR 0029 items 1 to 4).
- An MSP 2.0 user can change each field that MSP 2.0 edits. The categories are the only field that stays with no control.
- A feed with no `itunes:explicit` stays with no tag until the user sets the switch. Apple requires the tag, but Studio does not add it without the user.
- The page code reads its sections and fields from the model module (ADR 0042).
- ADR 0223 items 5 and 6 give the release search that was open as O-9.

## Non-Goals

- `itunes:category` (item 28).
- The fields of a live item (item 29).

## Invariants

Situational, until a decision replaces this layout:

- The left panel holds the Catalog, or the outline of an open release or playlist. The bottom group always shows at its bottom. On a server with a locked vault, "Unlock Studio" is in the place of "Lock Studio". At 760 px or less, the root page and the outline page hold the same content.
- Studio writes no field that the user did not set.
- A switch keeps the form of the value in the feed.

Durable:

- The sections of each page show in the sequence of the model module, and their tiers do not go down (ADR 0071).

## How This Decision Is Checked

Mechanical. The tests of the superseded ADRs enforce these items, and this ADR adds no test:

- The model modules of ADR 0042, with the Node tests in `crates/api/ui-tests/` (`shell.test.mjs`, `destinations.test.mjs`, `pages.test.mjs`, `explicit.test.mjs`, `credits.test.mjs`, `artwork.test.mjs`, `picker.test.mjs`):
  - With no feed open, the model of the left panel gives the destinations of item 1, and no text "Select a feed".
  - With a feed open, the outline model gives its items in the sequence of item 7.
  - The Catalog link shows the tree again, with the same open rows (item 8, ADR 0254 item 7).
  - The Releases model gives the rows in the sequence of each sort, and the letter groups of "A to Z".
  - The navigation model disables Forward when no subsequent page exists (musicindex ADR 0080 R-NAV-2 item 4).
  - At narrow width, the navigation model gives the sequence of item 14, and the root page below Releases (item 13).
  - The tier test of ADR 0071 item 6 examines each page of `pages.js`.
  - The model module of the credits selection, the `purpose` tokens and the near ratio.
- The core tests:
  - The corpus tests: each feed gives the same bytes after a read and a write with no edit.
  - A test for each new element of item 16: an edit changes only that element, and a new element takes its hosted position.
  - A test saves a sent `true` over a draft `no`, and over a draft `false`. The draft then holds `yes` and `true`. A sent `false` over a draft `Yes` gives `No`.
  - A test gives the kind of each lyrics extension, in lower case and in upper case.
  - A test sets "Credits for this track", then sets "Use the album credits" again. The item has no `podcast:person`.
  - A test keeps an unknown `purpose` token after an edit of the image.
- An API test lists the lyrics files of a release and of a test host. Each list holds only lyrics files.

Visual: no new visual check. The archived ADR 0039, ADR 0064 and ADR 0068 keep their visual checks.

## Item Map

| Superseded item | New item | Source of the change |
| --- | --- | --- |
| ADR 0206 item 1 | 1 | ADR 0241 items 1 and 8 and ADR 0242 items 1 and 6: the collection row and "Sign Out". |
| ADR 0206 items 2 to 5 | 2 to 5 | None. |
| ADR 0206 items 6 to 8 | 6 to 8 | ADR 0254 item 7: Catalog, with no publisher outline or back link to a level. |
| ADR 0206 item 9 | 9 | None. |
| ADR 0206 items 10 to 12 | 10 to 12 | musicindex ADR 0080 R-SHELL-2 item 6, R-TITLE-1 and R-TITLE-2 (`72b4e4a`). Item 11: R-TITLE-1 item 4 replaces `history.go(-n)` with `history.pushState`. |
| ADR 0206 item 13 | 13 | ADR 0223 item 7, as changed by ADR 0226 items 7 and 8, ADR 0241 item 3 and ADR 0242 item 3. |
| ADR 0206 item 14 | 14 | ADR 0223 item 8, ADR 0228 items 11 and 12, and ADR 0254 items 7 and 10. |
| ADR 0206 items 15 to 41 | 15 to 41 | None. The page sequences stay start values. |
| ADR 0206 open point O-9 | ADR 0223 items 5 and 6 | The release search replaces the open point. |
| ADR 0206 "Invariants" | "Invariants" | The Catalog changes of ADR 0254 item 7. |
| ADR 0206 "How This Decision Is Checked" | "How This Decision Is Checked" | The destination link opens the Catalog. |

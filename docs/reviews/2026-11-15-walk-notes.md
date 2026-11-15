# Walk Notes Of 2026-11-15

Date: 2026-11-15.

Status: advisory. This review records the comments of the operator about the candidate `0.1.0-rc.23`. It states no rule. The owner of each item owns the decision.

## Notes

Each note gives the comment of the operator in neutral form, and then the facts from the code.

### Note 1: The Row **Announce** In The Review Sheet

Comment: the row **Announce** in the review of a draft is not necessary. The check box "Send a podping" is in view and gives the same choice.

Facts:

- [ADR 0209](../adr/0209-the-review-sheet-keeps-the-podping-box-in-view-and-names-each-delete.md) items 1 to 3 own the two parts. Item 2 keeps the row **Announce** at the top, with the full words and the address. Item 3 puts the time of the send in that row.
- The consequences of ADR 0209 say that the address of the podping and the check box are in two places.
- `AnnounceLines` in `crates/api/ui/editor.js` makes the row. `announceLines` in `crates/api/ui/announce.js` gives its lines.
- The row gives four things that the check box does not give:
    - The address that the podping names. A move gives two addresses (ADR 0189 item 1).
    - The time of the send. A wait of Music Index moves that time (ADR 0190 items 6 and 7).
    - The line "No announcement. No index learns about this write until it reads the feed by itself." when the check box is off.
    - The cause of no announcement, when the feed has no public address or the write deletes the feed file. The sheet shows no check box in these two conditions.

Open: no decision. A removal of the row changes ADR 0209 items 2 and 3. Each of the four things then needs a new position, or a decision removes it.

### Note 2: The Desktop App Asks For The Passphrase At Each Start

Comment: a task must find the options that let the desktop app unlock the vault at its start, with no passphrase from the user.

Facts:

- [ADR 0003](../adr/0003-user-unlocked-secret-vault.md) item 2 owns the rule. The desktop shell keeps the vault as one encrypted local file, and the passphrase of the artist opens it.
- ADR 0003 names the keychain of the operating system as an alternative. The operator rejected it, because the encrypted file can move between computers.
- [ADR 0142](../adr/0142-a-user-selects-the-time-before-the-vault-locks.md) item 2 gives the desktop the lock time "Never" as the start value. That time applies only after an unlock.

Open: the task did not start. An unlock at the start must keep the vault key or the passphrase in a location that is not the vault file. That changes ADR 0003 item 2.

### Note 3: A Keyboard Shortcut For The Left Panel At A Narrow Width

Comment: a task must find the options for a keyboard shortcut that opens and closes the left panel at a narrow width. The shortcut is for a device with no touch screen.

Facts:

- At 760 px or less, Studio shows no left panel. `App` in `crates/api/ui/app.js` makes the `nav` element with the class `sidebar` only when the width is more than 760 px.
- [ADR 0088](../adr/0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) item 7 owns the narrow layout of the levels. Each level is a page of the stack, and Back opens the level above.
- [ADR 0085](../adr/0085-each-artist-and-label-is-a-destination.md) item 7 puts the destinations on the root page at that width. It names musicindex ADR 0069 R-SHELL-2 item 2 as the rule of the three apps.
- The edge grip shows only on a touch screen. A device with no touch screen has the Back button of the toolbar and the Back key of the browser.
- The search found no keyboard shortcut for a move between pages in `crates/api/ui`. The key handlers there move the focus in a list, a tree, a menu or a tab row.

Open: the task did not start. The shell has no panel to open at that width, so the task must first say what the shortcut shows. A panel that opens above the page changes ADR 0088 item 7, and musicindex ADR 0069 can also apply.

### Note 4: The Name "Edge Grip" Becomes "Edge Indicator"

Comment: the word "grip" is not a good name for the pill at the screen edge.

Facts:

- musicindex ADR 0071 owns the term "edge grip", its title and the rule names `R-GRIP-1` and `R-GRIP-2`. The term is in 10 files of `musicindex` and in 9 files of `musicindex-studio`.
- ADR 0071 says that the pill is not a control. "Grip" tells a reader that a person can hold the pill.
- Studio has the CSS class `edge-grip` and the function `showsEdgeGrip` in `crates/api/ui`.

Operator choice: the name is "edge indicator". The operator did not select "back indicator", because Studio has a Back button. "Home indicator" is the Apple term for the bar at the bottom edge of an iPhone, so it is not available.

Open: the orchestrator has the proposal to change the term in musicindex ADR 0071. Studio changes its text after the owner changes. The recommendation is to change the code names in the same change. The owner decides on the rule names `R-GRIP-1` and `R-GRIP-2`.

### Note 5: The Shell Does Not Reach The Bottom Of The Window

Comment: in step 2 of the walk, the page of a release stops above the bottom edge of the window of the AppImage.

Facts:

- The screenshot shows a window of 1440 by 860 px. The shell stops at 766 px, and 860 / 1.12 is 768. The factor 1.12 is the UI scale "Large" of [ADR 0134](../adr/0134-a-ui-scale-and-larger-release-tiles.md) item 2.
- `styles.css` divided `100dvh` and `100vw` by the factor of the level. That division is correct only when the engine scales these units by the CSS zoom.
- Chromium 157 and WebKitGTK 2.54.2 scale the units. A box of `100dvh` there is 963.2 px in a window of 860 px, and the shell filled the window before the change.
- The release workflow builds the AppImage on Ubuntu 22.04, which publishes WebKitGTK 2.50.4.
- Three more screenshots of the AppImage agree with an engine that does not scale the units:
    - At "Medium", the shell fills the window.
    - At "Extra large", the shell stops at 673 px in a window of 848 px, and 848 / 1.25 is 678.
    - At "Extra small", the shell is higher than the window. The lower part of the left panel is out of view, and the page has a scroll bar.
- A run of WebKitGTK 2.50.4 in the sandbox was not possible, so no agent measurement shows the engine.

Operator choice: none. This is a defect correction.

Correction: `measureWindowUnits` in `crates/api/ui/uiscale.js` measures a box of `100dvh` and a box that fills the window. When the two heights agree, it sets `--window-unit-zoom` to 1. The app calls it before the first render and after each change of the level. `crates/api/ui-tests/uiscale.test.mjs` examines the selection of the factor.

Open: the visual check on the AppImage of the next candidate, at the levels "Extra small", "Large" and "Extra large".

### Note 6: The Track Count In The Rows Of The Releases List

Comment: each row of the releases list must show the number of tracks of the release.

Facts:

- The comment came in step 3 of the walk. That step asks the operator to compare the kind of each release with its track count, and the list does not show the count.
- `ReleasesPage` in `crates/api/ui/app.js` makes each row and each tile. The line below the title shows the kind, then "Hidden from apps", then the file host in the rows view.
- The feed list handler in `crates/api/src/routes.rs` calculates `track_count` for each music feed. It uses the count only for `kind_shown`, and the reply does not contain the count.
- The line gets its kind from `feedKind` in `crates/api/ui/shell.js`.
- [ADR 0213](../adr/0213-a-release-starts-on-the-page-of-its-artist.md) has the rules of the release kind.

Open: no task. The reply needs a `track_count` field, and the row needs a position for the count. A playlist has no tracks of its own, so the task must say what its row shows.

### Note 7: The Field "Kind" Moves To The First Tab Of A Release

Comment: the field "Kind" must not be in the tab **Settings**. Its position is the first tab of a release, between "Title" and "Artist". The name of that tab is the kind: "Album", "EP" or "Single".

Facts:

- [ADR 0213](../adr/0213-a-release-starts-on-the-page-of-its-artist.md) item 13 owns the position. It puts the menu in the tab **Settings**. A change saves at once, is not a part of the draft and needs no write.
- `settings` in `crates/api/ui/pages.js` gives a release the section `kind` as the first section of the tab. The section is in `crates/api/ui/editor.js`, with `saveKind`.
- The other fields of the first tab are parts of the draft. "Title" and "Artist" change the feed file through Review and a write.
- The menu also has the value "From the tracks" (ADR 0213 item 12), and its hint says that the feed file has no tag for the kind (item 16).

Open: no task. The move changes ADR 0213 item 13. The task must say how a field that saves at once shows between fields of the draft.

## Results Of The Steps

The step numbers are those of the section "What to test" of the [notes of `0.1.0-rc.23`](../releases/0.1.0-rc.23.md).

- Step 3: the kind of each release agrees with its track count. A release of 2 tracks shows "Single" (two releases). A release of 7 tracks and a release of 19 tracks show "Album". The collection has no release of 4 to 6 tracks, so the upgrade to "EP" stays open.

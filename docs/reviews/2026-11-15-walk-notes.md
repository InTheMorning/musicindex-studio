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

### Note 8: The Domain Line After A Delete

Comment: the domain of the feed address assures only one thing, that the artist controls the `podcast:guid` of the feed. After the removal of the entry and the delete of the file, one danger stays. A different person can collect the payments that a music podcast sends to that GUID.

Facts:

- [ADR 0208](../adr/0208-the-delete-and-move-sheets-give-goals-state-points-and-the-domain-line.md) item 1 gives the words: "Keep the domain {host}. If it expires, its new owner can take over this feed and its payments." Item 4 gives the same words in four places.
- After a done delete, the line shows in the Retire block, with **Put the File Back** (ADR 0208 items 12 and 13).
- The context of ADR 0208 gives the same cause: the next owner of the domain can publish a feed with the same GUID and different payment recipients. It says that no test has shown this.
- `podcast_guid_for_url` in `crates/core/src/guid.rs` makes the GUID of a new feed from its address. A feed keeps its GUID when its address changes.
- After a delete, the user has no feed at the address and gets no payments there. The words "take over this feed and its payments" then do not give the danger of the comment.

Open: no decision. Different words after a done delete change ADR 0208 items 1 and 4. The orchestrator said on 2026-11-15 that ADR 0208 is not decided, so this session changed nothing.

### Note 9: The Review Sheet Has No Row **Announce**, And The Check Box Moves

Comment: the check box "Send a podping" moves to the right side of the review sheet, above the button **Write to the File Host**. The row **Announce** goes.

Facts:

- This note continues Note 1, which gives the four things that only the row **Announce** shows.
- [ADR 0209](../adr/0209-the-review-sheet-keeps-the-podping-box-in-view-and-names-each-delete.md) item 1 puts the check box in the footer, in its own row above the buttons. Items 2 and 3 keep the row **Announce**.

Operator choice: remove the row **Announce**, and put the check box at the right side above **Write to the File Host**.

Open: the choice changes ADR 0209 items 1 to 3. Each of the four things of Note 1 needs a new position, or a decision removes it. The orchestrator said on 2026-11-15 that ADR 0209 is not decided, so this session changed nothing.

### Note 10: One Row In Review For "Hide From Apps"

Comment: the text "Block in Apple Podcasts" is not good, because a musician has no interest in Apple Podcasts. The word "Blocks" is not clear. Review must show this change as one row with a good name.

Facts:

- [ADR 0063](../adr/0063-one-switch-hides-a-feed-from-apps.md) items 3 and 4 give the switch "Hide from apps". The switch changes two tags: `itunes:block`, and each `podcast:block` with no `id`.
- Review shows one row for each field of the feed. `KEY_WORDS` in `crates/api/ui/review.js` names the field `itunes_block` "Block in Apple Podcasts" and the field `blocks` "Blocks". Thus one change of the switch gives two rows.
- `crates/core/src/review.rs` gives the two fields to the review.
- A `podcast:block` with an `id` applies to one platform. The switch does not change it (ADR 0063 item 5), and it is in the same field `blocks`.

Operator choice: the row has the name "Hide from apps", and its values stay "Yes" and "No". The values are not "On" and "Off".

Open: no task. One row with the name of the switch is possible when the two tags change together. The task must say what Review shows when only one tag changes, and what it shows for a block of one platform.

### Note 11: A Confirmation That Music Index Got A Write

Comment: after a write with a podping, Studio must show a confirmation when Music Index has the change.

Facts:

- Studio has no such confirmation, and no task for it.
- The row "Announcement" of "Feed facts" shows the state of the podping in the queue of Studio. It does not show what Music Index did.
- [ADR 0175](../adr/0175-the-check-shows-what-music-index-holds.md) item 10 gives the one line about this subject. The tab **Check** says "Music Index has not read your last write yet." when the build date of the index is earlier than the date of the last write.
- The line shows only the negative state. The requests go out only when the user opens the check or selects **Check Again** (ADR 0175 item 1).
- ADR 0175 lists two alternatives that the operator did not select on that date: a request some minutes after each write, and a row for a last write that the index has not read.
- [ADR 0184](../adr/0184-after-a-move-studio-shows-the-podping-of-the-move.md) gives a notice with the podping state after a move only. The notice goes when Music Index gives the new address.

Open: no decision. A confirmation after each write needs a request to Music Index after the write. That changes ADR 0175 item 1 and replaces the alternative that the operator did not select.

### Note 12: An Added Live Item Shows Its Times In Review

Comment: the row of an added live item in Review shows only its title and "Added". The times of the show are also new, so the row must show them in a form that a user can read.

Facts:

- The comment came in step 10, from the simplified view of a write that adds the show "test show".
- The row function in `crates/api/ui/review.js` gives an added entry or a removed entry only the word "Added" or "Removed". It shows the fields of an entry only for a change.
- [ADR 0114](../adr/0114-the-review-shows-the-changes-in-words-and-marks-the-changed-part-of-a-line.md) owns the field view. Its revision of 2026-11-05 says that each time shows in local time.
- The row "Build date" of the same review shows its two times in local time.

Open: no task. The task must say which fields of an added live item the row shows, and if a removed live item shows the same fields.

### Note 13: The View Of Review Is A Segmented Control

Comment: the select menu "View" of Review, with "Simplified" and "Detailed", needs two clicks for a change. A segmented control needs one click.

Facts:

- The view bar in `crates/api/ui/editor.js` makes a `select` element from `VIEWS` and `VIEW_LABELS` of `crates/api/ui/review.js`. The button **Show the Whole File** is at its right side in the detailed view.
- musicindex ADR 0051 R-SEG-1 says that a segmented control selects one of two or more views of the same content. The two views of Review are such views.
- `styles.css` has the class `segmented` for R-SEG-1. The tabs of a feed page and the tabs of Settings use it ([ADR 0157](../adr/0157-the-page-tabs-are-a-segmented-control.md)).
- A select menu is the form for one setting with a small number of values, from the operator choice of 2026-11-06. The view of Review is a view, not a setting, so that choice does not apply.

Open: no task. The change is in one component. The hint below the label names the selected view, and it must stay.

### Note 14: The Cloud Mark On The Release Row Above The Tracks

Comment: a release with the orange cloud mark in the left panel loses the mark when the user opens it. The row of the release above the track list must show the mark too, so that the user finds the way to **Read Again**.

Facts:

- [ADR 0165](../adr/0165-the-left-panel-marks-the-state-of-each-feed.md) items 1 and 2 give the state marks of a row of the left panel. `STATE_MARKS` in `crates/api/ui/outsidecheck.js` gives the state `outside` the icon `cloud-alert` and the warning tone.
- The outline of an open feed in `crates/api/ui/app.js` has the row of the feed above its tracks. That row shows the cover, the title, the kind and the mark of a change in the library folder. It shows no state mark of ADR 0165.
- The notice "The hosted file changed outside Studio." with **Read Again** is on the feed page ([ADR 0150](../adr/0150-studio-shows-a-change-from-outside-when-a-page-opens.md) item 2).
- ADR 0165 item 6 says that one store in the shell keeps the results. The left panel and the tiles read that store, so the row of the outline can read it too.

Open: no task. The task must say if the row of the outline shows only the cloud mark, or each state mark of ADR 0165 item 2.

## Results Of The Steps

The step numbers are those of the section "What to test" of the [notes of `0.1.0-rc.23`](../releases/0.1.0-rc.23.md).

- Step 3: the kind of each release agrees with its track count. A release of 2 tracks shows "Single" (two releases). A release of 7 tracks and a release of 19 tracks show "Album". The collection has no release of 4 to 6 tracks, so the upgrade to "EP" stays open.
- Step 4: pass. The operator made a release with the kind "EP" from the page of the artist.
- Step 5: open. The collection of the operator has no release in the group "Not linked".
- Step 6: pass. The sheet shows the goals and the domain line with the real domain.
- Step 7: pass, from the report of the operator.
- Step 8: pass. The check box "Send a podping" stays in view in a long review.
- Step 9: pass. After a move, the group of the move shows **Delete Now…**, and the delete review has the red button with the count as its first button.
- Step 10: pass. The operator removed a test live show and wrote the feed.
- Step 11: pass. The operator deployed the candidate, and an account stays.
- Step 12: pass. The server follows the move of the desktop app, and it edits the feed after **Use This File Host**.
- Step 13: open. The operator did not do this step.
- Step 14: open. The operator did not do this step.

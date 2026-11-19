# Human Check Of 0.1.0-rc.27 On 2026-11-19

The operator did the steps of "What to test" in the [notes of 0.1.0-rc.27](../releases/0.1.0-rc.27.md). The app was a local AppImage of `98d8bfa`, which has rc.27 and the four commits after it.

## Results

| Step | Result |
| --- | --- |
| 1. Upgrade to schema version 44 on the real collection | Pass. Releases and People showed all feeds and persons. |
| 2. "Choose Folder…" in the first-run sheet | Pass. The dialog opened at the home folder, and the selected folder filled the field. |
| 3. The result of a write in Review | Pass. The sheet "Written" showed the host and the public address with Copy. The address opened the feed with the change. |
| 4. "Open on musicindex.org" | Fail. The link shows in the head. A click does nothing (finding 5). The context menu item "Open Link" opens the correct page. |
| 5. The phone layout on the iPhone | Pass. The fixed rows came first, a release opened with one tap, and Back went to the outline and then to the release page. |
| 6. The Arch package | Pass. The package built from GitHub `main` and installed. The window showed the UI, and an MP3 track played. `gst-libav` was not installed, because of a partial upgrade conflict on the computer. |
| 7. The version line in Settings (rc.26 step 1) | Pass. The Arch package showed `Music Index Studio 0.1.0-rc.27 (8425d74, 2026-11-19)`, and the copy gave the full line. |
| 8. The version line on the server (rc.26 step 2) | Pass. `Music Index Studio server 0.1.0-rc.27 (8425d74, 2026-11-19)`. |
| 9. An upload with a locked vault on the server (rc.25 step 1) | Pass. Studio asked for the unlock, and the upload completed. |
| 10. The file dialog of the desktop app (rc.24 step 4) | Pass. The MP3 files were not gray, and one could be selected. Step 9 also covers rc.24 step 3. |
| 11. A write with a podping (rc.24 step 5) | Fail. The waiting notice showed, and then it disappeared after the time limit (finding 7). |
| 12. The percentages with a fee (rc.24 step 6) | Pass. With **Fee** checked on a recipient of 69 shares, it showed "14.1 % fee", and the recipient of 420 shares showed 85.9 %. Findings 8 and 9 came from this step. |
| 13. The mark on the iPhone (rc.24 step 7) | Pass. The mark showed before Back, and a touch opened the root page at once. |
| 14. The title on the iPhone (rc.24 step 8) | Pass. A touch on the title opened the root page. |
| 15. The swipes on the iPhone (rc.24 steps 9 and 10) | Pass. The edge swipe on the root page left Studio, and a forward swipe after Back opened the release again. A touch screen has no Forward button. Finding 10 came from this step. |
| 16. The recipients on the tab Value Routes | Pass. Finding 11 came from this step. |
| 17. The arrow keys in the releases grid (rc.24 step 12) | Pass. With two lines, Up and Down kept the column, a shorter line took the focus to its last release, and Down on the last line did not move. |
| 18. The menu of a wallet address row on the iPhone (rc.23 step 15) | Pass. Findings 12 and 13 came during this step. |
| 19. A back swipe in a new tab before a touch (rc.23 step 17) | Result recorded. Safari did not leave Studio: the swipe opened the root page of Studio. A forward swipe after that did nothing (finding 15). |
| 20. Delete and put back the file of a test feed (rc.23 steps 6 and 7) | Skipped by the operator. Open. The agent did the sheet part of step 6 (see below). |

The operator ended the check after step 20. These steps stay open: rc.23 steps 7, 12 and 13, and the items of rc.23 step 19.

### Steps That The Agent Did On A Copy Of The Real Data

The operator copied the real data folder to `.studio-dev/real-copy` (ignored). The walk script `.studio-dev/journeys/run-real.sh` (mode `real`) runs the walk binary of `98d8bfa` on a copy of the collection with no vault file and no internet. The walks did not write to a file host.

| Step | Result |
| --- | --- |
| rc.23 step 3: the kind on the tiles | Pass. Walk `vreal-a`: 19 tracks show "Album", 4 tracks show "EP" and 7 tracks show "Album". No release has a kept kind. A release of 1 to 3 tracks is not in the data. The unit test of `release_kind_shown` covers it. |
| rc.23 step 6: the sheet of "Delete from File Host…" | Pass. Walk `vreal-c`: the sheet shows the rows "Put the feed on a different file host" and "Take the feed out of the apps", and the line "Keep the domain files.heycitizen.xyz." The third row shows only after Music Index removes the entry (ADR 0131 item 4). |
| rc.23 step 14: a person with two linked pages | Pass. Walk `vreal-b`: "Prosperous Soul" shows "On wavlake.com" and "On feeds.fountain.fm". Each picture and each website shows its source or "Added in Studio". |
| rc.23 step 16: the bar at the left edge | Not applicable. ADR 0206 item 11 removed the edge indicator. |

## Findings

1. The dialog of "Choose Folder…" starts at the home folder also when the field has a typed folder. ADR 0223 item 10 gives this start folder. Open: the recommendation is to start at the typed folder when it exists. The operator has not decided.
2. Defect: the row "Nostr profile" grew from 44 px to 52 px when the profile picture came. Operator choice: correct it. Now a key row keeps a 32 px place for the picture while it loads. The row "Artwork" does the same. Walk `.studio-dev/journeys/v225b.json` (mode `base`): the row with a key is 52 px before the profile comes.
3. The first tab of a release showed the kind, for example "Single", and its first section had the heading "Album". [ADR 0225](../adr/0225-the-first-tab-of-a-release-is-release.md) gives the tab "Release" and the heading of the kind. Walks `.studio-dev/journeys/v225a.json` and `v225b.json` (mode `base`).
4. Open: the operator does not like the position and the look of the check box "Send a podping" and its hint in the footer of Review. The check box is in the center of the footer, above "Discard the Draft" and "Write to the File Host". Next: drawings of other positions for the operator.
5. Defect: in the desktop app, a click on "Open on musicindex.org" does nothing. The context menu item "Open Link" opens the page in the browser. The cause is not examined yet.
6. The installed program of the Arch package is `studio-desktop-app`. The operator wants the name `musicindex-studio-desktop`.
7. Defect: after a write of a listed release with **Send a podping** on, the notice "You wrote this feed at {time}…" showed. After the time limit, the notice disappeared. The confirmation "Music Index has your write of {time}." did not show, and the text "Music Index has not read your write…" did not show. The app was the Arch package of `8425d74`. Cause: the write of the test album has the build date 1795112661, and the podping went at 1795112695. Music Index gives the feed `created_at` 1795112867, so the index held the feed only 172 seconds after the podping. Before that time, the route gave `index_read` null. `writeNotice` gave no notice for null after the podping, and `nextNoticeLoad` stopped the reads. Thus the confirmation could not come. Correction: after a podping, `index_read` null is a write that Music Index did not read (ADR 0175 item 1). The notice stays, the reads continue to the limit, and then the notice gives **Check Again**. A write with no podping gives no notice, as before. Test: `crates/api/ui-tests/outsidecheck.test.mjs`.
8. On the tab **Value Routes**, the check box **Fee** of an open recipient moves the recipient to the section "Fees", below **Add a recipient**. When the operator clears **Fee**, the recipient goes back and closes. The row moves away from the pointer. After the check, the other recipient (HeyCitizenNode) was open, and the fee recipient was closed. Cause: the group of a row followed the check box at once, but [ADR 0144](../adr/0144-recipients-use-one-picker-and-sort-by-split.md) item 4 says that a row does not move while the focus is in it. Also, the two groups were two lists, so a moved row got a new element. Correction: a row changes group only at the next sort, when the focus leaves the row. The rows and the headings are one keyed list, so that a moved row stays open. Walk `.studio-dev/journeys/vfee-a.json` (mode `base`). The walk page has no window focus, so the walk sends the `focusout` event itself.
9. Defect: in the sheet **Add a Recipient** of the desktop app, the type "LNURL Address" shows a very large "@" icon behind the hint and the field. The icon fills the width of the sheet. The app was the Arch package of `8425d74` (WebKitGTK). The browser screenshots of the walks did not show it. The cause is not examined yet. The same icon shows in the sheet **Add a Wallet** of a person page in Safari on the iPhone (WebKit). Cause: the `Icon` helper gives an `svg` element with no size, and `.type-panel > .icon` gave it no size. WebKitGTK drew it at 688 × 688 px. Correction: the default rule of musicindex ADR 0065 R-FIT-7 item 2 in `styles.css`, and 20 px for `.type-panel > .icon`. The walk tool runs `unsizedIcons` at each width. WebKit walk `.studio-dev/journeys/wkicon-a.json` with `.studio-dev/wk-run.sh`: 20 × 20 px after the change.
10. On the iPhone, a forward swipe from the root page always opens the page of the artist. The operator expected the page that was open before the root page: the outline "Sections" of a release. The cause is not examined yet.
11. On the tab **Value Routes**, the section "Fees" shows below "＋ Add a recipient". The operator wants "Fees" above "＋ Add a recipient". Operator choice: "＋ Add a recipient" is after the fees and the fee line. Walk `vfee-a`, screenshot `05-add-last-390`.
12. Open: the rows **Settings**, **People** and **Hosts** of the root page are at the top in the narrow layout and at the bottom in the wide layout. The operator does not like this difference. The owning decision is not examined yet. Next: drawings for the operator.
13. Open: the person pages have no tabs. The other long pages have tabs. The operator wants tabs on the person pages. Next: drawings of the tabs for the operator.
14. Open: the operator wants undo and redo where possible. Now only a merge of two persons has **Undo** ([ADR 0217](../adr/0217-a-new-artist-sees-the-result-of-a-write-the-payment-types-and-an-undo-of-a-merge.md) items 12 to 17). Next: a list of the operations that can have undo and redo, and a proposal for the operator. This is a design change across modules, so it needs a decision record.
15. Defect: in a new Safari tab on the iPhone, a back swipe before a touch opened the root page of Studio. A forward swipe after that did nothing. After a touch, a forward swipe opens the page again (step 15). The cause is not examined. A likely cause (inference): WebKit skips the history entries that a page adds before a user action. Examine it together with finding 10.

## AppImage Build On Arch

The local build of the AppImage needs `patchelf`. Give these variables to `cargo tauri build` in `crates/desktop-app`:

- `GSTREAMER_PLUGINS_DIR` from `tools/release_desktop.py appimage-plugins --source /usr/lib/gstreamer-1.0`
- `GSTREAMER_HELPERS_DIR=/usr/lib/gstreamer-1.0`
- `NO_STRIP=true`
- `APPIMAGE_EXTRACT_AND_RUN=1`

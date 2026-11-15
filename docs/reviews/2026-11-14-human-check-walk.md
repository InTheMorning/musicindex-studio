# Human Check Walk Of 2026-11-14

Date: 2026-11-14.

Status: advisory. This review records the results and the comments of the operator during a walk through the open human checks. It states no rule. The owner of each check owns the decision.

The operator uses the local release build `target/release/studio-desktop-app` from 13:20 on 2026-11-14, unless a row names a different build. The commits after that build change only documents and the `owner` text of `crates/api/ui/assets/tokens.json`.

## Checks In The Walk

| # | Check | Owner | Needs | Result |
|---|---|---|---|---|
| 1 | The review of a delete: a red **Delete N Files** first, and **Cancel the Delete** stops the delete | ADR 0198, task 137 | Desktop app, a hidden test feed on a file host | Passed |
| 2 | Check H: a change from outside Studio | Notes of 0.1.0-rc.21, step 27 | Desktop app, a test feed on a file host | Passed |
| 3 | Check E: pictures and the network, items 1 to 3 | ADR 0159, notes of 0.1.0-rc.21, step 28 | Desktop app, the network | Item 2 passed. Items 1 and 3 open |
| 4 | Check D: a release move and a return between two real file hosts | Notes of 0.1.0-rc.21, steps 24 to 26, ADR 0128, task 091 | Desktop app, two real file hosts | Passed |
| 5 | A timed **Play** in an AppImage after 0.1.0-rc.19 | Finding 18 of the local use findings of 2026-11-06 | The AppImage of 0.1.0-rc.22 | Passed |
| 6 | ADR 0175 in the AppImage | Step 11 of the findings of 0.1.0-rc.22 | An AppImage with 8a23157 | Open. Blocked until the next candidate |
| 7 | The server: an account stays, and two linked pages show their sources | Notes of 0.1.0-rc.22, steps 8 and 9 | The server | Step 8 passed. Step 9 failed in 0.1.0-rc.22, as expected. Open for the next candidate |
| 8 | The iPhone: the rows of a person fit, and each tab of Settings opens | Notes of 0.1.0-rc.22, steps 10 and 11 | The iPhone, the server | Passed |
| 9 | The stream checks: a paste in the stream address field on the desktop and the iPhone, and **Check Stream** on the server | Notes of 0.1.0-rc.21, task 100, ADR 0173 | A real Icecast mount, the server, the iPhone | Passed |

The older open human tests (Phase 12, tasks 065 to 084, and the parts of 0.1.0-rc.10 to 0.1.0-rc.20 that did not pass) are not in this walk. Their owners give their steps.

## Results And Comments

Each entry gives the step, the result and the words of the operator in neutral form.

### Check 1, Step 1: A Hidden Test Feed

Result: passed. The test feed is "HeyCitizen's Test Album" at `https://music.cancelbandcamp.com/TestAlbum/test.xml`. The operator turned on **Hide from apps** and wrote the feed. The tab **Check** said "Studio found no problem in this feed." and "Music Index does not hold this feed yet."

Observations of the operator:

1. The review of the write says "This feed is hidden from apps." before the write. The feed is not hidden until the write. The operator recommends a future tense, for example "This feed will be hidden from apps." Source: `crates/api/ui/editor.js:2180`.
2. The tab **Check** says "Music Index does not hold this feed yet." For a hidden feed, the word "yet" is not necessary and not correct, because Music Index does not take a hidden feed. Source: `crates/core/src/feed_check/index.rs:39` and line 274, with the test `crates/api/src/tests/feed_check.rs:374`.

### Check 1, Step 2: The Sheet "Delete the Feed File"

Result: comment, not a pass or a failure of ADR 0198. The sheet showed the host `heycitizen`, the path `TestAlbum/test.xml`, the check box with "Studio deletes 4 files. 1 file stays: 1 pointer file, and 0 files that another feed uses.", the line "“HeyCitizen” links to this feed.", the title field and **Delete** off.

Observation of the operator: the list "You lose these things:" has too much text, and most of it is misleading. It makes the user afraid of a loss in many words. It does not tell the real scope of a delete. The delete of a file is a different thing from the loss of a domain name. The three items are:

1. "The control of the entry in Music Index and in Podcast Index. The entry keeps its last payment recipients."
2. "A different person who gets this address can publish a feed with this GUID."
3. "A different address does not give the entry back. Only a write to this address brings the file back."

Source: `crates/api/ui/editor.js:2468` and line 2469. Owner: ADR 0131 items 2, 7, 10 and 14. Note of the agent: ADR 0131 item 10 wrote this list for **Delete Anyway**, before Music Index removed the entry. ADR 0200 removed **Delete Anyway**, so Studio now deletes only after Music Index removed the entry. Item 1 of the list possibly does not agree with ADR 0200 for Music Index.

### Check 1, Step 3: The Review Of The Delete

Result: passed for ADR 0198 items 1 to 4. The red **Delete 5 Files** is at the left edge. **Cancel the Delete** is at the right edge. The review has no **Write to the File Host**. The review listed four files with "Studio deletes it after the feed is written." and `TestAlbum/test.xml` on `heycitizen` with "This write deletes the feed file." The row **Announce** said "No announcement: this write deletes the feed file."

Observations of the agent from the screenshot. The operator did not comment on them:

1. A red row with the kind "Delete" shows `TestAlbum/test.xml` on the host `bastone`, with "Studio keeps this file. It tells the apps the new address of the feed." and **Undo**. The kind "Delete" and the text "Studio keeps this file" do not agree. The row uses the error color and `role="alert"`, but nothing is wrong. Source: `BlockedDeleteRow` in `crates/api/ui/editor.js:1790`. Owner: ADR 0036 item 13 and ADR 0131 item 13.
2. The review of a delete also shows the notice "This feed is hidden from apps. Apps and directories do not list it after the write." After a delete, no file is listed, so this notice possibly is not necessary. It also has the tense of observation 1 of step 1.

### Check 1, Step 4: Cancel The Delete

**Cancel the Delete** opened the confirm "Discard the draft?" with "Studio deletes each change that you did not write. The feed file on its file host does not change." and the buttons **Discard the Draft** (red, left) and **Cancel**.

Observation of the operator: replace "deletes each" with "discards any". The text then says "Studio discards any change that you did not write." Source: `crates/api/ui/confirms.js:33`.

### Check 1, Step 5: Discard The Draft

Result: passed. The operator clicked **Discard the Draft**, and the page showed the expected state. The agent then read the host. `test.xml` and the four files gave HTTP 200. So **Cancel the Delete** stopped the delete. The gate of ADR 0198 passed.

### Check 2, Step 1: A Change From Outside

Result: done. The operator made the change on studio.musicindex.org, not with a tool other than Studio. For the desktop app, that write is a change from outside, because the desktop app does not know about it. The operator changed `<description>` from `clock test 2` to `clock test 3`. The agent read the host: `last-modified` 22:26:26 GMT, ETag `67983f32…`, and `lastBuildDate` 22:26:23 GMT. An earlier write of the server at 22:04:06 GMT (ETag `996023f7…`) did not change the description.

### Check 2, Step 2: The Mark In The Desktop App

Result: passed. **Releases** showed "Changed outside Studio" on the tile of the test feed, and the left panel showed the red cloud on the row of its artist or label.

### Check 3, Step 1: A Release Page With No Network

Result: passed. With the network stopped, the page of a release opened, its tabs operated, and a different release opened. Check E item 2 passed.

### Check 3, Items 1 And 3: Not Run

The operator selected to skip items 1 and 3 for now. They need a person with a read-only link to a Music Index page that is not in the collection, and a new picture on that page. Items 1 and 3 of Check E stay open. The button of a suggestion is now **Add** (ADR 0182 and ADR 0196), not **Use**.

### Check 4: The Test Feed

Operator choice: Check D uses "HeyCitizen's Test Album" (GUID `7ea30e6c-c7ba-5b81-a7be-6465c1591b7a`). The operator unhides it for the check and hides it again at the end. The host `bastone` already has a pointer file at `TestAlbum/test.xml` from an earlier move, so a move to `bastone` also touches ADR 0178.

### Check 4, Step 1: Unhide The Test Feed

Result: passed. The operator read the feed again, selected **Stop Retiring** and wrote the feed. The agent read the host: `podcast:block` is `no`, `itunes:block` is `No`, and the description is `clock test 3`. Music Index then held the feed at `https://music.cancelbandcamp.com/TestAlbum/test.xml`.

Observation of the operator: the review sheet starts with the row **Announce**, but the podping check box ("Tell Podcast Index and send a podping") is at the bottom of the sheet. The user sees it only after a scroll to the bottom. The operator does not want this check box to be out of view. A future design must give it a better position. Source: `crates/api/ui/editor.js:1831`. This item needs a design decision.

### Check 4, Step 2: The Move Sheet To `bastone`

The **Files** tab showed the block "Old files to delete": "Move to heycitizen. Waiting. 4 files stay on bastone until the checks pass.", "New files ✓ All 4 are on heycitizen", "Music Index ✓ Follows the new feed address", "Waiting time 1 of 30 days", **Delete Now…** and **Keep the Files**.

Observation of the operator: the move sheet did not show the choice of 7 days. The operator thought that the cause was the wait of the earlier move.

Note of the agent from the code: the select "Keep the old files for" shows only when the plan copies files and old copies stay. Source: `crates/api/ui/editor.js:4271`, and `from_connector_ids` in `crates/api/src/routes.rs:4967`. The 4 files of the earlier move from `bastone` are still on `bastone`, so only the feed file moves, and the select does not show. The default wait is now 30 days. The runbook still says 7 days. The step of the runbook is out of date. This move goes back to the earlier host, so it is the return of Check D, and the first move of the check is the earlier move from `bastone` to `heycitizen`.

### Check 4, Step 2: Comments During The Review

Defect report of the operator: a scroll up and down in the review sheet is slower than expected. During the scroll, `WebKitWebProcess` becomes the process with the highest CPU use. With a usual scroll of the mouse wheel, the problem is small. With the fast scroll mode or the tilt wheel of the mouse, the problem is much worse.

Note of the agent, not verified: `crates/api/ui/styles.css` has `backdrop-filter: blur(20px) saturate(1.4)` on `.bar` (line 774) and `filter: blur(40px)` on `.center::before` and `.hero::before` (line 513). WebKitGTK possibly calculates these effects again at each scroll step. A profile in WebKitGTK must find the cause before a change. The size of the XML diff in the sheet is a different possible cause.

Observation of the operator: blocks of grey text are tiring to read. The operator wants the text easier to read. This item needs a design decision. An example is the grey hint text of the delete sheet, the move sheet and the review.

### Check 4, Step 2: The Review Of The Return To `bastone`

Result: passed for Check D item 1 and for ADR 0178. The review showed these items:

- At the top, the plain notice "This is a return to an earlier address. Music Index follows this return at once. Each return in one day makes the wait for the next return longer." ADR 0191 items 2 and 6 give a plain notice at the top for this case. The runbook says "below the move row". The runbook is out of date.
- A move row from `https://music.cancelbandcamp.com/TestAlbum/test.xml` to `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml` on `bastone`, with "The address from before gets a pointer file with the new address." and "The pointer file at the new address becomes the feed." So Studio replaced the old pointer file with no question.
- Four rows **Keep** on `heycitizen`, with "The old copy stays. Studio asks you before it deletes this file."
- The row **Announce** with the podping of the old address. ADR 0189 gives one podping of the address before the move.

Clarification of the operator about the grey text: the text in the previous comment is the grey sub-lines of a review row, for example "From …", "To …", "The address from before gets a pointer file with the new address." and "bastone" in the move row.

### Check 4, Step 3: Write The Return

Result: passed for Check D items 2 and 3. The operator wrote the return, and the podping was sent. The agent then read these values:

- Music Index gives `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml` for the GUID. It followed the return with no wait.
- The feed at the new address and the pointer file at `https://music.cancelbandcamp.com/TestAlbum/test.xml` both have `<itunes:new-feed-url>https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml`.
- Each file of the feed (two pictures, two audio files and one `.srt` file) gives HTTP 200 on `files.heycitizen.xyz`.

The **Files** tab showed the block "Old files to delete": "Move to bastone. Waiting. 4 files stay on heycitizen until the checks pass.", "New files ✓ All 4 are on bastone", "Music Index Gives a different feed address", "Waiting time 0 of 30 days".

Observation of the agent: the row "Music Index" said "Gives a different feed address", while Music Index already gave the new address when the agent read it. Possibly the page read Music Index before the index changed. The next step looks at the row again.

Observation of the operator: the move starts a new wait of 30 days before Studio removes the old files again.

### Check 4, Step 4: The Music Index Row Again

Result: passed. After the operator opened a different release and then the test feed again, the row "Music Index" said "Follows the new feed address".

Observation of the agent: directly after the write, the row showed "Gives a different feed address", but Music Index already gave the new address. The row did not read Music Index again after the write. A new open of the page corrected the row.

### Check 4, Step 5: The Confirm Of **Delete Now…**

The confirm said "Delete the 4 old files before the move is ready?" and "An app that still uses an old address gets an error. You review each deletion, and heycitizen keeps the files until you write." The buttons were **Add the Deletion** (red, left) and **Cancel**.

Observation of the operator: the operator is not certain about the button text **Add the Deletion**. Source: `crates/api/ui/confirms.js:14`.

Note of the agent: the button adds the deletes to the draft, and the move sheet uses **Add to the Draft** for the same kind of action. The title asks "Delete …?", so a user can expect a button that names the delete.

### Check 4, Step 6: The Review Of The Old File Deletes

Result: passed. The review showed four rows **Delete** on `heycitizen`, all below `TestAlbum/test/`, each with "Studio deletes it after the feed is written." It had no row for `TestAlbum/test.xml` on `heycitizen` and no row on `bastone`. The row **Announce** gave a podping of `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml`, and the check box "Tell Podcast Index and send a podping" was selected, with "Studio sends it 30 seconds after the write."

Observations of the agent from the screenshot. The operator did not comment on them:

1. The write deletes four files, but the button is the blue **Write to the File Host**. ADR 0198 gives a red **Delete N Files** only to the review of a feed delete. The review of old file deletes does not have the same style.
2. The write announces the feed with a podping, but the feed itself possibly does not change in a useful way for apps. It is not verified if a podping is necessary for a write of only old file deletes.
3. The podping check box is again below the row **Announce**, at the bottom of the sheet. This agrees with the comment of the operator in step 1.

### Check 4, Step 7: Write The Deletes

Result: passed. Check D items 1 to 4 passed. The operator wrote the deletes. The agent then read both hosts and Music Index:

- The four old files below `https://music.cancelbandcamp.com/TestAlbum/test/` give HTTP 404.
- The pointer file `https://music.cancelbandcamp.com/TestAlbum/test.xml` gives HTTP 200.
- The feed on `files.heycitizen.xyz` and each of its five files give HTTP 200.
- Music Index still gives `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml`.

The operator agreed with the three observations of the agent in step 6 and in step 3 of Check 1, and said that they need a correction later.

### Check 4, Step 8: Hide The Test Feed Again

Result: done. The operator turned on **Hide from apps** and wrote the feed. The agent read the host: `podcast:block` is `yes`, and `itunes:block` is `Yes`.

Observation of the operator: the operator sent a podping with the write. After a move, Studio shows the podping and its wait. After a usual write, Studio gives no confirmation of the podping. The operator wants a visual confirmation for each podping that Studio sends. This item needs a design decision. Owner of the notice after a move: ADR 0184, ADR 0189 and ADR 0191.

### Comment: The Names And The Flow Of A New Feed

Observation of the operator: the names of the add actions are not consistent, and the flow to make a feed has a fault.

1. **Releases** shows "＋ Add a Feed". This is correct, because the next page asks for the kind of feed. Source: `crates/api/ui/app.js:3443`.
2. The page of an artist shows "＋ Add an Album". It must be "＋ Add a Release". Source: `crates/api/ui/welcome.js:133` and line 145.
3. The page "Make a New Release" must have a kind select menu with album, EP and single. The namespace has no tag for the kind yet, but the operator wants the select menu.
4. The creation of feeds must make the user understand the hierarchy of four separate containers, one in the other: the file host (if applicable), the label (if applicable), the artist, and the releases, shows and playlists. The operator wants a new design of the creation flow for this.

These items need a design decision. The operator wants mockups before the decision.

### Check 5: A Timed Play In The AppImage Of 0.1.0-rc.22

Result: passed. The operator closed the local build and started the AppImage `musicindex-studio-v0.1.0-rc.22-linux-x86_64.AppImage`. It opened the data with no error. The operator clicked **Play** on three tracks that were not played in the session. Each track started in about 1 second. The long wait of finding 18 did not reproduce in the AppImage.

### Check 9: The State Of The Stream Checks

The operator selected the stream check as the next check. The agent read the owners before the first step:

- Steps 6 to 8 of the notes of 0.1.0-rc.21 passed on 2026-11-10 on the desktop. The check with a real Icecast mount passed again on 2026-11-12 after task 100.
- Step 9 (a space before the stream address) is a mechanical criterion of packet 100d. It needs no human.
- Open: the server part of **Check Stream** with a real Icecast mount, and the paste in the stream address field on the iPhone (task 100, ADR 0173). Both need the server.
- No drive examined the paste in WebKit on the desktop. The AppImage of 0.1.0-rc.22 has task 100 and ADR 0173, so the operator can examine it now.

### Check 9, Step 1: A Paste In The Stream Address Field

Result: passed in the AppImage of 0.1.0-rc.22. A paste and a delete in the stream address field did not move the page.

Defect report of the operator: after the operator added a live show and discarded the draft, Studio showed the page of the album. The screenshot shows this state:

- The top bar says "HeyCitizen's Test Album › Live show", and the tab **Live Shows** is selected.
- The body shows the section "Album" with the fields of the album (Title, Artist, Nostr profile, Artwork, Description, Website, Hide from apps, Explicit).
- The top bar says "No changes".

So the top bar and the tab still name the live show that the discard removed, but the body shows the album. Expected: after the discard, the page goes to the tab **Live Shows** with no live show, and the top bar agrees with the body. The cause is not verified. The AppImage is older than local `main`, so a run of the local build must examine it again.

Observation of the agent from the screenshot, not verified: below the content, the window has an empty black band of about 90 px. The band is under the left panel and the main column. Possibly the page does not fill the height of the window in the AppImage.

### Server, Step 1: The Version Of The Server

Result: passed for step 8 of the notes of 0.1.0-rc.22. `GET /api/status` needs the CSRF header of the session, so the operator read the answer in the Network tab of the browser. It gave `"version": "0.1.0-rc.22"`, `feeds` 4, `connectors` 2, `hold_assets` false, `storage` 1273250 of 10737418240 bytes, and `vault` "locked". So the server runs 0.1.0-rc.22, and the account stayed with its feeds and file hosts.

Observation of the agent: the server shows its version only in `GET /api/status`, which needs a session and the CSRF header. The UI does not show the version. A user and the operator cannot see the version of the server without the developer tools.

### Server: The Test Feed After The Move Of The Desktop App

The operator sent a screenshot of "HeyCitizen's Test Album" on the server, with no comment. The page showed these items:

- The head gives the address `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml`.
- The notice "Other sources give a different address." with "The public address of the folder: `https://music.cancelbandcamp.com/TestAlbum/test.xml`" and "Studio uses the public address above. Studio does not change the feed file." Source: `AddressDifferences` in `crates/api/ui/app.js:647`. Owner: ADR 0021.
- The notice "The hosted file changed outside Studio." with **Read Again**.

Note of the agent: the desktop app moved this feed from `heycitizen` to `bastone` today. The server did not make the move. So on the server the folder of the feed is still on `heycitizen`, where the pointer file is now. The server found the new address in the feed and shows both addresses. The server does not offer to follow the move of a different Studio. It is not verified if this is a defect or the expected result of ADR 0021.

Comments of the operator on the server page of the test feed:

1. The operator expects a way to follow the move of a different Studio and to remove the notices. This is a missing function.
2. Defect report: **Read Again** does not remove the two notices.
3. If these notices are warnings, they must follow the rules of a warning: the triangle icon and the Warning color (ADR 0191 item 1, musicindex ADR 0068 R-STATUS-1). Now they are plain notices.
4. On the page of the artist HeyCitizen on the server, the tile of the test feed shows "Changed outside Studio" in blue, while the left panel shows the red cloud. The operator thinks that the blue is not correct. Source: `outsideMark` in `crates/api/ui/outsidecheck.js:20`, which gives `tone: 'accent'` for a change. Owner: ADR 0150 items 2, 3 and 5.

Observation of the agent, not verified: the hosted file of the server is now a pointer file with `itunes:new-feed-url`. `outsideMark` has the state `moved`, with "Moved outside Studio" and the error tone. The server gave the state `changed`, not `moved`. So the check possibly does not find a move by a different Studio.

### Server, Step 2: A Person With Two Linked Pages

The server had no person with two linked pages. The operator added Prosperous Soul on the server, with two read-only links.

Result: failed in 0.1.0-rc.22, as expected. The two linked page rows show only "Prosperous Soul" and "Read only", with no host. The first picture shows "Added in Studio", not the page as its source. These are findings 7 and 8 of 0.1.0-rc.22. Task 138 corrects them on local `main` after the tag, and its human check passed in the desktop app on 2026-11-14. Step 9 of the notes of 0.1.0-rc.22 stays open for the next candidate on the server.

### Server, Step 3: The Stream Check On The Server

Result: passed for the server part of task 100. On the server, **Check** with a live Icecast mount and an empty **Stream type** showed "Live", the type and the time, and it filled **Stream type** with MP3.

The defect of check 9 occurred again on the server: after the discard, the top bar said "HeyCitizen's Test Album › Live show", the tab **Live Shows** was selected, and the body showed the section "Album". So the defect is in the UI of 0.1.0-rc.22 on the desktop and on the server.

### iPhone, Step 1: The Rows Of A Person

Result: passed for step 10 of the notes of 0.1.0-rc.22. On the iPhone, the wallet of Prosperous Soul fit in the screen. Each button and the tag **Default** were fully visible, and the node address wrapped.

Observation of the operator: the position of the buttons needs a new design. The screenshot shows these items:

- Row "LNURL address": the tag **Default** is adjacent to the label. Below the address, one line has "Not used", the copy icon, then **Edit**.
- Row "Node address": one line has "Not used", **Make Default** in the accent fill, then the copy icon. **Edit** is alone on the next line.
- **Remove Wallet** is red, at the bottom of the wallet.

Note of the agent: the two rows put the copy icon and **Edit** in a different sequence, and **Edit** wraps alone in the second row. The status text "Not used" is on the same line as the buttons.

### iPhone, Step 2: The Tabs Of Settings

Result: passed for step 11 of the notes of 0.1.0-rc.22. Each tab of Settings opened on the iPhone, and each part fit in the screen.

### iPhone, Step 3: A Paste In The Stream Address Field

Result: passed for the iPhone part of task 100 (ADR 0173). A paste and a delete in the stream address field did not move the page.

Observation of the operator: on the phone, the app has a back button, but no quick way to go directly to the left panel. The user must push back many times from a deep page. This item needs a design decision.

## Items For Later Sorting

Each item is in the entries above. This list groups them by kind. None of these items has a decision yet.

Defects:

1. After the discard of a new live show, the top bar and the tab still name the live show, and the body shows the album. It occurs on the desktop and on the server (check 9, server step 3).
2. A scroll in the review sheet is slow, and `WebKitWebProcess` uses the most CPU (check 4, step 2).
3. On the server, **Read Again** does not remove the notices of the feed that the desktop app moved (server).
4. Not verified: the server gives "Changed outside Studio", not "Moved outside Studio", for a feed that is now a pointer file (server).
5. Directly after a move, the row "Music Index" shows "Gives a different feed address" until the page opens again (check 4, step 4).

Missing functions:

1. A way to follow a move that a different Studio made, and to remove its notices (server).
2. A visual confirmation of each podping, not only after a move (check 4, step 8).
3. A quick way back to the left panel on the phone (iPhone, step 3).
4. The version of the server in the UI (server, step 1).

Texts:

1. "This feed is hidden from apps." before the write: use a future tense (check 1, step 1).
2. "Music Index does not hold this feed yet." for a hidden feed: remove "yet" (check 1, step 1).
3. The list "You lose these things:" of the delete sheet is long and misleading after ADR 0200 (check 1, step 2).
4. "Studio deletes each change that you did not write.": use "discards any" (check 1, step 4).
5. The button **Add the Deletion** is not clear (check 4, step 5).
6. "＋ Add an Album" on the page of an artist: use "＋ Add a Release" (check 4, step 8).
7. The move sheet does not say why the wait select is not there (check 4, step 2).

Design:

1. The podping check box is at the bottom of the review sheet, out of view (check 4, steps 1 and 6).
2. Grey sub-lines of the review rows are tiring to read (check 4, step 2).
3. The red row of a kept pointer file says "Delete" (check 1, step 3).
4. The notice "hidden from apps" in the review of a delete (check 1, step 3).
5. The review of old file deletes has the blue write button, not a destructive button (check 4, step 6).
6. Possibly no podping is necessary for a write of only old file deletes (check 4, step 6).
7. The flow to make a feed: a kind select menu and the four containers file host, label, artist and release (check 4, step 8).
8. The notices of a different address and of an outside change must follow the warning rules, and "Changed outside Studio" must not be blue (server).
9. The sequence of the buttons in the wallet rows on the phone (iPhone, step 1).

## Operator Choices Of 2026-11-14

The operator answered a questionnaire about the list above. These lines record the answers.

- Operator choice: the texts and the small defects come first. The small defects are defects 1, 3, 4 and 5. A new candidate comes after them.
- Operator choice: texts 1, 2, 4 and 6 need no ADR. Their new words are:
    - Text 1: "This feed will be hidden from apps."
    - Text 2: "Music Index does not hold this feed." for a draft that is hidden from apps. ADR 0175 item 4 has the change.
    - Text 4: "Studio discards any change that you did not write."
    - Text 6: "＋ Add a Release" for each role, also on the first-step row of an artist. ADR 0164 item 2 and ADR 0119 items 8 and 9 have the change.
- Operator choice: the older open human tests become one short walk on the next candidate. The walk leaves out each step that a later walk or an agent check proved.
- Operator choice: each of these four groups gets mockups before a decision:
    - The review sheet: design 1, 2, 3 and 5.
    - The flow to make a feed: design 7.
    - The moves by a different Studio: missing function 1 and design 8.
    - The phone and the live shows: missing function 3 and design 9.
- Defect 1 is corrected: a live show that the draft does not have gives the tab **Live Shows** (`viewAfterLiveGone` in `crates/api/ui/live.js`, with a test). An agent walk at 1280 px showed "Album 2 › Live shows", the tab **Live Shows** and "No live show yet." after the discard. The human check in the next candidate is open.
- Defect 5 is corrected: while the row "Music Index" of a move says "Gives a different feed address", the **Files** page reads the moves again each 20 seconds, 15 times at most (`indexNeedsRead` in `crates/api/ui/releasemove.js`, with a test). An agent walk with a test index showed the row change to "✓ Follows the new feed address" with no new open of the page.
- Defects 3 and 4 have one cause, from a read of the code. No test reproduced it.
    - **Read Again** reads the file in the folder of the file host. On the server, that file is the pointer file. `record_read` in `crates/core/src/outside.rs` keeps its hash.
    - The outside check reads the public address of the feed. On the server, that address is the new address on the other file host. Thus the two files are always different, and **Read Again** cannot remove the notice.
    - `compare` gives "moved" only when the new address is not the public address of the feed. Here the public address is already the new address, so the result is "changed".
    - A correction needs the decision of missing function 1, how Studio follows a move by a different Studio. Defects 3 and 4 go into that group of mockups.
- Correction of the line "Runbook text that is out of date" below: the three texts are in the steps of the notes of 0.1.0-rc.21, which the runbook names. The notes of a released candidate do not change. The notes of the next candidate give the current steps.
- Text 3 has a decision: [ADR 0208](../adr/0208-the-delete-and-move-sheets-give-goals-state-points-and-the-domain-line.md), with task 142. The examination of the text also found that no control of the UI starts the restore of a deleted feed.
- Design items 1, 2, 3 and 5 have a decision: [ADR 0209](../adr/0209-the-review-sheet-keeps-the-podping-box-in-view-and-names-each-delete.md), with task 143. Design item 6 stays open.
- Design item 9 and the removal of a live show have a decision: [ADR 0210](../adr/0210-a-row-has-one-menu-and-a-live-show-can-be-removed.md), with task 144.
- Missing function 3 has a decision. The orchestrator session relayed it on 2026-11-14: the narrow layout of Studio does not change, and the website gets the same stack (musicindex ADR 0070). A swipe from the screen edge is the quick way. A visual cue for the swipe is not decided.
- Finding from the website session, not examined: Studio makes the history entry of its root page at the start, with no input of the user (`crates/api/ui/app.js`, about line 3772). Chromium and Safari can skip such an entry at the Back button and the back swipe. Thus Back on the start page, before the first touch, possibly leaves Studio. The check tool of the agent cannot push the Back button of a browser, so only a person can examine this. No code changed.
- Defects 3 and 4, missing function 1 and design item 8 have a decision: [ADR 0211](../adr/0211-studio-follows-a-move-that-a-different-studio-made.md), with task 145.
- Design item 7, the flow to make a feed, has these operator choices of 2026-11-14. No ADR and no task hold them at this time:
    - The operator rejected a block "Where it goes" with the levels as a tree.
    - A release is made only from within an artist. The form does not show a hosting page or a label page. Studio presumes that they are the same as for the artist.
    - The root of the levels is a publisher feed with the role hosting, not a file host. A hosting page and a label page are optional. A release must have an artist page.
    - The release kind (album, EP, single) is a value that Studio keeps and shows. The feed file has no tag for it. Its start value comes from the count of the tracks: 1 to 3 tracks a single, 4 to 6 an EP, 7 or more an album.
    - A release with no artist page gets a finding on the tab **Check**, and Review also stops its write.
    - ADR 0073 owned the rows of the page "Add a Feed" and the menu "Type" of its form. It had three "Changed by" lines, which is the limit. The operator selected its consolidation, and [ADR 0212](../adr/0212-the-add-pages.md) is that consolidation.
    - Operator choice: the row "Album" leaves the page "Add a Feed". A release starts only on the page of an artist.
    - Operator choice: a show feed and a playlist do not need an artist page. They need a publisher feed: a hosting page, a label page or an artist page.
- Operator choice for text 5: the button of the confirm of **Delete Now…** is **Review the Deletion**. The click already opens Review.
- Operator choice for text 7: when a move copies no file and no old file stays, the plan of the move sheet has the row "Old files" with "None. No waiting time." Defect 2 needs a measurement in WebKitGTK.

Runbook text that is out of date: `docs/runbooks/operator-human-checks.md` says 7 days for a move, the button **Use** for a suggestion, and the return warning below the move row.

These checks stay open after this walk: check 6, items 1 and 3 of Check E, and step 9 of the notes of 0.1.0-rc.22. Step 9 needs the next candidate on the server.

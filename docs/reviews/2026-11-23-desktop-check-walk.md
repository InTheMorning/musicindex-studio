# Desktop Check Walk Of 2026-11-23

This record is advisory. It states no rule. Each step keeps its owner, which is the file or the ADR in its line.

The walk collects the open desktop steps before the push of the commits after `v0.1.0-rc.32`. The operator runs the release binary `target/release/musicindex-studio-desktop`, built from `847d17a`, on the real collection. The schema version stays 52, so the installed rc.32 app can open the collection after the walk.

The screenshot checks of the main session proved the screens of ADR 0250, ADR 0251 and finding 11. Thus the walk gives only the steps that need real file hosts, real writes, Music Index or the installed app.

## Steps

| Step | Check | Owner | Result |
| --- | --- | --- | --- |
| 1 | Studio opens the real collection with no error | rc.32 checks, step 1 | Passed |
| 2 | Review of five releases shows no GUID panel | rc.32 checks, step 2 | Passed |
| 3 | **Issues** of the Elijah Lied releases shows no changed GUID | rc.32 checks, step 3 | Passed |
| 4 | One review writes an artist page and a linked written release | ADR 0251 | Passed |
| 5 | **Move Artists Here…** opens one review, and **Discard** keeps the links | rc.32 checks, step 4 | Passed |
| 6 | A test hosting page shows Labels and Artists | rc.31 checks, step 3 | Passed |
| 7 | **Move To…** of a test release, then a discard | rc.31 checks, step 4 | Passed |
| 8 | **Move To…** of a test artist page below a label page | rc.31 checks, step 5 | Passed |
| 9 | A move plan with old copies on a different host | rc.30 checks, step 4 | Passed |
| 10 | **Send the Podping Again** stays hidden during the wait | rc.30 checks, step 5 | Passed |
| 11 | A deleted test release shows **Put the File Back** and **Check Again** | rc.30 checks, step 6 | Passed |
| 12 | **Remove from Collection** of a test release has no error | rc.30 checks, step 7 | Passed |
| 13 | **Find Other Feeds** on a real file host | Finding 2 of 2026-11-23 | Passed |
| 14 | The installed app: KeePassXC shows the program path | rc.30 checks, step 8 | Skipped by the operator, stays open |
| 15 | The installed app: no text shows through the toolbar | Older tests, step D1 | Passed |
| 16 | The installed app: a drop of files on the sheets | Older tests, step D2 | Passed on **Add tracks** |

Step D3 of the older tests uses the removed "Start from" flow, so it is not in the walk. Step D4 and items 1 and 3 of Check E need a second app or a person page on Music Index. They stay open, and the walk does not include them.

## Results

### Step 3: The Issues Of An Elijah Lied Release

Result: passed. No release showed "The GUID changed outside Studio".

Observation 1: the release has no file host. **Issues** shows three findings:

- Stop: "No artist page. This release has no artist page. Studio cannot write it before the link."
- Warning: "A link to Elijah Lied has an old GUID". The link names `0c00c530-1a0a-54d2-a9fc-ed6d50e95a2b`, and the feed Elijah Lied has `83685084-baf9-5932-ba75-08453d6eedc1`. This is the Warning `link_guid_old` of ADR 0247 item 7.
- Notice: "No self link".

The Stop "No artist page" shows together with a link to the artist page Elijah Lied. The cause is possibly the old GUID: the link does not match the page, so Studio finds no artist page (inference, not examined in the code). The Stop also shows on a feed that Studio does not write, because it has no file host. Examine the two findings after the walk.

### Step 4: A Removed Link From The Release Side

The test album is already linked to the artist page HeyCitizen. Thus the operator removed the artist from the release and selected **Review**.

Result: the locked row is correct. It reads "Also write HeyCitizen" and "Removes the link to HeyCitizen's Test Album from the artist page." The write is not possible, because the Stop "This release has no artist page." shows at the top. This Stop is correct by ADR 0213 item 18: a release with tracks needs an artist page.

Observation 2: the same rule can stop a removal from the page side. **Remove** on a release row of an artist page removes the link. Review of the page then writes the release first (ADR 0251 item 7). When this page is the only artist page of the release, the server refuses the write of the release with `no_artist_page` (`crates/api/src/routes.rs`, near `feed_page_refusal`). A failed write of a release writes nothing after it, so the page is not written either. Before ADR 0251, the page was written alone. Whether the page Review shows the Stop before the write is not examined (inference from the code, not seen on the screen). Examine this after the walk. A test in the harness can show it.

Observation 3 (improvement): the box "This release has no artist page." offers only the pages that exist. When no artist page fits, the user has no path from the box to make a new artist page. Give the box an action that makes an artist page and links it.

### Step 4: The Link Back From Review

Result: **Link and Continue** with HeyCitizen removed the Stop. The row "Also write HeyCitizen" did not show in the screenshot.

Observation 4: Review of the release still shows a change of the link. The written file has `<podcast:remoteItem medium="publisher" feedGuid=… feedUrl=… title="HeyCitizen" rel="artist" />`. The draft has the same attributes in a different order: `feedGuid`, `feedUrl`, `medium`, `title`, `rel`. The link is the same, but Review shows a removed line and an added line, and the write is possible. Studio possibly makes a new element at the link and does not keep the earlier element (inference, not examined in the code). Examine this after the walk.

### Step 4: The Artist Page After The Discard

The discard of the release draft did not change the draft of HeyCitizen. HeyCitizen kept a pencil mark.

Observation 5: the removal and the link back moved the test album to the end of the releases of HeyCitizen. Review of HeyCitizen shows the test album and "check 212" in changed positions. The discard of the release draft does not put back the draft of the page. Examine whether a link back must keep the earlier position of the release on the page.

Review of HeyCitizen also shows the locked row "Also write check 212" with "Adds HeyCitizen as the artist or label of check 212." The written file of HeyCitizen already names check 212. The draft of check 212 names HeyCitizen, and its written file does not. This is the linked feed of ADR 0251 item 2, so the row is correct.

### Step 4: One Write Of HeyCitizen And check 212

The operator put the test album back in its position and selected **Review** again. One write from HeyCitizen wrote check 212 and HeyCitizen with no error.

Result: passed.

The operator does not accept that a removal and a link back change the file (observations 4 and 5). A link back must give the file that the user had before. The main session agrees: correct observations 4 and 5 after the walk.

### Step 5: Move Artists Here

Result, first part: **Move Artists Here…** with HeyCitizen opened one review, "Move HeyCitizen to Test Hosting", with **Write 5 Feeds**. Each release of HeyCitizen gets a `podcast:remoteItem` with `rel="hosting"` for Test Hosting.

Observation 6 (improvement): the sheet "Make a new artist page" has **Find on Music Index**, but the search does not show the artist pages that are already in Studio. On a hosting page or a label page, the user must then know that an existing page goes below the level with **Move Artists Here…**. The sheet can show the artist pages of the collection and move the selected page.

Observation 7 (improvement): the body of a hosting page has the row "Find an artist that is online" ("Link an artist page that exists already."). Each add sheet starts with **Find on Music Index**, so this row is possibly not necessary. The operator recommends removing it. Examine whether another flow needs it before the removal.

Observation 4 also applies to a new link: the new element has `medium`, `feedGuid`, `feedUrl`, `rel`, `title`. Other elements in the same file have `title` before `rel`. One order for each element of a file is easier to read in Review.

### Step 7: Before The Release Move

Observation 8 (design): the left panel does not show the levels Hosting, Label and Artist as one hierarchy. The root panel lists "Hosting" and "Artists" as separate groups, and an open level shows only one level at a time. This needs a design with mockups before a decision.

Observation 9 (design): in the left panel, each release row has **⋯** with **Move To…**. The head of an open artist page and the artist row of the root panel have no **⋯**. The move of an artist page is only in the head of the page in the main pane (`crates/api/ui/app.js`, `canMovePage`). The operator could not find it. The moves also have three entry points with different scopes: **Move To…** of a release, **Move To…** of a page, which changes the links of each release, and **Move Artists Here…** of a level. This needs a design with mockups before a decision.

Observation 9 also applies to the sheet of **Move To…**: it uses many select menus, one for each role. The operator finds this difficult. The design of the moves includes this sheet.

### Step 7: The Release Move

Result, first part: the move of HeyCitizen's Test Album to Test Hosting opened one review with **Write 2 Feeds**. The release gets a `podcast:remoteItem` for Test Hosting with `rel="hosting"`, and its link to HeyCitizen stays.

Observation 10 (defect): the title of the review is "Move HeyCitizen's Test Album to HeyCitizen". The move adds only Test Hosting, so the title names the wrong page. The title possibly uses the first linked page and not the changed page (inference, not examined in the code).

Observation 11: while the change set is open, the root panel shows only the group "Hosting" with Test Hosting. The group "Artists" with HeyCitizen is not shown. The release link to Test Hosting possibly puts HeyCitizen below Test Hosting before the write (inference, not examined in the code). Examine whether the panel must show a level from a draft that is not written.

### Step 8: The Artist Page Move

Result, first part: the review has **Write 2 Feeds**: The Heycitizen Experience and test label. The release gets a link to test label with `rel="label"`.

Observation 4 also applies here: the move rewrites the existing artist link of the release. The written element has no `title`. The draft element adds `title="HeyCitizen"` at the end. The link does not change, but Review shows it as a change.

Open question: a move of the artist page changes the links of each release of the page (ADR 0240 items 20 to 25). HeyCitizen has four releases, but the review writes only one release. The source of the move (the head of the page or the row of a release) is not known yet.

Answer: the operator used **Move To…** of the release row in the left panel, not the head of the page. Thus the review with one release is correct.

Operator view (design input for observation 9): an artist can have releases that are not on the label. A move of an artist page below a label must therefore not always change each release of the artist. The design of the moves and of the page feed links must include this case. Today a move of an artist page changes the links of each release (ADR 0240 items 20 to 25).

### Step 9: The Plan With Old Copies

Result: passed. **Move This Release…** of the test album to CloudFlare showed "Feed file: bastone → CloudFlare", "7 files: Already on CloudFlare", the check box "Delete the files on bastone when the move is complete" and the menu "Keep the old files for". The operator selected **Cancel**.

### Step 11: A Deleted Release

Result: passed. **Delete from File Host…** of check 212 deleted `check-212.xml` on bastone. The notice "The feed file of this release is deleted from its file host." with **Put the File Back** and **Check Again** showed below the head on the tab **Issues** and on the tab **Settings**. **Put the File Back** opened a review of the last written file, and its write removed the notice.

### Step 13: Find Other Feeds

Result, first part: **Find Other Feeds** opened the list on bastone and on CloudFlare. The button operates after the first connection.

Observation 12 (not a defect): after step 12, the left panel shows HeyCitizen with 3 releases, and **Find Other Feeds** of bastone lists check 212 (`check-212.xml`). The operator did not select **Undo** in step 12. Thus check 212 was not in the collection, and the list is correct. Step 12 asks only for a removal with no error, so it stays passed.

Observation 13 (possible defect): **Find Other Feeds** of CloudFlare lists HeyCitizen's Test Album at `TestAlbum/test.xml` as a feed that is not in Studio. The collection has this release on bastone. The file on CloudFlare is possibly an old copy or the pointer file of an earlier move, with the same GUID (inference, not examined on the host). The code confirms the cause of the list: `connector_feeds` in `crates/api/src/routes.rs` sets `in_collection` only when a feed of the same file host has the same path. It does not compare the GUID, as `in_collection` of the discover route does. **Add 1 to Studio** can then make a second copy of the release. The operator did not add it on the real collection. Examine the add in the harness after the walk.

Result, last part: passed. **Add 1 to Studio** on bastone gave back check 212 below HeyCitizen, and the list then said "Each feed in this folder is in Studio."

### Steps 14 To 16: The Installed App

Step 14 was skipped by the operator and stays open. Step 15 passed: no text showed through the top bar during a scroll. Step 16 passed on **Add tracks** of check 212: a drop of two audio files gave two rows, and a drop of an audio file and a text file gave an error for the text file. The drops on **From MSP 2.0 backups** and **Import an account** did not run.

## Summary

Steps 1 to 13, 15 and 16 passed. Step 14 stays open.

Defects to correct:

- Observation 2: a removal from the page side can stop at the release write (to confirm in the harness).
- Observations 4 and 5: a removal and a link back change the attribute order and the position of the release.
- Observation 10: the title of a release move names the wrong page.
- Observation 13: **Find Other Feeds** does not compare the GUID.

Observation 1 needs an examination of the two findings. Observations 3, 6, 7, 8, 9 and 11 need a design with mockups.

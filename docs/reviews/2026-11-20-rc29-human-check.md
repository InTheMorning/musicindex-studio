# Human Check Of 0.1.0-rc.29 On 2026-11-20

The operator does the steps of "What to test" in the [notes of 0.1.0-rc.29](../releases/0.1.0-rc.29.md). The version commit is `6b54736`.

## Agent checks before the human check

The main session drove the release binary of `6b54736` on a private Xvfb with a copy of the operator data (`.studio-dev/tauri-run.sh`, results in `.studio-dev/d29/r1` to `r12`).

| Check | Result |
| --- | --- |
| rc.29 step 1 | Pass. The upgrade went from schema 44 to 45. Releases and People showed all feeds and persons. |
| rc.29 step 2 | Pass. "Matt Finlay is deleted." stayed for more than 15 s. **Undo** brought back the same rows. |
| Undo of the menu **Kind** | Pass. Change, **Undo**, **Redo** and **Undo** each showed the correct value. |
| Discard of a new live show (rc.23 step 19) | Open. The add of a live show passed. The review sheet asks for the vault passphrase, and the copy has a locked vault. After **Discard the Draft**, the sheet asked for the vault again, so the page after the discard did not show (`.studio-dev/d29/r20`). |
| Person head | Fail, then fixed in `f68c89c`. A person with only a read-only link kept the button "Find on Music Index" (ADR 0228 item 18). |

## Results

| Step | Result |
| --- | --- |
| 3. A write with a podping, then **Dismiss** | Pass. Findings 1 and 2 came during this step. |
| 4. Delete the leftover copy of the feed of Nathan Abbott | Pass. Studio offered the delete and deleted the files. The operator did not report the text of the sheet. Finding 3 came from this step. |
| 5. Write the test release again with the return to the earlier address | Pass. The write did not stop at `TestAlbum/test/cloned.mp3`. |
| 6. Delete and put back the file of a test feed (rc.23 steps 6 and 7) | Open. The operator skipped it. |
| 7a. A return between two real file hosts (rc.23 step 13) | Pass. The operator moved the test release from "bastone" (`files.heycitizen.xyz`) to "heycitizen" (`music.cancelbandcamp.com`) with **Move This Release…**, then back to the earlier address on "bastone". After the return, the tab **Files** shows the feed file and the media files on "bastone", and "Old files to delete" shows 7 files on "heycitizen". **Move It Back…** shows only after a move outside Studio, so it needs rc.23 step 12 first. |
| 7b. The discard of a new live show (rc.23 step 19) | Pass. After the discard, the page showed the tab **Live Shows** with no live show, and the top bar agreed with the body. |
| 7c. The tab **Issues** with real Music Index data (rc.23 step 19) | Pass. The Music Index lines of "HeyCitizen's Test Album" agreed with musicindex.org. |

## Findings

1. The tab **Files** of "HeyCitizen's Test Album" shows the feed file on the host "bastone" and the media files on the host "heycitizen". The operator says that all the files are on "bastone", and that the feed shows this. The cause is not examined.

2. The waiting notice after a write shows **Send the Podping Again** adjacent to "Sent · {time}". The operator says that this button is not necessary when Studio sent the podping with no error. This also gives the cause of finding 4 of the rc.28 check: the button is on the waiting notice after a podping that Studio sent with no error.

3. After the delete of the leftover copy, the folder `Songs/Albums/nathan-abbott-everything/` stays on the file host with no file in it. Studio deletes the files but does not remove the empty folder.

The desktop part of the check ended after step 7c. These steps stay open: rc.23 step 12 (it needs the server), items 1 and 3 of Check E of the walk of 2026-11-14 (they need a new picture on a Music Index page of a read-only link), steps 6 to 8 of the notes of rc.29, and the person head fix `f68c89c` in the next candidate.

## Desktop Check Of The Fixes After rc.29

The operator ran the release binary of main at `5c763de` on the real data.

| Step | Result |
| --- | --- |
| 1. Start the binary on the real data | Pass. |
| 2. Delete the test release "new Feed" with its media files on "bastone" | Pass. The folder `new-feed/` is gone. Step 2 replaced the move of the test album because of finding 9. |
| rc.23 step 6, the sheet of **Delete from File Host…** | Pass, from the screenshots of the operator. The sheet showed the three goals and the line "Keep the domain files.heycitizen.xyz." |
| 3. rc.23 step 7, **Put the File Back** after the delete of step 2 | Fail. The button is not on the page. See findings 10 and 12. |

### Findings Of The Check After rc.29

The operator changed the name of the file host "heycitizen" to "CloudFlare". It is the S3 host at `music.cancelbandcamp.com`. Thus "CloudFlare" in these findings is the host "heycitizen" of the rc.29 check. The media files of the two releases are on "bastone", not on this host.

1. On a trusted computer, the vault passphrase must not be necessary each time the desktop app starts. Use a standard method, for example the keyring of the system. The cause and the design are not examined.

2. The artist page has the button **Open the Feed**. The operator wants the label **Artist Page**.

3. The page **Releases** has no **+ Add a Release** at its head. The artist page has this button.

4. In the sheet "Make a new release", **Start from** gives the releases that Studio tracks. The operator wants the tool "Find on Music Index" there, to fill the fields from a Music Index entry.

5. An item in the left panel with a dotted circle ("not checked") does not start a check when the operator clicks it. The operator wants the click to do the check and correct the status.

6. A new release "new Feed" of the kind Single on "bastone": the tab **Files** shows "Media files: CloudFlare". The library file shows its copy on "bastone" at `https://files.heycitizen.xyz/Songs/Albums/new-feed/…`. This can be the same fault as finding 1 of the rc.29 check, on the path of a new release. The cause is not examined.

7. On the same feed, immediately after a write and a podping, the sheet "What do you want to do?" shows **Remove the file** with the line "Music Index removed the entry". The delete sheet also shows "✓ Music Index removed the entry". The feed is new, so it is possible that Music Index did not read it at that time. The cause is not examined.

8. **Delete from File Host** told the operator about a draft with changes. The operator thinks that the draft was a move from before. The operator discarded the draft, and the next try told about the draft again. The third try opened the delete sheet.

9. Step 2 stopped before **Add to the Draft**. On "HeyCitizen's Test Album", the tab **Files** shows "Media files: CloudFlare", and the move sheet to "CloudFlare" shows "7 files: Already on CloudFlare" and "Old files: None". Thus the sheet has no check box for the delete on "bastone".

   After step 7a of the rc.29 check, the feed and the media files were on "bastone". Inference, not examined: `move_release` in `crates/api/src/routes.rs` sets the media host at **Add to the Draft**. A discard of the draft possibly does not put back the earlier value. This can also be the cause of finding 6.

10. After the delete of "new Feed" in step 2, the head of the feed shows "A connected file host gives the address of this feed. Use it to edit the feed here." with **Use This File Host**. The tab **Files** shows "Feed file: Not on a file host" and "Media files: CloudFlare". The delete sheet tells the operator to use **Put the File Back**, but that button is not near the head. It is in the section **Feed** at the bottom of the tab **Release**. The operator did not find it.

11. After the delete in step 2, **Library files** of "new Feed" shows "Richard Stallman Free software Song 2.ogg" and "stallman-disappointed 2.webp" as "In use · Not in the library". It also shows "Richard Stallman Free software Song.ogg" as "Not in use · 0 copies". Before the delete, the tab showed "Richard Stallman Free software Song.ogg" on "bastone". The source of the names with " 2" is not examined.

12. After the delete in step 2, the section **Feed** of the tab **Release** of "new Feed" has only **Feed facts** and **Remove from Collection**. It has no block "Retire" and no **Put the File Back**. Thus the operator cannot put the deleted feed file back. The delete sheet said that this was possible. rc.23 step 7 fails.

## Desktop Check Of Tasks 205 To 209

The operator ran the release binary of main at `da76cb0` on the real data.

| Step | Result |
| --- | --- |
| 1. Start the binary on the real data | Pass. |
| 2. The Retire block of "new Feed" (task 206) | Pass. The section **Feed** of the tab **Release** shows **Put the File Back** and **Check Again**. It does not show **Stop Retiring** or **Use This File Host**. |
| 3. **Put the File Back** on "new Feed" (task 206) | Pass. Review shows a usual write of the full file. It does not show a move of a read-only feed. |
| 4. Write the file back (ADR 0230 item 30) | Pass. The head has no "READ-ONLY" label, and the section **Feed** has no Retire block. The media files stay deleted from the delete in the earlier check. |
| 5. The rows with " 2" on the tab **Files** (ADR 0230 item 32) | Fail. After the write, **Library files** still shows "Richard Stallman Free software Song 2.ogg" and "stallman-disappointed 2.webp" as "In use · Not in the library". The rows without " 2" show "Not in use · 0 copies". Refer to finding 4. |
| 6. A click on the box "new Feed" with the dotted circle (task 209) | Fail. The circle stays dotted. The public address gives HTTP 200, so a check can read the file. Refer to findings 5 and 6. |
| 7. The data repair of "new Feed": **Move This Release…** with "Media files only" to "bastone" (ADR 0230 item 31) | Stopped at the plan. The plan sends 1 audio file (1.9 MB) and 1 image (0 B) to "bastone", and says "2 files not in the library: Studio downloads them first". The delete of the earlier check removed these files from "bastone". The operator cancelled the sheet. Refer to finding 7. |
| 8. The plan of the data repair of "HeyCitizen's Test Album" (ADR 0230 item 31) | Pass. The plan shows "7 files: Already on bastone" and "Old files: None". |
| 9. **Add to the Draft** and the write of the repair (task 205) | Pass. After the write, **Media files** shows "bastone" and the head shows "No changes". |
| 10. **Find on Music Index** in the form of a new release (task 208) | Pass. The search "heyc" gives the artist row with "This row is not a release" and six results. A click on "The Heycitizen Experience" fills **Title** and **Artist name**. The parent card stays HeyCitizen. The operator closed the form with no release made. |

Findings:

1. The left panel shows the dotted circle ("Not checked") on "new Feed". The circle stays after a click on the item. The feed has no file after the delete, so Studio cannot check it. Thus the mark cannot go. Inference, not examined: `stateMark` in `crates/api/ui/outsidecheck.js` counts a feed with a done delete as a feed that is not checked.

2. The Retire block is in the section **Feed** at the bottom of the tab **Release**. The operator had to scroll to find it. Finding 10 of the earlier check gave the same problem, and ADR 0230 did not move the block.

3. On the page Releases, the row "HeyCitizen" in the left panel shows the pencil mark ("Changes not written"). Each release in the list shows "No changes". The cause is not examined.

4. The rows with " 2" of finding 11 of the earlier check stay after **Put the File Back** and its write. ADR 0230 item 32 expected them to go. Inference, not examined: the binary of `5c763de` made the rows at a start after the delete. Task 206 stops new rows of this kind, but no step removes the rows that exist.

5. On a release page, the head box of the left panel is drawn by `Outline` in `crates/api/ui/app.js` near line 3455. Its click opens the page and starts no check. Task 209 left it out, but ADR 0230 item 17 covers each row with the mark "Not checked".

6. The mark of "new Feed" comes from the check at the start of Studio, when the file was deleted. The write of step 4 did not start a new check. Thus the mark "Not checked" stayed after the file was on the host again.

7. The move plan of "new Feed" offers to download and send 2 media files that are on no host. The plan does not say that the files are gone. ADR 0230 item 31 expected that the repair copies no file. That is true only when the files are on the target host.

8. In the sheet **Find on Music Index** of a new release, the group title is "FEEDS" and each result row says "Feed · {artist}". The word "release" is clearer in this sheet. Refer to the operator rule about the word "feed" in labels.

## Desktop Check Of The Commits Up To 2d4fb25

The operator installed the package `0.1.0rc.29.r1028.g2d4fb25-1` on 2026-11-20.

| Step | Result |
| --- | --- |
| 1. Build and install the package | Pass. |
| 2a. The change that HeyCitizen had before the check | The only change was the link to "new Feed" and the build date. This is the change of finding 3 of the check of tasks 205 to 209. The sort menu shows "Page Order". |
| 2b. **Discard the Draft** on HeyCitizen | Pass. HeyCitizen has no pencil mark and shows "No changes". |
| 2c. Make the release "check 212" on bastone from the HeyCitizen level | Pass. The create gave no error, and HeyCitizen had no draft before it (ADR 0231 item 11). Studio then opened **Add tracks**. |
| 2d. Review of "check 212" | Pass. Review shows the check box "Also write HeyCitizen", on, with the note and the change of HeyCitizen (ADR 0231 item 7). |
| 2e. **Write to the File Host** with "Also write HeyCitizen" on and no podping | Pass. "check 212" shows "No changes". HeyCitizen has no pencil mark in the left panel, and its count is 5 with "check 212" (ADR 0231 items 8 and 10, finding 3 of the check of tasks 205 to 209). |
| 3. The tab **Files** of "new Feed" (task 213) | Pass. **Library files** has two rows, the OGG file and the WEBP file, each "In use · 0 copies". No name ends in " 2" (ADR 0231 items 12 to 14). |
| 4. The move plan of "new Feed" with "Media files only" to bastone (task 213) | Pass. The plan gives "1 audio file: The library → bastone · 1.9 MB", "1 image: The library → bastone · 56.4 KB" and "2 files: Not on a file host. Studio sends it from the library." It offers no download (ADR 0231 items 15 and 16). |
| 5. The repair of "new Feed": **Add to the Draft** and the write, no podping (ADR 0231 item 17) | Pass. **Media files** says "bastone". Each library file says "In use · 1 copy". The head says "No changes". |
| 6. The dotted circle of "new Feed" (task 210, step 6 of the check of tasks 205 to 209) | Pass. The screenshots of steps 3 and 5 show the head box of "new Feed" with no dotted circle, and the left panel of the page Releases shows no mark "Not checked". |

Findings:

1. On the artist page, the group "Releases" of the left panel has no action that makes a release. The operator wants **＋ Add release** there. A release page has **＋ Add tracks** at the same place.

   The operator wants the same rule at each level of the left panel: a group of the left panel has an add action for the items of that group. A release page has **＋ Add tracks** below its tracks now. Examine each level of the left panel and list the groups that have no add action.

2. On the page of the new release "check 212", the back row of the left panel says **‹ Releases**. The release has the parent HeyCitizen, and the artist page shows **‹ HeyCitizen**. Not examined: which level the back row must open from a new release.

3. After the walk, **Remove from Collection** on "new Feed" gave the error "Studio could not finish this." with the details "deleteUndo.clear is not a function". Cause, examined in the code: `FeedFacts` in `crates/api/ui/editor.js` near line 2559 calls `deleteUndo.clear()` when the remove is not undoable. `DeleteUndoProvider` in `crates/api/ui/undonotice.js` near line 86 gives only `model`, `record`, `undo` and `redo`, with no `clear`. `crates/api/ui/app.js` near line 2112 and `crates/api/ui/peoplepages.js` near line 1106 make the same call. Task 198 (`e9f4434`) made this fault. Inference, not examined: the remove route ran before the error, so the feed can be gone from the collection.

4. On the same screen, the Retire block of "new Feed" shows "Delete the file: The file is deleted." This is correct: after step 5, the operator did **Delete from File Host…** on "new Feed", then **Remove from Collection**. After the error, Studio opened the page Releases by itself. "new Feed" was gone, and HeyCitizen shows 4 releases. Thus the remove was done.

5. Found in the main session check of the fix of finding 3. On a collection where `migrate::into_library` succeeded, **Remove from Collection** fails with HTTP 500 and "no such table: draft_files". That move drops the old table `draft_files`. `feed_has_pending_removal` and `FEED_DELETE_TABLES` in `crates/core/src/people.rs` still read it (`f81e055`, ADR 0227). The real collection of the operator still has the table, so the remove worked there.

Operator choice (2026-11-20, main session): findings 3 and 5 are defect corrections with no new decision. `DeleteUndoProvider` gives each action of `DELETE_UNDO_ACTIONS`, and a node test checks each `deleteUndo` call against that list. The remove and its undo skip a table that is not in the collection, and a core test proves a remove and an undo after the drop of `draft_files`.

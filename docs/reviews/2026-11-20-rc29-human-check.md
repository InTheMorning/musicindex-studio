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

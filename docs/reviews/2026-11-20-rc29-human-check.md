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

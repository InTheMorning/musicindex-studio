# Human Check Of 0.1.0-rc.28 On 2026-11-20

The operator does the steps of "What to test" in the [notes of 0.1.0-rc.28](../releases/0.1.0-rc.28.md). The tag `v0.1.0-rc.28` is on `5803f67`.

## Results

| Step | Result |
| --- | --- |
| 1. Upgrade to schema version 45 on the real collection | Pass. Releases and People showed all feeds and persons, and Settings showed the rc.28 version line. The operator did not name the package. |
| 2. Ctrl+Z and Ctrl+Shift+Z in the title field of the desktop app | Pass. Finding 1 came from this step. |
| 3. Undo after Delete Person | Pass. The person came back with the same data. Finding 2 came from this step. |
| 4. The start folder of "Choose Folder…" | Pass. Finding 3 came during this step. |
| 5. "Open on musicindex.org" in the desktop app | Pass. One click opened the release in the browser. |
| 6. A write with a podping | Pass. The notice changed to the confirmation. Finding 4 came from this step. |
| 7. The icon of the type panel in the desktop app | Pass. The icon was small. |
| 8. The Arch package and the program `musicindex-studio-desktop` | Pass. The menu started one window with one task bar icon, and the program is in `/usr/bin`. |
| 9. The person page on the iPhone | Pass. The four tabs showed as the menu "Profile", and the bar showed **Undo** and **Redo**. Findings 5 and 6 came from this step. |
| 10. Back and Forward after the title on the iPhone | Pass. Finding 7 came from this step. |
| 11. The icon of the type panel in Safari | Pass. Finding 8 came during this step. |
| 12. Delete and put back the file of a test feed (rc.23 step 7) | Open. The operator skipped it. The operator first tried to delete a leftover feed file, and Studio did not offer the delete (finding 9). |
| 13a. "Follow the Move" and "Move It Back…" (rc.23 steps 12 and 13) | Open. A return of the test release on the file host "heycitizen" stopped the write (finding 10). |
| 13b. The open items of rc.23 step 19 | Open. Each item needs a special setup, for example a person with a read-only link to a Music Index page that is not in the collection. |
| 14. The Tab order of the narrow bar (rc.24 step 11) | Pass. Finding 11 came during this step. |

The operator ended the check after step 14. Step 15 of the notes stays open: `docs/reviews/2026-11-14-open-older-human-tests.md` has the open tests of Phase 12 and of tasks 049 to 093.

## Findings

1. The buttons **Undo** and **Redo** move to the left or right when the draft status at their right changes its text. The cause is not examined. A possible cause (inference): the buttons and the status are in one row at the right of the bar. Thus the width of the status text sets the position of the buttons.
2. The operator saw the **Undo** of the notice "{name} is deleted." only after the 10 seconds. The operator expected **Undo** at the top right, where the bar has **Undo** and **Redo**. The notice is at the bottom center of the window (`.undo-notice` in `styles.css`, `position: fixed`, `bottom: 16px`).

   Operator direction: the people operations use the same undo and redo as the rest of the app. People data must not look different from feed data. Open: the notice also serves **Remove from Collection** and the menu **Kind** of a release (ADR 0227 item 13). A decision must say if those two also move to the history of the bar. This changes ADR 0227, so it needs a decision record.
3. The page of an artist shows the artist row, and then the release rows of the artist at the same indent. The operator wants the release rows indented, so that they show as parts of the artist. Screenshot: the page "HeyCitizen" with the group "Artist" and three releases.
4. After the confirmation "Music Index has your write of {time}.", the notice still shows **Send the Podping Again**. The operator finds no use for the button at that time. After a time limit, the notice becomes a smaller row. ADR 0191 item 4 gives the button to each sent line (`podpingLines` in `outsidecheck.js`).

   Operator proposal: the notice keeps the same height, and it disappears only when the user selects a **Dismiss** button. This changes ADR 0191, so it needs a decision record.
5. The Nostr line of the person head shows "Nostr Sir HeyCitizen · Added by you". The operator does not see a use for "Added by you". The text is the evidence of the Nostr key (`keyEvidenceText` in `people.js`): a key that a lookup or a Nostr profile did not find gets "Added by you". Screenshot: the person "HeyCitizen" on the iPhone.
6. The person head shows the large button **Find on Music Index** also when the person is already linked to an artist page. The operator finds the button too prominent for that person. Screenshot: the same person, with the link "HeyCitizen · Artist" in the role line.
7. The button **Sections** at the top right of a release page opens the outline of the release. The operator wants a different name and position for it: "Overview" or "‹ Overview", probably at the left of the tab menu ("Release" in the screenshot). The position is an interpretation of the operator words, and the drawings must confirm it. Screenshot: "HeyCitizen's Lo-Fi Hip-Hop Beats to Study and Relax to" on the iPhone.

   The status "Saved, not written" of that release in the screenshot comes from changes of the operator during the check. It is not a defect.
8. The tab of the findings of a release has the name "Check {n}", for example "Check 3" (`tabLabel` in `feedcheck.js`, ADR 0174 item 17). The operator wants a better name, and the number in parentheses, for example "(3)".
9. The operator tried to delete a leftover feed file of "Nathan Abbott" at `https://files.heycitizen.xyz/Songs/Albums/nathan-abbott-everything/nathan-abbott-everything.xml`. The sheet "What do you want to do?" showed only "Put the feed on a different file host" and "Take the feed out of the apps". The page said: "Music Index reads this feed from a different address." The different address is `https://headstarts.uk/msp/publisher-feeds/nathan-abbott-everything.xml`.

   Cause: the row "Remove the file" shows only when `removal_done` is true (ADR 0200 item 4), and `read_removal` in `crates/core/src/feed_retire.rs` reads Music Index by GUID only. Music Index holds the GUID with the other address, so the answer is `NotYet`, and the delete never becomes available. `ensure_index_removed` also refuses the delete in the core (ADR 0200 item 6).

   The record of Music Index (`IndexFeed`) has its address, but the rule does not compare it with the address of this file. A change needs a decision record, because ADR 0200 items 4 and 6 give the rule. Screenshots: the delete sheet and the page notice on the iPhone.
10. Defect: a write of the test release at `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml` stopped with "Studio could not write the release." The fold "Details" gave "a file exists at TestAlbum/test/cloned.mp3 already". Review showed the notice "This is a return to an earlier address."

   The text comes from the upload step of the write (`pipeline.rs`, the upload of a library file with `Expected::Absent`). A possible cause (inference): the return puts the files on the earlier file host again. The files of the earlier write are still there, and the upload does not write over a file. The cause is not examined. Screenshot: the Review sheet in the desktop app.

   The tab **Files** of the release shows the pending move: "The feed moves to heycitizen when you write, at `https://music.cancelbandcamp.com/TestAlbum/test.xml`." The address from before, `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml`, gets a pointer file. The feed file and the media files both use the file host "heycitizen". Thus the return goes to an earlier address on the same file host, and the path `TestAlbum/test/cloned.mp3` agrees with that earlier address. Screenshot: the tab **Files** in the desktop app.

   Result: the cause is confirmed. The file on the host at `https://music.cancelbandcamp.com/TestAlbum/test/cloned.mp3` has 2168204 bytes and the SHA-256 `4fe821996393117519e65759b44c9a1670029bd715c5a8efa434427658251b04`. The library copy of the operator has the same size and the same SHA-256. The plan (`planned_path_on` in `inventory.rs`) uses a file on the host only when the inventory has a record of it. The inventory of the target had no record of the copy from before, so the plan gave an upload, and the upload stopped at the file. Why the record is missing is not examined. A possible cause (inference): nobody scanned the target after the earlier move.

   Correction: when an upload meets a file at its path, the write compares the SHA-256 of that file with the library file. With the same SHA-256, the write records the file and continues (ADR 0034 item 2). With a different SHA-256, it refuses as before (ADR 0034 item 3). This is a defect correction and needs no decision record. Tests: `an_upload_uses_a_file_with_the_same_bytes_that_the_inventory_does_not_know` and `an_upload_does_not_replace_a_different_file_that_the_inventory_does_not_know` in `crates/core/tests/pipeline.rs`. Without the correction, the first test gives the error of the operator. Open: the operator does the return again with the next build.
11. The Review sheet of a release with many file operations is very long. The operator must scroll past the file operations to examine the changes of the XML. After **Write to the File Host**, an error shows at the top of the sheet, far from the button. Finding 10 is an example: the error notice was above the list of the file operations.

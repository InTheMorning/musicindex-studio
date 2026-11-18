# Walks

Each walk operated the desktop binary of commit `a64637d` (`studio-desktop`, release build) in headless
Chromium. The window was 1280 × 860 px, or 390 × 844 px for the narrow walk. No network was available.

Two data sets were used:

- **Fresh.** An empty data folder. The first-run pages show.
- **Base.** A copy of a test collection with 20 feeds and one local file host. Most feeds are read-only.
  The feed "Loose Single" is an album on the local host with no tracks and no artist page.

A count in a walk is the number of steps that the walk did. The step method is in `README.md`.

## J01 First run to the artist page editor

Data: fresh. Screenshots: `shots/j01-*.png`.

| Step | Action | Result | Evidence |
|---|---|---|---|
| 1 | Type the library folder in `"Folder"`. | The field holds the path. | Shot `j01-01-library` |
| 2 | Click `"Use This Folder"`. | `"Studio set up your library"` shows the counts of folders, feed files and moved files. | Shot `j01-02-library-result` |
| 3 | Click `"Continue"`. | The Welcome page shows. | Shot `j01-03-welcome` |
| 4 | Click `"I make music"`. | A sheet opens. Its stage list shows `"Your name"`, `"Where it lives"` and `"Your first album"`. | Shot `j01-04-name` |
| 5 | Type the artist name. | The name field holds the name. | Shot `j01-04-name` |
| 6 | Click `"Continue"`. | The stage `"Where it lives"` shows a local folder form with a default folder. | Shot `j01-05-where` |
| 7 | Click `"Continue"`. | The sheet closes. The artist page editor opens with the card `"Add your first album"`. | Shot `j01-06-editor` |

Results:

- The library stage takes 2 steps with the default folder and 3 steps with a typed folder.
- The library form has a text field and no folder picker button.
- Welcome to the artist page editor takes 4 steps.
- The sheet shows 3 stages, but it closes after the second stage. The third stage `"Your first album"` is a
  card in the editor. It is not a stage of the sheet.

## J02 First release with one audio file, then the write

Data: fresh, continued from J01. Screenshots: `shots/j02-*.png`.

| Step | Action | Result | Evidence |
|---|---|---|---|
| 1 | Click `"Add a Release"` in the card. | The sheet `"Make a new release"` opens with `"Kind"`, `"Album title"`, `"Artist"`, the `"Feed file"` disclosure and `"Make the Release"`. | Shot `j02-01-new-release` |
| 2 | Type `"First Album"` in `"Album title"`. | | |
| 3 | Click `"Make the Release"`. | The album page opens. The import sheet does not open. A notice says `"The file host has no file for this release yet."` | Shot `j02-02-album-editor` |
| 4 | Click `"Add tracks"`. | The sheet `"Add tracks"` opens. | Shot `j02-02b-import-sheet` |
| 5 | Select one MP3 file in the native file dialog. | One row shows the title, the track number `—`, the duration and `"Track ready"`. | Shot `j02-03-import-rows` |
| 6 | Click `"＋ Add 1 Track"`. | The track shows in the album and in the sidebar outline. | |
| 7 | Click `"Review"`. | `"Review changes"` lists the upload `first-album/01 Low Tide.mp3` and the field changes. | Shot `j02-05-review` |
| 8 | Click `"Write to the File Host"`. | Review closes. The toolbar shows `"No changes"`. `"Read Again"` shows. | Shot `j02-06-after-write` |

Results:

- From the empty card to a written album with one track: 8 steps.
- The album can be written with no tracks. The write button shows before the import.
- After the write, no notice tells that the write succeeded. Only the toolbar status changes.
- Review showed `"The build date does not change."` because no time server was available.
- The new release showed `"ALBUM"` before the import, and `"SINGLE"` after the import of one track.
  `"Kind"` was `"From the tracks"`. The first page tab also changed from `"Album"` to `"Single"`.
- The Simplified diff showed an internal value: `"Podcast GUID"` `none → pending-60384c23-…`.

## J03 Base collection, home page

Data: base. No screenshot is kept.

- The sidebar groups the feeds under `"Hosting"`, `"Labels"`, `"Artists"`, `"Producers"`, `"No role"`,
  `"Not linked"` and `"Other feeds"`. Each row shows a count.
- The release list shows each feed with its kind, its track count and `"Read-only"` or the host name.

## J04 Change one field of an album with no artist page, then write

Data: base. Screenshots: `shots/j04-*.png`, `shots/j04c-*.png`.

| Step | Action | Result | Evidence |
|---|---|---|---|
| 1 | Click `"Loose Single"` in the release list. | The album page opens. | |
| 2 | Type a text in `"Description"`. | The toolbar shows `"Saved, not written"`. | |
| 3 | Click `"Review"`. | `"This release has no artist page."` shows with `"Link a Page"`. The footer has `"Discard the Draft"` only. | Shot `j04-03-review` |
| 4 | Click `"Link a Page"`. | Review closes. The page scrolls to `"Linked artist and label pages"`. | Shot `j04c-01-after-link-a-page` |
| 5 | Select `"Artist X"` in `"Link to an artist or label page"`. | | |
| 6 | Click `"Link to This Page"`. | The link shows. A notice says `"Studio also changed the draft of Artist X. Open it to examine and write it."` with `"Open Artist X"`. | Shot `j04c-02-linked` |
| 7 | Click `"Review"`. | The write button shows. `"Send a podping"` is set. | |
| 8 | Click `"Write to the File Host"`. | The write failed in the walk. See the result below. | Shot `j04c-04-after-write` |

Results:

- With an artist page, the journey is 4 steps (steps 1, 2, 3 and 8). With no artist page, it is 8 steps.
- The artist page draft also changes. To write it, the user must do 3 more steps: `"Open Artist X"`,
  `"Review"` and `"Write to the File Host"`. **Inference:** the full journey is 11 steps.
- The Check page shows the same condition as `"Notice"` `"No artist page"`. Review shows it as an error that
  stops the write. Shot `j06-07-check`.
- The write failed because the test feed has the address `https://music.example/loose.xml` and the walk had no
  network. The cause is the test data. The message is a fact of the UI: the title is
  `"Something went wrong."`, and the text is the server message in lower case, with the proxy error.

**Correction to inventory B, section 16.** Inventory B says that the write button is disabled when a link
refusal shows. The walk shows that the button is not there. The code agrees: `hasWork` is false when
`pageRefusal` is set (`editor.js:2127`), and the button needs `hasWork` (`editor.js:2228`).

## J05 Narrow layout, open a release

Data: base. Width 390 px. Screenshots: `shots/j05-*.png`.

| Step | Action | Result | Evidence |
|---|---|---|---|
| 1 | Tap `"Loose Single"` in the release list. | The outline page shows the release row and `"Add tracks"`. | |
| 2 | Tap the release row in the outline. | The album page opens. A select menu replaces the page tabs. | Shot `j05-03-feed-page` |

Results:

- Release list to the editor: 2 steps. The wide layout needs 1 step.
- The page select menu has `"Album"`, `"Live Shows"`, `"Value Routes"`, `"Credits"`, `"Files"`, `"Check 2"`
  and `"Settings"`.
- The logo button (`"Destinations"`) opens a page with the full sidebar list. `"People"`, `"File hosts"` and
  `"Settings"` are at the end of that list, below all the feed groups. Shot `j05-01b-destinations`.

## J06 Add a value recipient

Data: base, feed "Loose Single". Screenshots: `shots/j06-*.png`.

| Step | Action | Result | Evidence |
|---|---|---|---|
| 1 | Click `"Value Routes"`. | `"This feed has no recipients."` and `"＋ Add recipients"`. | |
| 2 | Click `"＋ Add recipients"`. | The empty lists `"Recipients"` and `"Fees"` show. | |
| 3 | Click `"Add a recipient"`. | The sheet `"Add a Recipient"` opens. `"Type"` is `"Node Address"`. `"Payees with no person"` shows 74 rows, closed. | Shot `j06-03-add-sheet` |
| 4 | Type `artist@getalby.com` in `"Address"`. | | |
| 5 | Click `"Add the Recipient"`. | The recipient shows at 100.0 %. Its row says `"Node address"` and `artist@getalby.com`. `"Split (shares)"` is empty. | Shot `j06-04-added` |
| 6 | Type `1` in `"Split (shares)"`. | | |

Results:

- Add one recipient with a split, from the album page: 6 steps. Inventory B gives 6 for a feed with no
  payment block. The walk agrees.
- The sheet accepted a Lightning address with the type `"Node Address"`. The field hint says
  `"The public key of the Lightning node, 66 hexadecimal characters."` The code checks only that the address
  is not empty and is not in the list (`values.js:247`, `values.js:279`).
- Review and the Check page showed no finding for that address. Shots `j06-06-review`, `j06-07-check`.
- The Simplified diff in Review showed two rows, `"Payments"` with no detail and `"Payments"` `"Added"`.
- With an empty `"Split (shares)"`, the bar showed 100.0 %.

## J07 People: make two persons from suggestions, then merge them

Data: base. Screenshots: `shots/j07-*.png`. At the start, the collection has no person.
`"Suggestions from your feeds"` shows 32 suggestions, and `"Payees with no person"` shows 71 rows.

| Step | Action | Result | Evidence |
|---|---|---|---|
| 1 | Click `"People"`. | The People page opens. | |
| 2 | Open `"Suggestions from your feeds"`. | The list shows each suggestion with `"Check"`. | |
| 3 | Click `"The Doerfels"`. | The sheet `"Check: The Doerfels"` opens. `"Person"` is `"A new person"`. The sheet lists the credits in groups and the payment addresses, all selected. | Shot `j07-03-check-doerfels` |
| 4 | Click `"Make the Person"`. | The person page `"The Doerfels"` opens. The toolbar shows `"No changes"`. | |
| 5 | Click `"People"`, then open the suggestions again. | | |
| 6 | Click `"Ben Doerfel"`. | The sheet `"Check: Ben Doerfel"` opens. The menu `"Person"` now also has `"The Doerfels"`. | Shot `j07-05-check-ben` |
| 7 | Click `"Make the Person"`. | The person page `"Ben Doerfel"` opens. | |
| 8 | Click `"Merge with a Person"`. | The sheet `"Merge with Ben Doerfel"` opens. `"Person to merge"` is already `"The Doerfels"`. The sheet lists the items of that person and the result after the merge. | Shot `j07-07-merge-sheet` |
| 9 | Click `"Merge the People"`. | The sheet closes. The page `"Ben Doerfel"` shows two pictures. | Shot `j07-09-after-merge` |

Results:

- Collect two suggestions as two persons and merge them: 9 steps. Step 5 is 2 steps, so the count is 10.
- A shorter path is in the code. In step 6, the user can select `"The Doerfels"` in `"Person"`. The button
  then changes from `"Make the Person"` to `"Add 1 Item"` or `"Add N Items"` (`people.js:2091`). This path
  adds the items to the person and needs no merge. **Inference:** 6 steps. No walk did this path.
- The merge sheet showed 4 rows with the same text, `"The Doerfels"` and `"Credit name"`. A row does not
  tell the feed of the credit (`people.js:2199`).
- The merge sheet said `"2 wallets · 6 credit names"` and `"Name, picture and default address stay as in
  Ben Doerfel"`.
- After the merge, no notice told that the merge succeeded. The sheet closed and the page showed the merged
  items.
- A person page has `"Discard"` and `"Save"` in the toolbar. After `"Make the Person"` and after the merge,
  the toolbar showed `"No changes"`.

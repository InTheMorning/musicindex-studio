# Findings Of Local Use On 2026-11-06

Date: 2026-11-06.

Status: advisory, and open. This review records the problems found during local use of the desktop app. It states no rule. A rule needs an ADR.

The operator runs `target/release/studio-desktop-app` on a Linux computer, from the current `main` (`143e0b9`, version `0.1.0-rc.19`). This binary is not an AppImage, so it uses the system WebKit and GStreamer. It uses the default data folder.

## Findings

### 1. The App Loads Very Slowly

Corrected for the next candidate (ADR 0124 item 28). On Linux with an NVIDIA card, the app starts itself again one time with this variable. The first load of the app took a very long time before this correction.

The terminal showed many lines of "Failed to import DMABuf with file descriptor". The computer has an NVIDIA GeForce GTX 750 Ti and an X11 session. The app sets no WebKit renderer option.

With `WEBKIT_DISABLE_DMABUF_RENDERER=1`, the app loads fast and the lines stop. One computer gave this result. No test examined the AppImage or a different graphics card.

### 2. The Warning Of The Account Export

Corrected for the next candidate. The warning of the export sheet did not tell the user the risk.

The texts are in `crates/api/ui/move.js`, lines 50, 51 and 118. The user guide has the same statement in `docs/user-guide/start-on-desktop.md` and `docs/user-guide/account.md`. ADR 0107 item 5 gives the facts for the sheet. The archive holds the feeds, the drafts, the versions and the people with no encryption. The vault and the grant vaults stay encrypted.

The sheet now says: "A person who has this file can read your feeds, drafts, versions and People list. Your passwords and keys stay locked in your vault. Keep this file private."

### 3. The Nostr Sheet Does Not Offer The People Of The User

Open. The sheet "Find a Nostr profile" searches only by npub, NIP-05 address or name. It does not show the Nostr keys of the People list, so the user must search for a key that Studio already has. The requirement is wider than this sheet: each field that a person can fill must offer the People list.

"Pick From Your People" is now in three locations: the recipients, the credits and the support links (task 066). The sheet is one component, `NostrSheet` of `crates/api/ui/editor.js`, and ADR 0123 gives each Nostr key field that sheet. A change to that sheet therefore covers each Nostr key field. Proposal, with no decision: the sheet lists the people with a key before the first search, and the search stays below this list. The other fields that a person can fill need an inventory first.

### 4. The Outline Of An Album Has No Clear Structure

Open. Only "Live shows" has a heading, and no heading ends its group, so the rows below it look like parts of that group. The tracks have no heading. The outline has four forms of row and two left edges. An empty "Live shows" takes two rows, and one of them is a blue link.

In an album with 19 tracks, the feed rows are below the track list. The user must scroll through each track to find them. The left panel uses levels: an artist holds its releases, and the page of the artist holds its settings. The album mixes its content and its settings in one list.

The direction is open. Published documents of other products show three patterns:

- The head of the object leads to its settings, and the list holds only content. Discord opens Server Settings from the server name. The Team Home of Linear has a shortcut to team settings. The album editor of Bandcamp has the track listing in its left column.
- One settings destination has its own tabs. Transistor has Settings at the bottom left of its menu, with an RSS Feed tab in it. Apple tells macOS apps not to put critical items at the bottom of a sidebar.
- The list collapses. Apple recommends disclosure controls for a sidebar with a lot of content, and a maximum of two levels.
- The edit page has tabs for each level of settings. Search summaries of the SoundCloud help center give Basic Info, Metadata and Permissions. Third-party guides give the tabs of the Apple Music info window and of the MusicBrainz release editor.
- The list is one row, and it opens in the main area. Apple Podcasts Connect and Spotify for Creators work in this way for a podcast.

The page behind the outline head has the settings that a user changes often, and "Feed settings" has the settings that a user seldom changes. Proposal, with no decision: the panel holds only the tracks, and the head opens the album page. The first tab of that page is the current page, and the other rows become later tabs. The next step is mockups of this proposal, before an ADR.

### 5. Studio Uses The Sizes Of macOS And iOS

Corrected for the next candidate (musicindex ADR 0041 R-SCALE-4). The operator selected this table after a compare of screenshots. With touch, Studio already has the iOS sizes. With a mouse, Studio uses the web scale of musicindex ADR 0029. It must change to the macOS table of musicindex ADR 0024 R-TYPE-1 and R-TARGET-1.

| Value, with a mouse | Now (`type.web`) | New (`type.desktop`) |
|---|---|---|
| Body | 15 / 22 | 13 / 16 |
| Headline | 15 / 20 | 13 / 16 |
| Title 1 | 28 / 34 | 22 / 26 |
| Large Title | 34 / 40 | 26 / 32 |
| Smallest text | 11 | 10 |
| Target, default | 32 | 28 |
| Target, minimum | 24 | 20 |

The change applies to each mouse profile, also to a browser on a server, because the UI has no code for one shell (ADR 0124). musicindex.org keeps the web scale, so Studio and that site have different sizes with a mouse. v4vmm uses the macOS table, as musicindex ADR 0029 records.

The operator confirmed this scope: the desktop app and the web app are a little too large, and the website stays as it is. "A little" can be less than the change in the table. The operator also gave a screenshot of the Apple Music web app. Its sidebar rows are 40 px apart in that image, and its text does not look like 13 px. The zoom of that image is not known, so these are not measured values. 
The change also gave a group row the height of the target plus 16 px, and the body text the line height of the table. It corrected the place of a field label and the inner space of a field with several lines.

### 6. The Tray Icon And The Close Of The Window

Corrected for the next candidate (ADR 0124 item 29). A left click on the tray icon did nothing on Linux. No message told the user that a close of the window keeps the app in the tray.

The first fault was a limit of the `libappindicator` backend, not of Linux. With i3 and `snixembed` on a virtual screen, that backend gave no left click, and the `ksni` backend opened the window. The app now uses `ksni` on Linux. With no tray host it shows no icon, so a close quits. The first close asks what to do, and Settings changes the choice.

### 7. One Announcement Check Box At The Write

Open. A user who saves frequently also sends many podpings. Requested change: one check box replaces the two settings of ADR 0052 items 5 and 6. It is at the Write button, and the user sets it for each write.

The hub sends the podping, so one check box loses only the choice of a pubnotify with no podping. These items need a decision in the record that changes ADR 0052:

- The planned announcements are a part of the diff ID (ADR 0052 item 12). A change of the check box must not make the user approve a new diff.
- The start state of the check box.
- A granted write has no Write button, so it needs a fixed rule.
- A change set writes many feeds, so it needs one check box or one for each feed.

### 8. A Preview In The Nostr And Artwork Rows

Open. The rows could be slightly taller and show a small preview. The artwork row can use the image that the feed gives. The Nostr row has only the key, so a picture and a name need a source. Proposal: show them only when Studio has them, with no request to a relay when a page opens.

### 9. The Text Of Lock Against Import

Corrected in the working tree, with no commit. The text did not tell the user three facts. The lock is a request. It can stop a move to a platform that imports the feed. The email address is public.

The new text is in `crates/api/ui/feedfacts.js`, `crates/api/ui/editor.js` and `docs/user-guide/edit-album.md`. The facts come from the `podcast:locked` page of the Podcasting 2.0 documentation. The statement about a move is a conclusion from that page.

### 10. "Other" Hides Parts Of The Share Bar

Corrected for the next candidate (ADR 0133). With six recipients at 16.7 %, the bar shows three parts and "Other" at 50 %. `shareParts` in `crates/api/ui/values.js` names at most three parts. Requested change: "Other" stays at a maximum of about 10 %.

### 11. The Time Of The Automatic Lock

Open. Requested change: a setting on the Settings page gives the time before Studio locks. ADR 0055 item 16 fixes the time of the server at 10 minutes with no use of a vault secret. The user guide states that the desktop app does not lock the vault by itself, so the setting adds a new function on the desktop.

Proposal, with no decision: one select menu. The desktop starts at "Never". The server keeps 10 minutes at first and has a maximum. A longer time keeps a vault key in the memory of a public server for longer. The change needs a record that changes ADR 0055 item 16.

### 12. The Export Does Not Tell The User How To Import

Text corrected for the next candidate. Finding 13 gives the larger solution. "Move Studio" named an import, but Settings had no import and no text about where it is. In ADR 0107 item 9, the import on the desktop is only on the first page of a collection with no feed. An import thus cannot replace data. The user with data must use a new data folder with `--data-dir`.

`crates/desktop-app/src/main.rs` reads no `--data-dir`, so the Tauri app always uses its default folder. A user of the installed app with feeds therefore cannot import without the command-line shell. The text of the first page also says "Moving from a different server?", and the file can come from a desktop.

### 13. Many Collections In The Desktop App

Open. Requested function: the desktop app can change between collections. One default folder holds each collection as a subfolder. The app can make, overwrite and delete a collection. Tests of a new start, an import and an export now need scattered data folders.

This function also gives finding 12 a solution, and it changes the rule of ADR 0107 item 9 that an import cannot replace data. Mockups come before an ADR ([ADR 0124](../adr/0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md) owns the current data folder). The record must decide these items:

- The move of the current data folder into a subfolder.
- The stop of the current collection, with a write that runs (ADR 0124 items 16 to 21).
- Connected apps and queued announcements of a collection that is not open. Only the open collection runs, so a granted write to another collection fails.
- A separate vault for each collection, with its own passphrase.
- Two collections with the same feed can overwrite each other on the file host. One feed is in this condition on 2026-11-06.
- A delete and an overwrite remove local data only. They need a confirmation sheet, and they change nothing on a file host.
- The route for the change. The UI has no code for Tauri, and a server must not have this function.

### 14. The Journey To Move A Release To A Different File Host

Open. This trace comes from the code and the user guide. No person ran it on a real file host.

1. The user connects the new file host in **File hosts**.
2. The user opens the release and selects **Files**.
3. **File host of the feed XML** puts a feed move into the draft.
4. **Default file host of the release files** changes only the host of new files. This change is not in the draft.
5. For each file, the user opens its line and selects **Move to**. This adds a copy, a change of the address and a deletion to the draft. A file that is not in the library asks first, and the write waits for its download.
6. Review shows the move and the operations. The user writes.
7. The write sends the files and the feed, reads the destination, writes the pointer file and records the new host. Then it does the deletions.
8. The hub gets the two addresses after 30 seconds.

The first three small items below have corrections for the next candidate: the user guide states the deletion of **Move to** and the pointer file, the removal message names a move, and the select of the feed XML host leaves out a Blossom host. The core already refuses that move, with no public address. The other work waits for mockups.

The trace found these gaps:

- ADR 0009 item 4 gives a move of some or all files. No action moves all the files of a release, and no action moves all the releases of a file host.
- A user can move the feed and leave the files on the old file host. Nothing warns about this condition.
- **Move to** also deletes the file from before, in `crates/api/src/routes.rs`. The user guide tells only about the copy. ADR 0009 has this as an open question.
- No connector gives `can_redirect` true, so each move gives a pointer file. The redirect path has only a test connector, and the UI and the guide offer two results.
- A removal of a file host with feeds tells the user to remove the feeds from the collection. A move is the correct advice for a user who leaves that file host.
- The select of the feed XML host lists each connector, also a Blossom host, which cannot keep a feed XML. The result of that selection is not examined.

This research used published documents. No person examined the signed-in screens of these products. The ADR must also examine the outline of a publisher feed and of a playlist.

### 15. The Desktop App Has No Size Control, And The Tiles Are Small

Corrected for the next candidate (ADR 0134). A browser can zoom a page, and the desktop app cannot. Settings now has the five UI scale levels of v4vmm, and a release tile has a minimum width of 200 px.

The terminal also shows a warning that `libayatana-appindicator` is deprecated. The tray library of Tauri causes it, it does not show outside a terminal, and it stays.

### 16. The Focus Ring Of A Select Menu

Corrected for the next candidate (musicindex ADR 0041 R-SCALE-4 item 6). The ring was too heavy, and it did not match a text field. The ring is now 2 px with no gap.

### 17. Test Data From Production

Open. A check of the tile dimension needs four real releases. The operator wants an easy way to bring the file host settings from production to a test collection. Finding 13 can cover this.

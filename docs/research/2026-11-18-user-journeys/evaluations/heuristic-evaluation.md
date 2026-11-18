# Heuristic evaluation of two Studio journeys

Method: heuristic evaluation with the 10 heuristics of Nielsen (H1 to H10).
Data: the walks at commit `a64637d`, the shots `j01-*`, `j02-*`, `j04*-*`, `j06-*` and `j07-*`, and the code in `crates/api/ui/`.
User: a new independent artist who does not know RSS, Nostr, Lightning, podping, LNURL, node addresses, feeds, value splits or hosts.

Marks: "Shot" means a screenshot shows the fact. "Code" means the code shows the fact and no walk examined it. "Inference" means the finding is a conclusion from other facts.

I read each file and changed none. Cohesion findings compare Studio with `../musicindex/search.html` and `../HitIt/src/view_models/value_routes.rs`. The rules come from ADRs 0023, 0026, 0064 and 0080 in `../musicindex/docs/adr/`. I did not read `tokens.json`.

## Journey 1: first run to a written release

### F1. The default host is a local folder, so the music never goes online
- Journey: 1. Step: J01 "Where it lives".
- Evidence: `j01-05-where.png`, `crates/api/ui/welcome.js:92` (the default row "Folder on this computer"), `crates/api/ui/outsidecheck.js:136-155` (`writeNotice` returns nothing for a feed with no public address). Mark: Shot and Code.
- Problem: The first row is selected and says "The public address is optional." The text above says "Studio does not keep your music. It puts your artist page and your songs on a file host that you choose." A new artist accepts the default and continues. The artist then sees a finished album. No listener can reach the album. The UI never says so. The artist believes the release is out.
- Principle: H5.
- Severity: 4.
- Confidence: medium. The walk used a test folder. A desktop user may have a different default.
- Direction: Do not preselect a host that has no public address. Or tell the user in plain words that nobody can hear the music yet.

### F2. Two contradicting texts say who keeps the music
- Journey: 1. Step: J01 library screen, then "Where it lives".
- Evidence: `crates/api/ui/app.js` `LibrarySheet` (about 1759-1795, "Studio keeps one folder for each release here: its feed file, its images and its audio files."), `crates/api/ui/welcome.js` `pathTexts().where`, `j01-05-where.png`. Mark: Code and Shot.
- Problem: One sheet says Studio keeps the files. The next says Studio does not keep the music. The artist cannot tell where the songs are and who holds them. The word "here" in the first text does not say which folder it means.
- Principle: H4.
- Severity: 3.
- Confidence: high.
- Direction: Use one description of the storage, in one place, with the same words.

### F3. The library sheet shows zero counts and a grey "Continue"
- Journey: 1. Step: J01 first screen.
- Evidence: `j01-01-library.png`, `j01-02-library-result.png`, `crates/api/ui/app.js` `LibrarySheet`. Mark: Shot and Code.
- Problem: The first screen a new user sees asks about a library folder and shows zero counts. The artist has no library. The grey "Continue" button looks off, and the next step is not clear. The question is about a thing that the artist does not have.
- Principle: H8.
- Severity: 2.
- Confidence: medium.
- Direction: Show the library sheet only when the folder has content, or start with the Welcome sheet.

### F4. Two questions ask "where" in one path
- Journey: 1. Step: J01 "Where it lives" and the library folder.
- Evidence: `j01-05-where.png` (the "Folder" field "The full path of the folder that holds your feed files." and "Public address of the folder (optional)"), `crates/api/ui/welcome.js` `PATHS.music.steps`. Mark: Shot and Code.
- Problem: The user answers a library folder, a host type, a folder path and a public address. Each has a different meaning. The terms "feed files" and "public address of the folder" are not known to the user. The form asks a technical question when the artist wants to put out a song.
- Principle: H2.
- Severity: 3.
- Confidence: medium.
- Direction: Ask one question about the outcome ("Do you want a public page now?") before the technical fields.

### F5. The host list names Nostr, S3, R2, Blossom and SFTP with no guide
- Journey: 1. Step: J01 "Where it lives".
- Evidence: `j01-05-where.png` ("Nostr site (nsite)", "S3 or Cloudflare R2", "SFTP server", "Blossom servers, signed with your Nostr key"). Mark: Shot.
- Problem: The new artist has no basis to choose between four host types. Each description uses a term that the artist does not know. The artist has no hint about which row is easy or free. The artist will choose the first row (see F1) or stop.
- Principle: H6.
- Severity: 3.
- Confidence: high.
- Direction: Give a short recommended choice and mark the others as advanced.

### F6. After the write, no message says that it worked
- Journey: 1. Step: J02 after "Write".
- Evidence: `j02-05-review.png`, `j02-06-after-write.png`. The toolbar says "No changes", the hero says "This feed has no public address yet." and the sidebar icon is a dashed circle. Mark: Shot.
- Problem: The most important action of the journey ends with no confirmation. The artist sees the same screen with a smaller toolbar. The text "This feed has no public address yet." sounds like an error. The artist cannot tell whether the write worked, where the files are or what to do next.
- Principle: H1.
- Severity: 3.
- Confidence: high.
- Direction: Show a success notice that says where the release is now and what the next step is.

### F7. No link to the release on musicindex.org after the write
- Journey: 1. Step: J02 after "Write".
- Evidence: `j02-06-after-write.png`, `crates/api/ui/app.js` `CopyButton` and `FeedAddress` (about 1925-1950), `../musicindex/docs/adr/0023` (R-LINK-1). HitIt shows "Open on musicindex.org". Mark: Shot, Code and Inference (the HitIt string).
- Problem: The user cannot open the result. ADR 0023 R-LINK-1 asks for a link from a record to its public page. The artist has no proof of the result and no way to show it to a friend.
- Principle: H1.
- Severity: 3.
- Confidence: medium.
- Direction: Add an "Open on musicindex.org" action when the feed has a public address.

### F8. The status icon "Not checked" is a dashed circle with no label
- Journey: 1. Step: J02 and J06, the sidebar row of the release.
- Evidence: `j02-06-after-write.png`, `crates/api/ui/outsidecheck.js:206-238` (`STATE_MARKS`, "Not checked", icon 'circle-dashed'). Mark: Shot and Code.
- Problem: The dashed circle looks like a loading spinner that stopped. The user cannot tell if the circle is a fault, a wait or a neutral state. The name "Not checked" shows only on hover or in a deeper screen.
- Principle: H1.
- Severity: 2.
- Confidence: medium.
- Direction: Give a visible text label for the state in the row, or remove the mark when no check is possible.

### F9. The two actions "Read Again" and "Review" are not explained
- Journey: 1. Step: J02 editor hero.
- Evidence: `j02-06-after-write.png` (button "Read Again", top right of the hero), `crates/api/ui/app.js:2027` and `:2288`. Mark: Shot and Code.
- Problem: "Read Again" has no object. The artist does not know what Studio reads, or from where. The button has no use for a feed that has no public address. A button in the hero looks like a main action. The button is not one.
- Principle: H2.
- Severity: 2.
- Confidence: medium.
- Direction: Name the object ("Read the feed again") and hide the button when it has no use.

### F10. The artist page can stay unwritten and the Review sheet stops the write with a hidden reason
- Journey: 1. Step: J04 Review without a linked artist page.
- Evidence: `j04-03-review.png`, `crates/api/ui/editor.js:2034-2064` (write) and `:2127` (`hasWork`), `:2228` (footer buttons). Mark: Shot and Code.
- Problem: The Review sheet shows an alert and one clear action in the footer, "Discard the Draft". The action to link a page is less clear. A new user can choose the wrong button. The wrong choice deletes the draft and the work of the user. This is a risk in a step where a stop reason is the first thing the user sees.
- Principle: H5.
- Severity: 3.
- Confidence: medium.
- Direction: Put the way forward as the first action. Move the destructive action away from the focus.

### F11. The step "Link a page" explains neither "artist page" nor why Studio needs it
- Journey: 1. Step: J04 after the alert (`j04c-01-after-link-a-page.png`, `j04c-02-linked.png`).
- Evidence: `j04c-01-after-link-a-page.png`, `j04c-02-linked.png`, `crates/api/ui/app.js` `StartSheet` (about 3543-3640). Mark: Shot and Code.
- Problem: The new artist made an artist page in the Welcome sheet (the title "Your artist page"). The Review sheet then says that no artist page is linked. The two uses of "artist page" do not match, and the user sees the same word for two things. The artist has to guess that a page on Music Index is a different page than the page of the Welcome sheet.
- Principle: H4.
- Severity: 3.
- Confidence: medium.
- Direction: Use two names for the two things, or link the page automatically.

### F12. The payment recipient form uses node and LNURL words and has no check
- Journey: 1. Step: J06 add a recipient.
- Evidence: `j06-03-add-sheet.png`, `crates/api/ui/values.js` (TYPES about line 100, `RecipientForm` 229-275, `defaultType` 511), `crates/api/ui/recipients.js:453` (`WALLET_TYPES`), `:218` (`lookupHint`). Mark: Shot and Code.
- Problem: The default type is "Node Address". The artist has a Lightning address, if any (for example name@getalby.com), and does not know the word node. The form does not check the type against the entered text. A Lightning address in a "Node Address" field is a fault that shows only when a listener tries to pay. The artist does not see it.
- Principle: H5.
- Severity: 3.
- Confidence: medium. The missing check is a conclusion from the code and not a tested fault.
- Direction: Detect the type from the text. Or make the default "Lightning address" as in the other apps.

### F13. Split inputs and the tab name "Value Routes" are not explained
- Journey: 1. Step: J06 add a recipient and the tab list.
- Evidence: `j06-03-add-sheet.png`, `j06-04-added.png`, `crates/api/ui/shell.js:226` (tab 'Value Routes'), `crates/api/ui/values.js` `AlbumPayments` (520-535), `crates/api/ui/recipients.js:34-42` (`percentages`). Mark: Shot and Code.
- Problem: The artist wants to get paid. The tab is "Value Routes", the row label is "Split (shares)" and there is a "Fee" switch. None of these terms has an explanation on the screen. A user cannot know if 50 shares means 50 percent. The result is a split that the artist did not intend.
- Principle: H2.
- Severity: 3.
- Confidence: medium.
- Direction: Name the tab "Payments". Show the percent of each person next to the shares.

### F14. Nothing checks that the release has a recipient, a cover and tracks
- Journey: 1. Step: J06 Review and Check (`j06-06-review.png`, `j06-07-check.png`).
- Evidence: `crates/core/src/feed_check.rs:20-50` (the list of finding kinds has no kind for a missing cover, a missing recipient or an empty album), `j06-07-check.png`, `j02-06-after-write.png` ("Artwork" shows "No cover"). Mark: Code and Shot.
- Problem: The artist can write a release with no cover and no way to get paid. The Check tab reports no problem. The artist wants to get paid, and this check is the one that matters. The result is a release that earns nothing, with a green check.
- Principle: H5.
- Severity: 3.
- Confidence: medium.
- Direction: Add a mild warning for each of the three conditions. The core owns the rules (ADR 0065), so this is a decision for the owner.

### F15. A write error shows the raw message of the server
- Journey: 1. Step: any step that can fail, for example Review "Write".
- Evidence: `crates/api/ui/app.js:441-444` (`ErrorNotice` shows `error.message`) and `:468-482` (`title()` default "Something went wrong."). Mark: Code.
- Problem: A failed write shows the text of the system. The artist sees an English technical message, such as a network or permission fault, under a title that has no information. ADR 0023 R-MSG-1 asks for a message that says what the user can do. The artist cannot recover without help, and no other person is there to ask.
- Principle: H9.
- Severity: 3.
- Confidence: medium. No walk caused a failure.
- Direction: Map the common faults (no network, no permission, bad address) to a sentence with one next step.

### F16. The editor uses terms from the feed standard without a gloss
- Journey: 1. Step: J02 album editor and import sheet.
- Evidence: `j02-06-after-write.png` ("The itunes:author of the feed.", "Nostr profile", "Hide from apps", "Apple asks for this tag. Off means clean."), `crates/api/ui/editor.js:2670`, `:2700`, `:3908`. Mark: Shot and Code.
- Problem: The hint of "Artist" uses a tag name. The user does not need the tag. The Explicit hint says "Off means clean" and "Studio adds the tag only when you set the switch." This is a rule about files, not about the music. The artist reads the text and gains no help.
- Principle: H2.
- Severity: 2.
- Confidence: high.
- Direction: Describe the effect for a listener. Keep tag names for an advanced view.

### F17. The import sheet gives file errors as codes and has no repair path
- Journey: 1. Step: J02 import sheet and rows.
- Evidence: `j02-02b-import-sheet.png`, `j02-03-import-rows.png`, `crates/api/ui/editor.js:1198-1330` (`IMPORT_ERRORS`, "Track ready"). Mark: Shot and Code.
- Problem: The path works for the walk file. For a file that fails (a type that Studio does not accept, a damaged file), the code uses a fixed list of error words. I could not confirm that each word says what to change. The artist knows MP3 and WAV, so the first step is a good fit. The failure case is the risk.
- Principle: H9.
- Severity: 2.
- Confidence: low.
- Direction: Check that each import error names the file and one fix. This finding needs a test with a bad file.

### F18. The build date and podping notices use words from the feed standard
- Journey: 1. Step: Review sheet, after the write.
- Evidence: `crates/api/ui/announce.js:24` ("Send a podping") and `:27` ("No podping: this feed has no public address."), `crates/api/ui/builddate.js`. Mark: Code.
- Problem: The artist meets "podping" and "build date" in the Review sheet. Neither is known. The sheet gives a reason in terms that need prior knowledge. The artist cannot decide whether to send a podping and does not know what happens if not.
- Principle: H10.
- Severity: 2.
- Confidence: medium.
- Direction: Say "Tell apps that the release is new" and give the effect in one line.

## Journey 2: People, collect and merge

### F19. The merge happens with one click, with a filled primary button and no undo
- Journey: 2. Step: J07 merge sheet.
- Evidence: `j07-07-merge-sheet.png` (button "Merge the People"), `crates/api/ui/peoplepages.js:1262-1320` (`MergeForm`), `crates/api/ui/confirms.js:62` (`deletePerson` has a confirm, no merge confirm exists). Mark: Shot and Code.
- Problem: The sheet shows a preview, "After the merge", and the text "Studio moves each item of the selected person to Ben Doerfel, then deletes that person." The preview is good. The step itself is final. The toolbar has "Discard", but the shot j07-09 shows no sign that the merge is in a draft. There is no "Undo" and no confirm. `deletePerson` has a confirm, so the app is not consistent (ADR 0080 R-ACT-3, ADR 0064 R-ACT-12).
- Principle: H3.
- Severity: 3.
- Confidence: medium. I did not run the merge, so I cannot say if "Discard" undoes it.
- Direction: Let the user undo the merge, or say in the sheet how to undo it.

### F20. The sheet preselects the first person in the list
- Journey: 2. Step: J07 merge sheet.
- Evidence: `j07-07-merge-sheet.png` (the select shows "The Doerfels"), `crates/api/ui/peoplepages.js:1262-1320`. Mark: Shot and Code.
- Problem: The select starts with a person who is first in the list, not the one that the user chose. The user can press the filled blue button at once and merge the wrong person. The text "then deletes that person" shows only in small grey type.
- Principle: H5.
- Severity: 3.
- Confidence: high.
- Direction: Start with no selection, or start with the person that the user came from.

### F21. After the merge, the page shows no result message
- Journey: 2. Step: J07 after the merge.
- Evidence: `j07-09-after-merge.png` (the toolbar says "No changes", the page shows Ben Doerfel with no notice), `crates/api/ui/people.js:2206` (`mergeResult`). Mark: Shot and Code.
- Problem: The sheet promised "2 wallets · 6 credit names". After the merge, the page does not repeat the result. The user must scroll to find the moved items. The toolbar says "No changes", which contradicts a merge that just happened. The user cannot tell whether the merge is saved.
- Principle: H1.
- Severity: 3.
- Confidence: medium.
- Direction: Show a notice such as "The Doerfels is merged into Ben Doerfel. 2 wallets and 6 credit names moved."

### F22. The merge sheet lists the same credit name four times
- Journey: 2. Step: J07 merge sheet.
- Evidence: `j07-07-merge-sheet.png` (four rows "The Doerfels / Credit name"). Mark: Shot.
- Problem: Four rows with the same text have no way to tell them apart. The sheet is long and the user cannot judge what moves. The line "2 wallets · 6 credit names" is a better summary than the list.
- Principle: H8.
- Severity: 2.
- Confidence: high.
- Direction: Group equal rows and show a count.

### F23. People with different names do not appear as merge candidates
- Journey: 2. Step: J07 People page and suggestions.
- Evidence: `crates/api/ui/people.js:70-74` (`EVIDENCE`), `:78` (`suggestionRow`), `crates/api/ui/peoplepages.js:131-240` (`PeoplePage`). Mark: Code and Inference.
- Problem: The purpose of People is to join scattered identities. My reading of the code is that Studio suggests a pair only from shared evidence, such as the same wallet. One musician with a stage name and a band name, and with two different wallets, has no common evidence. The user has to find the second person by hand. This is the case that the purpose names.
- Principle: H7.
- Severity: 3.
- Confidence: low. I did not test the matching rules.
- Direction: Let the user start a merge from a person and search by name.

### F24. Two paths join identities and nothing explains the difference
- Journey: 2. Step: J07 Check pages and merge links.
- Evidence: `j07-03-check-doerfels.png`, `j07-05-check-ben.png`, `crates/api/ui/people.js:2194` (`mergeLinks`), `crates/api/ui/peoplepages.js` (`CheckEntry`). Mark: Shot and Code.
- Problem: The user can link a person to a Music Index page ("Link to an Artist or Label Page") or merge two people. Both join identities. The words "linked artist and label pages", "payees" and "other values in your feeds and profiles" are not explained. The user cannot choose a path.
- Principle: H10.
- Severity: 2.
- Confidence: medium.
- Direction: Add one sentence to the People page that says when to use each path.

### F25. A person appears in more than one list with no explanation
- Journey: 2. Step: J07 sidebar.
- Evidence: `j07-07-merge-sheet.png` and `j07-09-after-merge.png` (the sidebar groups "Hosting", "Labels", "Artists", "Producers", "No role"; the rows "CityBeach 10" and "Test music feed 4" have counts and no explanation). Mark: Shot.
- Problem: The groups sound like a role system. The group "No role" holds feeds and not people. The numbers have no label. The user cannot learn what the count means (credits, feeds or wallets).
- Principle: H6.
- Severity: 2.
- Confidence: low. I could not read the walk data for these rows.
- Direction: Label the count, for example "10 feeds".

### F26. A button with no effect: the observation O1 handler
- Journey: 2. Step: outside-change notice.
- Evidence: `crates/api/ui/app.js` `OutsideNotices` (about 4830-4930, handler at about 4916) and `navValue` (about 4013, no `go`). Mark: Code and Inference.
- Problem: My reading is that one notice button calls a handler that has no target for this case, so the button has no effect. I did not run it. A dead control is a dead end with no explanation.
- Principle: H3.
- Severity: 2.
- Confidence: low.
- Direction: Check the observation O1 with a test and remove the button or give it a target.

## Cohesion with the Music Index website and HitIt

### F27. Wallet type names differ between Studio and the other two apps
- Journey: 1. Step: J06 add a recipient.
- Evidence: `j06-03-add-sheet.png` and `j07-07-merge-sheet.png` ("Node address", "LNURL address"), `crates/api/ui/recipients.js:453`, `../musicindex/search.html:5286` (`ROUTE_TYPE_WORDS`: "Lightning address", "Lightning node"), `../HitIt/src/view_models/value_routes.rs:500-502` and `:804`. Mark: Shot and Code.
- Problem: A user of two apps meets "Lightning address" and "Lightning node" on the website and in HitIt. Studio shows "Node Address", "LNURL Address" and "Account". ADR 0080 owns the shared words (R-WORD-1, R-VALUE-1 and R-VALUE-2). The same wallet gets three names.
- Principle: H4.
- Severity: 3.
- Confidence: high.
- Direction: Use the shared word list of ADR 0080 in Studio.

### F28. Studio names split and fee fields differently
- Journey: 1. Step: J06 add a recipient.
- Evidence: `j06-03-add-sheet.png` ("Split (shares)", "Fee"), `../HitIt/src/view_models/value_routes.rs:500-502` ("Type", "Shares", "Custom key", "Custom value"). Mark: Shot and Code.
- Problem: HitIt says "Shares" and "Custom value". Studio says "Split (shares)". The user of both apps must map the labels. ADR 0075 (R-VALUE-5) is the rule for value fields. I did not check each point of that rule.
- Principle: H4.
- Severity: 2.
- Confidence: medium.
- Direction: Use the same label for the same field in all three apps.

### F29. The tab name "Value Routes" differs from the user word for payment
- Journey: 1. Step: tab list of the editor.
- Evidence: `crates/api/ui/shell.js:226`, `../HitIt/src/view_models/value_routes.rs` (file name and strings). Mark: Code and Inference.
- Problem: HitIt also uses "Value Routes". Both apps agree, so there is no cohesion fault. The fault is for this user, who does not know the term (see F13). I record the point to stop a wrong change that makes Studio differ from HitIt.
- Principle: H2.
- Severity: 1.
- Confidence: medium.
- Direction: Change the word in all three apps in one change.

### F30. Status display and error notices differ from the shared rules
- Journey: 1. Step: J02 after the write and any failure.
- Evidence: `j02-06-after-write.png`, `crates/api/ui/app.js:441-444`, `../musicindex/docs/adr/0026` (R-NOTICE-1, R-LOADING-1, R-EMPTY-1), `../musicindex/docs/adr/0023` (R-MSG-1). Mark: Inference.
- Problem: I compared Studio with the written rules and not with every screen of the other apps. Studio shows raw error text (F15), a status mark with no label (F8) and no write notice (F6). Each of these is against a rule that the other two apps follow, as far as I could read. A user of two apps sees clear notices in the website and none in Studio.
- Principle: H4.
- Severity: 2.
- Confidence: low.
- Direction: Run the rule list of ADR 0026 against each Studio notice.

### F31. Two filled primary buttons on one screen, and "Try Again" against "Read Again"
- Journey: 1. Step: J01 editor (`j01-06-editor.png`) and J02 hero.
- Evidence: `j01-06-editor.png`, `j02-06-after-write.png` (the filled "Review" button and the grey "Read Again" button), `../musicindex/docs/adr/0080` (R-ACT-1 and R-ACT-2). The site uses "Try Again". Mark: Shot and Inference.
- Problem: In `j01-06-editor.png` the screen shows more than one filled blue action. ADR 0080 allows one main action in one place. The website says "Try Again" for a retry, and Studio says "Read Again". The user of both apps sees two names for the same kind of action.
- Principle: H4.
- Severity: 1.
- Confidence: low. I read this shot at a smaller scale and the count of filled buttons is a judgment.
- Direction: Keep one filled button in each view and use the shared word for a retry.

## Vocabulary

| Term | First place | Explained |
| --- | --- | --- |
| Feed | "This feed has no public address yet." (hero of the editor, `app.js:2021`) | No |
| Public address | "The public address is optional." (`welcome.js:92`, `j01-05-where.png`) | No |
| File host | "It puts your artist page and your songs on a file host that you choose." (`j01-05-where.png`) | Partly (the list of host types follows, with no guide) |
| Nsite / Nostr / Blossom | "Nostr site (nsite)" (`j01-05-where.png`) | No |
| S3, Cloudflare R2, SFTP | `j01-05-where.png` | No |
| Feed files | "The full path of the folder that holds your feed files." (`j01-05-where.png`) | No |
| itunes:author | "The itunes:author of the feed." (`editor.js:2670`) | No |
| Nostr profile | "Nostr profile / Not set" (`j02-06-after-write.png`) | No |
| Hide from apps | `j02-06-after-write.png` | Yes (one line below the switch) |
| Explicit | `j02-06-after-write.png` | Partly ("Apple asks for this tag. Off means clean.") |
| Read Again | hero of the editor (`app.js:2027`) | No |
| Review / Write | toolbar and Review sheet (`editor.js:1958-2260`) | Partly |
| Podping | "Send a podping" (`announce.js:24`) | No |
| Build date | `builddate.js` | No |
| Value Routes | tab (`shell.js:226`) | No |
| Node Address / LNURL Address | `j06-03-add-sheet.png` | No |
| Split (shares) / Fee | `j06-03-add-sheet.png` | No |
| Payees | People page folds (`peoplepages.js:131-240`) | No |
| Linked artist and label pages | `j07-09-after-merge.png` | Partly (one paragraph) |
| Credit name / Usual role | `j07-07-merge-sheet.png`, `j07-09-after-merge.png` | Partly |
| Wallet | "2 wallets · 6 credit names" (`j07-07-merge-sheet.png`) | No |
| Merge | "Merge with Ben Doerfel" (`j07-07-merge-sheet.png`) | Yes (the sheet says what moves and what is deleted) |

## The 5 findings with the most effect on this user

1. F1. The default host is a local folder, so the music never goes online. The user finishes and the music is not public.
2. F6. After the write, no message says that it worked. This is the end of the first journey.
3. F19 and F20. The merge has one click, a preselected first person, no confirm and no undo. This is the main action of the second journey.
4. F12 and F14. The payment recipient has no type check, and nothing warns about a release with no recipient. The artist wants to get paid.
5. F2 and F4. The storage questions contradict each other and ask in the words of the system. The artist cannot decide at the first step.

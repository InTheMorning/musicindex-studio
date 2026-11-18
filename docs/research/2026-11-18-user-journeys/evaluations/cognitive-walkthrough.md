The main risks are payment setup, public availability, and recovery after an incorrect person merge.

This walkthrough uses the walk tables and source code. I did not inspect PNG files or change files. User behavior is a prediction, not a measured result.

All short source references below are within `crates/api/ui/`. “Yes” for Q2 means the code makes the control available. It does not establish visual prominence.

**Step results**

The columns answer:

- **Q1:** Will the artist know this step is necessary?
- **Q2:** Will the artist find the correct control?
- **Q3:** Will the artist connect its label with the goal?
- **Q4:** Will the artist recognize the result as progress?

Each “No” refers to a finding below. Additional code branches cover required recovery and alternative paths.

**Journey 1 — J01: first run**

Sources: `app.js:1760`, `app.js:3501`, `app.js:3549`, `welcome.js:32`, `welcome.js:78`.

| Step | Action | Q1 | Q2 | Q3 | Q4 | Finding |
|---|---|---|---|---|---|---|
| J01.1 | Type the library path in “Folder”. | Yes | Yes | Yes | Yes | — |
| J01.2 | Select “Use This Folder”. | Yes | Yes | Yes | Yes | — |
| J01.3 | Select “Continue” after the library report. | Yes | Yes | Yes | Yes | — |
| J01.4 | Select “I make music”. | Yes | Yes | Yes | Yes | — |
| J01.5 | Type “Artist name”. | Yes | Yes | Yes | Yes | — |
| J01.6 | Select “Continue” to reach “Where it lives”. | Yes | Yes | Yes | Yes | — |
| J01.7 | Select “Continue” with the default local folder. | Yes | Yes | No | Yes | F2 |
| J01 code branch | Supply “Public address of the folder (optional)” for public access. | No | Yes | No | Yes | F1 |

The library report explicitly identifies completion. The new artist page and “Add your first album” card also show progress.

The optional typed path has no folder picker. I did not score this as a failure: a default exists, and the field identifies its purpose.

**Journey 1 — J02: release, track, and write**

Sources: `editor.js:4021`, `app.js:1414`, `editor.js:1243`, `editor.js:1473`, `editor.js:1973`, `editor.js:2034`.

| Step | Action | Q1 | Q2 | Q3 | Q4 | Finding |
|---|---|---|---|---|---|---|
| J02.1 | Select “Add a Release”. | Yes | Yes | Yes | Yes | — |
| J02.2 | Type “First Album” in “Album title”. | Yes | Yes | Yes | Yes | — |
| J02.3 | Select “Make the Release”. | Yes | Yes | Yes | Yes | — |
| J02.4 | Select “Add tracks”. | Yes | Yes | Yes | Yes | — |
| J02.4a, code | Select “＋ Choose Music and Image Files”. | Yes | Yes | Yes | Yes | — |
| J02.5 | Select the MP3 file. | Yes | Yes | Yes | Yes | — |
| J02.6 | Select “＋ Add 1 Track”. | Yes | Yes | Yes | Yes | — |
| J02.7 | Select “Review” and examine the changes. | Yes | Yes | Yes | No | F3 |
| J02.8 | Select “Write to the File Host”. | Yes | Yes | Yes | No | F4 |

J02.4a is a separate control in the code. The walk table combines its effect with native file selection.

“Track ready” and the new track row provide useful feedback. The change from “ALBUM” to “SINGLE” does not remove that feedback.

**Journey 1 — J04: missing artist page**

Sources: `editor.js:2672`, `editor.js:2126`, `editor.js:2525`, `editor.js:3811`, `feedcheck.js:72`.

| Step | Action | Q1 | Q2 | Q3 | Q4 | Finding |
|---|---|---|---|---|---|---|
| J04.1 | Open “Loose Single”. | Yes | Yes | Yes | Yes | — |
| J04.2 | Enter “Description”. | Yes | Yes | Yes | Yes | — |
| J04.3 | Select “Review”. | Yes | Yes | Yes | Yes | — |
| J04.4 | Select “Link a Page”. | Yes | Yes | Yes | Yes | — |
| J04.5 | Select “Artist X”. | Yes | Yes | Yes | Yes | — |
| J04.6 | Select “Link to This Page”. | Yes | Yes | Yes | Yes | — |
| J04.7 | Select “Review” again. | Yes | Yes | Yes | Yes | — |
| J04.8 | Select “Write to the File Host”; the write fails. | Yes | Yes | Yes | No | F7 |
| J04 code branch | Link a page when no candidate exists. | Yes | No | No | No | F6 |
| J04 code branch | Decide whether to keep “Send a podping” selected. | Yes | Yes | No | Yes | F5 |
| J04 continuation | Select “Open Artist X” from the related-draft notice. | Yes | Yes | Yes | Yes | — |
| J04 continuation | Select “Review” for “Artist X”. | Yes | Yes | Yes | Yes | — |
| J04 continuation | Write the artist page after a satisfactory review. | Yes | Yes | Yes | No | F4 |

The actual J04 link repair gives a clear next step and a clear result. It also explicitly identifies the second draft.

The three continuation steps were not walked. They follow the code and the notice.

**Journey 1 — J06: payment recipient**

Sources: `shell.js:226`, `values.js:135`, `values.js:229`, `values.js:520`, `recipients.js:34`.

| Step | Action | Q1 | Q2 | Q3 | Q4 | Finding |
|---|---|---|---|---|---|---|
| J06.1 | Select “Value Routes”. | No | Yes | No | Yes | F8 |
| J06.2 | Select “＋ Add recipients”. | Yes | Yes | Yes | Yes | — |
| J06.3 | Select “Add a recipient”. | Yes | Yes | Yes | Yes | — |
| J06.4 | Enter `artist@getalby.com` in “Address”. | No | Yes | No | Yes | F9 |
| J06.5 | Select “Add the Recipient”. | Yes | Yes | Yes | No | F9 |
| J06.6 | Enter `1` in “Split (shares)”. | No | Yes | No | No | F10 |
| J06 code branch | Select the correct “Type” before adding the address. | No | Yes | No | Yes | F9 |
| J06 follow-up | Examine “Payments” in “Review”. | Yes | Yes | Yes | No | F3, F9 |

J06.4 assumes the walk’s supplied address. This artist has no stated knowledge of obtaining a Lightning payment address.

**Journey 2 — J07: collect identities and merge**

Sources: `app.js:3331`, `peoplepages.js:131`, `peoplepages.js:286`, `peoplepages.js:1125`, `peoplepages.js:1272`.

| Step | Action | Q1 | Q2 | Q3 | Q4 | Finding |
|---|---|---|---|---|---|---|
| J07.1 | Select “People”. | Yes | Yes | Yes | Yes | — |
| J07.2 | Open “Suggestions from your feeds”. | Yes | Yes | Yes | Yes | — |
| J07.3 | Select “The Doerfels”. | Yes | Yes | Yes | Yes | — |
| J07.4 | Select “Make the Person”. | Yes | Yes | Yes | Yes | — |
| J07.5a | Return to “People”. | Yes | Yes | Yes | Yes | — |
| J07.5b | Open the suggestions again, if collapsed. | Yes | Yes | Yes | Yes | — |
| J07.6 | Select “Ben Doerfel”. | Yes | Yes | Yes | Yes | — |
| J07.7 | Select “Make the Person” for the second identity. | No | Yes | No | Yes | F11 |
| J07.8 | Select “Merge with a Person” and examine its preview. | Yes | Yes | Yes | No | F12 |
| J07.9 | Select “Merge the People”. | Yes | Yes | Yes | No | F13 |
| J07 alternative | Select the existing person in “Person”. | No | Yes | No | Yes | F11 |
| J07 alternative | Select “Add 1 Item” or “Add N Items”. | Yes | Yes | Yes | Yes | — |
| J07 recovery | Undo a merge of the incorrect person. | Yes | No | No | No | F14 |
| J07 failure branch | Find more identities after a suggestion request fails. | Yes | No | Yes | No | F15 |

The new person page is sufficient feedback for “Make the Person”. A separate completion message is less necessary there.

The shorter collection path exists. J07 does not establish whether users will discover it.

### F1. Local storage does not complete the artist’s publication goal

- Journey: 1. Step: J01 public-address branch, through J02.8.
- Evidence: [welcome.js:92](crates/api/ui/welcome.js:92), `app.js:787`, `app.js:2283`, `announce.js:27`. Mark: Code.
- Problem: The default folder and optional public address allow a local write. The editor states, “This feed has no public address yet.” But the interface gives this artist no procedure for obtaining public access. The artist can complete the recorded write while listeners still cannot reach the music.
- Principle: CW Q1, CW Q3.
- Severity: 3.
- Confidence: high.
- Direction: Distinguish local preparation from public availability and identify the remaining publication steps.

### F2. “Continue” does not predict creation of the artist page

- Journey: 1. Step: J01.7.
- Evidence: [app.js:3588](crates/api/ui/app.js:3588), `welcome.js:35`, `editor.js:4021`. J01 records the sheet closing. Mark: Code.
- Problem: The sheet lists “Your name”, “Where it lives”, and “Your first album”. The second “Continue” creates the artist page and closes the sheet. The next stage becomes a card elsewhere. The artist can see the new page, but the control does not predict this change.
- Principle: CW Q3.
- Severity: 2.
- Confidence: high.
- Direction: Make the stage sequence and the creation control describe the same process.

### F3. “Simplified” does not provide enough information to check important changes

- Journey: 1. Step: J02.7 and J06 follow-up.
- Evidence: [review.js:236](crates/api/ui/review.js:236), `review.js:172`, `editor.js:1552`. J02 and J06 record the resulting rows. Mark: Code.
- Problem: J02 shows “Podcast GUID” with a `pending-…` value. J06 shows “Payments” without details and “Payments” with “Added”. These rows do not tell the artist whether the payment address and shares are correct. The upload row still provides useful track feedback.
- Principle: CW Q4.
- Severity: 3.
- Confidence: high.
- Direction: Show the recipient, address type, address, and payment share in understandable terms.

### F4. The write result is not explicit

- Journey: 1. Step: J02.8 and the J04 artist-page continuation.
- Evidence: [editor.js:2059](crates/api/ui/editor.js:2059), `editor.js:2274`, `outsidecheck.js:136`. J02 records the result. Mark: Code.
- Problem: After the write, “Review” closes and the toolbar shows “No changes”. This describes the current draft, but does not explicitly identify the completed write. Music Index notices depend on other conditions and do not provide universal completion feedback.
- Principle: CW Q4.
- Severity: 3.
- Confidence: high.
- Direction: Identify the written release and destination, with public availability reported separately.

### F5. “Send a podping” does not identify its purpose

- Journey: 1. Step: J04.7 announcement decision.
- Evidence: [editor.js:1855](crates/api/ui/editor.js:1855), `announce.js:20`. Mark: Code.
- Problem: The footer presents “Send a podping” without a definition. This artist cannot connect that selection with notifying indexes about the release. The explanatory hint used by another announcement component is absent here.
- Principle: CW Q3.
- Severity: 2.
- Confidence: high.
- Direction: State the effect on discovery by music apps and indexes.

### F6. The missing-page repair can end without a next step

- Journey: 1. Step: J04 code branch with no candidate artist page.
- Evidence: [editor.js:2531](crates/api/ui/editor.js:2531), `editor.js:2574`, `editor.js:2127`, `feedcheck.js:77`. Mark: Inference.
- Problem: “Link a Page” opens the link section. With no candidates, it shows “Studio has no other artist or label page to link to.” There is no creation control there. “Review” also omits the write button. The artist must discover a separate creation process.
- Principle: CW Q2, CW Q3, CW Q4.
- Severity: 3.
- Confidence: high.
- Direction: Provide a direct next step when the required page does not exist.

### F7. The write error does not tell the artist how to recover

- Journey: 1. Step: J04.8.
- Evidence: [app.js:441](crates/api/ui/app.js:441), `app.js:481`, `editor.js:2041`, `editor.js:2164`. J04 records the proxy error. Mark: Code.
- Problem: The artist sees “Something went wrong.” followed by server text. The proxy error does not identify a useful recovery procedure for this user. The write control can remain available, but the message does not tell the artist what to correct before another attempt.
- Principle: CW Q4.
- Severity: 3.
- Confidence: high.
- Direction: Identify the operation that failed, the state of the write, and the applicable recovery step.

### F8. “Value Routes” does not identify payment setup

- Journey: 1. Step: J06.1.
- Evidence: [shell.js:226](crates/api/ui/shell.js:226), `values.js:523`. Mark: Code.
- Problem: The artist wants payment for music. “Value Routes” requires knowledge of the payment model before the artist opens it. The useful sentence about track payments appears inside the page, and only when tracks exist.
- Principle: CW Q1, CW Q3.
- Severity: 3.
- Confidence: medium.
- Direction: Identify payments at the navigation entry.

### F9. Payment setup requires unknown address knowledge and accepts an incorrect type

- Journey: 1. Step: J06.4, J06.5, and the type-selection branch.
- Evidence: [values.js:64](crates/api/ui/values.js:64), `values.js:247`, `values.js:261`, `values.js:279`, `values.js:511`. J06 records acceptance and the absence of a check finding. Mark: Code.
- Problem: The default is “Node Address”. Its hint requires a Lightning node key. The alternative uses “LNURL Address”. Neither tells this artist how to obtain a payment address. The walk’s `artist@getalby.com` is accepted as “Node Address”, and the percentage display suggests usable payment setup. No actual payment was tested.
- Principle: CW Q1, CW Q3, CW Q4.
- Severity: 3.
- Confidence: high.
- Direction: Guide address selection and detect a mismatch before presenting the recipient as ready.

### F10. Shares have no explanation, and the percentage hides the empty value

- Journey: 1. Step: J06.6.
- Evidence: [values.js:191](crates/api/ui/values.js:191), `values.js:143`, `recipients.js:34`. Mark: Code.
- Problem: “Split (shares)” has no explanation or example. One recipient shows 100.0% even with an empty field. Entering `1` leaves that percentage unchanged. The artist cannot tell why this step is necessary or how shares differ from percentages.
- Principle: CW Q1, CW Q3, CW Q4.
- Severity: 2.
- Confidence: high.
- Direction: State how shares determine payment percentages and identify the meaning of an empty field.

### F11. The interface directs identity collection toward separate person records

- Journey: 2. Step: J07.7 and the existing-person alternative.
- Evidence: [peoplepages.js:191](crates/api/ui/peoplepages.js:191), `peoplepages.js:326`, `people.js:2073`, `people.js:2082`, `people.js:2091`. Mark: Code.
- Problem: The suggestions note says, “Check each one, then make a person.” For an unmatched name, “Person” starts at “A new person”. The artist wants one record, but receives no explanation that this menu can collect different names into the same person. The helpful match hint appears only for an existing suggested match.
- Principle: CW Q1, CW Q3.
- Severity: 3.
- Confidence: high.
- Direction: Explain the existing-person path before the artist creates a second record.

### F12. The merge preview cannot distinguish repeated credit identities

- Journey: 2. Step: J07.8.
- Evidence: [people.js:2194](crates/api/ui/people.js:2194), `peoplepages.js:1301`. J07 records four identical credit rows. Mark: Code.
- Problem: Each credit row shows its name and “Credit name”. It omits the source feed, role, website, and key. Four rows can therefore have the same text. The artist cannot check which scattered identities the merge will collect.
- Principle: CW Q4.
- Severity: 3.
- Confidence: high.
- Direction: Include the source and distinguishing identity information in the merge preview.

### F13. The merge result is difficult to check

- Journey: 2. Step: J07.9.
- Evidence: [peoplepages.js:1288](crates/api/ui/peoplepages.js:1288), `peoplepages.js:1144`, `peoplepages.js:1130`. J07 records the resulting pictures and toolbar. Mark: Code.
- Problem: The sheet closes and the retained person page reloads. The walk shows two pictures and “No changes”. These changes provide some feedback, but no message identifies the two merged records or the collected identities. The artist must inspect the page to establish completion.
- Principle: CW Q4.
- Severity: 2.
- Confidence: high.
- Direction: Identify the completed merge and the retained person.

### F14. An incorrect merge has no direct recovery

- Journey: 2. Step: recovery after J07.9.
- Evidence: [peoplepages.js:1288](crates/api/ui/peoplepages.js:1288), `peoplepages.js:999`, `peoplepages.js:1131`, `people.js:2189`. Mark: Inference.
- Problem: The merge moves the identities and deletes the other person record. The interface provides no merge undo or person separation procedure. “Discard” applies to pending field changes. Manual removal and recreation are separate operations, with no guidance for restoring the previous records.
- Principle: CW Q2, CW Q3, CW Q4.
- Severity: 3.
- Confidence: high.
- Direction: Provide a recoverable merge result or a guided procedure to restore separate records.

### F15. Suggestion failures look like an absence of identities

- Journey: 2. Step: suggestion loading after J07.4, J07.7, or J07.9.
- Evidence: [peoplepages.js:856](crates/api/ui/peoplepages.js:856), `peoplepages.js:864`, `peoplepages.js:1035`, `peoplepages.js:160`. Mark: Inference.
- Problem: A request failure clears “More from your feeds” or profile-address suggestions. A recipient-list failure also leaves no payee list. The affected lists have no error or retry control. The artist cannot distinguish missing information from a complete identity search.
- Principle: CW Q2, CW Q4.
- Severity: 3.
- Confidence: high.
- Direction: Distinguish empty results from unavailable results and provide another attempt.

**Errors, recovery, and dead ends**

These checks include conditions that the recorded walks did not exercise.

| Steps or condition | What the artist sees | Recovery and undo |
|---|---|---|
| J01.1–3: library setup fails | An error notice. A partial move report lists file errors. | Correct “Folder” and submit again. A report still offers “Continue”. No setup undo appears. `app.js:1760`. |
| J01.7: local folder is invalid | Field text identifies an incorrect path or missing folder. | Correct “Folder” and select “Continue” again. `app.js:780`, `app.js:455`. |
| J01.7: host connects, page creation fails | The sheet says the file host connected and offers “Make the Page”. | Retry page creation without creating a second host. `app.js:3611`. |
| J02.3: release creation fails | An error remains in the creation form. Path errors open “Feed file”. | Correct the affected field and submit again. `app.js:1431`, `app.js:1452`. |
| J02.5: upload fails | Progress, automatic part retries, then a file-specific error. | Remove the failed row and select the file again. There is no row-level retry button. `editor.js:1210`, `editor.js:1243`. |
| J02.5: incorrect file selected | The import row identifies the file. | “Remove” excludes it. “Cancel” removes ready imports that were not added. `editor.js:1399`, `editor.js:1410`. |
| J02.6: incorrect track added | The track appears in the release. | Open it and select “Remove the Track”. Before writing, “Discard the Draft” can discard the whole draft. `editor.js:3416`, `editor.js:2067`. |
| J04.2 or J06: automatic save fails | “Not saved”, with error details available in “Review”. | Opening “Review” attempts to flush the draft again. A stale revision provides “Load the Draft Again”. `draft.js:244`, `editor.js:2141`. |
| J04.4: no artist candidate | A message replaces the selector and link button. | No direct creation step exists here. This is a conditional dead end, F6. |
| J04.6: incorrect page linked | The linked row appears. | “Remove” removes the link. Matched links can change both drafts again. `editor.js:2537`. |
| J04.8: remote write fails | The recorded general error and proxy text. | Another write attempt is possible after correction, but the correction is unclear, F7. |
| Review finds a malformed web address | Red field text and a refusal in “Review”. | Correct the identified fields. This protection did not catch the J06 payment-type mismatch. `editor.js:2165`. |
| J02/J04: correction after writing | “Versions” lists earlier files, when available. | “Put in the Draft”, review, and write again. This is not immediate undo. A first write can have no earlier version. `editor.js:2236`. |
| J07.1: main People request fails | An error replaces the loading state. | Leave and reopen “People” to load again. There is no local retry button. `peoplepages.js:139`, `peoplepages.js:226`. |
| J07.3/6: incorrect suggestion items selected | Selected checkboxes appear in the check sheet. | Clear those checkboxes before submission. After creation, remove individual identities from the person. `peoplepages.js:317`, `peoplepages.js:999`. |
| J07.4/7: person creation fails | The check sheet retains an error and its controls. | Correct the input or retry “Make the Person”. `peoplepages.js:293`, `peoplepages.js:336`. |
| J07.8: no second person exists | “No other person yet. Make a second person first.” | “Cancel” returns to the person. The merge button is disabled. `peoplepages.js:1298`, `peoplepages.js:1311`. |
| J07.8: incorrect person selected | The selected person and merge preview appear. | Change “Person to merge” or select “Cancel”. The first other person is selected automatically. `peoplepages.js:1278`. |
| J07.9: merge request fails | The merge sheet shows an error. | The artist can retry or cancel. There is no completed-merge undo, F14. `peoplepages.js:1288`, `peoplepages.js:1307`. |
| J07: supplemental reads fail | Some suggestion lists disappear. | No explicit recovery control appears, F15. |

No inert control was established on the recorded main paths. The strongest conditional dead ends are the missing artist candidate and the unavailable merge recovery.

**Vocabulary**

“Explained” means the UI gives a usable meaning at that location. A format example alone does not explain the underlying service.

| Term | First relevant place | Explained? |
|---|---|---|
| Library | “Studio set up your library”, `app.js:1773`; preceding text describes the release folder contents. | Yes |
| Feed / feed file | Library text and “Feed files”, `app.js:1775`, `app.js:1789`. | No |
| Full path | “Folder” hint, `app.js:785`. | No |
| Artist page | “I make music” description, `welcome.js:12`. | Yes — albums belong on it. |
| File host | “Where it lives” text, `welcome.js:64`. | Yes — it holds pages and songs. Obtaining one is not covered. |
| Public address | Local hosting choice and field, `welcome.js:92`, `app.js:787`. | No — its use is stated, but public access requirements are not. |
| Web server | Public-address hint, `app.js:789`. | No |
| Nostr | “Nostr site (nsite)”, `welcome.js:93`. | No |
| nsite | “Nostr site (nsite)”, `welcome.js:93`. | No |
| Blossom servers | Hosting-choice description, `welcome.js:93`. | No |
| Nostr key | Hosting-choice description, `welcome.js:93`. | No |
| S3 / Cloudflare R2 | Hosting choice, `welcome.js:94`. | No — classified as storage providers only. |
| Storage bucket | Hosting-choice description, `welcome.js:94`. | No |
| SFTP server | Hosting choice, `welcome.js:95`. | No |
| Draft | New-page notice, `app.js:2295`; longer status in `draft.js:108`. | Yes — saved edits remain separate from the hosted file. |
| Write | New-page notice, `app.js:2295`. | Yes — creates the file. Public availability remains unclear. |
| Nostr profile | Artist or album fields, `editor.js:2660`. | No |
| `itunes:author` | “Artist” hint, `editor.js:2670`. | No |
| `podcast:episode` / `itunes:episode` | Track-order note, `editor.js:2866`. | Yes — identified as track numbers. |
| Podcast GUID / GUID | J02 “Review”, `review.js:172`. | No — later placeholder text describes generation, not its purpose. |
| Build date | “The build date does not change.”, `builddate.js:6`. | No — the time-server failure is described. |
| XML | “Detailed” view hint, `review.js:16`, if selected. | No |
| Podping | Review footer, `announce.js:20`, `announce.js:27`. | No |
| Value routes | “Value Routes”, `shell.js:226`. | No |
| Recipient | “This feed has no recipients.”, `values.js:528`. | Yes, conditionally — the track-payment hint appears when tracks exist. |
| Node address / Lightning node / public key | Default payment type and hint, `values.js:65`. | No — required key format is given. |
| LNURL address | Payment type and hint, `values.js:66`. | No — an address example is given. |
| Split / shares | “Split (shares)”, `values.js:191`. | No |
| Fee | “Fees” and “Fee”, `values.js:194`, `values.js:210`. | Yes, after a fee exists — `recipients.js:53` describes its effect. |
| Custom key | Recipient form, `values.js:270`, `recipients.js:456`. | No — the hint says when to copy one. |
| Account on a node | “Account” hint, `values.js:271`. | No |
| Payee | “Payees with no person”, `values.js:278`; J07 suggestion rows, `people.js:81`. | No |
| Wallet | J07 suggestion evidence or person payment section, `people.js:73`, `peoplepages.js:1047`. | Yes on the person page — associated with a payment address. |
| Value block | J07 suggestion evidence, `people.js:72`. | No |
| Credit name | Merge preview, `people.js:2199`; earlier “Credited as” fold, if opened. | Yes only in that fold’s hint, `people.js:677`. |
| Merge | “Merge with a Person”, then its sheet, `people.js:2189`. | Yes — items move, the other person is deleted, and feeds do not change. |

RSS is part of the underlying feed model. The recorded main steps do not establish a literal “RSS” encounter. Its artwork-conflict branch is outside these steps.

**Five findings with the most effect**

1. **F9 — Incorrect payment type accepted.** The artist can believe payment setup is complete without evidence that the address is usable.
2. **F1 — Local writing does not establish public availability.** The recorded endpoint can leave the artist’s publication goal incomplete.
3. **F14 — No direct merge recovery.** An incorrect identity merge requires manual reconstruction.
4. **F12 — Merge identities cannot be distinguished.** Missing source details make the irreversible decision harder to check.
5. **F11 — Existing-person collection is poorly identified.** The interface encourages extra records and an avoidable merge.
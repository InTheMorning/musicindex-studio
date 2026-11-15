# The Open Older Human Tests, For One Walk On rc.23

Date: 2026-11-14

This record is advisory. It states no rule. Each test keeps its owner, which is the ADR in its line.

The record lists each human test of Phase 12 (tests 5 to 9) and of tasks 049 to 093 that was open on 2026-11-14.
A step is not in the walk when a later human walk or an agent check proved it.
The sources are the task files, `docs/reviews/2026-10-26-phase-12-human-tests.md`, the records of the human walks from rc.17 to rc.22, and `docs/reviews/2026-11-14-human-check-walk.md`.
ChatGPT made the search in two runs. The main session examined the sources of the closed items.

## Closed By A Later Record

| Test | Result | Evidence |
| --- | --- | --- |
| Task 092, build dates and roll back | Passed | `2026-11-05-rc17-human-test-findings.md`: the index build date, a roll back, a clock 10 minutes ahead and the next write. |
| Task 090, a real move, a refusal, a repair and a write | Passed | The same record. The return passed in `2026-11-11-local-human-test-findings.md` and in task 127. One part is open: step D4. |
| Task 088, the AppImage starts, keeps the collection, takes a drop, and Ctrl+Z | Passed | The rc.17 record. Play passed in the rc.22 AppImage. The close behavior passed in the rc.21 record. The upgrade of real data passed in task 097. |
| Task 086, the desktop part | Passed | Task 086 and the notes of rc.14. The server part is step A2. |
| Phase 12 test 9, on the desktop | Passed | `2026-11-11-local-human-test-findings.md`: the vault locked after the lock time. The phone part is step C4. |

## Replaced By A Later Decision

| Test | What replaced it |
| --- | --- |
| Task 049, make an album from "Add a Feed" | ADR 0213: a release starts on the page of its artist. The gate of ADR 0213 has the new flow. The part "add a track from an address on a phone" is step C2. |
| Task 066, the three pickers | ADR 0144 gives the recipient one sheet, **Add a Recipient**. Step B6 uses the current sheets. |
| Task 076, **Use** and "Saved" on the picture field | ADR 0196 gives lists of pictures and websites. Task 133 proved the lists with screenshots. Step B7 has the part with a real Nostr profile. |
| Tasks 081 and 082, "Member of" | ADR 0132 gives the links to artist and label pages. Step B8 (task 093) has the add and the removal. |
| Task 084, "Check your releases" | ADR 0203 names the page **Check your feeds**. Step B5 has the correction. |

## The Walk

Each step names its test. A step passes when the screen or the hosted file shows the result that the step gives.
Where a later decision changed a control name, the step gives the current name from that decision.

### A. A New User On The Server, With A Computer

1. Sign up on `studio.musicindex.org` with the NIP-07 extension. Sign out and sign in again. (Phase 12 test 6, ADR 0056.) Result: the two operations pass.
2. Select **I make music** and give a name. Make the pages in the temporary area. Connect a file host and move the pages to it. (Tasks 083 and 086, ADR 0119 and ADR 0122.) Result: each hosted feed has a real GUID, and no link has a placeholder GUID.
3. Add a release from the page of the artist, with the flow of ADR 0213. Write the two feeds through Review. Result: the release has the link to its artist page.

### B. Feeds And People On The Server, With A Computer

1. On a test release, set the OP3 switch to on. Set the role of one credit to **Vocalist**. Change a title that has an apostrophe. Write the feed. (Tasks 052, 074 and 078.) Result: Review marks only the changed words of the title. The hosted XML has `role="vocalist" group="music"`. A track plays in an app, and the OP3 link opens the OP3 page.
2. On the artist feed, add a recipient on the tab **Value Routes** and a support link on the tab **Credits**. Examine Review and write. (Task 050, ADR 0075.) Result: Review shows the two changes.
3. On the Recipients page, examine a Fountain address, an Alby address and a Strike address. Use **Use the LNURL Address** to write one feed. (Tasks 060 and 061, ADR 0207.) Result: Fountain and Alby are one entry with their node, and Strike is a separate entry. The Fountain node shows its provider name, and a node with no known provider shows its key. The hosted feed has `type="lnaddress"` and `method="lnaddress"`.
4. Find the key of a contributor by name and the key of an artist by NIP-05 address. Write the feed. Copy the album credits to a track. (Task 051, ADR 0074.) Result: each copied credit keeps its key.
5. Open a person whose credits have a different key. Open **Check your feeds** and select **Correct N Feeds**. Examine "What changes", select **Detailed**, then **Show the Whole File**, then **Show Only the Changes**. Write. Check again. (Tasks 075, 077 and 084.) Result: only the changed attribute has a change mark. After the write, each feed is correct.
6. Make a person from a suggestion. Use each sheet one time: **Use This Address**, **Rename**, **Add to All Releases** and **Remove from Feeds**. Add that person as a recipient, as a credit and as a support link. (Tasks 065 and 066, ADR 0203 and ADR 0144.) Result: each sheet lists the feeds that it changes, and each selection puts the values of the person into the draft.
7. On the page of one person: change two fields and select **Discard**, then change them again and select **Save**. Add the picture from the Nostr profile. Fill the picture and the website from **Find on Music Index**. Copy the full `npub`. (Tasks 067, 076, 079 and 080.) Result: Discard gives back the two values, Save keeps them after the page opens again, and the copy gives the full `npub`.
8. Merge a second person into the first person, and delete a test person. Link one person to an artist page and to a label page, give a different role in each credit sheet, and write the two feeds. Then remove one link, save, and write. (Tasks 067, 082 and 093, ADR 0132.) Result: the merge and the delete make no feed draft. Each feed page shows the person in its credits, and the second write removes the credit.
9. Sign in on two devices and change the passphrase on one. (Task 069, ADR 0106.) Result: the other device goes to the sign-in page.
10. Play a track and examine the Network panel of the browser. (Task 072, ADR 0109.) Result: the audio request goes to the file host.
11. Export the account. Import it in Studio on the desktop, open a draft and write it. (Task 071, ADR 0107.) Result: the write passes.

### C. The iPhone

1. Use "Add to Home Screen". (Phase 12 test 8, ADR 0059.) Result: Studio opens with no browser bar.
2. Add a track from an address. Add a second track from a file, write the draft, and select **Download to This Device**. (Tasks 049 and 070, ADR 0105.) Result: "Properties" starts closed with its summary. The file comes to the phone.
3. Upload a WAV file of more than 200 MiB on a mobile network, with one change of the network. (Phase 12 test 7, ADR 0059.) Result: the upload completes.
4. Do not use the phone for 15 minutes. (Phase 12 test 9, ADR 0055.) Result: the vault locks, the session stays, and the next write asks for an unlock.
5. Find the Nostr key of a person by name and add it. (Task 087, ADR 0123.) Result: the person has the key.
6. Optional: NIP-46 with a signer for the iPhone, if one works. (Phase 12 test 5.) Record the signer and the result.

### D. The Desktop AppImage

1. Scroll a page below the toolbar. (Task 088, ADR 0124.) Result: no text shows through the toolbar.
2. Drop two audio files on **Add tracks**, then one audio file and one text file together. Drop a file on **From MSP 2.0 backups** and on **Import an account**. (Task 089, ADR 0125.) Result: the list of the sheet shows the audio files, and the text file gives an error.
3. Make a new release with "Start from" an existing release. Replace one track and write. (Task 053, ADR 0077.) Result: the source release does not change.
4. After a move of a feed, read the new address with a second app. (Task 090, ADR 0126.) Result: the second app plays the feed.

### E. The Server Host, In The Terminal Of The Operator

1. After the deploy, run `stat -c %a /data /data/users /data/users/*/vault.json` and `grep core /proc/1/limits` in the container. (Task 068, ADR 0104.) Result: the modes are `700`, `700` and `600`, and the core limit is `0`.
2. Deploy with `tools/deploy-server.sh` and the image digest. (Task 072, ADR 0066.) Result: the script writes the digest into `.env` and the container has that digest.
3. After step C2, look for an audio copy in the data folder of the user. (Task 070, ADR 0105.) Result: the server keeps no audio copy.

## Blocked

- Phase 12 tests 3 and 4, and the Android part of test 8: the operator has no Android phone.
- The second Apple device of Phase 12 test 2.
- Task 088 on macOS and on Windows: no installed system is available.
- Task 073 and Phase 12 test 10: they wait for the client of the live publisher.
- Firefox and Safari on a computer for task 089: no record says that a test ran.

# Human Test Findings Of 0.1.0-rc.17

Date: 2026-11-05.

Status: advisory. This review records the results of the human test of `0.1.0-rc.17` and the items for the next candidate. It states no rule. A rule needs an ADR.

The operator ran the test on a Linux x86-64 computer, with the planning model. The test used the AppImage and `studio-desktop`. It used the real file hosts `bastone` (SFTP) and `heycitizen` (R2), and the live index `api.musicindex.org`.

## Results

| Check | Result | Evidence |
|---|---|---|
| The AppImage starts, and the collection stays | Passed | Screenshot |
| A drop on a track, and on "Add tracks" | Passed | The operator saw the address change and the outline. The list in the "Add tracks" sheet is not confirmed. |
| Ctrl+Z | Passed | The operator |
| A move from SFTP to R2 | Passed | Both public files, and the Review |
| The move and the build date reach the index | Passed | `feed_url` and `last_build_date` from `GET /v1/feeds/{guid}` |
| Each move announces both addresses | Passed | The local queue shows the state `sent` for each address |
| A refusal of a bad tag, a repair, and a write | Passed | Screenshot, and the public file |
| The conflict check stops a stale draft | Passed | Screenshot |
| A roll back sets a later date, and the index accepts it | Passed | The public file and the index |
| A clock 10 minutes ahead gives the server time | Passed | Review showed `14:18:09`, and the real time was `14:18:26` |
| The index accepts a write after the clock is corrected | Passed | The index gave `14:20:10` and "clock test 2" |
| Play in the AppImage | **Failed** | Finding 1 |
| A second app reads the new address | **Open** | Finding 5 |
| A return in the live index | **Open** | It waits for a release of Stophammer that holds its ADR 0083 |

Each result covers this run only. No test ran the macOS or the Windows installer.

## State To Know Before More Work

- **Production does not know about the test move.** The operator made the writes from the data folder `.studio-dev/import-test`. HeyCitizen's Test Album is now on `heycitizen` at `https://music.cancelbandcamp.com/TestAlbum/test.xml`. The production collection still has it on `bastone`, and that address now holds a pointer file. Do not write this feed from production until someone examines this.
- The feed has the description "clock test 2". Each change is a test change.
- `.studio-dev/import-test` holds a full unencrypted copy of the collection of the account, and the encrypted vault. Git ignores it.
- The operator keeps a local file `wrong-copy.bin`, a first download of the AppImage. See finding 12.

## Findings For The Next Candidate

Items 1 to 4, 6 and 7 are small. Items 8 to 10 need the review of the operator first.

### 1. Play Hangs The AppImage

Failed. Play hangs the app or gives a gray window. The output is `GStreamer element autoaudiosink not found`, then `g_signal_connect_data: assertion 'G_TYPE_CHECK_INSTANCE (instance)' failed`.

`crates/desktop-app/tauri.conf.json` does not set `bundleMediaFramework`. This is the likely cause, and no rebuild has proved it. The operator saw the fault after a replacement of the audio. No test examined Play on a track with no change.

### 2. The Page Title Shows Through The Toolbar

To confirm. In the AppImage, a page that scrolls shows its title and cover through the toolbar. They overlap the text of the toolbar. WebKitGTK can be the cause. The screenshot checks of `tools/drive-ui.mjs` use Chromium, so they do not examine this.

### 3. An Earlier Local Copy Showed "No Role"

Not reproduced. The AppImage data folder showed HeyCitizen with "No role" and a hollow feed dot. Production showed the role. A new export and import gave a filled artist dot, and **Read Again** kept it. The cause is not known. The earlier data folder can give the answer.

### 4. "Announce" Touches Its Text

Seen at a sheet width of about 750 px. The label is wider than the `.kind` column of `.host-op-row`, so the text "AnnounceTell" has no space.

### 5. Apps Do Not Show The Move

Open, and no Studio change now. v4vmusic and stablekraft do not show the move. v4vmusic gives the earlier address as its feed URL, so it can stay on the frozen pointer file. The operator tells the developer.

Podcast Index removed this test feed as spam, so other apps cannot show it. The cause in each app is not known. A `301` on the earlier server is a possible help, and no test examined it.

### 6. Read Again Leaves A Stale Draft

Confirmed. **Read Again** updates the feed record and not the draft (`pipeline::refresh`). Review then checks the earlier bytes, says "The draft is the same as the hosted file", and shows no refusal for a bad tag on the host. The conflict check still stops the write, so the file is safe. A draft with no edits can safely move to the new file.

### 7. The Tag Goes First In The Channel

Decision of the operator, for a person who reads the file. When Studio adds `itunes:new-feed-url`, the tag goes directly after `<channel>`. A tag that the feed already has stays in its position. This needs an amendment of ADR 0128, because ADR 0012 item 3 gives a different position.

### 8. Start Again Loses Each Edit

Needs mockups and an ADR. **Start Again from the Hosted File** removes each edit of the draft, also for one small outside change. The proposal: Studio applies the edits to the new file. An edit of a different field stays. When both sides changed one field, the user selects in Review.

### 9. The Refusal Does Not Explain

Needs the words of the operator. The box does not say where the address came from, why it is a danger, or what the buttons do. The operator said: "this doesn't explain anything". The core can give the source of each address. The self link notice and the return warning need the same examination. A draft of the text:

```text
Studio refuses this write.

This feed says that it moved to a new address, in the tag
itunes:new-feed-url. That address has no feed. An app or an index
that follows the tag would lose your feed.

  https://music.cancelbandcamp.com/TestAlbum/does-not-exist.xml
  The server says that the file is not there (404).

Nothing changed on a file host.

To continue, remove the tag, or put a feed at that address.
```

### 10. The Simplified View

Request of the operator, and this changes ADR 0114:

- The Simplified view shows no line diff, only the changes in words.
- It shows each time in local time.
- The Detailed view keeps the text of the file. It shows the local time in a different way. Hover is possible with a mouse, and touch needs a tap or a second line.

### 11. The Clock Steps Of The Notes

The notes of `0.1.0-rc.17` say one year ahead. That is a bad test, because a large error can make secure connections and R2 requests fail. Use `faketime -f '+10m'`, as this test did. It changes the clock of Studio only. No test examined a clock with a large error on a real host. The automatic tests cover that rule.

### 12. Smaller Items

- The first download of the AppImage had an arm64 header and a different hash. The release file has the hash `df38e561…` and it runs. How the wrong local copy occurred is not known.
- **Lock against import** has no help text, and the user guide does not explain it.
- A version with an R2 ETag shows a stray quote mark in the list of **Versions**.
- ADR 0008 item 6 said that a move writes `podcast:previousUrl`, and the pipeline has no code for it. On 2026-11-05 the operator removed the tag from that item.
- A wait in the index ends only at a later crawl of the destination (ADR 0128 "Consequences"). The Stophammer session can say how often this occurs.
- The "Desktop macos-arm64" job failed one time with `hdiutil: create failed - Resource busy`, and a re-run passed. A retry in the workflow is a correction if it occurs again.
- The Stophammer session asked for a report. Its note is `.idea/studio/stophammer-feed-contract-handoff.md` in `../stophammer`.

## Corrections Of 2026-11-05

No release candidate holds these corrections yet.

| Finding | Correction | Evidence |
|---|---|---|
| 1 | The AppImage holds the GStreamer plugins, and the release check refuses one with no audio plugin (ADR 0124 item 26). | The rc.17 file on Xvfb gave the two log lines of finding 1. A local AppImage with plugins gave neither. It found the audio sinks and stopped at a limit of the sandbox, which has no `/var/tmp` and no sound device. No test heard the audio, and no CI run built this form. |
| 2 | Confirmed, and corrected (ADR 0124 item 27). The cause is a blur that WebKitGTK does not draw. | Screenshots of the rc.17 file and the local AppImage on Xvfb, at the same scroll position. No test used a computer with a GPU. |
| 4 | The `.kind` column is 5.5rem. | The label is 67 px with a mouse and 80 px with touch, in a column of 88 px. |
| 6 | A draft with no edits moves to the file that Studio read (ADR 0007 item 6). | A core test, and a Chromium check of Read Again and Review. |
| 7 | A new tag goes directly after `<channel>` (ADR 0128 item 12). | A core test. |
| 8 | "Apply My Edits to the New File" merges the hosted file into the draft (ADR 0129). | Core tests, and a Chromium check. |
| 9 | The refusal gives its source, the danger, the state and the next step (ADR 0126 item 28). | A Node test, and a Chromium check. |
| 10 | "Simplified" shows only words, with local times. "Detailed" has a second line with the local time (ADR 0114 items 9 to 15). | Node tests, and a Chromium check. |
| 12 | The version label has no quote mark. "Lock against import" has a help text and a user guide text. The operator removed `podcast:previousUrl` from ADR 0008 item 6. | A Node test, and the text. The macOS retry waits for a second failure. |

The release workflow also needed `patchelf`, because the GStreamer plugin of the bundler stops without it. A local build found this.

## Recommended Sequence

1. Correct items 1, 2, 4, 6 and 7, and the small items of finding 12 that the operator selects.
2. Show the operator the mockups and the text of items 8, 9 and 10. Then write their ADRs.
3. Make the next candidate, with its notes from item 11.
4. Run the return test after a release of Stophammer holds its ADR 0083.

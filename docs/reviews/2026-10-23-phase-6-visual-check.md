# Phase 6 Visual Check

- Date: 2026-10-23
- Reviewed: the visual check of Phase 6 in the [Studio v1 phase plan](../plans/studio-v1-phase-plan.md): the operator makes a new release from a folder of music files and writes it
- Host: a new local folder with the public address `https://music.example/releases`, and a new data folder
- Checker: operator
- Result: pass

## Results

| Step | Procedure | Result |
| --- | --- | --- |
| 1 | The first run as a new user. Connect the folder as a host with its public address. | Pass |
| 2 | Make a new release with a folder, a file, a title and an artist | Pass |
| 3 | Import music files and images. Examine the facts of each file. | Pass after two corrections |
| 4 | Import a GIF image alone as the album art. Examine the track addresses. | Pass after three corrections |
| 5 | Examine the diff. It lists 27 files in `my-album/`, with no WAV file. | Pass |
| 6 | Approve and write 24 tracks and 3 images | Pass |
| 7 | Import a WAV file. Examine the notice and the facts. | Pass |
| 8 | Add the converted track, examine the diff, and write | Pass |

After step 6, the agent examined the folder:
- `my-album/` had the 27 files of the diff, with the sizes of the diff.
- The 10 pending files that the draft did not use were not on the host. No pending row or copy remained.
- Each of the 27 addresses in `my-album.xml` referred to a file in the folder.
- Studio wrote the files before the feed file.

After step 8, the folder had `my-album/Piano.ff.A0.flac` with the `fLaC` signature, and the feed had 25 items. No WAV file was on the host or in the pending store.

## Defects Found During The Check

- Step 3: the import sheet did not scroll when it had many files. The correction gives `.sheet .body` rows of their content height.
- Step 3: a pending file that the draft did not use stopped a new import of the same name with 409 `file_pending`. After the correction, the upload replaces such a file.
- Step 4: the import refused a GIF image. The operator accepted GIF art, and [ADR 0032](../adr/0032-new-releases-formats-paths-and-new-feeds.md) item 1 records it. The row of a GIF image shows a notice that Apple Podcasts asks for JPEG or PNG art.
- Step 4: with only an image in the sheet, the button "Add 0 tracks" was disabled. After the correction, the button says "Add the album art" and adds the art.
- Step 4: the address of each file with a space in its name was not correct, so Studio could not write. After the correction, the local folder connector encodes each segment of the path. The agent encoded the 9 addresses of the draft in the check data.

## Task 019 Check

On 2026-10-23 the operator did the visual check of [task 019](../tasks/studio-v1-task-019-lossless-import.md) ([ADR 0033](../adr/0033-a-lossless-import-gives-an-mp3-enclosure.md)) in the same release. Result: pass.

| Step | Procedure | Result |
| --- | --- | --- |
| 1 | Import a WAV file and a FLAC file. Examine the notices and the facts of each file. | Pass after two corrections |
| 2 | Add the FLAC track. Examine the alternative enclosures on the track page. | Pass |
| 3 | Examine the diff of the new item | Pass after one correction |
| 4 | Approve and write | Pass |

The agent examined the pending files after step 1. The MP3 file of the FLAC file has 245 kbit/s, which agrees with VBR V0, and the four tags of the source. After step 4, the folder had the MP3 file and the FLAC file, with the sizes of the diff. No WAV file was on the host or in the pending store.

Defects found during this check:

- Step 1: the Size and Type rows gave only the MP3 file. Each file line now gives its name, its type and its size.
- Step 1: while an audio file was sending, the disabled button said "Add the album art". It now says "Add 0 tracks".
- Step 3: the alternative enclosures went after `podcast:episode`, because a new item takes the sequence of the last hosted item. The item now writes its alternative enclosures last, and a test covers the hosted sequence.
- Step 3: an import with the name of a file that the draft uses gave the message "The sheet sent a file with this name already". The message now tells the user to remove that track or image from the draft first.

The operator saw "Not saved" once. The browser tab used an address of a stopped Studio. With the current address, the removal saved.

## Task 020 Check

On 2026-10-23 the operator did the check of [task 020](../tasks/studio-v1-task-020-reuse-the-file-on-the-host.md) ([ADR 0034](../adr/0034-an-import-reuses-the-same-file-on-the-host.md)). Result: pass.

- The operator imported a FLAC file that Studio had not written, added the track and wrote it. The operator then removed the track and imported the same file again. The two file rows showed "On the host already", and Studio added no pending file. The diff listed no file, and the write put only the feed file.
- An import of the FLAC file of the task 019 check gave `file_exists` for its MP3 file. Its MP3 file came from the build before task 020, which wrote the ID3 frames in a random sequence. The refusal is correct (ADR 0034 item 3).
- The badges of the file rows had different widths, so the file names were not in line. Each badge now has a width of 56 px.

## Task 024 Check

On 2026-10-25 the agent did the visual check of [task 024](../tasks/studio-v1-task-024-phase-6-open-items.md) ([ADR 0040](../adr/0040-replace-the-audio-of-a-track.md)). The agent used `tools/drive-ui.mjs` in headless Chromium, with a new data folder and a local folder host. Node.js was not on the PATH, so Bun ran the script. A DataTransfer object set the files of each file field. Result: pass.

| Item of the Phase 6 check | Procedure | Result |
| --- | --- | --- |
| A new import gives a new item GUID | Write a release with `tagged.mp3`. Replace the audio of its track with `tagged.wav` on the track page. Examine the track page and the review, and write. | Pass |
| The art preview of a pending file shows the placeholder | Import a PNG file as the album art, and examine the album page before the write | Pass |
| The row of a converted WAV file does not show the FLAC name | Import `tagged.wav` in a second new release | Pass. The row lists `tagged.mp3` and `tagged.flac`. Task 019 made this change. |
| "Add feeds" gives no hint with no host | Open "Add feeds" with one read-only feed and no host. Select "Connect a Host". | Pass. Hosts opens. |
| The MSP 2.0 field uses the native control | Open the MSP 2.0 step of the wizard, and select two files | Pass. The names show, with the error of the file that is not JSON. |
| "Read again" has no function on a new release | Open a new release with no hosted file | Pass. After the first write, "Read again" shows. |

After the replacement, the track page kept the title "Test Title". It showed the enclosure `tagged.8a152e8e.mp3` and the two alternative enclosures. The review listed the upload of `tagged.8a152e8e.mp3` and `tagged.flac`, and the deletion of `tagged.mp3`. The diff kept the item GUID. After the write, the host had `tagged.8a152e8e.mp3` and `tagged.flac`, and did not have `tagged.mp3`. The release folder kept `tagged.mp3` (ADR 0040 item 7).

The agent did not examine the focus ring of "Choose Backup Files". The headless page did not have the focus. The button uses the same style as the file button of the import sheet.

## Open Items

None. Task 024 closes the six open items of this check.

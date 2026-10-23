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

## Open Items

- The art preview of a pending file shows the placeholder, because the image is not on the host before the write. The preview can use the pending copy.
- The row of a converted WAV file shows the name of the WAV file. It does not show the name of the FLAC file.
- The "Add feeds" screen gives no hint to connect a host when the user has no host.
- The MSP 2.0 file field uses the native "Browse…" control.
- "Read again" on a new release with no hosted file has no function.

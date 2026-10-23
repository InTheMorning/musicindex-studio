# Phase 7 Milestone 7a Visual Check

- Date: 2026-10-23
- Reviewed: the visual check of milestone 7a in [task 021](../tasks/studio-v1-task-021-release-library-7a.md). The operator sets the library folder and finds the current release in it. The operator puts one file on a second host, moves one file, deletes one copy, and writes the release.
- Data: copies of the Phase 6 data and host folder in `.studio-dev/phase-7a/`, with the empty folder `hostB` as the second host
- Checker: operator
- Result: pass

## Results

| Step | Procedure | Result |
| --- | --- | --- |
| 1 | Set the library folder. Studio moves the collection into it. | Pass after one correction |
| 2 | Connect `hostB` with a public address | Pass |
| 3 | Import one MP3 file, and open the Files page | Pass |
| 4 | Copy the file to `hostB`, and examine the review sheet | Pass |
| 5 | Write the upload and the copy | Pass |
| 6 | Move the file from the first host to `hostB`, and examine the review sheet | Pass |
| 7 | Write the move | Pass |
| 8 | Delete a file that no feed uses, and a file that the feed uses | Pass |
| 9 | Write the deletions | Pass |

The agent examined the files and the records after each write:

- Step 1: the release folder `my-album` had a feed XML file with the bytes of the hosted file. The inventory had 33 host files.
- Step 5: the MP3 file was on the two hosts with the same size, and the inventory had its SHA-256 on each host.
- Step 7: the feed referred to the address on `hostB`. The first host no longer had the file. The library XML had the bytes of the hosted feed.
- Step 9: the write deleted the file that no feed used, and kept the file that the feed used. No draft operation stayed.

## Defects Found During The Check

- Step 1: the library setting was the default path in the home folder before the operator saw the sheet. The agent removed it, with the folder `~/musicindex/studio` that held only the copy of the feed XML. A second move into the library then stopped with an error, because the first move had dropped the table `draft_files`. After the correction, a missing table gives no pending file, and a test covers it.

## Open Items

- The Hosts screen cannot change the public address of a host. The operator removed `hostB` and connected it again with its address.
- "Copy to…" and "Move to…" need a selection in a list and a second button. The selection alone does nothing.
- The review sheet does not tell the difference between an upload and a copy. A line can say "The feed uses this file" or "A spare copy. The feed does not use it."
- The Files page does not show which copy the feed uses, which other host files the feed uses, or which deletion is pending.
- "Host of this file" shows the default host of the release, not "Use the default host of the release", for a file with no host of its own.
- The entry of a library file has an empty space between its copy rows and "Copy to…".
- The operator asked for button roles from the Apple HIG: an action button such as Copy or Move has the accent color, a destructive button such as Delete or Remove has red, and a button does not have the same fill as the surface below it. This needs a decision record.
- The move into the library puts only the feed XML into the library. The audio files and the images of a published release stay on their host until the downloads of milestone 7b.

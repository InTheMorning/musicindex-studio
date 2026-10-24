# Phase 7 Milestone 7b Visual Check

- Date: 2026-10-24
- Reviewed: the visual criterion of Phase 7 in the [phase plan](../plans/studio-v1-phase-plan.md), for milestone 7b ([task 023](../tasks/studio-v1-task-023-release-library-7b.md)). The operator adds a published feed, downloads its files, puts a file on two hosts, changes one audio file, and writes the release. The operator then moves the feed XML to a different host.
- Decisions: [ADR 0035](../adr/0035-a-release-library-is-the-master-of-each-feed.md), [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) and [ADR 0038](../adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md)
- Data: the data of the [milestone 7a visual check](2026-10-23-phase-7a-visual-check.md) in `.studio-dev/phase-7a/`, and the published feed "Live Check" in `.studio-dev/phase-7b/`. A local web server gives `hostC` at `http://127.0.0.1:8766`. A second web server at `http://127.0.0.1:8767` gives one audio file, and Studio has no host for it.
- Checker: operator for steps 1 to 12. The planning model did the light theme check and the check of the corrections with `tools/drive-ui.mjs`.
- Result: pass after four corrections

## Results

| Step | Procedure | Result |
| --- | --- | --- |
| 1 | Add `hostC` as a host, and add the feed "Live Check" that Studio found in it. | Pass after one correction |
| 2 | Open the Files page of "Live Check". Four files are not in the library. | Pass |
| 3 | Click "Download all". Two files come through the connector, and one file comes through HTTP. | Pass |
| 4 | Click "Scan the hosts". | Pass after one correction |
| 5 | Expand the entry of `01 Thrift.mp3`. | Pass |
| 6 | Click "Copy to hostB", and open the review sheet. | Pass |
| 7 | Write the copy. | Pass |
| 8 | Replace `02 Talljam.mp3` in the library folder, and click "Scan the library folder". | Pass |
| 9 | Click "Use the new file", and open the review sheet. | Pass |
| 10 | Write the new file. | Pass |
| 11 | Set the host of the feed XML to `hostB`, and open the review sheet. | Pass after one correction |
| 12 | Write the move of the feed XML. | Pass after one correction |
| 13 | Light theme: the Files page and the review sheet. | Pass |

After each write, the planning model compared the host folders with the library. The feed XML on the host and in the library had the same bytes. After step 10, `hostC` had `live/02 Talljam.c371ee4f.mp3` and not the file from before. After step 12, `hostB` had the feed with its new self link, and `hostC` had the pointer file with `itunes:new-feed-url`.

## Defects Found During The Check

- Step 1: after "Add the folder", the panel of the feeds that Studio found in the folder was at the top of the Hosts page. The form is at the bottom, so the panel was out of view. The Hosts page now brings the result of a new host and that panel into view.
- Step 4: Studio did not list the files of a new host until the user clicked "Scan the hosts". The Files page then did not show the copies on `hostC` that the feed uses. Studio now lists a new host when the user adds it, and [ADR 0036](../adr/0036-library-records-host-inventory-and-host-operations.md) item 6 adds this time.
- Step 11: the feed move row in the review sheet did not show the address from before. Thus the operator could not see the address of the pointer file. The hint in "Hosts" did not show the new address. The two texts now show the two addresses.
- Step 12: after the move, "Default host of the release files" changed to `hostB`. The release had no default host of its own, so it used the host of the feed XML. ADR 0038 item 16 says that the files stay on their host. The write of a move now gives the release the host from before as its default host of the files, when it has none. A test of the pipeline enforces this.

## Open Items

- A copy on a host in the folder of a different release shows on the Files page with "Delete", also when the draft of that release already deletes it.
- The notice "Studio found … changes in the library folder" shows above the toolbar. Its "Show" can open a feed that has no Files page.
- No row shows the pointer file when the feed XML is at the root of the host, because the Files page shows only the files of the release folder.
- "Use the new file" does not change the feed when the address of the file is on a web server that is not a host.
- An HTTP download has no idle timeout while it reads the body.
- The folder scan holds the lock of the workspace while it reads the files.
- In step 4, the first click on "Scan the hosts" did not change the copies. The scan through the API was correct. The cause is not known, and the operator possibly took the screenshot before the click.

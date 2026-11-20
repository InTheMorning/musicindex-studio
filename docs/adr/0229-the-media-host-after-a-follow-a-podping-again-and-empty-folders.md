# ADR 0229: The Media Host After A Follow, A Podping Again, And Empty Folders

- Status: Accepted
- Date: 2026-11-20
- Owner: operator
- Accepted: 2026-11-20 by the operator, in the human check of rc.29 (`docs/reviews/2026-11-20-rc29-human-check.md`, findings 1 to 3). The operator selected "Failed or no reply" for items 4 to 7 from a drawing in the questionnaire. The operator said in the chat that the button **Send the Podping Again** must not change the height of the notice.
- Technical choices: the main session selected items 1 to 3, item 7 and items 8 to 11. The operator did not receive them as a question.
- Bar: crosses modules (the core follow, the host connectors and the notices). An agent can also give a sent podping line the button again by mistake, because ADR 0191 item 4 gave it.
- Changes: [ADR 0211](0211-studio-follows-a-move-that-a-different-studio-made.md) items 11 and 16, the host of the media files after a follow (items 1 to 3).
- Changes: [ADR 0191](0191-a-return-warning-has-the-warning-color-and-a-sent-podping-can-go-again.md) item 4 and [ADR 0228](0228-undo-in-the-bar-a-notice-that-stays-and-the-rc28-check-findings.md) item 9, the button of a sent podping line (items 4 to 7).
- Changes: [ADR 0184](0184-after-a-move-studio-shows-the-podping-of-the-move.md) item 7, the button of a sent line of the move notice (item 6).
- Uses: [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) item 16, the delete of one file with its stamp. [ADR 0228](0228-undo-in-the-bar-a-notice-that-stays-and-the-rc28-check-findings.md) item 10, the height of the notice.

## Context

The operator did the human check of rc.29 on 2026-11-20. These findings need a decision:

- Finding 1: the tab **Files** of the test album showed the media files on the host "heycitizen". The feed and its media files were on "bastone". The cause: **Follow the Move** and **Use This File Host** call `set_feed_location`. That function changes the host of the feed file. It keeps the old host of the media files (`files_connector_id`).
- Finding 2: the notice after a write shows **Send the Podping Again** on a line that says "Sent", while Music Index has not answered yet. The button has no use in that state, and its row changes the height of the notice.
- Finding 3: the delete of a leftover copy deleted the files and kept the empty folder on the file host. No connector has an operation that removes a folder.

## Decision

### The Media Host After A Follow (Finding 1)

1. **Follow the Move** and **Use This File Host** can give the feed a connected file host. Then Studio also examines the media addresses of the feed that it read. When one connected file host gives each media address, the release files host becomes that file host.
2. When no connected file host gives each media address, the release files host does not change.
3. Items 1 and 2 change only the collection, as ADR 0211 item 9 gives. They write nothing to a file host.

### Send The Podping Again (Finding 2)

4. A podping line with the state "Sent" has no button while Music Index has not answered.
5. In the notice after a write, a line with the state "Failed" has **Send the Podping Again**. A sent line gets it in the state "no reply" of ADR 0175 item 1.
6. In the notice after a move, a failed line always has **Send the Podping Again**. A sent line gets it after the wait of the write notice (`WRITE_NOTICE_AGE`, 5 minutes) from the time of the podping.
7. The button is in the podping line, at its right. The address in the line becomes shorter to give the button its space. The button does not add a row, so the notice keeps its height when the button comes or goes (ADR 0228 item 10).

### Empty Folders On A File Host (Finding 3)

8. After Studio deletes a file from a file host with folders, it removes each folder that the delete left empty. It starts at the folder of the file and goes up.
9. Studio never removes a folder that holds a file or a folder. Studio never removes the root folder of the file host.
10. The SFTP connector and the local connector remove an empty folder. A connector with no folders (S3, Blossom and nsite) does nothing.
11. A failure to remove a folder does not stop the delete and gives no error to the user. The core logs it.

## Consequences

- The tab **Files** names the host that holds the media files after a follow.
- The notice shows **Send the Podping Again** only when Music Index did not follow the podping or the podping failed.
- A delete leaves no empty folder on an SFTP host. A folder that a different app uses stays, because it holds files.

## Alternatives Considered

- The button only on a failed line. Not chosen: a sent podping that Music Index does not follow then has no button, and that was the case of ADR 0191.
- A folder removal only after the delete of a leftover copy. Not chosen: the delete of old files after a move leaves the same empty folders.

## Checks

Mechanical:

- Core test: a follow to a host that gives each media address changes the release files host. A follow with media addresses on no connected host keeps it.
- UI test: the waiting notice has no podping action. The no reply notice and a failed line have **Send the Podping Again**. A move line gets it only after the wait.
- Connector suite test: a delete removes the empty folders up to the root, and keeps a folder with a different file.

Visual:

- Walk screenshots of the notice after a write in the waiting and no reply states, at 1280 px and 390 px. The notice keeps one height.
- Human check: delete a test file on the real SFTP host, and make sure that its empty folder goes.

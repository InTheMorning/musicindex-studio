# ADR 0148: A Move Keeps The Old Files Until The User Reviews The Deletion

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Accepted: 2026-11-06 by the operator, from a mockup and two questionnaires.
- Technical choices: items 8, 9, 13, 17 (the publisher feed) and 19 are choices of the planning model.
- Changes: the deletion of a move in [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) item 12. [ADR 0091](0091-a-change-set-edits-one-recipient-in-many-feeds.md) now also holds a move of releases. This ADR answers the open question of [ADR 0009](0009-studio-connects-to-hosts-and-moves-files.md) about the old copy of a moved file.
- Changed by: [ADR 0177](0177-the-files-page-has-one-way-to-move.md). Item 1: the sheet has "What moves", and the button is in the section "File hosts".
- Source: [findings of local use, finding 14](../reviews/2026-11-06-local-use-findings.md)

## Context

A move of a release needed one action for each file, and one more for the feed. **Move to** deleted the old copy in the same write, so an app with the old address got an error at once.

Studio can examine the new files, musicindex and the time since the move. It cannot examine each app. In the [rc.17 human test](../reviews/2026-11-05-rc17-human-test-findings.md) finding 5, two apps did not show a move.

## Decision

### Move One Release

1. The Files page has the button "Move This Release…". It opens one sheet with a select of the file host, the plan, the waiting time and the check box of the deletion.
2. The plan gives the feed file and the count and the size of the audio files and the images. It also gives the files that Studio must download first, and the files that do not move.
3. A file that is not in the library does not move, and the plan names it. A file that is already on the file host does not change. A Blossom host takes no feed file, so the feed file stays and the plan says so.
4. "Add to the Draft" puts each change in the draft. Review shows them, and nothing changes on a file host before the write.

### The Old Files

5. The check box "Delete the files on {host} when the move is complete" starts on. With the box off, the old files stay, and Studio does not ask again.
6. The waiting time is a select menu with 7, 30 and 90 days. It starts at 30 days.
7. A write that moves a file deletes no old copy. Studio records the old copy, the file host, the time and the waiting time. **Move to** on one file follows this rule, with 30 days.
8. The Files page lists a move that waits, with three lines: the new files, musicindex and the days.
9. A move is ready when its days are complete and musicindex does not give a different feed address. With no answer from musicindex, or with no entry for this feed, the days decide.
10. A ready move shows a notice with "Review the Deletion". This button puts a deletion for each old copy in the draft. The user reviews the diff and writes it.
11. "Delete Now…" does the same before the move is ready, after a confirmation sheet. "Keep the Files" removes the record, and the files stay.
12. Studio deletes an old copy only through `pipeline::write_draft`, with the ID of a diff that the user approved. It never deletes by itself.
13. A write that puts a file again at the path of a recorded old copy removes that record. The existing refusal of a deletion still applies to a file that a feed uses.
14. The pointer file of a feed stays, as before.

### A Part Of A Release Moves

15. A draft can move the feed file when other files of the release stay on the old file host. The Files page then shows a notice with the count and "Move the Files Too".

### Move All Releases

16. The page of a file host has "Move All Releases…". Its sheet has the select of the file host, the waiting time and the check box.
17. The result is a change set, with one row and one diff for each feed on that file host, also a publisher feed. Only the files on that file host move. Each feed writes through `pipeline::write_draft` with the ID of its own diff.
18. Discard puts back the draft of each feed from before the change, and removes the host operations of the move.
19. A release that has an open change set, or that Studio cannot start, stays out of the change set, and its row gives the cause. A draft that already moves its feed file also stays out, because Discard could not put that move back.

## Alternatives Considered

- A warning only. Not chosen by the operator: a release with many files would still need one action for each file.
- A deletion at the write. Not chosen by the operator: an active feed must not lose its files abruptly.
- An automatic deletion after the waiting time. Not chosen by the operator: it needs an open app and an open vault, and the user sees no diff.
- A fixed waiting time. Not chosen by the operator: a user who leaves a file host soon needs a shorter time.

## Consequences

- The file host from before keeps the old files for the waiting time, so a user can pay for two file hosts during that time.
- A check of musicindex does not prove that each app follows the move. The waiting time covers the other apps, and no measurement supports a specific number of days.
- A deletion is a second write, so a user must return to Studio to complete a move.

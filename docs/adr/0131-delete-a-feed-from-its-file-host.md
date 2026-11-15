# ADR 0131: Delete A Feed From Its File Host

- Status: Accepted
- Date: 2026-11-05
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-05 by the operator, through the questionnaire. The operator asked that the warnings say what a loss of control of a feed means.
- Changed by: [ADR 0198](0198-the-review-of-a-delete-names-the-delete-and-a-deleted-feed-shows-only-what-it-can-do.md) items 8 and 12 for a feed with a done delete.
- Changed by: [ADR 0200](0200-studio-deletes-a-feed-file-only-after-music-index-removes-the-entry.md) items 1, 4, 6 and 9: no delete of a feed that Music Index holds.
- Revised: 2026-11-15 by operator decision. Items 1 to 4, 7 and 10 have the texts that the operator selected on 2026-11-14, from drawings. Items 15 to 19 give the domain line. Item 16 has the words that the operator selected on 2026-11-15 for a feed with a done delete. These rules were in ADR 0208, which is deleted.
- Uses: [ADR 0007](0007-draft-diff-write-and-announce.md) for the review. [ADR 0038](../archive/adr/0038-published-feeds-downloads-folder-scan-and-feed-moves.md) for a move. [ADR 0063](0063-one-switch-hides-a-feed-from-apps.md) for the block tags. Stophammer ADR 0052, ADR 0056, ADR 0057, ADR 0067 and ADR 0079 own the rules of the index. This record cites them and does not restate them as rules of Studio.

## Context

Studio cannot delete a feed file from a file host. "Remove from Collection" changes no file. The operator asked for a delete with strong warnings. The delete must say that it is not the way to move a feed or to remove it from an index.

On 2026-11-05 the examination of the Stophammer repository found these facts for musicindex:

- A `404` retires a record only for a host in `SOURCE_GONE_HOSTS`. `api.musicindex.org` names only `wavlake.com`. Two `410` answers 24 hours apart retire a record from any host.
- Only content from the source URL can change a record.
- `podcast:block` with the value `yes`, read at the source URL, retires the record. After that, only that URL can bring the GUID back.
- A move needs a file at the earlier address that names the new address.
- The public proof flow is gone. Only a person with the admin token can delete a record.

Studio ADR 0063 says that Podcast Index acts on `itunes:block`. The examination found no check that Studio can make for Podcast Index. It did not examine what Podcast Index does when a file is gone.

Two risks are conclusions from these facts, and no test has shown them. The next holder of the address can publish a file with the GUID of the feed and different payment recipients. An app can continue to send payments to the recipients in an entry that is stale.

## Decision

### The First Sheet

1. The action "Delete from File Host" is on the page of a feed with a connector. It opens a sheet with the title "What do you want to do?". The sheet has no paragraph before its rows. It has three rows, and each row is a goal with one line of text. Only the last row is destructive. No row has the tag "Recommended".
2. The first row is "Put the feed on a different file host", with "Apps and Music Index follow. Nothing is lost." It opens the move of ADR 0038.
3. The second row is "Take the feed out of the apps", with "The file stays. You can stop this later." It starts the retire.
4. The third row is "Remove the file", with "Music Index removed the entry." It opens the sheet of item 10. It shows only when `removal_done` is true (ADR 0200 item 4). The domain line of item 15 is below the rows.

### Retire

5. A retire sets "Hide from apps" (ADR 0063) in the draft, and the user reviews and writes it. It is a usual write, with its announcement.
6. The page shows three steps: the feed asks to be removed, musicindex removes the entry, and the delete of the file. The second step reads `GET /v1/feeds/{guid}` when the page opens and when the user selects "Check Again". A request goes out only then.
7. The head of the Retire block says "Studio cannot check Podcast Index." The domain line follows it.
8. "Stop Retiring" clears the switch in the draft, and the user reviews and writes it.
9. Before musicindex removes the entry, the third step offers only "Delete Anyway". It opens the sheet of item 10.

### The Delete

10. A final sheet, "Delete the Feed File", names the host and the path. It shows three state points, and the domain line of item 15 below them. Its button stays off until the user types the exact title of the feed. A feed with no title uses its file name. The points are:
    - "✓ Music Index removed the entry"
    - "✓ Studio keeps a copy of the feed"
    - "↺ You can put the file back later with **Put the File Back** on this feed"
11. One check box, off at the start, adds the audio and the image files of the release. A file that a different feed uses, or a pointer file, stays. The sheet counts each of them.
12. The delete goes through `pipeline::write_draft`, with the ID of a diff that the user approved (ADR 0007). A snapshot of the file comes first. Studio reads the file before the delete, and the conflict check of ADR 0007 stops a delete when the file changed.
13. The core refuses the delete of a pointer file of a move. A publisher feed in the collection that links to this feed gets a line in the sheet.
14. After the delete, the feed stays in the collection as read-only, with its last copy. A write to the same address brings the file back. The text says that a different address does not give the entry back.

### The Domain Line

15. The domain line is one warning with these words: "Keep the domain {host}. If it expires, its new owner can take over this feed and its payments." `{host}` is the host of the public address of the feed.
16. For a feed with a done delete, the words are: "Keep the domain {host}. If it expires, its new owner can publish a feed with the same GUID and collect the payments that apps still send to it." The user has no feed at the address then, so the words of item 15 do not give the danger.
17. The domain line has the style of a warning: the triangle icon and the Warning color ([ADR 0191](0191-a-return-warning-has-the-warning-color-and-a-sent-podping-can-go-again.md) item 1, musicindex ADR 0068 R-STATUS-1).
18. The core gives the host as `address_host` on the feed row of the API. The UI does not get a host from an address. A feed with no public address, or with a placeholder address, has no `address_host`, and the line does not show.
19. The domain line shows in four places: the first sheet, the final sheet, the Retire block and the move sheet. The sheet "Move This Release…" ([ADR 0148](0148-a-move-keeps-the-old-files-until-the-user-reviews-the-deletion.md)) shows it below the plan when the plan moves the feed file (`plan.feed.moves`). The host there is the host of the address before the move.

## Alternatives Considered

- A warning with two links. Not chosen by the operator: a person can pass the links.
- A short sheet with the text in a row that opens. Not chosen by the operator: the part that matters most can go unread.
- Plain text in the place of the status of a retire. Not chosen by the operator: the user gets no evidence.
- A check box as the final check. Not chosen by the operator: people select it with no thought.
- Two short paragraphs, or a list "You lose these things:", on the final sheet. Not chosen by the operator: the list tells the user what to fear, and not what to do. ADR 0200 also made its first item false.
- "Keep the old address." as the domain line. Not chosen by the operator: too vague. The line must name the loss of the domain.
- The rows keep their action names (**Move It to a Different File Host**, **Retire It**, **Delete the File Only**). Not chosen: "Retire" is a word that the user must learn.
- Studio forgets the feed after the delete. Not chosen by the operator: Studio then has no copy to put back.

## Consequences

- The user reads one sentence about the one risk that stays, with the name of the domain.
- The words of items 15 and 16 give a conclusion that no test has shown.
- A `404` on a usual host leaves an entry in musicindex. Studio cannot change this.
- The text of a sheet can become wrong when a rule of an index changes. A change of the Stophammer records needs an examination of these texts.

## Invariants

- No delete occurs without a diff that the user approved (ADR 0007).
- Studio makes no index request unless the user opens the retire page or selects "Check Again".
- A text about Podcast Index states only what the sources show.

## Acceptance Criteria

Mechanical:

- A core test: a delete needs an approved diff, keeps a snapshot, and stops when the file changed.
- A core test: the delete refuses a pointer file, and keeps a file that a different feed uses.
- A core test: after the delete, the feed is read-only with its last copy, and a write to the same address brings the file back.
- A Node test: the model gives the steps of a retire from the state of the core, and the button waits for the exact title.
- A Node test: the rows of the first sheet have the titles and texts of items 2 to 4 and no tag, and the third row shows only with `removal_done: true`.
- A Node test: the domain line has the words of item 15 for a host, and no line for no host. The three points of item 10 are there, and the UI has no "You lose these things".
- A Node scan test: the two sheets, the Retire block and the move sheet use the domain line function.
- A Rust test: the feed row gives `address_host` for a public address, and none for a placeholder address.
- No test examines the words of item 16 at this date. The task for Note 8 of the [walk notes of 2026-11-15](../reviews/2026-11-15-walk-notes.md) adds it.

Visual, by the planning model: each sheet and the retire page, at 1280 px and at 390 px with touch.

Human, by the operator: retire and delete a test feed on a real file host. Read the index before and after each step.

# ADR 0131: Delete A Feed From Its File Host

- Status: Accepted
- Date: 2026-11-05
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-05 by the operator, through the questionnaire. The operator asked that the warnings say what a loss of control of a feed means.
- Uses: [ADR 0007](0007-draft-diff-write-and-announce.md) for the review. [ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md) for a move. [ADR 0063](0063-one-switch-hides-a-feed-from-apps.md) for the block tags. Stophammer ADR 0052, ADR 0056, ADR 0057, ADR 0067 and ADR 0079 own the rules of the index. This record cites them and does not restate them as rules of Studio.

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

1. The action "Delete from File Host" is on the page of a feed with a connector. It opens a sheet that starts with the text of item 2, then three rows. Delete is the last row, and only that row is destructive.
2. The text says that the address is the control of the feed. musicindex keeps an entry when the file is gone, and Podcast Index can also keep it. The entry keeps its last payment recipients. With no file, the user cannot correct the entry, retire it or move it.
3. "Move It to a Different File Host" opens the move of ADR 0038. Its text says that the user keeps control, and that the index follows. An app can stay at the earlier address.
4. "Retire It" is the recommended row. "Delete the File Only" says that the user loses control of the entry.

### Retire

5. A retire sets "Hide from apps" (ADR 0063) in the draft, and the user reviews and writes it. It is a usual write, with its announcement.
6. The page shows three steps: the feed asks to be removed, musicindex removes the entry, and the delete of the file. The second step reads `GET /v1/feeds/{guid}` when the page opens and when the user selects "Check Again". A request goes out only then.
7. The page says that Studio cannot check Podcast Index. It says that the user must keep the address, also after the delete, and why.
8. "Stop Retiring" clears the switch in the draft, and the user reviews and writes it.
9. Before musicindex removes the entry, the third step offers only "Delete Anyway". It opens the sheet of item 10.

### The Delete

10. A final sheet names the host and the path. It lists what the user loses, in the words of items 2 and 7. Its button stays off until the user types the exact title of the feed. A feed with no title uses its file name.
11. One check box, off at the start, adds the audio and the image files of the release. A file that a different feed uses, or a pointer file, stays. The sheet counts each of them.
12. The delete goes through `pipeline::write_draft`, with the ID of a diff that the user approved (ADR 0007). A snapshot of the file comes first. Studio reads the file before the delete, and the conflict check of ADR 0007 stops a delete when the file changed.
13. The core refuses the delete of a pointer file of a move. A publisher feed in the collection that links to this feed gets a line in the sheet.
14. After the delete, the feed stays in the collection as read-only, with its last copy. A write to the same address brings the file back. The text says that a different address does not give the entry back.

## Alternatives Considered

- A warning with two links. Not chosen by the operator: a person can pass the links.
- A short sheet with the text in a row that opens. Not chosen by the operator: the part that matters most can go unread.
- Plain text in the place of the status of a retire. Not chosen by the operator: the user gets no evidence.
- A check box as the final check. Not chosen by the operator: people select it with no thought.
- Studio forgets the feed after the delete. Not chosen by the operator: Studio then has no copy to put back.

## Consequences

- The operator accepted a long sheet, because this is the one location where the text must be read.
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

Visual, by the planning model: each sheet and the retire page, at 1280 px and at 390 px with touch.

Human, by the operator: retire and delete a test feed on a real file host. Read the index before and after each step.

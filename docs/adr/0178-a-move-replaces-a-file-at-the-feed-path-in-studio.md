# ADR 0178: A Move Replaces A File At The Feed Path In Studio

- Status: Accepted
- Date: 2026-11-11
- Owner: operator
- Accepted: 2026-11-11 by the operator, through the questionnaire of the human checks. The operator said that Studio must do this work. The user must not use a different tool.
- Technical choices: items 2, 4, 6 and 7 are choices of the planning model.
- Changes: [ADR 0126](0126-studio-refuses-a-bad-move-link.md) items 19 and 21. A file at the feed path with no record no longer stops a move.
- Uses: [ADR 0007](0007-draft-diff-write-and-announce.md) (no write without a diff), [ADR 0148](0148-a-move-keeps-the-old-files-until-the-user-reviews-the-deletion.md) for the move sheet.
- Source: finding 4 of the [human checks of 2026-11-11](../reviews/2026-11-11-local-human-test-findings.md).

## Context

On 2026-11-11, the operator moved only the feed file of a test release to a second file host. The second host had an old copy of the same feed at the same path, from an earlier move. The copy had the `podcast:guid` of the feed. The collection had no record of the copy, so `set_feed_host` refused the move with `FileExists`. The sheet showed "Something went wrong." The user could continue only with a different tool.

## Decision

### A Copy Of The Same Feed

1. The new host can have a file at the feed path that the collection has no record of. Then `set_feed_host` reads that file from the new host.
2. When the file is a feed with the `podcast:guid` of the draft, the move continues. The collection records the file as a placed file of the feed (ADR 0126 item 19). The write then replaces it, as ADR 0126 item 19 gives.
3. Review shows "Replaces an old copy of this feed" on the row of the feed file.

### A Different File

4. When the file has a different `podcast:guid`, or it is not a feed, `set_feed_host` refuses with `FileExists`. The refusal gives the path, the host, the title of the file when it is a feed, and its `podcast:guid` when it has one.
5. The move sheet shows this refusal with **Replace the File**. It does not show "Something went wrong."
6. **Replace the File** puts the move in the draft with a mark on that path. The core reads the file again at that time, and it stores the title, the GUID and the size of the file. Review shows the row with a warning that gives the title or the path. Nothing changes on a host before the write.
7. The write replaces the file only when the file on the host still has the GUID and the size that the core stored. Otherwise the write gives `FileExists`, and the sheet asks again.

### The Sheet

8. **Add to the Draft** is not available while the sheet shows a refusal.

## Alternatives Considered

- Refuse a different file with a clear cause. Not chosen by the operator: the user then cannot continue in Studio.
- Replace a different file with no question. Not chosen: the file can be the feed of a different release.

## Consequences

- A move to a host with an old copy of the same feed needs no other tool.
- A move reads one file from the new host before the draft gets the move. With no answer from the host, the refusal of ADR 0126 item 19 stays.

## Acceptance Criteria

Mechanical:

- A Rust test: a file with the GUID of the draft and with no record at the feed path gives a move. The write replaces the file.
- A Rust test: a file with a different GUID gives `FileExists` with the title and the GUID. With the mark of item 6, the write replaces it. When the file changed after the mark, the write gives `FileExists`.
- A Node test: the UI model of the move sheet gives **Replace the File** for that refusal, and **Add to the Draft** is not available.

Visual, by the planning model:

- The move sheet with the refusal and **Replace the File**, and the Review row of item 3 and item 6, at 1280 px and at 390 px.

Human, by the operator. This gate stays open until it runs:

- In the desktop app, move only the feed file of "HeyCitizen's Test Album" to `heycitizen`. The old copy at `TestAlbum/test.xml` becomes the feed.

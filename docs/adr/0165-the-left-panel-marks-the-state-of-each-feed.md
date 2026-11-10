# ADR 0165: The Left Panel Marks The State Of Each Feed

- Status: Accepted
- Date: 2026-11-10
- Owner: operator
- Accepted: 2026-11-10 by the operator, through the questionnaire. The operator selected "Exceptions only" and a mark that covers the feed and its releases. The operator selected the mockup with the pencil, the dashed circle and the red cloud. The operator also selected a startup check of the artist and label pages.
- Amended: 2026-11-10 by the operator. Items 9 and 10 make the pencil current after each save.
- Technical choices: items 6, 7 and the time of item 10 are choices of the planning model.
- Changes: [ADR 0150](0150-studio-shows-a-change-from-outside-when-a-page-opens.md) item 1.
- Uses: [ADR 0150](0150-studio-shows-a-change-from-outside-when-a-page-opens.md) for the check. [ADR 0088](0088-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) for the rows of the left panel. musicindex ADR 0032 R-DOT-1 keeps the dot for the type only.

## Context

A feed of Studio has one of three states. It is up to date, it has changes that are not written, or Studio did not check it against the hosted file. The Releases page shows the state of each release as text on its tile. The left panel showed no state, so a user had to open each group to find work.

The check of ADR 0150 read only the releases of the Releases page. Thus each artist and label page stayed not checked until the user opened it.

v4vmm shows the download state of a row as one icon in one column, with a tooltip and a label (v4vmm ADR 0083 item 6). The type dot shows only the type (musicindex ADR 0032 R-DOT-1), so a state cannot be a second dot.

## Decision

### The Marks

1. A row of the left panel shows at most one state mark, between its name and its count. A feed that is up to date has no mark.
2. The marks, from the most urgent:
    - **Changed outside Studio**: a cloud with "!", in the `--error` color. The check of ADR 0150 found the state "changed" or "moved". The tooltip gives "Changed outside Studio" or "Moved outside Studio".
    - **Changes not written**: a pencil, in the `--accent` color. The draft state is "new" or "changed".
    - **Not checked**: a dashed circle, in the `--muted` color. Since the start, Studio did not read the hosted file correctly. The check runs, the read did not succeed, the feed has no public address, or the check is off in Settings.
3. A row of an artist, a label or a hosting feed covers its feed and each of its releases. The row shows the most urgent mark of these feeds. "Not linked" and "Other feeds" cover the feeds that they hold. A release row of a level covers its feed only.
4. Each mark has a tooltip and an accessible name. The accessible name of the row adds the state. The shape of each mark is different, so the color is not the only sign.

### The Check At The Start

5. After the feed list loads, Studio checks each feed with a public address: the releases, the artist, label and hosting pages, and the other feeds. Each open of the Releases page checks again. This changes ADR 0150 item 1. ADR 0150 items 6, 7, 10 and 11 do not change: at most 4 reads at one time, no worker, and no request with the check off.
6. One store in the shell keeps the results. The left panel and the tiles of Releases read the same results.
7. The response of `GET /feeds/{id}/outside` gives `read`. It is true when the read of the hosted file succeeded in this request.

### The Pencil Stays Current

9. Each draft answer of the core gives `draft_state`, with the same value as the feed list. The UI puts this value in the row of that feed in the list. Thus the pencil shows or goes after each save, with no new read of the list. The core stays the source of the state (ADR 0065).
10. The feed list loads again when the window gets the focus again. Thus a change from a different window shows. At most one load runs in 30 seconds.

### Not In This Decision

8. A subsequent ADR decides the button "Check Everything" of the Releases page, the check of the media files and the report of the results. That ADR needs a mockup of the report first.

## Alternatives Considered

- **One icon for each state, also "up to date".** Not chosen by the operator: each row then has a mark, and the user does not see the marks that need work.
- **The state as words in the second line.** Not chosen by the operator: the line gets long, and it mixes two kinds of value.
- **A mark for the feed only.** Not chosen by the operator: the user must open each group to find a release with work.
- **No check of the artist and label pages at the start.** Not chosen by the operator: almost each artist row then shows "Not checked".

## Consequences

- Each start of Studio sends one read for each feed with a public address. Each file host can see these reads in its log.
- A function of the UI gives the mark of a set of feeds from the draft states and the results of the check.

## Acceptance Criteria

Mechanical:

- The mark function gives these results. A more urgent state wins.
    - "outside" for a set with a changed or a moved feed
    - "pending" for a set with a draft that is not written
    - "unchecked" for a set with a feed that Studio did not read correctly
    - no mark for a set of feeds that are up to date
- The set of a publisher row holds the feed and each ID of `release_ids`.
- The route gives `read: true` after a correct read. It gives `read: false` when the read does not succeed or the feed has no public address.
- Each draft answer gives `draft_state`: "unchanged" after the open of a written feed, and "changed" after a save that changes it.
- The Node tests and the Rust tests pass.

Visual, from agent screenshots at 1280 px in the dark and the light theme, and at 390 px:

- The marks show between the name and the count, as in the mockup.
- An up-to-date row has no mark.
- A tooltip shows on each mark.
- After an edit of a feed with no draft, the pencil shows on its row with no reload of the list.

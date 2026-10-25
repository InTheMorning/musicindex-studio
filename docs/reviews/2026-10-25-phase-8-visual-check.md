# Phase 8 Visual Check

- Date: 2026-10-25
- Reviewed: the visual check of Phase 8 in the [Studio v1 phase plan](../plans/studio-v1-phase-plan.md) and of [task 025](../tasks/studio-v1-task-025-publisher-feeds.md): the user makes a label feed and an artist feed for the releases in the collection, and writes each feed
- Decision: [ADR 0041](../adr/0041-publisher-feeds-and-two-way-links.md)
- Host: a new local folder with the public address `https://music.example/feeds`, and a new data folder
- Checker: the agent, with `tools/drive-ui.mjs` in headless Chromium. Node.js was not on the PATH, so Bun ran the script.
- Result: pass

## Procedure

The agent made the vault, the library, the host and two releases through the API, and wrote the two releases. It did each other step in the UI.

| Step | Procedure | Result |
| --- | --- | --- |
| 1 | In "Add feeds", make the label feed `label.xml` with the role Label. Add "Album One" and "Album Two" on its page. | Pass. Each add showed a notice that named the release. |
| 2 | In "Add feeds", make the artist feed `artist.xml` with the role Artist. Add the two releases on its page. | Pass |
| 3 | On the album page of "Album One", select "Move down" on "Check Label". Examine the review. | Pass. "Check Artist" became the primary publisher feed, and the review showed it first in `podcast:publisher`. |
| 4 | Write each of the four feeds with "Write to the host" in its review. | Pass. Each feed then showed "No changes". |
| 5 | Examine the album page and the publisher page in the light theme | Pass |

## The Files On The Host

- `label.xml` and `artist.xml` have `podcast:medium` `publisher`, and one `remoteItem` with `medium="music"` for each release. The `rel` is `label` in the label feed and `artist` in the artist feed.
- `album-one.xml` has `podcast:publisher` with the artist feed first, then the label feed. `album-two.xml` has the label feed first, then the artist feed.
- Each `feedGuid` is the `podcast:guid` of the feed that it names. Each link has the same `rel` on its two sides.

## Open Items

- No step examined the removal of a link that Studio did not make, or the text for `role_required`. The code of the two pages has these paths.
- A feed from a host has no medium before its first read. Thus the pages do not offer it for a link until then.
- The pages show one notice for each call. After two adds, only the notice of the second add shows.
- In the check of packet 025c, the first load of a draft in a new browser profile was slow. The cause is not known.

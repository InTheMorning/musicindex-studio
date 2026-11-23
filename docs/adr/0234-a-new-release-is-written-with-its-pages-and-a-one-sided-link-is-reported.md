# ADR 0234: A New Release Is Written With Its Pages, And A One-Sided Link Is Reported

- Status: Accepted
- Date: 2026-11-20
- Owner: operator
- Accepted: 2026-11-20 by the operator, after the desktop check of 9af319b (`docs/reviews/2026-11-20-rc29-human-check.md`, section "Desktop Check Of 9af319b", findings 1 to 5). The operator selected items 1, 3, 5 and 8 from drawings in the questionnaire.
- Technical choices: the main session selected items 2, 4, 6, 7 and 9 to 13. The operator did not receive them as a question. On 2026-11-20, before the first push, the main session added the kept bytes of the last write to item 7. A test showed that a feed made in Studio had no written copy.
- Bar: crosses modules. It changes the core remove, the move route, the feed list, the check, the Review sheet and the level rows. Without this record, an agent can keep ADR 0231 item 10 and let a release go out with no link on its artist page.
- Changes: [ADR 0231](0231-a-new-release-writes-its-artist-page-and-the-files-of-a-restored-release.md) item 10, the write of a new release without its pages (items 3 and 4).
- Changes: [ADR 0233](0233-each-add-form-finds-on-music-index-and-copies-the-found-feed.md) item 15, the remove of a release that was never written (item 10).
- Changed by: [ADR 0251](0251-review-writes-the-two-sides-of-a-changed-link.md) items 4 and 6, on 2026-11-23. Item 3: the check box is on and locked for each linked feed, and the text under the rows changes.
- Uses: [ADR 0227](0227-undo-and-redo.md), the undo of a remove. [ADR 0230](0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 20 and 21, the media host of a move. [ADR 0007](0007-draft-diff-write-and-announce.md), the write of a draft.

## Context

The operator did the desktop check of 9af319b on 2026-11-20. These findings need a decision or a correction:

- Finding 1: Review of a release that was never written shows **Discard the Draft**. A click gives "Studio could not write the release." with the detail "This release has no file on the file host yet. To remove it, remove it from the collection." `delete_draft` refuses a draft with no base version (`crates/api/src/routes.rs` near line 3615).
- Finding 2: the operator wrote a new release with "Also write HeyCitizen" off (ADR 0231 item 10). The release file names HeyCitizen in `podcast:publisher`, and the left panel shows the release below HeyCitizen. The written file of HeyCitizen does not name the release. **Issues** of the release says "Studio found no problem in this feed."
- Finding 3: the level of HeyCitizen lists a release of Matt Finlay with no artist name. The feed list gives no `itunes:author` (`crates/api/src/routes.rs` near line 1506).
- Finding 4: after the delete from the file host and the remove of that written release, the draft of HeyCitizen keeps its remote item. ADR 0233 item 15 removes such an item only for a release that was never written (`crates/core/src/people.rs` near line 3864).
- Finding 5: **Move This Release…** with "Media files only" to the host that has the files adds no operation to the draft. `move_release` still keeps the earlier media host before it sets the target (`crates/api/src/routes.rs` near line 5910). With no draft, no write and no discard clears the kept value. The collection of the operator has `files_connector_kept` 2 for the test album after that move.

## Decision

### Review Of A Release That Was Never Written (Finding 1)

1. Review of a release that was never written shows **Remove from Collection…** in the place of **Discard the Draft**. It opens the same confirmation as **Feed facts**, removes the release, closes Review and shows the notice with **Undo**.
2. When a discard fails, the error notice says "Studio could not discard the draft." It does not name a write.

### The First Write Of A New Release (Finding 2)

3. The first write of a new release writes each linked page that has a draft with the link to the release (ADR 0231 item 7). Each check box "Also write {title}" is on and disabled for this write. The text under the rows is "A new release is written with its pages."
4. ADR 0231 item 9 stays. When the write of the release fails, Studio writes no page. When the write of a page fails, the release stays written, and Review shows the error of that page. Item 5 then reports the link.

### A One-Sided Link Is Reported (Finding 2)

5. A written release that names a page of the collection in `podcast:publisher`, when the written file of that page does not name the release, has the finding "{page} does not list this release." on **Issues**. The reason is "Apps show the release below {page} only after a write of {page}." Its button is **Write {page}…**.
6. **Write {page}…** adds the remote item of the release to the draft of the page, when the draft does not have it. Studio starts a draft of the page when it has none. The release does not change. Studio then opens Review of the page.
7. Studio compares the release with the written file of the page: the copy in the library from the last write or read. A feed made in Studio has no release folder and no library copy. For such a feed, the collection keeps the bytes of its last write, and Studio uses them. A draft of the page does not end the finding, because the file host does not have it yet.
8. The row of the release below that page shows the line "⚠ {page} does not list it yet".

### The Artist Name Of A Release Row (Finding 3)

9. The feed list gives the artist name of each release: `itunes:author` of the draft, or else of the written file of item 7.
10. A release row below an artist page shows the line "Artist: {name}" when that name differs from the title of the artist page. Studio compares the two names with no change of case and with no space at the two ends. A row with the same name or with no name has no such line.

### The Remove Of A Release Clears A Link That Only A Draft Has (Finding 4)

11. **Remove from Collection** of a release removes its remote item from the draft of each page whose written file does not name the release. This applies to a release that was never written (ADR 0233 item 15) and to a written release. A link that the written file of the page has stays in the draft, because only a write of the page can remove it. When such a draft then has no change except the build date, the draft goes. **Undo** gives back the release and these drafts.

### A Move That Adds Nothing To The Draft (Finding 5)

12. When **Move This Release…** adds no operation and no feed move to the draft, it sets the media host to the target and clears the kept media host. Studio keeps no earlier host, because no discard can follow. A move that adds an operation keeps the earlier host as before (ADR 0230 item 20).
13. A migration clears each kept media host of a feed with no draft operation and no draft feed move. It repairs the collections that a move of item 12 changed before this decision.

## Item Map

| Earlier item | Change by this ADR |
| --- | --- |
| ADR 0231 item 10 | Item 3: the check box is on and disabled for the first write. |
| ADR 0233 item 15 | Item 11: the remove clears a link that only a draft has, also for a written release. |

## Consequences

- A new release cannot go out from Studio with no link on its pages. A failed page write and a change from outside Studio still give a one-sided link. Items 5 to 8 show it and give the repair.
- A release of a different artist below an artist page shows its own artist name.
- The kept media host of the test album of the operator goes at the migration.

## Alternatives Considered

- Report only, and keep the check box free. Not chosen by the operator.
- Lock the check box with no report. Not chosen by the operator.
- The artist name in the kind line ("Album · Matt Finlay · 7 tracks"). Not chosen by the operator: one labeled line for each value kind is clearer.
- No button in the place of **Discard the Draft**. Not chosen by the operator.
- Remove from the draft of a page each link to a removed release, also when the written file has it. Not chosen: the draft then removes a link that is on the file host, and the user did not ask for that write.

## Checks

Mechanical:

- UI test (`crates/api/ui-tests/`): `alsoRows` gives each row on and disabled, with the text of item 3.
- UI test: the Review model of a release that was never written gives the action **Remove from Collection…** and no **Discard the Draft**. A failed discard gives the title of item 2.
- UI test: the check rows give the button **Write {page}…** for the finding of item 5.
- UI test: the level row model gives "Artist: {name}" only when the name differs, and the line of item 8 for each page in `unlisted_by`.
- API test: the feed list gives `author` and `unlisted_by` of item 5 for a written release whose page has no remote item for it.
- API test: the check gives the finding of item 5. The route of item 6 adds the remote item to the draft of the page and does not change the release.
- API or core test: **Remove from Collection** of a written release removes its remote item from a page draft whose written file does not name it, and keeps a link that the written file has. **Undo** gives back both.
- API test: a move to the host that has the files sets the media host and gives `files_connector_kept` NULL. A later move and a discard keep the target host.
- Core test: the migration of item 13 clears a kept value with no draft operation and keeps one with a draft operation.

Visual, by the main session with screenshots at 1100 px and 390 px:

- Review of a new release with the disabled check box, and with **Remove from Collection…**.
- A level row with "Artist: {name}" and with the line of item 8.
- **Issues** with the finding of item 5.

Human check: the next desktop check repeats the discard of a move on the test album, and reads the kept media host after the migration.

# ADR 0232: Each Group Below An Open Feed Has An Add Row

- Status: Accepted
- Date: 2026-11-20
- Owner: operator
- Accepted: 2026-11-20 by the operator, after the desktop check of the commits up to `2d4fb25` (`docs/reviews/2026-11-20-rc29-human-check.md`, section "Desktop Check Of The Commits Up To 2d4fb25", findings 1 and 2). The operator selected items 1, 8 and 9 from drawings in the questionnaire.
- Technical choices: the main session selected the groups of item 2, the labels of item 6 and the rules of items 10 to 12. The operator did not receive them as a question.
- Bar: crosses modules. It changes the core levels, the collection tables, the routes and the UI. Without this record, an agent can write the label link into a feed file, against Stophammer ADR 0069.
- Changes: [ADR 0215](0215-the-left-panel-shows-the-levels-of-the-publisher-feeds.md) items 2, 5 and 6 (items 1 to 12).
- Uses: [ADR 0230](0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 2, 3 and 5, the add menu and the forms. [ADR 0231](0231-a-new-release-writes-its-artist-page-and-the-files-of-a-restored-release.md) item 6, the parent list of a new release. [ADR 0227](0227-undo-and-redo.md), the undo of a remove. Stophammer ADR 0069, section "A direct link between a label feed and an artist feed" (rejected).
- Changed by: [ADR 0233](0233-each-add-form-finds-on-music-index-and-copies-the-found-feed.md) item 3, on 2026-11-20. Item 8: the page **Add One You Have** is no longer a start of a new feed.

## Context

The operator did the desktop check of `2d4fb25` on 2026-11-20. Two findings need a decision:

- Finding 1: on the artist page, the group "Releases" of the left panel has no action that makes a release. A release page has **＋ Add tracks** at the end of its tracks. The operator wants the same rule at each level: a group has an add action for the items of that group.
- Finding 2: the new release "check 212" was made from the level of HeyCitizen. Its back row says **‹ Releases**, not **‹ HeyCitizen**. The cause: `addedFeed` in `crates/api/ui/app.js` opens the new feed with no `from`.

The groups of the left panel before this record:

| Level of the left panel | Groups | Add row |
| --- | --- | --- |
| Top | Hosting, Labels, Artists, Not linked, Other feeds | None |
| Level of a label | Artists, Releases | None |
| Level of an artist | Releases | None |
| Page of an artist | Releases | None |
| Page of a release | Tracks | **＋ Add tracks** |

A feed is below a label only when the two feeds share a release (ADR 0215 item 2). Thus a new artist with no release cannot show below a label. Stophammer ADR 0069 rejects a direct link between a label feed and an artist feed in the feed files. An artist feed can name only one publisher, and the link does not say which albums it covers.

## Decision

### The Add Rows (Finding 1)

1. Each group of the left panel below an open feed ends with an add row. This applies to the level of a publisher feed and to the outline of a publisher feed. The top of the left panel does not change. The page Releases keeps its **＋ Add** menu.
2. A level shows these groups, also when a group has no rows. Then the group shows its heading and its add row only.
   - The level of a feed at the hosting rank: Labels, Artists, Releases.
   - The level of a feed at the rank of a label: Artists, Releases.
   - The level of a feed at the artist rank: Releases.
   - A group of a different role heading shows only when it has rows, and it has no add row. The form of a publisher page offers only the roles artist, label and hosting.
3. The outline of a publisher feed shows the group Releases with its heading, also with no release, and its add row.
4. **＋ Add release** opens the form of a release (ADR 0230 item 3). On an artist, the form starts with that artist as the parent. On a label or a hosting feed, the form has no start parent (ADR 0230 item 2, ADR 0231 item 6).
5. **＋ Add artist** and **＋ Add label** open the form of a publisher page, with the role Artist or Label as the start value. The new page goes below the feed of the level (items 9 to 12).
6. The labels of the add rows are **＋ Add release**, **＋ Add artist** and **＋ Add label**. The add row **＋ Add a Release** in the body of an artist page stays (ADR 0230 item 5).
7. At 760 px or less, the level page and the outline page show the same groups and add rows.

### The Back Row Of A New Feed (Finding 2)

8. A new feed opens with the level where the user made it. This applies to a release, an artist page, a label page, a show and a playlist.
   - Made on a level: the back row names that level.
   - Made on the page of a publisher feed: the back row names the level of that feed. A release row of that outline opens with the same level (ADR 0215 item 6).
   - Made on the top page Releases, in "Not linked", in "Other feeds", on the page Welcome or on the page **Add One You Have**: the back row is **‹ Releases**.

### A Link That Only Studio Keeps

9. When the user makes an artist page or a label page from a level, Studio keeps a link in the collection: the new feed is below the feed of the level. Studio writes nothing in a feed file for this link (Stophammer ADR 0069).
10. The core adds each kept link to the rule of ADR 0215 item 2. Feed B is below feed A when B has a lower rank, and the two feeds share a release or have a kept link. The rule "directly below" and the lists `child_ids` and `parent_ids` do not change.
11. A kept link goes in these conditions:
    - Studio links a release to the two feeds, so a shared release puts the lower feed below the upper feed.
    - The user removes one of the two feeds from the collection. The undo of that remove gives the link back (ADR 0227).
12. Music Index, HitIt and the other apps read only the feed files. They show the new artist below the label only after a shared release.

## Item Map

| ADR 0215 item | Change by this ADR |
| --- | --- |
| 2 | Item 10 adds the kept link to the rule. |
| 5 | Items 1 to 7 add the add rows and the empty groups of a level. |
| 6 | Item 8 gives the level of a new feed. |

## Consequences

- The user can make a release, an artist or a label from the group where it shows.
- A new release goes back to the artist where the user made it.
- A new artist shows below its label in Studio immediately. The other apps show it after its first release on the label.
- The collection gets one table and one migration.

## Alternatives Considered

- An add row in each group of the top panel too. Not chosen by the operator: the top panel gets long.
- An add row only in the group Releases. Not chosen by the operator: the rule must be the same for each group below an open feed.
- The back row of a new release always names its first artist. Not chosen by the operator: a release from the top page must go back to the top page.
- **＋ Add artist** makes the artist and then opens the form of its first release. Not chosen by the operator: the artist must show below the label immediately.
- A link in the feed files, with a change of Stophammer ADR 0069. Not chosen by the operator: it needs changes in three other repositories first.

## Checks

Mechanical:

- UI test (`crates/api/ui-tests/shell.test.mjs`): the panel model of a level gives the groups and the add rows of item 2 for each rank, also with no rows.
- UI test: the outline model of a publisher feed gives the group Releases and its add row with no release.
- UI test: the add row of a level of an artist gives the start parent of that artist. The add row of a label gives no start parent.
- UI test: the view of a new feed gives the `from` of item 8. Test a new feed from a level, from the page of a publisher feed and from the top page.
- Core test: a kept link puts an artist with no release below a label in `child_ids` and `parent_ids`.
- Core test: a link of a release to the two feeds removes the kept link. The remove of a feed removes the kept link, and its undo gives it back.
- API test: the make of an artist page with the feed of a level gives the kept link. A feed of the same rank or a lower rank is refused.

Visual, by the main session with screenshots at 1100 px and 390 px:

- The level of a label with no artist shows the heading Artists and **＋ Add artist**.
- The artist page shows **＋ Add release** at the end of the group Releases.
- A new release made from HeyCitizen shows **‹ HeyCitizen**.

Human check: only the next desktop check of the candidate.

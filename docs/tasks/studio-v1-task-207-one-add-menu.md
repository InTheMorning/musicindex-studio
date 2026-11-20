# Studio v1 Task 207: One Add Menu

Decision: [ADR 0230](../adr/0230-one-add-menu-the-parent-of-a-new-release-and-the-check-after-rc29.md) items 1 to 7.

Status: Open.

## Problem

Findings 2 and 3 of the [check after rc.29](../reviews/2026-11-20-rc29-human-check.md). The page Releases has **＋ Add a Feed**, and a release starts only on an artist page. The head of an artist page has **Open the Feed**.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Menu button | reuse | crates/api/ui/menu.js | `MenuControl` near line 8. `SortMenu` in `app.js` near line 3491 shows a use. |
| Head of Releases | extend | crates/api/ui/app.js | `ReleasesPage` near line 3529. The buttons near lines 3562 to 3564. `more` near line 3535 from `releaseAdd` in `welcome.js:139`. |
| Add choices | extend | crates/api/ui/addfeeds.js | `addChoices` near line 9. Its groups near lines 11 to 24. |
| Add page | extend | crates/api/ui/app.js | `AddFeeds` near line 3057. The route to it near line 4134. |
| Head of a group | reuse | crates/api/ui/shell.js | `releasesList` near line 639 gives `head.typeLabel` near lines 649 to 654. |
| UI tests | extend | crates/api/ui-tests/addfeeds.test.mjs | The tests of `addChoices` and `makeTitle`. |

## Changes

1. Replace **＋ Add a Feed** and the button of `more` in the head of Releases with one `MenuControl` **＋ Add** (item 1). The values are Release, Artist, Label, Show and Playlist. A separator follows, then **Add One You Have…**.
2. On the page of a publisher, Release has that publisher as its start parent when it is an artist (item 2). On the top page and on a label or hosting page, Release has no start parent. Task 208 makes the form of a release show the parent. Until then, Release opens the existing form with `start.publisher_id` set when a parent exists.
3. Artist and Label open the form of a publisher page with the role Artist or Label as its start value. Show and Playlist open their forms (item 3).
4. **Add One You Have…** opens the page with the three rows of "Add a feed you have". The title of the page is "Add One You Have" (item 4). The rows keep their forms.
5. The add row **＋ Add a Release** of ADR 0164 in `editor.js` near line 4121 stays (item 5).
6. Replace **Open the Feed** with **{typeLabel} Page**, for example **Artist Page** (item 7).
7. Remove the word "feed" from each label of an add control, a menu value and an add page title (item 6). A text that explains a file can keep "feed file". Give the list of the changed labels in the report.

## Checks

Mechanical:

- Node test: the values of the add menu and their sequence.
- Node test: the menu of an artist page gives that artist as the start parent of Release. The menu of the top page gives none.
- Node test: no label of `addChoices`, of the add menu or of the head buttons has the word "Feed" or "feed", except "feed file".
- `bash tools/gate.sh` passes.

Visual:

- drive-ui screenshots at 1280 px and 390 px of the head of Releases with the open menu.
- drive-ui screenshots at 1280 px and 390 px of the head of an artist page with **Artist Page**.
- Run `$edges()` on each screenshot.

## Rules for the delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write a comment or a label.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When this task does not answer a question, stop and report.

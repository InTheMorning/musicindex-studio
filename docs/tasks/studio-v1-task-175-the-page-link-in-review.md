# Studio v1 Task 175: The Page Link In Review

Decision: [ADR 0218](../adr/0218-a-page-link-in-review-and-a-suggestion-starts-with-no-person.md) items 1 to 9. [ADR 0213](../adr/0213-a-release-starts-on-the-page-of-its-artist.md) items 17 to 21.

Status: Open.

## Problem

Review stops the write of a release with no artist page. **Link a Page** closes Review and goes to the section "Linked artist and label pages". While the refusal shows, the write button is not there (`editor.js:2142`). With no artist page in the collection, the user meets a dead end.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Review sheet | extend | crates/api/ui/editor.js | `ReviewSheet` near line 1959. The refusal panel is near line 2196. `hasWork` near line 2142 hides the write button. |
| Page refusal view | extend | crates/api/ui/feedcheck.js | `PAGE_FINDING_TEXT` and `pageRefusalView` near line 72. Add the line, the menu label and the write line of ADR 0218 item 2. |
| Page link action | reuse | crates/api/ui/editor.js | `addPublisher` near line 2579 and `changeTwoDrafts` near line 3825. The route is `POST /feeds/{page}/links` with `{ release }`. Use the same call. Do not copy the function: move the shared part to one function that the section and Review both call. |
| Page candidates | extend | crates/api/ui/editor.js | `candidates` near line 2560, in `ArtistsAndLabels`. Move the filter to one exported function that the section and Review both use. |
| Form that makes a page | reuse | crates/api/ui/editor.js | `AddFeedRowSheet` with `open="publisher"`, as `ReleaseArtistPicker` near line 2631 opens it. |
| Result panel | extend | crates/api/ui/writeresult.js | `writeResult(feed, connector)`. Add the line of ADR 0218 item 7. |
| Finding of the core | extend | crates/core/src/feed_check.rs | `check` near line 198. The reason of the page finding gets the line of ADR 0218 item 8. |

## Changes

1. UI: when the diff has `page_refusal`, the refusal panel in Review has the title and the line of ADR 0218 item 2. It also has the select menu with the placeholder "Select a page", and **Link and Continue**. The button is disabled until a selection.
2. UI: **Link and Continue** makes the link through the shared function of the section. Then Review reads the diff again. A link error shows in the panel.
3. UI: while the refusal shows, the footer has the write button disabled, with the write line of ADR 0218 item 2 above the footer.
4. UI: with no candidate page, the panel has **Make an Artist Page…** in place of the menu. It opens the form that makes an artist page, with the name from the field "Artist" (`itunes:author`) of the release. After the page is made, Studio links the release to the new page, returns to the release and opens Review.
5. UI: a link in this Review can change the draft of another page. Then the result panel after the write has "{page title} also changed." with **Open {page title}** under its first line.
6. Core: the reason of the finding "No artist page" is "This release has no artist page. Studio cannot write it before the link." The reason of "No linked page" gets the same second sentence. Change each test that reads the old text.
7. The section "Linked artist and label pages" works as before.

## Files

- Change: `crates/api/ui/editor.js`, `crates/api/ui/feedcheck.js`, `crates/api/ui/writeresult.js` and `crates/core/src/feed_check.rs`.
- Change `crates/api/ui/styles.css` only if the panel needs a rule.
- Change tests: the UI test file of `pageRefusalView`, `crates/api/ui-tests/writeresult.test.mjs` and the tests in `crates/core/src/feed_check.rs`.
- Change each API test that reads the old reason.

## Checks

Mechanical:

- UI: the refusal view gives the texts of ADR 0218 item 2 for each refusal. It also gives the state of **Link and Continue** with and without a selection.
- UI: the shared candidate function gives the same pages as before for the section.
- UI: the result panel gives the line of change 5 when a page changed, and no line when no page changed.
- Core: the reason of change 6.
- `tools/gate.sh` passes.

Visual, by the main session with screenshots (`.studio-dev/journeys/j04.json` as a base, data `base`, the release "Loose Single"):

- Review with the refusal panel before and after a selection, and after **Link and Continue**, at 1280 px and 390 px.
- The panel with **Make an Artist Page…** in a collection with no artist page.
- The result panel with the line of change 5.

## Rules for the delegate

- Write each comment and text in Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report the changed files and the check results.
- Do not make an architecture decision. When the ADR does not answer a question, stop and report.

# UI And UX Findings Of 2026-11-23

This file records the UI defects and the UI improvements that the operator finds in use of Studio. Each finding gives the related code and a recommendation. The operator has not decided a finding, and no finding is built. A finding that changes a rule of an ADR needs a new ADR before a task.

## Finding 1: a release that was never written offers a file host delete

The operator added a new release and did not write it. Then the operator tried to remove it from its page.

What the code does:

- The page shows **Delete from File Host…** when `feed.connector_id` is set (`crates/api/ui/editor.js` near line 2659). A new feed gets its file host when the user adds it. Thus the button shows before a file exists on the file host. The sheet of the button then gives steps for a file that is not there.
- The confirmation of **Remove from Collection** always says "The feed file on its file host does not change" (`crates/api/ui/confirms.js`, `removeFeed`). For a feed with no written version, this text is not applicable.
- The server gives `draft_state` "new" when a draft exists and no version was written (`crates/api/src/routes.rs`, `draft_state`). The page can use the same test, or `feed.last_version`.
- Review of the same release shows **Remove from Collection…** in the position of **Discard the Draft** ([ADR 0234](../adr/0234-a-new-release-is-written-with-its-pages-and-a-one-sided-link-is-reported.md) item 1). That behavior is correct.

Recommendation:

1. Defect: do not show **Delete from File Host…** for a feed with no written version.
2. Improvement: for a feed with no written version, change the label **Remove from Collection** to **Discard the Release…** on the page and in Review. Use the type of the feed in the label (release, artist, playlist). The action does not change, and **Undo** stays.
3. Improvement: for a feed with no written version, the confirmation says "Discard “Title”? Studio has not written this release. Nothing on the file host changes."
4. A feed with a written version keeps **Remove from Collection** and **Delete from File Host…**.

Items 2 and 3 change ADR 0234 item 1. The operator asked for a note only. A mockup and an ADR are open.

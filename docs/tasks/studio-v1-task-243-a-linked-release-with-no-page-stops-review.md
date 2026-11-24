# Studio v1 Task 243: A Linked Release With No Page Stops Review

Decision: [ADR 0251](../adr/0251-review-writes-the-two-sides-of-a-changed-link.md) items 9 to 11.

Status: Done.

## Problem

- An artist page removes a release in the section **Releases**, and the page is the only artist page of the release. Review of the page shows the locked row of the release and no Stop.
- **Write to the File Host** writes the release first (ADR 0251 item 7). The server refuses it with 409 `no_artist_page`, so the page is not written either.
- Source: observation 2 of `docs/reviews/2026-11-23-desktop-check-walk.md`. A temporary API test confirmed it.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Page refusal of a draft | reuse | crates/api/src/routes.rs | `feed_page_refusal` gives the code of a draft. Call it for each linked feed. |
| Rows of the diff route | extend | crates/api/src/routes.rs | `linked_feeds_to_write` adds the field `refusal`. |
| Refusal texts and panel model | extend | crates/api/ui/feedcheck.js | `PAGE_FINDING_TEXT`, `PAGE_REFUSAL_TEXT` and `pageRefusalView` give the panel of ADR 0218. Add a function for a linked release next to them. Do not copy `pageRefusalView`. |
| Page candidates | reuse | crates/api/ui/editor.js | `pageLinkCandidates(feeds, model)` gives the pages that a release can link. |
| Link call | reuse | crates/api/ui/editor.js | `linkPublisherPage(pageId, releaseId)` links a page and a release. |
| Review sheet | extend | crates/api/ui/editor.js | The Review sheet near `pageRefusalView(diff.page_refusal, …)` shows the panel of ADR 0218. Show the new panels in the same place and with the same markup. |

## Changes

1. Read ADR 0251 items 7 to 11 and ADR 0218 items 1 to 6 first. Then read `linked_feeds_to_write` and `feed_page_refusal` in `crates/api/src/routes.rs`, `pageRefusalView` and its text tables in `crates/api/ui/feedcheck.js`, and the Review sheet in `crates/api/ui/editor.js` (search `pageRefusal`, `alsoRows` and `linkAndContinue`).
2. API, item 9. In `linked_feeds_to_write`, add `"refusal": feed_page_refusal(w, changed.feed)?` to each entry. The value is a string or `null`.
3. UI model, items 10 and 11. In `crates/api/ui/feedcheck.js`, add `export function linkedRefusalView(row, candidates)`. `row` has `title` and `refusal`. It gives `null` when `row.refusal` is not a known code. Otherwise it gives:
    - `title`: "{title} has no other artist page." for `no_artist_page`, and "{title} has no other artist, label or hosting page." for `no_publisher_page`.
    - `line`: "A release needs an artist page." or "A release needs a page."
    - `label`: "Artist page" or "Page". `placeholder`: "Select a page".
    - `menu`: `candidates > 0`. `link`: "Link and Continue". `keep`: "Keep the Release".
    - `write`: "Link a page to {title} first."
4. UI sheet, items 10 and 11. In the Review sheet of `crates/api/ui/editor.js`:
    - For each `also_write` row with a `refusal`, show one Stop panel above the changes, with the markup of the panel of `pageRefusal`.
    - The menu lists `pageLinkCandidates(feeds, null)` without the reviewed feed. Each panel keeps its own selection.
    - **Link and Continue** calls `linkPublisherPage(selected, row.id)`. **Keep the Release** calls `linkPublisherPage(feedId, row.id)`. After each call, load the draft, the feed list and the diff again, as `linkAndContinue` does.
    - While a panel shows, the write button is off, as for `pageRefusal`, and the hint above the footer is the `write` text.
5. Tests:
    - Rust, API (`crates/api/src/tests.rs`): extend `a_changed_link_of_two_written_feeds_gives_a_row_on_each_side` so that each expected entry has `"refusal": null`. Update each other test that compares a full `also_write` entry.
    - Rust, API: add a test. Write an artist page and a release that links only that page. Unlink them with `DELETE /api/v1/feeds/{page}/links/{release}`. The diff of the page gives the release with `refusal` `no_artist_page`. Then `POST /api/v1/feeds/{page}/links` with the release, and the entry has no `refusal` or is gone.
    - Node (`crates/api/ui-tests/`): `linkedRefusalView` for the two codes, for `null`, and with zero candidates (`menu` is `false`).
    - Each failure message names ADR 0251 and its item, and the function to fix.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0251 and its item.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of each crate.
- The page rule stays in the core and the API. The UI shows the code that the API gives.
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- Run `tools/gate.sh` at the end and report its result.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.

## Acceptance Criteria

Mechanical:

- The API test of step 5 gives `refusal` `no_artist_page` after the unlink, and no refusal after the link back.
- Each `also_write` entry has the field `refusal`.
- The Node tests of `linkedRefusalView` pass.
- `tools/gate.sh` passes.

Visual, by the main session with screenshots at 1100 px and 390 px:

- Review of an artist page after the removal of its only release shows the Stop panel with the menu, **Link and Continue** and **Keep the Release**, and the write is off.
- **Keep the Release** removes the panel and the row.

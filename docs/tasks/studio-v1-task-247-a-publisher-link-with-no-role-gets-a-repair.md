# Studio v1 Task 247: A Publisher Link With No Role Gets A Repair

Decision: [ADR 0252](../adr/0252-a-publisher-link-with-no-role-gets-a-repair.md) items 1 to 6.

Status: Done.

## Problem

- `page_refusal` in `crates/core/src/feed_check.rs` counts only a publisher link with a `rel` that has the role `artist`. A release from another tool can have its link with no `rel`. **Issues** then gives the Stop "No artist page", and the only repair is **Link a Page**.
- On a read-only feed, `check_json` in `crates/api/src/feed_check.rs` removes the repairs, but the finding stays a Stop with the text "Studio cannot write it before the link."

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Page finding | extend | crates/core/src/feed_check.rs | The `page_refusal` block of `check` gives the line, the repair and the second repair. |
| Repair kinds | extend | crates/core/src/feed_check.rs | `RepairKind` gets `MakeItTheArtistLink`. |
| Repair plan | extend | crates/core/src/pipeline/check_repair.rs | `repair_step` and the plan match get the new kind, as `AddHttps` does. |
| Link targets | reuse | crates/core/src/feed_check.rs | `Facts::link_targets` gives the title of a linked feed of the collection. |
| Read-only check | extend | crates/api/src/feed_check.rs | `check_json` changes the level and the reason of the two page findings. |
| Repair route | reuse | crates/api/src/feed_check.rs | `repair` with `SentRepair` and `repair` = `make_it_the_artist_link`. |
| Diff route | extend | crates/api/src/routes.rs | `get_diff` gives the title of the link with no role adjacent to `page_refusal`. |
| Repair buttons | extend | crates/api/ui/feedcheck.js | `REPAIRS` and `ACTIONS` get the new kind. `pageRefusalView` gets the line and the button. |
| Review sheet | extend | crates/api/ui/editor.js | The refusal panel of `ReviewSheet` shows the button and calls the repair route. |

## Changes

1. Read ADR 0252, ADR 0218 items 1 and 8, and ADR 0256 items 17 to 19. Then read `page_refusal`, `RepairKind` and the `page_refusal` block of `check` in `crates/core/src/feed_check.rs`, `repair_step` and the plan match in `crates/core/src/pipeline/check_repair.rs`, `check_json`, `is_read_only` and `repair` in `crates/api/src/feed_check.rs`, `feed_page_refusal` and `get_diff` in `crates/api/src/routes.rs`, `REPAIRS`, `ACTIONS` and `pageRefusalView` in `crates/api/ui/feedcheck.js`, and the refusal panel in `ReviewSheet` in `crates/api/ui/editor.js`.
2. Core, item 1: add `pub fn link_with_no_role(feed: &Feed) -> Option<&RemoteItem>` (use the real type of the element). It gives the first `podcast:remoteItem` in `podcast:publisher` with `medium="publisher"` and no `rel`. Do not change `page_refusal`.
3. Core, items 2 and 3: when the finding is `NoArtistPage` and `link_with_no_role` gives a link, the reason is "The link to {title} has no role. Studio cannot write it before the link.", `repair` is `MakeItTheArtistLink`, `second_repair` is `LinkAPage`, `url` is the `feedUrl` of the link and `target_title` is `{title}`. `{title}` is the title of the `link_targets` entry with the same `feedGuid` or `feedUrl`, else the `title` attribute of the link, else its `feedUrl`. With no such link, the finding stays as now.
4. Core, item 4: add `RepairKind::MakeItTheArtistLink` (snake case `make_it_the_artist_link`). In `check_repair.rs`, `repair_step` gives `Step::Nothing`. The plan adds `rel="artist"` to the first link with no role and changes nothing else. It refuses with "this repair does not apply to the draft" when the draft has no link with no role.
5. API, item 6: in `check_json`, when the feed is read-only, a finding of kind `no_artist_page` or `no_publisher_page` gets the level `notice` and a reason with no sentence "Studio cannot write it before the link." The other findings stay as now.
6. API, item 5: `get_diff` gives `"page_refusal_link": "{title}"` when `page_refusal` is `no_artist_page` and the draft has a link with no role, else `null`. Use the title rule of item 3.
7. UI, items 2 and 5:
   - `REPAIRS.make_it_the_artist_link = 'Make It the Artist Link'` and `ACTIONS.make_it_the_artist_link = 'repair'`.
   - `pageRefusalView(reason, selected, noRoleTitle = null)`: when `noRoleTitle` is set, it gives `noRole: { line: 'The link to {noRoleTitle} has no role.', button: 'Make It the Artist Link' }`, else `noRole: null`.
   - In the refusal panel of `ReviewSheet`, when `view.noRole` is set, show the line under the first line and the button above the select menu. The button sends the repair route with `kind` `no_artist_page`, `repair` `make_it_the_artist_link` and the draft revision. Then load the diff again, as the link from the menu does.
8. Tests:
   - Core, in `crates/core/tests/feed_check_repair.rs` or adjacent to the page tests in `feed_check.rs`: a release with one link with no role gives the finding of item 3, with the two repairs. The plan adds `rel="artist"` to that link only. After the repair, `page_refusal` gives `None`. A second publisher link with a `rel` does not change.
   - Core: the title of item 3 for a `link_targets` entry, for a `title` attribute, and for a `feedUrl` only.
   - API, in `crates/api/src/tests.rs`: the check of a read-only release with no artist page gives the level `notice`, no repair, and no sentence "Studio cannot write it before the link.". The repair route with `make_it_the_artist_link` on a written release removes `page_refusal` from the diff.
   - API: `get_diff` gives `page_refusal_link` with the title.
   - Node, `crates/api/ui-tests/feedcheck.test.mjs`: `pageRefusalView` with a title gives `noRole` with the line and the button. Without it, `noRole` is `null`.
   - Each failure message names ADR 0252 and its item, and the function to fix.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0252 and its item.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of each crate.
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- Run `cargo test` and `tools/gate.sh` at the end, and report the results.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.

## Acceptance Criteria

Mechanical:

- The tests of step 8 pass.
- `tools/gate.sh` passes.

Visual, by the main session with screenshots at 1100 px and 390 px:

- **Issues** of a release with a link with no role shows the line and the buttons **Make It the Artist Link** and **Link a Page**.
- The refusal panel of Review shows the line and **Make It the Artist Link**. After a click, the panel goes and the write button works. A real write in the harness writes `rel="artist"`.

Human check: on the desktop, **Issues** of the Elijah Lied release shows a Notice, with no button.

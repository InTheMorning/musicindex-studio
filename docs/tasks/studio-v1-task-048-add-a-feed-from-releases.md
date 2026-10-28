# Studio v1 Task 048: Add A Feed From Releases

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The Editor Files And Covers".
Decisions: [ADR 0072](../adr/0072-add-a-feed-opens-from-releases.md), [ADR 0039](../adr/0039-the-shell-releases-and-the-outline.md), [ADR 0037](../adr/0037-button-roles.md) and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).

Status: Complete - 2026-10-28. The four commit checks pass. The screenshot visual check passed, and the [review checklist](../reviews/2026-10-28-task-048-review-checklist.md) records it. The operator examines it on a phone after the next release candidate.

## Goal

The left panel and the root page hold only Releases at the top. The Releases page shows "＋ Add a feed" in its head. At narrow width, Back from Add feeds opens Releases.

This task is one packet. It changes one model function set, one component and the guide pages.

## Files

Inspect: `crates/api/ui/shell.js` (`leftPanel`, `narrowStack`), `crates/api/ui-tests/shell.test.mjs`, `crates/api/ui/app.js` (`destinationRows`, `ReleasesPage`, the `add-feed` view in `Shell`), `crates/api/ui/styles.css` (`.releases-head`, `.releases-tools`), the guide pages below.

Change: `crates/api/ui/shell.js`, `crates/api/ui-tests/shell.test.mjs`, `crates/api/ui/app.js`, `docs/user-guide/add-feeds.md`, `docs/user-guide/playlists.md`, `docs/user-guide/artist-and-label-feeds.md`, `docs/user-guide/new-release.md`, `docs/user-guide/live-shows.md`, `docs/user-guide/find-your-way.md`, `docs/user-guide/use-on-a-phone.md`.

Do not change: `AddFeeds` and its forms, `pages.js`, the Rust crates, the ADRs. Change `styles.css` only when the visual check needs it, and then say so.

## Architecture

These choices are fixed. Do not change them.

- `leftPanel` gives `top` as `[{ id: 'releases', label: 'Releases', current }]`. `current` is `true` on the views `releases` and `add-feed`. `bottom` does not change.
- `narrowStack({ name: 'add-feed' })` gives `[root, releases, { name: 'add-feed' }]`.
- `ReleasesPage` shows `<button class="button action" type="button">＋ Add a feed</button>` in `.releases-tools`, after the view buttons, only when `hasRows` is `true`. It calls `onAddFeed`. The empty page keeps its primary button "Add a Feed".
- The comment of `destinationRows` names Releases only.
- The guide pages tell the user to select **＋ Add a feed** on the Releases page, not **Add a feed** in the left panel. `find-your-way.md` and `use-on-a-phone.md` remove **Add a feed** from the lists of the left panel and of the first page.

## Steps

1. Change `shell.js` and its tests:
   - The test "with no feed open, the left panel gives the four destinations" expects `top` as `['releases']`. Change its title so that it does not give a count.
   - Add a test: on the view `{ name: 'add-feed' }`, the row `releases` is current.
   - The narrow stack test expects `[root, releases, { name: 'add-feed' }]`.
   - The message of each new assertion names ADR 0072.
2. Change `app.js`.
3. Change the guide pages.
4. Run the four commit checks, and `ste_lint.py --check --no-heuristics` on each changed guide page.
5. The planning model does the visual check. Do not run it.

## Constraints

- No feed rule in the UI (ADR 0065). This packet adds no function that changes the model.
- Each comment, each test message, each UI text and each guide page is in ASD-STE100 Simplified Technical English.
- The new button has the role `action` (ADR 0037).

## Acceptance Criteria

Mechanical:

- `leftPanel` with no feed open gives `top` as Releases only.
- `leftPanel` gives Releases as current on the view `add-feed`.
- `narrowStack({ name: 'add-feed' })` gives the root page, Releases, then Add feeds.
- `grep -n "Add a feed\*\* in the left panel" docs/user-guide/*.md` finds nothing.
- The four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`. In the dark theme, in the light theme and at 390 px:

- The left panel and the root page show only Releases at the top.
- Releases with a feed shows "＋ Add a feed" in its head, and the button opens Add feeds.
- At 390 px, Back from Add feeds opens Releases.
- The empty Releases page shows only its primary button "Add a Feed".
- `$edges()` finds no content at the edge of a group.

## Test Commands

```sh
tools/node --test crates/api/ui-tests/*.test.mjs
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 ~/.agents/skills/asd-ste100/scripts/ste_lint.py --check --no-heuristics docs/user-guide/add-feeds.md docs/user-guide/playlists.md docs/user-guide/artist-and-label-feeds.md docs/user-guide/new-release.md docs/user-guide/live-shows.md docs/user-guide/find-your-way.md docs/user-guide/use-on-a-phone.md
```

## Escalation

- Stop and report when a part of the UI other than `destinationRows` reads the row `add-feed` of `leftPanel`.
- Stop and report when the change of `narrowStack` changes the stack of a different view.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0072-add-a-feed-opens-from-releases.md
- docs/tasks/studio-v1-task-048-add-a-feed-from-releases.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- Releases is the only row at the top of the left panel and the root page. The Releases page has the button "＋ Add a feed". At narrow width, Add feeds opens over Releases. The guide pages name the new button.

Constraints:
- Follow the Architecture section of the task file fully.
- No feed rule in the UI (ADR 0065).

Do not touch:
- AddFeeds and its forms, pages.js, the Rust crates, the ADRs.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- The commands of the Test Commands section.

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

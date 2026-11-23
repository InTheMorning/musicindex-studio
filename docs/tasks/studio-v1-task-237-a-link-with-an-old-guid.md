# Studio v1 Task 237: A Link With An Old GUID

Decision: [ADR 0247](../adr/0247-studio-restores-a-changed-feed-guid-and-corrects-a-link-with-an-old-guid.md) items 7 to 9. [ADR 0248](../adr/0248-music-index-changes-a-feed-guid-in-place-and-studio-restores-no-guid.md) item 3 keeps them current. Finding 13 of [the tutorial findings](../reviews/2026-11-22-tj-tutorial-findings.md).

Status: Done.

- ChatGPT built the task. The main session read the diff and ran `tools/gate.sh`.
- The main session added the Review label "Feed GUID" for `feed_guid` in `crates/api/ui/review.js`, with a Node test. Before, Review showed the model name `feed_guid`.
- Screenshots of `data-base` (`.studio-dev/v-230/o237a`): the tab **Issues** of Elijah Lied shows the Warning and **Use the GUID of This Is It**. After the button, Review shows one change "Feed GUID: 11f41bfe… → 24e9311b…".
- Screenshots of the move (`.studio-dev/v-230/o237b`): **Move Artists Here…** with Elijah Lied corrects the link in the drafts of Elijah Lied and "all in a day". No draft holds the old GUID after the move.
- In `data-base`, Elijah Lied has the role artist already. Thus `gather_pages` does not call `set_roles`, and no link of that page gets a `rel`. This applies to all six links and is the behavior before this task. The API test of step 8 proves the `rel` for a page with no roles.

## Problem

- The artist feed of Elijah Lied links "This Is It" with the GUID `11f41bfe-7cbe-5795-85c3-6deaccac20aa`. The file `thisisit.xml` has the GUID `24e9311b-73a9-5f48-b593-81148712cb14`. The `podcast:podroll` of "all in a day" has the same old GUID. The hosted files in `.studio-dev/v-230/host` and `.studio-dev/v-230/library` show this.
- `names` in `crates/core/src/publisher.rs` matches a link by its `feedGuid` when it has one. A link with an unknown `feedGuid` and a correct `feedUrl` names no feed.
- Thus `catalogs_with_kept` does not list "This Is It" under Elijah Lied, and "Move Artists Here…" (`gather_pages` in `crates/api/src/routes.rs`) gives that link no `rel`.
- The check gives no finding for such a link.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Link match | extend | crates/core/src/publisher.rs | `names` and `Party` decide which feed a link names (ADR 0205 item 12). A new public function finds and corrects the links of item 7 with the same `Party` data. |
| Feed identity | reuse | crates/core/src/pipeline.rs | `feed_identity` gives the current GUID and title of a feed, from the draft or the row. |
| URL spelling rule | reuse | crates/core/src/feed_check.rs | `same_but_spelling` gives the rule of ADR 0174 item 12. It gives false for equal text, so the caller also tests equal text. |
| Check facts | extend | crates/core/src/feed_check.rs | `Facts` gets the feeds of the collection that a link can name. |
| Check inputs | extend | crates/core/src/pipeline/check_repair.rs | `check_inputs` reads the feeds of the collection with no request. |
| Facts builder | extend | crates/core/src/feed_reads.rs | `facts_for` and `check_with` give the new facts to the check. |
| Check repair | extend | crates/core/src/pipeline/check_repair.rs | `repair_step`, `plan_of` and `Plan` get the new repair. |
| Finding rows | extend | crates/api/ui/feedcheck.js | `checkRows`, `REPAIRS` and `ACTIONS` give the button. |
| Gather route | extend | crates/api/src/routes.rs | `gather_pages` corrects the links before it reads the catalogs and before `set_roles`. |

## Changes

1. Read ADR 0247 items 7 to 9, ADR 0248, ADR 0174 items 7 and 12, ADR 0205 item 12 and ADR 0243 first. Then read `names`, `Party`, `party`, `catalogs_with_kept` and `set_roles` in `crates/core/src/publisher.rs`, `feed_identity` in `crates/core/src/pipeline.rs`, `Facts`, `check`, `check_links` and `same_but_spelling` in `crates/core/src/feed_check.rs`, `facts_for` and `check_with` in `crates/core/src/feed_reads.rs`, all of `crates/core/src/pipeline/check_repair.rs`, `crates/api/src/feed_check.rs`, `gather_pages` in `crates/api/src/routes.rs`, and `checkRows` in `crates/api/ui/feedcheck.js`.
2. Core, the link target. Add a public type `LinkTarget { guid: String, url: String, title: Option<String> }` in `crates/core/src/feed_check.rs` (or a module that both `feed_check` and `publisher` can use). A target is a feed of the collection with a `podcast:guid` and a public address. Add one core function that gives the targets of a collection: it reads `Collection::feeds()` and uses `feed_identity` for each row. It sends no request. Use one code path for the targets in the check, the repair and the move.
3. Core, item 7. Add one pure function that gives the old links of a feed: `old_links(feed: &Feed, targets: &[LinkTarget]) -> Vec<OldLink>`. It examines each `podcast:remoteItem` of `remote_items`, of `podroll` and of `publisher`. A link is old when each condition is true:
    - Its `feedGuid` is present and equal to the `guid` of no target.
    - Its `feedUrl` is equal to the `url` of a target, or `same_but_spelling` gives true for the two.
    - That target has a different `guid`. (This follows from the first condition. Keep the test explicit.)

   `OldLink` gives the old GUID and the target. Give one result for each target, in the sequence of the first link that names it. Add one function `correct_old_links(feed: &mut Feed, targets: &[LinkTarget], target_guid: Option<&str>) -> usize`. It sets the `feedGuid` of each old link to the `guid` of its target. With `Some(guid)` it changes only the links of that target. It does not change `feedUrl`, `title`, `medium` or `rel`. It gives the count of changed links.
4. Core, item 8, the finding. Add `FindingKind::LinkGuidOld` (serialized `link_guid_old`) and `RepairKind::UseTheGuidOf` (serialized `use_the_guid_of`). Add `link_targets: Vec<LinkTarget>` to `Facts`. `check` gives one finding for each result of `old_links`:
    - level: Warning.
    - title: "A link to {title} has an old GUID". Use the target `title`, else its `url`.
    - reason: "The link names the GUID {old}. The feed {title} has the GUID {guid}."
    - `url`: the `url` of the target. The repair uses this field to find the target.
    - repair: `UseTheGuidOf`. A read-only feed gets no repair, as the other findings.

   Add `link_targets` to `CheckInputs`, filled by `check_inputs` with the function of step 2. `facts_for` and `check_with` take the targets and put them in `Facts`. Change each caller in `crates/api/src/feed_check.rs` and each test that builds `Facts`.
5. Core, item 8, the repair. `repair_step` gives `Step::Nothing` for `UseTheGuidOf`. `plan_of` finds the target whose `url` is the `url` of the finding, and gives `Plan::LinkGuid { url }`. If no link of the draft is old for that target, it gives the error "this repair does not apply to the draft". `Plan::apply` calls `correct_old_links` with that target. The repair changes only the draft of this feed. The repair needs the targets: give them to `plan_of` from `apply_check_repair` with the function of step 2.
6. API, item 9, the move. In `gather_pages`:
    - After `files_feed` gives each model of `catalog_feeds`, correct the old links of each model in memory with `correct_old_links(model, &targets, None)`. Then `catalogs_with_kept` finds the release, and `touched` contains it.
    - After the snapshot and the `ensure_draft` loop, and before the first `set_roles`, correct the old links of the draft of each feed of `touched`, and save each changed draft. Use `pipeline::changed_draft` or the existing draft change function that the publisher functions use, so that the change shows in Review and Discard restores it.
    - Do not change a draft outside `touched`.
7. UI, item 8. In `crates/api/ui/feedcheck.js`, add `use_the_guid_of` to `ACTIONS` as `repair`. The button label is "Use the GUID of {title}". The core gives the title: add the field `target_title` to the finding, serialized only when present, and the UI builds the label from it. The UI does not read a GUID. The repair body is the body of the other repairs: `kind`, `entry`, `url`.
8. Tests:
    - Rust, `crates/core/src/feed_check.rs` tests: a channel link with an unknown `feedGuid` and the `feedUrl` of a target gives one `link_guid_old` Warning with the title and the reason of step 4. A link with a known `feedGuid` gives no finding. A link with no `feedGuid` gives no finding. A `feedUrl` that differs only in spelling (`HTTPS://Host:443/a.xml`) gives the finding. A podroll link and a `podcast:publisher` link give the finding. Two links to the same target give one finding.
    - Rust, `crates/core/tests/feed_check_repair.rs`: the repair sets the `feedGuid` of each old link to the target, in the channel and in the podroll, and changes no other attribute and no other draft. A second repair gives the error of step 5.
    - Rust, API tests near the gather tests in `crates/api/src/tests.rs` (`move_pages`, `move_draft_rows`): an artist page with a channel link of item 7 to a release, gathered under a hosting page with role artist. After the gather, the link has the GUID of the release and `rel="artist"`, and the release is linked to the hosting page. Discard of the change set restores the old GUID.
    - Node, `crates/api/ui-tests/feedcheck.test.mjs`: a `link_guid_old` finding with `target_title` "This Is It" gives the button "Use the GUID of This Is It", the action `repair` and the body with `kind`, `entry` and `url`.
    - Each failure message names ADR 0247 and its item, and the function to fix.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0247 and its item.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of each crate.
- Feed rules stay in the core (ADR 0065). The UI does not compare GUIDs or addresses.
- Do not restore or change the `podcast:guid` of a feed (ADR 0248 item 2). Only the `feedGuid` of a link changes.
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.

## Acceptance Criteria

Mechanical:

- The tests of step 8 pass.
- `tools/gate.sh` passes.

Visual (the main session checks with screenshots of the tutorial data, `.studio-dev/v-230/run.sh` with a copy of `data-base`):

- The tab **Issues** of Elijah Lied shows the Warning "A link to This Is It has an old GUID" with the button **Use the GUID of This Is It**.
- After the button, the row is gone, and Review shows one change of `feedGuid`.
- After **Move Artists Here…** on the hosting page with Elijah Lied, the review shows the link to "This Is It" with the relation artist.

## Report

Give the changed files, the new tests, the result of `tools/gate.sh` and `git status --short`.

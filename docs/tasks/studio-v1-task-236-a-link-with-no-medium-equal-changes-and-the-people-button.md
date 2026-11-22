# Studio v1 Task 236: A Link With No Medium, Equal Changes And The People Button

Decision: [ADR 0246](../adr/0246-a-link-with-no-medium-equal-changes-in-review-and-the-people-button.md). Finding 9 and finding 12 of [the tutorial findings](../reviews/2026-11-22-tj-tutorial-findings.md), and an open point of task 235.

Status: Done.

- Parts 1 and 2: the main session corrected three small faults in `review.js`. They are the summary of an empty value, the label of a group with example rows, and the fallback name.
- Part 3 corrects the six faults of the independent review. The main session put back the names in the collapsed line of ADR 0114 item 2.
- Screenshots of the move review (`data-base`, no artist roles) show 38 feeds. The Doerfels show two summary lines and one fold of 21 release titles. Elijah Lied shows one fold of 5 entries.
- The People button label is checked by its Node test only.

## Problem

- The links of `elijahliedpubfeed.xml` have no `medium`. `is_release(item.medium)` in `crates/core/src/publisher.rs` refuses them. Thus `set_roles` writes no `rel` in that file, and `linked_releases` and `publishers_linking` do not find its releases.
- A review group with 18 equal changes shows two equal example rows and a collapsed line of 16 more.
- The People button of a review builds a possessive from the name: "Use The Doerfels's Values in 8 Feeds".

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Release link test | extend | crates/core/src/publisher.rs | `is_release` and `names` decide a release link. A new private function adds the rule of ADR 0246 item 1, and the three callers use it. |
| Role setter | extend | crates/core/src/publisher.rs | `set_roles` sets the `rel`. It also sets `medium` for a link of item 1 (item 3). |
| Review field rows | extend | crates/api/ui/review.js | `cluster` and `fieldRows` make the summary and the `more` line (ADR 0114 item 2). |
| Review field view | extend | crates/api/ui/editor.js | The view near line 1562 draws `g.more` in a `details` element. |
| People panel model | extend | crates/api/ui/feedcheck.js | `peoplePanelView` gives the button label. |

## Changes

1. Read ADR 0246, ADR 0205 items 6 and 12, ADR 0114 item 2 and ADR 0245 item 11 first. Then read `is_release`, `names`, `party`, `linked_releases`, `publishers_linking` and `set_roles` in `crates/core/src/publisher.rs`, `cluster` and `fieldRows` in `crates/api/ui/review.js`, the field view in `crates/api/ui/editor.js` near line 1562, and `peoplePanelView` in `crates/api/ui/feedcheck.js`.
2. Core, item 1 and item 2: add one private function in `publisher.rs` that gives the medium of a release link, or `None`. When the link has a release `medium`, it gives that medium. When the link has no `medium`, it gives the medium of the release feed of the collection that the link names (the match of `names`), else `None`. A link with a `medium` that is not a release medium is never a release link. `linked_releases`, `publishers_linking` and `set_roles` use this function in place of `is_release(item.medium...)`. Do not read the collection again for each link if a list of the release rows can be read one time.
3. Core, item 3: in `set_roles`, a link of item 1 with no `medium` gets `medium` with the medium of the named release, together with the `rel`.
4. UI, item 4 and item 5: in `fieldRows`, when each change of the set from `cluster` also has the same old value, the group has `summary`, `rows` with only the changes that the set does not cover, and `more` = `{ summary: 'Show the {n} entries', rows: [], names: [...] }`. Use "credits" in place of "entries" for a credit group. `names` has the `entry` of each change of the set, in sequence. A change with no `entry` gives "Entry {k}", where k is its position in the set from 1. A set with more than one old value does not change.
5. UI, editor: when `g.more.names` is not empty, the `details` element lists the names, one on each line, in place of the rows. The fold stays closed when the review opens.
6. UI, item 6: `peoplePanelView` gives the button **Use the People Values in {n} Feeds**, or **Use the People Values in 1 Feed**. Correct the tests that use the old label.
7. Tests:
   - Rust unit tests in `publisher.rs` (or the existing publisher test file): a publisher draft with a link with no `medium` that names a `music` feed of the collection. `linked_releases` gives that feed. After `set_roles` with the role artist, the link has `rel="artist"` and `medium="music"`. A second link with no `medium` that names a feed that is not in the collection keeps no `rel` and no `medium`. `publishers_linking` gives the publisher feed for that release.
   - Node in `crates/api/ui-tests/review.test.mjs`: 18 changes of `rel` from none to `artist` give the summary "18 entries get the relation artist", no example row, and `more.summary` "Show the 18 entries" with 18 names. Three changes with one new value and two old values give two example rows and the collapsed line, as before. Correct an existing test only when its changes have equal old values, and say so in the report.
   - Node in `crates/api/ui-tests/feedcheck.test.mjs`: the person "The Doerfels" with one feed gives **Use the People Values in 1 Feed**. Two feeds give **Use the People Values in 2 Feeds**.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0246 and its item. A test failure message names ADR 0246 and its item, and the function to fix.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of each crate.
- Feed rules stay in the core (ADR 0065). The UI does not read `medium`.
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.
- Do not use `/etc/aiwrap-gpt`.

## Acceptance Criteria

Mechanical:

- The tests of step 7 pass.
- `tools/gate.sh` passes.

Visual (the main session checks with screenshots of `.studio-dev/v-230/data-tut`):

- After **Move 3 Artists**, the review contains the artist feed of Elijah Lied with the relation artist on its links.
- The review of The Doerfels shows the summary line and the closed fold "Show the 18 entries". The open fold lists the release titles.
- The People panel shows **Use the People Values in 8 Feeds**.

## Report

Give the changed files, the new tests, the result of `tools/gate.sh` and `git status --short`.

## Part 2: Link Names And One Fold For Each Group

The screenshot check of part 1 found two faults. The fold of The Doerfels lists "Entry 1" to "Entry 21", because a link change has no `entry`. Each link has a change of `medium` and a change of `rel`, and only the `rel` set goes into the fold. Thus "Medium: none → music" shows again for each link. ADR 0246 items 4 and 5 now give the correction.

Part 1 is in the working tree. Do not change `crates/core/src/publisher.rs` or `crates/api/ui/feedcheck.js`.

Changes:

1. Core, item 5: in `crates/core/src/review.rs`, `entry_label` gives the `title` of a `remote_items` entry, else its `feed_url`. Then in `crates/core/src/pipeline.rs` (`diff_planned`), each `remote_items` change whose link has no `title` gets the title of the collection feed that the link names, by the match of ADR 0205 item 12 (the `feedGuid` decides, else the `feedUrl`). The core review has no collection, so the pipeline gives the names. Keep the work to one read of the collection feeds for each diff. If the field change does not carry the `feedGuid`, add a field that the serializer skips (`#[serde(skip)]`), as `FieldChange` does already for other data.
2. UI, item 4: in `fieldRows` in `crates/api/ui/review.js`, find each set of more than two changes of one path with one new value and one old value, not only the largest set. Each set gives one summary line. The group then has `summaries` (a list of the summary texts), `summary` (the first one, so that old callers still work), the rows of the changes that no set covers, and one `more` with `{ summary: 'Show the {n} entries', rows: [], names }`. `names` lists each entry once, in the sequence of the first change of each entry. n is the count of `names`. A group with no equal set and a group with a set of more than one old value do not change (item 6).
3. UI, editor: the field view near line 1562 of `crates/api/ui/editor.js` shows each line of `summaries`.
4. Tests:
   - Rust in `crates/core/src/review.rs` tests or `crates/core/tests/`: a change of the `rel` of a link with `title="Album"` gives the entry "Album". A link with no `title` gives its `feedUrl`.
   - Rust in `crates/core/tests/` for the pipeline diff: a publisher draft with a link with no `title` that names a collection feed with the title "Album" gives the entry "Album".
   - Node in `crates/api/ui-tests/review.test.mjs`: 6 changes of `medium` from none to `music` and 6 changes of `rel` from none to `artist` on the same 6 entries give two summaries, no row, and one fold "Show the 6 entries" with 6 names.
   - Each test failure message names ADR 0246 and its item.

Checks: run the tests and `tools/gate.sh`. Correct the faults. Do not commit or stage.

## Part 3: Corrections From The Independent Review

An independent review of parts 1 and 2 found six faults. ADR 0246 items 2, 4, 5 and 6 now give the corrections. Parts 1 and 2 are in the working tree and staged. Do not stage, unstage or commit.

Changes:

1. Core, item 2: `roles` in `crates/core/src/publisher.rs` calls `inferred_roles(&feed)`, which reads only links with a release `medium`. Extend `inferred_roles` in `crates/core/src/collection.rs`: add `inferred_roles_with(feed, is_release_link)`, where `is_release_link` is a function of a `&RemoteItem`. `inferred_roles(feed)` calls it with the medium test, so that discovery and the store do not change. `roles` calls it with `release_medium(item, &releases).is_some()`, with one call of `release_parties`.
2. Core, item 5: move the body of `party` in `publisher.rs` that reads the GUID and the title of a feed into one function in `crates/core/src/pipeline.rs`: `pub(crate) fn feed_identity(collection: &Collection, row: &FeedRow) -> Result<(Option<String>, Option<String>), PipelineError>`. It gives the `podcast:guid` and the title of the draft when the feed has a draft, else of the row. The title falls back to `row.title`. `party` uses it. `name_review_links` uses it to make its two maps, and returns at once when no change needs a name.
3. Core, item 5: in `match_changes` in `crates/core/src/review.rs`, `remote_item_has_title` is true only when the entry name comes from a title: the new link has a `title`, or the new link gives no label and the old link has a `title`. Now a link that loses its `title` gets the title of the named collection feed.
4. Core, item 4: add `pub entry_index: Option<usize>` to `FieldChange`, serialized with `#[serde(skip_serializing_if = "Option::is_none")]`. Add `index: Option<usize>` to `Who`. In `match_changes`, `Match::Pair(i, j)` and `Match::Added(j)` give `Some(j)`, and `Match::Removed(i)` gives `Some(i)`. Each change under a matched entry carries the index of `Who`. The `hidden` change has `None`.
5. UI, items 4 and 6, in `fieldRows` in `crates/api/ui/review.js`:
   - The equal sets (each change of the set has the same old value) give one summary each, as now.
   - The largest set with more than one old value also gives its summary, after the equal summaries. Its first two changes give example rows, and the rest give `more = { summary, rows }` with the text of ADR 0114 item 2, as the branch with no equal set does now.
   - The equal sets give `fold = { summary: 'Show the {n} entries', names }` ("credits" for a credit group). `names` has one name for each entry, by `entry_index` (fall back to the `entry` text, then to the change index, when `entry_index` is missing). The sequence is the sequence of the first change of each entry. A name is the `entry` of the change, else "Entry {k}", where k is the position in `names` from 1. n is the count of `names`.
   - A change of an equal set whose `rowsFor` rows have a `warning` also shows in `rows`.
   - `rows` is: the example rows of the other set, then each change that no set covers, in sequence, with the warning rows of item 4 in their sequence.
   - The group gives `{ label, summaries, summary, rows, more, fold }`. `summary` is the first summary, or null. A group with no set keeps the heading rule of the current code. Keep one code path for the summary text; do not copy it.
6. UI, editor: in the field view near line 1578 of `crates/api/ui/editor.js`, draw `g.more` with its rows and `g.fold` with its names as two closed `details` elements. Remove the `g.more.names` branch.
7. Tests:
   - Rust, `crates/core/tests/publisher.rs`: a publisher draft with a link with no `medium` and `rel="artist"` that names a `music` feed of the collection. The publisher row has no confirmed roles. `roles` gives `[Artist]`.
   - Rust, `crates/core/tests/pipeline.rs`: a link with no `title` names a collection feed whose draft has a new `podcast:guid` and the title "New Album". The entry is "New Album". A link whose `title` is removed in the draft gets the title of the named collection feed.
   - Rust, `crates/core/src/review.rs` tests: two link changes of one list give `entry_index` 0 and 1 in the JSON of the review.
   - Node, `crates/api/ui-tests/review.test.mjs`: a group with 6 equal `rel` changes and 3 `medium` changes with two old values gives two summaries, two example rows, `more` with one row and `fold` with 6 names. Two entries with the same `entry` and different `entry_index` give a fold of 2. Three entries with no `entry`, each with a `medium` and a `rel` change, give "Show the 3 entries". Three equal credit `npub` changes where one profile name does not agree give one row with a warning. Move the existing tests that read `more.names` to `fold`.
   - Each failure message names ADR 0246 and its item, and the function to fix.

Constraints: the constraints of part 1 apply, but you may use the shell and `tools/node`. Read the STE skill at `/home/citizen/.agents/skills/asd-ste100/SKILL.md` for comments. Do not change a file in `docs/`.

Checks: `cargo test -p musicindex-core` (or the crate name in `crates/core/Cargo.toml`), `tools/node --test crates/api/ui-tests/review.test.mjs crates/api/ui-tests/feedcheck.test.mjs`, `tools/node --check` on `.mjs` copies of the changed UI files, and `tools/gate.sh`. Report the changed files, the tests and the gate result.

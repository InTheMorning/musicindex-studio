# Studio v1 Task 245: A Link Back Keeps The Written Element

Source: observations 4 and 5 of `docs/reviews/2026-11-23-desktop-check-walk.md`.

Status: Done.

Operator choice (defect correction, no ADR): a link that the user removes and adds again gives the written element back, at its written position. A draft that is the same as its base file, apart from the order of the bytes, has the bytes of the base file. A link that already exists keeps its attributes when they do not change.

## Problem

- `publisher::unlink` removes the entry. `publisher::link` then calls `upsert`, which adds a new `Entry::new` last (`crates/core/src/publisher.rs`, `upsert`). The writer makes a new element. Review then shows a removed line and an added line for the same link, and the release moves to the end of the list of the page.
- `upsert` sets `title` each time. A move calls `publisher::link` for each selected page, also for a link that does not change (`move_release_to_pages` in `crates/api/src/routes.rs`). A written element with no `title` then gets `title` in the draft.
- `xml::attribute_orders` takes the attribute order of the first source element of a name only. When that element has no `title`, a new element gets `title` after `rel`, but the other elements of the file have `title` before `rel`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Link and unlink | extend | crates/core/src/publisher.rs | `upsert` and `link` restore the base entry. |
| Draft edit | extend | crates/core/src/pipeline.rs | `edit_draft` gives the base bytes when the draft model is the same as the base model. |
| Attribute order | extend | crates/core/src/xml.rs | `attribute_orders` merges the order of each source element of a name. |
| Base file of a draft | reuse | crates/core/src/collection.rs | `DraftRow::base_xml` is the file when the draft started. |

## Changes

1. Read `link`, `unlink`, `upsert` and `names` in `crates/core/src/publisher.rs`, `edit_draft`, `changed_draft` and `same_apart_from_date` in `crates/core/src/pipeline.rs`, `Entry` in `crates/core/src/feed/sync.rs`, and `attribute_orders` and `order_new_attributes` in `crates/core/src/xml.rs`.
2. `upsert` with an existing entry: change only the fields whose value is different. Do not set `title` when the entry has no `title`. When no field changes, do not touch the entry.
3. `upsert` with no entry: when the base model of the draft (the feed from `DraftRow::base_xml`) has an entry in the same list that names the feed, add that base entry (with its key and origin, if the writer accepts them; else its value) in place of the new value. Change only the fields of item 2 that are different from the link. Put it at its base position: after the last current entry that comes before it in the base list, or first when no such entry exists. With no base entry, add the new entry last, as now.
   - `link` must read the base model for the page side and for the release side. Pass it into `upsert`, or read it in the closure of `edit_draft`.
4. `edit_draft`: after the change, when the draft model and the base model are the same apart from `lastBuildDate` and apart from the entry keys and origins, save the base bytes as the draft bytes. Use the date rule of `dated_like_base` as now. Then Review shows no change for that feed.
5. `attribute_orders`: for each name, start with the order of the first source element. For each later source element of the same name, add each attribute that the order does not have yet, directly after the attribute that comes before it in that element. Keep `order_new_attributes` as it is.
6. Tests in `crates/core/tests/publisher.rs` (and `crates/core/tests/` for the xml rule):
   - A written page with three releases, A, B and C, and a written release B. Unlink B, then link B again. The draft bytes of the page and of the release are the same as the base bytes.
   - The same with a second edit on the page that is not the link: the page draft has B at its earlier position, and the element of B has its written bytes.
   - A link of a page that is already linked, with no `title` in the written element: the draft has no `title`.
   - A file whose first `podcast:remoteItem` has no `title` and whose second has `title` before `rel`: a new element has `title` before `rel`.
   - Keep `a_new_link_goes_last_after_the_links_of_the_release` passing.
   - Each failure message names "observation 4" or "observation 5" of the walk of 2026-11-23 and the function to fix.

## Constraints

- Change only `crates/core/`. Do not change `crates/api/`, a file in `docs/` or `AGENTS.md`. A second task changes `crates/api/src/routes.rs` and `crates/api/src/tests.rs` at the same time.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of the crate.
- Run `cargo test -p studio-core` and `tools/gate.sh` at the end, and report the results. A gate failure in `crates/api` can come from the other task. Report it and do not correct it.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.

## Acceptance Criteria

Mechanical:

- The tests of step 6 pass.
- `cargo test -p studio-core` passes.

Visual, by the main session in the harness: an unlink and a link back on an artist page give Review with no change on the page and on the release.

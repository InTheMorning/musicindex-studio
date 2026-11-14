# Studio v1 Task 138: A Linked Page Row Names Its Host, And Add A Person Keeps The Source Of Its Values

Decision: [ADR 0199](../adr/0199-a-linked-page-row-names-its-host-and-add-a-person-keeps-the-source-of-its-values.md).

Status: Complete on 2026-11-14. The human check of ADR 0199 is open.

## Result

- `linkRows` gives each linked page that is not in the collection a `hostLine` from the page suggestions: `On {host}`, or `Host not known`. The person page shows it as one more line of the row. Rows of collection feeds get no host line.
- `feed_host` in the core gives the host for `page_suggestions` and for `add_person`.
- The body shape is `source: { feed_guid, title, feed_url, img_source, href_source }`. `PersonFields.source` is optional, so a body with no source does not change.
- The source data stays in the pending link of the sheet, not in the fields. Thus **Remove** of the link, or a pick of a feed of the collection, sends no source. The main session made this change after the independent review found old source data in the fields.
- The main session also added the cases of the review to the tests: a changed picture with the website source kept, and a body with values and no source. The core test fails on the code before the change, because `insert_person` gave no source.
- The walk found that the address of a picture row or a website row broke inside a word at 761 px (musicindex ADR 0065 R-FIT-3). These rows come from task 133. The address element now has `data-fit-allow-break`.
- Walk: `setup-138.py` with `steps-138-{light,dark}.json`. At 1280 px light and 390 px dark, the two "Prosperous Soul" rows show "On feeds.fountain.example" and "On wavlake.example". A person from **Add a Person** with "Ann Bell Music" shows `From "Ann Bell Music" on feeds.example` on the picture and the website. `$edges()`, `$overflow()` and the fit check at 390 px and 761 px give no finding.
- A first fit check at 390 px gave an overlap of **Save** and the role menu, because the page was scrolled below the top bar. With the page at the top, the check gives no finding.
- Open: ADR 0199 item 1 says that the host is the second line and that "Read only" stays on the first line. The row already shows "Read only" on its own line below the title, so the host is the third line. The main session kept the current row and added the host as one more line, as change 6 of this task says. The human check decides whether this is correct.

## Goal

1. Each linked page row on the page of a person has the second line `On {host}`, or `Host not known` (ADR 0199 items 1 to 4).
2. **Add a Person** from a page saves the picture and the website with the page as their source (items 5 to 7).

## Code Locations

These come from a search on 2026-11-14. Make sure that each one is correct before a change.

- `crates/api/ui/peoplepages.js:1086`: the linked page row shows `${row.title}` and `${row.line}`.
- `crates/api/ui/people.js:1130`: the rows of the external pages, with `title: p.title || 'Untitled feed'` and `line: [READ_ONLY_LINK, ...]`.
- `crates/api/ui/peoplepages.js:862`: the page reads `/people/${id}/page-suggestions` into `pageNotes`. Each entry has `feed_guid`, `host` and `missing`.
- `crates/core/src/people.rs:3545`: `page_suggestions` gets `host` from `values.feed_url` with `url::Url::parse` and `host_str`.
- `crates/api/ui/people.js:132` `addPersonSelection`, `:139` `addPersonPlan`, `:499` `addPersonBody`.
- `crates/api/ui/peoplepages.js:357`: the calls of the plan.
- `crates/api/src/people.rs:126`: `POST /people` takes `Json<PersonFields>` and calls `people::add_person`.
- `crates/core/src/people.rs:341` `insert_person` and `:356`: `add_person_value(..., None)` for the picture and the website. `:413` `add_person_value` fills `source_guid`, `source_title` and `source_host`.
- `crates/core/src/people.rs:238` `ValueSource`.
- Tests: `crates/api/ui-tests/people.test.mjs:233` (Add a Person), `:1873` (`personValueLine`), `:1904` (linked page labels). `crates/core/src/people.rs:5452` and `:5528`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| External page rows | extend | crates/api/ui/people.js | The rows get the host line from the page suggestions. |
| Page suggestions read | reuse | crates/api/ui/peoplepages.js | The page already reads the suggestions with the host. |
| Host of a feed address | extend | crates/core/src/people.rs | Move the host code of `page_suggestions` into one function, and use it in `page_suggestions` and in `add_person`. |
| Add a Person body | extend | crates/api/ui/people.js | `addPersonBody` sends the source. |
| Person fields | extend | crates/core/src/people.rs | `PersonFields` gets an optional source. |

## Rules For The Delegate

- Read ADR 0199, ADR 0196 and ADR 0192 first.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff, takes the screenshots and commits.
- Do not change `AGENTS.md`, an ADR or a task file.
- Do not change the schema of the collection.
- If a step needs an architecture decision that ADR 0199 does not give, stop and report.
- `node --check` does not find a template error in `crates/api/ui/*.js`. Check a copy with the extension `.mjs`.

## Changes

1. `crates/core/src/people.rs`: add `fn feed_host(feed_url: &str) -> Option<String>` with the code of `page_suggestions`. `page_suggestions` uses it.
2. `PersonFields` gets `source: Option<NewValueSource>`, with `feed_guid`, `title` and `feed_url`, and `#[serde(default)]`. `insert_person` makes a `ValueSource` with `host` from `feed_host`, for each value that change 3 selects. A source with no host or an empty GUID saves no source.
3. The body gets the source for each value separately: `img_source` and `href_source` are each `true` or `false`. Use the source for a value only when its flag is true. Keep a body with no source valid (ADR 0199 item 7). If a different body shape is simpler and keeps one source for the page, report it before you use it.
4. `crates/api/ui/people.js`: `addPersonSelection` keeps `feed_url` from `values.feed_url`. `addPersonBody` sends the source when the selection has a page. It sets the flag of a field when the field value is the same as the value of the page.
5. `crates/api/ui/people.js`, the external page rows: add `host` from the matching entry of the page suggestions, by `feed_guid`. Pass the suggestions from `peoplepages.js`. The second line is `On ${host}`, or `Host not known` when no entry gives a host.
6. `crates/api/ui/peoplepages.js:1086`: show the second line below the title, as one more line of the row. "Read only" stays on the first line.

## Acceptance Criteria

Mechanical:

1. A Node test: a linked page row with a suggestion that has `host: 'wavlake.com'` has the line `On wavlake.com`. A row with no suggestion and a row with `host: null` have `Host not known`. Each failure message starts with "ADR 0199" and gives the fix.
2. A Node test: `addPersonBody` for a selected page gives the source and both flags true. After a change of the picture field, the picture flag is false.
3. A Rust test: `add_person` with a source saves the picture and the website with `source_guid`, `source_title` and `source_host`. A second case with no source saves no source. Prove that the first case fails on the code before the change.
4. An API test: `POST /people` with a source gives the person with sources on both values.
5. The four checks pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.

Visual, by the main session:

1. The page of a person with two linked pages that have the same title and different hosts, at 1280 px light and 390 px dark. Walk: `setup-196.py` with new steps `steps-138-{light,dark}.json`. If `setup-196.py` gives no page with the same title, add a setup that does.
2. A person made with **Add a Person** from a page in the walk. The picture row and the website row say `From "{title}" on {host}`.

Human, by the operator: the gate of ADR 0199.

## Report

- The changed lines, with file and line.
- The body shape that you used for change 3.
- The test names, the proof of criterion 3, and the output of the four checks.

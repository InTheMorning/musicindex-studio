# Studio v1 Task 241: The Type Of A Feed With A Bad Or Missing Medium Tag

Decision: [ADR 0250](../adr/0250-studio-corrects-the-type-of-a-feed-with-a-bad-or-missing-medium-tag.md) items 1 to 10.

Status: Done.

## Problem

- A feed with `<podcast:meduim>music</podcast:meduim>` has no medium. Studio puts it in "Other feeds" and shows only the text of ADR 0090 item 1.
- Studio gives no cause and no correction. The user must edit the file on the server.
- The list of found feeds does not name the cause.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Medium read | reuse | crates/core/src/feed/sync.rs | `read_text(channel, Ns::Podcast, "medium")` stays the read of a correct tag. |
| Lossless tree | reuse | crates/core/src/xml.rs | `Element::elements`, `namespace`, `local_name` and `text` find the bad element. `Document::write_with_spans` and `ElementSpan` give the byte position for the line number. |
| Line numbers from spans | extend | crates/core/src/review.rs | `line_starts` converts a byte offset to a line. Make it `pub(crate)` or move it to `xml.rs`, and use it in both places. Do not copy it. |
| Draft repair | extend | crates/core/src/pipeline/check_repair.rs | The repair of item 7 goes through the draft as `apply_check_repair` does: the revision check, the draft change and the save. Share the revision check and the save. Do not copy them. |
| Draft start | reuse | crates/api/src/routes.rs | `ensure_draft` makes the draft of a connected feed. |
| Found feed candidate | extend | crates/core/src/discover.rs | `Candidate` gets `medium_problem`. `feed_candidates` fills it from the parsed document. |
| Feed page | extend | crates/api/ui/app.js | `ReadOnlyFeed` shows the notice and the menu. `FolderFeeds` and `FoundFeeds` show the cause in the row. |
| Feed type texts | extend | crates/api/ui/shell.js | `otherFeedText` and `mediumKind` stay. The notice texts go in a new exported function in this module, which has Node tests. |
| Select menu | reuse | crates/api/ui/app.js | `Field` with `select` makes the menu "Type of this feed", as the other select menus do. |

## Changes

1. Read ADR 0250, ADR 0090, ADR 0174 item 6 and ADR 0012 first. Then read `read_text` in `crates/core/src/feed/sync.rs`, the `Element` API and `write_with_spans` in `crates/core/src/xml.rs`, `line_starts` in `crates/core/src/review.rs`, `apply_check_repair` in `crates/core/src/pipeline/check_repair.rs`, `discover.rs`, `connector_feeds` and `ensure_draft` in `crates/api/src/routes.rs`, `crates/api/src/feed_check.rs` (the repair route), and `ReadOnlyFeed`, `FolderFeeds` and `FoundFeeds` in `crates/api/ui/app.js`.
2. Core, items 1 to 3. Add a module `crates/core/src/medium_problem.rs` with:
    - `pub enum MediumProblemKind { MisspelledTag, OtherNamespace, MisspelledValue, NoTag }`, serialized in snake case.
    - `pub struct MediumProblem { kind, tag: Option<String>, line: Option<u32>, suggested: Option<String> }`. `tag` is the full element as it is in the file, for example `<podcast:meduim>music</podcast:meduim>`. `suggested` is `music`, `musicL` or `publisher`, or `None`.
    - `pub fn medium_problem(document: &Document) -> Option<MediumProblem>` with the conditions of ADR 0250 item 1. The edit distance is the Levenshtein distance on lowercase text. A known medium that Studio edits (`music`, `musicL`, `publisher`, in uppercase or lowercase letters) gives `None`. A different known value, for example `podcast`, `audiobook`, `video`, `film`, `newsletter` or `blog`, gives `None` (item 3). Use the list of medium values of the Podcasting 2.0 namespace, with the `L` forms.
    - `suggested` for `MisspelledTag` and `OtherNamespace` is the trimmed text of the bad element when it is a medium of ADR 0250 item 5 (no difference of case; give the correct spelling `musicL`). For `MisspelledValue`, it is the value of the closest medium of the three. For `NoTag`, it is `None`.
    - `line` comes from the span of the bad element. Give `None` when the written bytes of the document are not equal to the read bytes.
3. Core, item 7. Add `pub fn correct_medium(collection, hosts, id, sent_revision, medium: &str) -> Result<DraftRow, PipelineError>` in `crates/core/src/pipeline/check_repair.rs` or a new file next to it.
    - It refuses a `medium` that is not `music`, `musicL` or `publisher`, and a draft with no medium problem, with `InvalidOperation`.
    - `MisspelledTag` and `OtherNamespace`: replace the bad element with `<podcast:medium>{medium}</podcast:medium>` at the same position. Remove each other element of item 1.
    - `MisspelledValue`: change only the text of `podcast:medium`.
    - `NoTag`: set `Feed.medium` through the typed model, so that the writer puts the tag at its usual position.
    - Each other element keeps its bytes (ADR 0008, ADR 0174 item 6).
4. API, items 6 and 10.
    - The feed JSON that `ReadOnlyFeed` gets (the feed row of `GET /feeds` and `GET /feeds/{id}`, or a new route `GET /feeds/{id}/medium-problem`) gives `medium_problem` from the last read bytes of the feed. Select the cheaper place, and give the reason in the report.
    - `POST /feeds/{id}/medium` with `{ "revision": "...", "medium": "music" }` calls `ensure_draft` and then `correct_medium`. It refuses a feed with no file host and a feed with no medium problem. It gives the draft as the check repair route does.
    - `GET /connectors/{id}/feeds` gives `medium_problem` for each candidate.
5. UI, items 4, 5, 6, 8 and 9.
    - In `crates/api/ui/shell.js`, add `mediumProblemView(problem, hasHost)` that gives `{ title, text, menu: boolean, start, hostLine }` with the texts of ADR 0250 items 4, 5 and 9, and `mediumProblemCause(problem)` that gives the row cause of item 10: "misspelled tag {tag}" or "no medium tag".
    - `ReadOnlyFeed`: when the feed has a medium problem, show a notice with that title and text in place of `otherFeedText`. With a file host, add the select menu "Type of this feed" with "Select a type", "Album (music)", "Playlist (musicL)" and "Artist or label page (publisher)", and the primary button **Review the Change**. The button is disabled until a type is selected. It calls the route of step 4 and then opens Review of the feed, as a check repair opens Review. Without a file host, show the host line.
    - After the write, the feed is read again, as each write does. Make sure that the left panel and the main pane then show the feed as a release (the feed list reloads).
    - `FolderFeeds` and `FoundFeeds`: put the cause of `mediumProblemCause` after the type in the `sub` line of the row.
6. Tests:
    - Rust, core: each case of the mechanical checks of ADR 0250 for `medium_problem`, and each case for `correct_medium` there. A test that `<podcast:meduim>music</podcast:meduim>` on line 12 of a test feed gives `line == Some(12)`.
    - Rust, API (`crates/api/src/tests.rs`): `POST /feeds/{id}/medium` refuses a feed with no problem and accepts a feed with `podcast:meduim`. The diff then has `podcast:medium` and no `podcast:meduim`. `GET /connectors/{id}/feeds` gives the `medium_problem` of a candidate.
    - Node: `mediumProblemView` for each kind, for no line, for no host and for the start value; `mediumProblemCause` for each kind.
    - Each failure message names ADR 0250 and its item, and the function to fix.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0250 and its item.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of each crate.
- Feed rules stay in the core (ADR 0065). The UI does not read XML or compare medium values. It shows the values of the API.
- Check each changed `crates/api/ui/*.js` file with `tools/node --check` on a `.mjs` copy. Plain `node` is not in PATH.
- Run `tools/gate.sh` at the end and report its result.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.

## Acceptance Criteria

Mechanical:

- The core tests of step 6 pass, for each condition of ADR 0250 item 1 and for item 3.
- The core repair test proves that each other element keeps its bytes.
- The API tests of step 6 pass.
- The Node tests of `mediumProblemView` and `mediumProblemCause` pass.
- `tools/gate.sh` passes.

Visual, by the main session with `tools/tutorial-walk/` and a test feed with `<podcast:meduim>music</podcast:meduim>`:

- The page of the feed shows the notice with the line number, and the menu starts on "Album (music)".
- **Review the Change** opens Review with the tag change. After the write, the feed opens in the editor.
- The list of found feeds shows "misspelled tag" on that feed.

## Review Fixes

The main session read the first diff. Make these changes to the diff in the working tree. Do not change `crates/core/src/media.rs` or `crates/api/ui/editor.js`. They have other work of the main session.

1. `list_feeds` in `crates/api/src/routes.rs` must not read a file from a host. The first diff calls `connector.read(path)` for each feed with no draft at each list request. An SFTP host makes the list slow. Remove the `medium_problem` code from `list_feeds`.
2. Add `GET /feeds/{id}/medium-problem`. It gives `{ "medium_problem": ... }` for one feed. The bytes come from the base of the draft when a draft exists. Else they come from the file host of the feed. Else, for a feed with no file host, they come from its public address: use the read that `GET /feeds/{id}/outside` uses, and do not copy it. `ReadOnlyFeed` calls this route when it opens. This gives item 9 for a feed with no file host.
3. Remove the field `source` and `matches_source` from `Document` in `crates/core/src/xml.rs`. The field changes the equality of each `Document` and keeps a second copy of each parsed feed. In its place, add `pub fn medium_problem_in_bytes(bytes: &[u8]) -> Option<MediumProblem>` in `medium_problem.rs`. It parses the bytes, and gives the line only when `document.write()` is equal to the bytes. Keep `medium_problem(&Document)` with no line, for the repair. The callers that have the bytes (`feed_candidates`, `from_sources`, the new route) use `medium_problem_in_bytes`. Change the test `changed_source_bytes_have_no_line_number` to the new function.
4. The texts of `mediumProblemView` must not have backtick characters. The UI shows them as text. Use `This feed has no <podcast:medium> tag.` and `Studio reads only <podcast:medium>.` Change the Node tests.
5. Remove the hint "Suggested type: ..." in `ReadOnlyFeed`. ADR 0250 has no such text. The menu start shows the suggested type.
6. Remove `onCorrected` and the local change of `feed.medium` in `app.js`. A later `load()` replaces it, and the editor then closes. In its place, `list_feeds` gives the medium of the draft model when a draft exists, the row has no medium that Studio edits (`music`, `musicL`, `publisher`), and the draft model has one of them. `files_feed` already gives that model. Then the feed opens in the editor while the draft exists, and Review can show the change (ADR 0250 item 8). Add an API test: after `POST /feeds/{id}/medium`, `GET /feeds` gives `medium` `music` for that feed. Keep the retry of `OPEN_REVIEW`; `editor.js` uses the same pattern.
7. Remove the unused `_hosts` argument of `correct_medium` and its caller.

Run `tools/gate.sh` at the end and report its result. Do not commit and do not stage.

## Result

- The main session read the diff after each run and made one more change: the menu starts on the suggested type when the route of the problem answers.
- Technical choice of the main session: `GET /feeds/{id}/medium-problem` gives the problem of one feed when its page opens. The feed list does not read a host. While a draft gives a medium that Studio edits, the feed list gives that medium, so the editor opens the feed and Review can show the change.
- Mechanical criteria: `tools/gate.sh` passed in the main session.
- Visual criteria: a walk with a debug build showed the three visual checks. The write itself was not done, because the test host has no public address that Studio accepts.

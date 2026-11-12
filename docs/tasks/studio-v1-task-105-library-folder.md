# Studio V1 Task 105: Library Folder Of A Release

Status: Complete, with its screenshot visual check on 2026-11-11. The operator accepted the screenshots. The human test with a real release folder passed on 2026-11-12, in the release build of local `main`.

A new release folder gets the default path when Studio imports or reads a published feed. `migrate.rs` keeps the folder names of ADR 0036. `feeds.folder` holds a path relative to the library folder, so no schema change was necessary. The routes are `PUT /feeds/{id}/library-folder` and `GET /library/folders`.

Owner of the rules: [ADR 0167](../adr/0167-each-release-has-an-editable-library-folder.md). It uses ADR 0035 and ADR 0038. It uses ADR 0088 for the levels and ADR 0105 for a server that holds no assets. It uses ADR 0149 for the library of a collection.

## Goal

A new release folder has a path by artist and label. The Files page shows the folder, and the user changes it or moves it to the default.

## Packets

- 105a, core: the default path, the move and the records (ADR 0167 items 1 to 3, 7 to 10).
- 105b, UI: the row **Library folder**, **Change…** and **Move to Default** (items 4 to 6 and 11).

## Files To Inspect

- `crates/core/src/library.rs`: `release_folder_name` (line 187), `give_published_folder` (line 250), `Library::path`, `put_file`, `write_feed_xml` and the check of a name part (around line 175).
- `crates/core/src/migrate.rs`: the use of `release_folder_name` (lines 104, 235 and 244).
- `crates/core/src/collection.rs`: `FeedRow.folder` (line 942) and `set_feed_folder` (line 2498), and the file records that hold a path.
- `crates/core/src/publisher.rs`: `Catalog` with `publishers`, `parents` and `children`, and `link_levels` (line 435). They give the artist and the label of a release (ADR 0088).
- `crates/core/src/lookup.rs`: `Level` and `classify`.
- `crates/api/src/routes.rs`: the Files data of a feed, `get_library`, `put_library`, `scan_library` (around lines 3338 to 3500 and 5163), and the check for a server with no assets (ADR 0105).
- `crates/api/ui/editor.js`: the Files page, the sections "Files", "File hosts" and "Library files" (around lines 4300 to 4350), with the label "File host of the feed XML".
- `crates/api/ui/app.js`: `assetMode` and `holdAssets` (around line 90).
- `crates/core/tests/` for the existing library tests, and `crates/api/ui-tests/` for the style of the model tests.

## Constraints

- No feed rule in the UI (ADR 0065).
- UI files are in `crates/api/ui/*.js` (Preact + htm). `node --check` misses template errors, so check a `.mjs` copy of each changed UI file.
- The core gives the default path. The UI has no copy of the layout of item 1.
- No start of Studio moves an existing folder (item 8). A rename of an artist or a label moves no folder (item 9).
- A move changes no feed and no file host (item 10). It changes only the library on this computer.
- A move that fails changes nothing (item 7).
- A server that holds no assets has no **Library folder** row (item 11).
- The screenshots of this task show the **Library folder** row to the operator before the commit (ADR 0167).

## Acceptance Criteria

Mechanical, with Rust tests in `studio-core` (105a):

- The default path of a release of an artist is `<library>/<artist>/<release title>/`.
- The default path of a release of an artist on a label is `<library>/<label>/<artist>/<release title>/`.
- The default path of a release with no artist is `<library>/Other/<release title>/`.
- A name in use gets ` 2`, ` 3` and so on.
- The feed XML keeps its file name in the release folder.
- A move of a release folder moves the files. The library records and the files stay correct.
- A move to a folder outside the library folder is refused, and nothing changes.
- A move that fails in the middle changes nothing.
- A rename of an artist moves no folder.
- A start of Studio with existing folders moves none of them.

Mechanical, with Rust tests in `crates/api` (105a): the data of the Files page gives the folder path, the default path and whether they differ. A server with no assets gives no folder.

Mechanical, with Node tests (105b): the model of the row always shows **Change…**. It shows **Move to Default** with the default path only when the path is not the default. A server with no assets gives no row.

Visual, by the agent (105b), with screenshots by `tools/drive-ui.mjs` at 1100 px and at 390 px with touch, in the dark and the light theme. Run `$edges()` and `$overflow()` with each screenshot:

- The **Library folder** row with the default path.
- The row with another path, the default path and **Move to Default**.
- The refusal of a folder outside the library, with its cause.

Human, by the operator: a move of a real release folder, then the files open in the release.

## Test Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation Triggers

- `FeedRow.folder` is one name, and `Library::path` takes one folder name. Store the path relative to the library folder. Name the schema change in the report. Stop if the change breaks the move of ADR 0149.
- ADR 0167 gives the cleanup of a name (item 2), more than one artist or label (item 3) and the folder sheet (item 6). A case that these items do not cover is an escalation.
- A release in the library of a collection that has a root outside the data folder (ADR 0149).
- Show the screenshots of the **Library folder** row and the folder sheet to the operator before the commit.

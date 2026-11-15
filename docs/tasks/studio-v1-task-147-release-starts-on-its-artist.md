# Studio v1 Task 147: A Release Starts On The Page Of Its Artist

Decision: [ADR 0213](../adr/0213-a-release-starts-on-the-page-of-its-artist.md).

Status: Open. The code is not built.

## Goal

1. A release has a kind that Studio keeps and shows (ADR 0213 items 10 to 16).
2. A new release gets the pages of its artist (items 7 and 8).
3. A feed with no page gets a finding, and Review stops its write (items 17 to 23).
4. A release starts only on the page of an artist (items 1 to 6).

The work has three packets. Do them in this sequence: 147a, 147b, 147c. One delegate does one packet.

## Code Locations

These come from a read on 2026-11-14. Make sure that each one is correct before a change.

- `crates/core/src/collection.rs`: `MIGRATIONS` (about line 26), `SCHEMA_VERSION`, `FeedRow` and the functions that read a feed row.
- `crates/api/src/routes.rs`: `new_album_feed` (the route `POST /feeds/new`), `list_feeds` with the feed row of the API, `write_feed` (about line 3406, the route `POST /feeds/{id}/write`) and `diff_json`.
- `crates/core/src/publisher.rs`: `link` (about line 214) joins a publisher feed and a release with roles.
- `crates/core/src/feed_check.rs`: `FindingKind`, `FindingLevel`, `RepairKind` and `check(feed, facts)`.
- `crates/core/src/feed.rs`: the typed feed, with its tracks, its live items and its `podcast:publisher` links.
- `crates/api/ui/addfeeds.js`: `addChoices`, `MAKE_TYPES`, `makeFields` and `makeTitle`.
- `crates/api/ui/welcome.js`: `releaseAdd(feed, name)` and `FIRST_STEPS`.
- `crates/api/ui/app.js`: `MakeFeedForm` (search `function MakeFeedForm`), the Add feeds page (search `addChoices`), and the list view of a group with the add action.
- `crates/api/ui/shell.js`: `mediumKind` and `feedKind` give the type label "Album".
- `crates/api/ui/editor.js`: `SettingsPage`, `CheckPage`, the review sheet with its refusals (search `Studio refuses this write`), and the section "Linked artist and label pages" (about line 2562).
- `crates/api/ui/feedcheck.js`: the words of each finding and each repair.
- Tests: `crates/api/ui-tests/addfeeds.test.mjs`, `welcome.test.mjs`, `shell.test.mjs`, `feedcheck.test.mjs`, `crates/api/src/tests.rs`, `crates/api/src/tests/feed_check.rs`, and the tests of `crates/core/src/collection.rs` and `feed_check.rs`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Collection schema | extend | crates/core/src/collection.rs | One migration adds the column of the kept kind. |
| Feed row of the API | extend | crates/api/src/routes.rs | The row gives the kept kind and the kind to show. |
| Route that makes a feed | extend | crates/api/src/routes.rs | The route gets the kind and the artist page of the new release. |
| Publisher link | reuse | crates/core/src/publisher.rs | `link` makes each link of the new release. |
| Feed check | extend | crates/core/src/feed_check.rs | Two finding kinds and one repair kind. |
| Write route and diff | extend | crates/api/src/routes.rs | The diff gives the reason, and the route refuses the write. |
| Add feeds model | extend | crates/api/ui/addfeeds.js | The rows and the types have no album. The release form has its fields. |
| Add action | extend | crates/api/ui/welcome.js | Only an artist page gives the action. |
| Type label | extend | crates/api/ui/shell.js | The label of a release shows its kind. |
| Form, pages and review | extend | crates/api/ui/app.js | The form, the hint, the menu of the kind and the refusal. |
| Check words | extend | crates/api/ui/feedcheck.js | The words of the two findings and the repair. |

## Rules For The Delegate

- Read ADR 0213 first, then ADR 0212, ADR 0164 and ADR 0174.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff, takes the screenshots and commits.
- Do not change `AGENTS.md`, an ADR or a task file.
- Change a shared component only as this task says. Do not remove a style or a comment that a different part uses. Search each caller before you change a signature.
- A check or a lookup that runs with a locked vault must not open a file host.
- If a step needs an architecture decision that ADR 0213 does not give, stop and report.
- `node` is not on the PATH. Use `tools/node`. `node --check` does not find a template error in `crates/api/ui/*.js`. Check a copy with the extension `.mjs`.
- Use the exact words of ADR 0213 for each text. Use tokens only in the styles.
- Each failure message of a new test names "ADR 0213" and gives the fix.

## Changes Of Packet 147a: The Kind And The Pages Of A New Release

1. `crates/core/src/collection.rs`: one new migration adds the column `release_kind TEXT` to `feeds`, with no default. The values are `album`, `ep` and `single`, or NULL. `FeedRow` gets the field. Add a function that sets it.
2. The core gets one pure function for the kind to show: the kept kind, or the kind from the count of the tracks (ADR 0213 item 11). Put it in the module that owns the typed feed or the collection, and say where in the report.
3. `crates/api/src/routes.rs`:
    - The feed row gets `release_kind` (the kept value or null) and `kind_shown` (`album`, `ep` or `single`) for a feed with the medium `music`. A playlist and a publisher feed get null for the two fields.
    - `POST /feeds/new` gets the optional fields `kind` and `publisher_id`. A value of `kind` that is not one of the three gives `400`. With `publisher_id`, the route links the new feed to that publisher feed with the role `artist`, and then to each hosting page and label page of the newest other release of that artist, with the roles of that release (items 7 and 8). The newest release is the release with the newest creation time in the collection.
    - Add `PUT /feeds/{id}/kind` with the body `{ "kind": "album" | "ep" | "single" | null }`. It saves at once, and it changes no draft.
4. Tests of 147a: the criteria 1 to 3 below.

## Changes Of Packet 147b: The Finding And The Stop

1. `crates/core/src/feed_check.rs`: add the finding kinds `NoArtistPage` and `NoPublisherPage`, and the repair kind `LinkAPage`. `check` gives them from the typed feed, with no network fact (items 17 to 20):
    - A feed with the medium `music` and one track or more, or with no live show, needs a `podcast:publisher` link whose roles include `artist`. Without it, the finding is `NoArtistPage`.
    - A feed with the medium `musicL`, and a feed with the medium `music` that has live shows and no track, needs one `podcast:publisher` link or more. Without it, the finding is `NoPublisherPage`.
    - A publisher feed gets no finding.
    - Select the level that fits a finding with a repair button.
2. The core gets one function that gives the reason of the stop for a typed feed: `no_artist_page`, `no_publisher_page` or none. The finding and the stop use it.
3. `crates/api/src/routes.rs`:
    - The diff of a feed gets `page_refusal`: the reason, or null. It is null for a diff that deletes the feed file and for a feed that is hidden from apps (item 22).
    - `write_feed` refuses with `409` and the code of the reason when `page_refusal` is not null. The route of a change set and the route of a granted write do not change.
4. Many tests write a release with no link. Change the fixtures of those tests so that the feed has an artist link, in the helper that makes the feed XML where one exists. Do not weaken the rule, and do not add a switch that turns it off.
5. Tests of 147b: the criteria 4 to 6 below.

## Changes Of Packet 147c: The UI

1. `crates/api/ui/addfeeds.js`: `addChoices` and `MAKE_TYPES` have no album. The group "Make a new feed" gets the hint of ADR 0213 item 2. Add the fields of the release form (item 6) and the values of the menu "Kind" (item 12).
2. `crates/api/ui/welcome.js`: `releaseAdd` gives the action only for an artist page (item 4). The first-step row "Add a release" of a label opens the list of the artist pages (item 5).
3. `crates/api/ui/app.js`: the release form opens only with a publisher feed. It has no menu "Type", it sends `kind` and `publisher_id`, and it shows the hint of item 16 at the menu. The generic form has no album.
4. `crates/api/ui/shell.js`: `feedKind` gives the label "Album", "EP" or "Single" from `kind_shown` (item 14).
5. `crates/api/ui/editor.js`: the tab **Settings** of a release has the menu "Kind" (item 13). It calls `PUT /feeds/{id}/kind` and then loads the feed list.
6. `crates/api/ui/feedcheck.js` and the tab **Check**: the words of the two findings and the button **Link a Page**, which opens the section "Linked artist and label pages" (items 18 and 19).
7. The review sheet: with `page_refusal`, it shows a refusal with the text of item 18 and **Link a Page**, and it has no write button (item 21).
8. Tests of 147c: the criteria 7 to 10 below.

## Acceptance Criteria

Mechanical:

1. A core test: a new collection and a collection of the version before have the column after the migration. The kind function gives a single for 1 to 3 tracks, an EP for 4 to 6, an album for 0 and for 7 or more, and the kept kind first.
2. An API test: `POST /feeds/new` with `kind` and `publisher_id` gives a feed row with that `release_kind`, a link to the artist page in the two drafts, and the hosting and label links of the newest other release of the artist. With no other release, only the artist link.
3. An API test: `PUT /feeds/{id}/kind` changes `release_kind` and `kind_shown`, changes no draft, and gives `400` for a wrong value.
4. A core test: each case of packet 147b, change 1.
5. An API test: the diff of a release with no artist page has `page_refusal`, and the write gives `409` with `no_artist_page`. The same for a playlist with `no_publisher_page`.
6. An API test: each write of ADR 0213 item 22 has no stop.
7. A Node test of `addChoices`, `MAKE_TYPES` and the release form fields.
8. A Node test of `releaseAdd`: the action for an artist page, and none for a label page and a hosting page.
9. A Node test of `feedKind`: the label of each kind, and "Album" with no `kind_shown`.
10. A Node test of the words of the two findings, the repair and the refusal.
11. The four checks pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.

Visual, by the main session:

1. The page "Add a Feed" and the release form from an artist page, at 1280 px light and 390 px dark.
2. The tab **Settings** with the menu "Kind", and a tile of each kind.
3. The finding on the tab **Check**, and the refusal in Review.

Human, by the operator: the gate of ADR 0213.

## Report

- The changed lines, with file and line.
- The test names for each criterion, and the output of the four checks.
- Each point where the code did not agree with this task.

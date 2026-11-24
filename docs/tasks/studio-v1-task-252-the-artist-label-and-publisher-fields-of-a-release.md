# Studio v1 Task 252: The Artist, Label And Publisher Fields Of A Release

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "Levels And Moves".
Decision: [ADR 0257](../adr/0257-a-release-has-one-artist-one-label-and-one-publisher-in-its-fields.md) items 1 to 8, 11, 13 and 16 to 20.
Mockup: `.studio-dev/design-brief/prototype/release-fields.html` version 4 (artifact VQtbDPfkjihUGaWkwjWRuX). Git ignores it.

Status: Packet 252a is done. Packets 252b to 252d are not started.

Technical choices of the main session (no ADR, because ADR 0257 gives the rule and this is how the code does it):

- One core function changes the page of one role of a release, for example `publisher::set_release_page(collection, release, role, page)`. It also applies the rule of the value that follows (ADR 0257 item 5) and the text of `itunes:author` (item 3). The UI has no copy of these rules (ADR 0065).
- The core gives the credits of a release in one read. They are the page of each role, the value that follows, and a flag that tells if **Publisher** follows. They also give the other pages of a role in the file (item 11), and the alias, if one exists.
- Typing in the alias field edits `itunes:author` in the draft, as the text field "Artist" does now. An empty alias calls the core, and the core writes the title of the artist page.

## Goal

The first tab of a release has the heading "Credits" with the select menus **Artist**, **Label** and **Publisher**, and the section "Release alias". A change of a field goes into the change set of the Catalog, with a notice and Undo.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Link with roles | reuse | crates/core/src/publisher.rs | `link` and `unlink` of task 250. |
| Catalog change set and steps | reuse | crates/core/src/change_set.rs | The functions of packet 251b. |
| Release page move route | extend | crates/api/src/routes.rs | `move_release_to_pages` becomes the route of one field, or goes when no caller stays. |
| Writable pages of a role | reuse | crates/api/ui/movepages.js | `writablePages`. |
| Release fields of the first tab | extend | crates/api/ui/editor.js | `ChannelFields` and `AlbumPage`. |
| Page field lists | extend | crates/api/ui/pages.js | `FEED_FIELDS` and `PAGES.album`. |
| Linked section | extend | crates/api/ui/editor.js | `ArtistsAndLabels` stays for a playlist, a show feed and a person only. |
| Undo notice | reuse | crates/api/ui/undonotice.js | The notice with **Undo** and **Review**. |
| Publisher page form | reuse | crates/api/ui/app.js | `MakeFeedForm` makes a new page from a menu value. |
| Refusal panel | extend | crates/api/ui/editor.js | `ReviewSheet` candidates, `linkAndContinue` and `pageLinkCandidates`. |
| Page findings | extend | crates/api/ui/app.js | `openFinding` opens the first tab and puts the focus on **Artist**. |
| Node tests | extend | crates/api/ui-tests/pages.test.mjs | The first tab of a release. |
| Node tests of the refusal | extend | crates/api/ui-tests/feedcheck.test.mjs | The refusal panel of ADR 0218. |

## Constraints

- Each rule is in the core (ADR 0065). The view model of the fields only shows what the core gives.
- Follow the mockup for the order, the texts and the hints. Use select menus, not radio groups.
- No change of a field writes a file. Each change goes into the change set of the Catalog (ADR 0257 item 8).
- A file with two pages of one role still loads and writes (ADR 0257 item 11).
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Follow the `rust-dev` skill for Rust. A UI module passes `node --check` on a `.mjs` copy.
- The agent does not commit and does not stage. The main session reads the diff, takes the screenshots, runs the gate and commits.

## Files To Read First

- ADR 0257, ADR 0256 items 8, 9, 13 and 17 to 19, ADR 0218 items 3 to 6, ADR 0146 item 1.
- The mockup `.studio-dev/design-brief/prototype/release-fields.html`.
- `crates/core/src/publisher.rs` after task 250, and `crates/core/src/change_set.rs` after packet 251b.
- `crates/api/src/routes.rs`: `move_release_to_pages` (near line 3680).
- `crates/api/ui/editor.js`: `ChannelFields` (near line 3147), `AlbumPage` (near line 3333), `ArtistsAndLabels` (near line 2996), `MoveSheet` (near line 2921), `ReviewSheet` (near line 2319), `linkAndContinue`, `pageLinkCandidates`.
- `crates/api/ui/pages.js`, `crates/api/ui/movepages.js`, `crates/api/ui/undonotice.js`, `crates/api/ui/app.js` (`openFinding`, `MakeFeedForm`).

## Packets

### 252a: The Credits Of A Release (Core)

1. `set_release_page(collection, release, role, page: Option<FeedId>)` for the roles `artist`, `label` and `hosting`. It removes the role from each link of the release that has it. A link with no role after that goes. It adds the role to the link of the selected page, or makes the link. It records one step of the Catalog change set.
2. Artist: `page` is required. With no alias, the core writes the title of the new artist page to `itunes:author`. An alias is an `itunes:author` that is not the title of the artist page before the change. An alias does not change.
3. ADR 0257 item 5: the core gives the value that follows. With a label, it is the publisher page directly above the label in the tree, first from A to Z. With no label, it is the publisher of the newest other release of the artist with no label, or else the publisher page directly above the artist page. With none, it is "Not published".
4. When **Publisher** had the value that follows before a change of **Artist** or **Label**, the change also sets **Publisher** to the value that follows after the change (item 5, last part). This is one step.
5. `clear_release_alias`: the core writes the title of the artist page to `itunes:author`.
6. `remove_other_pages(release, role)` (item 11): it keeps the first page of the role in the sequence of `podcast:publisher` and removes the role from the other links, as one step.
7. `release_credits(release)` gives the page of each role, the value that follows, `publisher_follows`, the other pages of each role, and `alias` (or none).
8. Tests in `crates/core/tests/publisher.rs`:
   - A change of label replaces the label link in one step, and Undo puts back the old link.
   - One page `artist hosting` in **Artist** and **Publisher** gives one link with `rel="artist hosting"`.
   - **Publisher** that follows changes with the label. **Publisher** that the user selected does not change.
   - An empty alias writes the title of the artist page. An alias stays after a change of artist.
   - A file with two label pages: `release_credits` gives the first and names the second. `remove_other_pages` keeps the first.
   - The written `podcast:publisher` has the sequence publisher, label, artist.

### 252b: The Routes (API)

1. `GET /api/v1/feeds/{id}/credits` gives `release_credits`.
2. `PUT /api/v1/feeds/{id}/credits` with the role and the page (or none) calls `set_release_page`. It gives the notice text of ADR 0257 item 8, the change set and the new credits. `role_taken` is not possible here, because the core removes the old link first.
3. `POST /api/v1/feeds/{id}/credits/alias/clear` and `POST /api/v1/feeds/{id}/credits/remove-others` with the role.
4. `move_release_to_pages` goes when no caller uses it, with its tests changed to the new route.
5. Tests in `crates/api/src/tests.rs`: a field change, the notice text, Undo through the route of packet 251c, and the read of a file with two artist pages.

### 252c: The Fields (UI)

1. A view model module, for example in `crates/api/ui/movepages.js` or a new `crates/api/ui/credits-fields.js`, gives the values of each menu, the placeholder, the hints of items 2, 3, 5 and 11, and the button **Follow the Label Again** or **Follow the Artist Page Again**. It reads only `release_credits` and the feed list. Reuse `writablePages`.
2. The first tab of a release: the heading "Credits", then **Artist**, the closed section "Release alias" (open when `alias` exists), **Label** and **Publisher**, below "Kind" (ADR 0256 item 13). The text field "Artist" of a release goes. A playlist and a show feed keep their fields.
3. "New Artist Page…", "New Label Page…" and "New Publisher Page…" open `MakeFeedForm` with that role. After the make, the field selects the new page.
4. A change calls the route of packet 252b and shows the notice with **Undo** and **Review** (reuse `undonotice.js`).
5. A release has no section "Linked artist and label pages" and no **Move Up** or **Move Down** for its links. The menu **⋯** of a release row has no "Move To…" or "Artist and label…". `MoveSheet` goes when no caller uses it.
6. Node tests: the view model gives the values of each menu, the value that follows and its hint, the hint of a selected publisher, the hint of a file with two artist pages, and the alias hint. The tests of `MoveSheet` and `moveMenus` that assert the replaced rule go.

### 252d: Link A Page And The Refusal Panel (UI)

1. ADR 0257 item 18: for a release, **Link a Page** opens the first tab and puts the focus on **Artist**. For a playlist and a show feed, it opens the section "Linked artist and label pages".
2. ADR 0257 item 19: for a release, the menu of the refusal panel has only the artist pages. **Link and Continue** calls the route of packet 252b for the role `artist`. "New artist page…" makes a page with the `itunes:author` text of the release as its name.
3. Node tests in `crates/api/ui-tests/feedcheck.test.mjs`: the candidates of a release are artist pages only, and the candidates of a playlist do not change.

## Acceptance Criteria

Mechanical:

- Each test named in the packets passes.
- `cargo test -p studio-core`, `cargo test -p studio-api` and `node --test crates/api/ui-tests/` pass. `tools/gate.sh` passes.
- `rg 'Artist and label…' crates/api/ui` gives no line.

Visual (the main session with screenshots, then the operator):

- The first tab of a release at 1280 px and at 390 px, in Light and Dark, with the alias open. It looks as the mockup does.
- A change of **Label** with **Publisher** that follows, the notice, then Undo.

## Stop And Ask When

- The core cannot tell an alias from the title of the artist page.
- A caller outside this task uses `move_release_to_pages` or `MoveSheet`.
- The mockup and the ADR give different texts.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- docs/tasks/studio-v1-task-252-the-artist-label-and-publisher-fields-of-a-release.md
- The files of "Files To Read First"

Goal:
- The packet that the main session names (252a, 252b, 252c or 252d). Do only that packet.

Constraints:
- The constraints of the task file.

Do not touch:
- docs/, AGENTS.md, crates/desktop/, crates/desktop-app/. In 252a and 252b, also crates/api/ui/ and crates/api/ui-tests/.

Acceptance criteria:
- The tests of the packet pass, and the mechanical criteria of the task file.

Test commands:
- cargo test -p studio-core
- cargo test -p studio-api
- node --test crates/api/ui-tests/
- tools/gate.sh

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

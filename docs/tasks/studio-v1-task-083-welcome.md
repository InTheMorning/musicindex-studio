# Studio v1 Task 083: The Welcome Page Of A New User

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The First Visit".
Decision: [ADR 0119](../adr/0119-a-new-user-starts-as-a-musician-or-a-label.md).

Status: Not started.

## Goal

A user with no feed sees "Welcome to Studio". The user starts as a musician or as a label, gives a name and picks a file host. Then Studio makes the draft of an artist page or a label page, with its next step. The first steps use no word "feed".

## Packets

The task changes only the UI, in one packet. The core and the API already make a publisher feed with a role (`POST /feeds/new`).

Visual check, by the planning model, before the commit: screens 2 to 6 of `.studio-dev/mockup-start/site/`, from a new data folder. The check uses 1100 px, and 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human test, by the operator, after the deploy: the human test of ADR 0119.

## Packet 083a: The Welcome Page And Its Sheet

### Files

Inspect: `crates/api/ui/app.js` (the empty Releases page near "No releases yet.", `onAddFeed`, `canImport`, `ImportFlow`, `MakeFeedForm` and its body for `POST /feeds/new`, `AddHostForm` and the forms of each file host type, `serverMode`), `crates/api/ui/addfeeds.js` (`addChoices`, `makeFields`, `feedFileStart`), `crates/api/ui/editor.js` (`PublisherPage`, its section `releases`), `crates/api/ui/shell.js` (`isPublisher`, `FIRST_CLASS_ROLES`), `crates/api/ui/pages.js`, `crates/api/ui-tests/addfeeds.test.mjs`, `.studio-dev/mockup-start/site/start.js` (`start`, `artistName`, `host`, `artistDone`, `labelDone`).

Change: a new model module `crates/api/ui/welcome.js`, `crates/api/ui/app.js` and `crates/api/ui/editor.js`. Change `crates/api/ui/styles.css` only for the large row and the step line. Add the test `crates/api/ui-tests/welcome.test.mjs` and the user guide page `docs/user-guide/first-steps.md`, with its lines in `README.md` and `docs/README.md`.

Do not change: the core, the API, the ADRs, the Add feeds page, `MakeFeedForm`.

### Architecture

These choices are fixed. Do not change them.

1. `welcome.js` has no Preact and no browser state. It gives:
    - `showWelcome(feeds)`: `true` when the feed list is empty.
    - `WELCOME`: the texts of ADR 0119 items 2 and 3, and the rows `music`, `label` and `online`, each with `icon`, `title` and `line`.
    - `welcomeSteps(path)`: the three step names of ADR 0119 item 4, for `music` or `label`.
    - `hostStep(connectors)`: `'pick'` with no connector, `'skip'` with one, `'menu'` with two or more (ADR 0119 item 6).
    - `HOST_ROWS(serverMode)`: the rows "Nostr (Blossom)" ("Store the files on Blossom servers, signed with your Nostr key."), "S3 or Cloudflare R2" ("Your own storage bucket.") and "My own server" ("SFTP, or a folder on this computer." on the desktop, and "An SFTP server." on a server). Each row gives the host type of `AddHostForm`.
    - `welcomeBody(path, name, connectorId)`: the body of `POST /feeds/new`: `medium: 'publisher'`, `role: 'artist'` or `'label'`, `title` and `author` equal to the name, `path` from `feedFileStart(name)`, and `connector_id`.
    - `nextStep(feed)`: `'album'` for a publisher feed with the role `artist` and no release, `'label'` for the role `label` with no release, else `null`. It reads the roles and the `child_ids` that the core gives.
2. The Releases page shows the Welcome page in the place of "No releases yet." when `showWelcome(feeds)` is true. The Welcome page is a page of rows, not a form. The large row is a `button.link-row` with an icon of 44 px, a title and one line.
3. "I make music" and "I run a label" open one sheet. It shows the step line, then step 1 or step 2. Step 1 has the field of the name and the hint of ADR 0119 item 5. The picture of item 5 is not in this packet: the Artwork sheet of the new page gives it later. "Continue" goes to step 2.
4. Step 2 follows `hostStep`. With `'pick'`, it shows `HOST_ROWS`, and a row shows the form of that type from `AddHostForm` in the same sheet. When the form adds the file host, the sheet sends `welcomeBody`. With `'skip'`, "Continue" of step 1 sends `welcomeBody` with the one connector. With `'menu'`, step 2 is a select menu of the connectors and "Make the Page".
5. After `POST /feeds/new`, the sheet closes and the page of the new feed opens, as `MakeFeedForm` does. Studio writes nothing (ADR 0007).
6. `PublisherPage` shows the large row of ADR 0119 item 8 above its sections when `nextStep(feed)` is not `null`. "Add an Album" opens the Add feeds sheet of an album with the artist as its start value. "Add an artist" opens it for a publisher feed with the role `artist`. "Add a release" opens it for an album. "Find an artist that is online" opens "By address or name".
7. "My music is online already" opens the form "By address or name" of the Add feeds page. "Import an Account" opens `ImportFlow` when `canImport` is true. Else the line of ADR 0119 item 3 does not show.
8. The texts of `welcome.js` use no word "feed". Each button label is in title case.
9. `docs/user-guide/first-steps.md` tells the two paths, the file host step and the first Review.

### Acceptance Criteria

Mechanical:

- A Node test: `showWelcome` for an empty list and for a list with one feed.
- A Node test: `welcomeSteps` for the two paths, and `hostStep` for zero, one and two connectors.
- A Node test: `welcomeBody` gives the role of the path, the name as title and author, and the path of `feedFileStart`.
- A Node test: `nextStep` for an artist with no release, a label with no release, and an artist with a release.
- A Node test: no text of `WELCOME`, `HOST_ROWS` or the steps has the word "feed".
- The test of title case, the test of the page rule and the tier test still pass.
- The four commit checks pass.

Visual: the visual check of this task.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0119-a-new-user-starts-as-a-musician-or-a-label.md
- docs/tasks/studio-v1-task-083-welcome.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 083a names.

Goal:
- A user with no feed sees the Welcome page, and starts an artist page or a label page in three steps.

Constraints:
- Follow the Architecture section of packet 083a fully.
- No feed rule in the UI (ADR 0065).
- Each button label is in title case. Each other text is in sentence case.
- A confirmation is a sheet of Studio, never `confirm()`.
- Each `setState` that combines the old value uses a functional update.
- Each comment and the user guide page are in ASD-STE100 Simplified Technical English.
- The UI has no build step. To check syntax, copy a changed file to a `.mjs` file in `.studio-dev/` and run `tools/node --check` on the copy.

Do not touch:
- The core, the API, the ADRs, the Add feeds page, `MakeFeedForm`.
- Do not commit. Do not do the visual check.

Acceptance criteria:
- The mechanical criteria of packet 083a.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

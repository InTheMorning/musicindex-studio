# Studio v1 Task 144: A Row Has One Menu, And A Live Show Can Be Removed

Decision: [ADR 0210](../adr/0210-a-row-has-one-menu-and-a-live-show-can-be-removed.md).

Status: Complete on 2026-11-14, with its screenshot visual check. The human gate of ADR 0210 is open.

## Result

- ChatGPT did the build. The main session read the diff, ran the five checks and took the screenshots.
- `MenuControl` is now in `crates/api/ui/menu.js`, because `peoplepages.js` and `app.js` cannot import each other. It gets the icon component as a prop. `crates/api/src/ui.rs` serves the new file.
- With `row`, the button holds the ellipsis icon and the text "Actions ⌄". The styles show the icon at 760 px or less, and the text above that width.
- The main session made these corrections. The delegate removed the comment of `ReleasesPage`, and it is back at its function. The wide button had no chevron. On the phone, the menu button was on its own line below the address. At 1280 px the group clipped the open menu.
- **Copy** writes the address to the clipboard and shows no "copied" text.
- Walk: `setup-144.py` with `steps-144-people.json`, and `setup-none.py` with `steps-144-live.json`. The wallet rows at 1280 px light and 390 px dark showed the two forms and the open menu. **Make Default** from the menu moved the tag. The live show walk showed the button, the confirm, and the tab **Live Shows** with no show after the removal. Each view gave no edge, overflow or fit finding with the page at its top.
- The walk binary shows no collection rows, because only the desktop app has collections. A temporary check binary, `.studio-dev/walk/examples/collections_walk.rs`, starts the desktop shell with collections. With it, `steps-144-collections.json` showed the row of a collection with its open menu. At 1280 px light the button is "Actions ⌄". At 390 px dark it is the ellipsis button.
- That walk found that the library folder path of the tab **Data** broke inside a word at 390 px. The path now has the mark of musicindex ADR 0065 R-FIT-3 item 3.
- The walk `steps-144-live-review.json` with `setup-144-live.py` removed a show that the feed file has. Review showed "Live item", "Friday Live" and "Removed".

## Goal

1. Each address row of a wallet has one menu with **Make Default**, **Edit** and **Copy** (ADR 0210 items 1, 2, 7 and 8).
2. The menu button of a row is an ellipsis icon button in the narrow layout and the text button **Actions** in the wide layout. This applies to the wallet rows and the collection rows (items 3 to 6).
3. The page of a live show has **Remove the Show** with a confirm (items 9 to 14).

## Code Locations

These come from a read on 2026-11-14. Make sure that each one is correct before a change.

- `crates/api/ui/app.js`, `MenuControl` (search `function MenuControl`): the menu button with `buttonLabel`, `accessibleName`, `menuLabel`, `items` and `onSelect`. The collection rows use it (about line 5005). `SortMenu` uses the same markup.
- `crates/api/ui/app.js` (about line 1879): the icon set with `alert`. It has no `ellipsis`.
- `crates/api/ui/peoplepages.js`, `AddressRow` (about line 409): the label with `tag`, the address, the count, `mark`, the copy button and `action`. The section **Payment** uses it (about line 1060) with **Make Default** and **Edit**. A suggestion uses it also (about line 1071).
- `crates/api/ui/people.js`: the pure functions of the page of a person.
- `crates/api/ui/editor.js`, `LivePage` (search `function LivePage`): the page of one live show. `removeTrack` of `TrackPage` (about line 3288) shows the pattern of a removal from the model.
- `crates/api/ui/live.js`: the pure functions of a live show, with `isLiveItem` and `viewAfterLiveGone`.
- `crates/api/ui/confirms.js`: each confirm, with `destructive(title, text, button)`.
- `crates/api/ui/styles.css`: `.sort`, `.sort-menu`, `.address-row`, `.collection-row`, and the container rules of the narrow layout.
- Tests: `crates/api/ui-tests/people.test.mjs`, `crates/api/ui-tests/live.test.mjs`, `crates/api/ui-tests/confirms.test.mjs`, `crates/api/ui-tests/collectionlist.test.mjs`, `crates/api/ui-tests/words.test.mjs`.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Menu control | extend | crates/api/ui/app.js | The button gets the icon form and the text form. The menu does not change. |
| Icon set | extend | crates/api/ui/app.js | The set gets the Lucide icon `ellipsis`. |
| Address row | extend | crates/api/ui/peoplepages.js | The row gets one menu in the place of its buttons, and one status line. |
| Address menu items | extend | crates/api/ui/people.js | A pure function gives the items of the menu. |
| Collection rows | reuse | crates/api/ui/app.js | The rows use the changed menu control. |
| Confirms | extend | crates/api/ui/confirms.js | The confirm of the removal of a live show. |
| Live show model | extend | crates/api/ui/live.js | A pure function removes one live item from the model. |
| Live show page | extend | crates/api/ui/editor.js | The page gets the button **Remove the Show**. |

## Rules For The Delegate

- Read ADR 0210 first, then ADR 0183 and ADR 0205 items 38 to 40.
- Write comments in Simplified Technical English. The skill is `/home/citizen/.agents/skills/asd-ste100/SKILL.md`.
- Do not commit. Leave the change in the working tree and report it. The main session reviews the diff, takes the screenshots and commits.
- Do not change `AGENTS.md`, an ADR or a task file.
- Do not change a route, the core or the schema. If the removal of a live item needs a change of the core, stop and report.
- Change a shared component only as this task says. Do not remove a style that a different page uses. Search each class before you remove its rule.
- If a step needs an architecture decision that ADR 0210 does not give, stop and report.
- `node` is not on the PATH. Use `tools/node`. `node --check` does not find a template error in `crates/api/ui/*.js`. Check a copy with the extension `.mjs`.
- Use the exact words of ADR 0210 for each text. Use tokens only in the styles.

## Changes

1. `crates/api/ui/app.js`:
    - Add the Lucide icon `ellipsis` to the icon set: three circles with `r="1"` at `cx` 5, 12 and 19 and `cy` 12.
    - `MenuControl` gets a prop `row`. With `row`, the button holds the icon and the text **Actions**, each in its own span, and the class `row-menu`. The accessible name stays `accessibleName`. Without `row`, the control does not change.
    - Export `MenuControl`, so that `peoplepages.js` can use it. If an import cycle occurs, move `MenuControl` to a module that the two files can import, and report it.
    - The collection rows pass `row`.
2. `crates/api/ui/styles.css`:
    - In the wide layout, `.row-menu` shows the text and hides the icon. In the narrow layout, it shows the icon and hides the text, and it is a round icon button with the target of its profile.
    - Use the same narrow condition as the other rules of the page of a person and of Settings. Do not add a new width value.
    - The open menu must stay in the window at 390 px.
3. `crates/api/ui/people.js`: add `export function addressMenu(isDefault, hasAddress)`. It gives the items in the sequence `make-default`, `edit`, `copy`, each with `action` and `label`. The default address has no `make-default`. An entry with no address has no `copy`.
4. `crates/api/ui/peoplepages.js`:
    - `AddressRow` of a wallet shows the label, the address, the account and one status line with the tag **Default** and the count text. It has one `MenuControl` with `row`, with the accessible name `Actions for {address}`.
    - **Make Default** and **Edit** do what the buttons of today do. **Copy** writes the address to the clipboard, as `CopyButton` does.
    - A suggestion row keeps its current controls.
    - **Remove Wallet** does not change.
5. `crates/api/ui/confirms.js`: add `removeLiveShow(title, live)`. The title is `Remove “{title}”?`, the button is **Remove the Show**, and the text is the text of ADR 0210 item 11. A show with no title uses "Untitled show".
6. `crates/api/ui/live.js`: add `export function withoutLive(model, key)`. It gives the model with no live item of that key, by `isLiveItem`.
7. `crates/api/ui/editor.js`, `LivePage`: the last section of the page has the destructive button **Remove the Show**. After the confirm, it updates the model with `withoutLive` and goes to the tab **Live Shows** of the feed. `LivePage` needs `go` for this.
8. Update each test that reads a changed text or function.

## Acceptance Criteria

Mechanical:

1. A Node test of `addressMenu`: three items for an address that is not the default, two items for the default address, and no `copy` with no address. Each failure message starts with "ADR 0210" and gives the fix.
2. A Node scan test: `peoplepages.js` and the collection rows use `MenuControl` with `row`, and no button label in the UI files is only the character "…".
3. A Node test of `removeLiveShow`: the title, the button and the two texts, for a show that is pending and for a show that is live.
4. A Node test of `withoutLive`: the show of the key is gone, and each other show stays. A key that no show has gives the same list.
5. A Node scan test of `editor.js`: `LivePage` has **Remove the Show** and calls `CONFIRMS.removeLiveShow`.
6. The four checks pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.

Visual, by the main session:

1. A wallet with two addresses at 1280 px light and 390 px dark, with the open menu.
2. The collection rows of Settings at 1280 px light and 390 px dark.
3. The page of a live show with **Remove the Show**, its confirm, the tab **Live Shows** after the removal, and Review.

Human, by the operator: the gate of ADR 0210.

## Report

- The changed lines, with file and line.
- The test names, and the output of the four checks.
- Each point where the code did not agree with this task.

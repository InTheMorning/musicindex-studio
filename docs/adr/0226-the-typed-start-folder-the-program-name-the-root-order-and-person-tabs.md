# ADR 0226: The Typed Start Folder, The Program Name, The Root Order And Person Tabs

- Status: Accepted
- Date: 2026-11-19
- Owner: operator
- Accepted: 2026-11-19 by the operator, in the human check of rc.27 (`docs/reviews/2026-11-19-rc27-human-check.md`, findings 1, 6, 12 and 13). The operator selected each item from a drawing or gave the name.
- Technical choices: the main session selected items 4 to 6, 10 and 11, and the section order in each tab (item 9). The operator did not get them as a question.
- Bar: opposite choice by mistake. Items 1 to 3 and items 7 and 8 change rules of accepted records. Item 5 changes a name that packages, desktop files and window managers use, so it is difficult to change later.
- Changes: [ADR 0223](0223-a-split-in-shares-or-percent-a-release-search-and-one-tap-on-a-phone.md) item 10. The folder dialog starts at the typed folder (items 1 to 4).
- Changes: [ADR 0216](0216-the-linux-packages-of-the-desktop-app.md) item 1 and its consequence on the binary name. The installed program is `musicindex-studio-desktop` (items 5 and 6).
- Changes: [ADR 0223](0223-a-split-in-shares-or-percent-a-release-search-and-one-tap-on-a-phone.md) item 7. The fixed rows People, File hosts and Settings are at the bottom in both layouts (items 7 and 8).
- Uses: [ADR 0182](0182-the-page-of-a-person-starts-with-its-links-and-has-a-nickname.md) item 10 (the section sequence of the person page). The tabs divide that sequence and do not change it.
- Uses: [ADR 0166](0166-settings-has-tabs-and-one-save-bar.md) items 1 to 5, 9 and 10 (the tabs of Settings, the select menu in a narrow window, the tier order in each tab, the mark of a tab with a change that is not saved, and no leave question on a tab change), for the person tabs (items 9 to 12).
- Changed by: [ADR 0241](0241-sign-out-and-switch-collection-are-rows-of-the-panel-footer.md). Item 7: the root page of a narrow window ends with the collection rows, the lock row, then "Sign Out".

## Context

The human check of rc.27 gave four findings that change accepted rules:

- Finding 1: "Choose Folder…" in the first-run library sheet opens the dialog at the home folder, also when the field has a typed folder.
- Finding 6: the Arch package installs the program `studio-desktop-app`. That name does not tell a user which suite app it is.
- Finding 12: on the root page, People, File hosts and Settings are at the top of the narrow layout and at the bottom of the wide left panel. The two layouts do not agree.
- Finding 13: the person page is a long page with no tabs. The release pages and Settings have tabs.

## Decision

### The start folder of "Choose Folder…"

1. When the field has an absolute path of a folder that exists, the dialog starts at that folder. Items 1 to 3 apply to each field with "Choose Folder…": the first-run library sheet and the folder sheet of the Files page (ADR 0180 item 8).
2. When the typed folder does not exist, the dialog starts at its nearest parent folder that exists.
3. When the field is empty, the path is not absolute, or no parent folder exists, the dialog starts as before: at the library folder when it is set, else at the home folder of the user.
4. The UI sends the text of the field to `POST /api/v1/choose-folder` as `start`. The route selects the start folder. The UI does not examine the file system.

### The program name

5. The Cargo binary of the package `studio-desktop-app` is `musicindex-studio-desktop`. Each package installs this program: the Arch package, the `.deb`, the `.rpm` and the AppImage. The Cargo package name `studio-desktop-app` does not change, so `cargo … -p studio-desktop-app` stays correct.
6. The Arch desktop file has `Exec=musicindex-studio-desktop` and `StartupWMClass=musicindex-studio-desktop`. The window class follows the program name, so the two lines change together.

### The root page order

7. On the root page of a narrow window, Releases comes first. Then come the feed groups. Then come People, File hosts and Settings. The collection mark and "Lock Studio" or "Unlock Studio" stay at the end.
8. The wide left panel has the same order as item 7.

### The person tabs

9. The person page has four tabs, in this order. In each tab, the sections keep the sequence of ADR 0182 item 10:
   - **Profile**: "Linked artist and label pages" and the row of the check.
   - **Feeds**: "Credits", "Where this person is" and "More from your feeds".
   - **Payment**: "Payment" and "Support links".
   - **Settings**: "Change your feeds", "Delete Person" and "Merge with a Person".
10. The head of the person page stays above the tabs: the picture, the name, the role line, the Nostr keys and "Find on Music Index". The merge notice with **Undo** and the error notice also stay above the tabs.
11. The section list of `pageSequence('person')` names the tab of each section, as the sections of `studio-settings` do. One function gives the tabs of a page from its sections. Settings and the person page use that function.
12. The open tab is part of the view, as in Settings (ADR 0166 items 3 and 4). The page opens on Profile. A tab with a change that is not saved shows the mark of ADR 0166 item 9. **Save** and **Discard** stay in the bar and apply to all tabs. A change of the tab keeps the draft and does not ask the leave question (ADR 0166 item 10).

## Alternatives Considered

- Start folder: the home folder in each case (ADR 0223 item 10). The operator rejected it, because a typed folder is the folder that the person wants to see.
- Program name: `musicindex-studio`. It is the name of the server program family and of the data folder, so a user can confuse the desktop program with the server.
- Program name: a rename of the Cargo package. Many documents, tests and scripts use `-p studio-desktop-app`. A rename of the binary only gives the same installed name with fewer changes.
- Root order: the fixed rows at the top in both layouts. The operator selected the bottom, because the releases are the main content.
- Person page: three tabs, with Payment in Profile. The operator selected four tabs.

## Consequences

- An installed `.deb` or Arch package of rc.27 or earlier has `/usr/bin/studio-desktop-app`. The upgrade removes it and installs `/usr/bin/musicindex-studio-desktop`. A user command or script with the old name stops.
- The data folder and the identifier of the app do not change. The program name is not part of them.
- A person page shows less content at one time. A section that a test or a walk opens by its heading must open its tab first.

## How This Decision Is Checked

Mechanical:

- API test: `/choose-folder` with a typed folder that exists, with a typed folder that does not exist, and with an empty field, with and without a library folder (items 1 to 4).
- Rust test in `crates/desktop-app`: `Cargo.toml` has the binary `musicindex-studio-desktop`, and the Arch desktop file has the same name in `Exec` and `StartupWMClass` (items 5 and 6). The mismatch of `Exec` and `StartupWMClass` is the incident: finding 6 would break the window match with no error.
- UI test: the root page model gives Releases, the feed groups, then People, File hosts and Settings, then the collection mark and the lock row (items 7 and 8).
- UI test: the person tabs and their sections in the sequence of item 9, and the Settings tabs from the same function (items 9 and 11).

Visual, by screenshots of the main session before each commit, at 1280 px and 390 px:

- The root page in both layouts (items 7 and 8).
- Each person tab, with the head above the tabs (items 9 and 10), and a tab with a change that is not saved (item 12).

Human:

- The operator selects "Choose Folder…" with a typed folder in the first-run sheet of the desktop app (items 1 to 3). A screenshot cannot open the dialog of the system.
- The operator installs the Arch package and starts Music Index Studio from the application menu. The window opens, and the window list shows the app icon (items 5 and 6).

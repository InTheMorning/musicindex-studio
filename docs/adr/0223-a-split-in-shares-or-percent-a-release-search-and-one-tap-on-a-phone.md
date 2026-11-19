# ADR 0223: A Split In Shares Or Percent, A Release Search, And One Tap On A Phone

- Status: Accepted
- Date: 2026-11-19
- Owner: operator
- Accepted: 2026-11-19 by the operator, through the questionnaire, from drawings. The operator selected "Shares or % menu", "Search field", "At the top" and "Open the page, Sections button".
- Technical choices: the main session selected items 4, 6, 9 and 10 to 12, and the texts of the task files. Items 10 to 12 are the recommended fixes of O10 and O12, which the operator did not get as a question.
- Bar: opposite choice by mistake. Items 1 to 4 change the recipient form of ADR 0144, and items 7 and 8 change the narrow stack of ADR 0206.
- Changes: [ADR 0144](0144-recipients-use-one-picker-and-sort-by-split.md) item 1. The split has the label "Split" and a mode menu (items 1 to 4).
- Changes: [ADR 0206](0206-the-shell-and-the-editor-pages.md) items 13 and 14. The fixed destinations come first on the root page, and a release opens in one tap (items 7 and 8).
- Changes: [ADR 0213](0213-a-release-starts-on-the-page-of-its-artist.md) item 11. A release with no track and no live item has the kind "Release" (item 11).
- Changes: [ADR 0180](0180-the-files-page-has-a-folder-picker-and-one-step-for-each-copy.md) item 8. The folder dialog also opens from the first-run library sheet (item 10).
- Uses: musicindex ADR 0075 R-VALUE-5 for the percentage of a recipient. [ADR 0091](0091-a-change-set-edits-one-recipient-in-many-feeds.md) item 13 and the core function `scale` for the scale of the other splits. [ADR 0222](0222-errors-give-a-step-terms-get-a-hint-and-the-first-run-has-two-stages.md) item 7 for the split hint. [ADR 0036](0036-library-records-host-inventory-and-host-operations.md) item 1 for the library folder.
- Source: [observations of the user journeys](../research/2026-11-18-user-journeys/observations.md) O10, O12, O13, O14, O15 and O16.
- Changed by: [ADR 0226](0226-the-typed-start-folder-the-program-name-the-root-order-and-person-tabs.md) items 1 to 4, 7 and 8, on 2026-11-19. Item 7: People, File hosts and Settings are at the bottom in both layouts. Item 10: the folder dialog starts at the typed folder.

## Context

- **O16.** After "Add the Recipient", the field "Split (shares)" is empty and the bar shows 100.0 %. musicindex R-VALUE-5 item 5 gives 100 % to one recipient for each share value, so 100 % is correct for one recipient. But the empty field gets no message. On 2026-11-15 the operator set the rule that each split input has two modes, shares and percent, and always shows the other value. The recipient form has only shares (`values.js:194`).
- **O13.** Releases has sort, view and add controls, and no search (`app.js:3480`). ADR 0206 records the search of the collection as open (O-9).
- **O14.** On the root page of a phone, People, File hosts and Settings are below each feed group (`app.js:3387`). A user with many feeds must scroll to the end.
- **O15.** On a phone, a tap on a release opens the outline page, and a second tap opens the release page (`shell.js:840`).
- **O10.** The first-run library sheet has a text field only (`app.js:1804`). The desktop app has a folder dialog for the Files page (ADR 0180 item 8), but its route needs a library folder that is set.
- **O12.** A new release with the kind "From the tracks" shows "ALBUM" with no track, and "SINGLE" after the first track. The rule of ADR 0213 item 11 gives an album for each count that is not 1 to 6, 0 included.

## Decision

### The Split

1. An open recipient has the field "Split" and a select menu with "Shares" and "Percent". The menu starts at "Shares". The choice is a state of the open row, and Studio does not keep it. The feed always holds shares.
2. Below the field, one line gives the other value:
    - With "Shares": "{percent} of this block", for example "25.0 % of this block".
    - With "Percent": "{shares} shares of {total}", for example "25 shares of 100".
3. With "Percent", the user types a whole number from 1 to 99. When the field loses the focus or the user presses Enter, Studio gives this recipient that number of shares. Then it scales the shares of each other recipient of the block, each fee included, so that their total is 100 minus that number. The scale uses the rule of the core function `scale`. The total of the block becomes 100, and the recipient gets the typed percentage.
4. An empty split shows the line "Give a split." in the place of the line of item 2, and the bar shows "—" for that recipient. With one recipient and a split, "Percent" shows 100, the field is read-only, and the line says "One recipient gets each payment.".

### Releases

5. Releases has a search field above the list, with the placeholder "Search releases". It shows on Releases and on each feed group. A row shows when its title, or the title of one of its artist or label pages, has the typed text. The search ignores letter case.
6. When no row matches, the list shows "No release has “{text}”.". The search does not change the sort, the view or the address of the page. The letter bar shows only for an empty search.

### The Narrow Layout

7. On the root page of a narrow window, the fixed destinations come first: Releases, People, File hosts and Settings. Then come the feed groups. The collection mark and "Lock Studio" or "Unlock Studio" stay at the end. The wide left panel does not change.
8. On a narrow window, a tap on a release or on a feed opens its feed page. The bar of the feed page has the button "Sections", which opens the outline page. The stack is Releases, the feed page, the outline page and a part. Back goes one level down in that stack.

### The Library Folder

9. The search field of Releases and of People is one shared component.
10. In the desktop app, the first-run library sheet has "Choose Folder…" adjacent to the field. It opens the folder dialog of the system. The selected folder fills the field. With no library folder set, the dialog starts at the home folder of the user. The field stays, and a different shell shows only the field.

### The Kind Of A New Release

11. A release with no kept kind, no track and no live item has the kind "release" and the label "Release". Each other count keeps the rule of ADR 0213 item 11.
12. The option "From the tracks" of the menu "Kind" adds the kind that it gives, for example "From the tracks (Single)". With no track, it is "From the tracks (no tracks yet)".

## Alternatives Considered

- Shares only, with a dash for an empty split. Not selected by the operator. It does not apply the rule of 2026-11-15.
- A search field and a status filter. Not selected by the operator. The toolbar gets more controls.
- No search. Not selected by the operator.
- A tab bar at the bottom of a phone. Not selected by the operator. It changes the narrow layout of each page.
- The outline page first, with no change. Not selected by the operator.

## Consequences

- Percent mode changes the shares of the other recipients of the block. Review shows each changed split.
- A phone user reaches the release page in one tap and the outline in two.
- A new release shows "RELEASE" until it has a track.

## How This Decision Is Checked

Mechanical:

- UI tests: the line of the other value in each mode, and the line of an empty split (items 2 and 4).
- UI tests: the percent function gives the typed percentage to the recipient and a total of 100 (item 3). Its results agree with the core function `scale` for the cases of the task file.
- UI tests: the release filter by title and by the title of an artist page, the letter case, and the empty result (items 5 and 6). People and Releases use the shared search component (item 9).
- UI tests: the sequence of the root page model (item 7). `openFeedView` gives the feed page at a narrow width, and the outline view keeps its back entry (item 8).
- Core test: `release_kind_shown` gives "release" for no track and no live item (item 11). UI test: the option text of "From the tracks" (item 12).
- API test: `/choose-folder` with no library folder set calls the picker with the home folder (item 10).
- `bash tools/gate.sh` and `cargo test -p studio-api` pass.

Visual, by screenshots of the main session before each commit, at 1280 px and 390 px:

- An open recipient in each mode, and an empty split.
- Releases with a search that has rows and with a search that has no row.
- The root page and a release page with "Sections" at 390 px.
- A new release with no track.

Human: the operator selects a folder with "Choose Folder…" in the first-run sheet of the desktop app (item 10). A screenshot cannot open the dialog of the system.

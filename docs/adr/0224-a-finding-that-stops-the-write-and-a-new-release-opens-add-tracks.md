# ADR 0224: A Finding That Stops The Write, And A New Release Opens Add Tracks

- Status: Accepted
- Date: 2026-11-19
- Owner: operator
- Accepted: 2026-11-19 by the operator, through the questionnaire, from drawings. For O3, the operator selected "Mark + checklist row". For O11, the operator selected "Add tracks sheet".
- Technical choices: the main session selected items 2, 3, 6 and 8, and the texts of the task file.
- Bar: opposite choice by mistake. Item 1 adds a level to the findings of ADR 0174, and item 7 changes the page that opens after **Make the Release** (ADR 0164).
- Changes: [ADR 0174](0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md) item 3. The levels get **Stop** (items 1 to 3).
- Changes: [ADR 0219](0219-a-release-checklist-and-a-check-of-the-address-type.md) items 6 and 7. The checklist gets the row "Artist page" (items 4 to 6).
- Changes: [ADR 0164](0164-the-releases-of-a-publisher-feed-have-an-add-row.md) item 3. A new release with no track opens with the sheet "Add tracks" (items 7 and 8).
- Uses: [ADR 0213](0213-a-release-starts-on-the-page-of-its-artist.md) items 18 and 19, and [ADR 0218](0218-a-page-link-in-review-and-a-suggestion-starts-with-no-person.md) items 1 to 9, for the stop in Review and **Link a Page**. [ADR 0223](0223-a-split-in-shares-or-percent-a-release-search-and-one-tap-on-a-phone.md) item 11 for the kind "Release".
- Source: [observations of the user journeys](../research/2026-11-18-user-journeys/observations.md) O3 and O11.

## Context

- **O3.** Review stops the write of a release with no artist page (ADR 0213 item 18). The Check tab shows the same condition with the level "Notice" (`feed_check.rs:236`). A notice tells the user nothing about the stop. The release checklist has four rows, and none of them is for the artist page (`feedcheck.js:123`).
- **O11.** After **Make the Release**, the release page opens (ADR 0164 item 3). The user must then find "Add tracks" in the outline. Each new release needs tracks first. The checklist row "Tracks" has **Add Tracks** (ADR 0219), but the user must find that row first.
- ADR 0218 decides observation O4: the write of the artist page stays separate from the write of the release, and the result panel tells it.

## Decision

### The Level Stop

1. The findings get a fourth level, **Stop**. A stop finding is a condition that makes Review refuse the write. The core gives the level `stop` to `no_artist_page` and to `no_publisher_page`. Each other finding keeps its level.
2. The Check tab shows a stop finding first, before each warning. Its mark is "Stops the write", with the error color of the theme.
3. The text and the repair of a stop finding do not change. **Link a Page** stays its button (ADR 0213 item 19). The stop in Review does not change (ADR 0218).

### The Checklist

4. The checklist of a release has a fifth row, "Artist page", after "Payment recipient". The row has a cross and **Link a Page** when the draft has the finding `no_artist_page`. Else it has a check mark.
5. The checklist shows when the draft has at least one finding of `no_tracks`, `no_cover`, `no_audio_file`, `no_recipient` or `no_artist_page`.
6. **Link a Page** in the checklist does the same action as **Link a Page** on the Check tab. The checklist and the Check tab use one function (ADR 0219 item 3).

### A New Release

7. **Make the Release** can make a release with no track and no live item. Then the page of the release opens, and the sheet "Add tracks" opens on it. A release made from a different feed with tracks opens with no sheet. A playlist opens with no sheet.
8. A close of the sheet keeps the release page open, with its checklist. The sheet does not open again by itself.

## Alternatives Considered

- O3: the mark "Stops the write" only, with four checklist rows. Not selected by the operator. The user sees the stop only on the Check tab.
- O3: no change. Not selected by the operator. The level "Notice" tells nothing about the stop.
- O11: the release page as before. Not selected by the operator. Each new release needs one more click.
- O11: a large drop area on the release page in the place of the empty track list. Not selected by the operator.

## Consequences

- A new artist sees the missing artist page on the release page, before Review.
- A new release starts at the step that adds tracks.
- A new finding that stops the write must get the level `stop`.

## How This Decision Is Checked

Mechanical:

- Core test: `no_artist_page` and `no_publisher_page` have the level `stop`. Each other finding keeps its level (item 1).
- UI tests: the Check rows put a stop finding first and give the mark "Stops the write" (item 2). The checklist gives five rows, and the row "Artist page" has a cross with `no_artist_page` (items 4 and 5). The checklist shows with only `no_artist_page` (item 5).
- UI test: the rule of item 7 opens the sheet for a new release with no track, and not for a playlist or a release with tracks.
- `bash tools/gate.sh` passes.

Visual, by screenshots of the main session before each commit, at 1280 px and 390 px:

- The Check tab with the finding "No artist page".
- A release page with the checklist row "Artist page" and a cross.
- The sheet "Add tracks" over a new release, and the release page after a close.

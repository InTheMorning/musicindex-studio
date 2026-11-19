# ADR 0221: The Welcome Page And The First Steps

- Status: Accepted
- Date: 2026-11-19
- Owner: operator
- Accepted: 2026-11-19. The global decision on ADR overuse (musicindex-research `df7b823`) sets one current ADR for an area after three changes. This ADR makes no new decision. It restates each live rule of ADR 0119, with each change from a subsequent ADR.
- Supersedes: [ADR 0119](../archive/adr/0119-a-new-user-starts-as-a-musician-or-a-label.md), A New User Starts As A Musician Or A Label.
- Changed by: [ADR 0222](0222-errors-give-a-step-terms-get-a-hint-and-the-first-run-has-two-stages.md) item 8, on 2026-11-19. Item 4: the sheet has two stages.
- Uses: these current ADRs own a part of this area. This ADR names each one at its item. [ADR 0122](0122-a-new-user-starts-with-a-default-file-host.md) items 1 to 4 (the rows of step 2), [ADR 0164](0164-the-releases-of-a-publisher-feed-have-an-add-row.md) item 3 (the link of the new release to its artist), [ADR 0212](0212-the-add-pages.md) (the form "Add tracks" and the search), [ADR 0020](0020-first-run-asks-one-question.md) items 3 to 15 (the search of "My music is online already").
- Uses: [ADR 0007](0007-draft-diff-write-and-announce.md) for the first write. [ADR 0045](0045-phase-9-crates-test-servers-and-host-forms.md) for the file host forms. [ADR 0082](0082-a-publisher-feed-has-a-set-of-roles.md) and [ADR 0083](0083-hosting-is-a-first-class-role.md) for the roles. [ADR 0107](0107-a-user-can-export-and-import-an-account.md) for the import.

## Context

- ADR 0119 gave the Welcome page and the sheet of the first steps on 2026-11-02. ADR 0122 changed its item 6, and ADR 0164 changed its item 9. The operator changed the button of its items 8 and 9 on 2026-11-14.
- ADR 0119 had three "Changed by" lines. The decision of 2026-11-19 on errors, terms and the first run (ADR 0222) changes its item 4. This ADR is the current ADR for the area before that change.
- ADR 0119 is in `docs/archive/adr/` for research. It keeps its context, its alternatives and its checks. The item map at the end of this ADR gives the new item of each superseded item.

## Decision

1. A user with no feed in the collection sees the page "Welcome to Studio" in the place of the empty Releases page. A user with one feed or more sees Releases.
2. The Welcome page has the text "Studio publishes your music so that every music app can play it. What do you want to start with?" It has two groups of large rows, each with an icon, a title, one line and `›`:
    - "I make music": "Start with your artist page. Your albums go on it." It is the primary row.
    - "I run a label": "Start with your label page. Your artists and releases go on it."
    - In a second group, "My music is online already": "Find your artist or albums, and bring them into Studio." It opens the form "By address or name" of ADR 0212.
3. Below the groups, the line "Moving from a different server? Import an Account" opens the import of ADR 0107, where the shell permits it.
4. "I make music" and "I run a label" open a sheet with three steps, and a step line at its top:
    - Musician: "Your name", "Where it lives", "Your first album".
    - Label: "Label name", "Where it lives", "Your first artist".
5. Step 1 asks for the name ("Artist name" or "Label name") and an optional picture. The musician sheet has the hint "A band is one artist. Add the members later, in People."
6. Step 2, "Where it lives", has the text "Studio does not keep your music. It puts your {artist page or label page} and your songs on a file host that you choose. You can move them later." Step 2 always shows, also with a connected file host. ADR 0122 items 1 to 4 give its rows, the row that is selected at the start, and the select menu of the connected file hosts. A Blossom host holds media only (ADR 0048 item 2), so it is not a row. A row opens the form of that file host (ADR 0045).
7. After step 2, the core makes the publisher feed as a draft, with the role `artist` or `label`. The sheet uses the route of the form that makes a feed. Studio writes nothing. The feed file gets its start value from the name (ADR 0212 item 10).
8. Then the page of the new feed opens. The toolbar shows the draft status and "Review", as on each editor page. The page has one large row at its top:
    - Musician: "Add your first album", with "Drop the songs in. Studio reads their names and order, and makes the album page." and the button "Add a Release".
    - Label: the rows "Add an artist", "Add a release" and "Find an artist that is online".
9. "Add a Release" opens "Add tracks" of ADR 0212 for a new album of this artist. Studio links the new album or playlist to the publisher feed, and the page of the new release opens (ADR 0164 item 3). The first write of the artist page and the album goes through Review (ADR 0007).
10. The first steps use the words "artist page", "label page", "album", "release" and "file host". They do not use the word "feed". The Add feeds page, the settings and the details keep the word "feed".
11. The Welcome page replaces the question of the first-run wizard, "Do you already have a feed?", and its page "Welcome" (ADR 0020 items 1 and 2). A new user goes from the library folder, or from the sign-up on a server, to the Welcome page. "My music is online already" uses the same search as the answer "Yes" of ADR 0020, and ADR 0020 items 3 to 15 apply to it. Below the groups, the line "Moving from MSP 2.0? Import the Backups" opens the MSP 2.0 import.

## Consequences

- The empty Releases page of a user with no feed is the Welcome page. The Add feeds page does not change for a user with feeds.
- The page of a new publisher feed has the row of item 8 while the feed has no release.

## Non-goals

- A write with no review.
- A new connector or a new file host form.

## How This Decision Is Checked

Mechanical. The tests of the superseded ADR enforce these items, and this ADR adds no test:

- `crates/api/ui-tests/welcome.test.mjs` examines these items:
    - The Welcome page for an empty feed list, and Releases for a list with one feed (item 1).
    - The steps of each path (item 4).
    - The model of step 2 with no connected file host and with two connected file hosts (item 6, ADR 0122 items 1 to 4).
    - The texts of items 2, 5, 6 and 8, with no word "feed" (item 10).
    - The role in the body of the form that makes the feed (item 7).
- `bash tools/gate.sh` passes.

Visual: the visual check of ADR 0119 is closed (task 083). This ADR changes no screen.

Human: the operator does these steps on the server:

1. Sign up as a new user and select "I make music".
2. Give a name and connect a file host.
3. Add an album, and write the two feeds through Review.

## Item Map

The map gives the item of this ADR for each item of the superseded ADR.

| Superseded item | New item |
| --- | --- |
| ADR 0119 items 1 to 5 | 1 to 5 |
| ADR 0119 item 6, with ADR 0122 items 1 to 4 | 6 |
| ADR 0119 items 7 and 8 | 7 and 8 |
| ADR 0119 item 9, with ADR 0164 item 3 | 9 |
| ADR 0119 items 10 and 11 | 10 and 11 |
| ADR 0119 "Acceptance Criteria" | "How This Decision Is Checked" |

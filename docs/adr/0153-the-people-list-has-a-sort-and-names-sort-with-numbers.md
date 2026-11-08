# ADR 0153: The People List Has A Sort, And Names Sort With Numbers

- Status: Accepted
- Date: 2026-11-07
- Owner: operator
- Accepted: 2026-11-07 by the operator, through the orchestrator session, with musicindex ADR 0053.
- Uses: musicindex ADR 0053 R-LIST-8 to R-LIST-12, musicindex ADR 0027 R-LIST-1 and musicindex ADR 0043 R-LIST-6 and R-LIST-7.
- Changed by: [ADR 0155](0155-the-sort-menu-names-both-directions.md). The sort menu names both directions, and the list has no reverse icon.

## Context

musicindex ADR 0053 gives the lists of the three apps that have a sort, a reverse icon and view icons. Before this record, only Releases and the page of an artist or a label had these controls in Studio. The People list was always A to Z. The payees with no person came in the order of their addresses. The library files of a feed came in the order of their paths as text, so "15.mp3" came before "2.mp3".

## Decision

1. The People list has the sorts "A to Z", the default, and "Newest", with the reverse icon (R-LIST-9). It has no view icons, because its rows have no art (R-LIST-10). "Newest" gives the person that the user added last first. The core gives the time when the user added each person.
2. The sort control of the People list is at the right end of the row of "Add a Person" and "Export People" (R-LIST-11 item 1). "A to Z" shows the letter bar of R-LIST-1 item 2 in the fold "People".
3. The People list keeps its sort and its reversed state in the navigation record of the session. The page of an artist or a label does the same (ADR 0085 item 8). The address does not keep them.
4. Each sort by name in the UI compares the names in the language of the reader. It reads a number in a name as a number (R-LIST-12 item 1). This applies to these lists:
   - Releases and the left panel.
   - The menus of people and feeds, and the album menu of a new album.
   - The file picker and the file lists of the Files page.
5. The payees with no person sort by name. A payee with no name comes after the names, by the address that its row shows (R-LIST-12 item 2). The same order applies in the sheet "Add a Wallet".
6. Each other list of Studio keeps its order and has no control (R-LIST-8):
   - The tracks and the releases that a position menu orders (ADR 0081).
   - The recipients (ADR 0144) and the credits.
   - The suggestions, and the results of each picker (ADR 0151 item 8).
   - The lists of the page of a person.
   - The file hosts, the collections, the change sets and the versions.
7. The page of an artist or a label keeps "Feed Order" as its default (ADR 0085 item 4). The user edits this sequence of releases in the publisher feed. The row "The releases of an artist, a label or a name" of R-LIST-9 has no "Feed Order". This difference stays open until a musicindex record decides it.

## How This Decision Is Checked

Mechanical:

- `crates/api/ui-tests/people.test.mjs`: the People model gives "A to Z" first and as the default, "Newest" by the time of addition, the reversed orders and the letters. The payees sort by name, with a payee with no name last.
- `crates/api/ui-tests/shell.test.mjs`: the name order reads numbers as numbers, and Releases "A to Z" puts "Volume 9" before "Volume 10".
- `studio-core` test `a_person_gives_the_time_when_the_user_added_it`.

Visual: the People list at 1280 px in "A to Z", "Newest" and "Oldest", and at 390 px with touch. The Files page shows "2.mp3" before "15.mp3".

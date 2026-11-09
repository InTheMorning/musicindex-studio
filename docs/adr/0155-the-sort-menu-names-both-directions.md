# ADR 0155: The Sort Menu Names Both Directions

- Status: Accepted
- Date: 2026-11-08
- Owner: operator
- Accepted: 2026-11-08 by the operator, through the orchestrator session, with musicindex ADR 0054.
- Uses: musicindex ADR 0054 R-LIST-13 and R-LIST-14, musicindex ADR 0043 R-LIST-6.
- Changes: [ADR 0153](0153-the-people-list-has-a-sort-and-names-sort-with-numbers.md) item 1, and the reverse icon of task 094 on Releases.

## Context

Releases, the page of an artist or a label, "Not linked", "Other feeds" and the People list had a sort menu and a reverse icon. musicindex ADR 0054 replaces the reverse icon with a menu that names both directions of each sort.

## Decision

1. The sort menu of each list of Studio lists each sort in its usual direction, then in its reversed direction, as words: "Newest", "Oldest", "A to Z", "Z to A", "Feed Order", "Feed Order, Reversed" (R-LIST-13 item 2).
2. No list of Studio has a reverse icon. The trigger names the current entry, and only that entry has the check mark.
3. Each list of Studio with a control has two or more sorts. Thus no list of Studio has the "Reverse" toggle of R-LIST-14.
4. The state of a list does not change: it is a sort and a direction. Releases keeps a reversed direction in the address as `reversed=1`. A feed group and the People list keep it in the navigation record (ADR 0085 item 8, ADR 0153 item 3). A letter bar of "Z to A" shows its letters from Z to A.
5. The view icons of Releases do not change.

## How This Decision Is Checked

Mechanical: `crates/api/ui-tests/shell.test.mjs` examines the entries of the menu and their sequence. The tests of ADR 0153 and task 094 examine the reversed order of the rows and of the letters.

Visual, by the agent: the menu of Releases and of the People list at 1280 px and at 390 px with touch. No list shows a reverse icon.

Visual result: passed on 2026-11-08 after a correction of the Releases menu at 390 px ([visual checks of 2026-11-08](../reviews/2026-11-08-visual-checks.md)).

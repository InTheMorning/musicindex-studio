# Task 049 Review Checklist, 2026-10-28

Task: [task 049](../tasks/studio-v1-task-049-add-pages.md). Decision: [ADR 0073](../adr/0073-the-add-pages-ask-one-question-at-a-time.md).

## Decisions Of The Planning Model

- Each row of the Add feeds page opens a sheet, not a page. The forms are short after the change, and a sheet needs no change to the navigation model.
- "Feed file" and "Properties" use the closed row of the track page ([ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md)). The UI then has one pattern for a value that Studio gives.
- The row "Artist or label" keeps the Role select menu. Type has four values, as the four rows.

## Packet 049a

- [x] `addfeeds.js` has no Preact and no browser state.
- [x] `feedFileStart` follows ADR 0073 item 8, and the core continues to check the path.
- [x] The request body of each type is the body of the current form of that type.
- [x] The three forms of the current page are removed.
- [x] The forms of "Add a feed you have" did not change.
- [x] No new UI function changes the model (ADR 0065). The start value of item 8 is an edit aid.

## Packet 049b

- [ ] The outline has one row `add-tracks`.
- [ ] The import queue of `ImportSheet` did not change.
- [ ] "Properties" opens only when the lookup did not find the two facts.
- [ ] The playlist "Add a track" did not change.

## Visual Check

- [ ] The visual criteria of task 049, with screenshots and `$edges()`.

## Result Of Packet 049a, 2026-10-28

- The packet passed. The coding model stopped at one test that did not pass: a new UI file must be in `FILES` of `crates/api/src/ui.rs`, and the packet did not name that file. The planning model added the entry.
- The planning model made these corrections:
  - `feedFileStart` writes the accent range as `\u0300-\u036f`, not as the raw characters.
  - The sheet of a make row has the title "Make a new feed", because the user can change Type.
  - The field in "Feed file" has the label "Path", so that the sheet does not show "Feed file" two times.
  - The two group titles use the caption of `page-section`, as the editor pages.
  - Each row shows its title above its text, and the text wraps. At 390 px the text was cut.
  - The first-run text and `first-run.md` name the row "Album".
- Visual check at 1100 px in the two themes and at 390 px: the page is 900 px high at 1100 px, before 2942 px. Type changes the fields. A title "Ça va — Live!" gives `ca-va-live.xml`. A submit to a host with no public address opens "Feed file" with the error. `$edges()` gave empty lists, and `$small()` gave an empty list at 390 px.

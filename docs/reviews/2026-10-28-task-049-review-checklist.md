# Task 049 Review Checklist, 2026-10-28

Task: [task 049](../tasks/studio-v1-task-049-add-pages.md). Decision: [ADR 0073](../adr/0073-the-add-pages-ask-one-question-at-a-time.md).

## Decisions Of The Planning Model

- Each row of the Add feeds page opens a sheet, not a page. The forms are short after the change, and a sheet needs no change to the navigation model.
- "Feed file" and "Properties" use the closed row of the track page ([ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md)). The UI then has one pattern for a value that Studio gives.
- The row "Artist or label" keeps the Role select menu. Type has four values, as the four rows.

## Packet 049a

- [ ] `addfeeds.js` has no Preact and no browser state.
- [ ] `feedFileStart` follows ADR 0073 item 8, and the core continues to check the path.
- [ ] The request body of each type is the body of the current form of that type.
- [ ] The three forms of the current page are removed.
- [ ] The forms of "Add a feed you have" did not change.
- [ ] No new UI function changes the model (ADR 0065). The start value of item 8 is an edit aid.

## Packet 049b

- [ ] The outline has one row `add-tracks`.
- [ ] The import queue of `ImportSheet` did not change.
- [ ] "Properties" opens only when the lookup did not find the two facts.
- [ ] The playlist "Add a track" did not change.

## Visual Check

- [ ] The visual criteria of task 049, with screenshots and `$edges()`.

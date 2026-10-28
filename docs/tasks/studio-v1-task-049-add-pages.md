# Studio v1 Task 049: The Add Pages

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The Editor Files And Covers".
Decisions: [ADR 0073](../adr/0073-the-add-pages-ask-one-question-at-a-time.md), [ADR 0028](../adr/0028-summary-lines-and-one-open-entry.md), [ADR 0029](../adr/0029-editor-pages-toolbar-and-three-levels.md), [ADR 0037](../adr/0037-button-roles.md), [ADR 0042](../adr/0042-ui-models-with-node-tests.md) and [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md).

Status: Open.

## Goal

The Add feeds page is a list of choices, and one form makes each new feed. The outline has one row "Add tracks", and the address sheet shows the size and the type only when Studio needs them.

## Packets

The two packets change different parts of the UI. They go in this sequence, because the two packets change `app.js`:

1. [049a: the Add feeds page and one form to make a feed](studio-v1-task-049a-add-feeds-page.md).
2. [049b: Add tracks](studio-v1-task-049b-add-tracks.md).

## Decisions Of The Planning Model

These decisions are in the limits of ADR 0073. The [review checklist](../reviews/2026-10-28-task-049-review-checklist.md) records them.

- A new model module `crates/api/ui/addfeeds.js` gives the rows of the Add feeds page, the fields of each type and the start value of "Feed file".
- The text of each row:

  | Row | Text |
  | --- | --- |
  | Album | A release with its tracks |
  | Artist or label | A list of your releases |
  | Show | A feed for live shows |
  | Playlist | Tracks from other feeds |
  | By address or name | Search musicindex, or give a feed address |
  | From a host folder | A feed file on a connected host |
  | From MSP 2.0 backups | The .json files that MSP 2.0 saved |

- Each sheet is a `dialog class="sheet small"` with `useSheet`, as the other sheets. The existing forms go in its body with their own submit buttons.
- "Feed file" and "Properties" use the closed `details class="disclosure"` row of the track page, with a `summary class="link-row"`.
- A show feed has the medium `music`, as the current show form has.

## Acceptance Criteria

Mechanical: the mechanical criteria of each packet, and the four commit checks pass.

Visual, for the planning model with `tools/drive-ui.mjs`. In the dark theme, in the light theme and at 390 px:

- The Add feeds page shows the two groups of rows, and no form. Each row opens its sheet.
- The form to make a feed shows Type, the fields of the type and the row "Feed file". A change of Type changes the fields.
- With one host, the form shows no Host field.
- "Add tracks" opens the file sheet. Its link opens the address sheet, and the link of that sheet opens the file sheet.
- After a found address, "Properties" is closed and shows the summary.
- `$edges()` finds no content at the edge of a group.

Human test, by the operator: on a phone, make a new album, and add a track from an address.

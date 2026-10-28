# Task 047 Review Checklist, 2026-10-28

Task: [task 047](../tasks/studio-v1-task-047-settings-page-and-tiers.md). Decision: [ADR 0071](../adr/0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md).

## Decisions Of The Planning Model

- The tier of each section is in the table of task 047. The publisher section `role` is `setup`, because the user sets the role one time. The track sections `credits` and `lyrics` are `details`.
- The editor page "Feed settings" keeps the key `settings` in `pages.js`. The Studio Settings page has the key `studio-settings`, and the view `{ name: 'settings' }`.
- The row "Settings" has no type dot, because it is not an entity.

## Packet 047a

- [ ] Each section has a tier of `TIERS`, and the sequence of the sections did not change.
- [ ] The fixed lists of ADR 0068 are deleted from `pages.test.mjs`.
- [ ] The tier test examines each page, with and without `server`.
- [ ] A bad list gives the message of ADR 0071 item 6.
- [ ] `editor.js` did not change.

## Packet 047b

- [ ] No settings panel is on the Hosts page.
- [ ] The notices of a new host stay above the host cards.
- [ ] The two UI texts name Settings.
- [ ] Each changed guide page passes the STE check.
- [ ] No new UI function changes the model (ADR 0065).

## Visual Check

- [ ] The criteria of task 047, with screenshots and `$edges()`.

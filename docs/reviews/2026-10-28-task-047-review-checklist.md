# Task 047 Review Checklist, 2026-10-28

Task: [task 047](../tasks/studio-v1-task-047-settings-page-and-tiers.md). Decision: [ADR 0071](../adr/0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md).

## Decisions Of The Planning Model

- The tier of each section is in the table of task 047. The publisher section `role` is `setup`, because the user sets the role one time. The track sections `credits` and `lyrics` are `details`.
- The editor page "Feed settings" keeps the key `settings` in `pages.js`. The Studio Settings page has the key `studio-settings`, and the view `{ name: 'settings' }`.
- The row "Settings" has no type dot, because it is not an entity.

## Packet 047a

- [x] Each section has a tier of `TIERS`, and the sequence of the sections did not change.
- [x] The fixed lists of ADR 0068 are deleted from `pages.test.mjs`.
- [x] The tier test examines each page, with and without `server`.
- [x] A bad list gives the message of ADR 0071 item 6.
- [x] `editor.js` did not change.

## Packet 047b

- [x] No settings panel is on the Hosts page.
- [x] The notices of a new host stay above the host cards.
- [x] The two UI texts name Settings.
- [x] Each changed guide page passes the STE check.
- [x] No new UI function changes the model (ADR 0065).

## Visual Check

- [x] The criteria of task 047, with screenshots and `$edges()`.

## Result Of The Review, 2026-10-28

- Packet 047a passed. The planning model moved one comment, wrapped the `bottom` list of `leftPanel`, and gave the root page test its new title. The test of the field Language names its owner, ADR 0068 item 4. The step of packet 047a that named ADR 0071 for that message was incorrect.
- `pages.js` writes the file name of its messages as two strings. A Rust test reads the text "/ui/" as a served address.
- Packet 047b passed. The planning model gave `hosts.md` a list of the panels of the Settings page, with the Library limits.
- The visual check found an empty grid row above the host cards. It was the scroll anchor of the result notices. After the correction, the anchor shows only after a new host. The host cards start at the top of the page.
- The visual check ran on the desktop shell and on the server shell, at 1100 px in the two themes and at 390 px. The Hosts page shows the host cards, then "Add a host". With no host, it shows only "Connect your first host". The Settings page shows Account (on the server only), Connected apps, Announcements and Library. `$edges()` and `$small()` gave empty lists.

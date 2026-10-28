# Task 048 Review Checklist, 2026-10-28

Task: [task 048](../tasks/studio-v1-task-048-add-a-feed-from-releases.md). Decision: [ADR 0072](../adr/0072-add-a-feed-opens-from-releases.md).

## Decisions Of The Planning Model

- The button is in the head of Releases, not in the bar. musicindex ADR 0027 R-TOOLBAR-1 item 1 gives the bar the back control and the title.
- The button shows only when the collection has a feed. The empty page has its primary button.
- The Releases row is current on Add feeds, because Add feeds is a page of the Releases destination.

## Packet

- [ ] `leftPanel` gives Releases only at the top, and `bottom` did not change.
- [ ] `narrowStack` changed only for the view `add-feed`.
- [ ] The button has the role `action` and the label "＋ Add a feed".
- [ ] `AddFeeds` did not change.
- [ ] Each changed guide page passes the STE check.
- [ ] No new UI function changes the model (ADR 0065).

## Visual Check

- [ ] The visual criteria of task 048, with screenshots and `$edges()`.

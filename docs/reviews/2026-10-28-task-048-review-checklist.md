# Task 048 Review Checklist, 2026-10-28

Task: [task 048](../tasks/studio-v1-task-048-add-a-feed-from-releases.md). Decision: [ADR 0072](../adr/0072-add-a-feed-opens-from-releases.md).

## Decisions Of The Planning Model

- The button is in the head of Releases, not in the bar. musicindex ADR 0027 R-TOOLBAR-1 item 1 gives the bar the back control and the title.
- The button shows only when the collection has a feed. The empty page has its primary button.
- The Releases row is current on Add feeds, because Add feeds is a page of the Releases destination.

## Packet

- [x] `leftPanel` gives Releases only at the top, and `bottom` did not change.
- [x] `narrowStack` changed only for the view `add-feed`.
- [x] The button has the role `action` and the label "＋ Add a feed".
- [x] `AddFeeds` did not change.
- [x] The changed lines of each guide page pass the STE check. The pages keep older findings.
- [x] No new UI function changes the model (ADR 0065).

## Visual Check

- [x] The visual criteria of task 048, with screenshots and `$edges()`.

## Result Of The Review, 2026-10-28

- The packet passed. The coding model found one defect in the fixed architecture: on Add feeds the row Releases is current, and the left panel did not open a current row. The planning model added the key `opens` to the row in `leftPanel`, with a test. After the correction, a click on Releases on the Add feeds page opens Releases.
- The planning model added one sentence to `add-feeds.md` for the empty page, which shows "Add a Feed".
- The visual check found that "＋ Add a feed" went 22 px over the right margin at 390 px. After the correction, `.releases-tools` wraps, and the button goes to a second line. At 320 px, 390 px and 1100 px each control is in the margins.
- The visual check ran at 1100 px in the two themes and at 390 px. Back from Add feeds opens Releases. The root page shows Releases, Hosts and Settings. `$edges()` and `$small()` gave empty lists.
- The image proxy gives 422 for the GIF cover of the corpus feed in the check data. This task did not cause it.

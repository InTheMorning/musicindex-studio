# ADR 0116: The Page Of A Person Has Save And Discard

- Status: Accepted
- Date: 2026-11-02
- Owner: operator
- Accepted: 2026-11-02 by the operator, from screens 1, 5 and 6 of the mockups in `.studio-dev/mockup-lookup/site/`.
- Changes: [ADR 0111](0111-the-page-of-a-person-checks-the-feeds.md) item 2, the save of each field when the user leaves it. [ADR 0113](0113-the-page-of-a-person-suggests-a-picture-and-an-information-address.md) item 8, the save of "Use".
- Uses: [ADR 0039](0039-the-shell-releases-and-the-outline.md) and [ADR 0029](0029-editor-pages-toolbar-and-three-levels.md) for the toolbar, [ADR 0037](0037-button-roles.md) for the button roles, and musicindex ADR 0023 R-CONFIRM-1 for the confirmation sheet.

## Context

- A field of the page of a person saves when the user leaves it (ADR 0111 item 2). A menu saves when the user selects a value. "Use" on a suggestion saves immediately (ADR 0113 item 8).
- The People test of `0.1.0-rc.10` on 2026-11-02 found that nothing showed the save, and the page had no "Save Person" (ADR 0111, Context). ADR 0111 added the status "Saved" below each field.
- ADR 0117 adds a lookup that fills many fields at one time. The user must see the values before Studio keeps them, and must be able to put back the old values.
- On 2026-11-02 the operator asked for Save and Discard on the page of a person. The operator asked for them in the toolbar, at the place of the draft toolbar of a feed.
- Each screen gives its own toolbar content (`crates/api/ui/app.js`, the comment of the toolbar of a screen). The page of a person opens no feed, so its toolbar has no draft of a feed.

## Decision

1. The page of a person keeps each change of these controls as a pending change until the user selects Save:
    - The fields "Name", "Picture address" and "Information address".
    - The menus "Usual role", with its usual group (ADR 0112), and "Member of", and the button "Make Default" ([ADR 0139](0139-the-default-address-is-on-its-row.md)).
    - The fills of the lookup of [ADR 0117](0117-the-page-of-a-person-finds-values-on-musicindex.md).
2. The toolbar of the page has the status, "Discard" (normal) and "Save" (primary), at the place of the draft toolbar of a feed. They use the markup of the draft toolbar. The toolbar stays in view at all scroll positions and widths (ADR 0039).
3. The status is "No changes", "1 change not saved" or "N changes not saved". With no pending change, "Discard" and "Save" are disabled.
4. A control with a pending change has a mark at its start edge. The status "Saved" below a field of ADR 0111 item 2 is removed. The hint "Studio saves each field when you leave it." is removed.
5. Save sends all the pending changes in one request. The core applies them in one transaction. A failure changes nothing, and the status shows the error. After a save, the status is "No changes".
6. Discard puts back the saved values in the page. It sends no request.
7. Save writes no feed. A person is local data. The primary button of the page is "Save", and the primary button of a feed is "Review". Thus the user can see that the two buttons do different things.
8. Before the user leaves the page with a pending change, the page opens the sheet "Leave without saving?". The sheet names the pending changes. The buttons are "Keep Editing" (normal) and "Discard Changes" (destructive). The texts are in `confirms.js`.
9. These sheets and operations keep their own buttons, and save their items when the user confirms the sheet: "Add a Wallet", the Nostr keys, the credit names, the support links, "More from your feeds", "Check your feeds", each operation of "Change your feeds", "Merge with a Person" and "Delete Person". These use the saved values of the person. With a pending change, the page opens the sheet of item 8 before it opens one of them.
10. When the browser closes or loads the page again with a pending change, the page asks through the `beforeunload` event of the browser. This is not a call of `confirm(`. A web page cannot show its own sheet at that time.

## Alternatives Considered

- A bar at the bottom of the page that shows only with pending changes. Not chosen by the operator: the toolbar is the place of the pending changes of a feed, so the user learns one place.
- Save also for the sheets of item 9. Not chosen by the operator: each of these sheets has its own confirmation, and a pending wallet or key must then show as pending in each list.
- The save of each field when the user leaves it, as before. Not chosen: a fill of many fields then saves values that the user did not examine.

## Consequences

- The API gets one route that applies the pending changes of item 1 in one transaction. The routes of each field stay for the sheets.
- The UI gets a model of the pending changes, with no Preact and no browser state (ADR 0042).
- "Use" of ADR 0113 puts the value in the field as a pending change.
- A task file gives the work.

## Non-goals

- Versions of a person. Save keeps one current value, as before.
- A pending change that stays after the page closes.

## Acceptance Criteria

Mechanical:

- A Node test: the model counts the pending changes, gives the status texts of item 3, and gives the body of the save request. Discard gives the saved values.
- A Node test: "Use" of ADR 0113 makes a pending change and sends no request.
- A Node test: `confirms.js` has the texts of item 8.
- An API test: the save route applies all the changes, or none after a failure.
- The four commit checks pass.

Visual: screens 1, 5 and 6 of the mockups, at 1100 px and at 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human: the operator changes two fields of a person, selects Discard, and sees the old values. The operator changes them again, selects Save, and opens the page again.

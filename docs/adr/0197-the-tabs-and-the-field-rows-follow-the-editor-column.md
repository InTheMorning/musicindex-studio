# ADR 0197: The Tabs And The Field Rows Follow The Width Of The Editor Column

- Status: Accepted
- Date: 2026-11-13
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-13 by the operator, through the questionnaire after the human test of 0.1.0-rc.22. The operator selected "Switch to the menu", "Label above value" and "Fit menu, native list".
- Technical choices: items 2, 5, 6 and 8 and the width of item 7 are choices of the planning model.
- Changes: [ADR 0157](0157-the-page-tabs-are-a-segmented-control.md) item 3, [ADR 0135](0135-the-panel-holds-the-content-and-the-page-has-tabs.md) item 6 and [ADR 0166](0166-settings-has-tabs-and-one-save-bar.md) item 3. The menu no longer starts at a window width of 760 px. It starts when the tabs do not fit their column.
- Uses: musicindex ADR 0051 R-SEG-1 items 4 and 5, and musicindex ADR 0065 R-FIT-2 and R-FIT-3.
- Source: finding 1 and finding 2 of the [human test findings of 0.1.0-rc.22](../reviews/2026-11-13-rc22-human-test-findings.md).

## Context

Studio has one layout breakpoint at a window width of 760 px. From 761 px, the left panel takes about 300 px, and the editor column keeps its wide form in about 450 px. At that width, the tab labels break inside words ("Valu e Rou tes"), and the field values are cut off. The rows of the tab **Check** wrap each title on many lines. ADR 0157 item 3 let a label wrap in its segment, and `.segment` has `overflow-wrap: anywhere`, so a label could also break inside a word.

The window width does not tell the width of the editor column, because the left panel takes a part of it. The fit walk of musicindex ADR 0065 now checks 390 px and 761 px, and it fails on the six tab labels at 761 px.

At a narrow width, the closed tab menu fills the width of the page. Its open list is the native list of GTK, which is not aligned with the closed menu. musicindex.org has no select menu in the place of a segmented control. Its segmented control fills the row in a narrow list (`search.html`, `@container list-page (max-width: 520px)`). Thus no other app gives the form of the menu.

## Decision

### The Tabs

1. The tabs of a feed page and the tabs of Settings are a segmented control when each segment can hold its longest label on one line. Each label then has a minimum of 4 px at each side (musicindex ADR 0051 R-SEG-1 items 4 and 5). If a label cannot have this space, the tabs are one select menu.
2. Studio measures the width that the tabs have in their column, not the window width. Studio measures again when that width changes. The width of a label comes from the font of the page, so the UI scale of ADR 0134 changes it.
3. A segment label stays on one line. It does not wrap, and it does not break inside a word.
4. The closed select menu is as wide as its longest label, with its padding and its arrow. It does not fill the row. Its left edge is on the left edge of the page content.
5. The open list of the menu is the native list of the platform. Studio does not style it.

### The Field Rows

6. A field row in a group puts its label to the left of its value when the group is 560 px wide or more. When the group is narrower, the label is above the value. Studio measures the group, not the window.
7. A row of the tab **Check** follows the same width of 560 px. In a narrower group, the button of the row goes below its text.
8. Each row that has a label column and a value column follows item 6. The fit walk finds a row that does not.

## Alternatives Considered

- A second breakpoint for the window, for example at 1000 px. Not chosen: the left panel can change its width, and the UI scale changes the width of each label.
- Shorter tab labels. Not chosen: the Check tab adds its count, and a label of a different feed type can be longer.
- Tabs that scroll to the side. Not chosen for the same reason as ADR 0135: a tab starts outside the column.
- A styled list for the open menu. Not chosen: the native list has the keyboard and the screen reader behavior of the platform.
- A closed menu that fills the row, as now. Not chosen by the operator: the rectangle is wider than its text and looks different from the indented rows of the page.

## Consequences

- At 761 px with the left panel, a feed page shows the tab menu, and each field has its label above its value.
- A wide window with a wide left panel can also show the menu.
- The menu and the segmented control show the same tabs, with the count of the Check tab (ADR 0174 item 17).
- The rule of ADR 0157 item 3 that a label wraps in its segment no longer applies.

## How This Decision Is Checked

Mechanical:

- A Node test of the UI model: from the width of the column and the width of each label, the model gives the segmented control or the menu, with the 4 px of item 1.
- The fit walk of musicindex ADR 0065 at 390 px and 761 px gives no finding on a feed page, on the tab **Check** and on Settings.

Visual, by the agent, with screenshots:

- A feed page at window widths of 761, 900, 1000 and 1280 px with the left panel, and at 390 px, in the light and the dark theme.
- The closed menu at 761 px: its width and its left edge.

Human, by the operator, in the desktop app:

- Make the window narrower, from wide to about 760 px. The tabs change to the menu before a label breaks, and the fields put the label above the value.
- Open the menu. The native list opens, and each tab is in it.

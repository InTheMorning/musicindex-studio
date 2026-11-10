# ADR 0166: Settings Has Tabs And One Save Bar

- Status: Accepted
- Date: 2026-11-10
- Owner: operator
- Accepted: 2026-11-10 by the operator, through the questionnaire. The operator compared the mockups in `.studio-dev/settings-mockup/`, and selected the segmented control, four groups, one Save bar and a preview of the UI scale.
- Technical choices: items 5, 9, 12, 13 and 14 are choices of the planning model.
- Applies: musicindex ADR 0059 R-SET-1 to R-SET-5, at musicindex `83142d2`.
- Amended: 2026-11-10. Items 17 and 18 apply musicindex ADR 0059 R-SET-3 item 6.
- Changes: [ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) item 10.
- Uses: [ADR 0135](0135-the-panel-holds-the-content-and-the-page-has-tabs.md) item 6 for the narrow tabs. [ADR 0116](0116-the-page-of-a-person-has-save-and-discard.md) items 8 and 10 for the leave question. [ADR 0165](0165-the-left-panel-marks-the-state-of-each-feed.md) item 2 for the pencil.

## Context

The Settings page was one long page with ten sections. Appearance, Lock, Window and Outside changes saved at each change, and showed no result. Nostr and Library had their own Save button. Connected apps, Move Studio and Collections are actions. No section had Discard.

musicindex ADR 0059 gives the shared layout of Settings for Studio and v4vmm. This record applies it to Studio.

## Decision

### The Tabs

1. The Settings page has a segmented control below its bar (musicindex ADR 0059 R-SET-1). The tabs, in this sequence:
    - **General**: Appearance, Lock, Window and Outside changes.
    - **Account**, on a server only: the Account section.
    - **Nostr**: the Nostr section.
    - **Apps**: Connected apps.
    - **Data**: Collections, Move Studio and Library on the desktop. On a server: Library, or "Files on this server" when the server holds no assets.
2. The desktop app has four tabs, and a server has five.
3. At 760 px or less, the tabs are one select menu, as on a feed page (ADR 0135 item 6).
4. The page opens on General. The open tab is part of the view, so Back and Forward go between tabs.
5. `crates/api/ui/pages.js` gives the tabs and the sections of each tab. The sections of one tab keep the tier order of ADR 0071 item 1, and the tier test examines each tab.

### The Rows

6. Each setting is one row: the name and a hint at the start, and the control at the end (musicindex ADR 0059 R-SET-2). The Nostr search relay and the two library limits are text fields, and they can be below their name.

### The Draft

7. These settings are part of the draft: the UI scale, the lock time, the close of the window, the check of outside changes, the Nostr search relay and the two library limits. A change of one of them stays pending until Save (musicindex ADR 0059 R-SET-3).
8. The bar shows the state, **Discard** and **Save**, with the texts of the page of a person: "No changes", "1 change not saved", "N changes not saved". After a save, it shows "Saved".
9. A changed row and a tab with a pending change show the pencil of ADR 0165 item 2, in the `--accent` color. Its accessible name is "Not saved".
10. The draft stays when the user goes to a different tab. A change of the tab does not ask the leave question.
11. When the user leaves Settings with a pending change, Studio opens the sheet "Leave without saving?" (ADR 0116 item 8). A close or a load of the page asks through `beforeunload` (ADR 0116 item 10).
12. Save sends each pending setting to its current route. When a route refuses a value, that setting stays pending, and its row shows the cause. The other settings keep their save.
13. The UI scale shows immediately when the user selects it (musicindex ADR 0059 R-SET-4). Save stores it on this computer. Discard, or a leave with no save, puts back the saved scale.
14. The Nostr section and the Library section have no Save button of their own.

### The Actions

15. Connected apps, Collections, Move Studio and each operation of Account keep their buttons and their sheets. They run immediately, and Save and Discard do not include them (musicindex ADR 0059 R-SET-5).
16. **Open** of a different collection restarts the app. With a pending change, Studio asks the leave question first.

### A Draft That Cannot Be Saved

17. Studio knows before Save that it cannot save the draft in these conditions (musicindex ADR 0059 R-SET-3 item 6):
    - A library limit is not a number greater than 0.
    - The Nostr search relay is not empty, and does not start with `wss://` or `ws://`. The route refuses this value with `invalid_relay`.
18. In each condition of item 17, Save is off. The page bar gives the cause, and the row of the field shows the cause. Discard stays on.

## Consequences

- The Settings page has no long scroll on the desktop.
- Each setting saves the same way, and the user sees the result.
- A pending change on a different tab stays visible through the pencil on its tab.

## How This Decision Is Checked

Mechanical:

- A Node test of `pages.js` examines the tabs on the desktop and on a server: the labels, the sequence, the sections of each tab, and the tier order in each tab.
- A Node test of the draft model examines the state texts, Discard, and a refusal that keeps one setting pending.
- A Node test of the draft model examines each condition of item 17: Save is off, and the model gives the cause.
- A browser drive changes the UI scale and the lock time, goes to a different tab, selects Discard, then does the same with Save.

Visual, for the operator: the shots of the agent cover the layout in the light and the dark theme. They cover the desktop width and 390 px. The operator examines the select menu of the tabs on the iPhone.

## Alternatives Considered

musicindex ADR 0059 gives the alternatives: a list of groups, tabs in a column at the side, and one page with jump links.

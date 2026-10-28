# ADR 0071: Settings Is A Destination, And Each Page Orders Its Sections By Tier

- Status: Accepted
- Date: 2026-10-28
- Owner: operator
- Accepted: 2026-10-28 by the operator.

## Context

The Hosts page shows these sections in this sequence: Library, Announcements, Account (on a server only), Connected apps, the host cards, and "Add a host". Each section came with a different task. Hosts was the only page for settings, so each task put its section there. [ADR 0052](0052-announcements-through-the-podcast-index-hub.md) item 97 puts the announcement settings on the Hosts page.

On 2026-10-28 the operator examined the page. The first field on the page is "Largest download, in MB". The host cards are the function of the page, but they are at the bottom of the window. The operator asked for a settings page. The operator also asked for a guard that stops the settings with the minimum importance from coming first.

[ADR 0068](0068-the-sequence-of-the-sections-and-fields-of-an-editor-page.md) item 1 gives a principle for the editor pages. Identity comes first, then content, then money and credits. The settings that the user sets one time come next, and the technical facts are last. Its test compares each page with a list in the ADR. Thus each new section needs a change to the ADR. The operator does not want a new ADR for each new feature.

The Studio journeys in `../musicindex-research` (`research_notes/ui-design/journeys/studio/`) count the steps of six tasks. They give no data on how frequently a user does each task. They are research, not rules.

v4vmm has a destination "Settings". musicindex ADR 0027 R-SHELL-1 item 4 lets an app pin a destination to the bottom of the left panel.

## Decision

### The Tiers

1. Each section of a page has one tier. The tiers are, in their sequence:
   1. `identity`: the item of the page, or the objects that the page is for. Examples: the album fields, the host cards.
   2. `content`: the parts of the item that the user changes in most visits. Examples: the tracks, "Add a host".
   3. `details`: money, credits, lyrics and the links to other feeds.
   4. `setup`: the settings that the user sets one time.
   5. `limits`: the technical facts, and the limits that most users keep at their default values.
2. A page shows its sections in the sequence of their tiers. Two sections with the same tier can come in a sequence that the module selects. 
3. A row of a section has the tier of its section. A row does not move a section to a different tier.
4. A notice is not a section. A result notice and an error notice show where the user can see them, and they have no tier.

### The Model Module And The Guard

5. `crates/api/ui/pages.js` gives the sections of each page, in their sequence, each with its tier. It gives the editor pages of ADR 0068, the Hosts page and the Settings page. Each of these pages shows its sections from this list.
6. A Node test examines each page of the module:
   - Each section has a tier of item 1.
   - The tiers do not go down in the sequence of a page.
   - The error message names this ADR, the page, the section and its tier, and the fix: "Give the section its tier of ADR 0071 item 1, and put it after each section of a lower tier in crates/api/ui/pages.js."
7. A new section, or a new page in the module, needs no ADR for its position. The task packet gives the tier of each new section, and why. The inspection of the diff compares that tier with item 1. When the tier is not sure, the task packet escalates to the operator.
8. An ADR is necessary only to change the tiers of item 1, or to change this guard.

### The Settings Page

9. "Settings" is a destination. It is at the bottom of the left panel, between Hosts and Lock Studio. It stays at that position with the outline, as Hosts does ([ADR 0039](0039-the-shell-releases-and-the-outline.md) item 1). At 760 px or less, the root page shows it at the same position.
10. The Settings page has these sections:
    1. "Account" (`setup`), on a server only.
    2. "Connected apps" (`setup`).
    3. "Announcements" (`setup`).
    4. "Library" (`limits`): the library folder and the two download limits.
11. The bar title of the page is "Settings". The editor page "Feed settings" keeps its name. It is the settings of one feed.

### The Hosts Page

12. The Hosts page has these sections:
    1. The host cards (`identity`).
    2. "Add a host" (`content`). When no host is connected, "Connect your first host" is the only section.
13. The result notices of a new host stay above the host cards.
14. The announcement settings of ADR 0052 item 97 are on the Settings page. This changes ADR 0052 item 97.

### ADR 0068

15. This ADR replaces ADR 0068 item 11 and its check. The sequences of ADR 0068 items 2 to 9 stay as the start values of the module. After the change, the module owns the sequence of each page, and the tier test is the guard. The test that compares each page with a fixed list is deleted.

## Alternatives Considered

- Put the host cards first, and keep all the settings on the Hosts page. Not recommended: the page name "Hosts" then continues to have two jobs. Each new setting then goes to the Hosts page again.
- A new user journey study before each feature, to find the position of each new section. Not recommended: the journeys of 2026-10-24 give step counts, not frequencies, and a journey study is not a rule. The tier of item 1 holds the frequency fact in the code, where a test can examine it.
- Keep the fixed lists of ADR 0068 and add the Hosts page and the Settings page to them. Rejected because of what the operator asked: each new section then needs a change to an ADR.
- A number for each section, not a tier. Not recommended: a number gives a sequence but not why. The inspection of the diff cannot compare a number with a definition.

## Consequences

- The left panel gets one more row at its bottom. ADR 0039 items 1, 5 and 11 name Settings with Hosts.
- The Hosts page shows the host cards first.
- A new section needs a tier in the module, and no ADR.
- The rule "the tiers do not go down" is a durable rule. This ADR is the record that adds it to the durable set.
- The user guide pages `hosts.md`, `announcements.md`, `account.md`, `connected-apps.md`, `find-your-way.md` and `first-run.md` name the new page.
- Two sections of one tier can change their sequence, and the test continues to pass. The visual check examines that sequence.

## Invariants

- On each page of the module, the tiers of the sections do not go down.
- The host cards are the first section of the Hosts page when a host is connected.
- No setting of Studio is on the Hosts page.

## Non-Goals

- No new setting, and no change to a setting.
- No change to the outline, to the editor pages, or to their sequence.
- No change to the shared shell rules of musicindex ADR 0027.

## Verification

Mechanical:

- The Node test of item 6 passes for each page of the module.
- A Node test finds the destination "Settings" between Hosts and Lock Studio in the left panel model of `crates/api/ui/shell.js`.

Visual:

- Examine the Hosts page and the Settings page at desktop width and at 390 px. Examine them in the two themes, on the desktop shell and on the server shell. The `$edges()` check of `tools/drive-ui.mjs` finds no content at the edge of a group.

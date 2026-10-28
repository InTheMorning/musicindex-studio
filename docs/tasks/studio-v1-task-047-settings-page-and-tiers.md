# Studio v1 Task 047: The Settings Page And The Section Tiers

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The Editor Pages And The Outline".
Decisions: [ADR 0071](../adr/0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md), [ADR 0068](../adr/0068-the-sequence-of-the-sections-and-fields-of-an-editor-page.md), [ADR 0039](../adr/0039-the-shell-releases-and-the-outline.md), [ADR 0052](../adr/0052-announcements-through-the-podcast-index-hub.md) and [ADR 0042](../adr/0042-ui-models-with-node-tests.md).

Status: Complete - 2026-10-28, except the human test. The four commit checks pass. The screenshot visual check passed, and the [review checklist](../reviews/2026-10-28-task-047-review-checklist.md) records it. The human test on the server from a phone is open.

## Goal

Each section of a page has a tier, and a Node test stops a section from coming before a section of a lower tier. "Settings" is a destination with the Studio settings. The Hosts page shows the host cards first.

## Packets

The packets go in this sequence:

1. [047a: the tiers and the guard](studio-v1-task-047a-tiers-and-guard.md). It changes the model modules and their tests only.
2. [047b: the Settings page and the Hosts page](studio-v1-task-047b-settings-and-hosts-pages.md). It depends on 047a.

## Decisions Of The Planning Model

These decisions are in the limits of ADR 0071. The [review checklist](../reviews/2026-10-28-task-047-review-checklist.md) records them.

- The tiers of the editor pages, from ADR 0071 item 1:

  | Page | Section | Tier |
  | --- | --- | --- |
  | album | `album` | `identity` |
  | album | `tracks` | `content` |
  | album | `details`, `publishers` | `details` |
  | album | `feed` | `setup` |
  | publisher | `channel` | `identity` |
  | publisher | `releases` | `content` |
  | publisher | `role`, `feed` | `setup` |
  | playlist | `channel` | `identity` |
  | playlist | `feed` | `setup` |
  | settings (Feed settings) | each section | `setup` |
  | track | `track` | `identity` |
  | track | `credits`, `lyrics`, `payments` | `details` |
  | track | `more` | `setup` |

- Two new pages in `pages.js`: `hosts` with `cards` (`identity`) and `add` (`content`), and `studio-settings` with `account`, `apps`, `announcements` (each `setup`) and `library` (`limits`). The key `settings` stays the editor page "Feed settings".
- `pageSequence(name)` keeps its result shape. Each section object gets the key `tier`. A new export `TIERS` gives the five tiers in their sequence.
- `pageSequence('studio-settings', { server })` leaves out `account` when `server` is false.
- The view of the Settings page is `{ name: 'settings' }`. The left panel row has the ID `settings` and the label "Settings". It has no type dot, as "Lock Studio" has none: Settings is not an entity (musicindex ADR 0032 R-DOT-1).

## Acceptance Criteria

Mechanical:

- The mechanical criteria of each packet.
- The four commit checks pass.

Visual, for the agent with `tools/drive-ui.mjs`. Each one in the dark theme, in the light theme and at 390 px, on the desktop shell and on the server shell:

- The left panel shows Hosts, Settings and Lock Studio at its bottom. The root page at 390 px shows the same rows.
- The Hosts page shows the host cards first, then "Add a host". It shows no setting.
- The Settings page shows its sections in the sequence of ADR 0071 item 10.
- `$edges()` finds no content at the edge of a group.

Human test, by the operator:

- On the server, open Settings from a phone, and add a passkey from "Account".

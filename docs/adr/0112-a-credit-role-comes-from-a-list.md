# ADR 0112: A Credit Role Comes From A List

- Status: Accepted
- Date: 2026-11-02
- Owner: operator
- Accepted: 2026-11-02 by the operator, from the mockups in `.studio-dev/mockup-people-2/site/`.
- Changes: [ADR 0098](0098-the-people-module.md) item 5, the usual role of a person. [ADR 0064](0064-the-editor-edits-the-fields-that-msp-2-0-edits.md), the role field of a credit.
- Uses: [ADR 0065](0065-the-core-applies-each-feed-rule.md) for the group of a role, and [ADR 0012](0012-lossless-tree-with-a-typed-model.md) for the bytes of a credit that the user does not change.

## Context

- The role of a credit is a free text field. Its hint is "For example vocalist. Empty means host." The operator wants a list of roles before a free text.
- The `podcast:person` tag of the Podcasting 2.0 namespace says that `role` and `group` "should" reference the Podcast Taxonomy Project list. With no `role`, apps take "host". With no `group`, apps take "cast". The two attributes are case-insensitive.
- The list is `taxonomy.json` of the podcast-namespace repository, under CC0 1.0. On 2026-11-02 it has 63 roles in 11 groups. Most roles are for podcasts. The music roles are Songwriter, Composer, Audio Engineer, Music Production, Music Contributor and Cover Art Designer. The role "Editor" is in two groups.
- MSP 2.0 adds a group "Music" that is not in the list: Band, Vocalist, Guitarist, Bassist, Drummer, Keyboardist and Musician. It writes `group="music"`.
- The mockups are in `.studio-dev/mockup-people-2/site/` (screens 8a and 8b). The operator selected the list of MSP 2.0 and the official list, then a free text.

## Decision

1. The core keeps a list of roles. It has the group "Music" of MSP 2.0 first, then the 11 groups of `taxonomy.json`, with each role of each group. The core is the owner of the list, and the API gives it to the UI.
2. The role field of a credit is a select menu, with one group of options for each group of the list. Its last option is "Other role…".
3. "Other role…" shows the text field "Other role" and the select menu "Group". The hint of the text field is "Use a role from the menu when one fits. Apps know those roles."
4. When the user selects a role of the list, the core writes the role and the group of that role at the draft save. A role that is in two groups gets the group of the option that the user selected.
5. Studio writes the role and the group in lowercase, for example `role="vocalist"` and `group="music"`, as MSP 2.0 does. For a group of `taxonomy.json`, Studio writes the official name in lowercase, for example `group="audio production"`.
6. The menu shows a role of a feed that is in the list, with no regard to case. A role that is not in the list shows as "Other role…" with its text. A credit that the user does not change keeps its bytes.
7. The usual role of a person uses the same menu. A person keeps a usual group, for a usual role from "Other role…". "Pick from your people" and "Add to All Releases" write the usual role and the usual group.
8. A new credit starts with no role selected. The menu has the placeholder "Select a role", and the hint says that apps show a credit with no role as "Host".

## Alternatives Considered

- The official list only. Not chosen by the operator: it has no Vocalist, Guitarist or Drummer.
- A free text with suggestions. Not chosen by the operator: it permits a different spelling of one role in each feed.
- Write the slug groups of MSP 2.0, for example `audio-production`. Not chosen: the namespace compares the group with the official names, and a slug does not match them.

## Consequences

- The core gets the list and the group rule at the draft save. The list is data of the core, so the UI has no copy of a feed rule (ADR 0065).
- The group "Music" is not official, so some apps may not know it. The operator accepted this, for the same values as MSP 2.0.
- When `taxonomy.json` changes, a change of the core list follows it. The list names the commit of `taxonomy.json` that it uses.
- The check of ADR 0111 item 9 finds a credit with no role.
- A task file gives the work.

## Acceptance Criteria

Mechanical:

- A core test: the list has each role of `taxonomy.json` and the group "Music", and "Editor" is in two groups.
- A core test: a role of the list gets its group at the draft save, and a credit that the user does not change keeps its bytes.
- A core test: a role in a feed matches the list with no regard to case. A role that is not in the list stays as it is.
- A Node test: the menu shows a role of the list, and an unknown role as "Other role…" with its text.
- The four commit checks pass.

Visual: screens 8a and 8b of the mockups, at 1100 px and at 390 px with touch. `$fieldText()` is empty.

Human: the operator sets the role "Vocalist" on a credit, writes the feed, and the file host has `role="vocalist" group="music"`.

# ADR 0252: A Publisher Link With No Role Gets A Repair

- Status: Accepted
- Date: 2026-11-23
- Owner: operator
- Accepted: 2026-11-23 by the operator, through the questionnaire, from drawings. For a link with no `rel`, the operator selected "Stop with a repair". For a feed that Studio cannot write, the operator selected "Notice".
- Technical choices: the main session selected the texts, the link of item 3 and the order of the buttons in items 2 and 4.
- Changes: [ADR 0218](0218-a-page-link-in-review-and-a-suggestion-starts-with-no-person.md) items 1 and 8. The refusal panel and the finding get the repair of a link with no role, and the finding of a read-only feed is a Notice.
- Uses: [ADR 0256](0256-a-release-starts-on-the-page-of-its-artist.md) items 17, 18 and 23, the page rule and its finding. [ADR 0174](0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md), a repair changes the draft. [ADR 0249](0249-studio-strongly-discourages-a-feed-guid-change.md) item 6, a finding with a second repair.
- Source: observation 1 of [the desktop check walk of 2026-11-23](../reviews/2026-11-23-desktop-check-walk.md).
- Bar: opposite choice by mistake. An agent can count a link with no `rel` as an artist link, or keep the Stop on a feed that Studio cannot write.

## Context

- `page_refusal` in `crates/core/src/feed_check.rs` finds an artist page only from a `podcast:remoteItem` in `podcast:publisher` with `medium="publisher"` and a `rel` that has the role `artist`.
- The Podcasting 2.0 standard does not make `rel` necessary in `podcast:publisher`. Feeds from other tools frequently have the link with no `rel`.
- On the walk of 2026-11-23, a release of Elijah Lied had such a link. **Issues** gave the Stop "No artist page" adjacent to a link to the artist page. The release has no file host, so Studio cannot write it. The operator confirmed that the link has no `rel`.

## Decision

### The Link With No Role

1. A link with no role is a `podcast:remoteItem` in `podcast:publisher` with `medium="publisher"` and no `rel` attribute. It is not an artist link. ADR 0256 item 17 does not change.

### The Finding

2. When a release has the finding "This release has no artist page." (ADR 0256 item 18) and a link with no role, the finding gets the line "The link to {title} has no role." Its first button is **Make It the Artist Link**, and its second button is **Link a Page** (ADR 0256 item 19).
3. `{title}` is the title of the linked feed in the collection. With no such feed, it is the `title` attribute of the link. With no `title`, it is the `feedUrl` of the link.
4. **Make It the Artist Link** adds `rel="artist"` to the first link with no role, in the draft (ADR 0174). It changes no other attribute and no other link. The user then writes the draft in Review.

### Review

5. The refusal panel of ADR 0218 item 1 for a release with a link with no role has the line of item 2 under its first line. It has the button **Make It the Artist Link** above the select menu. The button does the repair of item 4 and loads the diff again. When the refusal goes, the write button works.

### A Feed That Studio Cannot Write

6. On a read-only feed (ADR 0174), the findings "This release has no artist page." and "This feed has no artist, label or hosting page." have the level Notice. They have no button and no line "Studio cannot write it before the link." (ADR 0218 item 8). On a feed that Studio can write, the level stays Stop.

## Item Map

| Earlier item | Change by this ADR |
| --- | --- |
| ADR 0218 item 1 | Item 5: the panel has the line and the button of a link with no role. |
| ADR 0218 item 8 | Items 2 and 6: the finding gets the repair of a link with no role, and it is a Notice on a read-only feed. |

## Consequences

- A release from another tool with a link with no role needs one click and one write. The file then names the role.
- A read-only feed shows the fact with no Stop, because Studio writes nothing for it.
- A link with no role to a label or hosting page also gets the repair. The user can see this from the title in the line of item 2, and the button **Link a Page** stays.

## Alternatives Considered

- A link with no role counts as the artist link. Not chosen by the operator.
- No change. Not chosen by the operator.
- The Stop stays on a read-only feed. Not chosen by the operator.

## Checks

Mechanical:

- Core test: a release with a link with no role gives the finding with the line of item 2, the repair **Make It the Artist Link** and the second repair **Link a Page**.
- Core test: the repair adds `rel="artist"` to the first link with no role only, and the page refusal then goes.
- Core or API test: the title of item 3 for a linked feed of the collection, for a `title` attribute, and for a `feedUrl` only.
- API test: on a read-only feed, the finding has the level Notice and no repair.
- UI test (`crates/api/ui-tests/`): the refusal panel model gives the line and the button of item 5 when the diff names a link with no role.

Visual, by the main session with screenshots at 1100 px and 390 px:

- **Issues** of a release with a link with no role, with the two buttons.
- The refusal panel of Review with **Make It the Artist Link**, before and after the repair.

Human check: on the desktop, **Issues** of the Elijah Lied release shows a Notice, with no button.

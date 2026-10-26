# ADR 0041: Publisher Feeds And Two-Way Links

- Status: Accepted
- Date: 2026-10-25
- Owner: operator, with the technical choices of the planning model
- Amended: 2026-10-26 by operator decision. [ADR 0054](0054-live-items-connected-apps-and-show-grants.md) item 18 adds one exception to the invariant: a change that a show grant of the user covers.

## Context

Phase 8 of the [Studio v1 phase plan](../plans/studio-v1-phase-plan.md) makes and edits publisher feeds for labels and artists. The plan asks for a decision on two questions: how a release refers to its publisher feed, and what Studio changes in the release feeds.

[ADR 0008](0008-feed-types-and-feed-identity.md) item 1 makes publisher feeds a v1 feed type, with `remoteItem` and the `rel` role set of Podcast Namespace PR #793. The core reads and writes `podcast:publisher` and the channel `podcast:remoteItem` elements now.

The corpus and the published feeds use one structure:

- A publisher feed has `podcast:medium` `publisher`, no items, and one channel `podcast:remoteItem medium="music"` for each release, with `feedGuid`, `feedUrl` and `title`.
- A release has `podcast:publisher`, with one `podcast:remoteItem medium="publisher"` for each publisher feed.

Stophammer is the index of musicindex.org. It confirms a link only when the link goes in the two directions (Stophammer ADR 0069 §1a and §2). The release names the publisher feed in `podcast:publisher`, and the publisher feed lists the release. Stophammer confirms a role only when the two sides give it in `rel` (Stophammer ADR 0049 §6a). It ranks the publisher feeds of a release in their sequence (Stophammer ADR 0069 §3a). The namespace does not define `rel` at this time (namespace discussion #579).

On 2026-10-25 the operator decided the questions of items 5, 8, 9 and 11.

## Decision

### Publisher Feeds

1. A publisher feed has `podcast:medium` `publisher`. It has a role, `artist` or `label`. The collection keeps the role of each publisher feed.
2. The user makes a new publisher feed in "Add feeds", with a host, a file name, a name and a role. The feed GUID is the UUIDv5 of its address ([ADR 0008](0008-feed-types-and-feed-identity.md) item 2). The draft gives the name to `title` and to `itunes:author`. The feed gets a release folder ([ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md) item 1).
3. A publisher feed on a connected host is editable. Its page has the title, the name, the description, the image, the website, the role and the list of releases. It has a Files page, as a release has.
4. A publisher feed from a host can have no recorded role. Then Studio takes the `rel` of its `medium="music"` items when each one gives the same value, `artist` or `label`. If not, the user selects the role before the first change of a link.

### The Link

5. A link joins a publisher feed and a release of the collection. Each of the two feeds is on a connected host. Studio does not offer a release that is not in the collection, or a release that is read-only.
6. In the publisher feed, a link is a channel `podcast:remoteItem` with `medium="music"`. It has the `feedGuid` and the `feedUrl` of the release, its title in `title`, and the role in `rel`.
7. In the release, a link is a `podcast:remoteItem` with `medium="publisher"` in `podcast:publisher`. It has the `feedGuid` and the `feedUrl` of the publisher feed, its title in `title`, and the role in `rel`. Studio does not write a channel `remoteItem` with `medium="publisher"` that is not in `podcast:publisher` (Stophammer ADR 0069 §1a).
8. An add or a removal of a link changes the draft of each of the two feeds. Studio starts a draft for a feed that has none. Each feed keeps its own diff and its own write ([ADR 0007](0007-draft-diff-write-and-announce.md)). After the change, the page names the other feed that has a changed draft.
9. A release can have more than one publisher feed. The album page shows them in the sequence of `podcast:publisher`, and the first one is the primary publisher. The user changes the sequence there. A new link goes last.
10. The publisher page shows the releases in the sequence of the `medium="music"` items. The user changes the sequence there. A new link goes last.
11. A change of the role changes the `rel` of each link in the draft of the publisher feed. It also changes the `rel` of the link in the draft of each linked release.
12. The `feedGuid` of a link is the `podcast:guid` of the draft of the other feed. When the other feed has no draft, it is the `podcast:guid` of its hosted file. The `feedUrl` is the public address of the other feed.
13. Studio keeps each `remoteItem` that it did not make. A link to a feed that is not in the collection shows as "Not in the collection". The user can remove it from the open feed. That removal changes no other feed.
14. In a release, Studio changes only `podcast:publisher`. In a publisher feed, it changes only the fields of item 3 and the channel `remoteItem` elements.

## Alternatives Considered

- One review sheet that writes the publisher feed and each changed release at the same time. Not chosen by the operator: each feed keeps its own diff and its own write.
- Change only the publisher feed, and let the user add the link in each release. Not chosen by the operator: if the user does not add it, Stophammer confirms no link.
- Links to releases that are not in the collection, as one-way links. Not chosen by the operator: each link that Studio makes goes in the two directions.
- A free text field for `rel`, or no `rel`. Not chosen by the operator: the role of the publisher feed gives the same `rel` on the two sides. Stophammer can then confirm the role.
- One publisher feed for each release. Not chosen by the operator: an album can name an artist and a label.

## Consequences

- The collection gets a column for the role of a publisher feed. A schema change is a new migration ([ADR 0014](0014-collection-in-sqlite.md)).
- One link changes two drafts. The user writes each one.
- Studio does not have the shell of [ADR 0039](0039-the-shell-releases-and-the-outline.md) at this time. Phase 8 uses the current feed tree. The work of ADR 0039 moves the publisher pages into the outline.

## Non-Goals

- A value block in a publisher feed, `podcast:podroll`, and the `feedImg` attribute.
- A change of the `title` or the `feedUrl` of a link after a change of the title or the address of the other feed. A subsequent decision can add it.
- A link to a feed that is not in the collection.
- The announcement of a new publisher feed. Phase 10 adds announcements.

## Verification

Mechanical, at the owning layer:

- A link gives the two `remoteItem` elements of items 6 and 7 in the two drafts, with the same `rel`. A removal takes the two elements out.
- A link to a release with no draft starts its draft.
- A change of the role changes each `rel` in each draft of item 11.
- A link that Studio did not make stays in the draft after each other change.

Visual, for the operator:

- The operator makes a label feed and an artist feed for the releases in the collection, and writes each feed.

## Invariants

- No feed changes without a diff that the user saw, or a change that a show grant of the user covers ([ADR 0007](0007-draft-diff-write-and-announce.md), [ADR 0054](0054-live-items-connected-apps-and-show-grants.md)).
- A link has the same `rel` on its two sides.
- A link does not change the feed GUID or an item GUID ([ADR 0008](0008-feed-types-and-feed-identity.md)).

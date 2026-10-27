# ADR 0063: One Switch Hides A Feed From Apps

- Status: Accepted
- Date: 2026-10-27
- Owner: operator

## Context

On 2026-10-27, during the human tests of task 039, the operator found that the test feed `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml` is blocked. Its channel has `<podcast:block>yes</podcast:block>` and `<itunes:block>yes</itunes:block>`. Studio did not show this, and the editor had no control to change it. The core did not model the two tags, so the tree kept them as the source wrote them ([ADR 0012](0012-lossless-tree-with-a-typed-model.md) item 4).

Two tags ask apps not to show a feed:

- `itunes:block` with the value `Yes` asks Apple Podcasts and the podcast directories not to show the feed. Podcast Index acts on this tag.
- `podcast:block` with the value `yes` and no `id` asks each app that reads the Podcasting 2.0 namespace not to show the feed. With an `id`, such as `spotify`, it applies to one platform only.

MSP 2.0 accepted a change for these tags (branch `itunes-block`, commit `0d304f1` of 2026-10-17, `src/utils/listing.ts`). To hide a feed, MSP 2.0 writes both tags: `<itunes:block>Yes</itunes:block>` and `<podcast:block>yes</podcast:block>`. It reads "yes" with capital letters or small letters as a block. It keeps each block for one platform, and it does not offer a control for each platform. [ADR 0019](0019-select-a-feed-from-the-host.md) shows "blocked" for a feed file in a host folder.

The operator decided on 2026-10-27:

- Studio writes the tags as MSP 2.0 does.
- A change back to a listed feed writes `no`. It does not remove the tags.
- Studio does not encourage a block for one platform. The user sees one simple switch. The blocks for one platform are in a collapsed "Advanced" row, which shows only when the feed has such a block before the edit.

## Decision

1. The core models the channel `itunes:block` as text, and each channel `podcast:block` as an entry with its text and its `id`. A read and a write with no edit give the same bytes.
2. A feed is hidden when its `itunes:block` says `yes`, or when a `podcast:block` with no `id` says `yes`. The compare ignores capital letters and the space at the ends. A block for one platform does not make a feed hidden.
3. The album page, the playlist page and the publisher page have one switch, "Hide from apps". When the user sets it to on, `itunes:block` gets `Yes` and each `podcast:block` with no `id` gets `yes`. When the feed does not have one of these tags, Studio adds it.
4. When the user sets the switch to off, each of these tags that the feed has gets `No` or `no`. Studio adds no tag and removes no tag.
5. The switch does not change a block for one platform. When the feed has a block for one platform, a collapsed "Advanced" row follows the switch. It has one check box for each such block, and the check box changes its value to `yes` or `no`. With no block for one platform, the row does not show.
6. The release list shows "Hidden from apps" on the row of a hidden feed. The review sheet shows a notice when the draft of a hidden feed has a change to write.
7. The block tags of an item are not in this decision. The tree keeps them.

## Alternatives Considered

- Keep the form of the feed, and change only the block tags that the feed has. Rejected by the operator on 2026-10-27: Studio follows MSP 2.0.
- Remove the tags when the user shows the feed again. Rejected by the operator on 2026-10-27, in favor of item 4.
- Offer three conditions, as MSP 2.0 does: listed, hidden from the podcast directories and hidden everywhere. Rejected by the operator on 2026-10-27: the user sees one simple switch.
- Replace each block for one platform when the user changes the switch, as MSP 2.0 does. Rejected: the operator asked for these blocks in "Advanced", so the switch keeps them.
- Show and edit each block for one platform in all feeds. Rejected by the operator on 2026-10-27: Studio does not encourage these blocks.

## Consequences

- A feed that the user hides gets both tags, also when it had only one.
- A feed that the user shows again keeps its tags with the value `no`.
- The publisher feeds and the playlists get the same switch.

## Verification

Mechanical:

- A core test reads a channel with both tags and a block for one platform. A read and a write with no edit give the same bytes.
- A core test writes `no` into both tags where they are, and keeps the block for one platform.
- A core test hides a feed with no block tag. The feed gets both tags.
- A core test examines a hidden feed with capital letters and small letters, with a block for one platform, and with the value `no`.
- The Node tests examine `listing.js`: a hidden feed, a hide, a show, and the change of a block for one platform.
- A Node test examines the mark of a hidden feed in the release list.

Visual:

- The album page of a hidden feed with a block for one platform: the switch shows "Hidden", and "Advanced" lists the platform. In the dark theme, in the light theme and at 390 px.
- The release list shows "Hidden from apps".
- The review sheet of a hidden feed with a change shows the notice.

## Invariants

- The switch never changes a block for one platform.
- A change of the switch never removes a block tag.

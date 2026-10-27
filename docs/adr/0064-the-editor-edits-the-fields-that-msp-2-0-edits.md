# ADR 0064: The Editor Edits The Fields That MSP 2.0 Edits

- Status: Proposed
- Date: 2026-10-27
- Owner: operator

## Context

Studio replaces MSP 2.0. On 2026-10-27, Claude compared MSP 2.0 (`../MSP-2.0`, branch `publisher-parties`, commit `09f3cf9`) with Studio. MSP 2.0 edits some fields that the Studio editor does not show. The tree keeps each of these fields with no change ([ADR 0012](0012-lossless-tree-with-a-typed-model.md) item 4), but the user cannot change them.

The operator decided on 2026-10-27 that `itunes:category` waits for a subsequent decision.

### The Fields

| Field | Parent | Studio model | Studio UI | Corpus feeds |
| --- | --- | --- | --- | --- |
| `itunes:explicit` | channel and item | No | No | 21 of 27 |
| `itunes:owner` | channel | No | No | 17 of 27 |
| `itunes:keywords` | channel | No | No | 7 of 27 |
| `managingEditor` | channel | No | No | 6 of 27 |
| `webMaster` | channel | No | No | 3 of 27 |
| `podcast:locked` | channel | Yes | No | 6 of 27 |
| `podcast:season` | item | No | No | 8 of 27 |
| `podcast:person` | item | Yes | No | Not counted |
| `podcast:transcript` | item | Yes | No | 5 of 27 |
| `podcast:image` | channel and item | Yes | No | Not counted |

The corpus is `crates/core/tests/corpus`. A count gives the number of feeds with one or more of the tag.

### Facts From The Specifications And The Corpus

- Apple lists `itunes:explicit` as a required channel tag. The values are `true` and `false` (`../musicindex-research/sources/podcast-namespace/itunes_reference.md`). The corpus has `true` 492 times, `false` 57 times, `no` 15 times, `No` 2 times, `yes` 1 time and `Yes` 1 time.
- `podcast:locked` is `yes` or `no`, with an optional `owner` email address (`docs/tags/locked.md` of the namespace). The corpus has one `owner=""`.
- `podcast:season` has an integer and an optional `name` (`docs/tags/season.md`). The corpus has the value `1` 19 times, and no other value. MSP 2.0 does not edit the season. It writes `<podcast:season>1</podcast:season>` into each item (`src/utils/xmlGenerator.ts:487`). Wavlake also writes `1`.
- An item `podcast:person` replaces all the channel persons for that item (`docs/tags/person.md`). MSP 2.0 writes the item persons only when the user sets "override" for the track.
- An item can have more than one `podcast:transcript`. Each has a `url` and a `type`, and an optional `language` and `rel="captions"` (`docs/tags/transcript.md`). MSP 2.0 edits one transcript for each track, and calls it lyrics.
- `podcast:image` has `href`, and the optional `alt`, `aspect-ratio`, `width`, `height`, `type` and `purpose` (`docs/tags/image.md`). MSP 2.0 reads the width and the height from the image, and gives a near ratio such as `16/9` (`src/utils/imageMetadata.ts`).
- The corpus has `managingEditor` values that are not email addresses, such as `Kolomona Myer`.

## Decision

### The Core

1. The core models these elements: `itunes:explicit` of the channel and of each item, `itunes:owner` with `itunes:name` and `itunes:email`, `itunes:keywords`, `managingEditor`, `webMaster`, and `podcast:season` with its `name`. A read and a write with no edit give the same bytes.
2. Each new element takes its hosted position ([ADR 0031](0031-a-new-element-takes-its-hosted-position.md)).
3. Studio writes no field that the user did not set. A feed with no `itunes:explicit` gets none from Studio. A new track gets no `podcast:season`.
4. A switch keeps the form of the feed. When the feed has `yes` or `no`, a change writes `yes` or `no`, with the same capital letter. When the tag has no value, a change writes `true` or `false`, as Apple specifies.
5. Studio keeps a text value as the user types it, with no space at its ends ([ADR 0061](0061-a-text-value-has-no-space-at-its-ends.md)). Studio does not refuse a `managingEditor`, a `webMaster` or an owner email that is not an email address.

### The Channel Fields

6. The album page, the playlist page and the publisher page get these fields, as [ADR 0063](0063-one-switch-hides-a-feed-from-apps.md) item 3 does for its switch:
   - "Explicit": one switch for `itunes:explicit`.
   - "Owner": the owner name and the owner email.
   - "Keywords": one text field.
   - "Lock against import": one switch for `podcast:locked`, and the owner email of the lock.
   - "Managing editor" and "Webmaster": one text field each.
7. "Explicit" is in the first section of the page, after "Hide from apps". Apple requires this tag. The other fields of item 6 are on one page that a row of the details section opens. The user does not change them in each visit ([ADR 0028](0028-summary-lines-and-one-open-entry.md)).

### The Track Fields

8. The track page gets these fields:
   - "Explicit": one switch for the item `itunes:explicit`.
   - "Credits": a selection between "Use the album credits" and "Credits for this track". The second selection shows the People control of the album. When the user selects it, the list starts with a copy of the album credits, because the item list replaces the channel list. A change back to "Use the album credits" removes each item `podcast:person`, after a confirmation.
   - "Lyrics": a list of `podcast:transcript` entries. Each entry has the address, the type, the language and a "Captions" check box for `rel="captions"`. The address field uses the file picker ([ADR 0062](0062-a-file-picker-for-each-file-field.md)).
   - "Season": a number field and a name field, in the "Identity" row.

### More Artwork

9. The album page and the track page get a list of `podcast:image` entries. Each entry has the address, the description for accessibility (`alt`), the `purpose` tokens and the dimensions. The address field uses the file picker.
10. When the user selects an image, Studio reads its width, its height and its type from the file. Studio gives the ratio, and selects a near ratio from a list, such as `16/9`. The user can change each value.
11. The `purpose` field offers the tokens of the specification as check boxes: `artwork`, `social`, `canvas`, `banner`, `publisher`, `circular` and `poster`. Studio keeps each other token of the feed.

### Not In This Decision

12. `itunes:category`: deferred by the operator on 2026-10-27.
13. The fields of a live item.

## Alternatives Considered

- Write `<podcast:season>1</podcast:season>` into each new track, as MSP 2.0 does. Rejected: the feed then holds a value that its author did not set (item 3).
- Always write `true` or `false`, as MSP 2.0 does. Rejected: the change of one switch then changes the form of a feed that uses `yes` or `no` (item 4).
- Refuse a `managingEditor` that is not an email address. Rejected: the corpus has such values, and the RSS readers accept them.
- Put all the channel fields in the first section. Not chosen: the first section then gets fields that the user sets one time.
- Edit one lyrics file for each track, as MSP 2.0 does. Rejected: the specification permits more than one, and Studio keeps more than one.

## Consequences

- An MSP 2.0 user can change each field that MSP 2.0 edits. The categories are the only field that stays with no control.
- A feed with no `itunes:explicit` stays with no tag until the user sets the switch. Apple requires the tag, but this decision does not add it without the user.
- Three tasks can do this work: the channel fields, the track fields and the artwork list.

## Verification

Mechanical:

- The corpus tests: each feed gives the same bytes after a read and a write with no edit.
- A core test for each new element: an edit changes only that element, and a new element takes its hosted position.
- A core test changes `itunes:explicit` in a feed with `no` and in a feed with `false`. The value keeps its form.
- A core test sets "Credits for this track", then sets "Use the album credits" again. The item has no `podcast:person`.
- A core test keeps an unknown `purpose` token after an edit of the image.
- The Node tests examine the model module of the credits selection, the `purpose` tokens and the near ratio.

Visual:

- The album page with the explicit switch, and the page of the other channel fields. In the dark theme, in the light theme and at 390 px.
- The track page with the explicit switch, the credits selection, two lyrics entries and the season.
- The artwork list of the album, after the user selects an image.

## Invariants

- Studio writes no field that the user did not set.
- A switch keeps the form of the value in the feed.

# ADR 0250: Studio Corrects The Type Of A Feed With A Bad Or Missing Medium Tag

- Status: Accepted
- Date: 2026-11-23
- Owner: operator
- Accepted: 2026-11-23 by the operator, from drawings in the questionnaire. The page of the feed has the select menu "Type of this feed" for a bad tag and for a missing tag. The list of found feeds names the cause.
- Technical choices: the main session selected the conditions of items 1 to 3 and the values of item 5. It also selected the position of the new tag in item 7 and the texts.
- Changed by: [ADR 0263](0263-overview-is-the-start-page-and-each-page-keeps-the-sequence-of-its-feed.md) item 8, on 2026-11-25. A feed with a corrected type shows in Overview (item 2).
- Changes: [ADR 0090](0090-studio-names-the-feeds-that-it-does-not-edit.md) item 1. A feed with a bad or missing medium tag gets the text and the menu of this record, not only the facts.
- Changes: [ADR 0019](0019-select-a-feed-from-the-host.md) item 5. A candidate with a bad or missing medium tag names the cause.
- Uses: [ADR 0174](0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md) item 6 for a repair through the draft, [ADR 0012](0012-lossless-tree-with-a-typed-model.md) for the unknown elements, [ADR 0008](0008-feed-types-and-feed-identity.md) for the feed types, [ADR 0065](0065-the-core-applies-each-feed-rule.md) for the feed rules in the core.
- Source: [the UI findings of 2026-11-23](../reviews/2026-11-23-ui-findings.md), findings 3 and 10.
- Bar: crosses modules and changes which feeds Studio writes. Items 1 to 3 are in the core, and item 6 is in the API. Items 4, 8 and 9 are in the UI. Without this record, an agent can keep ADR 0090 item 1 and show only the facts of such a feed.

## Context

- The feed of White Triangles had `<podcast:meduim>music</podcast:meduim>` as its only medium tag. The core reads the medium only from a child of the channel with the name `podcast:medium` (`read_text`, `crates/core/src/feed/sync.rs`). Thus the feed had no medium, and Studio put it in "Other feeds" with the text of ADR 0090 item 1.
- The text gave no cause, and Studio gave no correction. The artist had to edit the file on the server with a different tool and then select **Read Again**.
- The lossless tree of ADR 0012 keeps the misspelled element. Thus the core can find it and replace it.
- The core and the API can make a draft and a write for a feed that the editor does not open. Only the UI has no control for it.
- No musicindex rule gives a default medium for a feed with no `podcast:medium` tag (musicindex ADR 0093 R-FEED-1 item 3 leaves the medium filter to each app).

## Decision

### The Core

1. The core gives a feed a **medium problem** when its channel has no `podcast:medium` value that Studio edits. These values are `music`, `musicL` and `publisher`, in uppercase or lowercase letters. One of these conditions must also apply:
    - **Misspelled tag**: a child of the channel in the `podcast` namespace has a local name that is not `medium`. Its distance to `medium` is 1 or 2 edits, in uppercase or lowercase letters. An example is `podcast:meduim`.
    - **Other namespace**: a child of the channel has the local name `medium` in a namespace that is not `podcast`, or in no namespace.
    - **Misspelled value**: `podcast:medium` has a value that is not a known medium, and its distance to `music`, `musicL` or `publisher` is 1 or 2 edits. An example is `msuic`.
    - **No tag**: the channel has no `podcast:medium` and no element of the three conditions above, and one or more items have an `enclosure` with an `audio/` type.
2. A medium problem gives its condition and the full tag as it is in the file, for example `<podcast:meduim>music</podcast:meduim>`. It gives the line number when the core can find it. It also gives the suggested medium: the value of the bad tag when that value is a medium of item 5, else no value.
3. A feed with `podcast:medium` `podcast`, or with a different known medium, has no medium problem. ADR 0090 item 1 stays for that feed.

### The Page Of The Feed

4. The page of a feed with a medium problem shows a notice. The notice replaces the text of ADR 0090 item 1:
    - The title is "Studio does not know the type of this feed".
    - For a misspelled tag, an other namespace or a misspelled value: "Line {n} has {tag}. Studio reads only `<podcast:medium>`." With no line number, the text starts with "This feed has {tag}.".
    - For no tag: "This feed has no `<podcast:medium>` tag."
5. The notice has the select menu "Type of this feed", with the values "Album (music)", "Playlist (musicL)" and "Artist or label page (publisher)". The menu starts on the suggested medium of item 2, or on "Select a type" when there is no suggested medium.
6. The primary button **Review the Change** is disabled until the menu has a type. It makes a repair of ADR 0174 item 6 through the draft of the feed, and then opens Review. The API gives one route for this repair. The route refuses a feed with no medium problem.
7. The repair removes each element of item 1 that caused the problem and writes `<podcast:medium>` with the selected value. For a misspelled tag or an other namespace, the new tag goes in the position of the bad element. For a misspelled value, only the value changes. For no tag, the core puts the tag in the position where its writer puts a new `podcast:medium`. The repair changes no other element.
8. Review shows the change as each other change of a draft. The user writes it with the write button of Review, or discards it. After the write, Studio reads the feed again. The feed then has its type, and Studio shows it in Releases, "Not linked" or the level of its role, and opens it in the editor.
9. A feed with a medium problem and no file host shows the notice of item 4 with no menu. The notice ends with "Connect the folder that holds this file in File hosts to correct it."

### The Found Feeds

10. A candidate of ADR 0019 item 4 also gives the medium problem of item 1. The row of the list of found feeds then shows the cause after the type: "misspelled tag {tag}" for a misspelled tag, an other namespace or a misspelled value, and "no medium tag" for no tag. The user can add the feed. The page of item 4 then offers the correction.

## Alternatives Considered

- A correction of a misspelled tag only. Not chosen by the operator: a feed with no medium tag then needs a different tool again.
- A message only, with the correction on the server. Not chosen by the operator: this is the problem of finding 10.
- A finding in Issues (ADR 0174). Not chosen: Issues is a page of the editor, and the editor does not open such a feed.
- A default medium `music` for a feed with no tag and with audio files. Not chosen: a podcast with no tag also has audio files, and Studio must not change the type of a feed without the user.

## Checks

Mechanical:

- Core tests of the medium problem: `podcast:meduim` with `music` gives a misspelled tag with the suggested medium `music` and the line number. A `medium` element with no namespace gives an other namespace. `podcast:medium` `msuic` gives a misspelled value. A feed with audio enclosures and no medium tag gives no tag with no suggested medium. `podcast:medium` `podcast`, and a feed with no medium tag and no audio enclosure, give no problem. `podcast:mediums` with `music` and a correct `podcast:medium` `music` give no problem.
- A core test of the repair: each condition of item 1 gives a draft with one `podcast:medium` and no bad element. A misspelled tag keeps its position in the channel. Each other element keeps its bytes.
- An API test: the route of item 6 refuses a feed with no medium problem, and accepts the feed of the first core test. The route `GET /connectors/{id}/feeds` gives the medium problem of a candidate.
- A Node test of the UI model: the notice texts of item 4 for each condition and for no line number, the start value of the menu of item 5, the text of item 9 and the row cause of item 10.

Visual, by the main session with screenshots from a walk with `tools/tutorial-walk/`:

- The page of a feed with `<podcast:meduim>music</podcast:meduim>` shows the notice, and the menu starts on "Album (music)".
- Review shows the change of the tag, and after the write the feed opens in the editor.
- The list of found feeds shows "misspelled tag" on that feed.

## Consequences

- An artist with a misspelled or missing medium tag can correct the feed in Studio.
- Studio writes a feed that it did not edit before, but only after a selection of the user and through Review.
- ADR 0090 item 1 applies only to a feed with a known type that Studio does not edit, for example a podcast.

# ADR 0061: A Text Value Has No Space At Its Ends

- Status: Accepted
- Date: 2026-10-27
- Owner: operator

## Context

On 2026-10-27 the operator opened the feed `https://files.heycitizen.xyz/Songs/Albums/TestAlbum/test.xml` in the album editor. The field "Description" started with blank space. In the source, the text of the channel `description` starts and ends with a line break and spaces:

```xml
<description>
    testing stophammer MusicIndex.org
</description>
```

The DeMu feed template gives this format (`de-mu/demu-feed-template`, commit `4e2a1dc` of 2025-08-27). In `feed-with-comments.xml`, the text of the channel `description` is on its own indented lines. In `feed.xml`, it is on one line. Many feeds are made from these files.

The core reads the text of an element as the source has it (`Element::text` in `crates/core/src/xml.rs`). The editor then shows the space. The space comes from the format of the source lines, not from the author of the text.

[ADR 0012](0012-lossless-tree-with-a-typed-model.md) item 3 requires that the writer keeps the source bytes of each element with the same typed value. Thus a read that removes the space must also compare without the space. If it does not, a save with no edit changes the element.

MSP 2.0 reads each text with `trimValues: true` (`src/utils/xmlParser.ts` at commit `09f3cf9`). It writes the full feed again on each save, so it does not keep the source format.

## Decision

1. The core reads each modeled text value with no space at its start and at its end. Space is the XML white space: space, tab, carriage return and line feed. Space in the text does not change.
2. An element that holds only space gives an empty value.
3. The writer removes the same space from a new value before it compares and writes.
4. The writer compares the new value with the source text after the removal of item 1. When the two are equal, the writer keeps the source bytes of the element ([ADR 0012](0012-lossless-tree-with-a-typed-model.md) item 3).
5. When the two are different, the writer writes the new value with no added space. A CDATA section stays a CDATA section.
6. Items 1 to 5 apply to the text of each modeled element of each feed type, in the channel and in each item. The value of an attribute does not change.

## Alternatives Considered

- Keep the text as the source has it, and show the space. Rejected: the editor then shows the format of the source lines as a part of the text.
- Remove the space only in the UI. Rejected: the API and the connected apps of [ADR 0054](0054-live-items-connected-apps-and-show-grants.md) then receive a different value than the editor shows.
- Remove the space on read, and compare the values as they are. Rejected: each save with no edit then changes each element with space at its ends, and ADR 0012 forbids that.
- Write the source format again around an edited value. Rejected: the writer cannot know the format that the author wants, and the result is a guess.

## Consequences

- A feed made from `feed-with-comments.xml` shows its description with no blank space.
- A text value that the user edits loses the format of its source lines. A text value that the user does not edit keeps it.
- A user cannot keep space at the ends of a value. RSS gives no meaning to that space.

## Verification

Mechanical:

- A core test reads a `description` with a line break and spaces before and after its text. The value has no space at its start and at its end. A write with no edit gives the same bytes.
- A core test writes a new value to that element. The element holds the value with no added space.
- A core test writes a value with space at its ends. The element holds the value with no space at its ends.
- A core test writes the same value with space at its ends to an element. The source text of that element is equal to the value after the removal. The writer keeps the source bytes.
- A core test reads an element that holds only space. The value is empty.
- The same tests pass with a CDATA section, and the CDATA section stays.
- A core test reads a copy of the DeMu `feed-with-comments.xml` in `crates/core/tests/corpus`, with its entry in `manifest.json`. A read and a write with no edit give the same bytes.

Visual:

- The album editor with the feed of the operator. The field "Description" starts with "testing". In the dark theme, in the light theme and at 390 px.

## Invariants

- A modeled text value has no space at its start and at its end.
- A text value that the user did not change keeps its source bytes.

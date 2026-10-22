# ADR 0012: A Lossless XML Tree With A Typed Model

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

[ADR 0008](0008-feed-types-and-feed-identity.md) requires that a read and a write with no edit keep the meaning of a feed.
It also requires that the core does not both model an element and pass it through.
[ADR 0007](0007-draft-diff-write-and-announce.md) shows a diff before each write. A diff of a full feed that the writer put in a new sequence is not easy to examine.

## Decision

1. The reader keeps the source as a lossless XML tree. The tree keeps each element, attribute, text, CDATA section, comment and the XML declaration, with the source bytes of each tag.
2. The typed model is a projection of the tree. The core reads the modeled elements from the tree into typed values.
3. The writer starts from the source tree. It does these steps:
   - It keeps each modeled element that has the same typed value as in the source. It keeps the source bytes of that element.
   - It writes a new element in the position of each changed element.
   - It adds each new element after the last element of the same name, or at the end of the parent.
   - It removes each element that the user removed.
4. An element that the core does not model stays in the tree. The writer does not touch it.
5. The reader accepts UTF-8 only. A feed in a different encoding gives an error.

## Alternatives Considered

- A typed model with a list of unknown elements, and a writer that writes the full feed again. Rejected: the writer changes the sequence and the format of the feed, and each diff then shows the full feed.

## Consequences

- A read and a write with no edit give the same bytes. A test can compare the bytes.
- A diff shows only the elements that changed.
- The writer must find the namespace prefix that the source uses for each namespace.

## Invariants

- The writer does not change an element that the user did not change.

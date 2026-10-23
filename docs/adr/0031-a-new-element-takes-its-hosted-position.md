# ADR 0031: A New Element Takes Its Hosted Position

- Status: Accepted
- Date: 2026-10-23
- Owner: operator

## Context

The draft keeps its XML file and changes it at each save ([ADR 0022](0022-the-editor-sends-the-model-with-entry-keys.md)). An empty field removes its element (ADR 0022 item 6).

On 2026-10-23, in the visual check of Phase 5, the operator deleted the album description and typed a new text. The automatic save of the empty field removed `<description>` from the draft. The new text then added a new `<description>`. The writer put it before the first `<item>`. A new channel element goes before the first element that must come after it. The diff removed the element at its first position and added it near the end of the channel.

[ADR 0012](0012-lossless-tree-with-a-typed-model.md) says that the writer changes only the elements that the user changed. The user changed only the text of the description. The draft also keeps the bytes of the hosted file when the draft started (`base_xml`).

## Decision

1. The writer can add a direct child element to the channel or to an item. When the hosted parent has an element of the same type, the new element goes after a sibling. That sibling is the nearest element before it in the hosted parent that the draft parent also has.
2. The hosted parent of an item is the item of the hosted file with the same GUID. A new item has no hosted parent.
3. When the hosted parent has no element of that type, or the draft parent has no element that comes before it, the current rule applies. The element goes before the first element of a type that must come after it.
4. The pipeline gives the hosted file of the draft to the writer at each change of the draft.
5. This rule is for the direct children of the channel and of an item. The children of a nested element, such as `podcast:value`, keep the current rule.

## Alternatives Considered

- The UI removes the element of an empty field only when the field loses the focus. Rejected by the operator: an element that the user removes and saves, and then adds again, continues to move.
- A fixed sequence of the channel elements. Not chosen: published feeds use different sequences, and the hosted file gives the sequence of this feed.

## Consequences

- A field that the user clears and types again gives a diff of its text only.
- An element that the hosted file does not have goes to the same position as before this ADR.

## Invariants

- When the user removes a direct child of the channel or of an item and adds it again, the element is at its hosted position.

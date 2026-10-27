// The rows of "Advanced" for the listing of a feed (ADR 0063, ADR 0065). The
// core applies the listing rule and tells the UI whether the feed is hidden.
// This module has no Preact and no browser state.
// `blocks` is the list of `podcast:block` entries, each with its text `value`
// and its `id`. An entry with no `id` applies to each app. An entry with an
// `id` applies to one platform.

const saysYes = (text) => typeof text === 'string' && text.trim().toLowerCase() === 'yes';

// The blocks for one platform, each with its position in `blocks` and its
// state (ADR 0063 item 5).
export function platformBlocks(model) {
  return (model.blocks || [])
    .map((block, index) => ({ index, id: block.id, hidden: saysYes(block.value), block }))
    .filter((row) => row.block.id != null);
}

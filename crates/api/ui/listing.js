// The listing rule of a feed (ADR 0063). The functions work on the model JSON
// that the editor sends. They have no Preact and no browser state.
// `itunes_block` is the text of the channel `itunes:block`. `blocks` is the
// list of `podcast:block` entries, each with its text `value` and its `id`.
// An entry with no `id` applies to each app. An entry with an `id` applies to
// one platform.

const saysYes = (text) => typeof text === 'string' && text.trim().toLowerCase() === 'yes';
const forEachApp = (block) => block.id == null;

// True when the feed asks apps not to list it: the `itunes:block` or a
// `podcast:block` with no `id` says "yes", in any letter case (ADR 0063
// item 2). A block for one platform does not count.
export function isHidden(model) {
  return saysYes(model.itunes_block)
    || (model.blocks || []).some((block) => forEachApp(block) && saysYes(block.value));
}

// A new model that hides the feed from apps, or shows it again (ADR 0063
// items 3 and 4). To hide, `itunes:block` gets "Yes" and each `podcast:block`
// with no `id` gets "yes". A tag that the feed does not have is added. To
// show, each of these tags that the feed has gets "No" or "no", and no tag is
// added. A block for one platform does not change. A new entry has no `_cid`.
export function withHidden(model, hidden) {
  const blocks = model.blocks || [];
  if (hidden) {
    const hasForEachApp = blocks.some(forEachApp);
    return {
      ...model,
      itunes_block: 'Yes',
      blocks: hasForEachApp
        ? blocks.map((block) => (forEachApp(block) ? { ...block, value: 'yes' } : block))
        : [{ value: 'yes', id: null }, ...blocks],
    };
  }
  return {
    ...model,
    itunes_block: model.itunes_block == null ? null : 'No',
    blocks: blocks.map((block) => (forEachApp(block) ? { ...block, value: 'no' } : block)),
  };
}

// The blocks for one platform, each with its position in `blocks` and its
// state (ADR 0063 item 5).
export function platformBlocks(model) {
  return (model.blocks || [])
    .map((block, index) => ({ index, id: block.id, hidden: saysYes(block.value), block }))
    .filter((row) => !forEachApp(row.block));
}

// A new model in which the block at `index` hides the feed from its
// platform, or shows it again.
export function withPlatformBlock(model, index, hidden) {
  return {
    ...model,
    blocks: (model.blocks || []).map((block, i) => (i === index ? { ...block, value: hidden ? 'yes' : 'no' } : block)),
  };
}

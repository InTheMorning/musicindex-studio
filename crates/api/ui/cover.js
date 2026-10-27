// The cover rule of a channel (ADR 0060 items 2 to 6). The functions work on
// the model JSON that the editor sends. They have no Preact and no browser
// state. `image` is the `itunes:image` of the channel. `rss_image` is the
// RSS `image` of the channel, and its `url` is the address of the image.

const present = (text) => typeof text === 'string' && text !== '';

// The address of the cover: the `itunes:image`, else the RSS `image`, else
// an empty text.
export function coverOf(model) {
  if (present(model.image)) return model.image;
  const rss = model.rss_image && model.rss_image.url;
  return present(rss) ? rss : '';
}

// The tags that the channel has: `rss-only`, `itunes-only`, `both` or `none`.
export function coverForm(model) {
  const itunes = model.image != null;
  const rss = model.rss_image != null;
  if (itunes && rss) return 'both';
  if (rss) return 'rss-only';
  if (itunes) return 'itunes-only';
  return 'none';
}

// The address of the RSS `image` when both tags have an address and the two
// addresses are different. Else `null`.
export function coverConflict(model) {
  const rss = model.rss_image && model.rss_image.url;
  if (!present(model.image) || !present(rss)) return null;
  return model.image === rss ? null : rss;
}

// A new model with `address` in each cover tag that the model has. With
// neither tag, the address goes to `image`. An empty address removes
// `image` and the full `rss_image`.
export function withCover(model, address) {
  const next = { ...model };
  if (address == null || address === '') {
    next.image = null;
    next.rss_image = null;
    return next;
  }
  const form = coverForm(model);
  if (form === 'none' || form === 'itunes-only' || form === 'both') next.image = address;
  if (form === 'rss-only' || form === 'both') next.rss_image = { ...model.rss_image, url: address };
  return next;
}

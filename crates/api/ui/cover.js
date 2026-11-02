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

// The cover of a model as a pair: `image` and the RSS `image` address, with
// `null` for a missing tag.
const coverPair = (model) => [model.image ?? null, (model.rss_image && model.rss_image.url) ?? null];

// A new model with the cover tags of `saved`, the model that the core gave
// back, when the cover of `local` is the same as the cover of `sent`. Else
// the user changed the cover during the save, and the model stays as it is.
// The core keeps the form of the cover (ADR 0065 item 6).
export function withSavedCover(local, sent, saved) {
  const [a, b] = coverPair(local);
  const [c, d] = coverPair(sent);
  if (a !== c || b !== d) return local;
  return { ...local, image: saved.image ?? null, rss_image: saved.rss_image ?? null };
}

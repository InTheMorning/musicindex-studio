// The Nostr keys in the editor (ADR 0074 items 18 to 25). This module has no
// Preact and no browser state. The UI sends the key as the user selected or
// typed it. The core gives the `npub` form (ADR 0065).

// A short form of a key for a summary line: the first 8 characters, an
// ellipsis and the last 4 characters. An empty key gives an empty text.
export function shortKey(npub) {
  const text = String(npub ?? '');
  if (!text) return '';
  return text.length <= 13 ? text : `${text.slice(0, 8)}…${text.slice(-4)}`;
}

// The summary line of a row with a Nostr key.
export function keySummary(name, npub) {
  const short = shortKey(npub);
  if (!short) return 'Not set';
  const label = String(name ?? '').trim();
  return label ? `${label} · ${short}` : short;
}

// The value of the first channel `podcast:txt` entry with the purpose `npub`,
// or `null`.
export function feedKey(model) {
  const entry = (model.txts || []).find((t) => t.purpose === 'npub');
  return entry ? entry.value : null;
}

// The model with the feed key `npub`. It sets the value of the first entry
// with the purpose `npub`, or adds the entry. A `null` key removes the entry.
export function withFeedKey(model, npub) {
  const txts = model.txts || [];
  const index = txts.findIndex((t) => t.purpose === 'npub');
  if (npub === null) {
    return index < 0 ? model : { ...model, txts: txts.filter((_, i) => i !== index) };
  }
  if (index < 0) return { ...model, txts: [...txts, { purpose: 'npub', value: npub }] };
  return { ...model, txts: txts.map((t, i) => (i === index ? { ...t, value: npub } : t)) };
}

// The text for one source of a result of `GET /nostr/profiles`.
export function sourceText(source) {
  if (source.kind === 'collection') return 'In your feeds';
  if (source.kind === 'music_index') {
    return source.feeds === 1 ? 'In 1 musicindex feed' : `In ${source.feeds} musicindex feeds`;
  }
  return 'Nostr search';
}

// The person with the key `npub`. `picked` can give the profile `name` and
// the profile `picture`. The person takes a value only when it is not
// `null`. A `null` key removes the key. This is the selection of the user,
// not a feed rule.
export function withPersonKey(person, npub, picked = {}) {
  const next = { ...person, npub };
  if (picked.name) next.name = picked.name;
  if (picked.picture) next.img = picked.picture;
  return next;
}

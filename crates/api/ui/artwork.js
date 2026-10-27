// The model of the artwork list (ADR 0064 items 10 to 12, ADR 0042). The
// functions have no Preact and no browser state. `editor.js` shows the list
// and loads the image.

// The tokens of `purpose` that the specification names.
export const PURPOSES = ['artwork', 'social', 'canvas', 'banner', 'publisher', 'circular', 'poster'];

// The tokens of a `purpose` value. The separator is one space or more. A
// `null` value gives an empty list.
export function purposeTokens(text) {
  return (text || '').split(/ +/).filter(Boolean);
}

// The new `purpose` value after the user sets or clears one token. A new
// token goes at the end. Each other token keeps its position, also a token
// that `PURPOSES` does not list (ADR 0064 item 12). An empty result is
// `null`.
export function withPurpose(text, token, on) {
  const tokens = purposeTokens(text);
  let next;
  if (on) next = tokens.includes(token) ? tokens : [...tokens, token];
  else next = tokens.filter((t) => t !== token);
  return next.length > 0 ? next.join(' ') : null;
}

// The ratios that the UI offers as the near ratio of an image.
const NEAR_RATIOS = [
  [1, 1], [5, 4], [4, 5], [4, 3], [3, 4], [3, 2], [2, 3], [16, 10], [10, 16], [16, 9], [9, 16],
  [21, 9], [9, 21], [2, 1], [1, 2], [3, 1], [1, 3], [4, 1], [1, 4],
];

// The largest relative difference that still gives a ratio of the list.
const NEAR_LIMIT = 0.02;

function gcd(a, b) {
  return b === 0 ? a : gcd(b, a % b);
}

// The ratio of an image as text, such as `16/9`. An image within 2 % of a
// ratio of the list gives the nearest ratio of the list. The difference is
// relative to the ratio of the list. Each other image gives its width and
// height divided by their greatest common divisor. A width or a height of 0
// or of no value gives `null`. This is a start value for the user.
export function nearRatio(width, height) {
  const w = Number(width);
  const h = Number(height);
  if (!(w > 0) || !(h > 0)) return null;
  let best = null;
  let bestDifference = Infinity;
  for (const [a, b] of NEAR_RATIOS) {
    const difference = Math.abs(w / h - a / b) / (a / b);
    if (difference <= NEAR_LIMIT && difference < bestDifference) {
      best = `${a}/${b}`;
      bestDifference = difference;
    }
  }
  if (best) return best;
  const divisor = gcd(w, h);
  return `${w / divisor}/${h / divisor}`;
}

// The facts of an image as text values, for the fields of an entry. The
// names are the JSON names of the model.
export function imageFacts(width, height, mediaType) {
  const known = Number(width) > 0 && Number(height) > 0;
  return {
    width: Number(width) > 0 ? String(width) : null,
    height: Number(height) > 0 ? String(height) : null,
    media_type: mediaType || null,
    aspect_ratio: known ? nearRatio(width, height) : null,
  };
}

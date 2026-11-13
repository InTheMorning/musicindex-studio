// The parts of a text for the fit check of musicindex ADR 0065 R-FIT-3 item 3.
// A web address can break inside a word, and other text cannot. The UI puts
// each address part in an element with the attribute data-fit-allow-break.

// The end of an address: a space, or a full stop or a comma before a space or
// the end of the text.
const ADDRESS = /https?:\/\/\S+?(?=[.,]?(?:\s|$))/g;

// Gives the parts of the text in sequence. Each part is { text, address }.
export function addressParts(text) {
  const value = String(text ?? '');
  const parts = [];
  let last = 0;
  for (const match of value.matchAll(ADDRESS)) {
    if (match.index > last) parts.push({ text: value.slice(last, match.index), address: false });
    parts.push({ text: match[0], address: true });
    last = match.index + match[0].length;
  }
  if (last < value.length) parts.push({ text: value.slice(last), address: false });
  return parts;
}

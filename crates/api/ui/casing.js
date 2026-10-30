// The title case of a button label (musicindex ADR 0023 R-CASE-1, task 058).
// This module is for the test of the labels. The UI does not call it, and
// each label stays a literal text in its template.

// These words stay in lowercase, except as the first word or the last word.
const SMALL_WORDS = new Set([
  'a', 'an', 'the', 'and', 'but', 'for', 'nor', 'or', 'so', 'yet', 'as', 'at', 'by', 'from',
  'in', 'into', 'of', 'off', 'on', 'onto', 'over', 'to', 'up', 'via', 'with',
]);

// A small word that is part of the verb starts with a capital letter. The key
// is the verb word before it.
const VERB_PAIRS = {
  sign: ['in', 'up'],
  back: ['up'],
  set: ['up'],
  log: ['in'],
  look: ['up'],
};

// A name keeps its form. Each entry is the exact form of a word.
export const NAMES = [
  'nsite', 'MSP', '2.0', 'Amber', 'R2', 'S3', 'SFTP', 'npub', 'nsec', 'Blossom', 'Nostr',
  'musicindex', 'OP3', 'Studio', 'MP3', 'FLAC', 'RSS', 'URL', 'ID', 'NIP-46',
];

const NAME_FORMS = new Map(NAMES.map((name) => [name.toLowerCase(), name]));

// A word with its leading and trailing punctuation set apart.
function splitWord(word) {
  const match = /^([^\p{L}\p{N}]*)(.*?)([^\p{L}\p{N}]*)$/su.exec(word);
  return { lead: match[1], core: match[2], trail: match[3] };
}

// The text in title case (the title style of the Apple HIG).
export function titleCase(text) {
  const words = String(text).split(/(\s+)/);
  const indexes = [];
  words.forEach((word, i) => { if (!/^\s*$/.test(word)) indexes.push(i); });
  const firstLetter = indexes.find((i) => /\p{L}/u.test(words[i]));
  const last = indexes[indexes.length - 1];
  let before = '';
  for (const i of indexes) {
    const { lead, core, trail } = splitWord(words[i]);
    const lower = core.toLowerCase();
    let result = core;
    if (core === '' || !/\p{L}/u.test(core)) {
      result = core;
    } else if (NAME_FORMS.has(lower)) {
      result = NAME_FORMS.get(lower);
    } else if (SMALL_WORDS.has(lower) && i !== firstLetter && i !== last
      && !(VERB_PAIRS[before] || []).includes(lower)) {
      result = lower;
    } else {
      result = core.charAt(0).toUpperCase() + core.slice(1);
    }
    words[i] = lead + result + trail;
    before = lower;
  }
  return words.join('');
}

// True when the text is in title case.
export function isTitleCase(text) {
  return titleCase(text) === text;
}

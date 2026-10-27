// The explicit switch (ADR 0064 item 4, ADR 0065). This module gives the
// display value of the switch only. The core keeps the form of the feed at
// the draft save, so a change of the switch sends `true` or `false`.

// Is the text of `itunes:explicit` an explicit mark? The text `true` or `yes`
// in any letter case is. Each other text, and no text, is not.
export function isExplicit(text) {
  if (typeof text !== 'string') return false;
  const value = text.trim().toLowerCase();
  return value === 'true' || value === 'yes';
}

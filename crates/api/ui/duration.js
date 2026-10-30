// The duration model (ADR 0094, ADR 0042). This module has no Preact and no
// browser state. The Node tests in `ui-tests/duration.test.mjs` examine it.
// The functions are edit aids. The core makes the `itunes:duration` text
// (ADR 0065).

const DIGITS = /^[0-9]+$/;

// The number of seconds in the text of a user, or null. The forms are
// seconds (`261`), `M:SS` and `H:MM:SS`, with the rules of `parse_duration`
// in the core: a part after the first part is from 0 to 59, and a fraction of
// the seconds rounds down.
export function readDuration(text) {
  if (typeof text !== 'string') return null;
  const parts = text.trim().split(':');
  if (parts.length > 3) return null;
  let total = 0;
  for (let index = 0; index < parts.length; index += 1) {
    let whole = parts[index];
    if (index === parts.length - 1 && whole.includes('.')) {
      const dot = whole.indexOf('.');
      const fraction = whole.slice(dot + 1);
      if (fraction !== '' && !DIGITS.test(fraction)) return null;
      whole = whole.slice(0, dot);
    }
    if (!DIGITS.test(whole)) return null;
    const value = Number(whole);
    if (index > 0 && value > 59) return null;
    total = total * (index === 0 ? 1 : 60) + value;
  }
  return Number.isSafeInteger(total) ? total : null;
}

// The text of a number of seconds: `M:SS` below one hour, else `H:MM:SS`.
// Null gives an empty text.
export function durationText(seconds) {
  if (typeof seconds !== 'number' || !Number.isFinite(seconds) || seconds < 0) return '';
  const total = Math.floor(seconds);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const rest = String(total % 60).padStart(2, '0');
  if (hours > 0) return `${hours}:${String(minutes).padStart(2, '0')}:${rest}`;
  return `${minutes}:${rest}`;
}

export const DURATION_ERROR = 'Type the duration as minutes and seconds, for example 4:21.';

// The hint for a duration that the core cannot read.
export function unreadDurationHint(text) {
  return `The feed has the duration “${text}”. Studio cannot read it. Type a duration to replace it.`;
}

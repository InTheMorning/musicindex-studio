// The date and time models (ADR 0093, ADR 0042). This module has no Preact
// and no browser state. The Node tests in `ui-tests/datetime.test.mjs`
// examine it. The time zone is the zone of the process that runs it.

const pad = (n) => String(n).padStart(2, '0');

// The offset text of a number of minutes east of UTC, such as `+01:00`.
function offsetSuffix(minutes) {
  const sign = minutes < 0 ? '-' : '+';
  const abs = Math.abs(minutes);
  return `${sign}${pad(Math.floor(abs / 60))}:${pad(abs % 60)}`;
}

// The local offset, in minutes east of UTC, at the time of a `Date`.
function localOffset(time) {
  return -time.getTimezoneOffset();
}

// RFC 3339 text from the value of a `datetime-local` field, such as
// `2026-10-30T20:00`. The offset is the local offset of that date. An empty
// or incorrect value gives null.
export function localInputToIso(value) {
  if (typeof value !== 'string') return null;
  const match = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})$/.exec(value);
  if (!match) return null;
  const [year, month, day, hour, minute] = match.slice(1).map(Number);
  const time = new Date(year, month - 1, day, hour, minute);
  if (Number.isNaN(time.getTime())) return null;
  // A date such as 2026-02-31 moves to another day. Refuse it.
  if (time.getFullYear() !== year || time.getMonth() !== month - 1 || time.getDate() !== day) return null;
  return `${value}:00${offsetSuffix(localOffset(time))}`;
}

// The value of a `datetime-local` field from ISO 8601 text, in local time.
// A text that is not a time gives an empty text.
export function isoToLocalInput(text) {
  if (!text) return '';
  const time = new Date(text);
  if (Number.isNaN(time.getTime())) return '';
  return `${time.getFullYear()}-${pad(time.getMonth() + 1)}-${pad(time.getDate())}T${pad(time.getHours())}:${pad(time.getMinutes())}`;
}

// The text of an offset in minutes, such as `UTC+01:00`.
export function offsetText(minutes) {
  return `UTC${offsetSuffix(minutes)}`;
}

// The hint below a field: the zone name and the offset. The offset is the
// local offset of `iso`, or of the `Date` `now` when `iso` is null or not a
// time. The caller gives the zone name.
export function zoneHint(iso, zone, now) {
  let time = iso ? new Date(iso) : now;
  if (Number.isNaN(time.getTime())) time = now;
  return `${zone}, ${offsetText(localOffset(time))}`;
}

// The value of a `datetime-local` field for the `Date` `now`, with the
// precision of one minute.
export function nowInput(now) {
  return isoToLocalInput(now.toISOString());
}

// The hint for a date that the core cannot read.
export function unreadDateHint(text) {
  return `The feed has the date “${text}”. Studio cannot read it. Select a time to replace it.`;
}

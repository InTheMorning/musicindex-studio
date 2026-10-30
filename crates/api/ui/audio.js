// The model of the audio section of a track (ADR 0062 items 14 to 16 and 23
// to 29, ADR 0042). The functions have no Preact and no browser state.
// `editor.js` shows the player and does the requests.

// The short format name of a media type, for a badge and for the summary of
// "Properties". An unknown type gives the part after the slash, in capitals.
export function shortType(mediaType) {
  const known = { 'audio/mpeg': 'MP3', 'audio/flac': 'FLAC', 'audio/mp4': 'M4A', 'audio/ogg': 'OGG', 'image/jpeg': 'JPEG' };
  return known[mediaType] || (mediaType || '').split('/').pop().toUpperCase();
}

const MISSING = '—';

// The summary line of "Properties": the size in MB with one decimal, and the
// short type, for example "3.8 MB · MP3". A missing value gives "—" for its
// part. A size that is not a whole number of bytes is missing.
export function propertiesSummary(enclosure) {
  const { length, media_type: mediaType } = enclosure || {};
  const bytes = length === null || length === undefined || String(length).trim() === '' ? NaN : Number(length);
  const size = Number.isFinite(bytes) && bytes >= 0 ? `${(bytes / (1024 * 1024)).toFixed(1)} MB` : MISSING;
  const type = mediaType ? shortType(mediaType) : MISSING;
  return `${size} · ${type}`;
}

const isEmpty = (value) => value === null || value === undefined || String(value).trim() === '';

// Must "Properties" be open? It must when the size or the media type is
// empty, or when one of them has an error. `errors` lists the names of the
// fields with an error: `length` and `media_type`.
export function propertiesOpen(enclosure, errors) {
  const { length, media_type: mediaType } = enclosure || {};
  const list = errors || [];
  return isEmpty(length) || isEmpty(mediaType) || list.includes('length') || list.includes('media_type');
}

const pad = (number) => String(number).padStart(2, '0');

// Must the player read the duration from the file? It must after a selection
// from "This release" or "Hosts", because the library records no duration for
// them. An upload gives false, because the import reads the duration from
// the file (ADR 0062 item 16). `source` is the id of the source of the sheet.
export function needsMetadataRead(source) {
  return source === 'release' || source === 'hosts';
}

// The text below the duration field after a read that failed.
export const DURATION_NOTICE = 'Type the duration of the new file';

// The text of a position in the player, `M:SS` or `H:MM:SS`. A value that is
// not finite gives "—".
export function clockText(seconds) {
  if (typeof seconds !== 'number' || !Number.isFinite(seconds) || seconds < 0) return MISSING;
  const total = Math.floor(seconds);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  return hours > 0 ? `${hours}:${pad(minutes)}:${pad(total % 60)}` : `${minutes}:${pad(total % 60)}`;
}

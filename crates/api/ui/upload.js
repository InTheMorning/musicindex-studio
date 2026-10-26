// The model of an upload in parts (ADR 0059 items 1 to 6, ADR 0042). It has
// no Preact and no browser state. `app.js` does the requests.

// The size of a part, as the start route gives it, in bytes.
export const PART_SIZE = 8 * 1024 * 1024;

// The waits before the three tries again of a failed part, in milliseconds.
export const RETRY_DELAYS = [1000, 2000, 4000];

// The number of parts of a file. An empty file has no part.
export function partCount(size, partSize) {
  return size > 0 ? Math.ceil(size / partSize) : 0;
}

// The numbers of the parts that Studio does not have yet, from small to
// large. `received` is the list of part numbers that Studio has.
export function missingParts(size, partSize, received) {
  const have = new Set(received);
  const missing = [];
  for (let n = 0; n < partCount(size, partSize); n += 1) {
    if (!have.has(n)) missing.push(n);
  }
  return missing;
}

// The bytes of part `n`, as `{ start, end }` for `File.slice(start, end)`.
// The result is `null` for a part outside the file.
export function partRange(n, size, partSize) {
  if (!Number.isInteger(n) || n < 0 || n >= partCount(size, partSize)) return null;
  const start = n * partSize;
  return { start, end: Math.min(size, start + partSize) };
}

// The open upload of the same file: the same name and the same size. When
// more than one upload fits, the one with the most parts wins. The result is
// `null` when no upload fits.
export function findUpload(uploads, name, size) {
  let found = null;
  for (const upload of uploads) {
    if (upload.name !== name || upload.size !== size) continue;
    if (found === null || upload.received.length > found.received.length) found = upload;
  }
  return found;
}

// The bytes of the received parts. The progress of a file is this sum.
export function receivedBytes(size, partSize, received) {
  let total = 0;
  for (const n of new Set(received)) {
    const range = partRange(n, size, partSize);
    if (range) total += range.end - range.start;
  }
  return total;
}

// The whole percent of the progress, from 0 to 100.
export function progressPercent(sent, size) {
  if (size <= 0) return 0;
  return Math.min(100, Math.floor((sent / size) * 100));
}

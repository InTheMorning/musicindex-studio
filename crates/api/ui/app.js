// Music Index Studio UI (ADR 0016). Preact and htm, with no build step.
import { html, render, useState, useEffect, useCallback, useRef, useContext, createContext } from './vendor/htm-preact-standalone.module.js';
import { checkCount } from './feedcheck.js';
import { pageSequence, settingsTabs } from './pages.js';
import { useFieldList, RowList } from './pickers.js';
import { UI_SCALES, SCALE_KEY, normalizeScale } from './uiscale.js';
import { LOCK_HINT, lockChoiceText } from './vaultlock.js';
import { serverFilesRows, sizeText, SERVER_FILES_NOTE } from './assets.js';
import { failedLine } from './failures.js';
import { OPEN_MERGE, MoveAllSheet, AlbumEditor, DraftToolbar, useSheet, DiffView, signPlanned, currentSignerChoice, ANNOUNCE_RELOAD, AnnounceChoice } from './editor.js';
import {
  recipientKey,
  outcomeText, blockedText, operationLine, nameOf, writableIds, firstOpenId, initialStatuses, refreshedStatuses, startWrite,
  finishWrite, nextToWrite, retryStatuses, statusText, statusError, writeLabel, hasWrittenFeed,
} from './recipients.js';
import {
  bootView, signInEvent, normalEmail, authErrorText, unlockMethods, accountName, deleteConfirmation,
  recoveryAccount, addMethodChoices, recoveryErrorText,
  toBase64Url, creationOptions, requestOptions, registrationPayload, assertionPayload,
  prfOutputOf, passkeyBrowserCode, defaultPasskeyName,
  nip46State, nip46ErrorText, isBunkerAddress, qrSource, UNLOCK_AGAIN_TEXT, needsRecentUnlock,
} from './account.js';
import { addChoices, MAKE_TYPES, makeFields, makeTitle, feedFileStart, startChoices } from './addfeeds.js';
import { WELCOME, HOST_STEP, showWelcome, welcomeSteps, pathTexts, hostStepModel, welcomeBody, releaseAdd } from './welcome.js';
import { isLiveItem } from './live.js';
import { appRowLines, pairingPhase, pairingNotice, doneRemovesApp } from './apps.js';
import { CONFIRMS } from './confirms.js';
import { changeTitle, afterDiscard, changeLine } from './people.js';
import { makeSettingsDraft, changeSetting, settingsDraftView, discardSettings, saveSettings, asksSettingsLeave } from './settingsdraft.js';
import { PeoplePage, PersonPage, CheckPage, WalletPage } from './peoplepages.js';
import { OUTSIDE_LIMIT, OUTSIDE_SETTING_LABEL, OUTSIDE_SETTING_HINT, outsideMark, outsideNotice, indexNotice, eachLimited, panelSet, stateMark } from './outsidecheck.js';
import { collectionMark, collectionRow, collectionSectionActions, collectionSheet, collectionAction, CLOSED_EXPORT_HINT, COLLECTIONS_HINT, OPENING_TEXT } from './collectionlist.js';
import { localInputToIso, isoToLocalInput, zoneHint, nowInput, unreadDateHint } from './datetime.js';
import { durationText, readDuration, DURATION_ERROR, unreadDurationHint } from './duration.js';
import { coverOf } from './cover.js';
import { heldMap, mediaSource } from './media.js';
import { SignerConnect } from './signer.js';
import { dropHandlers, watchFileDrags, accountDropResult } from './filedrop.js';
import { useDraft } from './draft.js';
import { roleChoice, fromChoice, roleOptions, ROLE_PLACEHOLDER, OTHER_ROLE, OTHER_HINT, NO_ROLE_HINT } from './roles.js';
import { RETRY_DELAYS, findUpload, missingParts, partCount, partRange, receivedBytes } from './upload.js';
import {
  exportView, exportTitle, EXPORT_DONE, RECOVERY_KEY_TEXT, importedNotice, showsImportNotice, sendingText, CHECKING_TEXT,
  importFacts, importOpenView, importErrorText, importSelectText, IMPORT_HINT, DESKTOP_IMPORT_HINT,
} from './move.js';
import {
  mediumKind, feedKind, FIRST_CLASS_ROLES, roleWord, kindGlow, isEditable, isReadOnly, isPublisher, trackKey, isTrack,
  leftPanel, releasesList, listQuery, queryText, sortLabel, sortEntries, mosaicCells, otherFeedText, feedCover,
  startNav, pushNav, backNav, forwardNav, releasesView, canBack, canForward, setScroll,
  isFeedView, openFeedView, navAtWidth, pageTabs, segmentKey, levelFeedSet } from './shell.js';
import {
  eventsToSign, signErrorText,
  npubOf, addressLines, linesText, invalidConfigField, blossomDefaultName, siteAddress, siteNameFromGuid, isNsec,
} from './signing.js';
import { onTreeKey } from './listkeys.js';

// ---- Access token (ADR 0017) ----
// The API gives the UI address with the token in the fragment. The UI keeps
// it in session storage and removes it from the address bar.

const TOKEN_KEY = 'studio-token';

function takeToken() {
  const match = location.hash.match(/token=([0-9a-f]+)/);
  if (match) {
    try { sessionStorage.setItem(TOKEN_KEY, match[1]); } catch (_) {}
    history.replaceState(null, '', location.pathname + location.search);
    return match[1];
  }
  try { return sessionStorage.getItem(TOKEN_KEY); } catch (_) { return null; }
}

const token = takeToken();

class ApiError extends Error {
  constructor(status, code, message, details) {
    super(message);
    this.status = status;
    this.code = code;
    // The data that the core gives with the error, such as the hops of
    // `bad_move_link` (ADR 0126 item 10).
    this.details = details;
  }
}

// ---- The mode of the server (ADR 0055 items 12 and 14 to 17) ----
// `GET /auth/v1/session` tells the UI which shell serves it. On a server,
// `api()` sends the CSRF token of the session in `X-Studio-CSRF` and sends no
// `Authorization` header. The CSRF token stays in memory only.

let serverMode = false;
let csrfToken = null;
// False for a server with `--hold-assets off` (ADR 0105). `Studio` sets it
// from the status route before it shows the shell.
let holdAssets = true;
// True in the desktop app, which has many collections (ADR 0149 item 5).
let hasCollections = false;
// `App` sets these two functions. `onUnlockNeeded(reason)` gives a promise
// that is true after an unlock and false when the user closes the unlock
// sheet. `reason` is a text for the sheet, or null.
let onUnlockNeeded = null;
let onSessionLost = null;
let unlocking = null;

// Keep the mode and the CSRF token of a session response.
function applySession(session) {
  serverMode = session.mode === 'server';
  hasCollections = session.collections === true;
  csrfToken = serverMode && session.signed_in !== false ? session.csrf || null : null;
}

// The mode of the server for the pages: `server` is true on a server, and
// `holdAssets` is false when the server keeps no audio and no images, and
// `collections` is true when the shell has a list of collections.
export function assetMode() {
  return { server: serverMode, holdAssets: !serverMode || holdAssets, collections: hasCollections };
}

// Ask for an unlock. Parallel requests share one sheet.
function askUnlock(reason = null) {
  if (!onUnlockNeeded) return Promise.resolve(false);
  if (!unlocking) unlocking = onUnlockNeeded(reason).finally(() => { unlocking = null; });
  return unlocking;
}

// Run a request. The answer 423 `vault_locked` opens the unlock sheet (ADR 0142 item 6), and after an unlock the request runs one more time. The answer 401
// `unlock_required` does the same, in both shells, and the sheet says why (ADR 0106 items 4 and 7). If the user
// closes the sheet, the request gives its error. The answer 401
// `session_required` or `session_ended` shows the sign-in page.
async function withUnlock(work) {
  for (let attempt = 0; ; attempt += 1) {
    try {
      return await work();
    } catch (error) {
      if (error.status === 423 && error.code === 'vault_locked' && attempt === 0 && await askUnlock()) continue;
      if (needsRecentUnlock(error) && attempt === 0 && await askUnlock(UNLOCK_AGAIN_TEXT)) continue;
      if (serverMode && error.status === 401 && (error.code === 'session_required' || error.code === 'session_ended') && onSessionLost) {
        onSessionLost();
      }
      throw error;
    }
  }
}

// One request and its answer. `raw` is true for a response that is not JSON,
// such as an image (ADR 0025). It then gives the body as a `Blob` instead of
// parsed JSON. A `File` or `Blob` body, such as the body of an upload (task
// 018d), goes to the route as-is, with no JSON wrapper and no Content-Type
// header of this function. The browser sets the Content-Type header of a
// `Blob` body.
async function exchange(url, headers, { method = 'GET', body, raw = false }) {
  const isBlob = body instanceof Blob;
  if (body !== undefined && !isBlob) headers['Content-Type'] = 'application/json';
  let response;
  try {
    response = await fetch(url, {
      method,
      headers,
      body: body === undefined ? undefined : isBlob ? body : JSON.stringify(body),
    });
  } catch (_) {
    throw new ApiError(0, 'offline', 'Studio does not answer. Make sure that it is still running.');
  }
  if (raw && response.ok) return response.blob();
  const data = await response.json().catch(() => null);
  if (!response.ok) {
    const error = data && data.error ? data.error : { code: 'unknown', message: response.statusText };
    throw new ApiError(response.status, error.code, error.message, error.details);
  }
  return data;
}

// Save the file of a media ticket to this device (ADR 0105 item 15, ADR
// 0107 item 7). The link has the `download` attribute, so the tab does not
// navigate. A navigation stops each request of the page that has not ended,
// and Studio then shows "Studio does not answer". The server gives the file
// name, and `name` is only a start value for the browser.
export function saveTicket(ticket, name = '') {
  const link = document.createElement('a');
  link.href = `/media/download?ticket=${encodeURIComponent(ticket)}`;
  link.download = name;
  link.hidden = true;
  document.body.appendChild(link);
  link.click();
  link.remove();
}

export function api(path, options = {}) {
  // A change can add or remove a file that only Studio holds (ADR 0109
  // item 5), so the next read of that list asks again.
  if (options.method && options.method !== 'GET') { heldCache = null; heldImages.clear(); }
  return withUnlock(() => {
    const headers = serverMode ? { 'X-Studio-CSRF': csrfToken || '' } : { Authorization: `Bearer ${token}` };
    return exchange(`/api/v1${path}`, headers, options);
  });
}

// ---- Uploads in parts (ADR 0059 items 1 to 6) ----

// A part is sent again only after a failure that a new try can fix: no
// answer, or an error of the server.
function canTryAgain(error) {
  return error.status === 0 || error.status >= 500;
}

const wait = (milliseconds) => new Promise((resolve) => { setTimeout(resolve, milliseconds); });

// Send one file to a feed in parts, and finish the upload. The result is the
// facts of the import, as the full upload route gave them before. An open
// upload of the same file continues from its first missing part, also after
// a closed page. A failed part is sent again 3 times, after 1, 2 and 4
// seconds. `onProgress(bytes)` gets the bytes that Studio has, and
// `onRetry(true)` and `onRetry(false)` tell that a part waits for a new try.
export async function sendInParts(feedId, file, { replace = false, onProgress = () => {}, onRetry = () => {} } = {}) {
  const open = await api(`/feeds/${feedId}/uploads`);
  let upload = findUpload(open.filter((u) => u.replace === replace), file.name, file.size);
  if (!upload) {
    const started = await api(`/feeds/${feedId}/uploads`, { method: 'POST', body: { name: file.name, size: file.size, replace } });
    upload = { upload: started.upload, part_size: started.part_size, received: [] };
  }
  let received = upload.received;
  onProgress(receivedBytes(file.size, upload.part_size, received));
  for (const n of missingParts(file.size, upload.part_size, received)) {
    const range = partRange(n, file.size, upload.part_size);
    for (let attempt = 0; ; attempt += 1) {
      try {
        const state = await api(`/uploads/${upload.upload}/parts/${n}`, { method: 'PUT', body: file.slice(range.start, range.end) });
        received = state.received;
        onRetry(false);
        break;
      } catch (error) {
        if (!canTryAgain(error) || attempt >= RETRY_DELAYS.length) throw error;
        onRetry(true);
        await wait(RETRY_DELAYS[attempt]);
      }
    }
    onProgress(receivedBytes(file.size, upload.part_size, received));
  }
  return api(`/uploads/${upload.upload}/finish`, { method: 'POST' });
}

// A request to the auth routes of a server. It sends no access token.
function authCall(path, options = {}) {
  const headers = csrfToken ? { 'X-Studio-CSRF': csrfToken } : {};
  return exchange(`/auth/v1${path}`, headers, { method: 'POST', ...options });
}

// ---- Images and audio from their hosts (ADR 0109) ----
// The browser loads an image from its `https` address. A file that only
// Studio holds is the exception: it has no host copy yet, so the UI reads it
// through the API with the access token and shows it as a `data:` address.
// `media.js` chooses the source.

// The library files that Studio holds and that no host has, as a map from
// their planned address to the file ID. `null` after a change, so that the
// next read asks again.
let heldCache = null;

function loadHeld() {
  if (!heldCache) {
    const entry = { map: null, promise: null };
    entry.promise = api('/media/held').then((answer) => {
      entry.map = heldMap(answer);
      return entry.map;
    }, () => {
      entry.map = new Map();
      return entry.map;
    });
    heldCache = entry;
  }
  return heldCache;
}

// The map of the held files, when it is ready (ADR 0109 item 5).
export function heldFilesNow() {
  return loadHeld().promise;
}

// The map of the held files, or `null` while it loads. `key` makes the hook
// read the map again when it changes, for example the address of a file.
export function useHeldFiles(key) {
  const [map, setMap] = useState(() => (heldCache && heldCache.map) || null);
  useEffect(() => {
    let current = true;
    const entry = loadHeld();
    if (entry.map) setMap(entry.map);
    else entry.promise.then((loaded) => { if (current) setMap(loaded); });
    return () => { current = false; };
  }, [key]);
  return map;
}

// Each held image that the page already read, by its file ID, for the life
// of the page.
const heldImages = new Map();

// Check that the browser can load an image from its host, so that a broken
// image shows the mark and not the broken-image icon.
function probeImage(src) {
  return new Promise((resolve) => {
    const image = new Image();
    image.onload = () => resolve(true);
    image.onerror = () => resolve(false);
    image.src = src;
  });
}

// The image at an address: `{ src, failed }`. `src` is the `https` address,
// a `data:` address of a held file, or `null` while the image loads, when the
// address gives no source (an `http` address) or when the image does not
// load.
export function useImage(url) {
  const [result, setResult] = useState(null);
  useEffect(() => {
    // A new address has no result yet, even when the old address had one.
    setResult(null);
    if (!url) return undefined;
    let current = true;
    const finish = (found) => { if (current) setResult(found); };
    loadHeld().promise.then((held) => {
      const source = mediaSource(url, held);
      if (source.kind === 'direct') {
        probeImage(source.src).then((ok) => finish({ src: ok ? source.src : null, failed: !ok }));
      } else if (source.kind === 'held') {
        const cached = heldImages.get(source.file);
        if (cached) { finish(cached); return; }
        api(`/images?file=${source.file}`, { raw: true }).then((blob) => {
          const reader = new FileReader();
          reader.onload = () => {
            const found = { src: reader.result, failed: false };
            heldImages.set(source.file, found);
            finish(found);
          };
          reader.onerror = () => finish({ src: null, failed: true });
          reader.readAsDataURL(blob);
        }, () => finish({ src: null, failed: true }));
      } else {
        finish({ src: null, failed: false });
      }
    });
    return () => { current = false; };
  }, [url]);
  return result || { src: null, failed: false };
}

// An image at an address, in a box of `className` (ADR 0025). With no
// address, or with an image that Studio cannot read, it shows the current
// mark, tinted with `glow`. For an image that Studio cannot read, the title
// of the box gives its address. `always` keeps the box, with the mark, when
// there is no address: the compact header always shows a cover, but a field
// preview with no address shows nothing (ADR 0032 items 13 and 14). With
// `onOpen`, the box is the content of a button with the name `label` (ADR
// 0070 items 1 to 3).
export function ImageThumb({ url, glow = 'var(--c-feed)', className, always = false, onOpen, label }) {
  const { src, failed } = useImage(url);
  if (!url && !always) return null;
  const box = src
    ? html`<div class=${className}><img src=${src} alt="" /></div>`
    : html`<div class=${className} title=${failed ? url : undefined}>
      <span class="mark" style=${{ '--glow': glow }}></span>
    </div>`;
  if (!onOpen) return box;
  return html`<button type="button" class="cover-button" aria-label=${label} title="Change the artwork" onClick=${onOpen}>${box}</button>`;
}

// Art with its type dot on the bottom right corner (musicindex ADR 0038
// R-DOT-3 and R-DOT-4). The art is the child.
function ArtWithDot({ dot, children }) {
  return html`<span class="art-dot">${children}<span class=${`dot ${dot}`} aria-hidden="true"></span></span>`;
}

// The art of a publisher feed in the left panel (musicindex ADR 0021 R-ART-3
// item 2, musicindex ADR 0038 R-DOT-3). One image shows as it is. More
// images show as a mosaic in the logo shape. No image shows the logo shape
// in `glow`. `dot` puts the type dot on the bottom right corner.
function FeedArt({ art, glow, dot, className = 'panel-art' }) {
  const cells = mosaicCells(art || []);
  return html`<span class=${className} aria-hidden="true">
    <span class=${`art-box${cells.length > 1 ? ' mosaic' : ''}`}>
      ${cells.length === 0
        ? html`<span class="mark" style=${{ '--glow': glow }}></span>`
        : cells.map((url, i) => html`<${ArtCell} key=${i} url=${url} />`)}
    </span>
    ${dot && html`<span class=${`dot ${dot}`}></span>`}
  </span>`;
}

// One image of FeedArt. An image that Studio cannot read leaves the fill.
function ArtCell({ url }) {
  const { src } = useImage(url);
  return src ? html`<img src=${src} alt="" />` : html`<span class="art-empty"></span>`;
}

// ---- Small parts ----

function ThemeToggle() {
  const [theme, setTheme] = useState(document.documentElement.dataset.theme);
  const next = theme === 'dark' ? 'light' : 'dark';
  const toggle = () => {
    document.documentElement.dataset.theme = next;
    try { localStorage.setItem('studio-theme', next); } catch (_) {}
    setTheme(next);
  };
  // The same icons as musicindex.org search.html.
  return html`<button class="theme-toggle" type="button" onClick=${toggle}
    aria-label=${`Use the ${next} theme`} title=${`Use the ${next} theme`}>
    <svg class="icon-sun" aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
      <circle cx="12" cy="12" r="4.2"></circle>
      <path d="M12 2.5v2.2M12 19.3v2.2M2.5 12h2.2M19.3 12h2.2M5.3 5.3l1.6 1.6M17.1 17.1l1.6 1.6M5.3 18.7l1.6-1.6M17.1 6.9l1.6-1.6"></path>
    </svg>
    <svg class="icon-moon" aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round">
      <path d="M20 14.5A8.5 8.5 0 0 1 9.5 4a8.5 8.5 0 1 0 10.5 10.5Z"></path>
    </svg></button>`;
}

// The Music Index lockup, as on musicindex.org, with the product name.
function Brand() {
  return html`<span class="brand"><img src="/ui/assets/musicindex-lockup-horizontal.svg" alt="Music Index" />
    <span class="product">Studio</span></span>`;
}

// One drop target for files (ADR 0125). `handlers` go on its element.
export function useFileDrop({ busy, onFiles, stop = false }) {
  const [over, setOver] = useState(false);
  const handlers = dropHandlers({ isBusy: () => busy, setOver, onFiles, stop });
  return { over, handlers };
}

// The class of a drop area of a sheet (ADR 0125 item 2).
export function dropAreaClass(over, busy) {
  return `import-pick drop-area${busy ? '' : ' drop-target'}${over ? ' drop-over' : ''}`;
}

export function ErrorNotice({ error }) {
  if (!error) return null;
  return html`<div class="notice error" role="alert"><strong>${title(error)}</strong>${error.message}</div>`;
}

// The message of an API error that belongs to one field (ADR 0018). The
// form shows it as red text below that field, not as a notice.
const FIELD_MESSAGES = {
  wrong_key: 'The passphrase is not correct.',
  passphrase_too_short: 'The passphrase is too short.',
  passphrase_too_weak: 'The passphrase is too easy to guess.',
  not_found: 'Studio did not find this file.',
  invalid_path: 'This path is not in the folder.',
  invalid_feed: 'This file is not a feed that Studio can read.',
  invalid_config: 'Give the full path of the folder, for example /home/me/feeds.',
  folder_not_found: 'Studio did not find this folder on this computer.',
  no_public_address: 'Give this file host a public address first, in File hosts.',
  file_exists: 'A file exists at this path already.',
  invalid_address: 'Give a full address. Start it with https:// or http://.',
};

// Split an API error into a field message and an error for a notice.
function splitError(error, codes) {
  if (error && codes.includes(error.code)) return { field: FIELD_MESSAGES[error.code], other: null };
  return { field: null, other: error };
}

function title(error) {
  switch (error.code) {
    case 'wrong_key': return 'The passphrase is not correct.';
    case 'passphrase_too_short': return 'The passphrase is too short.';
    case 'passphrase_too_weak': return 'The passphrase is too easy to guess.';
    case 'invalid_path': return 'That path is not in the folder.';
    case 'not_found': return 'Studio did not find the file.';
    case 'invalid_feed': return 'That file is not a feed that Studio can read.';
    case 'offline': return 'No connection to Studio.';
    // The title of the crate for a field error (musicindex ADR 0052 R-QUERY-8).
    case 'query_field': return 'This search cannot run.';
    case 'index_answer': return 'Music Index gave a bad answer.';
    case 'stream_not_live': return 'Go Live refused.';
    default: return 'Something went wrong.';
  }
}

// A labelled input. `error` is a line of red text below the field (ADR 0018).
// The label and the input sit in one body div, so that a field in a group
// can put the label at the left at a wide width (ADR 0029, task 014c).
export function Field({ id, label, hint, error, children, rootProps, ...input }) {
  const described = [hint && `${id}-hint`, error && `${id}-error`].filter(Boolean).join(' ');
  return html`<div class="field" ...${rootProps}>
    <label for=${id}>${label}</label>
    <div class="field-body">
      ${hint && html`<span class="hint" id=${`${id}-hint`}>${hint}</span>`}
      <input id=${id} aria-invalid=${error ? 'true' : undefined} aria-describedby=${described || undefined} ...${input} />
      ${error && html`<span class="field-error" id=${`${id}-error`}>${error}</span>`}
      ${children}
    </div>
  </div>`;
}

// A field for a date and a time (ADR 0093). It shows local time with the
// native picker. `value` and `onChange` use RFC 3339 text, or null for no
// time. `sourceText` is the date text of the feed that the core cannot read.
// `note` is an optional hint below the zone hint.
export function DateTimeField({ id, label, value, onChange, required = false, sourceText = null, note = null }) {
  const unread = !value && sourceText;
  const hint = unread
    ? unreadDateHint(sourceText)
    : zoneHint(value, Intl.DateTimeFormat().resolvedOptions().timeZone, new Date());
  return html`<${Field} id=${id} label=${label} type="datetime-local" value=${isoToLocalInput(value)}
    required=${required} aria-describedby=${`${id}-zone`}
    onInput=${(e) => {
      // A value that the user has not completed is empty. It must not remove the time.
      if (e.target.validity.badInput) return;
      onChange(localInputToIso(e.target.value));
    }}>
    <span class="hint" id=${`${id}-zone`}>${hint}</span>
    ${note && html`<span class="hint">${note}</span>`}
    <div class="datetime-actions">
      <button class="button" type="button" onClick=${() => onChange(localInputToIso(nowInput(new Date())))}>Now</button>
      ${!required && html`<button class="button" type="button" disabled=${!value} onClick=${() => onChange(null)}>Clear</button>`}
    </div>
  <//>`;
}

// The field "Duration" of a track (ADR 0094). `value` is the number of
// seconds, or null. `sourceText` is the `itunes:duration` text that the core
// cannot read. `notice` is a hint that replaces the other hints. The field
// keeps the typed text while the user types, and shows `value` again when it
// loses the focus.
export function DurationField({ id, value, onChange, sourceText = null }) {
  const [typed, setTyped] = useState(null);
  const [error, setError] = useState(null);
  let hint = 'Minutes and seconds, for example 4:21.';
  if (value == null && sourceText) hint = unreadDurationHint(sourceText);
  return html`<${Field} id=${id} label="Duration" type="text" autocomplete="off"
    value=${typed == null ? durationText(value) : typed} hint=${hint} error=${error}
    onInput=${(e) => {
      const text = e.target.value;
      setTyped(text);
      if (text.trim() === '') { setError(null); onChange(null); return; }
      const seconds = readDuration(text);
      if (seconds == null) { setError(DURATION_ERROR); return; }
      setError(null);
      onChange(seconds);
    }}
    onBlur=${() => {
      // Text that is not a duration stays with its error, so that the user
      // sees it. Correct text shows again in the form of the model.
      if (error) return;
      setTyped(null);
    }} />`;
}

// The eye button of a secret value (ADR 0078 item 3). `name` completes
// "Show …" and "Hide …".
function EyeButton({ shown, name, controls, onToggle }) {
  return html`<button class="button quiet icon-button" type="button" aria-label=${`${shown ? 'Hide' : 'Show'} ${name}`}
    aria-pressed=${shown ? 'true' : 'false'} aria-controls=${controls} onClick=${onToggle}>
    <svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7S2 12 2 12z"></path><circle cx="12" cy="12" r="3"></circle>
      ${shown && html`<path d="M3 3l18 18"></path>`}
    </svg>
  </button>`;
}

// The control of a secret value with its eye button (ADR 0078). The value
// is hidden until the user selects the button. With `rows`, the control is a
// text area for a value with more than one line, and CSS masks its text
// (item 5). Without `rows`, it is a password field while it hides the value.
export function SecretInput({ id, name, rows, ...input }) {
  const [shown, setShown] = useState(false);
  const control = rows
    ? html`<textarea id=${id} rows=${rows} spellcheck="false" class=${shown ? undefined : 'masked'} ...${input}></textarea>`
    : html`<input id=${id} spellcheck="false" ...${input} type=${shown ? 'text' : 'password'} />`;
  return html`<div class="secret">
    ${control}
    <${EyeButton} shown=${shown} name=${name} controls=${id} onToggle=${() => setShown(!shown)} />
  </div>`;
}

// A field of a secret value: a Field with a SecretInput (ADR 0078 item 7).
// `name` completes "Show …" and "Hide …". Its default is the label.
export function SecretField({ id, label, name, hint, error, children, rootProps, ...input }) {
  const described = [hint && `${id}-hint`, error && `${id}-error`].filter(Boolean).join(' ');
  return html`<div class="field" ...${rootProps}>
    <label for=${id}>${label}</label>
    <div class="field-body">
      ${hint && html`<span class="hint" id=${`${id}-hint`}>${hint}</span>`}
      <${SecretInput} id=${id} name=${name || label.replace(/ \(optional\)$/, '').replace(/^./, (c) => c.toLowerCase())}
        aria-invalid=${error ? 'true' : undefined} aria-describedby=${described || undefined} ...${input} />
      ${error && html`<span class="field-error" id=${`${id}-error`}>${error}</span>`}
      ${children}
    </div>
  </div>`;
}

// A display of a value that Studio shows one time only (ADR 0078 item 6).
// It shows dots until the user selects the eye button.
export function SecretText({ id, name, value, className, label }) {
  const [shown, setShown] = useState(false);
  return html`<div class="secret">
    <div id=${id} class=${className} aria-label=${label}>${shown ? value : '•'.repeat(value.length)}</div>
    <${EyeButton} shown=${shown} name=${name} controls=${id} onToggle=${() => setShown(!shown)} />
  </div>`;
}

// One feed that Studio could not read: its name, its site and the reason
// (musicindex ADR 0023 R-MSG-1).
function FailedLine({ failed }) {
  const line = failedLine(failed);
  return html`<li><strong>${line.name}</strong>${line.site && html` · <span class="mono">${line.site}</span>`}
    <br />${line.text}</li>`;
}

// Run an API action with a busy state and an error state.
export function useAction() {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState(null);
  const run = useCallback(async (work) => {
    setBusy(true);
    setError(null);
    try {
      return await work();
    } catch (e) {
      setError(e);
      return undefined;
    } finally {
      setBusy(false);
    }
  }, []);
  return { busy, error, run };
}

// The source of a feed address (ADR 0021). `editor.js` uses this for the
// Feed facts row of the album page.
export function addressSource(source) {
  switch (source) {
    case 'read': return 'Studio read the feed at this address';
    case 'self_link': return 'The self link in the feed';
    case 'folder': return 'The public address of the folder';
    case 'new_feed_url': return 'The new feed address in the feed';
    default: return 'Not known';
  }
}

// Each address that is different from the public address. Studio shows
// them and does not select silently (ADR 0021).
function AddressDifferences({ feed }) {
  const differences = feed.address_differences || [];
  if (differences.length === 0) return null;
  return html`<div class="notice" role="status">
    <strong>Other sources give a different address.</strong>
    <ul class="address-list">
      ${differences.map((d) => html`<li>${addressSource(d.source)}: <span class="mono">${d.url}</span></li>`)}
    </ul>
    Studio uses the public address above. Studio does not change the feed file.
  </div>`;
}

// ---- First-run wizard (ADR 0005) ----

const MIN_CHARS = 12;
const STRENGTH_WORDS = ['Very weak', 'Weak', 'Fair', 'Strong', 'Very strong'];

// Ask the API for the strength when the user stops typing (ADR 0018).
function useStrength(passphrase, signedOut = false) {
  const [strength, setStrength] = useState(null);
  useEffect(() => {
    setStrength(null);
    if (!passphrase) return undefined;
    let current = true;
    const timer = setTimeout(async () => {
      try {
        const body = { passphrase };
        const result = signedOut ? await authCall('/strength', { body }) : await api('/vault/strength', { method: 'POST', body });
        if (current) setStrength(result);
      } catch (_) {
        // The vault still checks the passphrase when the user submits it.
      }
    }, 300);
    return () => { current = false; clearTimeout(timer); };
  }, [passphrase, signedOut]);
  return strength;
}

function Meter({ strength }) {
  if (!strength) return null;
  return html`<div class=${`meter${strength.acceptable ? '' : ' weak'}`} aria-live="polite">
    <div class="bars" aria-hidden="true">
      ${[1, 2, 3, 4].map((n) => html`<span class=${strength.score >= n ? 'on' : ''}></span>`)}
    </div>
    <span class="label">Strength: ${STRENGTH_WORDS[strength.score]} (${strength.score} of 4)</span>
  </div>`;
}

function passphraseError(passphrase, strength) {
  if (!passphrase) return null;
  const chars = [...passphrase].length;
  if (chars < MIN_CHARS) return `Too short: ${chars} of ${MIN_CHARS} characters.`;
  if (strength && !strength.acceptable) {
    return `Too easy to guess.${strength.warning ? ` ${strength.warning}` : ''}`;
  }
  return null;
}

function CreateVault({ onDone }) {
  const [passphrase, setPassphrase] = useState('');
  const [confirm, setConfirm] = useState('');
  const [recovery, setRecovery] = useState(true);
  const { busy, error, run } = useAction();
  const strength = useStrength(passphrase);
  const vaultError = splitError(error, ['passphrase_too_short', 'passphrase_too_weak']);
  const passError = passphraseError(passphrase, strength) || vaultError.field;
  const confirmError = confirm && confirm !== passphrase ? 'The passphrases are not the same.' : null;
  const ready = strength && strength.acceptable && passphrase === confirm;
  const submit = (event) => {
    event.preventDefault();
    if (!ready) return;
    run(async () => {
      const result = await api('/vault', { method: 'POST', body: { passphrase, recovery_key: recovery } });
      onDone(result.recovery_key);
    });
  };
  return html`<form class="form" onSubmit=${submit} noValidate>
    <div><h1>Protect your keys</h1>
      <p>This step keeps a secret, so Studio makes a vault for it first. Only your passphrase opens the vault.</p></div>
    <${SecretField} id="passphrase" label="Passphrase" autocomplete="new-password" required
      value=${passphrase} onInput=${(e) => setPassphrase(e.target.value)} error=${passError}>
      <${Meter} strength=${strength} />
    <//>
    <${SecretField} id="confirm" label="Type the passphrase again" name="the second passphrase" autocomplete="new-password" required
      value=${confirm} onInput=${(e) => setConfirm(e.target.value)} error=${confirmError} />
    <label class="check"><input type="checkbox" checked=${recovery} onChange=${(e) => setRecovery(e.target.checked)} />
      Make a recovery key, in case I forget the passphrase</label>
    <${ErrorNotice} error=${vaultError.other} />
    <div class="actions">
      <button class="button primary" type="submit" disabled=${busy || !ready}>${busy ? 'Making the Vault…' : 'Make the Vault'}</button>
    </div>
  </form>`;
}

function ShowRecoveryKey({ recoveryKey, onDone }) {
  const [kept, setKept] = useState(false);
  return html`<div class="form">
    <div><h1>Write down your recovery key</h1>
      <p>Studio shows this key one time only. With it, you can open the vault if you forget your passphrase.</p></div>
    <${SecretText} id="recovery-key-shown" className="recovery" label="Recovery key" name="recovery key" value=${recoveryKey} />
    <label class="check"><input type="checkbox" checked=${kept} onChange=${(e) => setKept(e.target.checked)} />
      I wrote down the recovery key in a safe place</label>
    <div class="actions"><button class="button" type="button" disabled=${!kept} onClick=${onDone}>Continue</button></div>
  </div>`;
}

// `startRoot` is the folder that the form starts with (ADR 0122 item 3). The
// submit button has the text `submitLabel`.
function LocalHostForm({ onAdded, startRoot = '', submitLabel = 'Add the Folder' }) {
  const [name, setName] = useState('');
  const [root, setRoot] = useState(startRoot);
  const [baseUrl, setBaseUrl] = useState('');
  // The start folder can arrive after the form opens. A folder that the user
  // typed stays.
  useEffect(() => {
    if (startRoot) setRoot((current) => current || startRoot);
  }, [startRoot]);
  const { busy, error, run } = useAction();
  const submit = (event) => {
    event.preventDefault();
    run(async () => {
      const connector = await api('/connectors', {
        method: 'POST',
        // An empty name takes the last part of the folder path.
        body: { kind: 'local', name: name.trim() || root.replace(/\/+$/, '').split('/').pop() || root, root, base_url: baseUrl || null },
      });
      await onAdded(connector);
    });
  };
  const hostError = splitError(error, ['invalid_config', 'folder_not_found']);
  return html`<form class="form" onSubmit=${submit}>
    <${Field} id="host-name" label="Name (optional)" hint="Studio uses the folder name when you leave this empty."
      value=${name} onInput=${(e) => setName(e.target.value)} />
    <${Field} id="host-root" label="Folder" required placeholder="/home/me/feeds"
      hint="The full path of the folder that holds your feed files."
      error=${hostError.field} value=${root} onInput=${(e) => setRoot(e.target.value)} />
    <${Field} id="host-url" label="Public address of the folder (optional)" type="url"
      placeholder="https://music.example.com/feeds"
      hint="When a web server publishes this folder, Studio uses this address for each feed."
      value=${baseUrl} onInput=${(e) => setBaseUrl(e.target.value)} />
    <${ErrorNotice} error=${hostError.other} />
    <div class="actions"><button class="button primary" type="submit" disabled=${busy || !root}>${busy ? 'Adding…' : submitLabel}</button></div>
  </form>`;
}

// The fields of an S3 or R2 bucket (ADR 0046). The secret access key goes
// to the vault. The form never shows it again.
function S3HostForm({ onAdded }) {
  const [values, setValues] = useState({
    name: '', endpoint: '', region: 'auto', bucket: '', prefix: '',
    url_style: 'path', base_url: '', access_key_id: '', secret_access_key: '',
  });
  const [needVault, setNeedVault] = useState(false);
  const [recoveryKey, setRecoveryKey] = useState(null);
  const { busy, error, run } = useAction();
  // A functional update, so that two input events before the next render
  // keep both values, as when a password manager fills the form.
  const set = (key) => (e) => {
    const { value } = e.target;
    setValues((current) => ({ ...current, [key]: value }));
  };
  const send = () => run(async () => {
    let connector;
    try {
      connector = await api('/connectors', {
        method: 'POST',
        // An empty name takes the bucket name.
        body: {
          kind: 's3',
          name: values.name.trim() || values.bucket.trim(),
          endpoint: values.endpoint.trim(),
          region: values.region.trim() || 'auto',
          bucket: values.bucket.trim(),
          prefix: values.prefix.trim(),
          url_style: values.url_style,
          base_url: values.base_url.trim() || null,
          access_key_id: values.access_key_id.trim(),
          secret_access_key: values.secret_access_key,
        },
      });
    } catch (e) {
      if (e.code === 'vault_required') { setNeedVault(true); return; }
      throw e;
    }
    await onAdded(connector);
  });
  const submit = (event) => {
    event.preventDefault();
    send();
  };
  const ready = values.endpoint.trim() && values.bucket.trim() && values.access_key_id.trim() && values.secret_access_key;
  // An invalid address belongs to the endpoint field or to the public
  // address field. The message of the route names the field.
  const addressError = error && error.code === 'invalid_address' ? FIELD_MESSAGES.invalid_address : null;
  const publicAddressError = addressError && /public address/.test(error.message) ? addressError : null;
  const endpointError = addressError && !publicAddressError ? addressError : null;
  let notice = null;
  if (error && !addressError) {
    switch (error.code) {
      case 'no_conditions':
        notice = html`<div class="notice error" role="alert"><strong>Studio cannot write to this provider safely</strong>${error.message}</div>`;
        break;
      case 'host_unauthorized':
        notice = html`<div class="notice error" role="alert"><strong>The provider refused the access key.</strong></div>`;
        break;
      case 'host_unavailable':
        notice = html`<div class="notice error" role="alert"><strong>Studio cannot connect to the endpoint.</strong></div>`;
        break;
      case 'vault_locked':
        notice = html`<div class="notice error" role="alert"><strong>Unlock the vault first.</strong></div>`;
        break;
      default:
        notice = html`<${ErrorNotice} error=${error} />`;
    }
  }

  if (recoveryKey) {
    return html`<${ShowRecoveryKey} recoveryKey=${recoveryKey} onDone=${() => { setRecoveryKey(null); send(); }} />`;
  }
  if (needVault) {
    return html`<${CreateVault} onDone=${(key) => { setNeedVault(false); if (key) setRecoveryKey(key); else send(); }} />`;
  }
  return html`<form class="form" onSubmit=${submit}>
    <${Field} id="s3-name" label="Name (optional)" hint="Studio uses the bucket name when you leave this empty."
      value=${values.name} onInput=${set('name')} />
    <${Field} id="s3-endpoint" label="Endpoint address" type="url" required
      placeholder="https://<account>.r2.cloudflarestorage.com"
      hint="The S3 API address that your provider gives."
      error=${endpointError} value=${values.endpoint} onInput=${set('endpoint')} />
    <${Field} id="s3-region" label="Region"
      hint="R2 uses auto. AWS uses the region of the bucket, such as us-east-1."
      value=${values.region} onInput=${set('region')} />
    <${Field} id="s3-bucket" label="Bucket" required value=${values.bucket} onInput=${set('bucket')} />
    <${Field} id="s3-prefix" label="Folder in the bucket (optional)" placeholder="feeds/"
      value=${values.prefix} onInput=${set('prefix')} />
    <div class="field"><label for="s3-style">Address style</label>
      <div class="field-body">
        <select id="s3-style" class="button" value=${values.url_style} onChange=${set('url_style')}>
          <option value="path">Path style</option>
          <option value="virtual">Virtual host style</option>
        </select>
      </div></div>
    <${Field} id="s3-url" label="Public address of the bucket (optional)" type="url"
      hint="The custom domain that publishes the bucket. R2 needs a custom domain for a public feed."
      error=${publicAddressError} value=${values.base_url} onInput=${set('base_url')} />
    <${SecretField} id="s3-key" label="Access key ID" required autocomplete="off"
      value=${values.access_key_id} onInput=${set('access_key_id')} />
    <${SecretField} id="s3-secret" label="Secret access key" required autocomplete="off"
      hint="Studio keeps this key in your vault."
      value=${values.secret_access_key} onInput=${set('secret_access_key')} />
    ${notice}
    <div class="actions"><button class="button primary" type="submit" disabled=${busy || !ready}>${busy ? 'Testing the Bucket…' : 'Add the Bucket'}</button></div>
  </form>`;
}

// The fields of an SFTP server (ADR 0047). Step 1 asks for the server and
// gets its host key fingerprint. Step 2 shows the fingerprint. Studio adds
// the host only after the user accepts it. After the add, Studio does not
// show a secret again.
function SftpHostForm({ onAdded }) {
  const [values, setValues] = useState({
    name: '', host: '', port: '22', user: '', root: '', base_url: '',
    login: 'key', private_key: '', passphrase: '', password: '',
  });
  const [step, setStep] = useState(1);
  const [fingerprint, setFingerprint] = useState('');
  const [needVault, setNeedVault] = useState(false);
  const [recoveryKey, setRecoveryKey] = useState(null);
  const { busy, error, run } = useAction();
  // A functional update, so that two input events before the next render
  // keep both values, as when a password manager fills the form.
  const set = (key) => (e) => {
    const { value } = e.target;
    setValues((current) => ({ ...current, [key]: value }));
  };
  const hostName = values.host.trim();
  const port = Number(values.port) || 22;
  const connect = () => run(async () => {
    const result = await api('/connectors/fingerprint', { method: 'POST', body: { host: hostName, port } });
    setFingerprint(result.fingerprint);
    setStep(2);
  });
  const send = () => run(async () => {
    let connector;
    try {
      connector = await api('/connectors', {
        method: 'POST',
        // An empty name takes the host name.
        body: {
          kind: 'sftp',
          name: values.name.trim() || hostName,
          host: hostName,
          port,
          user: values.user.trim(),
          root: values.root.trim(),
          base_url: values.base_url.trim() || null,
          login: values.login,
          private_key: values.login === 'key' ? values.private_key : null,
          passphrase: values.login === 'key' && values.passphrase ? values.passphrase : null,
          password: values.login === 'password' ? values.password : null,
          host_key: fingerprint,
        },
      });
    } catch (e) {
      if (e.code === 'vault_required') { setNeedVault(true); return; }
      // A field error belongs to a field of step 1.
      if (['invalid_address', 'invalid_config', 'folder_not_found'].includes(e.code)) setStep(1);
      throw e;
    }
    await onAdded(connector);
  });
  const submit = (event) => {
    event.preventDefault();
    if (step === 1) connect(); else send();
  };
  const secretReady = values.login === 'key' ? values.private_key.trim()
    : values.login === 'password' ? values.password : true;
  const ready = hostName && values.user.trim() && values.root.trim() && secretReady;
  const addressError = error && error.code === 'invalid_address' ? FIELD_MESSAGES.invalid_address : null;
  const rootError = error && error.code === 'invalid_config' ? FIELD_MESSAGES.invalid_config
    : error && error.code === 'folder_not_found' ? 'Studio did not find this folder on the server.' : null;
  let notice = null;
  if (error && !addressError && !rootError) {
    switch (error.code) {
      case 'no_conditions':
        notice = html`<div class="notice error" role="alert"><strong>Studio cannot write to this server safely</strong>${error.message}</div>`;
        break;
      case 'host_unauthorized':
        notice = html`<div class="notice error" role="alert"><strong>The server refused the login.</strong></div>`;
        break;
      case 'host_unavailable':
        notice = html`<div class="notice error" role="alert"><strong>Studio cannot connect to the server.</strong></div>`;
        break;
      case 'vault_locked':
        notice = html`<div class="notice error" role="alert"><strong>Unlock the vault first.</strong></div>`;
        break;
      default:
        notice = html`<${ErrorNotice} error=${error} />`;
    }
  }

  if (recoveryKey) {
    return html`<${ShowRecoveryKey} recoveryKey=${recoveryKey} onDone=${() => { setRecoveryKey(null); send(); }} />`;
  }
  if (needVault) {
    return html`<${CreateVault} onDone=${(key) => { setNeedVault(false); if (key) setRecoveryKey(key); else send(); }} />`;
  }
  if (step === 2) {
    return html`<form class="form" onSubmit=${submit}>
      <p>Studio connected to ${hostName}. Make sure that this is the fingerprint of the server:</p>
      <p class="mono" aria-label="Host key fingerprint">${fingerprint}</p>
      ${notice}
      <div class="actions">
        <button class="button primary" type="submit" disabled=${busy}>${busy ? 'Adding…' : 'Accept and Add the Server'}</button>
        <button class="button" type="button" disabled=${busy} onClick=${() => setStep(1)}>Back</button>
      </div>
    </form>`;
  }
  return html`<form class="form" onSubmit=${submit}>
    <${Field} id="sftp-name" label="Name (optional)" hint="Studio uses the host name when you leave this empty."
      value=${values.name} onInput=${set('name')} />
    <${Field} id="sftp-host" label="Server" required placeholder="sftp.example.com" autocomplete="off"
      value=${values.host} onInput=${set('host')} />
    <${Field} id="sftp-port" label="Port" type="number" min="1" max="65535"
      value=${values.port} onInput=${set('port')} />
    <${Field} id="sftp-user" label="User name" required autocomplete="off"
      value=${values.user} onInput=${set('user')} />
    <${Field} id="sftp-root" label="Folder on the server" required placeholder="/home/me/public_html/feeds"
      hint="The folder that holds your feed files. It is often not the first folder after the login."
      error=${rootError} value=${values.root} onInput=${set('root')} />
    <${Field} id="sftp-url" label="Public address of the folder (optional)" type="url"
      hint="The web address that publishes this folder."
      error=${addressError} value=${values.base_url} onInput=${set('base_url')} />
    <${ChoiceSelect} id="sftp-login" label="Login" value=${values.login}
      options=${[['key', 'Private key'], ['password', 'Password'], ['agent', 'ssh-agent']]}
      onChange=${(login) => setValues((current) => ({ ...current, login }))} />
    ${values.login === 'key' && html`<${SecretField} id="sftp-key" label="Private key" rows="6" required autocomplete="off"
      hint="Paste the text of the key file, such as id_ed25519. Studio keeps it in your vault."
      value=${values.private_key} onInput=${set('private_key')} />`}
    ${values.login === 'key' && html`<${SecretField} id="sftp-passphrase" label="Passphrase of the key (optional)"
      autocomplete="off" value=${values.passphrase} onInput=${set('passphrase')} />`}
    ${values.login === 'password' && html`<${SecretField} id="sftp-password" label="Password" required
      autocomplete="off" hint="Studio keeps it in your vault."
      value=${values.password} onInput=${set('password')} />`}
    ${values.login === 'agent' && html`<p class="hint">Studio uses the keys of the ssh-agent on this computer, and keeps no secret.</p>`}
    ${notice}
    <div class="actions"><button class="button primary" type="submit" disabled=${busy || !ready}>${busy ? 'Connecting…' : 'Connect'}</button></div>
  </form>`;
}

// A text area with a label, a hint and a line of red text, as `Field` does
// for an input.
function AreaField({ id, label, hint, error, rows = 3, ...input }) {
  const described = [hint && `${id}-hint`, error && `${id}-error`].filter(Boolean).join(' ');
  return html`<div class="field">
    <label for=${id}>${label}</label>
    <div class="field-body">
      ${hint && html`<span class="hint" id=${`${id}-hint`}>${hint}</span>`}
      <textarea id=${id} rows=${rows} autocomplete="off" spellcheck="false" aria-invalid=${error ? 'true' : undefined}
        aria-describedby=${described || undefined} ...${input}></textarea>
      ${error && html`<span class="field-error" id=${`${id}-error`}>${error}</span>`}
    </div>
  </div>`;
}

const noticeText = (text) => html`<div class="notice error" role="alert"><strong>${text}</strong></div>`;

// The key of an nsite or Blossom host (ADR 0050 item 1). The key comes from
// a NIP-07 signer or from a pasted private key. Studio reads the lists of the
// key and gives them to `onLists`. The private key stays in this state only.
// It is not in the browser storage or the address.
function useNostrKey(onLists) {
  const [mode, setMode] = useState('signer');
  const [secret, setSecret] = useState('');
  const [key, setKey] = useState(null);
  const [noLists, setNoLists] = useState(false);
  const signer = useAction();
  const lists = useAction();
  const lookup = (text) => lists.run(async () => {
    const result = await api('/connectors/nostr-lists', { method: 'POST', body: { key: text } });
    setKey({ public_key: result.public_key, npub: result.npub });
    setNoLists(result.relays.length === 0 && result.servers.length === 0);
    onLists(result);
  });
  // The key of a signer, from a NIP-07 signer or a NIP-46 signer.
  const takeSignerKey = async (hex) => {
    setKey({ public_key: hex, npub: npubOf(hex) });
    setNoLists(false);
    await lookup(hex);
  };
  // With no NIP-07 signer, the form uses the NIP-46 signer of the
  // workspace, or it opens the connection of one (ADR 0056 item 17).
  const [connecting, setConnecting] = useState(false);
  const askSigner = () => signer.run(async () => {
    if (window.nostr) {
      await takeSignerKey(await window.nostr.getPublicKey());
      return;
    }
    const remote = await api('/signer');
    if (remote.state === 'connected' && remote.pubkey) {
      await takeSignerKey(remote.pubkey);
      return;
    }
    setConnecting(true);
  });
  const connected = (hex) => {
    setConnecting(false);
    signer.run(() => takeSignerKey(hex));
  };
  const choose = (next) => {
    setMode(next);
    setSecret('');
    setKey(null);
    setNoLists(false);
    setConnecting(false);
  };
  const typeSecret = (text) => {
    setSecret(text);
    setKey(null);
  };
  const ready = mode === 'signer' ? !!key : !!secret.trim();
  // The key fields of a new host.
  const fields = () => ({
    signing: mode,
    public_key: key ? key.public_key : null,
    secret_key: mode === 'vault' ? secret : null,
  });
  return { mode, secret, key, noLists, signer, lists, lookup, askSigner, connecting, connected, cancelConnect: () => setConnecting(false), choose, typeSecret, ready, fields, clear: () => setSecret('') };
}

// The "Signing" fieldset and the key of the host.
function SigningStep({ nk, idPrefix, keyError }) {
  return html`<${ChoiceSelect} id=${`${idPrefix}-signing`} label="Signing" value=${nk.mode}
      options=${[['signer', 'My signer'], ['vault', 'Key in the vault']]} onChange=${nk.choose} />
    ${nk.mode === 'signer' && html`<div class="actions">
      <button class="button action" type="button" disabled=${nk.signer.busy} onClick=${nk.askSigner}>
        ${nk.signer.busy ? 'Asking Your Signer…' : 'Use My Signer'}</button>
    </div>`}
    ${nk.mode === 'signer' && nk.connecting && html`<${SignerConnect} onConnected=${nk.connected} onCancel=${nk.cancelConnect} />`}
    ${nk.mode === 'signer' && nk.signer.error && (nk.signer.error.code === 'no_signer'
      ? noticeText(nk.signer.error.message)
      : html`<${ErrorNotice} error=${nk.signer.error} />`)}
    ${nk.mode === 'vault' && html`<${SecretField} id=${`${idPrefix}-secret`} label="Private key of the site"
      autocomplete="off" hint="Paste the nsec of the key. Studio keeps it in your vault."
      error=${keyError} value=${nk.secret} onInput=${(e) => nk.typeSecret(e.target.value)}
      onChange=${(e) => { if (isNsec(e.target.value)) nk.lookup(e.target.value.trim()); }} />`}
    ${nk.key && html`<p class="mono" aria-label="Public key">${nk.key.npub}</p>`}
    ${nk.noLists && html`<p class="hint" role="status">The key has no list. Type the addresses.</p>`}
    <${ErrorNotice} error=${nk.lists.error} />`;
}

// The errors of an nsite or Blossom form. `shown` names the fields of the
// form. An error of a field shows at the field, and the other errors give a
// notice (ADR 0050 item 3).
function nostrFormErrors(error, shown) {
  const out = { fields: {}, notice: null };
  if (!error) return out;
  const at = (field, text) => {
    if (!shown.includes(field)) return false;
    out.fields[field] = text;
    return true;
  };
  switch (error.code) {
    case 'invalid_config': {
      const field = invalidConfigField(error.message);
      const text = `${error.message.charAt(0).toUpperCase()}${error.message.slice(1)}.`;
      if (field && at(field, text)) return out;
      break;
    }
    case 'invalid_address':
      if (at('servers', FIELD_MESSAGES.invalid_address)) return out;
      break;
    case 'key_mismatch':
      if (at('secret_key', 'This private key is not the key of the public key.')) return out;
      break;
    case 'host_unavailable':
      out.notice = noticeText('Studio cannot connect to the relays or the servers.');
      return out;
    case 'vault_locked':
      out.notice = noticeText('Unlock the vault first.');
      return out;
    default:
  }
  out.notice = html`<${ErrorNotice} error=${error} />`;
  return out;
}

// The add step that both forms share. `makeBody` gives the request body.
// `vault_required` opens `CreateVault`, as the S3 form does. After a
// successful add, `nk.clear` removes the private key from the form state.
function useNostrHostAdd(nk, makeBody, onAdded) {
  const [needVault, setNeedVault] = useState(false);
  const [recoveryKey, setRecoveryKey] = useState(null);
  const { busy, error, run } = useAction();
  const send = () => run(async () => {
    const body = makeBody();
    let connector;
    try {
      connector = await api('/connectors', { method: 'POST', body });
    } catch (e) {
      if (e.code === 'vault_required') { setNeedVault(true); return; }
      throw e;
    }
    nk.clear();
    await onAdded(connector);
  });
  const gate = (form) => {
    if (recoveryKey) {
      return html`<${ShowRecoveryKey} recoveryKey=${recoveryKey} onDone=${() => { setRecoveryKey(null); send(); }} />`;
    }
    if (needVault) {
      return html`<${CreateVault} onDone=${(key) => { setNeedVault(false); if (key) setRecoveryKey(key); else send(); }} />`;
    }
    return form;
  };
  return { busy, error, send, gate };
}

// The fields of an nsite site (ADR 0048, ADR 0050).
function NsiteHostForm({ feeds, onAdded }) {
  const [values, setValues] = useState({
    name: '', feed_id: '', d: '', relays: '', servers: '', gateway: 'nsite.lol', title: '',
  });
  // A functional update, so that two input events before the next render
  // keep both values.
  const set = (key) => (e) => {
    const { value } = e.target;
    setValues((current) => ({ ...current, [key]: value }));
  };
  const nk = useNostrKey((result) => setValues((current) => ({
    ...current, relays: linesText(result.relays), servers: linesText(result.servers),
  })));
  const relays = addressLines(values.relays);
  const servers = addressLines(values.servers);
  const add = useNostrHostAdd(nk, () => {
    const d = values.d.trim();
    // With no site name, the core takes the site name from the GUID of the
    // release, as `siteNameFromGuid` does.
    const feed = (feeds || []).find((f) => String(f.id) === values.feed_id);
    const siteName = d || (feed && feed.podcast_guid ? siteNameFromGuid(feed.podcast_guid) : '');
    return {
      kind: 'nsite',
      name: values.name.trim() || (siteName ? `nsite ${siteName}` : 'nsite'),
      ...nk.fields(),
      d: d || null,
      feed_id: values.feed_id ? Number(values.feed_id) : null,
      relays,
      servers,
      gateway: values.gateway.trim() || null,
      title: values.title.trim() || null,
    };
  }, onAdded);
  const submit = (event) => {
    event.preventDefault();
    add.send();
  };
  const ready = nk.ready && relays.length > 0 && servers.length > 0;
  const errors = nostrFormErrors(add.error, ['relays', 'servers', 'gateway', 'd', 'secret_key']);
  return add.gate(html`<form class="form" onSubmit=${submit}>
    <${SigningStep} nk=${nk} idPrefix="nsite" keyError=${errors.fields.secret_key} />
    <${Field} id="nsite-name" label="Name (optional)" hint="Studio uses nsite and the site name when you leave this empty."
      value=${values.name} onInput=${set('name')} />
    <div class="field"><label for="nsite-feed">Release (optional)</label>
      <div class="field-body">
        <select id="nsite-feed" class="button" value=${values.feed_id} onChange=${set('feed_id')}>
          <option value="">No release</option>
          ${(feeds || []).map((f) => html`<option key=${f.id} value=${String(f.id)}>${f.title || f.path}</option>`)}
        </select>
      </div></div>
    <${Field} id="nsite-d" label="Site name" placeholder="From the release" autocomplete="off"
      hint="1 to 13 characters: a to z, 0 to 9 and -. A site that exists keeps its name."
      error=${errors.fields.d} value=${values.d} onInput=${set('d')} />
    <${AreaField} id="nsite-relays" label="Relays" error=${errors.fields.relays}
      value=${values.relays} onInput=${set('relays')} />
    <${AreaField} id="nsite-servers" label="Blossom servers" error=${errors.fields.servers}
      value=${values.servers} onInput=${set('servers')} />
    <${Field} id="nsite-gateway" label="Gateway" autocomplete="off" error=${errors.fields.gateway}
      value=${values.gateway} onInput=${set('gateway')} />
    <${Field} id="nsite-title" label="Title of the site (optional)" value=${values.title} onInput=${set('title')} />
    ${errors.notice}
    <div class="actions"><button class="button primary" type="submit" disabled=${add.busy || !ready}>${add.busy ? 'Adding…' : 'Add the Site'}</button></div>
  </form>`);
}

// The fields of a Blossom host (ADR 0048, ADR 0050).
function BlossomHostForm({ onAdded }) {
  const [values, setValues] = useState({ name: '', servers: '' });
  const set = (key) => (e) => {
    const { value } = e.target;
    setValues((current) => ({ ...current, [key]: value }));
  };
  const nk = useNostrKey((result) => setValues((current) => ({ ...current, servers: linesText(result.servers) })));
  const servers = addressLines(values.servers);
  const add = useNostrHostAdd(nk, () => ({
    kind: 'blossom',
    // An empty name takes the host name of the first server.
    name: values.name.trim() || blossomDefaultName(servers),
    ...nk.fields(),
    servers,
  }), onAdded);
  const submit = (event) => {
    event.preventDefault();
    add.send();
  };
  const ready = nk.ready && servers.length > 0;
  const errors = nostrFormErrors(add.error, ['servers', 'secret_key']);
  return add.gate(html`<form class="form" onSubmit=${submit}>
    <p class="hint">A Blossom file host keeps media only. The feed XML needs a different file host.</p>
    <${SigningStep} nk=${nk} idPrefix="blossom" keyError=${errors.fields.secret_key} />
    <${Field} id="blossom-name" label="Name (optional)" hint="Studio uses the host name of the first server when you leave this empty."
      value=${values.name} onInput=${set('name')} />
    <${AreaField} id="blossom-servers" label="Blossom servers" error=${errors.fields.servers}
      value=${values.servers} onInput=${set('servers')} />
    ${errors.notice}
    <div class="actions"><button class="button primary" type="submit" disabled=${add.busy || !ready}>${add.busy ? 'Adding…' : 'Add the Blossom File Host'}</button></div>
  </form>`);
}

// One setting with a small number of values, as a select menu in one row.
// `options` is a list of [value, label] pairs.
// A `placeholder` adds a first entry with an empty value, for a setting that
// has no value yet.
export function ChoiceSelect({ id, label, value, options, placeholder, hint, onChange, pending = false }) {
  return html`<div class="field">
    <label for=${id}>${label}</label>
    <div class="field-body">
      ${hint && html`<span class="hint" id=${`${id}-hint`}>${hint}</span>`}
      <select id=${id} value=${value} class=${pending ? 'pending' : undefined} aria-describedby=${hint ? `${id}-hint` : undefined} onChange=${(e) => onChange(e.target.value)}>
        ${placeholder && html`<option value="">${placeholder}</option>`}
        ${options.map(([option, text]) => html`<option value=${option}>${text}</option>`)}
      </select>
    </div>
  </div>`;
}

// The role list of the core. The UI reads it one time for each page load and
// keeps it in memory (ADR 0112). A failed read gives an empty list, and the
// menu then offers only "Other role…".
let roleListRequest = null;
export function useRoleList() {
  const [list, setList] = useState(null);
  useEffect(() => {
    if (roleListRequest === null) {
      roleListRequest = api('/roles').then((r) => r.groups, () => { roleListRequest = null; return []; });
    }
    let live = true;
    roleListRequest.then((groups) => { if (live) setList(groups); });
    return () => { live = false; };
  }, []);
  return list;
}

// The role of a credit, or the usual role of a person: a menu of the role
// list, then "Other role…" with a text field and a group menu (ADR 0112).
// `onChange(role, group)` gets the display names, or empty text. The text of
// "Other role" goes to `onChange` when the field loses the focus, as the
// other text fields do. `hint` replaces the hint of an empty role. `status`
// is an optional `{ text, error }` below the menu.
export function RoleMenu({ id, label = 'Role', role, group, hint, status, pending = false, onChange }) {
  const list = useRoleList();
  const [otherPicked, setOtherPicked] = useState(false);
  const [otherText, setOtherText] = useState(role ?? '');
  useEffect(() => { setOtherText(role ?? ''); }, [role]);
  if (list === null) return html`<${ChoiceSelect} id=${id} label=${label} value="" placeholder="Loading the roles…" options=${[]} onChange=${() => {}} />`;
  const choice = otherPicked ? 'other' : roleChoice(role, group, list);
  const options = roleOptions(list);
  const pick = (value) => {
    setOtherPicked(value === 'other');
    if (value === 'other') return onChange('', group ?? '');
    const picked = fromChoice(value, list);
    onChange(picked ? picked.role : '', picked ? picked.group : '');
  };
  return html`<div class="field">
      <label for=${id}>${label}</label>
      <div class="field-body">
        ${choice === '' && (hint || NO_ROLE_HINT) && html`<span class="hint" id=${`${id}-hint`}>${hint || NO_ROLE_HINT}</span>`}
        <select id=${id} value=${choice} class=${pending ? 'pending' : undefined} aria-describedby=${choice === '' ? `${id}-hint` : undefined} onChange=${(e) => pick(e.target.value)}>
          <option value="">${ROLE_PLACEHOLDER}</option>
          ${options.groups.map((g) => html`<optgroup label=${g.label}>
            ${g.options.map(([value, text]) => html`<option value=${value}>${text}</option>`)}</optgroup>`)}
          <option value=${options.other[0]}>${OTHER_ROLE}</option>
        </select>
        ${status && status.text && html`<span class=${status.error ? 'field-error' : 'hint'} role="status">${status.text}</span>`}
      </div>
    </div>
    ${choice === 'other' && html`<${Field} id=${`${id}-other`} label="Other role" hint=${OTHER_HINT} value=${otherText}
      onInput=${(e) => setOtherText(e.target.value)} onChange=${(e) => onChange(e.target.value, group ?? '')} />
      <${ChoiceSelect} id=${`${id}-group`} label="Group" value=${group ?? ''} placeholder="No group"
        options=${list.map((g) => [g.name, g.name])} onChange=${(value) => onChange(role ?? '', value)} />`}`;
}

// The first choice is the type of the host. The form below it follows the
// type (ADR 0045 item 4). A server has no folder host (ADR 0055).
// `start` is the type that the form opens with (ADR 0119 item 6). `showType`
// hides the choice when the sheet shows the rows of step 2 (ADR 0122 item 2).
// `startRoot` and `submitLabel` go to the folder form (ADR 0122 item 3).
function AddHostForm({ feeds, onAdded, start, showType = true, startRoot = '', submitLabel = 'Add the Folder' }) {
  const [type, setType] = useState(start || (serverMode ? 's3' : 'local'));
  return html`<div class="form">
    ${showType && html`<${ChoiceSelect} id="host-type" label="Type of file host" value=${type} onChange=${setType} options=${[
      ...(serverMode ? [] : [['local', 'Folder on this computer']]),
      ['s3', 'S3 or R2 bucket'],
      ['sftp', 'SFTP server'],
      ['nsite', 'nsite site'],
      ['blossom', 'Blossom server'],
    ]} />`}
    ${type === 's3'
      ? html`<${S3HostForm} onAdded=${onAdded} />`
      : type === 'sftp'
        ? html`<${SftpHostForm} onAdded=${onAdded} />`
        : type === 'nsite'
          ? html`<${NsiteHostForm} feeds=${feeds} onAdded=${onAdded} />`
          : type === 'blossom'
            ? html`<${BlossomHostForm} onAdded=${onAdded} />`
            : html`<${LocalHostForm} onAdded=${onAdded} startRoot=${startRoot} submitLabel=${submitLabel} />`}
  </div>`;
}

// One form makes each new feed, as a draft with no file on the host yet
// (ADR 0073 items 5 to 9, ADR 0032 items 6 to 9, ADR 0041 item 2, ADR 0053
// item 2). "Feed file" starts from the title until the user types in it.
// `start` gives optional start values: `author` and `role` (ADR 0119 item 8).
// `onKind` gets the type and the role, for the title of the sheet.
function MakeFeedForm({ connectors, feeds, startType, start = {}, onAdded, onKind }) {
  const [type, setType] = useState(startType);
  const [connectorId, setConnectorId] = useState(connectors[0] ? connectors[0].id : null);
  const [title, setTitle] = useState('');
  const [author, setAuthor] = useState(start.author || '');
  const [role, setRole] = useState(start.role || null);
  const [typedPath, setTypedPath] = useState(null);
  const [from, setFrom] = useState('');
  const [authorTyped, setAuthorTyped] = useState(false);
  const [pathOpen, setPathOpen] = useState(false);
  const { busy, error, run } = useAction();
  const authorTypedRef = useRef(false);
  authorTypedRef.current = authorTyped;
  useEffect(() => { if (onKind) onKind({ type, role }); }, [type, role]);
  const fields = makeFields(type);
  const path = typedPath ?? feedFileStart(title);
  const formError = splitError(error, ['no_public_address', 'file_exists']);
  const starts = startChoices(feeds);
  const showFrom = type === 'album' && starts.length > 1;
  // "Start from" is for the type Album only. Another type clears it.
  const changeType = (value) => {
    setType(value);
    if (value !== 'album') setFrom('');
  };
  // An album gives the start value of "Artist" when the user did not type
  // there. This is an edit aid. The core makes the copy (ADR 0077, ADR 0065).
  const changeFrom = async (value) => {
    setFrom(value);
    if (!value) return;
    try {
      const draft = await api(`/feeds/${value}/draft`);
      if (!authorTypedRef.current && draft.model && draft.model.author) setAuthor(draft.model.author);
    } catch (e) {
      // The start value is optional. The core reports a real error at the save.
    }
  };
  // Open "Feed file" when the API gives an error for it.
  useEffect(() => { if (formError.field) setPathOpen(true); }, [formError.field]);
  const submit = (event) => {
    event.preventDefault();
    const name = title;
    run(async () => {
      const body = { connector_id: connectorId, path, title: name, author: fields.authorLabel === null ? name : author, medium: fields.medium };
      if (fields.role) body.role = role;
      if (showFrom && from) body.from = Number(from);
      onAdded(await api('/feeds/new', { method: 'POST', body }));
    });
  };
  const ready = connectorId !== null && path.trim() && title.trim()
    && (fields.authorLabel === null || author.trim()) && (!fields.role || role !== null);
  return html`<form class="form" onSubmit=${submit}>
    <${ChoiceSelect} id="make-type" label="Type" value=${type} options=${MAKE_TYPES} onChange=${changeType} />
    ${showFrom && html`<${ChoiceSelect} id="make-from" label="Start from" value=${from} options=${starts.slice(1)}
      placeholder=${starts[0][1]} onChange=${changeFrom} />`}
    ${connectors.length > 1 && html`<div class="field"><label for="make-host">File host</label>
      <select id="make-host" class="button" value=${connectorId}
        onChange=${(e) => setConnectorId(Number(e.target.value))}>
        ${connectors.map((c) => html`<option value=${c.id}>${c.name}</option>`)}
      </select></div>`}
    <${Field} id="make-title" label=${fields.titleLabel} required
      value=${title} onInput=${(e) => setTitle(e.target.value)} />
    ${fields.authorLabel !== null && html`<${Field} id="make-author" label=${fields.authorLabel} required
      value=${author} onInput=${(e) => { setAuthorTyped(true); setAuthor(e.target.value); }} />`}
    ${fields.role && html`<${ChoiceSelect} id="make-role" label="Role" value=${role ?? ''} placeholder="Select a role"
      options=${FIRST_CLASS_ROLES.map((r) => [r, roleWord(r)])} onChange=${(value) => setRole(value || null)} />`}
    <div class="group"><details class="disclosure" open=${pathOpen} onToggle=${(e) => setPathOpen(e.currentTarget.open)}>
      <summary class="link-row"><span class="label">Feed file</span>
        <span class="summary">${path || 'From the title'}</span></summary>
      <div class="form">
        <${Field} id="make-path" label="Path" required
          hint="The path of the new feed file in the folder of the file host."
          error=${formError.field} value=${path} onInput=${(e) => setTypedPath(e.target.value)} />
      </div>
    </details></div>
    <${ErrorNotice} error=${formError.other} />
    <div class="actions"><button class="button primary" type="submit" disabled=${busy || !ready}>
      ${busy ? fields.busyLabel : fields.button}</button></div>
  </form>`;
}

// Choose a feed file from the host, or type its path (ADR 0019).
function AddFeedForm({ connectors, onAdded }) {
  const [connectorId, setConnectorId] = useState(connectors[0] ? connectors[0].id : null);
  const [found, setFound] = useState(null);
  const [listError, setListError] = useState(null);
  const [selected, setSelected] = useState('');
  const [typing, setTyping] = useState(false);
  const [path, setPath] = useState('');
  const { busy, error, run } = useAction();

  useEffect(() => {
    if (connectorId === null) return;
    setFound(null);
    setListError(null);
    setSelected('');
    api(`/connectors/${connectorId}/feeds`).then((result) => {
      setFound(result);
      if (result.feeds.every((f) => f.in_collection)) setTyping(true);
    }, setListError);
  }, [connectorId]);

  const target = typing ? path : selected;
  const submit = (event) => {
    event.preventDefault();
    if (!target) return;
    run(async () => {
      const feed = await api('/feeds', { method: 'POST', body: { connector_id: connectorId, path: target } });
      let read;
      try {
        read = await api(`/feeds/${feed.id}/refresh`, { method: 'POST' });
      } catch (e) {
        // Keep no feed that Studio cannot read.
        await api(`/feeds/${feed.id}`, { method: 'DELETE' }).catch(() => {});
        throw e;
      }
      onAdded(read);
    });
  };
  if (connectors.length === 0) {
    return html`<div class="notice">Add a file host first. Studio writes each feed through its file host.</div>`;
  }
  const feedError = splitError(error, ['not_found', 'invalid_path', 'invalid_feed']);
  const available = found ? found.feeds.filter((f) => !f.in_collection) : [];
  return html`<form class="form" onSubmit=${submit}>
    ${connectors.length > 1 && html`<div class="field"><label for="feed-host">File host</label>
      <select id="feed-host" class="button" value=${connectorId}
        onChange=${(e) => setConnectorId(Number(e.target.value))}>
        ${connectors.map((c) => html`<option value=${c.id}>${c.name}</option>`)}
      </select></div>`}
    ${!typing && html`<div class="field">
      <label for="feed-choice">Feeds in this folder</label>
      <div class="field-body">
        ${listError && html`<${ErrorNotice} error=${listError} />`}
        ${!found && !listError && html`<p class="empty" role="status">Looking for feed files…</p>`}
        ${found && found.feeds.length === 0 && html`<p class="empty">Studio found no feed files in this folder.</p>`}
        ${found && found.feeds.length > 0 && html`<select id="feed-choice" value=${selected}
          aria-describedby=${feedError.field ? 'feed-choice-error' : undefined}
          onChange=${(e) => setSelected(e.target.value)}>
          <option value="">Choose a feed file</option>
          ${found.feeds.map((f) => html`<option value=${f.path} disabled=${f.in_collection}>
            ${f.title || f.path} · ${feedKind(f).label}${f.blocked ? ' · blocked' : ''} · ${f.path}${f.in_collection ? ' · in your collection' : ''}</option>`)}
        </select>`}
        ${found && found.truncated && html`<p class="hint">The folder has more files than Studio lists. Type the path of a feed that is not in the list.</p>`}
        ${feedError.field && html`<span class="field-error" id="feed-choice-error">${feedError.field}</span>`}
      </div>
    </div>`}
    ${typing
      ? html`<${Field} id="feed-path" label="Feed file" required placeholder="album/feed.xml"
          hint="The path of the feed file in the folder of the file host."
          error=${feedError.field} value=${path} onInput=${(e) => setPath(e.target.value)} />`
      : html`<button class="button link" type="button" onClick=${() => setTyping(true)}>The Feed Is Not in the List</button>`}
    ${typing && available.length > 0 && html`<button class="button link" type="button" onClick=${() => setTyping(false)}>Select from the List</button>`}
    <${ErrorNotice} error=${feedError.other} />
    <div class="actions"><button class="button primary" type="submit" disabled=${busy || !target}>${busy ? 'Reading the Feed…' : 'Add the Feed'}</button></div>
  </form>`;
}

// ---- Find feeds by address or name (ADR 0020) ----

// Pasted addresses and musicindex search results, with one action at the
// end that uses both (ADR 0020).
function FindFeeds({ onFound }) {
  const [addresses, setAddresses] = useState('');
  const [query, setQuery] = useState('');
  const [hits, setHits] = useState(null);
  const fields = useFieldList({ id: 'search', value: query, setValue: setQuery });
  const [note, setNote] = useState(null);
  const [guids, setGuids] = useState([]);
  const search = useAction();
  const find = useAction();
  const lines = addresses.split('\n').map((l) => l.trim()).filter(Boolean);
  const count = lines.length + guids.length;

  const runSearch = (event) => {
    event.preventDefault();
    if (!query.trim()) return;
    setHits(null);
    setNote(null);
    search.run(async () => {
      const found = await api(`/search?q=${encodeURIComponent(query.trim())}`);
      setHits(found.feeds);
      setNote(found.note);
    });
  };
  const toggle = (guid) => setGuids(guids.includes(guid) ? guids.filter((g) => g !== guid) : [...guids, guid]);
  const runFind = () => find.run(async () => onFound(await api('/discover', { method: 'POST', body: { addresses: lines, guids } })));
  return html`<div class="form">
    <div class="field">
      <label for="addresses">Feed addresses</label>
      <span class="hint" id="addresses-hint">One on each line. An artist feed or a label feed brings all of its releases.</span>
      <textarea id="addresses" rows="3" aria-describedby="addresses-hint" placeholder="https://music.example/feed.xml"
        value=${addresses} onInput=${(e) => setAddresses(e.target.value)}></textarea>
    </div>
    <form class="form" onSubmit=${runSearch} role="search">
      <div class="field">
        <label for="search">Or search Music Index by name</label>
        <div class="inline"><input id="search" type="search" value=${query} ...${fields.input}
          placeholder="Artist or feed name" />
          <button class="button" type="submit" disabled=${search.busy || !query.trim()}>${search.busy ? 'Searching…' : 'Search'}</button></div>
        ${fields.list}
      </div>
      <${ErrorNotice} error=${search.error} />
      ${note && html`<p class="hint" role="status">${note.message}</p>`}
      ${hits && hits.length === 0 && !note && html`<p class="empty">Music Index found no feed with that name.</p>`}
      ${hits && hits.length > 0 && html`<fieldset class="choices"><legend>Select your feeds</legend>
        ${hits.map((h) => html`<label class="choice">
          <input type="checkbox" checked=${guids.includes(h.feed_guid)} onChange=${() => toggle(h.feed_guid)} />
          <span class="dot feed"></span>
          <span class="text"><span class="title">${h.title || h.feed_guid}</span>
            <span class="sub">${h.artist || 'Unknown artist'}</span></span></label>`)}
      </fieldset>`}
    </form>
    <${ErrorNotice} error=${find.error} />
    <div class="actions"><button class="button primary" type="button" disabled=${find.busy || count === 0} onClick=${runFind}>
      ${find.busy ? 'Reading the Feeds…' : count > 0 ? `Find My Feeds (${count})` : 'Find My Feeds'}</button></div>
  </div>`;
}

function hostLabel(group) {
  if (group.service) return `${group.service}: Studio can read these feeds only.`;
  return 'Studio can write these feeds when you connect the folder that holds them.';
}

// The feeds that discovery found, grouped by host (ADR 0020 items 7 to 12).
function FoundFeeds({ found, onAdded, onNothingToAdd }) {
  const addable = found.feeds.filter((f) => !f.in_collection);
  const [chosen, setChosen] = useState(addable.map((f) => f.url));
  const { busy, error, run } = useAction();
  const groups = [];
  for (const feed of found.feeds) {
    const host = feed.host || 'Unknown host';
    let group = groups.find((g) => g.host === host);
    if (!group) groups.push(group = { host, service: feed.service, feeds: [] });
    group.feeds.push(feed);
  }
  const toggle = (url) => setChosen(chosen.includes(url) ? chosen.filter((u) => u !== url) : [...chosen, url]);
  const add = () => run(async () => onAdded(await api('/feeds/import', { method: 'POST', body: { urls: chosen } }), groups));
  return html`<div class="form">
    <p>${found.feeds.length === 0 ? 'Studio found no feeds.' : `Studio found ${found.feeds.length} feed${found.feeds.length > 1 ? 's' : ''}.`}</p>
    ${groups.map((g) => html`<fieldset class="choices">
      <legend>${g.host}</legend>
      <p class="hint">${hostLabel(g)}</p>
      ${g.feeds.map((f) => {
        const kind = feedKind(f);
        return html`<label class=${`choice${f.in_collection ? ' taken' : ''}`}>
          <input type="checkbox" disabled=${f.in_collection} checked=${f.in_collection || chosen.includes(f.url)}
            onChange=${() => toggle(f.url)} />
          <span class=${`dot ${kind.dot}`}></span>
          <span class="text"><span class="title">${f.title || f.url}</span>
            <span class="sub">${kind.label}${f.in_collection ? ' · in your collection' : ''}</span></span></label>`;
      })}
    </fieldset>`)}
    ${found.truncated && html`<p class="hint">Studio stopped at 100 feeds. Add the others in a second search.</p>`}
    ${found.failed.length > 0 && html`<div class="field"><span class="field-error">Studio could not read:</span>
      <ul class="failed">${found.failed.map((f) => html`<${FailedLine} failed=${f} />`)}</ul></div>`}
    <${ErrorNotice} error=${error} />
    ${addable.length === 0 && found.feeds.length > 0
      ? html`<p role="status">Each of these feeds is in your collection.</p>
        ${onNothingToAdd && html`<div class="actions"><button class="button primary" type="button" onClick=${onNothingToAdd}>Open Studio</button></div>`}`
      : addable.length > 0 && html`<div class="actions"><button class="button primary" type="button" disabled=${busy || chosen.length === 0} onClick=${add}>
          ${busy ? 'Adding…' : `Add ${chosen.length} Feed${chosen.length === 1 ? '' : 's'}`}</button></div>`}
  </div>`;
}

// Import MSP 2.0 backup files (ADR 0020). The edit tokens are secrets, so
// Studio makes the vault first when it does not exist.
function MspImport({ onAdded }) {
  const [backups, setBackups] = useState([]);
  const [fileError, setFileError] = useState(null);
  const [needVault, setNeedVault] = useState(false);
  const [recoveryKey, setRecoveryKey] = useState(null);
  const [result, setResult] = useState(null);
  const { busy, error, run } = useAction();

  const [names, setNames] = useState([]);
  const read = async (files) => {
    setFileError(null);
    const parsed = [];
    setNames(files.map((file) => file.name));
    for (const file of files) {
      try {
        parsed.push(JSON.parse(await file.text()));
      } catch (_) {
        setFileError(`${file.name} is not a JSON backup file.`);
      }
    }
    setBackups(parsed);
  };
  const backupDrop = useFileDrop({ busy: false, onFiles: read, stop: true });
  const send = () => run(async () => {
    try {
      const done = await api('/import/msp', { method: 'POST', body: { backups } });
      setResult(done);
    } catch (e) {
      if (e.code === 'vault_required') { setNeedVault(true); return; }
      throw e;
    }
  });

  if (recoveryKey) {
    return html`<${ShowRecoveryKey} recoveryKey=${recoveryKey} onDone=${() => { setRecoveryKey(null); send(); }} />`;
  }
  if (needVault) {
    return html`<${CreateVault} onDone=${(key) => { setNeedVault(false); if (key) setRecoveryKey(key); else send(); }} />`;
  }
  if (result) {
    return html`<div class="form">
      <p>Studio added ${result.added.length} feed${result.added.length === 1 ? '' : 's'} and keeps the edit tokens in your vault.</p>
      ${result.failed.length > 0 && html`<div class="field"><span class="field-error">Studio could not import:</span>
        <ul class="failed">${result.failed.map((f) => html`<${FailedLine} failed=${f} />`)}</ul></div>`}
      <div class="actions"><button class="button" type="button" onClick=${() => onAdded(result)}>Continue</button></div>
    </div>`;
  }
  return html`<div class="form">
    <div class=${`field ${dropAreaClass(backupDrop.over, false)}`} data-drop-many="" ...${backupDrop.handlers}>
      <p class="hint drop-text">Drop the backup files here.</p>
      <span class="hint" id="msp-files-hint">The .json files that MSP 2.0 saved for your hosted feeds. You can select more than one.</span>
      <input id="msp-files" class="visually-hidden" type="file" accept=".json,application/json" multiple aria-describedby="msp-files-hint"
        aria-invalid=${fileError ? 'true' : undefined} onChange=${(e) => read(Array.from(e.target.files))} />
      <label for="msp-files" class="button action">Choose Backup Files</label>
      ${names.length > 0 && html`<p class="hint mono">${names.join(', ')}</p>`}
      ${fileError && html`<span class="field-error">${fileError}</span>`}
    </div>
    <${ErrorNotice} error=${error} />
    <div class="actions"><button class="button primary" type="button" disabled=${busy || backups.length === 0} onClick=${send}>
      ${busy ? 'Importing…' : `Import ${backups.length || ''} Backup${backups.length === 1 ? '' : 's'}`}</button></div>
  </div>`;
}

// ---- The library folder (ADR 0036 items 1 to 6) ----
// Studio keeps one folder for each release: its feed file, its images and
// its audio files. The app asks for this folder once, after the vault and
// before the editor, and runs with no Shell yet.

function LibrarySheet({ defaultRoot, onDone }) {
  const [path, setPath] = useState(defaultRoot || '');
  const [report, setReport] = useState(null);
  const { busy, error, run } = useAction();
  const submit = (event) => {
    event.preventDefault();
    run(async () => {
      setReport(await api('/library', { method: 'PUT', body: { root: path.trim() } }));
    });
  };
  if (report) {
    return html`<main class="center"><div class="card">
      <${Brand} />
      <div><h1>Studio set up your library</h1></div>
      <dl class="facts">
        <dt>Release folders</dt><dd>${report.release_folders}</dd>
        <dt>Feed files</dt><dd>${report.feed_files}</dd>
        <dt>Files moved</dt><dd>${report.moved_files}</dd>
      </dl>
      ${report.errors.length > 0 && html`<div class="notice error" role="alert">
        <strong>Studio could not move each file into the library.</strong>
        <ul class="failed">${report.errors.map((e) => html`<li>Feed ${e.feed_id}: ${e.message}</li>`)}</ul>
      </div>`}
      <div class="actions"><button class="button" type="button" onClick=${onDone}>Continue</button></div>
    </div></main>`;
  }
  return html`<main class="center"><form class="card" onSubmit=${submit}>
    <${Brand} />
    <div><h1>Where Studio keeps your releases</h1>
      <p>Studio keeps one folder for each release here: its feed file, its images and its audio files.</p></div>
    <${Field} id="library-path" label="Folder" required value=${path} onInput=${(e) => setPath(e.target.value)} />
    <${ErrorNotice} error=${error} />
    <div class="actions"><button class="button primary" type="submit" disabled=${busy || !path.trim()}>
      ${busy ? 'Using This Folder…' : 'Use This Folder'}</button></div>
  </form></main>`;
}

// ---- Unlock and no-token screens ----

function Unlock({ onUnlocked }) {
  const [passphrase, setPassphrase] = useState('');
  const { busy, error, run } = useAction();
  const submit = (event) => {
    event.preventDefault();
    run(async () => {
      await api('/vault/unlock', { method: 'POST', body: { passphrase } });
      onUnlocked();
    });
  };
  const unlockError = splitError(error, ['wrong_key']);
  return html`<main class="center"><form class="card" onSubmit=${submit}>
    <${Brand} />
    <div><h1>Unlock Studio</h1><p>Your passphrase opens the vault with the keys of your file hosts.</p></div>
    <${SecretField} id="unlock" label="Passphrase" autocomplete="current-password" required autofocus
      error=${unlockError.field} value=${passphrase} onInput=${(e) => setPassphrase(e.target.value)} />
    <${ErrorNotice} error=${unlockError.other} />
    <div class="actions"><button class="button primary" type="submit" disabled=${busy || !passphrase}>${busy ? 'Unlocking…' : 'Unlock'}</button></div>
  </form></main>`;
}

function NoToken() {
  return html`<main class="center"><div class="card">
    <${Brand} />
    <div><h1>Open Studio from its address</h1>
      <p>For your safety, Studio accepts only the address that it showed when it started. Copy that full address, with the part after “#”, into the browser.</p></div>
  </div></main>`;
}

// ---- Shell ----

// Does a media query apply? This follows each change, such as a resize or a
// turn of the device.
export function useMedia(query) {
  const [matches, setMatches] = useState(() => matchMedia(query).matches);
  useEffect(() => {
    const media = matchMedia(query);
    const onChange = () => setMatches(media.matches);
    media.addEventListener('change', onChange);
    return () => media.removeEventListener('change', onChange);
  }, [query]);
  return matches;
}

// Is the window 760 px wide or less (ADR 0029 item 15)?
function useNarrow() {
  return useMedia('(max-width: 760px)');
}

// The back and forward state of the shell, for each Bar (ADR 0039 item 9).
const NavContext = createContext(null);

// The toolbar of a screen. Each screen gives its own content, such as a
// title or a breadcrumb, after the back control. The root page has no back
// control (ADR 0039 item 11), so `back` is false there. The kind of pointer
// sets the back control: `‹ ›` with a mouse, and one circular `‹` named
// "Back" by touch (musicindex ADR 0027 R-NAV-2, ADR 0031 R-NAV-3,
// R-PROFILE-1).
// A chevron icon of the back control and of the forward control, with the
// path of the Lucide icons "chevron-left" and "chevron-right" (musicindex
// ADR 0022). Its size is the line height of its text (musicindex ADR 0025).
function Chevron({ direction }) {
  return html`<svg class="chevron" aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor"
    stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d=${direction === 'back' ? 'm15 18-6-6 6-6' : 'm9 18 6-6-6-6'}></path>
  </svg>`;
}

// The icon of a fixed destination (ADR 0092 item 5). The elements come from
// the Lucide icons of musicindex-assets (musicindex ADR 0022), in
// icons/lucide/<name>.svg. The icon is decoration, and the label of the row
// gives the accessible name.
const ICONS = {
  cloud: () => html`<path d="M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z" />`,
  upload: () => html`<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="17 8 12 3 7 8" /><line x1="12" x2="12" y1="3" y2="15" />`,
  library: () => html`<path d="m16 6 4 14" /><path d="M12 6v14" /><path d="M8 8v12" /><path d="M4 4v16" />`,
  music: () => html`<path d="M9 18V5l12-2v13" /><circle cx="6" cy="18" r="3" /><circle cx="18" cy="16" r="3" />`,
  tag: () => html`<path d="M12.586 2.586A2 2 0 0 0 11.172 2H4a2 2 0 0 0-2 2v7.172a2 2 0 0 0 .586 1.414l8.704 8.704a2.426 2.426 0 0 0 3.42 0l6.58-6.58a2.426 2.426 0 0 0 0-3.42z" /><circle cx="7.5" cy="7.5" r=".5" fill="currentColor" />`,
  search: () => html`<path d="m21 21-4.34-4.34" /><circle cx="11" cy="11" r="8" />`,
  disc: () => html`<circle cx="12" cy="12" r="10" /><circle cx="12" cy="12" r="2" />`,
  users: () => html`<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" /><circle cx="9" cy="7" r="4" /><path d="M22 21v-2a4 4 0 0 0-3-3.87" /><path d="M16 3.13a4 4 0 0 1 0 7.75" />`,
  wallet: () => html`<path d="M19 7V4a1 1 0 0 0-1-1H5a2 2 0 0 0 0 4h15a1 1 0 0 1 1 1v4h-3a2 2 0 0 0 0 4h3a1 1 0 0 0 1-1v-2a1 1 0 0 0-1-1" /><path d="M3 5v14a2 2 0 0 0 2 2h15a1 1 0 0 0 1-1v-4" />`,
  folder: () => html`<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z" />`,
  'hard-drive': () => html`<path d="M10 16h.01" /><path d="M2.212 11.577a2 2 0 0 0-.212.896V18a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-5.527a2 2 0 0 0-.212-.896L18.55 5.11A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z" /><path d="M21.946 12.013H2.054" /><path d="M6 16h.01" />`,
  settings: () => html`<path d="M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915" /><circle cx="12" cy="12" r="3" />`,
  lock: () => html`<rect width="18" height="11" x="3" y="11" rx="2" ry="2" /><path d="M7 11V7a5 5 0 0 1 10 0v4" />`,
  'lock-open': () => html`<rect width="18" height="11" x="3" y="11" rx="2" ry="2" /><path d="M7 11V7a5 5 0 0 1 9.9-1" />`,
  copy: () => html`<rect width="14" height="14" x="8" y="8" rx="2" ry="2" /><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />`,
  check: () => html`<path d="M20 6 9 17l-5-5" />`,
  zap: () => html`<path d="M15.914 4a1.5 1.5 0 00-2.474-1.561l-9 9A1.5 1.5 0 005.5 14h4.002a.5.5 0 01.471.666L8.086 20a1.5 1.5 0 002.475 1.56l9-9A1.5 1.5 0 0018.5 10h-3.997a.5.5 0 01-.472-.667z" />`,
  globe: () => html`<circle cx="12" cy="12" r="10" /><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" /><path d="M2 12h20" />`,
  play: () => html`<path d="M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z" />`,
  pause: () => html`<rect x="14" y="3" width="5" height="18" rx="1" /><rect x="5" y="3" width="5" height="18" rx="1" />`,
  pencil: () => html`<path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" /><path d="m15 5 4 4" />`,
  'circle-dashed': () => html`<path d="M10.1 2.182a10 10 0 0 1 3.8 0" /><path d="M13.9 21.818a10 10 0 0 1-3.8 0" /><path d="M17.609 3.721a10 10 0 0 1 2.69 2.7" /><path d="M2.182 13.9a10 10 0 0 1 0-3.8" /><path d="M20.279 17.609a10 10 0 0 1-2.7 2.69" /><path d="M21.818 10.1a10 10 0 0 1 0 3.8" /><path d="M3.721 6.391a10 10 0 0 1 2.7-2.69" /><path d="M6.391 20.279a10 10 0 0 1-2.69-2.7" />`,
  'cloud-alert': () => html`<path d="M12 12v4" /><path d="M12 20h.01" /><path d="M17 18h.5a1 1 0 0 0 0-9h-1.79A7 7 0 1 0 7 17.708" />`,
  alert: () => html`<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3" /><path d="M12 9v4" /><path d="M12 17h.01" />`,
};

export function Icon({ name }) {
  return html`<svg class="icon" aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor"
    stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${ICONS[name]?.()}</svg>`;
}

// The results of the check for a change from outside, with the feeds. The
// shell gives them to the left panel (ADR 0165 item 6).
const OutsideContext = createContext({ feeds: [], results: {} });

// The state mark of a row of the left panel, for the feeds that `ids` covers
// (ADR 0165 items 1 to 4). `set` is a list of feeds. It gives nothing for a
// set that is up to date. The text is a tooltip and an accessible name.
function useStateMark(set) {
  const { results } = useContext(OutsideContext);
  return stateMark(set, results);
}

function StateMark({ mark }) {
  if (!mark) return null;
  return html`<span class=${`state-mark ${mark.tone}`} role="img" title=${mark.text} aria-label=${mark.text}>
    <${Icon} name=${mark.icon} /></span>`;
}

// The public address of a feed in the header of its page. The link opens
// the feed in a new tab. A truncated address shows the full address as a
// tooltip. The button copies the address.
// An icon button that copies `text` to the clipboard. It shows "Copied" for
// 2 seconds (task 067d item 1).
export function CopyButton({ text, label }) {
  const [copied, setCopied] = useState(false);
  useEffect(() => {
    if (!copied) return undefined;
    const timer = setTimeout(() => setCopied(false), 2000);
    return () => clearTimeout(timer);
  }, [copied]);
  const copy = () => {
    if (!navigator.clipboard) return;
    navigator.clipboard.writeText(text).then(() => setCopied(true), () => {});
  };
  return html`<button class="button quiet icon-button copy-button" type="button" aria-label=${`Copy ${label}`} onClick=${copy}>
      <${Icon} name=${copied ? 'check' : 'copy'} /></button>
    <span class="visually-hidden" role="status">${copied ? 'Copied' : ''}</span>`;
}

function FeedAddress({ url }) {
  // The tooltip shows only when the link does not show the full address.
  const showTitle = (e) => {
    const link = e.currentTarget;
    link.title = link.scrollWidth > link.clientWidth ? url : '';
  };
  return html`<p class="sub feed-address">
    <a class="mono" href=${url} target="_blank" rel="noopener noreferrer" onMouseEnter=${showTitle} onFocus=${showTitle}>${url}</a>
    <${CopyButton} text=${url} label="the feed address" />
  </p>`;
}

export function Bar({ back = true, children }) {
  const nav = useContext(NavContext);
  return html`<div class="bar">
    ${back && nav && (nav.coarse
      ? html`<button class="button icon-button back" type="button" aria-label="Back"
          disabled=${!nav.canBack} onClick=${nav.back}><${Chevron} direction="back" /></button>`
      : html`<button class="button icon-button" type="button" aria-label="Back" title="Back"
          disabled=${!nav.canBack} onClick=${nav.back}><${Chevron} direction="back" /></button>
        <button class="button icon-button" type="button" aria-label="Forward" title="Forward"
          disabled=${!nav.canForward} onClick=${nav.forward}><${Chevron} direction="forward" /></button>`)}
    ${children}
  </div>`;
}

// A page with pending changes sets its guard while it has them (ADR 0116
// item 10). `guard` gives a promise of true when the page may leave.
export function useLeaveGuard(active, guard) {
  const nav = useContext(NavContext);
  const setGuard = nav ? nav.setLeaveGuard : null;
  const latest = useRef(guard);
  latest.current = guard;
  useEffect(() => {
    if (!active || !setGuard) return undefined;
    setGuard((target) => latest.current(target));
    return () => setGuard(null);
  }, [active, setGuard]);
}

// A feed that the editor cannot open yet: the full hero and the facts, as
// before ADR 0029.
function ReadOnlyFeed({ feed, onChanged, onRemoved }) {
  const { busy, error, run } = useAction();
  const kind = feedKind(feed);
  const refresh = () => run(async () => onChanged(await api(`/feeds/${feed.id}/refresh`, { method: 'POST' })));
  const remove = async () => {
    const name = feed.title || feed.path || 'this feed';
    if (!await askConfirm(CONFIRMS.removeFeed(name))) return;
    run(async () => {
      await api(`/feeds/${feed.id}`, { method: 'DELETE' });
      onRemoved();
    });
  };
  const glow = kindGlow(kind);
  return html`<div>
    <${Bar}><span class="bar-title">${feed.title || 'Untitled feed'}</span><//>
    <section class="hero" style=${{ '--glow': glow }}>
      <${ImageThumb} url=${feedCover(feed)} glow=${glow} className="hero-cover" always />
      <div>
        <span class="eyebrow"><span class=${`dot ${kind.dot}`}></span>${kind.label}${feed.connector_id ? '' : ' · read-only'}</span>
        <h1>${feed.title || 'Untitled feed'}</h1>
        <p class="summary">${feed.public_url || 'This feed has no public address yet.'}</p>
        <p class="hint">${otherFeedText(feed)}</p>
        <${OutsideNotices} feed=${feed} busy=${busy} refresh=${refresh} />
        <div class="actions">
          <button class="button primary" type="button" disabled=${busy || !(feed.connector_id || feed.public_url)} onClick=${refresh}>
            ${busy ? 'Reading…' : 'Read Again'}</button>
          <button class="button destructive" type="button" disabled=${busy} onClick=${remove}>Remove from Collection</button>
        </div>
      </div>
    </section>
    <section class="section">
      <${ErrorNotice} error=${error} />
      <${AddressDifferences} feed=${feed} />
      <h2>Facts</h2>
      <dl class="facts">
        <dt>Address source</dt><dd>${feed.public_url ? addressSource(feed.address_source) : '—'}</dd>
        <dt>Feed GUID</dt><dd class="mono">${feed.podcast_guid || '—'}</dd>
        <dt>File</dt><dd class="mono">${feed.path || 'Read-only. Connect the folder that holds this file in File hosts.'}</dd>
        <dt>Version</dt><dd class="mono">${feed.last_version || 'Not read yet'}</dd>
      </dl>
      ${feed.connector_id && html`<p class="hint">The editor for this feed type comes in a later phase.</p>`}
    </section>
  </div>`;
}

// The names of the pages that are not the album page or a track.
const PAGE_NAMES = { lives: 'Live shows', credits: 'Credits', check: 'Check', settings: 'Settings', recipients: 'Value routes', files: 'Files', tracks: 'Tracks' };

// The keys of a segmented control (musicindex ADR 0051 R-SEG-1 item 7, ADR
// 0157). The selected segment is the one Tab stop of the row, and the arrow
// keys, Home and End move the focus between the segments.
export function onSegmentKey(event) {
  const segments = [...event.currentTarget.querySelectorAll('.segment')];
  const index = segments.indexOf(document.activeElement);
  if (index < 0) return;
  const next = segmentKey(index, segments.length, event.key);
  if (next === null) return;
  event.preventDefault();
  segments[next].focus();
}

// The findings of the check of one feed (ADR 0174). The tab shows the count
// of `check`, and the page of the tab shows its rows. The hook asks for the
// findings with no read when the draft is loaded and after each save (item
// 16). The reads run only in `readAgain`. The UI gives each answer of the API
// as it is, and it keeps no feed rule (ADR 0065).
export function useFeedCheck(feedId, draft) {
  const [check, setCheck] = useState(null);
  const [reading, setReading] = useState(false);
  const [repairing, setRepairing] = useState(null);
  const [error, setError] = useState(null);
  // The message of a refused repair, with the key of its row.
  const [refused, setRefused] = useState(null);
  const turn = useRef(0);
  const running = useRef(false);
  // A repair puts its own answer here. The draft that the repair gives then
  // needs no second request.
  const skip = useRef(false);
  const active = useRef(feedId);
  active.current = feedId;

  useEffect(() => {
    turn.current += 1;
    running.current = false;
    skip.current = false;
    setCheck(null);
    setReading(false);
    setRepairing(null);
    setError(null);
    setRefused(null);
  }, [feedId]);

  const load = async () => {
    const mine = turn.current;
    try {
      const answer = await api(`/feeds/${feedId}/check`);
      if (mine === turn.current && !running.current && active.current === feedId) setCheck(answer);
    } catch (e) {
      // The page shows the error of a read. The count stays as it is.
    }
  };

  // The draft is saved when its status is "saved". The model then changes
  // at each save and at each answer of a route, not at each keystroke.
  const saved = draft.status === 'saved' ? draft.model : null;
  useEffect(() => {
    if (!saved) return;
    if (skip.current) { skip.current = false; return; }
    load();
  }, [feedId, saved]);

  const readAgain = async () => {
    const mine = ++turn.current;
    running.current = true;
    setReading(true);
    setError(null);
    setRefused(null);
    try {
      if (!await draft.flush()) return;
      const answer = await api(`/feeds/${feedId}/check/reads`, { method: 'POST' });
      if (mine === turn.current) setCheck(answer);
    } catch (e) {
      if (mine === turn.current) setError(e);
    } finally {
      if (mine === turn.current) { running.current = false; setReading(false); }
    }
  };

  const repair = async (row) => {
    if (!check || !row.body) return;
    setRepairing(row.key);
    setRefused(null);
    setError(null);
    try {
      if (!await draft.flush()) return;
      const answer = await api(`/feeds/${feedId}/check/repair`, { method: 'POST', body: { ...row.body, revision: check.revision } });
      skip.current = true;
      draft.show(answer.draft);
      setCheck(answer.check);
    } catch (e) {
      if (e.code === 'repair_refused') {
        setRefused({ key: row.key, message: e.message });
      } else if (e.code === 'stale_revision' || e.code === 'no_such_finding' || e.code === 'read_only') {
        await load();
        setRefused({ key: row.key, message: 'The feed changed. Studio checked it again. Select the button again if the finding is still here.' });
      } else {
        setError(e);
      }
    } finally {
      setRepairing(null);
    }
  };

  return { check, count: checkCount(check), reading, repairing, error, refused, readAgain, repair };
}

// The tabs of the page of a feed (ADR 0135), or the `tabs` of a different
// page with `current` and `onSelect`, such as Settings (ADR 0166 item 3). At a
// narrow width they are one select menu, so that no tab starts outside the
// window.
function PageTabs({ feed, view, go, checkCount = 0, tabs: suppliedTabs, current: selected, onSelect, label = 'Pages of this feed' }) {
  const narrow = useNarrow();
  const settings = suppliedTabs !== undefined;
  const tabs = settings
    ? suppliedTabs.map((tab) => ({ page: tab.id, label: tab.label, current: tab.id === selected, pending: tab.pending }))
    : pageTabs(feed, view, checkCount);
  if (tabs.length === 0) return null;
  const open = settings
    ? (page) => onSelect(page)
    : (page) => go({ name: 'feed', id: feed.id, page, ...(view.from ? { from: view.from } : {}) });
  if (narrow) {
    return html`<select class="button page-tab-menu" aria-label=${settings ? label : 'Page of this feed'}
      value=${tabs.find((tab) => tab.current)?.page} onChange=${(e) => open(e.target.value)}>
      ${tabs.map((tab) => html`<option key=${tab.page} value=${tab.page}>${tab.label}${tab.pending ? ' (Not saved)' : ''}</option>`)}
    </select>`;
  }
  return html`<nav class="page-tabs segmented" aria-label=${settings ? label : 'Pages of this feed'} onKeyDown=${onSegmentKey}>
    ${tabs.map((tab) => html`<button key=${tab.page} class="segment" type="button" tabIndex=${tab.current ? 0 : -1}
      aria-current=${tab.current ? 'page' : undefined} onClick=${() => open(tab.page)}>${tab.label}<${SettingsPencil} pending=${tab.pending} /></button>`)}
  </nav>`;
}

// The toolbar title: the feed title on the album page, or the feed title in
// the muted color, then the name of the open page (review of task 014c,
// defect 5).
function Crumb({ feed, view, draft }) {
  const page = view.page || 'album';
  const name = feed.title || 'Untitled feed';
  if (page === 'album') return html`<span class="bar-title">${name}</span>`;
  let label = PAGE_NAMES[page];
  if (page === 'track') {
    const item = draft.model && draft.model.items.find((i) => isTrack(i, view.track));
    label = item ? item.title || 'Untitled track' : 'Track';
  }
  if (page === 'live') {
    const live = draft.model && (draft.model.live_items || []).find((l) => isLiveItem(l, view.track));
    label = live ? live.item.title || 'Untitled show' : 'Live show';
  }
  return html`<span class="bar-title"><span class="muted">${name}</span> › ${label}</span>`;
}

// An editable album: the compact header, not the hero (ADR 0029 item 10).
// The toolbar stays in view at each scroll position (ADR 0029 item 1).
function EditableFeed({ feed, feeds, draft, view, go, adding, onAdding, importing, onImporting, onChanged, onListChanged, onRemoved, connectors, differences, onRescan }) {
  const { busy, error, run } = useAction();
  const kind = feedKind(feed);
  // The core moves a draft with no edits to the file that it reads, so the
  // edits of the user go first, and the draft loads again (ADR 0007 item 6).
  const refresh = () => run(async () => {
    if (!await draft.flush()) return;
    onChanged(await api(`/feeds/${feed.id}/refresh`, { method: 'POST' }));
    await draft.load();
  });
  const glow = kindGlow(kind);
  // A host operation can change the draft XML, and `draft.changed` from
  // the server counts each pending operation (ADR 0036 item 14). After
  // each change of the operations, the shell saves the edits of the user
  // and loads the draft again.
  const opsChanged = async () => {
    if (await draft.flush()) await draft.load();
  };
  // After a write, a discard of the draft and an "Undo" in the review
  // sheet, the Files page loads its list again (task 022b, review of
  // milestone 7a, visual check of task 022).
  const [filesReloadKey, setFilesReloadKey] = useState(0);
  const reloadFiles = () => setFilesReloadKey((n) => n + 1);
  // The Artwork sheet that is open: null, `{ kind: 'feed' }` or
  // `{ kind: 'track', key }` (ADR 0070).
  const [artwork, setArtwork] = useState(null);
  // The findings of the check of this feed (ADR 0174).
  const check = useFeedCheck(feed.id, draft);
  return html`<div>
    <${Bar}>
      <${Crumb} feed=${feed} view=${view} draft=${draft} />
      <${DraftToolbar} key=${feed.id} feedId=${feed.id} feedConnectorId=${feed.connector_id} draft=${draft} onWritten=${onChanged} connectors=${connectors}
        onOpenHosts=${() => go({ name: 'hosts' })}
        onOpsChanged=${() => { opsChanged(); reloadFiles(); }} onOpsCleared=${reloadFiles} />
    <//>
    <div class="feed-head">
      <header class="head">
        <${ImageThumb} url=${draft.model && coverOf(draft.model)} glow=${glow} className="cover" always
          onOpen=${() => setArtwork({ kind: 'feed' })} label=${`Artwork of ${feed.title || 'Untitled feed'}`} />
        <div class="text">
          <span class="eyebrow"><span class=${`dot ${kind.dot}`}></span>${kind.label}${isReadOnly(feed) ? ' · read-only' : ''}</span>
          <h1>${feed.title || 'Untitled feed'}</h1>
          ${feed.public_url
            ? html`<${FeedAddress} url=${feed.public_url} />`
            : html`<p class="sub mono">This feed has no public address yet.</p>`}
        </div>
        ${feed.last_version !== null && html`<button class="button" type="button" disabled=${busy} onClick=${refresh}>
          ${busy ? 'Reading…' : 'Read Again'}</button>`}
      </header>
      <${ErrorNotice} error=${error} />
      ${feed.temporary_expired
        ? html`<div class="notice" role="status">${HOST_STEP.expired}</div>`
        : feed.last_version === null && html`<div class="notice" role="status">
        <strong>The file host has no file for this ${feed.medium === 'publisher' ? 'feed' : 'release'} yet.</strong>
        Open Review and write the draft to make the file.
      </div>`}
      <${AddressDifferences} feed=${feed} />
      <${OutsideNotices} feed=${feed} busy=${busy} refresh=${refresh} />
    </div>
    <${PageTabs} feed=${feed} view=${view} go=${go} checkCount=${check.count} />
    <${AlbumEditor} key=${feed.id} feed=${feed} feeds=${feeds} draft=${draft} view=${view} go=${go}
      adding=${adding} onAdding=${onAdding} importing=${importing} onImporting=${onImporting} onRemoved=${onRemoved}
      connectors=${connectors} onOpsChanged=${opsChanged} filesReloadKey=${filesReloadKey}
      differences=${differences} onRescan=${onRescan} onListChanged=${onListChanged}
      artwork=${artwork} onArtwork=${setArtwork} check=${check} />
  </div>`;
}

// The feed open in the main pane: the editor for an editable album, or the
// read-only view for every other feed (ADR 0029 item 10).
function FeedDetail({ feed, feeds, draft, view, go, adding, onAdding, importing, onImporting, onChanged, onListChanged, onRemoved, connectors, differences, onRescan }) {
  if (isEditable(feed)) {
    return html`<${EditableFeed} feed=${feed} feeds=${feeds} draft=${draft} view=${view} go=${go}
      adding=${adding} onAdding=${onAdding} importing=${importing} onImporting=${onImporting}
      onChanged=${onChanged} onListChanged=${onListChanged} onRemoved=${onRemoved} connectors=${connectors}
      differences=${differences} onRescan=${onRescan} />`;
  }
  return html`<${ReadOnlyFeed} feed=${feed} onChanged=${onChanged} onRemoved=${onRemoved} />`;
}

// The feed files in a folder that are not in the collection. Studio shows
// them after a folder match, so that a feed that no index lists, such as a
// blocked test feed, is not missed (ADR 0019).
function FolderFeeds({ connector, onAdded }) {
  const [found, setFound] = useState(null);
  const [chosen, setChosen] = useState([]);
  const [result, setResult] = useState(null);
  const { busy, error, run } = useAction();
  const panelRef = useRef(null);
  useEffect(() => {
    if (found && found.length > 0 && panelRef.current) panelRef.current.scrollIntoView({ block: 'nearest' });
  }, [found]);
  useEffect(() => {
    api(`/connectors/${connector.id}/feeds`).then((list) => {
      const others = list.feeds.filter((f) => !f.in_collection);
      setFound(others);
      setChosen(others.map((f) => f.path));
    }, () => setFound([]));
  }, [connector.id]);
  if (!found || found.length === 0) return null;
  if (result) return html`<div class="notice" role="status">Studio added ${result} feed${result === 1 ? '' : 's'} from “${connector.name}”.</div>`;
  const toggle = (path) => setChosen(chosen.includes(path) ? chosen.filter((p) => p !== path) : [...chosen, path]);
  const add = () => run(async () => {
    for (const path of chosen) {
      const feed = await api('/feeds', { method: 'POST', body: { connector_id: connector.id, path } });
      await api(`/feeds/${feed.id}/refresh`, { method: 'POST' });
    }
    setResult(chosen.length);
    onAdded();
  });
  return html`<div class="panel form" ref=${panelRef}>
    <h2>Studio also found these feeds in this folder</h2>
    <fieldset class="choices"><legend class="visually-hidden">Feeds to add</legend>
      ${found.map((f) => {
        const kind = feedKind(f);
        return html`<label class="choice">
          <input type="checkbox" checked=${chosen.includes(f.path)} onChange=${() => toggle(f.path)} />
          <span class=${`dot ${kind.dot}`}></span>
          <span class="text"><span class="title">${f.title || f.path}</span>
            <span class="sub">${kind.label}${f.blocked ? ' · blocked: directories do not show it' : ''} · <span class="mono">${f.path}</span></span></span>
        </label>`;
      })}
    </fieldset>
    <${ErrorNotice} error=${error} />
    <div class="actions"><button class="button primary" type="button" disabled=${busy || chosen.length === 0} onClick=${add}>
      ${busy ? 'Adding…' : `Add ${chosen.length} Feed${chosen.length === 1 ? '' : 's'}`}</button></div>
  </div>`;
}

// The library folder and the download limits (ADR 0036 items 1 and 17).
// A move of the library to a new folder is not part of this version, so
// this panel shows the folder as a fact and lets the user change only the
// download limits.
// The Settings section of a server that holds no assets (ADR 0105 item 17).
function FilesOnServerPanel() {
  const [storage, setStorage] = useState(undefined);
  useEffect(() => {
    api('/status').then((status) => setStorage(status.storage || null), () => setStorage(null));
  }, []);
  if (storage === undefined) return null;
  return html`<section class="page-section settings-section">
    <h2>Files on this server</h2><div class="group settings-group">
    <p class="hint">${SERVER_FILES_NOTE}</p>
    <div class="group rows">
      ${serverFilesRows(storage, sizeText).map((row) => html`<div class="link-row static" key=${row.label}>
        <span class="text"><span class="label">${row.label}</span><span class="summary">${row.summary}</span></span>
      </div>`)}
    </div>
  </div></section>`;
}

function LibraryPanel({ draft, draftState, changed, causes }) {
  const [library, setLibrary] = useState(null);
  const [maxMb, setMaxMb] = useState('');
  const [minMb, setMinMb] = useState('');
  const load = () => api('/library').then((result) => {
    setLibrary(result);
    setMaxMb(draftState.values.libraryMax ?? String(Math.round(result.download_max_file_bytes / (1024 * 1024))));
    setMinMb(draftState.values.libraryMin ?? String(Math.round(result.download_min_free_bytes / (1024 * 1024))));
  });
  useEffect(() => { load(); }, []);
  // Both limits go in one request, so a change of one limit records both.
  const savedMb = (bytes) => String(Math.round(bytes / (1024 * 1024)));
  const updateLimits = (max, min) => {
    setMaxMb(max); setMinMb(min);
    draft('libraryMax', savedMb(library.download_max_file_bytes), max);
    draft('libraryMin', savedMb(library.download_min_free_bytes), min);
  };
  const updateMax = (value) => updateLimits(value, minMb);
  const updateMin = (value) => updateLimits(maxMb, value);
  if (!library) return null;
  return html`<section class="page-section settings-section">
    <h2>Library</h2><div class="group settings-group">
    ${!serverMode && html`<dl class="facts"><dt>Folder</dt><dd class="mono">${library.root}</dd></dl>`}
    <div class="form">
        <${Field} id="limit-max" label=${html`Largest download, in MB<${SettingsPencil} pending=${changed('libraryMax')} />`} inputmode="numeric" required
          error=${causes.libraryMax} value=${maxMb} onInput=${(e) => updateMax(e.target.value)} />
        <${Field} id="limit-min" label=${html`Free space to keep, in MB<${SettingsPencil} pending=${changed('libraryMin')} />`} inputmode="numeric" required
          error=${causes.libraryMin} value=${minMb} onInput=${(e) => updateMin(e.target.value)} />
    </div>
  </div></section>`;
}

// The Nostr search relay (ADR 0074 items 16 and 17). An empty field stops the
// relay search. The core checks the address.
function NostrPanel({ draft, draftState, changed, causes }) {
  const [saved, setSaved] = useState(null);
  const [relay, setRelay] = useState('');
  const load = () => api('/settings/nostr').then((result) => {
    setSaved(result.search_relay);
    setRelay(draftState.values.nostr ?? result.search_relay);
  });
  useEffect(() => { load(); }, []);
  if (saved === null) return null;
  return html`<section class="page-section settings-section">
    <h2>Nostr</h2><div class="group settings-group">
    <div class="form">
      <${Field} id="nostr-relay" label=${html`Nostr search relay<${SettingsPencil} pending=${changed('nostr')} />`} type="text" placeholder="wss://search.nos.today"
        hint="Studio sends the text of a name search to this relay. Leave the field empty to search only your feeds and Music Index."
        error=${causes.nostr} value=${relay} onInput=${(e) => { setRelay(e.target.value); draft('nostr', saved, e.target.value); }} />
    </div>
  </div></section>`;
}

// The connected apps (ADR 0054 items 7 to 10, ADR 0110 items 5 to 11). The
// user gives a name. Studio then shows the pairing text one time, and the
// app pairs with its own key. The pairing text stays in the state of this
// panel only. It goes out of the state when the app pairs, when the user
// selects Done and when the panel closes. The only write to the clipboard is
// in `copy`, which the Copy button calls.
function ConnectedAppsPanel() {
  const [apps, setApps] = useState(null);
  const [adding, setAdding] = useState(false);
  const [name, setName] = useState('');
  const [made, setMade] = useState(null);
  const [phase, setPhase] = useState('waiting');
  const [copied, setCopied] = useState(false);
  const { busy, error, run } = useAction();
  const load = () => api('/apps').then(setApps);
  useEffect(() => { load(); }, []);
  const timeText = (seconds) => new Date(seconds * 1000).toLocaleString();
  const clockText = (seconds) => new Date(seconds * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  // While the app waits, read its state every 2 seconds. The app pairs when
  // its state is not `waiting`. A missing app (404) means that the code ended.
  const waitingId = made && phase === 'waiting' ? made.id : null;
  useEffect(() => {
    if (waitingId === null) return undefined;
    const timer = setInterval(async () => {
      let next;
      try {
        next = pairingPhase(await api(`/apps/${waitingId}`));
      } catch (e) {
        if (e.status !== 404) return;
        next = pairingPhase(null);
      }
      if (next === 'waiting') return;
      setPhase(next);
      setMade((current) => (current && current.id === waitingId ? { ...current, pairingText: null } : current));
      setCopied(false);
      load().catch(() => {});
    }, 2000);
    return () => clearInterval(timer);
  }, [waitingId]);
  const start = (appName) => run(async () => {
    const result = await api('/apps', { method: 'POST', body: { name: appName } });
    setMade({ id: result.id, name: result.name, pairingText: result.pairing_text });
    setPhase('waiting');
    setCopied(false);
    setAdding(false);
    setName('');
    await load();
  });
  const make = (event) => {
    event.preventDefault();
    start(name.trim());
  };
  const remove = async (app) => {
    if (!await askConfirm(CONFIRMS.removeApp(app.name))) return;
    run(async () => {
      await api(`/apps/${app.id}`, { method: 'DELETE' });
      if (made && made.id === app.id) setMade(null);
      await load();
    });
  };
  // An app from before the key pairs has a token, which Studio refuses. The
  // button removes it and makes a pairing text for the same name.
  const connectAgain = (app) => run(async () => {
    await api(`/apps/${app.id}`, { method: 'DELETE' });
    await start(app.name);
  });
  const copy = () => {
    if (navigator.clipboard && made && made.pairingText) {
      navigator.clipboard.writeText(made.pairingText).then(() => setCopied(true), () => {});
    }
  };
  const done = () => run(async () => {
    const id = made.id;
    const removes = doneRemovesApp(phase);
    setMade(null);
    if (removes) await api(`/apps/${id}`, { method: 'DELETE' }).catch((e) => { if (e.status !== 404) throw e; });
    await load();
  });
  if (!apps) return null;
  const notice = made && pairingNotice(made, phase);
  return html`<section class="page-section settings-section">
    <h2>Connected apps</h2><div class="group settings-group">
    ${apps.length === 0 && !adding && html`<p class="hint">No app is connected.</p>`}
    ${apps.length > 0 && html`<ul class="app-list">
      ${apps.map((a) => {
        const lines = appRowLines(a, timeText, clockText);
        return html`<li key=${a.id}>
          <span class="app-name">${a.name}
            <span class="hint">Key <span class="mono">${lines.key}</span></span>
            <span class="hint">${lines.status}</span>
            ${lines.address && html`<span class="hint">Live value address <span class="mono">${lines.address}</span></span>`}</span>
          <span class="app-buttons">
            ${lines.connectAgain && html`<button class="button action" type="button" disabled=${busy}
              aria-label=${`Connect the app ${a.name} again`} onClick=${() => connectAgain(a)}>Connect Again</button>`}
            <button class="button destructive" type="button" disabled=${busy}
              aria-label=${`Remove the app ${a.name}`} onClick=${() => remove(a)}>Remove</button>
          </span>
        </li>`;
      })}
    </ul>`}
    ${notice && html`<div class="notice" role="status">
      <strong>${notice.title}</strong>
      ${notice.text && html`<${SecretInput} id="app-pairing-text" name="the pairing text" class="mono token-field" readonly
        aria-label="Pairing text" value=${notice.text} />`}
      <p class="hint">${notice.hint}</p>
      <div class="actions">
        ${notice.copy && html`<button class="button" type="button" onClick=${copy}>${copied ? 'Copied' : 'Copy'}</button>`}
        <button class="button" type="button" disabled=${busy} onClick=${done}>Done</button>
      </div>
    </div>`}
    ${adding
      ? html`<form class="form" onSubmit=${make}>
          <${Field} id="app-name" label="Name of the app" required autocomplete="off"
            hint="A name for this computer or this copy of the app."
            value=${name} onInput=${(e) => setName(e.target.value)} />
          <div class="actions">
            <button class="button primary" type="submit" disabled=${busy || name.trim() === ''}>
              ${busy ? 'Making…' : 'Make Pairing Text'}</button>
            <button class="button" type="button" disabled=${busy} onClick=${() => setAdding(false)}>Cancel</button>
          </div>
        </form>`
      : html`<div class="actions"><button class="button action" type="button"
          onClick=${() => { setMade(null); setAdding(true); }}>Connect an App</button></div>`}
    <${ErrorNotice} error=${error} />
  </div></section>`;
}

// The ID of the newest manifest among the relays of a site state.
function newestManifest(state) {
  let newest = null;
  for (const r of state.relays) {
    if (r.id && (!newest || r.created_at > newest.created_at)) newest = r;
  }
  return newest ? newest.id : null;
}

// One host, a local folder or an S3 or R2 bucket. "Edit" opens the name field and the public
// address field, with "Save" and "Cancel" (task 022b, ADR 0037). "Save"
// calls `PUT /connectors/{id}`.
function HostCard({ connector, connectors, feeds, removeBusy, onRemove, onOpen, onChanged, onMoved }) {
  const [movingAll, setMovingAll] = useState(false);
  const config = JSON.parse(connector.config);
  const written = feeds.filter((f) => f.connector_id === connector.id);
  const disagreeing = feeds.filter((f) => (connector.disagreeing_feeds || []).includes(f.id));
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(connector.name);
  const [baseUrl, setBaseUrl] = useState(config.base_url || '');
  const isNsite = connector.kind === 'nsite';
  const isBlossom = connector.kind === 'blossom';
  const isNostr = isNsite || isBlossom;
  // The temporary area of a server has no settings, so it has no form (ADR 0122 item 9).
  const isTemporary = connector.kind === 'temporary';
  // The fields of an nsite or Blossom host that "Edit" changes (ADR 0050).
  const [relays, setRelays] = useState(linesText(config.relays));
  const [servers, setServers] = useState(linesText(config.servers));
  const [gateway, setGateway] = useState(config.gateway || 'nsite.lol');
  const { busy, error, run } = useAction();
  const startEdit = () => {
    setName(connector.name);
    setBaseUrl(config.base_url || '');
    setRelays(linesText(config.relays));
    setServers(linesText(config.servers));
    setGateway(config.gateway || 'nsite.lol');
    setEditing(true);
  };
  const save = (event) => {
    event.preventDefault();
    run(async () => {
      const body = isNostr
        ? { name: name.trim(), servers: addressLines(servers), ...(isNsite ? { relays: addressLines(relays), gateway: gateway.trim() } : {}) }
        : { name: name.trim(), base_url: baseUrl };
      await api(`/connectors/${connector.id}`, { method: 'PUT', body });
      setEditing(false);
      onChanged();
    });
  };
  const addressError = splitError(error, ['invalid_address']);
  const nostrErrors = nostrFormErrors(error, ['relays', 'servers', 'gateway']);
  // The state of the relays of an nsite site: `GET /connectors/{id}/site`.
  const site = useAction();
  const [siteState, setSiteState] = useState(null);
  const checkRelays = () => {
    setSiteState(null);
    site.run(async () => { setSiteState(await api(`/connectors/${connector.id}/site`)); });
  };
  const signingText = config.signing === 'vault' ? 'Key in the vault' : 'My signer';
  const isS3 = connector.kind === 's3';
  const isSftp = connector.kind === 'sftp';
  const noun = isS3 ? 'bucket' : 'folder';
  // A check of an SFTP host lists its folder. A changed host key gives
  // host_unauthorized with the new fingerprint in the message. The card
  // then offers to accept the new key.
  const check = useAction();
  const accept = useAction();
  const [newKey, setNewKey] = useState(null);
  const checkHost = () => {
    setNewKey(null);
    check.run(async () => {
      try {
        await api(`/connectors/${connector.id}/feeds`);
      } catch (e) {
        const found = e.code === 'host_unauthorized' && /host key changed/.test(e.message)
          ? /SHA256:[A-Za-z0-9+/=]+/.exec(e.message) : null;
        if (!found) throw e;
        setNewKey(found[0]);
      }
    });
  };
  const acceptKey = () => accept.run(async () => {
    await api(`/connectors/${connector.id}`, { method: 'PUT', body: { host_key: newKey } });
    setNewKey(null);
    onChanged();
  });
  const mismatch = accept.error && accept.error.code === 'host_key_mismatch';
  return html`<article class="panel host">
    <${MoveAllSheet} open=${movingAll} connector=${connector} connectors=${connectors} onMoved=${onMoved} onClose=${() => setMovingAll(false)} />
    <header class="host-head">
      <span class="eyebrow">${isTemporary ? 'Temporary area' : isS3 ? 'S3 or R2 bucket' : isSftp ? 'SFTP server' : isNsite ? 'nsite site' : isBlossom ? 'Blossom server' : 'Folder on this computer'}</span>
      ${!editing && html`<div class="actions">
        ${connectors.length > 1 && html`<button class="button action" type="button" disabled=${removeBusy}
          onClick=${() => setMovingAll(true)}>Move All Releases…</button>`}
        ${!isTemporary && html`<button class="button" type="button" disabled=${removeBusy} onClick=${startEdit}>Edit</button>`}
        <button class="button destructive" type="button" disabled=${removeBusy}
          aria-label=${`Remove the file host ${connector.name}`} onClick=${onRemove}>Remove</button>
      </div>`}
    </header>
    ${editing
      ? html`<form class="form" onSubmit=${save}>
          <${Field} id=${`host-${connector.id}-name`} label="Name" required
            value=${name} onInput=${(e) => setName(e.target.value)} />
          ${isNostr && html`${isNsite && html`<${AreaField} id=${`host-${connector.id}-relays`} label="Relays"
              error=${nostrErrors.fields.relays} value=${relays} onInput=${(e) => setRelays(e.target.value)} />`}
            <${AreaField} id=${`host-${connector.id}-servers`} label="Blossom servers"
              error=${nostrErrors.fields.servers} value=${servers} onInput=${(e) => setServers(e.target.value)} />
            ${isNsite && html`<${Field} id=${`host-${connector.id}-gateway`} label="Gateway" autocomplete="off"
              error=${nostrErrors.fields.gateway} value=${gateway} onInput=${(e) => setGateway(e.target.value)} />`}
            ${nostrErrors.notice}`}
          ${!isNostr && html`<${Field} id=${`host-${connector.id}-url`} label=${`Public address of the ${noun}`} type="url"
            placeholder="https://music.example.com/feeds" error=${addressError.field}
            value=${baseUrl} onInput=${(e) => setBaseUrl(e.target.value)} />`}
          ${!isNostr && html`<${ErrorNotice} error=${addressError.other} />`}
          <div class="actions">
            <button class="button primary" type="submit" disabled=${busy}>${busy ? 'Saving…' : 'Save'}</button>
            <button class="button" type="button" disabled=${busy} onClick=${() => setEditing(false)}>Cancel</button>
          </div>
        </form>`
      : html`<h2>${connector.name}</h2>
          ${isTemporary && html`<p class="hint">${HOST_STEP.temporary}</p>`}
          <dl class="facts">
            ${isS3
              ? html`<dt>Endpoint</dt><dd class="mono">${config.endpoint}</dd>
                <dt>Bucket</dt><dd class="mono">${config.bucket}</dd>
                <dt>Folder in the bucket</dt><dd class="mono">${config.prefix || '—'}</dd>`
              : isSftp
                ? html`<dt>Server</dt><dd class="mono">${`${config.user}@${config.host}:${config.port}`}</dd>
                  <dt>Folder</dt><dd class="mono">${config.root}</dd>
                  <dt>Login</dt><dd>${{ key: 'Private key', password: 'Password', agent: 'ssh-agent' }[config.login] || config.login}</dd>
                  <dt>Host key</dt><dd class="mono">${config.host_key}</dd>`
                : isNostr
                  ? html`<dt>Public key</dt><dd class="mono">${npubOf(config.public_key) || config.public_key}</dd>
                    ${isNsite && html`<dt>Site name</dt><dd class="mono">${config.d}</dd>
                      <dt>Address</dt><dd class="mono">${siteAddress(config) || '—'}</dd>
                      <dt>Relays</dt><dd><ul class="address-list">${(config.relays || []).map((r) => html`<li class="mono">${r}</li>`)}</ul></dd>`}
                    <dt>Blossom servers</dt><dd><ul class="address-list">${(config.servers || []).map((r) => html`<li class="mono">${r}</li>`)}</ul></dd>
                    <dt>Signing</dt><dd>${signingText}</dd>`
                  : isTemporary ? null : html`<dt>Folder</dt><dd class="mono">${config.root}</dd>`}
            ${!isNostr && !isTemporary && html`<dt>Public address</dt><dd><span class="mono">${config.base_url || '—'}</span>
              ${config.base_url && config.base_url_inferred && html`<span class="sub"> · Studio found it from the feeds</span>`}</dd>`}
            <dt>Feeds</dt><dd>${written.length === 0 ? 'None yet' : html`<details class="disclosure host-feeds">
              <summary>${written.length} ${written.length === 1 ? 'feed' : 'feeds'}</summary>
              <ul class="feed-list">
              ${written.map((f) => html`<li><button class="button link" type="button" onClick=${() => onOpen(f.id)}>
                <span class=${`dot ${feedKind(f).dot}`}></span>${f.title || f.path}</button></li>`)}</ul></details>`}</dd>
            ${disagreeing.length > 0 && html`<dt>Different addresses</dt><dd>
              These feeds have an address that does not agree with the folder.
              <ul class="address-list">
                ${disagreeing.map((f) => html`<li><button class="button link" type="button" onClick=${() => onOpen(f.id)}>${f.title || f.path}</button>
                  <span class="mono">${f.public_url}</span></li>`)}
              </ul></dd>`}
          </dl>
          ${isNsite && html`<div class="actions">
            <button class="button" type="button" disabled=${site.busy} onClick=${checkRelays}>
              ${site.busy ? 'Checking…' : 'Check the Relays'}</button>
          </div>`}
          ${isNsite && siteState && (siteState.consistent
            ? html`<p class="hint" role="status">Each relay has the newest manifest.</p>`
            : html`<div class="notice error" role="status">
                <strong>The relays do not agree yet. Studio shows the site as inconsistent until each relay has the newest manifest.</strong>
                <ul class="address-list">${siteState.relays
                  .filter((r) => r.error || r.id !== newestManifest(siteState))
                  .map((r) => html`<li><span class="mono">${r.url}</span>: ${r.error || (r.id ? `older manifest ${r.id.slice(0, 8)}` : 'no manifest')}</li>`)}</ul>
              </div>`)}
          ${isNsite && html`<${ErrorNotice} error=${site.error} />`}
          ${isSftp && html`<div class="actions">
            <button class="button" type="button" disabled=${check.busy || accept.busy} onClick=${checkHost}>
              ${check.busy ? 'Checking…' : 'Check the Connection'}</button>
          </div>`}
          ${isSftp && newKey && html`<div class="notice error" role="alert">
            <strong>The server gives a new host key.</strong>
            <p>Make sure that this is the fingerprint of the server before you accept it:</p>
            <p class="mono">${newKey}</p>
            <div class="actions"><button class="button action" type="button" disabled=${accept.busy} onClick=${acceptKey}>
              ${accept.busy ? 'Accepting…' : 'Accept the New Key'}</button></div>
          </div>`}
          ${mismatch && html`<div class="notice error" role="alert"><strong>The server gives a different key now.</strong>Connect again to see it.</div>`}
          ${!mismatch && html`<${ErrorNotice} error=${accept.error} />`}
          ${isSftp && html`<${ErrorNotice} error=${check.error} />`}`}
  </article>`;
}

function Hosts({ connectors, feeds, onChanged, onOpen, onMoved }) {
  const { busy, error, run } = useAction();
  const [linked, setLinked] = useState(null);
  const [adding, setAdding] = useState(false);
  // The result of a new host shows above the host cards, and the form is
  // below them, so the page brings the result into view.
  const resultRef = useRef(null);
  useEffect(() => {
    if (linked && resultRef.current) resultRef.current.scrollIntoView({ block: 'start' });
  }, [linked]);
  const remove = async (connector) => {
    if (!await askConfirm(CONFIRMS.removeFileHost(connector.name))) return;
    run(async () => {
      await api(`/connectors/${connector.id}`, { method: 'DELETE' });
      onChanged();
    });
  };
  // A new folder takes each read-only feed whose file it holds (ADR 0020).
  // The add form waits for this, so that its button stays busy until the
  // new host card shows. A second click then cannot add the host again.
  const added = (connector) => run(async () => {
    const result = await api(`/connectors/${connector.id}/match`, { method: 'POST' });
    setLinked({ connector, feeds: result.linked, folder: result.folder_address });
    await onChanged();
    setAdding(false);
  });
  const showForm = adding || connectors.length === 0;
  // The sections of the page come from `pageSequence('hosts')` (ADR 0071).
  const sections = {
    cards: () => html`${connectors.map((c) => html`<${HostCard} key=${c.id} connector=${c} connectors=${connectors} onMoved=${onMoved} feeds=${feeds} removeBusy=${busy}
        onRemove=${() => remove(c)} onOpen=${onOpen} onChanged=${onChanged} />`)}`,
    add: () => (showForm
      ? html`<div class="panel"><h2>${connectors.length === 0 ? 'Connect your first file host' : 'Add a file host'}</h2>
          <${AddHostForm} feeds=${feeds} onAdded=${added} />
          ${connectors.length > 0 && html`<button class="button link" type="button" onClick=${() => setAdding(false)}>Cancel</button>`}</div>`
      : html`<div class="actions"><button class="button action" type="button" onClick=${() => setAdding(true)}>＋ Add a File Host</button></div>`),
  };
  return html`<div>
    <${Bar}><span class="bar-title">File hosts</span><//>
    <section class="section">
      ${error && error.code === 'in_use'
        ? html`<div class="notice error" role="alert"><strong>Feeds use this file host.</strong>Move those feeds to a different file host, or remove them from your collection.</div>`
        : html`<${ErrorNotice} error=${error} />`}
      ${linked && html`<div ref=${resultRef}></div>`}
      ${linked && linked.feeds.length > 0 && html`<div class="notice" role="status">
        Studio can now write ${linked.feeds.length} feed${linked.feeds.length > 1 ? 's' : ''} through “${linked.connector.name}”.
      </div>`}
      ${linked && linked.folder && linked.folder.state === 'inferred' && linked.folder.new && html`<div class="notice" role="status">
        <strong>Studio found the public address of “${linked.connector.name}” from its feeds.</strong>
        <span class="mono">${linked.folder.url}</span>
      </div>`}
      ${linked && linked.folder && linked.folder.state === 'different' && html`<div class="notice" role="status">
        <strong>The feeds in “${linked.connector.name}” give different public addresses.</strong>
        Studio did not set a public address for the folder. The file host card lists the feeds.
      </div>`}
      ${linked && html`<${FolderFeeds} key=${linked.connector.id} connector=${linked.connector} onAdded=${onChanged} />`}
      ${pageSequence('hosts').map((section) => sections[section.id]())}
    </section>
  </div>`;
}

// The Settings page (ADR 0071 items 5 and 9, ADR 0166 items 1 to 5). It
// shows the tabs of `settingsTabs`, and the panels of the open tab. The open
// tab is part of the view, so Back and Forward go between tabs. The page
// opens on General.
function SettingsPage({ view, go }) {
  const [draft, setDraft] = useState(() => makeSettingsDraft({}));
  const [revision, setRevision] = useState(0);
  const [saving, setSaving] = useState(false);
  const [saveErrors, setSaveErrors] = useState({});
  const draftView = settingsDraftView(draft);
  const record = (key, saved, value) => {
    setSaveErrors((old) => { const next = { ...old }; delete next[key]; return next; });
    setDraft((old) => {
      const seeded = Object.prototype.hasOwnProperty.call(old.saved, key) ? old : { ...old, saved: { ...old.saved, [key]: saved }, values: { ...old.values, [key]: saved } };
      return changeSetting(seeded, key, value);
    });
  };
  const leave = async (target) => {
    const tabOnly = target && target.name === 'settings' && view.name === 'settings' && target.tab !== view.tab;
    if (!asksSettingsLeave(draft, tabOnly ? 'tab' : 'page')) return true;
    const allowed = await askConfirm(CONFIRMS.leavePerson('Your Settings changes are not saved.'));
    if (allowed && Object.hasOwn(draft.saved, 'scale')) document.documentElement.dataset.uiScale = draft.saved.scale;
    return allowed;
  };
  useLeaveGuard(draftView.count > 0, leave);
  useEffect(() => {
    if (!draftView.count) return undefined;
    const warn = (event) => { event.preventDefault(); event.returnValue = ''; };
    addEventListener('beforeunload', warn);
    return () => removeEventListener('beforeunload', warn);
  }, [draftView.count]);
  const discard = () => {
    const next = discardSettings(draft);
    setDraft(next);
    setSaveErrors({});
    if (Object.hasOwn(next.saved, 'scale')) {
      document.documentElement.dataset.uiScale = next.saved.scale;
    }
    setRevision((value) => value + 1);
  };
  const save = async () => {
    setSaving(true);
    try {
      const routes = {
        scale: async (value) => { localStorage.setItem(SCALE_KEY, value); },
        lock: (value) => api('/settings/lock', { method: 'PUT', body: { minutes: Number(value) } }),
        window: (value) => api('/window-close', { method: 'PUT', body: { on_close: value } }),
        outside: (value) => api('/settings/outside-check', { method: 'PUT', body: { enabled: value } }),
        nostr: (value) => api('/settings/nostr', { method: 'PUT', body: { search_relay: value.trim() } }),
        library: (max, min) => api('/library/limits', { method: 'PUT', body: { download_max_file_bytes: Math.round(Number(max) * 1024 * 1024), download_min_free_bytes: Math.round(Number(min) * 1024 * 1024) } }),
      };
      const result = await saveSettings(draft, routes);
      setDraft(result.draft);
      setSaveErrors(result.errors);
    } finally { setSaving(false); }
  };
  const panels = {
    scale: ScalePanel,
    lock: LockPanel,
    window: WindowPanel,
    account: AccountPanel,
    apps: ConnectedAppsPanel,
    nostr: NostrPanel,
    outside: OutsidePanel,
    move: MovePanel,
    collections: CollectionsPanel,
    library: LibraryPanel,
    files: FilesOnServerPanel,
  };
  const tabs = settingsTabs(assetMode());
  const current = tabs.some((tab) => tab.id === view.tab) ? view.tab : 'general';
  const openTab = (tab) => go({ ...view, tab });
  const sections = tabs.find((tab) => tab.id === current)?.sections || [];
  const changed = (key) => draftView.rows.includes(key);
  // The bar gives the first cause that turns Save off (ADR 0166 item 18).
  const barCause = draftView.pending.map((key) => draftView.causes[key]).find(Boolean);
  return html`<div>
    <${Bar}><span class="bar-title">Settings</span>
      <div class="draft-toolbar"><span class=${barCause ? 'draft-status error' : 'draft-status'} role="status">${barCause || draftView.status}</span>
        <button class="button" type="button" disabled=${draftView.discardDisabled || saving} onClick=${discard}>Discard</button>
        <button class="button primary" type="button" disabled=${draftView.saveDisabled || saving} onClick=${save}>Save</button></div>
<//>
    <div class="settings-tabs"><${PageTabs} tabs=${tabs.map((tab) => ({ ...tab, pending: draftView.tabs.includes(tab.id) }))} current=${current} onSelect=${openTab} label="Settings tabs" /></div>
    <section class="section settings-content">
      ${sections.map((section) => {
        const Panel = panels[section.id];
        return html`<${Panel} key=${`${section.id}-${revision}`} draft=${record} draftState=${draft} changed=${changed} causes=${{ ...draftView.causes, ...saveErrors }} />`;
      })}
    </section>
  </div>`;
}

// The mark of a setting or a tab with a change that is not saved (ADR 0166
// item 9): the pencil of ADR 0165 item 2.
const NOT_SAVED_MARK = { icon: 'pencil', tone: 'accent', text: 'Not saved' };
function SettingsPencil({ pending }) {
  return pending && html` <${StateMark} mark=${NOT_SAVED_MARK} />`;
}

// The sheet of one row of the Add feeds page (ADR 0073 item 3). `open` is the
// ID of the row, or null. The forms of "Add a feed you have" do not change.
// The title of a make row follows the type and the role in the form (ADR 0164
// item 7).
function AddFeedSheet({ open, title, connectors, feeds, found, setFound, start, onAdded, onClose }) {
  const ref = useSheet(open !== null);
  const makeType = ['album', 'publisher', 'show', 'playlist'].includes(open) ? open : null;
  // The type and the role of the open form. A value of an earlier row is not used.
  const [kind, setKind] = useState(null);
  const now = kind && kind.open === open ? kind : { type: makeType, role: (start || {}).role };
  const shownTitle = makeType ? makeTitle(now.type, now.role) : title;
  let body = null;
  if (makeType) {
    body = html`<${MakeFeedForm} key=${open} connectors=${connectors} feeds=${feeds} startType=${makeType} start=${start}
      onAdded=${onAdded} onKind=${(k) => setKind({ ...k, open })} />`;
  } else if (open === 'find') {
    body = found
      ? html`<${FoundFeeds} found=${found} onAdded=${(result) => onAdded(result.added[0])} />
        <div class="actions"><button class="button link" type="button" onClick=${() => setFound(null)}>Search Again</button></div>`
      : html`<${FindFeeds} onFound=${setFound} />`;
  } else if (open === 'folder') {
    body = html`<${AddFeedForm} connectors=${connectors} onAdded=${onAdded} />`;
  } else if (open === 'msp') {
    body = html`<${MspImport} onAdded=${(result) => onAdded(result.added[0])} />`;
  }
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="add-feed-title" onClose=${onClose}>
    <header>
      <h2 id="add-feed-title">${shownTitle}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body">${body}</div>
  </dialog>`;
}

// The Add feeds page: two groups of rows. A row opens its form in a sheet
// (ADR 0073 items 1 to 4).
function AddFeeds({ connectors, feeds, onAdded, go }) {
  const [open, setOpen] = useState(null);
  const [found, setFound] = useState(null);
  const groups = addChoices({ hasHost: connectors.length > 0 });
  const rows = groups.flatMap((group) => group.rows);
  const close = () => { setOpen(null); setFound(null); };
  // The sheet gives the title of a make row itself.
  const title = (rows.find((row) => row.id === open) || {}).title || '';
  return html`<div><${Bar}><span class="bar-title">Add feeds</span><//><section class="section">
    ${groups.map((group) => html`<section key=${group.id} class="page-section"><h2>${group.label}</h2>
      ${group.connect && html`<p class="hint">Studio puts the files of a release on a file host. Connect a file host first.</p>
        <div class="actions"><button class="button action" type="button" onClick=${() => go({ name: 'hosts' })}>Connect a File Host</button></div>`}
      ${group.rows.length > 0 && html`<${RowList}>
        ${group.rows.map((row) => html`<button key=${row.id} class="link-row" type="button" onClick=${() => setOpen(row.id)}>
          <span class="text"><span class="label">${row.title}</span><span class="summary">${row.text}</span></span><span class="go">›</span>
        </button>`)}
      <//>`}
    </section>`)}
    <${AddFeedSheet} open=${open} title=${title} connectors=${connectors} feeds=${feeds} found=${found} setFound=${setFound}
      onAdded=${onAdded} onClose=${close} />
  </section></div>`;
}

// ---- Wallets and change sets (ADR 0091, ADR 0099) ----

// A checkbox row of a feed, with its title and its places (ADR 0091 item 3).
export function FeedCheckRow({ title, text, lines, checked, onChange }) {
  const shown = lines || (text ? [text] : []);
  return html`<label class="link-row use-row"><input type="checkbox" checked=${checked} onChange=${onChange} />
    <span class="text"><span class="label">${title}</span>${shown.map((line) => html`<span class="summary">${line}</span>`)}</span></label>`;
}

// Write one feed of a change set: the diff, the signer, then the write
// (ADR 0091 item 23, task 054c item 8). A plan that expired gets one new try.
async function writeChangeSetFeed(feedId) {
  for (let attempt = 0; ; attempt++) {
    const diff = await api(`/feeds/${feedId}/diff`);
    const toSign = eventsToSign(diff);
    const choice = await currentSignerChoice(toSign);
    const signedEvents = await signPlanned(toSign, choice, () => {});
    try {
      await api(`/feeds/${feedId}/write`, { method: 'POST', body: { diff_id: diff.id, signed_events: signedEvents } });
      return;
    } catch (e) {
      if (e.code !== 'plan_expired' || attempt > 0) throw e;
    }
  }
}

// The text of a failed write of one feed.
function writeFailText(e) {
  if (e.code === 'conflict') return 'The hosted file changed after the draft started. Studio did not write this feed.';
  return signErrorText(e.code) || e.message || 'Studio did not write this feed.';
}

// The review of a change set and its write (ADR 0091 items 21 to 24).
// The event after an unlock. A page that keeps a state from a locked vault
// reads it again.
const UNLOCKED = 'studio-unlocked';

function ChangeSetPage({ id, go, onChanged }) {
  const [set, setSet] = useState(null);
  const [known, setKnown] = useState([]);
  const [personName, setPersonName] = useState('');
  const [statuses, setStatuses] = useState({});
  const [openId, setOpenId] = useState(null);
  const [diffs, setDiffs] = useState({});
  const [writing, setWriting] = useState(false);
  const { busy, error, run } = useAction();
  useEffect(() => {
    run(async () => {
      const loaded = await api(`/change-sets/${id}`);
      setSet(loaded);
      setStatuses(initialStatuses(loaded));
      setOpenId(firstOpenId(loaded));
      setKnown(await api('/recipients'));
      if (loaded.person_id != null) {
        const person = await api(`/people/${loaded.person_id}`).catch(() => null);
        setPersonName(person ? person.name : '');
      }
    });
  }, [id]);
  // The server marks each feed `vault_locked` while the vault is locked.
  // After an unlock, the page reads the change set again, so that the cause
  // goes away and the feeds can be written.
  useEffect(() => {
    const again = () => api(`/change-sets/${id}`).then((loaded) => {
      setSet(loaded);
      setStatuses((before) => refreshedStatuses(before, loaded));
    }, () => {});
    window.addEventListener(UNLOCKED, again);
    return () => window.removeEventListener(UNLOCKED, again);
  }, [id]);
  // A feed opens its diff in a closed entry. One entry is open at a time.
  const toggle = (feedId, isOpen) => {
    setOpenId((current) => (isOpen ? feedId : current === feedId ? null : current));
    if (isOpen && !diffs[feedId]) {
      api(`/feeds/${feedId}/diff`).then(
        (diff) => setDiffs((all) => ({ ...all, [feedId]: { diff } })),
        (e) => setDiffs((all) => ({ ...all, [feedId]: { error: e } })),
      );
    }
  };
  const order = set ? writableIds(set) : [];
  const pending = Object.values(statuses).filter((s) => s.state === 'waiting' || s.state === 'failed').length;
  const writeAll = async () => {
    setWriting(true);
    let current = retryStatuses(statuses);
    setStatuses(current);
    for (let feedId = nextToWrite(current, order); feedId !== null; feedId = nextToWrite(current, order)) {
      current = startWrite(current, feedId);
      setStatuses(current);
      let failure = null;
      try {
        await writeChangeSetFeed(feedId);
        window.dispatchEvent(new Event(ANNOUNCE_RELOAD));
      } catch (e) {
        failure = writeFailText(e);
      }
      current = finishWrite(current, feedId, failure);
      setStatuses(current);
    }
    setWriting(false);
    await onChanged();
    try { setSet(await api(`/change-sets/${id}`)); } catch (_) { /* The set closes when each feed is written. */ }
  };
  const discard = async () => {
    if (hasWrittenFeed(set, statuses) && !await askConfirm(CONFIRMS.discardChanges())) return;
    run(async () => {
      await api(`/change-sets/${id}`, { method: 'DELETE' });
      await onChanged();
      go(afterDiscard(set), { replace: true });
    });
  };
  const notFound = error && error.status === 404;
  const pendingWrite = set !== null && pending > 0;
  return html`<div><${Bar}><span class="bar-title">Review the changes</span>
    ${set && html`${pendingWrite && html`<button class="button primary" type="button" disabled=${writing || busy} onClick=${writeAll}>
        ${writing ? 'Writing…' : writeLabel(pending)}</button>`}
      <button class="button destructive" type="button" disabled=${writing || busy} onClick=${discard}>Discard</button>`}<//><section class="section">
    ${notFound
      ? html`<div class="notice" role="status"><strong>Studio did not find these changes.</strong>It is written or discarded.
          <div class="actions"><button class="button" type="button" onClick=${() => go({ name: 'people' })}>Open People</button></div></div>`
      : html`<${ErrorNotice} error=${error} />`}
    ${set === null && !error && html`<p class="empty" role="status">Loading the changes…</p>`}
    ${set && html`<header class="recipient-head"><h1>${changeTitle(set.operation)}</h1>
      <p class="hint">${set.person_id != null ? changeLine(set, personName) : operationLine(set.operation, nameOf(known, set.recipient))}</p></header>
      <${AnnounceChoice} />
      <section class="page-section"><h2>Feeds</h2><div class="group">${set.feeds.map((feed) => {
        const outcome = outcomeText(feed.outcome, set.operation);
        const blocked = blockedText(feed.blocked);
        const status = statuses[feed.feed_id];
        const cause = [blocked, statusError(status)].filter(Boolean);
        const summary = html`<span class="text"><span class="label">${feed.title || 'Untitled feed'}</span>
            <span class="summary">${outcome.text}</span>
            ${cause.map((text) => html`<span class="summary field-error">${text}</span>`)}</span>
          ${status && html`<span class=${`status ${status.state}`} role="status">${statusText(status)}</span>`}`;
        if (!outcome.changed) return html`<div key=${feed.feed_id} class="link-row static">${summary}</div>`;
        const shown = diffs[feed.feed_id];
        return html`<details key=${feed.feed_id} class="disclosure" open=${openId === feed.feed_id}
          onToggle=${(e) => toggle(feed.feed_id, e.currentTarget.open)}>
          <summary class="link-row">${summary}</summary>
          <div class="change-diff">
            ${!shown && html`<p class="hint" role="status">Loading the changes…</p>`}
            ${shown && shown.error && html`<${ErrorNotice} error=${shown.error} />`}
            ${shown && shown.diff && (shown.diff.changed
              ? html`<${DiffView} diff=${shown.diff} />`
              : html`<p class="hint">The draft is the same as the hosted file.</p>`)}
          </div>
        </details>`;
      })}</div></section>`}
  </section></div>`;
}

// The left panel (ADR 0039 items 1, 5 and 6). It shows the destinations, or
// the outline of the open feed, and Hosts, Settings and Lock Studio at the
// bottom. `panel` is the value of `leftPanel()`. Only the scroll area scrolls,
// so that the brand and the footer stay in view (visual check of task 022).
// At 760 px or less the left panel is not shown: its content is the root
// page and the outline page of the stack (ADR 0039 item 11, musicindex ADR
// 0030 R-SHELL-2).
function LeftPanel({ panel, draft, changedFeeds, go, showReleases, onAdding, onImporting, onLock }) {
  if (panel.kind === 'level') {
    return html`<header><${Brand} /><${ThemeToggle} /></header>
      <div class="sidebar-scroll"><${Level} panel=${panel} go=${go} back /></div>
      <${LeftPanelFooter} panel=${panel} go=${go} onLock=${onLock} />`;
  }
  if (panel.kind === 'destinations') {
    return html`<header><${Brand} /><${ThemeToggle} /></header>
      <div class="sidebar-scroll">${destinationRows(panel.top, go)}</div>
      <${LeftPanelFooter} panel=${panel} go=${go} onLock=${onLock} />`;
  }
  return html`<header><${Brand} /><${ThemeToggle} /></header>
    <div class="sidebar-scroll">
      <${Outline} panel=${panel} draft=${draft} changedFeeds=${changedFeeds} go=${go} showReleases=${showReleases}
        onAdding=${onAdding} onImporting=${onImporting} />
    </div>
    <${LeftPanelFooter} panel=${panel} go=${go} onLock=${onLock} />`;
}

// The current row of a list gets aria-current.
const currentAttr = (item) => (item.current ? 'page' : undefined);

// One destination row. A row below Releases shows the state mark of the
// feeds that it covers, between the name and the count (ADR 0165 items 1 to 4).
function DestinationRow({ item, open }) {
  const { feeds } = useContext(OutsideContext);
  const mark = useStateMark(item.group === undefined ? [] : panelSet(feeds, item.group));
  const releases = item.group === undefined ? '' : `${item.label}, ${item.count} ${item.count === 1 ? 'release' : 'releases'}`;
  return html`<button class="row" type="button" aria-current=${currentAttr(item)}
    aria-label=${item.group === undefined ? undefined : (mark ? `${releases}, ${mark.text}` : releases)}
    onClick=${() => open(item)}>
    ${item.icon && html`<${Icon} name=${item.icon} />`}
    ${item.group !== undefined && (item.art
      ? html`<${FeedArt} art=${item.art} glow=${item.glow} dot=${item.dot} />`
      : html`<span class="art-space" aria-hidden="true"></span>`)}
    <span class="row-label">${item.label}</span>
    <${StateMark} mark=${mark} />
    ${item.group !== undefined && html`<span class="count" aria-hidden="true">${item.count}</span>`}
  </button>`;
}

// The destination rows: Releases, then each artist, label or other feed with
// roles, then "Not linked" (ADR 0072 item 1, ADR 0085 items 1 and 3). A row
// opens its destination when `opens` is true. A row below Releases shows the
// dot of the feed and the number of its releases.
function destinationRows(top, go) {
  const open = (item) => {
    if (!item.opens) return;
    go(item.group === undefined ? { name: item.id } : { name: 'releases', group: item.group });
  };
  const row = (item) => html`<${DestinationRow} item=${item} open=${open} />`;
  const groups = top.filter((item) => item.group !== undefined);
  return html`${top.filter((item) => item.group === undefined).map(row)}
    ${groups.length > 0 && html`<div class="destination-group">${groups.map((item) => html`
      ${item.heading && html`<h2 class="panel-heading">${item.heading}</h2>`}${row(item)}`)}</div>`}`;
}

// A section row or a release row of a level. A section row covers its feed
// and its releases. A release row covers its feed only (ADR 0165 item 3).
// `count` is given for a section row, which shows art and a number.
function LevelRow({ item, set, label, count, attrs, onOpen }) {
  const mark = useStateMark(set);
  const section = count !== undefined;
  const base = section ? `${label}, ${count} ${count === 1 ? 'release' : 'releases'}` : label;
  return html`<button class="row" type="button" ...${attrs} aria-label=${mark ? `${base}, ${mark.text}` : (section ? base : undefined)}
    onClick=${onOpen}>
    ${section && html`<${FeedArt} art=${item.art} glow=${item.glow} dot=${item.dot} />`}
    <span class="row-label">${label}</span>
    <${StateMark} mark=${mark} />
    ${section && html`<span class="count" aria-hidden="true">${count}</span>`}
  </button>`;
}

// One level of the hierarchy (ADR 0088 item 5): "‹" to the level above when
// `back` is true, the head of the feed, the feeds directly below it under
// their role headings, then its releases. The head opens the feed page, and
// a release opens its outline at narrow width.
function Level({ panel, go, back }) {
  const narrow = useNarrow();
  const [current, setCurrent] = useState(null);
  const { feeds } = useContext(OutsideContext);
  const head = panel.head;
  const headMark = useStateMark(levelFeedSet(feeds, head.id));
  let cursor = back ? 1 : 0;
  const headIndex = cursor++;
  // A different level can have fewer rows, so the current row stays in range.
  const total = cursor + panel.sections.reduce((n, section) => n + section.items.length, 0) + panel.releases.length;
  const currentIndex = current === null || current >= total ? (back ? 0 : headIndex) : current;
  // A row with no child rows has no `aria-expanded` (musicindex ADR 0061
  // R-LISTKEY-2 item 1).
  const rowAttrs = (index, parent, kind = 'page') => ({
    role: 'treeitem', tabindex: index === currentIndex ? 0 : -1,
    'data-list-kind': kind, 'data-list-parent': parent === null ? undefined : String(parent),
  });
  return html`<div role="tree" aria-label=${head.title} onKeyDown=${(event) => onTreeKey(event, currentIndex, setCurrent)}>
    ${back && html`<button class="row" type="button" ...${rowAttrs(0, null)} onClick=${() => go(panel.back.view)}><${Chevron} direction="back" />${panel.back.label}</button>`}
    <button class="outline-feed" type="button" ...${rowAttrs(headIndex, null, 'tree')} aria-expanded="true" aria-label=${`${head.title}, ${head.typeLabel}${headMark ? `, ${headMark.text}` : ''}`}
      onClick=${() => go({ name: 'feed', id: head.id, page: 'album', from: head.from })}>
      <${ArtWithDot} dot=${head.dot}><${FeedArt} art=${head.art} glow=${head.glow} className="outline-cover" /><//>
      <div class="text"><span class="title">${head.title}</span><span class="sub">${head.typeLabel}</span></div>
      <${StateMark} mark=${headMark} />
    </button>
    ${panel.sections.map((section) => html`<div class="destination-group">
      <h2 class="panel-heading">${section.heading}</h2>
      ${section.items.map((item) => { const index = cursor++; return html`<${LevelRow} item=${item} set=${panelSet(feeds, item.id)} label=${item.label} count=${item.count}
        attrs=${rowAttrs(index, headIndex)} onOpen=${() => go(item.view)} />`; })}
    </div>`)}
    ${panel.releases.length > 0 && html`<div class="destination-group">
      <h2 class="panel-heading">Releases</h2>
      ${panel.releases.map((item) => { const index = cursor++; return html`<${LevelRow} item=${item} set=${feeds.filter((feed) => feed.id === item.id)} label=${item.title}
        attrs=${rowAttrs(index, headIndex)} onOpen=${() => go(openFeedView(item.id, narrow, item.from))} />`; })}
    </div>`}</div>`;
}

// A level as a page of a narrow window (ADR 0088 item 7). Back opens the
// page below it in the stack.
function LevelPage({ panel, go }) {
  return html`<div class="panel-page">
    <${Bar}><span class="bar-title">${panel.head.title}</span><//>
    <section class="section outline-page"><${Level} panel=${panel} go=${go} back=${false} /></section>
  </div>`;
}

// The outline of an open feed (ADR 0039 item 6): the cover, the title and the
// type label, then one row for each item. The left panel shows it at a wide
// width, and the outline page shows it at a narrow width. `showReleases` is
// given at a wide width only, where the item "Releases" is in the panel.
function Outline({ panel, draft, changedFeeds, go, showReleases, onAdding, onImporting }) {
  const [current, setCurrent] = useState(null);
  const feed = panel.feed;
  const releases = panel.items.find((item) => item.id === 'releases');
  const rest = panel.items.filter((item) => item.id !== 'releases');
  const childRows = rest.filter((item) => !item.heading);
  const feedIndex = releases ? 1 : 0;
  const activeChild = childRows.findIndex((item) => item.current);
  const defaultCurrent = activeChild >= 0 ? feedIndex + 1 + activeChild : feedIndex;
  // A different feed can have fewer rows, so the current row stays in range.
  const currentIndex = current === null || current > feedIndex + childRows.length ? defaultCurrent : current;
  // A row with no child rows has no `aria-expanded` (musicindex ADR 0061
  // R-LISTKEY-2 item 1).
  const rowAttrs = (index, parent, kind = 'page') => ({
    role: 'treeitem', tabindex: index === currentIndex ? 0 : -1,
    'data-list-kind': kind, 'data-list-parent': parent === null ? undefined : String(parent),
  });
  let childIndex = feedIndex + 1;
  const hasBadAddress = (number) => draft.bad.some((b) => b.field.startsWith(`items/${number - 1}/`));
  // A feed opened from a level goes back to that level (ADR 0088 item 6).
  const up = () => (releases.target ? go(releases.target) : showReleases(feed.id));
  return html`<div role="tree" aria-label=${feed.label} onKeyDown=${(event) => onTreeKey(event, currentIndex, setCurrent)}>
    ${releases && html`<button class="row" type="button" ...${rowAttrs(0, null)} onClick=${up}><${Chevron} direction="back" />${releases.label}</button>`}
    <button class="outline-feed" type="button" ...${rowAttrs(feedIndex, null, 'tree')} aria-expanded="true" aria-current=${currentAttr(feed)}
      aria-label=${changedFeeds.has(feed.id) ? `${feed.label}, changes in the library folder` : feed.label}
      onClick=${() => { if (!feed.first) go({ name: 'feed', id: feed.id, page: feed.page }); }}>
      <${ArtWithDot} dot=${feed.dot}><${ImageThumb} url=${feed.image} glow=${feed.glow} className="outline-cover" always /><//>
      <div class="text">
        <span class="title">${feed.title}${changedFeeds.has(feed.id) && html`<span class="change-mark"></span>`}</span>
        <span class="sub">${feed.typeLabel}</span>
      </div>
    </button>
    <ul class="tree" role="group">
      ${rest.map((item) => {
        if (item.heading) {
          return html`<li key=${item.id} class="tree-heading"><span class="label">${item.label}</span></li>`;
        }
        if (item.action) {
          const open = () => {
            go({ name: 'feed', id: feed.id, page: 'album' });
            if (item.action === 'add-tracks') onImporting(true);
            else onAdding(item.action === 'add-live' ? 'live' : true);
          };
          const index = childIndex++;
          return html`<li key=${item.id}><button class="node add" type="button" ...${rowAttrs(index, feedIndex)} onClick=${open}>
            <span class="num">＋</span><span class="label">${item.label}</span></button></li>`;
        }
        const open = () => go(item.target || {
          name: 'feed', id: feed.id, page: item.page,
          ...(item.track === undefined ? {} : { track: item.track }),
        });
        const numbered = item.number !== undefined;
        const index = childIndex++;
        return html`<li key=${item.id}><button class="node" type="button" ...${rowAttrs(index, feedIndex)} aria-current=${currentAttr(item)} onClick=${open}>
          ${numbered && html`<span class="num">${item.number}</span>`}
          <span class="label">${item.label}</span>
          ${item.status && html`<span class="sub">${item.status}</span>`}
          ${numbered && hasBadAddress(item.number) && html`<span class="field-error">Address not correct</span>`}
        </button></li>`;
      })}
    </ul></div>`;
}

// The footer of the left panel and of the root page: People, File hosts, Settings, and Lock
// Studio when the vault is open. On a server with a locked vault, Unlock Studio
// opens the unlock sheet.
function LeftPanelFooter({ panel, go, onLock }) {
  return html`<footer>
    ${panel.bottom.map((item) => (item.id === 'lock'
      ? html`<button class="row" type="button" onClick=${onLock}><${Icon} name=${item.icon} />Lock Studio</button>`
      : item.id === 'unlock'
      ? html`<button class="row" type="button" onClick=${() => { askUnlock(); }}><${Icon} name=${item.icon} />Unlock Studio</button>`
      : item.id === 'collection'
      ? html`<button class="row collection-mark" type="button" onClick=${() => go({ name: 'settings' })}><${Icon} name=${item.icon} /><span class="row-label">${item.label}</span></button>`
      : html`<button class="row" type="button" aria-current=${currentAttr(item)}
          onClick=${() => { if (item.opens ?? !item.current) go({ name: item.id }); }}><${Icon} name=${item.icon} />${item.label}</button>`))}
  </footer>`;
}

// The root page of a narrow window (ADR 0039 item 11): the destinations, with
// Hosts, Settings and Lock Studio at its bottom. It has no back control.
function RootPage({ panel, go, onLock }) {
  return html`<div class="panel-page">
    <${Bar} back=${false}><span class="bar-title">Studio</span><//>
    <div class="stack-page">
      <header><${Brand} /><${ThemeToggle} /></header>
      ${destinationRows(panel.top, go)}
      <${LeftPanelFooter} panel=${panel} go=${go} onLock=${onLock} />
    </div>
  </div>`;
}

// The outline page of a narrow window (ADR 0039 item 11a). Its bar has the
// back control, the title of the feed and the draft toolbar of the feed.
function OutlinePage({ feed, panel, draft, changedFeeds, go, onAdding, onImporting, onChanged, connectors }) {
  // A host operation can change the draft XML. After it, the shell saves the
  // edits of the user and loads the draft again (ADR 0036 item 14).
  const opsChanged = async () => {
    if (await draft.flush()) await draft.load();
  };
  return html`<div class="panel-page">
    <${Bar}><span class="bar-title">${feed.title || 'Untitled feed'}</span>
      ${isEditable(feed) && html`<${DraftToolbar} key=${feed.id} feedId=${feed.id} feedConnectorId=${feed.connector_id} draft=${draft}
        onWritten=${onChanged} connectors=${connectors} onOpenHosts=${() => go({ name: 'hosts' })} onOpsChanged=${opsChanged} onOpsCleared=${() => {}} />`}
    <//>
    <section class="section outline-page">
      <${Outline} panel=${panel} draft=${draft} changedFeeds=${changedFeeds} go=${go} showReleases=${null}
        onAdding=${onAdding} onImporting=${onImporting} />
    </section>
  </div>`;
}

// The Releases page, the start page (ADR 0039 items 2, 2a and 3, musicindex
// ADR 0027 R-LIST-1). `list` is the sort and the view from the address.
// `marked` is the feed that the last Back selection left marked. With a
// `group`, the page shows the releases of one destination below Releases,
// and the sort of that page is `groupSort` (ADR 0085 items 4 to 6).
// A menu button and its menu use the sort menu styles and Escape behavior.
function MenuControl({ buttonLabel, accessibleName, menuLabel, items, onSelect }) {
  const [open, setOpen] = useState(false);
  const button = useRef(null);
  const pick = (entry) => {
    setOpen(false);
    onSelect(entry);
    if (button.current) button.current.focus();
  };
  // Escape closes the menu and keeps the focus on its button.
  const onKey = (event) => {
    if (event.key === 'Escape' && open) {
      event.stopPropagation();
      setOpen(false);
      if (button.current) button.current.focus();
    }
  };
  return html`<div class="sort" onKeyDown=${onKey}>
    <button class="button" type="button" ref=${button} aria-haspopup="menu"
      aria-label=${accessibleName} aria-expanded=${open ? 'true' : 'false'} onClick=${() => setOpen(!open)}>
      ${buttonLabel} ⌄</button>
    ${open && html`<ul class="sort-menu" role="menu" aria-label=${menuLabel}>
      ${items.map((entry) => html`${entry.separatorBefore && html`<li class="menu-separator" role="separator"></li>`}
        <li><button class=${entry.destructive ? 'menu-destructive' : ''} type="button" role=${entry.role || 'menuitem'}
          aria-checked=${entry.checked === undefined ? undefined : (entry.checked ? 'true' : 'false')}
          onClick=${() => pick(entry)}>${entry.mark !== undefined && html`<span class="sort-mark" aria-hidden="true">${entry.mark}</span>`}${entry.label}</button></li>`)}
    </ul>`}
  </div>`;
}

// The sort menu of a list (musicindex ADR 0027 R-LIST-1, ADR 0043 R-LIST-6).
// Each sort has two entries: its usual direction, then its reversed direction
// (musicindex ADR 0054 R-LIST-13, ADR 0155). `onSort` gets the sort and direction.
export function SortMenu({ sorts, sort, reversed, onSort }) {
  const items = sortEntries(sorts).map((entry) => ({
    ...entry,
    role: 'menuitemradio',
    checked: entry.sort === sort && entry.reversed === reversed,
    mark: entry.sort === sort && entry.reversed === reversed ? '✓' : '',
  }));
  return html`<${MenuControl} buttonLabel=${sortLabel(sort, reversed)} menuLabel="Sort" items=${items}
    onSelect=${(entry) => onSort(entry.sort, entry.reversed)} />`;
}

// The letter bar of "A to Z" (musicindex ADR 0027 R-LIST-1 item 2). A letter
// scrolls its first row into view. `idOf` gives the element ID of a row.
export function LetterBar({ letters, idOf }) {
  const show = (firstId) => {
    const row = document.getElementById(idOf(firstId));
    if (row) row.scrollIntoView({ block: 'start' });
  };
  return html`<nav class="letters" aria-label="Letters">
    ${letters.map((g) => html`<button class="button letter" type="button" onClick=${() => show(g.firstId)}>${g.letter}</button>`)}
  </nav>`;
}

function ReleasesPage({ feeds, connectors, marked, list, group, groupSort, groupReversed, changedFeeds, onQuery, onGroupSort, onOpen, onAddFeed, onAddedFeed }) {
  const inGroup = group !== undefined;
  const releases = releasesList({ feeds, connectors, sort: inGroup ? groupSort : list.sort, reversed: inGroup ? groupReversed : list.reversed, view: list.view, marked, group });
  const { results: outside } = useContext(OutsideContext);
  const head = releases.head;
  const title = head ? head.title : 'Releases';
  // The add action of the group of a publisher feed (ADR 0164 item 6). A new
  // album or playlist becomes a release of that feed before its page opens.
  // When the link fails, the group stays and shows the error.
  const headFeed = inGroup && head && head.id !== null ? feeds.find((f) => f.id === head.id) : null;
  const more = releaseAdd(headFeed, headFeed && headFeed.title);
  const [adding, setAdding] = useState(false);
  const addAction = useAction();
  const addedRelease = async (f) => {
    setAdding(false);
    if (f && ['music', 'musicl'].includes((f.medium || '').toLowerCase())) {
      const done = await addAction.run(async () => {
        await api(`/feeds/${headFeed.id}/links`, { method: 'POST', body: { release: f.id } });
        return true;
      });
      if (done !== true) return;
    }
    onAddedFeed(f);
  };
  // A feed group keeps its own sort and direction (ADR 0085 item 8).
  const pickSort = (id, reversed) => {
    if (inGroup) onGroupSort(id, reversed);
    else onQuery({ sort: id, reversed });
  };
  const hasRows = releases.rows.length > 0;
  const viewButton = (name, label, icon) => html`<button class="button quiet icon-button view-button" type="button"
    aria-label=${label} title=${label} aria-pressed=${list.view === name ? 'true' : 'false'}
    onClick=${() => onQuery({ view: name })}>${icon}</button>`;
  // A collection with no feed starts at the Welcome page (ADR 0119 item 1).
  if (!inGroup && showWelcome(feeds)) return html`<${WelcomePage} connectors=${connectors} feeds=${feeds} onAdded=${onAddedFeed} />`;
  return html`<div>
    <${Bar}><span class="bar-title">${title}</span><//>
    <section class="releases">
      <div class="releases-head">
        <h1>${title}</h1>
        ${(hasRows || (head && head.id !== null)) && html`<div class="releases-tools">
          ${hasRows && html`<${SortMenu} sorts=${releases.sorts} sort=${releases.sort} reversed=${releases.reversed} onSort=${pickSort} />`}
          ${hasRows && viewButton('rows', 'Rows', html`<svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M4 6h16M4 12h16M4 18h16"></path></svg>`)}
          ${hasRows && viewButton('tiles', 'Tiles', html`<svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="4" width="6.5" height="6.5" rx="1.5"></rect><rect x="13.5" y="4" width="6.5" height="6.5" rx="1.5"></rect><rect x="4" y="13.5" width="6.5" height="6.5" rx="1.5"></rect><rect x="13.5" y="13.5" width="6.5" height="6.5" rx="1.5"></rect></svg>`)}
          ${!inGroup && html`<button class="button action" type="button" onClick=${onAddFeed}>＋ Add a Feed</button>`}
          ${more && html`<button class="button action" type="button" onClick=${() => setAdding(true)}>＋ ${more.label}</button>`}
          ${head && head.id !== null && html`<button class="button" type="button" onClick=${() => onOpen(head.id)}>Open the Feed</button>`}
        </div>`}
      </div>
      ${head && head.id !== null && html`<p class="releases-roles"><span class=${`dot ${head.dot}`} aria-hidden="true"></span>${head.typeLabel}</p>`}
      ${head && head.id === null && html`<p class="hint releases-roles">${head.note}</p>`}
      ${releases.levelTile && html`<${LevelFeedTile} tile=${releases.levelTile} view=${releases.view}
        outside=${outside[releases.levelTile.id]} onOpen=${onOpen} />`}
      <${ErrorNotice} error=${addAction.error} />
      ${releases.letters && html`<${LetterBar} letters=${releases.letters} idOf=${(id) => `release-${id}`} />`}
      ${hasRows
        ? html`<${RowList} as="ul" className=${`releases-list ${releases.view}`} mode=${releases.view === 'tiles' ? 'grid' : 'list'} selector=":scope > li > button">
            ${releases.rows.map((row) => html`<li id=${`release-${row.id}`} data-marked=${row.marked ? 'true' : 'false'}>
              <button class=${releases.view === 'tiles' ? 'releases-tile' : 'releases-row'} type="button" onClick=${() => onOpen(row.id)}>
                <${ArtWithDot} dot=${row.dot}><${ImageThumb} url=${row.image} glow=${row.glow} className="releases-cover" always /><//>
                <span class="releases-text">
                  <span class="title">${row.title}${changedFeeds.has(row.id) && html`<span class="change-mark"><span class="visually-hidden"> Changes in the library folder</span></span>`}</span>
                  <span class="sub">${row.typeLabel}${row.hidden ? ' · Hidden from apps' : ''}${releases.view === 'rows' ? ` · ${row.host}` : ''}</span>
                  ${row.note && html`<span class="sub">${row.note}</span>`}
                  ${outsideMark(outside[row.id]) && html`<span class=${`sub outside-mark ${outsideMark(outside[row.id]).tone}`}>${outsideMark(outside[row.id]).text}</span>`}
                  ${releases.view === 'rows' && html`<span class="sub">${row.draftLabel}</span>`}
                </span>
              </button>
            </li>`)}
          <//>`
        : inGroup
        ? html`<div class="empty-releases"><p class="empty">No releases are linked yet.</p></div>`
        : html`<div class="empty-releases">
            <p class="empty">No releases yet.</p>
            <div class="actions"><button class="button primary" type="button" onClick=${onAddFeed}>Add a Feed</button></div>
          </div>`}
    </section>
    ${adding && html`<${AddFeedRowSheet} open="album" connectors=${connectors} feeds=${feeds}
      start=${more.start} onAdded=${addedRelease} onClose=${() => setAdding(false)} />`}
  </div>`;
}

// The feed of an open level comes before its releases, in the same view
// (ADR 0181 item 2). It stays outside the release RowList, so the list keeps
// its ADR 0172 keyboard model.
function LevelFeedTile({ tile, view, outside, onOpen }) {
  const changed = outsideMark(outside);
  const label = `${tile.title}, ${tile.typeLabel}, ${tile.draftLabel}`;
  return html`<ul class=${`releases-list ${view}`}><li>
    <button class=${view === 'tiles' ? 'releases-tile' : 'releases-row'} type="button" aria-label=${label} onClick=${() => onOpen(tile.id)}>
    <${ArtWithDot} dot=${tile.dot}><${ImageThumb} url=${tile.image} glow=${tile.glow} className="releases-cover" always /><//>
    <span class="releases-text">
      <span class="title">${tile.title}</span>
      <span class="sub">${tile.typeLabel}</span>
      ${changed && html`<span class=${`sub outside-mark ${changed.tone}`}>${changed.text}</span>`}
      <span class="sub">${tile.draftLabel}</span>
    </span>
  </button></li></ul>`;
}

// The Welcome page of a collection with no feed (ADR 0119 items 1 to 3). Each
// row opens a sheet. The page writes nothing.
function WelcomePage({ connectors, feeds, onAdded }) {
  const [sheet, setSheet] = useState(null);
  const [importing, setImporting] = useState(false);
  // The import is offered where the shell permits it (ADR 0119 item 3).
  const canImport = !assetMode().server;
  return html`<div>
    <${Bar}><span class="bar-title">Welcome</span><//>
    <section class="section welcome">
      <h1>${WELCOME.title}</h1>
      <p class="welcome-intro">${WELCOME.intro}</p>
      <div class="group welcome-group">
        <${WelcomeRow} row=${WELCOME.rows.music} primary=${true} onClick=${() => setSheet('music')} />
        <${WelcomeRow} row=${WELCOME.rows.label} onClick=${() => setSheet('label')} />
      </div>
      <div class="group">
        <${WelcomeRow} row=${WELCOME.rows.online} onClick=${() => setSheet('find')} />
      </div>
      <p class="welcome-import">${WELCOME.mspBefore}
        <button class="button link" type="button" onClick=${() => setSheet('msp')}>${WELCOME.mspButton}</button></p>
      ${canImport && html`<p class="welcome-import">${WELCOME.importBefore}
        <button class="button link" type="button" onClick=${() => setImporting(true)}>${WELCOME.importButton}</button></p>
        <p class="hint">${DESKTOP_IMPORT_HINT}</p>`}
    </section>
    ${(sheet === 'music' || sheet === 'label') && html`<${StartSheet} key=${sheet} path=${sheet}
      connectors=${connectors} feeds=${feeds} onAdded=${onAdded} onClose=${() => setSheet(null)} />`}
    ${(sheet === 'find' || sheet === 'msp') && html`<${AddFeedRowSheet} open=${sheet} connectors=${connectors} feeds=${feeds}
      onAdded=${onAdded} onClose=${() => setSheet(null)} />`}
    ${importing && html`<${ImportFlow} onBack=${() => setImporting(false)} />`}
  </div>`;
}

// One large row of the Welcome page: an icon, a title, one line and the arrow.
// `selected` is set on a row of step 2, and it marks the selected row (ADR 0122 item 2).
function WelcomeRow({ row, primary = false, selected = null, onClick }) {
  return html`<button class="link-row welcome-row" type="button" aria-pressed=${selected === null ? null : String(selected)} onClick=${onClick}>
    <span class=${primary ? 'welcome-icon primary' : 'welcome-icon'} aria-hidden="true"><${Icon} name=${row.icon} /></span>
    <span class="text"><span class="label">${row.title}</span><span class="summary">${row.line}</span></span>
    <span class="go" aria-hidden="true">›</span>
  </button>`;
}

// The sheet of a musician or a label (ADR 0119 items 4 to 7, ADR 0122 item 2).
// Step 1 asks the name. Step 2 always shows. It has the select menu of the
// connected file hosts, when one exists, and the rows of the file host types.
// Then the draft of the publisher feed is made. Studio writes nothing (ADR 0007).
// The sheet mounts for each open, so each open starts at step 1.
function StartSheet({ path, connectors, feeds, onAdded, onClose }) {
  const ref = useSheet(true);
  const text = pathTexts(path);
  const names = welcomeSteps(path);
  const step2 = hostStepModel(connectors, serverMode);
  const [step, setStep] = useState('name');
  const [name, setName] = useState('');
  // The row of step 2 that is selected, or null when the menu of the
  // connected file hosts is selected. Only the selected choice shows its
  // primary button.
  const [hostType, setHostType] = useState(step2.selected);
  // The file host that this sheet connected. A new try after a failure uses
  // it, and does not connect a second file host.
  const [madeHost, setMadeHost] = useState(null);
  // The connected file host of the select menu. The first one is selected at the start.
  const [connectorId, setConnectorId] = useState(step2.menu ? step2.menu.value : '');
  // The first folder of the desktop (ADR 0122 item 3).
  const [defaultRoot, setDefaultRoot] = useState('');
  const { busy, error, run } = useAction();
  useEffect(() => {
    if (serverMode) return;
    api('/library').then((library) => setDefaultRoot(library.default_host_root || ''), () => {});
  }, []);
  const now = step === 'name' ? 0 : 1;
  const makePage = async (id) => onAdded(await api('/feeds/new', { method: 'POST', body: welcomeBody(path, name.trim(), id) }));
  const createFeed = (id) => run(() => makePage(id));
  // The row "Try it here (temporary)" sends the kind `temporary` with no other field (ADR 0122 item 4).
  const addTemporary = () => run(async () => {
    const connector = await api('/connectors', { method: 'POST', body: { kind: 'temporary' } });
    setMadeHost(connector);
    await makePage(connector.id);
  });
  // Continue of step 1 opens step 2, which always shows (ADR 0122 item 2).
  const next = (event) => {
    event.preventDefault();
    setStep('where');
  };
  const make = (event) => {
    event.preventDefault();
    createFeed(Number(connectorId));
  };
  // The form of the selected row. Its submit button adds the file host and makes the page.
  const selectedForm = hostType === null ? null : hostType === 'temporary'
    ? html`<div class="actions"><button class="button primary" type="button" disabled=${busy} onClick=${addTemporary}>${busy ? 'Making the Page…' : 'Continue'}</button></div>`
    : html`<${AddHostForm} key=${hostType} feeds=${feeds} start=${hostType} showType=${false}
        startRoot=${defaultRoot} submitLabel="Continue"
        onAdded=${(connector) => { setMadeHost(connector); createFeed(connector.id); }} />`;
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="start-title" onClose=${onClose}>
    <header>
      <h2 id="start-title">${text.sheet}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body">
      <ol class="steps-line">${names.map((label, i) => {
        const state = i < now ? 'done' : i === now ? 'now' : 'later';
        return html`<li key=${label} class=${state}><span class="step-mark" aria-hidden="true">${state === 'done' ? '✓' : i + 1}</span>${label}</li>`;
      })}</ol>
      ${step === 'name'
        ? html`<form class="form" onSubmit=${next}>
            <${Field} id="start-name" label=${text.nameLabel} required value=${name} onInput=${(e) => setName(e.target.value)} />
            ${text.hint && html`<p class="hint">${text.hint}</p>`}
            <div class="actions"><button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>
              <button class="button primary" type="submit" disabled=${busy || name.trim() === ''}>Continue</button></div>
          </form>`
        : madeHost !== null
          ? html`<div class="form">
              <p>${`Studio connected the file host ${madeHost.name}.`}</p>
              <div class="actions"><button class="button primary" type="button" disabled=${busy}
                onClick=${() => createFeed(madeHost.id)}>${busy ? 'Making the Page…' : 'Make the Page'}</button></div>
            </div>`
          : html`<div class="form">
              <p>${text.where}</p>
              ${step2.menu && html`<form class="form" onSubmit=${make}>
                <${ChoiceSelect} id="start-host" label="File host" value=${connectorId}
                  options=${step2.menu.options} onChange=${(value) => { setConnectorId(value); setHostType(null); }} />
                ${hostType === null && html`<div class="actions"><button class="button primary" type="submit" disabled=${busy || connectorId === ''}>${busy ? 'Making the Page…' : 'Make the Page'}</button></div>`}
              </form>`}
              <div class="group">${step2.rows.map((row) => html`<${WelcomeRow} key=${row.type} row=${row}
                primary=${row.primary === true} selected=${row.type === hostType} onClick=${() => setHostType(row.type)} />`)}</div>
              ${selectedForm}
              <div class="actions"><button class="button" type="button" onClick=${() => setStep('name')}>Back</button></div>
            </div>`}
      <${ErrorNotice} error=${error} />
    </div>
  </dialog>`;
}

// A row sheet of the Add feeds page, for the first steps (ADR 0119 items 6
// and 8). The forms are the same as on the Add feeds page.
export function AddFeedRowSheet({ open, connectors, feeds, start, onAdded, onClose }) {
  const [found, setFound] = useState(null);
  const rows = addChoices({ hasHost: connectors.length > 0 }).flatMap((group) => group.rows);
  const title = (rows.find((row) => row.id === open) || {}).title || '';
  return html`<${AddFeedSheet} open=${open} title=${title} connectors=${connectors} feeds=${feeds}
    found=${found} setFound=${setFound} start=${start} onAdded=${onAdded} onClose=${() => { setFound(null); onClose(); }} />`;
}

// The navigation state of the session (ADR 0039 item 8). A reload keeps it
// in session storage. If the read fails, or the state is not valid, Studio
// starts again at Releases.
const NAV_KEY = 'studio-nav';

// The shortest time between two loads of the feed list on a focus of the
// window (ADR 0165 item 10).
const FOCUS_RELOAD_MS = 30000;

function readNav() {
  try {
    const saved = JSON.parse(sessionStorage.getItem(NAV_KEY));
    if (saved && Array.isArray(saved.entries) && saved.entries.length > 0
      && Number.isInteger(saved.index) && saved.index >= 0 && saved.index < saved.entries.length) {
      return saved;
    }
  } catch (_) {}
  return startNav({ name: 'releases' });
}

// If the write fails, the state stays in memory only.
function writeNav(nav) {
  try { sessionStorage.setItem(NAV_KEY, JSON.stringify(nav)); } catch (_) {}
}

// The shell. It owns the list of views, the left panel at a wide width, the
// one stack at a narrow width and the Releases page. Each new view is a history entry, so
// that the browser Back button works (musicindex ADR 0027 R-NAV-2).
function Shell({ vault, onLock }) {
  const [feeds, setFeeds] = useState(null);
  const [connectors, setConnectors] = useState([]);
  // At 760 px or less the window is one stack (musicindex ADR 0030 R-SHELL-2).
  const narrow = useNarrow();
  const [nav, setNavState] = useState(() => navAtWidth(readNav(), narrow));
  // The newest state. A scroll change saves here, without a new render.
  const navRef = useRef(nav);
  const [list, setList] = useState(() => listQuery(location.search));
  const [error, setError] = useState(null);
  // The Add a track sheet, by the left panel and by the album page (ADR 0029 item 7).
  const [adding, setAdding] = useState(false);
  // The import sheet, by the left panel and by the album page (task 018d).
  const [importing, setImporting] = useState(false);
  const coarse = useMedia('(pointer: coarse)');
  // The scroll area of the page. Each new view starts at its own scroll.
  const mainRef = useRef(null);
  const view = nav.entries[nav.index];

  // Saves the scroll of the page into the current entry.
  const withScroll = () => {
    if (mainRef.current) navRef.current = setScroll(navRef.current, mainRef.current.scrollTop);
    return navRef.current;
  };
  // Shows a navigation state, and saves it.
  const commit = (next) => {
    navRef.current = next;
    writeNav(next);
    setNavState(next);
  };
  // The guard of a page with pending changes (ADR 0116 item 10). It gives a
  // promise of true when the page may leave. `useLeaveGuard` sets it.
  const leaveGuard = useRef(null);
  const setLeaveGuard = useCallback((guard) => { leaveGuard.current = guard; }, []);
  // Set for the one leave that the guard allowed, so that its popstate does
  // not ask again.
  const leaving = useRef(false);
  // Runs `action` when the page may leave. Without a guard it runs at once.
  const guarded = (action, target) => {
    if (!leaveGuard.current) action();
    else leaveGuard.current(target).then((leave) => { if (leave) action(); });
  };
  // A new view, or with `replace`, the current view changes. The guard asks first.
  const go = (target, options) => guarded(() => goNow(target, options), target);
  const goNow = (target, { replace = false } = {}) => {
    const base = withScroll();
    // A page of the same feed keeps the level that the feed came from
    // (ADR 0088 item 6).
    const current = base.entries[base.index];
    const next = isFeedView(target) && current && isFeedView(current) && current.id === target.id
      && current.from && !target.from ? { ...target, from: current.from } : target;
    if (replace) {
      const entries = base.entries.slice();
      entries[base.index] = next;
      commit({ entries, index: base.index });
      history.replaceState({ scope: 'shell', index: base.index, mirrored: mirrored.current }, '');
      return;
    }
    const pushed = pushNav(base, next);
    commit(pushed);
    history.pushState({ scope: 'shell', index: pushed.index, mirrored: mirrored.current }, '');
  };
  // True while the browser history has one entry for each entry of the
  // state, up to the current one. Then the browser Back button and the back
  // control do the same (musicindex ADR 0030 R-SHELL-2).
  const mirrored = useRef(false);
  // Back. When the history agrees with the state, the browser goes back,
  // and `popstate` moves the state. Otherwise the state moves in place, for
  // example at index 0, where a feed gets a Releases entry before it.
  const back = () => {
    const current = withScroll();
    if (mirrored.current && current.index > 0 && history.state
      && history.state.scope === 'shell' && history.state.index === current.index) {
      // The browser goes back, and `popstate` asks the guard.
      history.back();
      return;
    }
    guarded(() => {
      const next = backNav(withScroll());
      commit(next);
      history.replaceState({ scope: 'shell', index: next.index, mirrored: mirrored.current }, '');
    });
  };
  const forward = () => guarded(() => {
    if (!canForward(nav)) return;
    const next = forwardNav(withScroll());
    commit(next);
    history.pushState({ scope: 'shell', index: next.index, mirrored: mirrored.current }, '');
  });
  // The link "Releases" of an open feed. It opens Releases as a new view,
  // and it does not go back in the history (ADR 0069). The view keeps the
  // scroll of the nearest Releases entry.
  const showReleases = (feedId) => {
    go(releasesView(withScroll(), feedId));
  };
  // The browser Back and Forward buttons. The state moves until its index
  // agrees with the index of the history entry.
  useEffect(() => {
    const index = navRef.current.index;
    const kept = history.state;
    if (kept && kept.scope === 'shell' && kept.index === index && kept.mirrored === true) {
      // A reload keeps the history entries of the state.
      mirrored.current = true;
    } else {
      // A new start makes one history entry for each entry of the state, so
      // that the browser Back button goes down the stack.
      history.replaceState({ scope: 'shell', index: 0, mirrored: true }, '');
      for (let i = 1; i <= index; i += 1) history.pushState({ scope: 'shell', index: i, mirrored: true }, '');
      mirrored.current = true;
    }
    // Applies the index of a popstate event to the state.
    const applyPop = (event) => {
      const target = event.state && event.state.scope === 'shell' ? event.state.index : 0;
      mirrored.current = Boolean(event.state && event.state.mirrored === true);
      let next = withScroll();
      while (next.index > target) {
        const stepped = backNav(next);
        if (stepped.index === next.index) break;
        next = stepped;
      }
      while (next.index < target && canForward(next)) next = forwardNav(next);
      commit(next);
    };
    // A page with pending changes asks first. The entry of the page goes back
    // in the history, so that "Keep Editing" keeps the page (ADR 0116 item 10).
    const onPop = (event) => {
      const targetIndex = event.state && event.state.scope === 'shell' ? event.state.index : 0;
      const currentView = navRef.current.entries[navRef.current.index];
      const targetView = navRef.current.entries[targetIndex];
      if (currentView && targetView && currentView.name === 'settings' && targetView.name === 'settings'
        && currentView.tab !== targetView.tab) {
        applyPop(event);
        return;
      }
      if (!leaveGuard.current || leaving.current) {
        leaving.current = false;
        applyPop(event);
        return;
      }
      history.pushState({ scope: 'shell', index: navRef.current.index, mirrored: mirrored.current }, '');
      leaveGuard.current().then((leave) => {
        if (!leave) return;
        leaving.current = true;
        history.back();
      });
    };
    addEventListener('popstate', onPop);
    return () => removeEventListener('popstate', onPop);
  }, []);
  // A change of width makes the state of the new width (ADR 0039 item 11a).
  // The current view stays. A state that already fits the width is kept.
  useEffect(() => {
    const current = navRef.current;
    const next = navAtWidth(current, narrow);
    if (next === current) return;
    commit(next);
    // The history entries before this one belong to the state of the other
    // width, so Back moves the state in place until the next load.
    mirrored.current = false;
    history.replaceState({ scope: 'shell', index: next.index, mirrored: false }, '');
  }, [narrow]);
  // After a change of view, the page scrolls to the saved position. With no
  // saved position, it scrolls to the marked row of Releases, or to the top.
  // A marked row that the saved position does not show moves into view by
  // the smallest scroll (musicindex ADR 0027 R-NAV-2 item 2).
  useEffect(() => {
    const main = mainRef.current;
    if (!main) return;
    const marked = view.name === 'releases' ? main.querySelector('[data-marked="true"]') : null;
    if (typeof view.scroll === 'number') {
      main.scrollTop = view.scroll;
      if (marked) marked.scrollIntoView({ block: 'nearest' });
      return;
    }
    if (marked) marked.scrollIntoView({ block: 'center' });
    else main.scrollTop = 0;
  }, [nav.index, view, feeds === null]);
  // The sort and the view write their query into the address. The hash stays.
  const changeList = (patch) => {
    const next = { ...list, ...patch };
    setList(next);
    const text = queryText(next);
    history.replaceState(history.state, '', `${location.pathname}${text ? `?${text}` : ''}${location.hash}`);
  };

  const load = useCallback(async () => {
    try {
      const [f, c] = await Promise.all([api('/feeds'), api('/connectors')]);
      setFeeds(f);
      setConnectors(c);
      setError(null);
    } catch (e) {
      setError(e);
    }
  }, []);
  useEffect(() => { load(); }, [load]);
  // The list loads again when the window gets the focus again, so that a
  // change from a different window shows. At most one load runs in each
  // FOCUS_RELOAD_MS (ADR 0165 item 10).
  const loadedAt = useRef(0);
  useEffect(() => {
    const again = () => {
      if (document.visibilityState !== 'visible' || Date.now() - loadedAt.current < FOCUS_RELOAD_MS) return;
      loadedAt.current = Date.now();
      load();
    };
    window.addEventListener('focus', again);
    document.addEventListener('visibilitychange', again);
    return () => {
      window.removeEventListener('focus', again);
      document.removeEventListener('visibilitychange', again);
    };
  }, [load]);

  // The differences between the library folder and the records (ADR 0038
  // items 13 to 15). One scan runs after the feeds load. The Files page
  // runs it again on request and after each action.
  const [differences, setDifferences] = useState([]);
  const [noticeClosed, setNoticeClosed] = useState(false);
  const rescan = useCallback(async () => {
    // A server that holds no assets has no folder scan (ADR 0105 item 14).
    if (!assetMode().holdAssets) return [];
    const result = await api('/library/scan', { method: 'POST' });
    setDifferences(result.differences);
    return result.differences;
  }, []);
  const scanned = useRef(false);
  useEffect(() => {
    if (feeds === null || scanned.current) return;
    scanned.current = true;
    // A failed first scan shows nothing. The Files page shows the error
    // of a scan that the user starts.
    rescan().catch(() => {});
  }, [feeds, rescan]);
  const changedFeeds = new Set(differences.map((d) => d.feed_id));

  const lock = async () => { await api('/vault/lock', { method: 'POST' }).catch(() => {}); onLock(); };
  const selected = isFeedView(view) && feeds ? feeds.find((f) => f.id === view.id) : null;
  // The shell holds the one draft of the selected feed (ADR 0029 item 4).
  // A feed with no connector, or a feed that is not an album, needs no draft.
  const draftFeedId = selected && isEditable(selected) ? selected.id : null;
  const draft = useDraft(draftFeedId);
  // Each draft answer gives the state of its feed. The row of that feed in
  // the list takes it, with no new read of the list (ADR 0165 item 9).
  useEffect(() => {
    const given = draft.draftState;
    if (!given || given.id !== draftFeedId) return;
    setFeeds((list) => (list && list.some((f) => f.id === given.id && f.draft_state !== given.state)
      ? list.map((f) => (f.id === given.id ? { ...f, draft_state: given.state } : f))
      : list));
  }, [draftFeedId, draft.draftState]);
  // A different feed closes the Add a track sheet and the import sheet of
  // the feed before it.
  useEffect(() => { setAdding(false); setImporting(false); }, [draftFeedId]);
  const collection = useCollectionMark();
  // The Releases page checks again each time it opens (ADR 0165 item 5).
  const releasesOpened = useRef({ shown: false, count: 0 });
  if (view.name === 'releases' && !releasesOpened.current.shown) releasesOpened.current.count += 1;
  releasesOpened.current.shown = view.name === 'releases';
  const results = useOutsideChecks(feeds, releasesOpened.current.count);
  const outsideValue = { feeds: feeds || [], results };
  const panel = leftPanel({
    view, feeds: feeds || [], model: draft.model, vaultOpen: vault === 'open',
    vaultLocked: serverMode && vault === 'locked', narrow, collection,
  });

  // A new feed opens its page. The list is loaded before the page opens.
  const addedFeed = async (f) => { await load(); go(f ? { name: 'feed', id: f.id } : { name: 'releases' }); };
  let main;
  // A screen with no bar of its own gets one.
  const withBar = (content) => html`<div><${Bar}><span class="bar-title">Studio</span><//>${content}</div>`;
  if (error) {
    main = withBar(html`<section class="section"><${ErrorNotice} error=${error} /></section>`);
  } else if (feeds === null) {
    main = withBar(html`<section class="section"><p class="empty" role="status">Loading your collection…</p></section>`);
  } else if (view.name === 'root') {
    main = html`<${RootPage} panel=${panel} go=${go} onLock=${lock} />`;
  } else if (view.name === 'hosts') {
    main = html`<${Hosts} connectors=${connectors} feeds=${feeds} onChanged=${() => load()}
      onOpen=${(id) => go({ name: 'feed', id })} onMoved=${async (id) => { await load(); go({ name: 'change-set', id, area: 'hosts' }); }} />`;
  } else if (view.name === 'settings') {
    main = html`<${SettingsPage} view=${view} go=${go} />`;
  } else if (view.name === 'people') {
    // The People list keeps its sort in the navigation record, as a feed
    // group does (ADR 0085 item 8, ADR 0153).
    main = html`<${PeoplePage} feeds=${feeds} go=${go} sort=${view.sort} reversed=${view.reversed === true}
      onOrder=${(order) => go({ ...view, ...order }, { replace: true })} />`;
  } else if (view.name === 'person') {
    main = html`<${PersonPage} key=${view.id} id=${view.id} feeds=${feeds} go=${go} />`;
  } else if (view.name === 'check') {
    main = html`<${CheckPage} key=${view.id} id=${view.id} feeds=${feeds} go=${go} />`;
  } else if (view.name === 'wallet') {
    main = html`<${WalletPage} key=${JSON.stringify(recipientKey(view.recipient))} recipient=${view.recipient} feeds=${feeds} go=${go} />`;
  } else if (view.name === 'change-set') {
    main = html`<${ChangeSetPage} key=${view.id} id=${view.id} go=${go} onChanged=${() => load()} />`;
  } else if (view.name === 'add-feed') {
    main = html`<${AddFeeds} connectors=${connectors} feeds=${feeds} go=${go} onAdded=${addedFeed} />`;
  } else if (selected && view.name === 'outline') {
    main = html`<${OutlinePage} feed=${selected} panel=${panel} draft=${draft} changedFeeds=${changedFeeds} go=${go}
      onAdding=${setAdding} onImporting=${setImporting} onChanged=${() => load()} connectors=${connectors} />`;
  } else if (selected) {
    main = html`<${FeedDetail} feed=${selected} feeds=${feeds} draft=${draft} view=${view} go=${go}
      adding=${adding} onAdding=${setAdding} importing=${importing} onImporting=${setImporting} onChanged=${() => load()}
      onListChanged=${() => load()}
      onRemoved=${async () => { await load(); go({ name: 'releases' }, { replace: true }); }} connectors=${connectors}
      differences=${differences.filter((d) => d.feed_id === selected.id)} onRescan=${rescan} />`;
  } else if (narrow && panel.kind === 'level') {
    main = html`<${LevelPage} panel=${panel} go=${go} />`;
  } else {
    // A release of a level goes back to that level (ADR 0088 item 6).
    const from = view.name === 'releases' && typeof view.group === 'number'
      ? { group: view.group, trail: view.trail || [] } : undefined;
    main = html`<${ReleasesPage} feeds=${feeds} connectors=${connectors} marked=${view.marked} list=${list}
      group=${view.name === 'releases' ? view.group : undefined} groupSort=${view.sort} groupReversed=${view.reversed === true}
      onGroupSort=${(sort, reversed) => go({ ...view, sort, reversed }, { replace: true })}
      changedFeeds=${changedFeeds} onQuery=${changeList}
      onOpen=${(id) => go(openFeedView(id, narrow, from))} onAddFeed=${() => go({ name: 'add-feed' })} onAddedFeed=${addedFeed} />`;
  }

  const navValue = { coarse, canBack: canBack(nav), canForward: canForward(nav), back, forward, setLeaveGuard };
  const panelProps = { panel, draft, changedFeeds, go, showReleases, onAdding: setAdding, onImporting: setImporting, onLock: lock };
  return html`<${NavContext.Provider} value=${navValue}><${OutsideContext.Provider} value=${outsideValue}><div class="shell">
    ${!narrow && html`<nav class="sidebar" aria-label="Collection">
      <${LeftPanel} ...${panelProps} />
    </nav>`}
    <main class="main" ref=${mainRef}>
      ${differences.length > 0 && !noticeClosed && html`<div class="notice scan-notice" role="status">
        <span>Studio found ${differences.length} ${differences.length === 1 ? 'change' : 'changes'} in the library folder.</span>
        <div class="actions">
          <button class="button" type="button"
            onClick=${() => go({ name: 'feed', id: differences[0].feed_id, page: 'files' })}>Show</button>
          <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => setNoticeClosed(true)}>✕</button>
        </div>
      </div>`}
      ${main}
    </main>
  </div><//><//>`;
}

// ---- Sign-in and sign-up on a server (ADR 0055, ADR 0056) ----

// The text of a failed auth request. Studio shows its own text for each
// code of the auth routes.
function AuthNotice({ error, importing = false }) {
  if (!error) return null;
  const text = error.code === 'offline' || error.code === 'signer'
    ? error.message
    : (importing && importErrorText(error.code, error.message)) || nip46ErrorText(error.code);
  return html`<div class="notice error" role="alert">${text}</div>`;
}

// Sign in or sign up with a NIP-07 signer (ADR 0056 items 5, 6, 9 and 11).
// For a new account, the signer encrypts the vault key to its own public key,
// and the server keeps the result as the Nostr wrapper.
async function signerAuth({ invite, recoveryKey }) {
  const signer = window.nostr;
  if (!signer) throw new ApiError(0, 'no_signer', authErrorText('no_signer'));
  if (!signer.nip44) {
    throw new ApiError(0, 'signer', 'This signer does not support NIP-44. Studio needs NIP-44 to open the vault.');
  }
  const { challenge } = await authCall('/challenge');
  let event;
  try {
    event = await signer.signEvent(signInEvent(location.origin, challenge, Math.floor(Date.now() / 1000)));
  } catch (_) {
    throw new ApiError(0, 'signer', 'The signer did not sign the sign-in request.');
  }
  const result = await authCall('/nostr', { body: { event, invite: invite || null, recovery_key: recoveryKey } });
  if (result.new_account) {
    csrfToken = result.csrf;
    try {
      const pubkey = await signer.getPublicKey();
      const { key } = await api('/vault/signer-key', { method: 'POST' });
      const ciphertext = await signer.nip44.encrypt(pubkey, key);
      await api('/vault/nostr-wrapper', { method: 'POST', body: { pubkey, ciphertext } });
    } catch (error) {
      if (error instanceof ApiError) throw error;
      throw new ApiError(0, 'signer', 'The signer did not encrypt the vault key.');
    }
  }
  csrfToken = result.csrf;
  return result;
}

// ---- A remote signer through NIP-46 (ADR 0056 items 12 to 16) ----

// The wait for a signer. The server answers a request for the state after a
// change or after 25 seconds. The UI asks again until the state changes.
async function nip46Wait(connection, alive) {
  while (alive()) {
    const answer = await exchange(`/auth/v1/nip46/${connection}`, {}, { method: 'GET' });
    const view = nip46State(answer);
    if (view.phase !== 'waiting') return view;
  }
  return null;
}

// End a connection that the user does not use. A failure does not matter.
function nip46Stop(connection) {
  exchange(`/auth/v1/nip46/${connection}`, csrfToken ? { 'X-Studio-CSRF': csrfToken } : {}, { method: 'DELETE' }).catch(() => {});
}

// The page that connects a remote signer and then signs in with it. The page
// starts a `nostrconnect://` connection at once, and the user can paste a
// bunker address instead. `onDone` follows a sign-in, and `onBack` follows
// the Stop button.
function Nip46Page({ title, invite, recoveryKey, onDone, onBack, onConnected = null }) {
  const [link, setLink] = useState(null);
  const [bunker, setBunker] = useState('');
  const [phase, setPhase] = useState('starting');
  const qr = useAction();
  const paste = useAction();
  const current = useRef(null);
  const alive = useRef(true);

  // Sign in through a connected signer. The connection ID then stops working.
  const signIn = async (connection) => {
    setPhase('signing');
    try {
      // The import of an account opens the vault of the file with the
      // connection, in place of a sign-in.
      if (onConnected) {
        await onConnected(connection);
        current.current = null;
        return;
      }
      const result = await authCall(`/nip46/${connection}/signin`, { body: { invite: invite || null, recovery_key: !!recoveryKey } });
      current.current = null;
      csrfToken = result.csrf;
      onDone(result.recovery_key || null);
    } catch (e) {
      setPhase('failed');
      throw e;
    }
  };

  const start = () => qr.run(async () => {
    setPhase('starting');
    const made = await authCall('/nip46/connect', { body: {} });
    current.current = made.connection;
    setLink(made);
    setPhase('waiting');
    const mine = () => alive.current && current.current === made.connection;
    let view;
    try {
      view = await nip46Wait(made.connection, mine);
    } catch (e) {
      // The user ended this connection, for example with a bunker address.
      if (!mine()) return;
      throw e;
    }
    if (!view) return;
    if (view.phase === 'failed') {
      setPhase('failed');
      throw new ApiError(0, view.error, nip46ErrorText(view.error));
    }
    await signIn(made.connection);
  });

  useEffect(() => {
    alive.current = true;
    start();
    return () => {
      alive.current = false;
      if (current.current) nip46Stop(current.current);
    };
  }, []);

  const connectBunker = (event) => {
    event.preventDefault();
    paste.run(async () => {
      const old = current.current;
      current.current = null;
      if (old) nip46Stop(old);
      setPhase('connecting');
      let made;
      try {
        made = await authCall('/nip46/connect', { body: { bunker: bunker.trim() } });
      } catch (e) {
        setPhase('failed');
        throw e;
      }
      current.current = made.connection;
      await signIn(made.connection);
    });
  };

  const waiting = phase === 'waiting' || phase === 'starting';
  return html`<main class="center"><div class="card">
    <${Brand} />
    <div><h1>${title}</h1>
      <p>Scan the QR code with your signer app. On a phone, open the link instead. Or paste a bunker address from your signer.</p></div>
    ${link && html`<img class="qr" alt="QR code of the signer link" src=${qrSource(link.qr_svg)} width="240" height="240" />`}
    ${link && html`<div class="actions"><a class="button action" href=${link.uri}>Open in the Signer App</a></div>`}
    <form class="form" onSubmit=${connectBunker}>
      <${Field} id="nip46-bunker" label="Bunker address" autocomplete="off" spellcheck="false"
        placeholder="bunker://…" value=${bunker} onInput=${(e) => setBunker(e.target.value)} />
      <div class="actions"><button class="button primary" type="submit" disabled=${paste.busy || phase === 'signing' || !isBunkerAddress(bunker)}>Connect</button></div>
    </form>
    ${phase === 'waiting' && html`<p role="status">Waiting for the signer…</p>`}
    ${phase === 'connecting' && html`<p role="status">Connecting to the signer…</p>`}
    ${phase === 'signing' && html`<p role="status">${onConnected ? 'Opening the vault. Allow the request in your signer app.' : 'Signing in. Allow the request in your signer app.'}</p>`}
    <${AuthNotice} error=${paste.error || qr.error} importing=${!!onConnected} />
    <div class="actions">
      ${(waiting || phase === 'failed') && html`<button class="button" type="button" onClick=${onBack}>${waiting ? 'Stop' : 'Back'}</button>`}
      ${phase === 'failed' && html`<button class="button action" type="button" onClick=${start}>Try Again</button>`}
    </div>
  </div></main>`;
}

// True when the browser can make and use passkeys (ADR 0056 item 20).
const hasPasskeys = typeof window.PublicKeyCredential === 'function';

// Run a call of `navigator.credentials`. The user can cancel it, and the
// error becomes an `ApiError` with a code that the UI has text for.
async function credentialCall(work) {
  try {
    return await work();
  } catch (e) {
    if (e instanceof ApiError) throw e;
    const code = passkeyBrowserCode(e);
    throw new ApiError(0, code, authErrorText(code));
  }
}

function missingPrf() {
  return new ApiError(422, 'passkey_no_prf', authErrorText('passkey_no_prf'));
}

// Add a passkey to the open vault (ADR 0056 items 21 to 23). The browser
// makes the credential, and then it does one assertion with the PRF
// extension. The server checks both before it trusts the PRF output.
async function registerPasskey(name) {
  const options = await withUnlock(() => authCall('/passkeys/options', { body: {} }));
  const { created, asserted } = await credentialCall(async () => {
    const made = await navigator.credentials.create({ publicKey: creationOptions(options) });
    const again = await navigator.credentials.get({
      publicKey: requestOptions({
        ...options,
        challenge: options.assertion_challenge,
        allow: [toBase64Url(made.rawId)],
      }),
    });
    return { created: made, asserted: again };
  });
  const registration = registrationPayload(created);
  if (!registration) throw new ApiError(0, 'passkey_unsupported', authErrorText('passkey_unsupported'));
  const prf = prfOutputOf(asserted);
  if (!prf) throw missingPrf();
  return authCall('/passkeys', {
    body: { name, registration, assertion: assertionPayload(asserted), prf_output: prf },
  });
}

// Sign in with a passkey, or open the vault with it when a session exists
// (ADR 0056 items 22 and 24). One assertion gives the proof and the PRF output.
async function passkeyAuth() {
  const options = await authCall('/passkey/options', { body: {} });
  const credential = await credentialCall(() => navigator.credentials.get({ publicKey: requestOptions(options) }));
  const prf = prfOutputOf(credential);
  if (!prf) throw missingPrf();
  return authCall('/passkey', { body: { assertion: assertionPayload(credential), prf_output: prf } });
}

function SignInPage({ onDone, onSignUp, onRecovery }) {
  const [email, setEmail] = useState('');
  const [passphrase, setPassphrase] = useState('');
  const [remote, setRemote] = useState(false);
  const { busy, error, run } = useAction();
  const submit = (event) => {
    event.preventDefault();
    run(async () => {
      const result = await authCall('/signin/email', { body: { email: normalEmail(email), passphrase } });
      csrfToken = result.csrf;
      onDone(null);
    });
  };
  const signer = () => run(async () => {
    const result = await signerAuth({ invite: null, recoveryKey: false });
    onDone(result.recovery_key || null);
  });
  const byPasskey = () => run(async () => {
    const result = await passkeyAuth();
    csrfToken = result.csrf;
    onDone(null);
  });
  if (remote) {
    return html`<${Nip46Page} title="Sign in with Amber or a bunker" invite=${null} recoveryKey=${false}
      onDone=${onDone} onBack=${() => setRemote(false)} />`;
  }
  return html`<main class="center"><form class="card" onSubmit=${submit}>
    <${Brand} />
    <div><h1>Sign in</h1></div>
    <${Field} id="sign-in-email" label="Email address" type="email" autocomplete="username" required autofocus
      value=${email} onInput=${(e) => setEmail(e.target.value)} />
    <${SecretField} id="sign-in-passphrase" label="Passphrase" autocomplete="current-password" required
      value=${passphrase} onInput=${(e) => setPassphrase(e.target.value)} />
    <${AuthNotice} error=${error} />
    <div class="actions">
      <button class="button primary" type="submit" disabled=${busy || !email.trim() || !passphrase}>${busy ? 'Signing In…' : 'Sign In'}</button>
      ${window.nostr && html`<button class="button action" type="button" disabled=${busy} onClick=${signer}>Sign In with a Signer Extension</button>`}
      ${hasPasskeys && html`<button class="button action" type="button" disabled=${busy} onClick=${byPasskey}>Sign In with a Passkey</button>`}
      <button class="button action" type="button" disabled=${busy} onClick=${() => setRemote(true)}>Sign In with Amber or a Bunker</button>
    </div>
    <button class="button link" type="button" onClick=${onSignUp}>Make an Account</button>
    <button class="button link" type="button" onClick=${onRecovery}>Sign In with the Recovery Key</button>
  </form></main>`;
}

// Sign in with the recovery key (ADR 0056 item 3). The session then has a
// limit, and the page "Add a sign-in method" follows.
function RecoveryPage({ onDone, onBack }) {
  const [account, setAccount] = useState('');
  const [key, setKey] = useState('');
  const { busy, error, run } = useAction();
  const submit = (event) => {
    event.preventDefault();
    run(async () => {
      const parsed = recoveryAccount(account);
      const result = await authCall('/recovery', {
        body: { account: parsed ? parsed.value : account.trim(), recovery_key: key.trim() },
      });
      csrfToken = result.csrf;
      onDone();
    });
  };
  return html`<main class="center"><form class="card" onSubmit=${submit}>
    <${Brand} />
    <div><h1>Sign in with the recovery key</h1>
      <p>Give the email address or the public key of the account, and the recovery key.</p></div>
    <${Field} id="recovery-account" label="Email address or npub" autocomplete="username" required autofocus
      value=${account} onInput=${(e) => setAccount(e.target.value)} />
    <${SecretField} id="recovery-key" label="Recovery key" autocomplete="off" required
      value=${key} onInput=${(e) => setKey(e.target.value)} />
    ${error && html`<div class="notice error" role="alert">${error.code === 'offline' ? error.message : recoveryErrorText(error.code)}</div>`}
    <div class="actions">
      <button class="button primary" type="submit" disabled=${busy || !account.trim() || !key.trim()}>${busy ? 'Signing In…' : 'Sign In'}</button>
    </div>
    <button class="button link" type="button" onClick=${onBack}>Back to Sign In</button>
  </form></main>`;
}

// The page after a recovery sign-in (ADR 0056 item 3). The session can do
// nothing else until the user adds a sign-in method.
function AddMethodPage() {
  const { session, reload } = useContext(SessionContext);
  const [passphrase, setPassphrase] = useState('');
  const { busy, error, run } = useAction();
  const strength = useStrength(passphrase, true);
  const choices = addMethodChoices(session, !!window.nostr, hasPasskeys);
  const vaultError = splitError(error, ['passphrase_too_short', 'passphrase_too_weak']);
  const passError = passphraseError(passphrase, strength) || vaultError.field;
  const ready = strength && strength.acceptable;
  const byPassphrase = (event) => {
    event.preventDefault();
    if (!ready) return;
    run(async () => {
      await api('/vault/passphrase', { method: 'POST', body: { passphrase } });
      await reload();
    });
  };
  const bySigner = () => run(async () => {
    const signer = window.nostr;
    if (!signer.nip44) {
      throw new ApiError(0, 'signer', 'This signer does not support NIP-44. Studio needs NIP-44 to open the vault.');
    }
    try {
      const pubkey = await signer.getPublicKey();
      if (pubkey !== session.pubkey) {
        throw new ApiError(0, 'signer', 'This signer has another key than this account.');
      }
      const { key } = await api('/vault/signer-key', { method: 'POST' });
      const ciphertext = await signer.nip44.encrypt(pubkey, key);
      await api('/vault/nostr-wrapper', { method: 'POST', body: { pubkey, ciphertext } });
    } catch (e) {
      if (e instanceof ApiError) throw e;
      throw new ApiError(0, 'signer', 'The signer did not encrypt the vault key.');
    }
    await reload();
  });
  const byPasskey = () => run(async () => {
    await registerPasskey('Passkey 1');
    await reload();
  });
  const signOut = () => run(async () => {
    await authCall('/signout');
    await reload();
  });
  const signerError = error && (error.code === 'signer' || error.code === 'no_signer') ? error : null;
  return html`<main class="center"><div class="card">
    <${Brand} />
    <div><h1>Add a sign-in method</h1>
      <p>The recovery key opened the vault. Add a new way to sign in before you continue.</p></div>
    ${choices.includes('passphrase') && html`<form class="form" onSubmit=${byPassphrase} noValidate>
      <${SecretField} id="add-method-passphrase" label="New passphrase" autocomplete="new-password" required autofocus
        value=${passphrase} onInput=${(e) => setPassphrase(e.target.value)} error=${passError}>
        <${Meter} strength=${strength} />
      <//>
      <div class="actions">
        <button class="button primary" type="submit" disabled=${busy || !ready}>${busy ? 'Saving…' : 'Save the Passphrase'}</button>
      </div>
    </form>`}
    ${choices.includes('nip07') && html`<div class="actions">
      <button class="button action" type="button" disabled=${busy} onClick=${bySigner}>Use the Signer Extension</button></div>`}
    ${choices.includes('passkey') && html`<div class="actions">
      <button class="button action" type="button" disabled=${busy} onClick=${byPasskey}>Use a Passkey</button></div>`}
    ${choices.length === 0 && html`<p class="hint">This account signs in with a Nostr signer. Studio found no signer extension in this browser.</p>`}
    <${AuthNotice} error=${signerError || vaultError.other} />
    <div class="actions"><button class="button" type="button" disabled=${busy} onClick=${signOut}>Sign Out</button></div>
  </div></main>`;
}

function SignUpPage({ signup, onDone, onSignIn, onImport }) {
  const [email, setEmail] = useState('');
  const [passphrase, setPassphrase] = useState('');
  const [invite, setInvite] = useState('');
  const [recovery, setRecovery] = useState(true);
  const [remote, setRemote] = useState(false);
  const { busy, error, run } = useAction();
  const strength = useStrength(passphrase, true);
  const needInvite = signup === 'invite';
  const serverError = splitError(error, ['passphrase_too_short', 'passphrase_too_weak']);
  const passError = passphraseError(passphrase, strength) || serverError.field;
  const emailError = error && error.code === 'invalid_email' ? 'Give an email address.' : null;
  const ready = strength && strength.acceptable && email.trim() !== '' && (!needInvite || invite.trim() !== '');
  const submit = (event) => {
    event.preventDefault();
    if (!ready) return;
    run(async () => {
      const result = await authCall('/signup/email', {
        body: { email: normalEmail(email), passphrase, invite: needInvite ? invite.trim() : null, recovery_key: recovery },
      });
      csrfToken = result.csrf;
      onDone(result.recovery_key || null);
    });
  };
  const signer = () => run(async () => {
    const result = await signerAuth({ invite: needInvite ? invite.trim() : null, recoveryKey: recovery });
    onDone(result.recovery_key || null);
  });
  if (remote) {
    return html`<${Nip46Page} title="Sign up with Amber or a bunker" invite=${needInvite ? invite.trim() : null} recoveryKey=${recovery}
      onDone=${onDone} onBack=${() => setRemote(false)} />`;
  }
  return html`<main class="center"><form class="card" onSubmit=${submit} noValidate>
    <${Brand} />
    <div><h1>Make an account</h1>
      <p>Your passphrase opens the vault with the keys of your file hosts.</p></div>
    <${Field} id="sign-up-email" label="Email address" type="email" autocomplete="username" required autofocus
      error=${emailError} value=${email} onInput=${(e) => setEmail(e.target.value)} />
    ${needInvite && html`<${Field} id="sign-up-invite" label="Invite code" autocomplete="off" required
      value=${invite} onInput=${(e) => setInvite(e.target.value)} />`}
    <${SecretField} id="sign-up-passphrase" label="Passphrase" autocomplete="new-password" required
      value=${passphrase} onInput=${(e) => setPassphrase(e.target.value)} error=${passError}>
      <${Meter} strength=${strength} />
    <//>
    <label class="check"><input type="checkbox" checked=${recovery} onChange=${(e) => setRecovery(e.target.checked)} />
      Make a recovery key, in case I forget the passphrase</label>
    <${AuthNotice} error=${serverError.other && error.code !== 'invalid_email' ? error : null} />
    <div class="actions">
      <button class="button primary" type="submit" disabled=${busy || !ready}>${busy ? 'Making the Account…' : 'Make the Account'}</button>
      ${window.nostr && html`<button class="button action" type="button" disabled=${busy || (needInvite && invite.trim() === '')}
        onClick=${signer}>Sign Up with a Signer Extension</button>`}
      <button class="button action" type="button" disabled=${busy || (needInvite && invite.trim() === '')}
        onClick=${() => setRemote(true)}>Sign Up with Amber or a Bunker</button>
    </div>
    <div class="actions"><button class="button action" type="button" disabled=${busy} onClick=${onImport}>Import an Account</button></div>
    <button class="button link" type="button" onClick=${onSignIn}>I Have an Account</button>
  </form></main>`;
}

// The pages of a user with no session. After a sign-up, the recovery key
// shows before the shell opens.
function SignedOut({ session, onSignedIn }) {
  const [page, setPage] = useState('sign_in');
  const [recoveryKey, setRecoveryKey] = useState(null);
  const done = (key) => { if (key) setRecoveryKey(key); else onSignedIn(); };
  if (recoveryKey) {
    return html`<main class="center"><div class="card"><${Brand} />
      <${ShowRecoveryKey} recoveryKey=${recoveryKey} onDone=${onSignedIn} /></div></main>`;
  }
  return page === 'sign_in'
    ? html`<${SignInPage} onDone=${done} onSignUp=${() => setPage('sign_up')} onRecovery=${() => setPage('recovery')} />`
    : page === 'recovery'
      ? html`<${RecoveryPage} onDone=${onSignedIn} onBack=${() => setPage('sign_in')} />`
      : page === 'import'
        ? html`<${ImportFlow} signup=${session.signup} onDone=${onSignedIn} onBack=${() => setPage('sign_up')} />`
        : html`<${SignUpPage} signup=${session.signup} onDone=${done} onSignIn=${() => setPage('sign_in')}
          onImport=${() => setPage('import')} />`;
}

// The unlock sheet (ADR 0056 items 7 and 10). `onDone(true)` follows an
// unlock. `onDone(false)` follows a close of the sheet.
function UnlockSheet({ session, reason, onDone }) {
  const ref = useSheet(true);
  const [passphrase, setPassphrase] = useState('');
  const { busy, error, run } = useAction();
  const settled = useRef(false);
  const finish = (ok) => {
    if (settled.current) return;
    settled.current = true;
    onDone(ok);
  };
  const methods = unlockMethods(session, !!window.nostr, hasPasskeys);
  const byPassphrase = (event) => {
    event.preventDefault();
    run(async () => {
      await api('/vault/unlock', { method: 'POST', body: { passphrase } });
      finish(true);
    });
  };
  const bySigner = () => run(async () => {
    const wrapper = await api('/vault/nostr-wrapper');
    let key;
    try {
      key = await window.nostr.nip44.decrypt(wrapper.pubkey, wrapper.ciphertext);
    } catch (_) {
      throw new ApiError(0, 'signer', 'The signer did not decrypt the vault key.');
    }
    await api('/vault/unlock-key', { method: 'POST', body: { key } });
    finish(true);
  });
  const byPasskey = () => run(async () => {
    await passkeyAuth();
    finish(true);
  });
  const byRemoteSigner = () => run(async () => {
    await authCall('/nip46/unlock', { body: {} });
    finish(true);
  });
  const passError = splitError(error, ['wrong_key']);
  // A password manager finds its entry by the username. The form thus holds
  // the email of the account in a hidden field, as the sign-in form does.
  const signerError = error && error.code === 'wrong_key' && methods.includes('nip07') && !methods.includes('passphrase') ? error : null;
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="unlock-title" onClose=${() => finish(false)}>
    <header>
      <h2 id="unlock-title">Unlock Studio</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body">
      <p>This step needs the vault. ${serverMode ? 'Your passphrase, your signer or your passkey opens it.' : 'Your passphrase opens it.'}</p>
      ${reason && html`<p class="hint" role="status">${reason}</p>`}
      ${methods.includes('passphrase') && html`<form class="form" onSubmit=${byPassphrase}>
        ${session && session.email && html`<input class="visually-hidden" type="email" autocomplete="username" value=${session.email}
          readonly tabindex="-1" aria-hidden="true" />`}
        <${SecretField} id="unlock-sheet-passphrase" label="Passphrase" autocomplete="current-password" required autofocus
          error=${passError.field} value=${passphrase} onInput=${(e) => setPassphrase(e.target.value)} />
        <div class="actions"><button class="button primary" type="submit" disabled=${busy || !passphrase}>${busy ? 'Unlocking…' : 'Unlock'}</button></div>
      </form>`}
      ${methods.includes('nip07') && html`<div class="actions">
        <button class="button action" type="button" disabled=${busy} onClick=${bySigner}>Unlock with a Signer Extension</button></div>`}
      ${methods.includes('nip46') && html`<div class="actions">
        <button class="button action" type="button" disabled=${busy} onClick=${byRemoteSigner}>Use the Remote Signer</button></div>
        ${busy && html`<p class="hint" role="status">Allow the request in your signer app.</p>`}`}
      ${methods.includes('passkey') && html`<div class="actions">
        <button class="button action" type="button" disabled=${busy} onClick=${byPasskey}>Use a Passkey</button></div>`}
      ${methods.length === 0 && html`<p class="hint">This account opens with a Nostr signer. Studio found no signer extension in this browser.</p>`}
      <${AuthNotice} error=${signerError || passError.other} />
    </div>
    <footer><button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button></footer>
  </dialog>`;
}

// The session of the signed-in user, and the function that reloads it.
export const SessionContext = createContext(null);

// The passkeys of the account, with "Add a passkey" and "Remove" (ADR 0056
// items 3 and 20 to 23). The browser must support passkeys.
function PasskeyList() {
  const { reload } = useContext(SessionContext);
  const [list, setList] = useState(null);
  const [name, setName] = useState('');
  const { busy, error, run } = useAction();
  const load = useCallback(async () => {
    try {
      const next = await authCall('/passkeys', { method: 'GET' });
      setList(next);
      setName((current) => current || defaultPasskeyName(next));
    } catch (_) {
      setList([]);
    }
  }, []);
  useEffect(() => { load(); }, [load]);
  const add = (event) => {
    event.preventDefault();
    run(async () => {
      await registerPasskey(name.trim() || defaultPasskeyName(list));
      setName('');
      await load();
      await reload();
    });
  };
  const remove = (item) => run(async () => {
    await withUnlock(() => authCall(`/passkeys/${item.credential_id}`, { method: 'DELETE' }));
    setName('');
    await load();
    await reload();
  });
  return html`<form class="form" onSubmit=${add}>
    <h3>Passkeys</h3>
    ${list && list.length > 0 && html`<ul class="passkey-list">
      ${list.map((item) => html`<li key=${item.credential_id}>
        <span>${item.name}<span class="hint"> · added ${new Date(item.created_at * 1000).toLocaleDateString()}</span></span>
        <button class="button destructive" type="button" disabled=${busy} onClick=${() => remove(item)}>Remove</button>
      </li>`)}
    </ul>`}
    ${list && list.length === 0 && html`<p class="hint">This account has no passkey.</p>`}
    <${Field} id="passkey-name" label="Name of the new passkey" autocomplete="off" value=${name}
      onInput=${(e) => setName(e.target.value)} />
    <div class="actions"><button class="button action" type="submit" disabled=${busy}>${busy ? 'Waiting for the Passkey…' : 'Add a Passkey'}</button></div>
    <${AuthNotice} error=${error} />
  </form>`;
}

// The Account panel of the Settings page, on a server only (ADR 0055 item 9).
function AccountPanel() {
  const { session, reload } = useContext(SessionContext);
  const [deleting, setDeleting] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [typed, setTyped] = useState('');
  const { busy, error, run } = useAction();
  const confirmation = deleteConfirmation(session, typed);
  const closeNotice = () => run(async () => {
    await authCall('/import-notice/close');
    await reload();
  });
  const byEmail = !!session.email;
  const signOut = () => run(async () => {
    await authCall('/signout');
    await reload();
  });
  const remove = (event) => {
    event.preventDefault();
    if (!confirmation) return;
    run(async () => {
      await withUnlock(() => authCall('/account/delete', { body: { confirm: confirmation } }));
      await reload();
    });
  };
  return html`<section class="page-section settings-section">
    <h2>Account</h2><div class="group settings-group">
    <dl class="facts"><dt>${byEmail ? 'Email address' : 'Nostr key'}</dt><dd class="mono">${accountName(session)}</dd></dl>
    ${showsImportNotice(session) && html`<div class="notice" role="status">
      <strong>${importedNotice({ origin: location.origin, passkeys: hasPasskeys }).strong}</strong>
      ${importedNotice({ origin: location.origin, passkeys: hasPasskeys }).text}
      <div class="actions"><button class="button" type="button" disabled=${busy} onClick=${closeNotice}>Close</button></div>
    </div>`}
    <div class="form"><h3>Move this account</h3>
      <p class="hint">Make one file with your feeds, drafts, versions, people and vault. To import it, select Import an Account when you make an account on a different server, or in a new Studio desktop before you add a feed.</p>
      <div class="actions"><button class="button action" type="button" disabled=${busy} onClick=${() => setExporting(true)}>Export Account</button></div></div>
    ${exporting && html`<${ExportSheet} onClose=${() => setExporting(false)} />`}
    ${hasPasskeys && html`<${PasskeyList} />`}
    ${deleting
      ? html`<form class="form" onSubmit=${remove}>
          <p class="hint">Studio removes your feeds, your vault and your library files from the server. This cannot be undone.</p>
          <${Field} id="delete-confirm" label=${`Type ${byEmail ? 'your email address' : 'your npub'} to confirm`}
            autocomplete="off" value=${typed} onInput=${(e) => setTyped(e.target.value)} />
          <div class="actions">
            <button class="button destructive" type="submit" disabled=${busy || !confirmation}>${busy ? 'Deleting…' : 'Yes, Delete the Account'}</button>
            <button class="button" type="button" disabled=${busy} onClick=${() => { setDeleting(false); setTyped(''); }}>Cancel</button>
          </div>
        </form>`
      : html`<div class="actions">
          <button class="button" type="button" disabled=${busy} onClick=${signOut}>Sign Out</button>
          <button class="button destructive" type="button" disabled=${busy} onClick=${() => setDeleting(true)}>Delete the Account</button>
        </div>`}
    <${AuthNotice} error=${error} />
  </div></section>`;
}

// ---- Export and import of an account (ADR 0107) ----

// The sheet that exports the account (screens 1 to 3). It reads the facts,
// asks for a recovery key when no method opens the vault on a different
// server, and then starts the download of the archive. `folder` is the folder
// of a collection that is not open (ADR 0176 item 7), or null for the account
// that is open.
function ExportSheet({ onClose, folder = null }) {
  const ref = useSheet(true);
  const [facts, setFacts] = useState(null);
  const [include, setInclude] = useState(true);
  const [recoveryKey, setRecoveryKey] = useState(null);
  const [kept, setKept] = useState(false);
  const [done, setDone] = useState(false);
  const { busy, error, run } = useAction();
  const factsPath = folder ? `/collections/${folder}/export-facts` : '/account/export-facts';
  const load = () => run(async () => { setFacts(await api(factsPath)); });
  useEffect(() => { load(); }, []);
  const view = facts && exportView(facts, { server: serverMode, includeAssets: include, collection: hasCollections });
  const makeKey = () => run(async () => {
    const made = await api('/vault/recovery-key', { method: 'POST' });
    setKept(false);
    setRecoveryKey(made.recovery_key);
  });
  // The vault can move now, so the sheet reads the facts again.
  const afterKey = () => run(async () => {
    setFacts(await api(factsPath));
    setRecoveryKey(null);
  });
  const send = () => run(async () => {
    const made = await api(folder ? `/collections/${folder}/export` : '/account/export', { method: 'POST', body: { include_assets: view.checkbox ? include : true } });
    saveTicket(made.ticket, made.file_name);
    setDone(true);
  });
  const title = view ? view.title : exportTitle(serverMode, hasCollections);
  let body = null;
  let footer = html`<button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>`;
  if (done) {
    body = html`<div class="notice" role="status"><strong>${EXPORT_DONE.strong}</strong> ${EXPORT_DONE.text}</div>`;
    footer = html`<button class="button primary" type="button" onClick=${() => ref.current.close()}>Close</button>`;
  } else if (recoveryKey) {
    body = html`<div class="form">
      <p>${RECOVERY_KEY_TEXT}</p>
      <${SecretText} id="export-recovery-key" className="recovery" label="Recovery key" name="recovery key" value=${recoveryKey} />
      <label class="check"><input type="checkbox" checked=${kept} onChange=${(e) => setKept(e.target.checked)} />
        I wrote down the recovery key in a safe place</label></div>`;
    footer = html`<button class="button primary" type="button" disabled=${busy || !kept} onClick=${afterKey}>Continue</button>`;
  } else if (view) {
    body = html`${view.intro && html`<p>${view.intro}</p>`}
      <div class=${`notice${view.error ? ' error' : ''}`} role="status"><strong>${view.notice.strong}</strong> ${view.notice.text}</div>
      ${view.rows.length > 0 && html`<div class="group rows">${view.rows.map((row) => html`<div key=${row.label} class="link-row static">
        <span class="text"><span class="label">${row.label}</span><span class="summary">${row.summary}</span></span></div>`)}</div>`}
      ${view.checkbox && html`<label class="check"><input type="checkbox" checked=${include} onChange=${(e) => setInclude(e.target.checked)} /> ${view.checkbox}</label>`}
      ${view.size && html`<p class="hint">${view.size}</p>`}
      <p class="hint">${folder && view.screen === 'warn' ? CLOSED_EXPORT_HINT : view.hint}</p>`;
    footer = html`<button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>
      ${!(folder && view.screen === 'warn') && html`<button class="button primary" type="button" disabled=${busy}
        onClick=${view.screen === 'warn' ? makeKey : send}>${view.primary}</button>`}`;
  } else if (!error) {
    body = html`<p class="empty" role="status">Reading the account…</p>`;
  }
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="export-title" onClose=${onClose}>
    <header>
      <h2 id="export-title">${title}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body">${body}<${ErrorNotice} error=${error} /></div>
    <footer>${footer}</footer>
  </dialog>`;
}

// The section "UI scale" of the Settings page (ADR 0134). With touch, the
// system gives the size, so the section does not show.
function ScalePanel({ draft, changed }) {
  const [level, setLevel] = useState(() => normalizeScale(document.documentElement.dataset.uiScale));
  if (matchMedia('(pointer: coarse)').matches) return null;
  const change = (next) => {
    document.documentElement.dataset.uiScale = next;
    draft('scale', normalizeScale(localStorage.getItem(SCALE_KEY)), next);
    setLevel(next);
  };
  return html`<section class="page-section settings-section">
    <h2>Appearance</h2><div class="group settings-group">
    <div class="field setting-row"><label for="ui-scale">UI scale<${SettingsPencil} pending=${changed('scale')} /></label>
      <div class="field-body">
        <span class="hint">The size of Studio on this computer.</span>
        <select id="ui-scale" class="button" value=${level} onChange=${(e) => change(e.target.value)}>
          ${UI_SCALES.map(([id, name]) => html`<option value=${id}>${name}</option>`)}
        </select>
      </div>
    </div>
  </div></section>`;
}

// The time before the vault locks (ADR 0142). The core gives the times.
function LockPanel({ draft, draftState, changed, causes }) {
  const [setting, setSetting] = useState(null);
  const { busy, error, run } = useAction();
  useEffect(() => { run(async () => {
    const setting = await api('/settings/lock');
    setSetting(draftState.values.lock === undefined ? setting : { ...setting, minutes: Number(draftState.values.lock) });
  }); }, []);
  const change = (value) => { draft('lock', String(setting.minutes), String(value)); setSetting({ ...setting, minutes: Number(value) }); };
  return html`<section class="page-section settings-section">
    <h2>Lock</h2><div class="group settings-group">
    <${ErrorNotice} error=${error} />
    ${setting && html`<div class="field setting-row"><label for="lock-minutes">Lock Studio after<${SettingsPencil} pending=${changed('lock')} /></label>
      <div class="field-body">
        <span class="hint">${LOCK_HINT}</span>
        <select id="lock-minutes" class="button" disabled=${busy} value=${String(setting.minutes)} onChange=${(e) => change(e.target.value)}>
          ${setting.choices.map((minutes) => html`<option value=${String(minutes)}>${lockChoiceText(minutes)}</option>`)}
        </select>
        ${causes.lock && html`<span class="field-error">${causes.lock}</span>`}
      </div>
    </div>`}
  </div></section>`;
}

// What a close of the window does (ADR 0124 item 29). The route says if this
// shell has a window, so a browser shows no section.
function WindowPanel({ draft, draftState, changed, causes }) {
  const [state, setState] = useState(null);
  useEffect(() => { api('/window-close').then((state) => setState(draftState.values.window === undefined ? state : { ...state, on_close: draftState.values.window }), () => {}); }, []);
  if (!state || !state.available) return null;
  const change = (onClose) => { draft('window', state.on_close || 'tray', onClose); setState({ ...state, on_close: onClose }); };
  return html`<section class="page-section settings-section">
    <h2>Window</h2><div class="group settings-group">
    <div class="field setting-row"><label for="window-close">When I close the window<${SettingsPencil} pending=${changed('window')} /></label>
      <div class="field-body">
        <span class="hint">A connected app can only write while Studio runs. If your desktop has no tray, Studio quits.</span>
        <select id="window-close" class="button" value=${state.on_close || 'tray'} onChange=${(e) => change(e.target.value)}>
          <option value="tray">Keep Studio in the tray</option>
          <option value="quit">Quit Studio</option>
        </select>
        ${causes.window && html`<span class="field-error">${causes.window}</span>`}
      </div>
    </div>
  </div></section>`;
}

// The one store of the results of the check for a change from outside (ADR
// 0165 items 5 and 6). It checks each feed after the list loads, and again
// each time the Releases page opens (`key` changes). At most 4 reads run at
// one time (ADR 0150 item 11). No worker runs. A failed request keeps the
// result before it.
function useOutsideChecks(feeds, key) {
  const [results, setResults] = useState({});
  const signature = feeds === null ? null : feeds.map((feed) => feed.id).join(',');
  useEffect(() => {
    if (signature === null || signature === '') return undefined;
    let live = true;
    eachLimited(signature.split(',').map(Number), OUTSIDE_LIMIT, async (id) => {
      const check = await api(`/feeds/${id}/outside`);
      if (live) setResults((old) => ({ ...old, [id]: check }));
    });
    return () => { live = false; };
  }, [signature, key]);
  return results;
}

// The notices of the check on a feed page (ADR 0150 items 2 to 5). Each new
// read of the feed checks again.
function OutsideNotices({ feed, busy, refresh }) {
  const [check, setCheck] = useState(null);
  const { error, run } = useAction();
  const load = () => api(`/feeds/${feed.id}/outside?index=true`).then(setCheck, () => {});
  useEffect(() => { setCheck(null); load(); }, [feed.id, feed.last_version]);
  const notice = outsideNotice(check);
  const index = indexNotice(check);
  const keep = () => run(async () => {
    await api(`/feeds/${feed.id}/outside/keep`, { method: 'POST' });
    await load();
  });
  return html`${notice && html`<div class=${notice.tone === 'error' ? 'notice error' : 'notice'} role=${notice.tone === 'error' ? 'alert' : 'status'}>
      <strong>${notice.strong}</strong> ${notice.text}${notice.address && html` <span class="mono">${notice.address}</span>.`}${notice.after && ` ${notice.after}`}
      <div class="actions">
        ${notice.again && html`<button class="button action" type="button" disabled=${busy} onClick=${refresh}>Read Again</button>`}
        ${notice.difference && html`<button class="button action" type="button" onClick=${() => window.dispatchEvent(new Event(OPEN_MERGE))}>Show the Difference</button>`}
        ${notice.keep && html`<button class="button" type="button" onClick=${keep}>Keep This File Host</button>`}
      </div>
      <${ErrorNotice} error=${error} />
    </div>`}
    ${index && html`<div class="notice" role="status"><strong>${index.strong}</strong> <span class="mono">${index.index}</span>. Your feed is at <span class="mono">${index.own}</span>. ${index.after}</div>`}`;
}

// The check box of the check for a change from outside (ADR 0150 item 10).
function OutsidePanel({ draft, draftState, changed, causes }) {
  const [enabled, setEnabled] = useState(null);
  useEffect(() => { api('/settings/outside-check').then((r) => setEnabled(draftState.values.outside ?? r.enabled), () => {}); }, []);
  if (enabled === null) return null;
  const change = (next) => { draft('outside', enabled, next); setEnabled(next); };
  return html`<section class="page-section settings-section">
    <h2>Outside changes</h2><div class="group settings-group">
    <label class="check"><input type="checkbox" checked=${enabled} onChange=${(e) => change(e.target.checked)} />
      ${OUTSIDE_SETTING_LABEL}<${SettingsPencil} pending=${changed('outside')} /></label>
    <p class="hint">${OUTSIDE_SETTING_HINT}</p>
    ${causes.outside && html`<span class="field-error">${causes.outside}</span>`}
  </div></section>`;
}

const COLLECTIONS_CHANGED = 'studio-collections-changed';

// The text of the mark of a collection in the left panel (ADR 0176 item 12).
// A shell with no collections sends no request.
function useCollectionMark() {
  const [mark, setMark] = useState(null);
  useEffect(() => {
    if (!hasCollections) return undefined;
    const read = () => api('/collections').then((r) => setMark(collectionMark(r.collections)), () => {});
    read();
    // The section "Collections" tells the panel after a change of the list.
    addEventListener(COLLECTIONS_CHANGED, read);
    return () => removeEventListener(COLLECTIONS_CHANGED, read);
  }, []);
  return mark;
}

// The sheet that makes a collection, and then opens it (ADR 0149 items 7 to
// 11 and 14). `kind` is `new`, `copy` or `import`, or null.
function CollectionSheet({ kind, openName, onDone, onClose }) {
  const ref = useSheet(kind !== null);
  const [size, setSize] = useState(null);
  const [name, setName] = useState('');
  const { busy, error, run } = useAction();
  useEffect(() => {
    if (kind === null) return;
    setName(kind === 'copy' ? `${openName} copy` : '');
    setSize(null);
    if (kind === 'copy') api('/collections/copy-size').then((r) => setSize(sizeText(r.size)), () => {});
  }, [kind]);
  if (kind === null) return html`<dialog ref=${ref} class="sheet small" onClose=${onClose}></dialog>`;
  const words = collectionSheet(kind, openName, size || '…');
  const make = (e) => {
    e.preventDefault();
    run(async () => {
      const made = await api(kind === 'copy' ? '/collections/copy' : '/collections', { method: 'POST', body: { name } });
      onDone(made);
    });
  };
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="collection-title" onClose=${onClose}>
    <header>
      <h2 id="collection-title">${words.title}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <form class="body" id="collection-form" onSubmit=${make}>
      <div class="group"><${Field} id="collection-name" label="Name" value=${name} onInput=${(e) => setName(e.target.value)} /></div>
      ${words.lines.map((line) => html`<p class="hint" key=${line}>${line}</p>`)}
      ${words.warning && html`<div class="notice error" role="alert"><strong>${words.warning.strong}</strong> ${words.warning.text}</div>`}
      <${ErrorNotice} error=${error} />
    </form>
    <footer><button class="button primary" type="submit" form="collection-form" disabled=${busy || name.trim() === ''}>
      ${busy ? words.busy : words.primary}</button></footer>
  </dialog>`;
}

// The sheet of a row: rename, replace or delete (ADR 0176 items 5 to 9). The
// core refuses a replace and a delete of the open collection, and the row has
// no such button.
function CollectionActionSheet({ kind, collection, onDone, onClose }) {
  const ref = useSheet(true);
  const words = collectionAction(kind, collection);
  const [name, setName] = useState(words.start || '');
  const [file, setFile] = useState(null);
  const [exporting, setExporting] = useState(false);
  const picker = useRef(null);
  const { busy, error, run } = useAction();
  const submit = (e) => {
    e.preventDefault();
    run(async () => {
      if (kind === 'rename') await api(`/collections/${collection.folder}`, { method: 'PATCH', body: { name } });
      else if (kind === 'replace') await sendReplace(collection.folder, file);
      else await api(`/collections/${collection.folder}`, { method: 'DELETE' });
      onDone();
    });
  };
  const ready = kind === 'rename' ? name.trim() !== '' : kind !== 'replace' || file !== null;
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="collection-action-title" onClose=${onClose}>
    <header>
      <h2 id="collection-action-title">${words.title}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <form class="body" id="collection-action-form" onSubmit=${submit}>
      ${kind === 'rename' && html`<div class="group"><${Field} id="collection-name" label="Name" value=${name} onInput=${(e) => setName(e.target.value)} /></div>`}
      ${words.lines.map((line) => html`<p class="hint" key=${line}>${line}</p>`)}
      ${words.file && html`<input ref=${picker} type="file" accept=".tar,application/x-tar" hidden
        onChange=${(e) => setFile(e.target.files[0] || null)} />
        <div class="group rows"><div class="link-row static">
          <span class="text"><span class="label">${file ? file.name : 'No file selected'}</span>${file && html`<span class="summary">${sizeText(file.size)}</span>`}</span>
          <div class="actions"><button class="button action" type="button" disabled=${busy} onClick=${() => picker.current.click()}>Select the File</button></div>
        </div></div>`}
      <${ErrorNotice} error=${error} />
    </form>
    <footer>
      ${words.exportFirst && html`<button class="button action" type="button" disabled=${busy} onClick=${() => setExporting(true)}>Export First…</button>`}
      <button class=${`button ${words.destructive ? 'destructive' : 'primary'}`} type="submit" form="collection-action-form" disabled=${busy || !ready}>
        ${busy ? words.busy : words.primary}</button>
    </footer>
    ${exporting && html`<${ExportSheet} folder=${collection.folder} onClose=${() => setExporting(false)} />`}
  </dialog>`;
}

// The section "Collections" of the Settings page, in the desktop app only
// (ADR 0149 items 5 and 6, ADR 0176 items 2, 3 and 10). The core keeps the
// list and applies each rule.
function CollectionsPanel() {
  const [list, setList] = useState(null);
  const [sheet, setSheet] = useState(null);
  const [action, setAction] = useState(null);
  const [exporting, setExporting] = useState(false);
  const [opening, setOpening] = useState(false);
  const { error, run } = useAction();
  const load = () => api('/collections').then((r) => {
    setList(r.collections);
    dispatchEvent(new Event(COLLECTIONS_CHANGED));
  });
  useEffect(() => { if (hasCollections) load().catch(() => {}); }, []);
  if (!list) return null;
  const openName = (list.find((c) => c.open) || {}).name || '';
  // The quit rules can stop a restart, so the text goes after a short time.
  const open = (folder) => run(async () => {
    await api(`/collections/${folder}/open`, { method: 'POST' });
    setOpening(true);
    setTimeout(() => setOpening(false), 10000);
  });
  return html`<section class="page-section settings-section">
    <h2>Collections</h2><div class="group settings-group collections-group">
    <p class="hint">${COLLECTIONS_HINT}</p>
    <div class="group collection-rows">
      ${list.map((collection) => {
        const row = collectionRow(collection);
        const choose = (kind) => setAction({ kind, collection });
        return html`<div class="link-row static collection-row" key=${collection.folder}>
          <span class="text"><span class="label">${row.name}${row.badge && html` <span class="default-tag"><${Icon} name="check" />${row.badge}</span>`}</span><span class="summary">${row.line}</span></span>
          <${MenuControl} buttonLabel="Actions" accessibleName=${`Actions for ${collection.name}`}
            menuLabel=${`Actions for ${collection.name}`} items=${row.actions.map((entry) => ({
              ...entry,
              role: 'menuitem',
            }))} onSelect=${(entry) => {
              if (entry.action === 'open') open(collection.folder);
              else if (entry.action === 'export') setExporting(collection);
              else if (entry.action === 'copy') setSheet('copy');
              else choose(entry.action);
            }} />
        </div>`;
      })}
    </div>
    ${opening && html`<p class="hint" role="status">${OPENING_TEXT}</p>`}
    <${ErrorNotice} error=${error} />
    <div class="actions">
      ${collectionSectionActions().map((entry) => html`<button class="button action" type="button"
        onClick=${() => setSheet(entry.action)}>${entry.label}</button>`)}
    </div>
    <${CollectionSheet} kind=${sheet} openName=${openName} onClose=${() => setSheet(null)}
      onDone=${async (made) => { setSheet(null); await load(); await open(made.folder); }} />
    ${action && html`<${CollectionActionSheet} kind=${action.kind} collection=${action.collection}
      onClose=${() => setAction(null)} onDone=${async () => { setAction(null); await load(); }} />`}
    ${exporting !== false && html`<${ExportSheet} folder=${exporting && !exporting.open ? exporting.folder : null}
      onClose=${() => setExporting(false)} />`}
  </div></section>`;
}

// The section "Move Studio" of the Settings page on the desktop (ADR 0107
// item 3). A server has "Export Account" in the Account panel.
function MovePanel() {
  const [open, setOpen] = useState(false);
  return html`<section class="page-section settings-section">
    <h2>Move Studio</h2><div class="group settings-group">
    <p class="hint">Make one file with your feeds, drafts, versions, people and vault. To import it, select Import an Account in a new Studio before you add a feed, or when you make an account on a server.</p>
    <div class="actions"><button class="button action" type="button" onClick=${() => setOpen(true)}>Export Studio</button></div>
    ${open && html`<${ExportSheet} onClose=${() => setOpen(false)} />`}
  </div></section>`;
}

// A call of an import route: the auth routes of a server need no session,
// and the desktop uses its API with the access token.
function importCall(path, options = {}) {
  return serverMode
    ? exchange(`/auth/v1/import${path}`, {}, { method: 'POST', ...options })
    : api(`/import${path}`, { method: 'POST', ...options });
}

// Send a file in parts (ADR 0059). `routes` gives the calls of the upload:
// `start`, `part(upload, n)` and `cancel(upload)`. The result is the ID of the
// upload. A failed part is sent again like the part of a feed upload.
async function sendParts(file, routes, onProgress) {
  const made = await routes.start();
  try {
    for (let n = 0; n < partCount(file.size, made.part_size); n += 1) {
      const range = partRange(n, file.size, made.part_size);
      for (let tries = 0; ; tries += 1) {
        try {
          await routes.part(made.upload, n, file.slice(range.start, range.end));
          break;
        } catch (e) {
          if (!canTryAgain(e) || tries >= RETRY_DELAYS.length) throw e;
          await wait(RETRY_DELAYS[tries]);
        }
      }
      onProgress(range.end);
    }
  } catch (e) {
    routes.cancel(made.upload).catch(() => {});
    throw e;
  }
  return made.upload;
}

// Send the archive of an import. The result is the ID of the upload.
function sendImport(file, invite, onProgress) {
  return sendParts(file, {
    start: () => importCall('', { body: { size: file.size, invite: invite || null } }),
    part: (upload, n, body) => importCall(`/${upload}/parts/${n}`, { method: 'PUT', body }),
    cancel: (upload) => importCall(`/${upload}`, { method: 'DELETE' }),
  }, onProgress);
}

// Send the account file for the replace of a collection, and read it into
// a new folder (ADR 0176 item 8). A file that is not good changes nothing.
async function sendReplace(folder, file) {
  const upload = await sendParts(file, {
    start: () => api(`/collections/${folder}/replace`, { method: 'POST', body: { size: file.size } }),
    part: (id, n, body) => api(`/collection-uploads/${id}/parts/${n}`, { method: 'PUT', body }),
    cancel: (id) => api(`/collection-uploads/${id}`, { method: 'DELETE' }),
  }, () => {});
  return api(`/collection-uploads/${upload}/replace`, { method: 'POST', body: { folder } });
}

// The frame of an import page: a page with a card on a server, a sheet on the
// desktop.
function ImportShell({ title, intro, onClose, children }) {
  const ref = useSheet(true);
  if (serverMode) {
    return html`<main class="center"><div class="card">
      <${Brand} />
      <div><h1>${title}</h1><p>${intro}</p></div>
      ${children}
    </div></main>`;
  }
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="import-title" onClose=${onClose}>
    <header>
      <h2 id="import-title">${title}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body"><p>${intro}</p>${children}</div>
  </dialog>`;
}

// "Import an Account" (ADR 0107 items 9 to 14, screens 5 to 7). The first
// step selects the file and sends it. The second step opens the vault of the
// file. On a server, `onDone` follows the new session. The desktop reads its
// state again.
function ImportFlow({ signup, onDone, onBack }) {
  const server = serverMode;
  const needInvite = server && signup === 'invite';
  const [invite, setInvite] = useState('');
  const [sending, setSending] = useState(null);
  const [check, setCheck] = useState(null);
  const [mode, setMode] = useState(null);
  const [passphrase, setPassphrase] = useState('');
  const [recoveryKey, setRecoveryKey] = useState('');
  const [email, setEmail] = useState('');
  const [remote, setRemote] = useState(false);
  const picker = useRef(null);
  const pick = useAction();
  const open = useAction();

  const [dropError, setDropError] = useState(null);
  const choose = (event) => {
    const file = event.target.files && event.target.files[0];
    event.target.value = '';
    if (file) sendFile(file);
  };
  const dropFile = (files) => {
    const result = accountDropResult(files);
    setDropError(result.error || null);
    if (result.file) sendFile(result.file);
  };
  const accountBusy = pick.busy || (needInvite && invite.trim() === '');
  const accountDrop = useFileDrop({ busy: accountBusy, onFiles: dropFile, stop: true });
  const sendFile = (file) => {
    setDropError(null);
    pick.run(async () => {
      setSending({ name: file.name, size: file.size, sent: 0, checking: false });
      try {
        const upload = await sendImport(file, needInvite ? invite.trim() : null,
          (sent) => setSending((current) => ({ ...current, sent })));
        setSending((current) => ({ ...current, checking: true }));
        setCheck(await importCall(`/${upload}/check`));
      } catch (e) {
        setSending(null);
        throw e;
      }
    });
  };

  // End the import that waits, so that the server removes the file.
  const leave = () => {
    if (check) importCall(`/${check.upload}`, { method: 'DELETE' }).catch(() => {});
    onBack();
  };

  const view = check && importOpenView(check, { server, hasNip07: !!window.nostr, mode });
  const finish = async (credential) => {
    const body = { ...credential };
    if (view.needsEmail && (credential.passphrase !== undefined || credential.recovery_key !== undefined)) {
      body.email = normalEmail(email);
    }
    const result = await importCall(`/${check.upload}/open`, { body });
    if (server) {
      csrfToken = result.csrf;
      onDone();
    } else {
      location.reload();
    }
  };
  const submit = (event) => {
    event.preventDefault();
    open.run(() => finish(view.secret === 'recovery' ? { recovery_key: recoveryKey.trim() } : { passphrase }));
  };
  const bySigner = () => open.run(async () => {
    const signer = window.nostr;
    if (!signer.nip44) {
      throw new ApiError(0, 'signer', 'This signer does not support NIP-44. Studio needs NIP-44 to open the vault.');
    }
    let pubkey;
    let key;
    try {
      pubkey = await signer.getPublicKey();
      const wrapper = check.nostr_wrappers.find((w) => w.pubkey === pubkey);
      if (!wrapper) throw new ApiError(0, 'pubkey_not_in_archive', importErrorText('pubkey_not_in_archive'));
      key = await signer.nip44.decrypt(pubkey, wrapper.ciphertext);
    } catch (e) {
      if (e instanceof ApiError) throw e;
      throw new ApiError(0, 'signer', 'The signer did not decrypt the vault key.');
    }
    await finish(server ? { key, pubkey } : { key });
  });

  if (remote) {
    return html`<${Nip46Page} title="Open the vault with Amber or a bunker" invite=${null} recoveryKey=${false}
      onConnected=${(connection) => finish({ connection })} onDone=${() => {}} onBack=${() => setRemote(false)} />`;
  }

  if (!check) {
    const row = sending && { label: sending.name, summary: sending.checking ? CHECKING_TEXT : sendingText(sending.sent, sending.size) };
    return html`<${ImportShell} title="Import an account" intro=${importSelectText(server)} onClose=${onBack}>
      <div class="form">
        ${needInvite && html`<${Field} id="import-invite" label="Invite code" autocomplete="off" required
          value=${invite} onInput=${(e) => setInvite(e.target.value)} />`}
        <input ref=${picker} type="file" accept=".tar,application/x-tar" hidden onChange=${choose} />
        <div class=${dropAreaClass(accountDrop.over, accountBusy)} ...${accountDrop.handlers}>
          <p class="hint drop-text">Drop the account file here.</p>
          <button class="button action" type="button" disabled=${accountBusy}
            onClick=${() => picker.current.click()}>Select the File</button>
          ${dropError && html`<p class="field-error" role="alert">${dropError}</p>`}
        </div>
        ${row && html`<div class="group rows"><div class="link-row static">
          <span class="text"><span class="label">${row.label}</span><span class="summary">${row.summary}</span></span></div></div>`}
        <${AuthNotice} error=${pick.error} importing=${true} />
        <p class="hint">${IMPORT_HINT}</p>
        <button class="button link" type="button" onClick=${onBack}>${server ? 'Back to Sign Up' : 'Cancel'}</button>
      </div>
    <//>`;
  }

  return html`<${ImportShell} title="Open the vault of the file" intro=${view.intro} onClose=${leave}>
    <form class="form" onSubmit=${submit}>
      <div class="group rows">${importFacts(check).map((fact) => html`<div key=${fact.label} class="link-row static">
        <span class="text"><span class="label">${fact.label}</span><span class="summary">${fact.summary}</span></span></div>`)}</div>
      ${view.needsEmail && html`<${Field} id="import-email" label="Email address" type="email" autocomplete="username" required
        value=${email} onInput=${(e) => setEmail(e.target.value)} />`}
      ${view.secret === 'passphrase' && html`<${SecretField} id="import-passphrase" label="Passphrase" autocomplete="current-password" required autofocus
        value=${passphrase} onInput=${(e) => setPassphrase(e.target.value)} />`}
      ${view.secret === 'recovery' && html`<${SecretField} id="import-recovery-key" label="Recovery key" autocomplete="off" required autofocus
        value=${recoveryKey} onInput=${(e) => setRecoveryKey(e.target.value)} />`}
      ${view.primary && html`<div class="actions"><button class="button primary" type="submit"
        disabled=${open.busy || (view.secret === 'passphrase' ? !passphrase : !recoveryKey.trim()) || (view.needsEmail && !email.trim())}>${open.busy ? 'Opening the Vault…' : view.primary}</button></div>`}
      ${view.others.map((other) => html`<div key=${other.id} class="actions"><button class="button action" type="button" disabled=${open.busy}
        onClick=${other.id === 'nip07' ? bySigner : other.id === 'nip46' ? () => setRemote(true) : () => setMode(other.id)}>${other.label}</button></div>`)}
      ${view.noMethod && html`<p class="hint">This browser has no way to open the vault of the file. Use a signer extension or a different computer.</p>`}
      <${AuthNotice} error=${open.error} importing=${true} />
      <button class="button link" type="button" onClick=${leave}>Cancel</button>
    </form>
  <//>`;
}

// ---- App ----

// The studio of a user: the desktop, or a signed-in user of a server.
// `epoch` changes after an unlock, so that the studio reads its state again.
function Studio({ epoch }) {
  const [status, setStatus] = useState(null);
  const [library, setLibrary] = useState(null);
  const [error, setError] = useState(null);
  const load = useCallback(async () => {
    try {
      const [nextStatus, nextLibrary] = await Promise.all([api('/status'), api('/library')]);
      holdAssets = nextStatus.hold_assets !== false;
      setStatus(nextStatus);
      setLibrary(nextLibrary);
      setError(null);
    } catch (e) {
      setError(e);
    }
  }, []);
  useEffect(() => { if (serverMode || token) load(); }, [load, epoch]);

  if (!serverMode && !token) return html`<${NoToken} />`;
  if (error && error.status === 401) {
    return serverMode ? html`<p class="boot" role="status">Loading Studio…</p>` : html`<${NoToken} />`;
  }
  if (error) return html`<main class="center"><div class="card"><${Brand} /><${ErrorNotice} error=${error} />
    <div class="actions"><button class="button" type="button" onClick=${load}>Try Again</button></div></div></main>`;
  if (!status || !library) return html`<p class="boot" role="status">Loading Studio…</p>`;
  // On a server, a locked vault does not stop the start. A route that needs
  // the vault opens the unlock sheet (ADR 0056 item 9).
  if (status.vault === 'locked' && !serverMode) return html`<${Unlock} onUnlocked=${load} />`;
  // Studio needs its library folder before the editor, so that a write has
  // a release folder to keep the hosted file in (ADR 0036 items 1 and 2).
  if (library.root === null) {
    return html`<${LibrarySheet} defaultRoot=${library.default_root} onDone=${load} />`;
  }
  // A new user with no feed sees the Welcome page in the Shell (ADR 0119
  // item 11). The vault comes when the user first adds a secret (ADR 0020).
  return html`<${Shell} vault=${status.vault} onLock=${load} />`;
}

// Ask the user to confirm an action with a sheet of Studio (musicindex ADR
// 0023 R-CONFIRM-1). `entry` is the result of an entry of `CONFIRMS`. The
// promise gives true for the action button, and false for each other close.
let showConfirm = null;
export function askConfirm(entry) {
  return new Promise((resolve) => {
    if (showConfirm) showConfirm({ entry, resolve });
    else resolve(false);
  });
}

// The sheet of `askConfirm`. It is a modal `dialog`, so it shows above an
// open sheet. The focus starts on "Cancel".
function ConfirmHost() {
  const [ask, setAsk] = useState(null);
  const ref = useRef(null);
  const cancelRef = useRef(null);
  const result = useRef(false);
  const current = useRef(null);
  useEffect(() => {
    showConfirm = (next) => {
      // A new question ends a pending question with "no".
      if (current.current) current.current.resolve(false);
      current.current = next;
      result.current = false;
      setAsk(next);
    };
    return () => { showConfirm = null; };
  }, []);
  useEffect(() => {
    if (!ask || !ref.current) return;
    if (!ref.current.open) ref.current.showModal();
    if (cancelRef.current) cancelRef.current.focus();
  }, [ask]);
  const onClose = () => {
    const done = current.current;
    if (!done) return;
    current.current = null;
    done.resolve(result.current);
    setAsk(null);
  };
  if (!ask) return null;
  const { title, text, action, role, cancel } = ask.entry;
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="confirm-title" onClose=${onClose}>
    <header>
      <h2 id="confirm-title">${title}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body"><p>${text}</p></div>
    <footer>
      <button class="button" type="button" ref=${cancelRef} onClick=${() => ref.current.close()}>${cancel || 'Cancel'}</button>
      <button class=${`button ${role}`} type="button" onClick=${() => { result.current = true; ref.current.close(); }}>${action}</button>
    </footer>
  </dialog>`;
}

function App() {
  const [session, setSession] = useState(null);
  const [bootError, setBootError] = useState(null);
  const [unlockAsk, setUnlockAsk] = useState(null);
  const [epoch, setEpoch] = useState(0);
  const reload = useCallback(async () => {
    try {
      const next = await authCall('/session', { method: 'GET' });
      applySession(next);
      setSession(next);
      setBootError(null);
    } catch (e) {
      setBootError(e);
    }
  }, []);
  useEffect(() => { reload(); }, [reload]);
  // A pending unlock ends when the user is no longer signed in.
  // The sheet shows in the shell only. A request in another view gets its error.
  useEffect(() => {
    const inShell = session && (session.mode === 'local' || (session.signed_in && bootView(session, null) === 'shell'));
    if (unlockAsk && !inShell) { unlockAsk.resolve(false); setUnlockAsk(null); }
  }, [session, unlockAsk]);
  useEffect(() => {
    onUnlockNeeded = (reason) => new Promise((resolve) => setUnlockAsk({ resolve, reason: reason || null }));
    onSessionLost = () => {
      csrfToken = null;
      setSession((current) => (current && current.mode === 'server'
        ? { mode: 'server', signup: current.signup, signed_in: false } : current));
    };
    return () => { onUnlockNeeded = null; onSessionLost = null; };
  }, []);

  if (bootError) {
    return html`<main class="center"><div class="card"><${Brand} /><${ErrorNotice} error=${bootError} />
      <div class="actions"><button class="button" type="button" onClick=${reload}>Try Again</button></div></div></main>`;
  }
  if (!session) return html`<p class="boot" role="status">Loading Studio…</p>`;
  const view = bootView(session, null);
  let body;
  if (view === 'local') {
    body = html`<${Studio} epoch=${0} />`;
  } else if (view === 'sign_in') {
    body = html`<${SignedOut} session=${session} onSignedIn=${reload} />`;
  } else if (view === 'add_method') {
    body = html`<${SessionContext.Provider} value=${{ session, reload }}><${AddMethodPage} /><//>`;
  } else {
    body = html`<${SessionContext.Provider} value=${{ session, reload }}><${Studio} epoch=${epoch} /><//>`;
  }
  const closeUnlock = (ok) => {
    unlockAsk.resolve(ok);
    setUnlockAsk(null);
    if (ok) {
      setEpoch((n) => n + 1);
      window.dispatchEvent(new Event(UNLOCKED));
    }
  };
  return html`${body}<${ConfirmHost} />${unlockAsk && (view === 'shell' || view === 'local') && html`<${UnlockSheet} session=${session} reason=${unlockAsk.reason} onDone=${closeUnlock} />`}`;
}

watchFileDrags(window, document.documentElement);
render(html`<${App} />`, document.getElementById('app'));

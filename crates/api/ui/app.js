// Music Index Studio UI (ADR 0016). Preact and htm, with no build step.
import { html, render, useState, useEffect, useCallback, useRef, useContext, createContext } from './vendor/htm-preact-standalone.module.js';
import { AlbumEditor, DraftToolbar, useSheet } from './editor.js';
import {
  bootView, signInEvent, normalEmail, authErrorText, unlockMethods, accountName, deleteConfirmation,
  recoveryAccount, addMethodChoices, recoveryErrorText,
  toBase64Url, creationOptions, requestOptions, registrationPayload, assertionPayload,
  prfOutputOf, passkeyBrowserCode, defaultPasskeyName,
  nip46State, nip46ErrorText, isBunkerAddress, qrSource,
} from './account.js';
import { isLiveItem } from './live.js';
import { coverOf } from './cover.js';
import { SignerConnect } from './signer.js';
import { useDraft } from './draft.js';
import { RETRY_DELAYS, findUpload, missingParts, partRange, receivedBytes } from './upload.js';
import {
  mediumKind, isEditable, isPublisher, trackKey, isTrack,
  leftPanel, releasesList, listQuery, queryText,
  startNav, pushNav, backNav, forwardNav, canBack, canForward, setScroll,
  isFeedView, openFeedView, navAtWidth,
} from './shell.js';
import {
  npubOf, addressLines, linesText, invalidConfigField, blossomDefaultName, siteAddress, siteNameFromGuid, isNsec,
} from './signing.js';

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
  constructor(status, code, message) {
    super(message);
    this.status = status;
    this.code = code;
  }
}

// ---- The mode of the server (ADR 0055 items 12 and 14 to 17) ----
// `GET /auth/v1/session` tells the UI which shell serves it. On a server,
// `api()` sends the CSRF token of the session in `X-Studio-CSRF` and sends no
// `Authorization` header. The CSRF token stays in memory only.

let serverMode = false;
let csrfToken = null;
// `App` sets these two functions. `onUnlockNeeded` gives a promise that is
// true after an unlock and false when the user closes the unlock sheet.
let onUnlockNeeded = null;
let onSessionLost = null;
let unlocking = null;

// Keep the mode and the CSRF token of a session response.
function applySession(session) {
  serverMode = session.mode === 'server';
  csrfToken = serverMode && session.signed_in !== false ? session.csrf || null : null;
}

// Ask for an unlock. Parallel requests share one sheet.
function askUnlock() {
  if (!onUnlockNeeded) return Promise.resolve(false);
  if (!unlocking) unlocking = onUnlockNeeded().finally(() => { unlocking = null; });
  return unlocking;
}

// Run a request. On a server, the answer 423 `vault_locked` opens the unlock
// sheet, and after an unlock the request runs one more time. If the user
// closes the sheet, the request gives its error. The answer 401
// `session_required` or `session_ended` shows the sign-in page.
async function withUnlock(work) {
  for (let attempt = 0; ; attempt += 1) {
    try {
      return await work();
    } catch (error) {
      if (serverMode && error.status === 423 && error.code === 'vault_locked' && attempt === 0 && await askUnlock()) continue;
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
    throw new ApiError(response.status, error.code, error.message);
  }
  return data;
}

export function api(path, options = {}) {
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

// ---- Images, through the local API (ADR 0025, ADR 0032 items 13 and 14) ----
// The browser cannot put the access token on an <img src> request, so the
// UI asks the API for each image and reads it as a data: address instead.
// The UI loads no image from another server (ADR 0016).

// Each image that the page already read, by its address, for the life of
// the page.
const imageResults = new Map();

// The image at an address: `{ src, failed }`. `src` is a data: address, or
// `null` while the image loads or when Studio could not read it.
export function useImage(url) {
  const [result, setResult] = useState(() => (url ? imageResults.get(url) || null : null));
  useEffect(() => {
    // A new address has no result yet, even when the old address had one.
    // With no reset here, the box would keep the image of the old address
    // until the new one loads or fails.
    if (!url) { setResult(null); return undefined; }
    const cached = imageResults.get(url);
    setResult(cached || null);
    if (cached) return undefined;
    let current = true;
    const finish = (found) => {
      imageResults.set(url, found);
      if (current) setResult(found);
    };
    api(`/images?url=${encodeURIComponent(url)}`, { raw: true }).then((blob) => {
      const reader = new FileReader();
      reader.onload = () => finish({ src: reader.result, failed: false });
      reader.onerror = () => finish({ src: null, failed: true });
      reader.readAsDataURL(blob);
    }, () => finish({ src: null, failed: true }));
    return () => { current = false; };
  }, [url]);
  return result || { src: null, failed: false };
}

// An image at an address, in a box of `className` (ADR 0025). With no
// address, or with an image that Studio cannot read, it shows the current
// mark, tinted with `glow`. For an image that Studio cannot read, the title
// of the box gives its address. `always` keeps the box, with the mark, when
// there is no address: the compact header always shows a cover, but a field
// preview with no address shows nothing (ADR 0032 items 13 and 14).
export function ImageThumb({ url, glow = 'var(--c-feed)', className, always = false }) {
  const { src, failed } = useImage(url);
  if (!url && !always) return null;
  if (src) return html`<div class=${className}><img src=${src} alt="" /></div>`;
  return html`<div class=${className} title=${failed ? url : undefined}>
    <span class="mark" style=${{ '--glow': glow }}></span>
  </div>`;
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
  no_public_address: 'Give this host a public address first, in Hosts.',
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

// ---- Navigation with the browser history ----
// Each screen keeps its view in history.state under its own scope, so that
// the browser Back button goes to the previous step or view.

function useNav(scope, initial) {
  const [view, setView] = useState(() => {
    const state = history.state;
    return state && state.scope === scope ? state.view : initial;
  });
  useEffect(() => {
    if (!history.state || history.state.scope !== scope) {
      history.replaceState({ scope, view }, '');
    }
    const onPop = (event) => {
      if (event.state && event.state.scope === scope) setView(event.state.view);
    };
    addEventListener('popstate', onPop);
    return () => removeEventListener('popstate', onPop);
  }, [scope]);
  // push: a new history entry. replace: no way back to the current view.
  const go = useCallback((next, { replace = false } = {}) => {
    if (replace) history.replaceState({ scope, view: next }, '');
    else history.pushState({ scope, view: next }, '');
    setView(next);
  }, [scope]);
  return [view, go];
}

// A back selection changes no value, so it is a normal button (ADR 0037
// item 1).
function BackButton() {
  return html`<button class="button" type="button" onClick=${() => history.back()}>← Back</button>`;
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
    <${Field} id="passphrase" label="Passphrase" type="password" autocomplete="new-password" required
      value=${passphrase} onInput=${(e) => setPassphrase(e.target.value)} error=${passError}>
      <${Meter} strength=${strength} />
    <//>
    <${Field} id="confirm" label="Type the passphrase again" type="password" autocomplete="new-password" required
      value=${confirm} onInput=${(e) => setConfirm(e.target.value)} error=${confirmError} />
    <label class="check"><input type="checkbox" checked=${recovery} onChange=${(e) => setRecovery(e.target.checked)} />
      Make a recovery key, in case I forget the passphrase</label>
    <${ErrorNotice} error=${vaultError.other} />
    <div class="actions">
      <button class="button primary" type="submit" disabled=${busy || !ready}>${busy ? 'Making the vault…' : 'Make the vault'}</button>
    </div>
  </form>`;
}

function ShowRecoveryKey({ recoveryKey, onDone }) {
  const [kept, setKept] = useState(false);
  return html`<div class="form">
    <div><h1>Write down your recovery key</h1>
      <p>Studio shows this key one time only. With it, you can open the vault if you forget your passphrase.</p></div>
    <div class="recovery" aria-label="Recovery key">${recoveryKey}</div>
    <label class="check"><input type="checkbox" checked=${kept} onChange=${(e) => setKept(e.target.checked)} />
      I wrote down the recovery key in a safe place</label>
    <div class="actions"><button class="button" type="button" disabled=${!kept} onClick=${onDone}>Continue</button></div>
  </div>`;
}

function LocalHostForm({ onAdded }) {
  const [name, setName] = useState('');
  const [root, setRoot] = useState('');
  const [baseUrl, setBaseUrl] = useState('');
  const { busy, error, run } = useAction();
  const submit = (event) => {
    event.preventDefault();
    run(async () => {
      const connector = await api('/connectors', {
        method: 'POST',
        // An empty name takes the last part of the folder path.
        body: { kind: 'local', name: name.trim() || root.replace(/\/+$/, '').split('/').pop() || root, root, base_url: baseUrl || null },
      });
      onAdded(connector);
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
    <div class="actions"><button class="button primary" type="submit" disabled=${busy || !root}>${busy ? 'Adding…' : 'Add the folder'}</button></div>
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
    onAdded(connector);
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
    <${Field} id="s3-key" label="Access key ID" required autocomplete="off"
      value=${values.access_key_id} onInput=${set('access_key_id')} />
    <${Field} id="s3-secret" label="Secret access key" type="password" required autocomplete="off"
      hint="Studio keeps this key in your vault."
      value=${values.secret_access_key} onInput=${set('secret_access_key')} />
    ${notice}
    <div class="actions"><button class="button primary" type="submit" disabled=${busy || !ready}>${busy ? 'Testing the bucket…' : 'Add the bucket'}</button></div>
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
    onAdded(connector);
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
        <button class="button primary" type="submit" disabled=${busy}>${busy ? 'Adding…' : 'Accept and add the server'}</button>
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
    ${values.login === 'key' && html`<div class="field">
      <label for="sftp-key">Private key</label>
      <div class="field-body">
        <span class="hint" id="sftp-key-hint">Paste the text of the key file, such as id_ed25519. Studio keeps it in your vault.</span>
        <textarea id="sftp-key" rows="6" required autocomplete="off" spellcheck="false" aria-describedby="sftp-key-hint"
          value=${values.private_key} onInput=${set('private_key')}></textarea>
      </div>
    </div>`}
    ${values.login === 'key' && html`<${Field} id="sftp-passphrase" label="Passphrase of the key (optional)" type="password"
      autocomplete="off" value=${values.passphrase} onInput=${set('passphrase')} />`}
    ${values.login === 'password' && html`<${Field} id="sftp-password" label="Password" type="password" required
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
        ${nk.signer.busy ? 'Asking your signer…' : 'Use my signer'}</button>
    </div>`}
    ${nk.mode === 'signer' && nk.connecting && html`<${SignerConnect} onConnected=${nk.connected} onCancel=${nk.cancelConnect} />`}
    ${nk.mode === 'signer' && nk.signer.error && (nk.signer.error.code === 'no_signer'
      ? noticeText(nk.signer.error.message)
      : html`<${ErrorNotice} error=${nk.signer.error} />`)}
    ${nk.mode === 'vault' && html`<${Field} id=${`${idPrefix}-secret`} label="Private key of the site" type="password"
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
    onAdded(connector);
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
    <div class="actions"><button class="button primary" type="submit" disabled=${add.busy || !ready}>${add.busy ? 'Adding…' : 'Add the site'}</button></div>
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
    <p class="hint">A Blossom host keeps media only. The feed XML needs a different host.</p>
    <${SigningStep} nk=${nk} idPrefix="blossom" keyError=${errors.fields.secret_key} />
    <${Field} id="blossom-name" label="Name (optional)" hint="Studio uses the host name of the first server when you leave this empty."
      value=${values.name} onInput=${set('name')} />
    <${AreaField} id="blossom-servers" label="Blossom servers" error=${errors.fields.servers}
      value=${values.servers} onInput=${set('servers')} />
    ${errors.notice}
    <div class="actions"><button class="button primary" type="submit" disabled=${add.busy || !ready}>${add.busy ? 'Adding…' : 'Add the Blossom host'}</button></div>
  </form>`);
}

// One setting with a small number of values, as a select menu in one row.
// `options` is a list of [value, label] pairs.
// A `placeholder` adds a first entry with an empty value, for a setting that
// has no value yet.
function ChoiceSelect({ id, label, value, options, placeholder, onChange }) {
  return html`<div class="field">
    <label for=${id}>${label}</label>
    <div class="field-body">
      <select id=${id} value=${value} onChange=${(e) => onChange(e.target.value)}>
        ${placeholder && html`<option value="">${placeholder}</option>`}
        ${options.map(([option, text]) => html`<option value=${option}>${text}</option>`)}
      </select>
    </div>
  </div>`;
}

// The first choice is the type of the host. The form below it follows the
// type (ADR 0045 item 4). A server has no folder host (ADR 0055).
function AddHostForm({ feeds, onAdded }) {
  const [type, setType] = useState(serverMode ? 's3' : 'local');
  return html`<div class="form">
    <${ChoiceSelect} id="host-type" label="Type of host" value=${type} onChange=${setType} options=${[
      ...(serverMode ? [] : [['local', 'Folder on this computer']]),
      ['s3', 'S3 or R2 bucket'],
      ['sftp', 'SFTP server'],
      ['nsite', 'nsite site'],
      ['blossom', 'Blossom server'],
    ]} />
    ${type === 's3'
      ? html`<${S3HostForm} onAdded=${onAdded} />`
      : type === 'sftp'
        ? html`<${SftpHostForm} onAdded=${onAdded} />`
        : type === 'nsite'
          ? html`<${NsiteHostForm} feeds=${feeds} onAdded=${onAdded} />`
          : type === 'blossom'
            ? html`<${BlossomHostForm} onAdded=${onAdded} />`
            : html`<${LocalHostForm} onAdded=${onAdded} />`}
  </div>`;
}

// Make a new album feed in a connected folder, as a draft with no file on
// the host yet (ADR 0032 items 6 to 9).
function NewReleaseForm({ connectors, onAdded, show = false }) {
  const [connectorId, setConnectorId] = useState(connectors[0] ? connectors[0].id : null);
  const [path, setPath] = useState('');
  const [title, setTitle] = useState('');
  const [author, setAuthor] = useState('');
  const { busy, error, run } = useAction();
  const submit = (event) => {
    event.preventDefault();
    run(async () => {
      const feed = await api('/feeds/new', {
        method: 'POST',
        body: { connector_id: connectorId, path, title, author, medium: 'music' },
      });
      onAdded(feed);
    });
  };
  const prefix = show ? 'show' : 'release';
  const releaseError = splitError(error, ['no_public_address', 'file_exists']);
  const ready = connectorId !== null && path.trim() && title.trim() && author.trim();
  return html`<form class="form" onSubmit=${submit}>
    ${connectors.length > 1 && html`<div class="field"><label for=${`${prefix}-host`}>Host</label>
      <select id=${`${prefix}-host`} class="button" value=${connectorId}
        onChange=${(e) => setConnectorId(Number(e.target.value))}>
        ${connectors.map((c) => html`<option value=${c.id}>${c.name}</option>`)}
      </select></div>`}
    <${Field} id=${`${prefix}-path`} label="Feed file" required placeholder=${show ? 'my-shows.xml' : 'my-album.xml'}
      hint="The path of the new feed file in the folder of the host."
      error=${releaseError.field} value=${path} onInput=${(e) => setPath(e.target.value)} />
    <${Field} id=${`${prefix}-title`} label=${show ? 'Show feed title' : 'Album title'} required
      value=${title} onInput=${(e) => setTitle(e.target.value)} />
    <${Field} id=${`${prefix}-author`} label="Artist" required
      value=${author} onInput=${(e) => setAuthor(e.target.value)} />
    <${ErrorNotice} error=${releaseError.other} />
    <div class="actions"><button class="button primary" type="submit" disabled=${busy || !ready}>
      ${busy ? (show ? 'Making the show feed…' : 'Making the release…') : show ? 'Make the show feed' : 'Make the release'}</button></div>
  </form>`;
}

// Make a publisher feed in a connected folder, as a draft with no file on
// the host yet (ADR 0041 item 2). The name goes to `title` and to `author`.
function NewPublisherForm({ connectors, onAdded }) {
  const [connectorId, setConnectorId] = useState(connectors[0] ? connectors[0].id : null);
  const [path, setPath] = useState('');
  const [name, setName] = useState('');
  const [role, setRole] = useState(null);
  const { busy, error, run } = useAction();
  const submit = (event) => {
    event.preventDefault();
    run(async () => {
      const feed = await api('/feeds/new', {
        method: 'POST',
        body: { connector_id: connectorId, path, title: name, author: name, medium: 'publisher', role },
      });
      onAdded(feed);
    });
  };
  const formError = splitError(error, ['no_public_address', 'file_exists']);
  const ready = connectorId !== null && path.trim() && name.trim() && role !== null;
  return html`<form class="form" onSubmit=${submit}>
    ${connectors.length > 1 && html`<div class="field"><label for="publisher-host">Host</label>
      <select id="publisher-host" class="button" value=${connectorId}
        onChange=${(e) => setConnectorId(Number(e.target.value))}>
        ${connectors.map((c) => html`<option value=${c.id}>${c.name}</option>`)}
      </select></div>`}
    <${Field} id="publisher-path" label="Feed file" required placeholder="my-label.xml"
      hint="The path of the new feed file in the folder of the host."
      error=${formError.field} value=${path} onInput=${(e) => setPath(e.target.value)} />
    <${Field} id="publisher-name" label="Name" required
      value=${name} onInput=${(e) => setName(e.target.value)} />
    <${ChoiceSelect} id="publisher-role" label="Role" value=${role ?? ''} placeholder="Select a role"
      options=${[['artist', 'Artist'], ['label', 'Label']]} onChange=${(value) => setRole(value || null)} />
    <${ErrorNotice} error=${formError.other} />
    <div class="actions"><button class="button action" type="submit" disabled=${busy || !ready}>
      ${busy ? 'Making the publisher feed…' : 'Make the Publisher Feed'}</button></div>
  </form>`;
}

// Make a playlist in a connected folder, as a draft with no file on the host
// yet (ADR 0053 item 2). The author goes to `author`, and `medium` is musicL.
function NewPlaylistForm({ connectors, onAdded }) {
  const [connectorId, setConnectorId] = useState(connectors[0] ? connectors[0].id : null);
  const [path, setPath] = useState('');
  const [title, setTitle] = useState('');
  const [author, setAuthor] = useState('');
  const { busy, error, run } = useAction();
  const submit = (event) => {
    event.preventDefault();
    run(async () => {
      const feed = await api('/feeds/new', {
        method: 'POST',
        body: { connector_id: connectorId, path, title, author, medium: 'musicL' },
      });
      onAdded(feed);
    });
  };
  const formError = splitError(error, ['no_public_address', 'file_exists']);
  const ready = connectorId !== null && path.trim() && title.trim() && author.trim();
  return html`<form class="form" onSubmit=${submit}>
    ${connectors.length > 1 && html`<div class="field"><label for="playlist-host">Host</label>
      <select id="playlist-host" class="button" value=${connectorId}
        onChange=${(e) => setConnectorId(Number(e.target.value))}>
        ${connectors.map((c) => html`<option value=${c.id}>${c.name}</option>`)}
      </select></div>`}
    <${Field} id="playlist-path" label="Feed file" required placeholder="my-playlist.xml"
      hint="The path of the new feed file in the folder of the host."
      error=${formError.field} value=${path} onInput=${(e) => setPath(e.target.value)} />
    <${Field} id="playlist-title" label="Playlist name" required
      value=${title} onInput=${(e) => setTitle(e.target.value)} />
    <${Field} id="playlist-author" label="Author" required
      value=${author} onInput=${(e) => setAuthor(e.target.value)} />
    <${ErrorNotice} error=${formError.other} />
    <div class="actions"><button class="button primary" type="submit" disabled=${busy || !ready}>
      ${busy ? 'Making the playlist…' : 'Make the playlist'}</button></div>
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
    return html`<div class="notice">Add a host first. Studio writes each feed through its host.</div>`;
  }
  const feedError = splitError(error, ['not_found', 'invalid_path', 'invalid_feed']);
  const available = found ? found.feeds.filter((f) => !f.in_collection) : [];
  return html`<form class="form" onSubmit=${submit}>
    ${connectors.length > 1 && html`<div class="field"><label for="feed-host">Host</label>
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
            ${f.title || f.path} · ${mediumKind(f.medium).label}${f.blocked ? ' · blocked' : ''} · ${f.path}${f.in_collection ? ' · in your collection' : ''}</option>`)}
        </select>`}
        ${found && found.truncated && html`<p class="hint">The folder has more files than Studio lists. Type the path of a feed that is not in the list.</p>`}
        ${feedError.field && html`<span class="field-error" id="feed-choice-error">${feedError.field}</span>`}
      </div>
    </div>`}
    ${typing
      ? html`<${Field} id="feed-path" label="Feed file" required placeholder="album/feed.xml"
          hint="The path of the feed file in the folder of the host."
          error=${feedError.field} value=${path} onInput=${(e) => setPath(e.target.value)} />`
      : html`<button class="button link" type="button" onClick=${() => setTyping(true)}>The feed is not in the list</button>`}
    ${typing && available.length > 0 && html`<button class="button link" type="button" onClick=${() => setTyping(false)}>Select from the list</button>`}
    <${ErrorNotice} error=${feedError.other} />
    <div class="actions"><button class="button primary" type="submit" disabled=${busy || !target}>${busy ? 'Reading the feed…' : 'Add the feed'}</button></div>
  </form>`;
}

// ---- Find feeds by address or name (ADR 0020) ----

// Pasted addresses and musicindex search results, with one action at the
// end that uses both (ADR 0020).
function FindFeeds({ onFound }) {
  const [addresses, setAddresses] = useState('');
  const [query, setQuery] = useState('');
  const [hits, setHits] = useState(null);
  const [guids, setGuids] = useState([]);
  const search = useAction();
  const find = useAction();
  const lines = addresses.split('\n').map((l) => l.trim()).filter(Boolean);
  const count = lines.length + guids.length;

  const runSearch = (event) => {
    event.preventDefault();
    if (!query.trim()) return;
    search.run(async () => setHits(await api(`/search?q=${encodeURIComponent(query.trim())}`)));
  };
  const toggle = (guid) => setGuids(guids.includes(guid) ? guids.filter((g) => g !== guid) : [...guids, guid]);
  const runFind = () => find.run(async () => onFound(await api('/discover', { method: 'POST', body: { addresses: lines, guids } })));
  return html`<div class="form">
    <div class="field">
      <label for="addresses">Feed addresses</label>
      <span class="hint" id="addresses-hint">One on each line. A publisher feed brings all of its releases.</span>
      <textarea id="addresses" rows="3" aria-describedby="addresses-hint" placeholder="https://music.example/feed.xml"
        value=${addresses} onInput=${(e) => setAddresses(e.target.value)}></textarea>
    </div>
    <form class="form" onSubmit=${runSearch} role="search">
      <div class="field">
        <label for="search">Or search musicindex by name</label>
        <div class="inline"><input id="search" type="search" value=${query} onInput=${(e) => setQuery(e.target.value)}
          placeholder="Artist or feed name" />
          <button class="button" type="submit" disabled=${search.busy || !query.trim()}>${search.busy ? 'Searching…' : 'Search'}</button></div>
      </div>
      <${ErrorNotice} error=${search.error} />
      ${hits && hits.length === 0 && html`<p class="empty">musicindex found no feed with that name.</p>`}
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
      ${find.busy ? 'Reading the feeds…' : count > 0 ? `Find my feeds (${count})` : 'Find my feeds'}</button></div>
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
        const kind = mediumKind(f.medium);
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
      <ul class="failed">${found.failed.map((f) => html`<li><span class="mono">${f.source}</span>: ${f.message}</li>`)}</ul></div>`}
    <${ErrorNotice} error=${error} />
    ${addable.length === 0 && found.feeds.length > 0
      ? html`<p role="status">Each of these feeds is in your collection.</p>
        ${onNothingToAdd && html`<div class="actions"><button class="button primary" type="button" onClick=${onNothingToAdd}>Open Studio</button></div>`}`
      : html`<div class="actions"><button class="button primary" type="button" disabled=${busy || chosen.length === 0} onClick=${add}>
          ${busy ? 'Adding…' : `Add ${chosen.length} feed${chosen.length === 1 ? '' : 's'}`}</button></div>`}
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
  const read = async (event) => {
    setFileError(null);
    const parsed = [];
    const files = Array.from(event.target.files);
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
        <ul class="failed">${result.failed.map((f) => html`<li><span class="mono">${f.source}</span>: ${f.message}</li>`)}</ul></div>`}
      <div class="actions"><button class="button" type="button" onClick=${() => onAdded(result)}>Continue</button></div>
    </div>`;
  }
  return html`<div class="form">
    <div class="field import-pick">
      <span class="hint" id="msp-files-hint">The .json files that MSP 2.0 saved for your hosted feeds. You can select more than one.</span>
      <input id="msp-files" class="visually-hidden" type="file" accept=".json,application/json" multiple aria-describedby="msp-files-hint"
        aria-invalid=${fileError ? 'true' : undefined} onChange=${read} />
      <label for="msp-files" class="button action">Choose Backup Files</label>
      ${names.length > 0 && html`<p class="hint mono">${names.join(', ')}</p>`}
      ${fileError && html`<span class="field-error">${fileError}</span>`}
    </div>
    <${ErrorNotice} error=${error} />
    <div class="actions"><button class="button primary" type="button" disabled=${busy || backups.length === 0} onClick=${send}>
      ${busy ? 'Importing…' : `Import ${backups.length || ''} backup${backups.length === 1 ? '' : 's'}`}</button></div>
  </div>`;
}

// ---- First-run wizard: one question (ADR 0020) ----

function Wizard({ onFinish }) {
  const [step, go] = useNav('wizard', 'question');
  const [found, setFound] = useState(null);
  const [added, setAdded] = useState(null);
  let body;
  if (step === 'question') {
    body = html`<div class="form">
      <div><h1>Do you already have a feed?</h1>
        <p>Studio keeps your feeds in one collection, and edits them where you host them.</p></div>
      <div class="actions">
        <button class="button primary" type="button" onClick=${() => go('find')}>Yes, I have feeds</button>
        <button class="button" type="button" onClick=${() => go('new')}>No, I am new</button>
      </div>
      <button class="button link" type="button" onClick=${() => go('msp')}>Import from MSP 2.0</button>
    </div>`;
  } else if (step === 'msp') {
    body = html`<div class="form"><div><h1>Import from MSP 2.0</h1></div>
      <${MspImport} onAdded=${onFinish} />
      <div class="actions"><${BackButton} /></div></div>`;
  } else if (step === 'find') {
    body = html`<div class="form"><div><h1>Find your feeds</h1></div>
      <${FindFeeds} onFound=${(result) => { setFound(result); go('results'); }} />
      <div class="actions"><${BackButton} /></div></div>`;
  } else if (step === 'results' && found) {
    body = html`<div class="form"><div><h1>Your feeds</h1></div>
      <${FoundFeeds} found=${found} onNothingToAdd=${onFinish}
        onAdded=${(result, groups) => { setAdded({ result, groups }); go('done', { replace: true }); }} />
      <div class="actions"><${BackButton} /></div></div>`;
  } else if (step === 'done' && added) {
    const own = added.groups.filter((g) => !g.service).map((g) => g.host);
    body = html`<div class="form"><div><h1>Studio is ready</h1>
      <p>Studio added ${added.result.added.length} feed${added.result.added.length === 1 ? '' : 's'} to your collection.</p></div>
      ${own.length > 0 && html`<p>To edit the feeds on ${own.join(', ')}, connect the folder that holds them in Hosts.</p>`}
      <div class="actions"><button class="button primary" type="button" onClick=${onFinish}>Open Studio</button></div></div>`;
  } else if (step === 'new') {
    body = html`<div class="form"><div><h1>Welcome</h1>
      <p>To make a new release, connect the folder that your web server publishes in Hosts. Then use “Make a new release” in Add a feed.</p></div>
      <div class="actions"><${BackButton} /><button class="button primary" type="button" onClick=${onFinish}>Open Studio</button></div></div>`;
  } else {
    // A reload lost the wizard memory. Start again at the question.
    body = html`<div class="form"><div><h1>Do you already have a feed?</h1></div>
      <div class="actions"><button class="button primary" type="button" onClick=${() => go('find')}>Yes, I have feeds</button>
        <button class="button" type="button" onClick=${() => go('new')}>No, I am new</button></div></div>`;
  }
  return html`<main class="center"><div class="card"><${Brand} />${body}</div></main>`;
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
      ${busy ? 'Using this folder…' : 'Use this folder'}</button></div>
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
    <div><h1>Unlock Studio</h1><p>Your passphrase opens the vault with the keys of your hosts.</p></div>
    <${Field} id="unlock" label="Passphrase" type="password" autocomplete="current-password" required autofocus
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
function useMedia(query) {
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

function Bar({ back = true, children }) {
  const nav = useContext(NavContext);
  return html`<div class="bar">
    ${back && nav && (nav.coarse
      ? html`<button class="button icon-button back" type="button" aria-label="Back"
          disabled=${!nav.canBack} onClick=${nav.back}><${Chevron} direction="back" /></button>`
      : html`<button class="button quiet icon-button" type="button" aria-label="Back" title="Back"
          disabled=${!nav.canBack} onClick=${nav.back}><${Chevron} direction="back" /></button>
        <button class="button quiet icon-button" type="button" aria-label="Forward" title="Forward"
          disabled=${!nav.canForward} onClick=${nav.forward}><${Chevron} direction="forward" /></button>`)}
    ${children}
  </div>`;
}

// A feed that the editor cannot open yet: the full hero and the facts, as
// before ADR 0029.
function ReadOnlyFeed({ feed, onChanged, onRemoved }) {
  const { busy, error, run } = useAction();
  const kind = mediumKind(feed.medium);
  const refresh = () => run(async () => onChanged(await api(`/feeds/${feed.id}/refresh`, { method: 'POST' })));
  const remove = () => {
    const name = feed.title || feed.path || 'this feed';
    if (!confirm(`Remove “${name}” from your collection? The feed file on the host does not change.`)) return;
    run(async () => {
      await api(`/feeds/${feed.id}`, { method: 'DELETE' });
      onRemoved();
    });
  };
  const glow = `var(--c-${kind.dot === 'playlist' ? 'playlist' : kind.dot === 'publisher' ? 'publisher' : 'feed'})`;
  return html`<div>
    <${Bar}><span class="bar-title">${feed.title || 'Untitled feed'}</span><//>
    <section class="hero" style=${{ '--glow': glow }}>
      <div class="art" aria-hidden="true"></div>
      <div>
        <span class="eyebrow"><span class=${`dot ${kind.dot}`}></span>${kind.label}${feed.connector_id ? '' : ' · read-only'}</span>
        <h1>${feed.title || 'Untitled feed'}</h1>
        <p class="summary">${feed.public_url || 'This feed has no public address yet.'}</p>
        <div class="actions">
          <button class="button primary" type="button" disabled=${busy || !(feed.connector_id || feed.public_url)} onClick=${refresh}>
            ${busy ? 'Reading…' : 'Read again'}</button>
          <button class="button destructive" type="button" disabled=${busy} onClick=${remove}>Remove from collection</button>
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
        <dt>File</dt><dd class="mono">${feed.path || 'Read-only. Connect the folder that holds this file in Hosts.'}</dd>
        <dt>Version</dt><dd class="mono">${feed.last_version || 'Not read yet'}</dd>
      </dl>
      ${feed.connector_id && html`<p class="hint">The editor for this feed type comes in a later phase.</p>`}
    </section>
  </div>`;
}

// The names of the pages that are not the album page or a track.
const PAGE_NAMES = { credits: 'Credits and links', recipients: 'Value routes', files: 'Files', tracks: 'Tracks' };

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
  const kind = mediumKind(feed.medium);
  const refresh = () => run(async () => onChanged(await api(`/feeds/${feed.id}/refresh`, { method: 'POST' })));
  const glow = `var(--c-${kind.dot === 'playlist' ? 'playlist' : kind.dot === 'publisher' ? 'publisher' : 'feed'})`;
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
  return html`<div>
    <${Bar}>
      <${Crumb} feed=${feed} view=${view} draft=${draft} />
      <${DraftToolbar} key=${feed.id} feedId=${feed.id} feedConnectorId=${feed.connector_id} draft=${draft} onWritten=${onChanged} connectors=${connectors}
        onOpsChanged=${() => { opsChanged(); reloadFiles(); }} onOpsCleared=${reloadFiles} />
    <//>
    <div class="feed-head">
      <header class="head">
        <${ImageThumb} url=${draft.model && coverOf(draft.model)} glow=${glow} className="cover" always />
        <div class="text">
          <span class="eyebrow">${kind.label}</span>
          <h1>${feed.title || 'Untitled feed'}</h1>
          <p class="sub mono">${feed.public_url || 'This feed has no public address yet.'}</p>
        </div>
        ${feed.last_version !== null && html`<button class="button" type="button" disabled=${busy} onClick=${refresh}>
          ${busy ? 'Reading…' : 'Read again'}</button>`}
      </header>
      <${ErrorNotice} error=${error} />
      ${feed.last_version === null && html`<div class="notice" role="status">
        <strong>The host has no file for this ${feed.medium === 'publisher' ? 'publisher feed' : 'release'} yet.</strong>
        Open Review and write the draft to make the file.
      </div>`}
      <${AddressDifferences} feed=${feed} />
    </div>
    <${AlbumEditor} key=${feed.id} feed=${feed} feeds=${feeds} draft=${draft} view=${view} go=${go}
      adding=${adding} onAdding=${onAdding} importing=${importing} onImporting=${onImporting} onRemoved=${onRemoved}
      connectors=${connectors} onOpsChanged=${opsChanged} filesReloadKey=${filesReloadKey}
      differences=${differences} onRescan=${onRescan} onListChanged=${onListChanged} />
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
        const kind = mediumKind(f.medium);
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
      ${busy ? 'Adding…' : `Add ${chosen.length} feed${chosen.length === 1 ? '' : 's'}`}</button></div>
  </div>`;
}

// The library folder and the download limits (ADR 0036 items 1 and 17).
// A move of the library to a new folder is not part of this version, so
// this panel shows the folder as a fact and lets the user change only the
// download limits.
function LibraryPanel() {
  const [library, setLibrary] = useState(null);
  const [maxMb, setMaxMb] = useState('');
  const [minMb, setMinMb] = useState('');
  const { busy, error, run } = useAction();
  const load = () => api('/library').then((result) => {
    setLibrary(result);
    setMaxMb(String(Math.round(result.download_max_file_bytes / (1024 * 1024))));
    setMinMb(String(Math.round(result.download_min_free_bytes / (1024 * 1024))));
  });
  useEffect(() => { load(); }, []);
  const submit = (event) => {
    event.preventDefault();
    run(async () => {
      await api('/library/limits', {
        method: 'PUT',
        body: {
          download_max_file_bytes: Math.round(Number(maxMb) * 1024 * 1024),
          download_min_free_bytes: Math.round(Number(minMb) * 1024 * 1024),
        },
      });
      await load();
    });
  };
  if (!library) return null;
  const ready = Number(maxMb) > 0 && Number(minMb) > 0;
  return html`<article class="panel">
    <h2>Library</h2>
    ${!serverMode && html`<dl class="facts"><dt>Folder</dt><dd class="mono">${library.root}</dd></dl>`}
    <form class="form" onSubmit=${submit}>
      <div class="grid2">
        <${Field} id="limit-max" label="Largest download, in MB" inputmode="numeric" required
          value=${maxMb} onInput=${(e) => setMaxMb(e.target.value)} />
        <${Field} id="limit-min" label="Free space to keep, in MB" inputmode="numeric" required
          value=${minMb} onInput=${(e) => setMinMb(e.target.value)} />
      </div>
      <${ErrorNotice} error=${error} />
      <div class="actions"><button class="button primary" type="submit" disabled=${busy || !ready}>
        ${busy ? 'Saving…' : 'Save the limits'}</button></div>
    </form>
  </article>`;
}

// The two announcement settings (ADR 0052 items 5 and 6). The podping check box
// is off and disabled while the first check box is off.
function AnnouncePanel() {
  const [saved, setSaved] = useState(null);
  const [pubnotify, setPubnotify] = useState(false);
  const [podping, setPodping] = useState(false);
  const { busy, error, run } = useAction();
  const load = () => api('/settings/announcements').then((result) => {
    setSaved(result);
    setPubnotify(result.pubnotify);
    setPodping(result.podping);
  });
  useEffect(() => { load(); }, []);
  const submit = (event) => {
    event.preventDefault();
    run(async () => {
      await api('/settings/announcements', { method: 'PUT', body: { pubnotify, podping } });
      await load();
    });
  };
  if (!saved) return null;
  const same = pubnotify === saved.pubnotify && podping === saved.podping;
  return html`<article class="panel">
    <h2>Announcements</h2>
    <form class="form" onSubmit=${submit}>
      <label class="check"><input type="checkbox" checked=${pubnotify}
        onChange=${(e) => setPubnotify(e.target.checked)} />
        <span>Tell Podcast Index after each write</span></label>
      <label class="check"><input type="checkbox" checked=${pubnotify && podping} disabled=${!pubnotify}
        onChange=${(e) => setPodping(e.target.checked)} />
        <span>Send a podping with it</span></label>
      <p class="hint">Studio sends the announcement 30 seconds after a write.</p>
      <${ErrorNotice} error=${error} />
      <div class="actions"><button class="button primary" type="submit" disabled=${busy || same}>
        ${busy ? 'Saving…' : 'Save'}</button></div>
    </form>
  </article>`;
}

// The connected apps (ADR 0054 items 7 to 10). "Make the token" shows the
// token one time. The token stays in the state of this panel only.
function ConnectedAppsPanel() {
  const [apps, setApps] = useState(null);
  const [adding, setAdding] = useState(false);
  const [name, setName] = useState('');
  const [made, setMade] = useState(null);
  const [copied, setCopied] = useState(false);
  const { busy, error, run } = useAction();
  const load = () => api('/apps').then(setApps);
  useEffect(() => { load(); }, []);
  const make = (event) => {
    event.preventDefault();
    run(async () => {
      const result = await api('/apps', { method: 'POST', body: { name: name.trim() } });
      setMade(result);
      setCopied(false);
      setAdding(false);
      setName('');
      await load();
    });
  };
  const remove = (app) => {
    if (!confirm(`Remove the app “${app.name}”? Its token stops working.`)) return;
    run(async () => {
      await api(`/apps/${app.id}`, { method: 'DELETE' });
      if (made && made.id === app.id) setMade(null);
      await load();
    });
  };
  // An app from before the app key has no key. "Connect again" removes it
  // and makes a new app with the same name and a new token (ADR 0058 item 15).
  const connectAgain = (app) => {
    if (!confirm(`Connect the app “${app.name}” again? Its old token stops working.`)) return;
    run(async () => {
      await api(`/apps/${app.id}`, { method: 'DELETE' });
      const result = await api('/apps', { method: 'POST', body: { name: app.name } });
      setMade(result);
      setCopied(false);
      await load();
    });
  };
  const copy = () => {
    if (navigator.clipboard) {
      navigator.clipboard.writeText(made.token).then(() => setCopied(true), () => {});
    }
  };
  if (!apps) return null;
  return html`<article class="panel">
    <h2>Connected apps</h2>
    ${apps.length === 0 && !adding && html`<p class="hint">No app is connected.</p>`}
    ${apps.length > 0 && html`<ul class="app-list">
      ${apps.map((a) => html`<li key=${a.id}>
        <span class="app-name">${a.name}
          ${!a.has_key && html`<span class="hint">This app has no key, so it cannot use a grant.</span>`}</span>
        <span class="app-buttons">
          ${!a.has_key && html`<button class="button action" type="button" disabled=${busy}
            aria-label=${`Connect the app ${a.name} again`} onClick=${() => connectAgain(a)}>Connect again</button>`}
          <button class="button destructive" type="button" disabled=${busy}
            aria-label=${`Remove the app ${a.name}`} onClick=${() => remove(a)}>Remove</button>
        </span>
      </li>`)}
    </ul>`}
    ${made && html`<div class="notice" role="status">
      <strong>The token of “${made.name}”</strong>
      <input class="mono token-field" type="text" readonly aria-label=${`Token of ${made.name}`}
        value=${made.token} onFocus=${(e) => e.target.select()} />
      <p class="hint">Give this token to the app now. Studio does not show it again.</p>
      <div class="actions">
        <button class="button" type="button" onClick=${copy}>${copied ? 'Copied' : 'Copy'}</button>
        <button class="button" type="button" onClick=${() => setMade(null)}>Done</button>
      </div>
    </div>`}
    ${adding
      ? html`<form class="form" onSubmit=${make}>
          <${Field} id="app-name" label="Name of the app" required autocomplete="off"
            value=${name} onInput=${(e) => setName(e.target.value)} />
          <div class="actions">
            <button class="button primary" type="submit" disabled=${busy || name.trim() === ''}>
              ${busy ? 'Making…' : 'Make the token'}</button>
            <button class="button" type="button" disabled=${busy} onClick=${() => setAdding(false)}>Cancel</button>
          </div>
        </form>`
      : html`<div class="actions"><button class="button action" type="button"
          onClick=${() => { setMade(null); setAdding(true); }}>Connect an app</button></div>`}
    <${ErrorNotice} error=${error} />
  </article>`;
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
function HostCard({ connector, feeds, removeBusy, onRemove, onOpen, onChanged }) {
  const config = JSON.parse(connector.config);
  const written = feeds.filter((f) => f.connector_id === connector.id);
  const disagreeing = feeds.filter((f) => (connector.disagreeing_feeds || []).includes(f.id));
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(connector.name);
  const [baseUrl, setBaseUrl] = useState(config.base_url || '');
  const isNsite = connector.kind === 'nsite';
  const isBlossom = connector.kind === 'blossom';
  const isNostr = isNsite || isBlossom;
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
    <header class="host-head">
      <span class="eyebrow"><span class="dot hosting"></span>${isS3 ? 'S3 or R2 bucket' : isSftp ? 'SFTP server' : isNsite ? 'nsite site' : isBlossom ? 'Blossom server' : 'Folder on this computer'}</span>
      ${!editing && html`<div class="actions">
        <button class="button" type="button" disabled=${removeBusy} onClick=${startEdit}>Edit</button>
        <button class="button destructive" type="button" disabled=${removeBusy}
          aria-label=${`Remove the host ${connector.name}`} onClick=${onRemove}>Remove</button>
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
                  : html`<dt>Folder</dt><dd class="mono">${config.root}</dd>`}
            ${!isNostr && html`<dt>Public address</dt><dd><span class="mono">${config.base_url || '—'}</span>
              ${config.base_url && config.base_url_inferred && html`<span class="sub"> · Studio found it from the feeds</span>`}</dd>`}
            <dt>Feeds</dt><dd>${written.length === 0 ? 'None yet' : html`<ul class="feed-list">
              ${written.map((f) => html`<li><button class="button link" type="button" onClick=${() => onOpen(f.id)}>
                <span class=${`dot ${mediumKind(f.medium).dot}`}></span>${f.title || f.path}</button></li>`)}</ul>`}</dd>
            ${disagreeing.length > 0 && html`<dt>Different addresses</dt><dd>
              These feeds have an address that does not agree with the folder.
              <ul class="address-list">
                ${disagreeing.map((f) => html`<li><button class="button link" type="button" onClick=${() => onOpen(f.id)}>${f.title || f.path}</button>
                  <span class="mono">${f.public_url}</span></li>`)}
              </ul></dd>`}
          </dl>
          ${isNsite && html`<div class="actions">
            <button class="button" type="button" disabled=${site.busy} onClick=${checkRelays}>
              ${site.busy ? 'Checking…' : 'Check the relays'}</button>
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
              ${check.busy ? 'Checking…' : 'Check the connection'}</button>
          </div>`}
          ${isSftp && newKey && html`<div class="notice error" role="alert">
            <strong>The server gives a new host key.</strong>
            <p>Make sure that this is the fingerprint of the server before you accept it:</p>
            <p class="mono">${newKey}</p>
            <div class="actions"><button class="button action" type="button" disabled=${accept.busy} onClick=${acceptKey}>
              ${accept.busy ? 'Accepting…' : 'Accept the new key'}</button></div>
          </div>`}
          ${mismatch && html`<div class="notice error" role="alert"><strong>The server gives a different key now.</strong>Connect again to see it.</div>`}
          ${!mismatch && html`<${ErrorNotice} error=${accept.error} />`}
          ${isSftp && html`<${ErrorNotice} error=${check.error} />`}`}
  </article>`;
}

function Hosts({ connectors, feeds, onChanged, onOpen }) {
  const { busy, error, run } = useAction();
  const [linked, setLinked] = useState(null);
  const [adding, setAdding] = useState(false);
  // The result of a new host shows above the host cards, and the form is
  // below them, so the page brings the result into view.
  const resultRef = useRef(null);
  useEffect(() => {
    if (linked && resultRef.current) resultRef.current.scrollIntoView({ block: 'start' });
  }, [linked]);
  const remove = (connector) => {
    if (!confirm(`Remove the host “${connector.name}”? The files in it do not change.`)) return;
    run(async () => {
      await api(`/connectors/${connector.id}`, { method: 'DELETE' });
      onChanged();
    });
  };
  // A new folder takes each read-only feed whose file it holds (ADR 0020).
  const added = (connector) => run(async () => {
    const result = await api(`/connectors/${connector.id}/match`, { method: 'POST' });
    setLinked({ connector, feeds: result.linked, folder: result.folder_address });
    setAdding(false);
    onChanged();
  });
  const showForm = adding || connectors.length === 0;
  return html`<div>
    <${Bar}><span class="bar-title">Hosts</span><//>
    <section class="section">
      <${LibraryPanel} />
      <${AnnouncePanel} />
      ${serverMode && html`<${AccountPanel} />`}
      <${ConnectedAppsPanel} />
      ${error && error.code === 'in_use'
        ? html`<div class="notice error" role="alert"><strong>Feeds use this host.</strong>Remove those feeds from your collection first.</div>`
        : html`<${ErrorNotice} error=${error} />`}
      <div ref=${resultRef}></div>
      ${linked && linked.feeds.length > 0 && html`<div class="notice" role="status">
        Studio can now write ${linked.feeds.length} feed${linked.feeds.length > 1 ? 's' : ''} through “${linked.connector.name}”.
      </div>`}
      ${linked && linked.folder && linked.folder.state === 'inferred' && linked.folder.new && html`<div class="notice" role="status">
        <strong>Studio found the public address of “${linked.connector.name}” from its feeds.</strong>
        <span class="mono">${linked.folder.url}</span>
      </div>`}
      ${linked && linked.folder && linked.folder.state === 'different' && html`<div class="notice" role="status">
        <strong>The feeds in “${linked.connector.name}” give different public addresses.</strong>
        Studio did not set a public address for the folder. The host card lists the feeds.
      </div>`}
      ${linked && html`<${FolderFeeds} key=${linked.connector.id} connector=${linked.connector} onAdded=${onChanged} />`}
      ${connectors.map((c) => html`<${HostCard} key=${c.id} connector=${c} feeds=${feeds} removeBusy=${busy}
        onRemove=${() => remove(c)} onOpen=${onOpen} onChanged=${onChanged} />`)}
      ${showForm
        ? html`<div class="panel"><h2>${connectors.length === 0 ? 'Connect your first host' : 'Add a host'}</h2>
            <${AddHostForm} feeds=${feeds} onAdded=${added} />
            ${connectors.length > 0 && html`<button class="button link" type="button" onClick=${() => setAdding(false)}>Cancel</button>`}</div>`
        : html`<div class="actions"><button class="button action" type="button" onClick=${() => setAdding(true)}>＋ Add a host</button></div>`}
    </section>
  </div>`;
}

function AddFeeds({ connectors, onAdded, go }) {
  const [found, setFound] = useState(null);
  return html`<div><${Bar}><span class="bar-title">Add feeds</span><//><section class="section">
    <div class="panel"><h2>Make a new release</h2>
      ${connectors.length > 0
        ? html`<${NewReleaseForm} connectors=${connectors} onAdded=${onAdded} />`
        : html`<p class="hint">Studio puts the files of a release on a host. Connect a host first.</p>
          <div class="actions"><button class="button action" type="button" onClick=${() => go({ name: 'hosts' })}>Connect a Host</button></div>`}
    </div>
    ${connectors.length > 0 && html`<div class="panel"><h2>Make a publisher feed</h2>
      <${NewPublisherForm} connectors=${connectors} onAdded=${onAdded} /></div>`}
    ${connectors.length > 0 && html`<div class="panel"><h2>Make a show feed</h2>
      <p class="hint">A show feed holds live shows. It is a music feed.</p>
      <${NewReleaseForm} connectors=${connectors} onAdded=${onAdded} show />
    </div>`}
    ${connectors.length > 0 && html`<div class="panel"><h2>Make a playlist</h2>
      <${NewPlaylistForm} connectors=${connectors} onAdded=${onAdded} /></div>`}
    <div class="panel"><h2>Find feeds by address or name</h2>
      ${found
        ? html`<${FoundFeeds} found=${found} onAdded=${(result) => onAdded(result.added[0])} />
          <div class="actions"><button class="button link" type="button" onClick=${() => setFound(null)}>Search again</button></div>`
        : html`<${FindFeeds} onFound=${setFound} />`}
    </div>
    ${connectors.length > 0 && html`<div class="panel"><h2>From a connected folder</h2>
      <${AddFeedForm} connectors=${connectors} onAdded=${onAdded} /></div>`}
    <div class="panel"><h2>From MSP 2.0 backup files</h2>
      <${MspImport} onAdded=${(result) => onAdded(result.added[0])} /></div>
  </section></div>`;
}

// The left panel (ADR 0039 items 1, 5 and 6). It shows the destinations, or
// the outline of the open feed, and Hosts and Lock Studio at the bottom.
// `panel` is the value of `leftPanel()`. Only the scroll area scrolls, so
// that the brand and the footer stay in view (visual check of task 022).
// At 760 px or less the left panel is not shown: its content is the root
// page and the outline page of the stack (ADR 0039 item 11, musicindex ADR
// 0030 R-SHELL-2).
function LeftPanel({ panel, draft, changedFeeds, go, showReleases, onAdding, onImporting, onLock }) {
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

// The destination rows: Releases and Add a feed.
function destinationRows(top, go) {
  return top.map((item) => html`<button class="row" type="button" aria-current=${currentAttr(item)}
    onClick=${() => { if (!item.current) go({ name: item.id }); }}>${item.label}</button>`);
}

// The outline of an open feed (ADR 0039 item 6): the cover, the title and the
// type label, then one row for each item. The left panel shows it at a wide
// width, and the outline page shows it at a narrow width. `showReleases` is
// given at a wide width only, where the item "Releases" is in the panel.
function Outline({ panel, draft, changedFeeds, go, showReleases, onAdding, onImporting }) {
  const feed = panel.feed;
  const releases = panel.items.find((item) => item.id === 'releases');
  const rest = panel.items.filter((item) => item.id !== 'releases');
  const hasBadAddress = (number) => draft.bad.some((b) => b.field.startsWith(`items/${number - 1}/`));
  return html`${releases && html`<button class="row" type="button" onClick=${() => showReleases(feed.id)}>${releases.label}</button>`}
    <div class="outline-feed">
      <${ImageThumb} url=${feed.image} className="outline-cover" always />
      <div class="text">
        <span class="title">${feed.title}${changedFeeds.has(feed.id) && html`<span class="change-mark"><span class="visually-hidden"> Changes in the library folder</span></span>`}</span>
        <span class="sub">${feed.typeLabel}</span>
      </div>
    </div>
    <ul class="tree">
      ${rest.map((item) => {
        if (item.heading) {
          return html`<li key=${item.id} class="tree-heading"><span class="label">${item.label}</span></li>`;
        }
        if (item.action) {
          const open = () => {
            go({ name: 'feed', id: feed.id, page: 'album' });
            if (item.action === 'import') onImporting(true);
            else onAdding(item.action === 'add-live' ? 'live' : true);
          };
          return html`<li key=${item.id}><button class="node add" type="button" onClick=${open}>
            <span class="num">＋</span><span class="label">${item.label}</span></button></li>`;
        }
        const open = () => go({
          name: 'feed', id: feed.id, page: item.page,
          ...(item.track === undefined ? {} : { track: item.track }),
        });
        const numbered = item.number !== undefined;
        return html`<li key=${item.id}><button class="node" type="button" aria-current=${currentAttr(item)} onClick=${open}>
          ${numbered && html`<span class="num">${item.number}</span>`}
          <span class="label">${item.label}</span>
          ${item.status && html`<span class="sub">${item.status}</span>`}
          ${numbered && hasBadAddress(item.number) && html`<span class="field-error">Address not correct</span>`}
        </button></li>`;
      })}
    </ul>`;
}

// The footer of the left panel and of the root page: Hosts, and Lock Studio
// when the vault is open. On a server with a locked vault, Unlock Studio
// opens the unlock sheet.
function LeftPanelFooter({ panel, go, onLock }) {
  return html`<footer>
    ${panel.bottom.map((item) => (item.id === 'lock'
      ? html`<button class="row" type="button" onClick=${onLock}>Lock Studio</button>`
      : item.id === 'unlock'
      ? html`<button class="row" type="button" onClick=${() => { askUnlock(); }}>Unlock Studio</button>`
      : html`<button class="row" type="button" aria-current=${currentAttr(item)}
          onClick=${() => { if (!item.current) go({ name: 'hosts' }); }}><span class="dot hosting"></span>${item.label}</button>`))}
  </footer>`;
}

// The root page of a narrow window (ADR 0039 item 11): the destinations, with
// Hosts and Lock Studio at its bottom. It has no back control.
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
        onWritten=${onChanged} connectors=${connectors} onOpsChanged=${opsChanged} onOpsCleared=${() => {}} />`}
    <//>
    <section class="section outline-page">
      <${Outline} panel=${panel} draft=${draft} changedFeeds=${changedFeeds} go=${go} showReleases=${null}
        onAdding=${onAdding} onImporting=${onImporting} />
    </section>
  </div>`;
}

// The sort names of the Releases list (ADR 0039 item 2a).
const SORTS = [{ id: 'newest', label: 'Newest' }, { id: 'az', label: 'A to Z' }];

// The Releases page, the start page (ADR 0039 items 2, 2a and 3, musicindex
// ADR 0027 R-LIST-1). `list` is the sort and the view from the address.
// `marked` is the feed that the last Back selection left marked.
function ReleasesPage({ feeds, connectors, marked, list, changedFeeds, onQuery, onOpen, onAddFeed }) {
  const [sortOpen, setSortOpen] = useState(false);
  const sortButton = useRef(null);
  const releases = releasesList({ feeds, connectors, sort: list.sort, view: list.view, marked });
  const pickSort = (id) => {
    setSortOpen(false);
    onQuery({ sort: id });
    if (sortButton.current) sortButton.current.focus();
  };
  // Escape closes the sort menu and keeps the focus on its button.
  const onSortKey = (event) => {
    if (event.key === 'Escape' && sortOpen) {
      event.stopPropagation();
      setSortOpen(false);
      if (sortButton.current) sortButton.current.focus();
    }
  };
  // A letter scrolls its first row into view.
  const showLetter = (firstId) => {
    const row = document.getElementById(`release-${firstId}`);
    if (row) row.scrollIntoView({ block: 'start' });
  };
  const hasRows = releases.rows.length > 0;
  const viewButton = (name, label, icon) => html`<button class="button quiet icon-button view-button" type="button"
    aria-label=${label} title=${label} aria-pressed=${list.view === name ? 'true' : 'false'}
    onClick=${() => onQuery({ view: name })}>${icon}</button>`;
  return html`<div>
    <${Bar}><span class="bar-title">Releases</span><//>
    <section class="releases">
      <div class="releases-head">
        <h1>Releases</h1>
        ${hasRows && html`<div class="releases-tools">
          <div class="sort" onKeyDown=${onSortKey}>
            <button class="button" type="button" ref=${sortButton} aria-haspopup="menu"
              aria-expanded=${sortOpen ? 'true' : 'false'} onClick=${() => setSortOpen(!sortOpen)}>
              ${list.sort === 'az' ? 'A to Z' : 'Newest'} ⌄</button>
            ${sortOpen && html`<ul class="sort-menu" role="menu" aria-label="Sort">
              ${SORTS.map((s) => html`<li><button type="button" role="menuitemradio" aria-checked=${list.sort === s.id ? 'true' : 'false'}
                onClick=${() => pickSort(s.id)}><span class="sort-mark" aria-hidden="true">${list.sort === s.id ? '✓' : ''}</span>${s.label}</button></li>`)}
            </ul>`}
          </div>
          ${viewButton('rows', 'Rows', html`<svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M4 6h16M4 12h16M4 18h16"></path></svg>`)}
          ${viewButton('tiles', 'Tiles', html`<svg aria-hidden="true" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="4" width="6.5" height="6.5" rx="1.5"></rect><rect x="13.5" y="4" width="6.5" height="6.5" rx="1.5"></rect><rect x="4" y="13.5" width="6.5" height="6.5" rx="1.5"></rect><rect x="13.5" y="13.5" width="6.5" height="6.5" rx="1.5"></rect></svg>`)}
        </div>`}
      </div>
      ${releases.letters && html`<nav class="letters" aria-label="Letters">
        ${releases.letters.map((g) => html`<button class="button letter" type="button" onClick=${() => showLetter(g.firstId)}>${g.letter}</button>`)}
      </nav>`}
      ${hasRows
        ? html`<ul class=${`releases-list ${releases.view}`}>
            ${releases.rows.map((row) => html`<li id=${`release-${row.id}`} data-marked=${row.marked ? 'true' : 'false'}>
              <button class=${releases.view === 'tiles' ? 'releases-tile' : 'releases-row'} type="button" onClick=${() => onOpen(row.id)}>
                <${ImageThumb} url=${row.image} className="releases-cover" always />
                <span class="releases-text">
                  <span class="title">${row.title}${changedFeeds.has(row.id) && html`<span class="change-mark"><span class="visually-hidden"> Changes in the library folder</span></span>`}</span>
                  <span class="sub">${row.typeLabel}${row.hidden ? ' · Hidden from apps' : ''}${releases.view === 'rows' ? ` · ${row.host}` : ''}</span>
                  ${releases.view === 'rows' && html`<span class="sub">${row.draftLabel}</span>`}
                </span>
              </button>
            </li>`)}
          </ul>`
        : html`<div class="empty-releases">
            <p class="empty">No releases yet.</p>
            <div class="actions"><button class="button primary" type="button" onClick=${onAddFeed}>Add a Feed</button></div>
          </div>`}
    </section>
  </div>`;
}

// The navigation state of the session (ADR 0039 item 8). A reload keeps it
// in session storage. If the read fails, or the state is not valid, Studio
// starts again at Releases.
const NAV_KEY = 'studio-nav';

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
  // A new view, or with `replace`, the current view changes.
  const go = (next, { replace = false } = {}) => {
    const base = withScroll();
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
      history.back();
      return;
    }
    const next = backNav(current);
    commit(next);
    history.replaceState({ scope: 'shell', index: next.index, mirrored: mirrored.current }, '');
  };
  const forward = () => {
    if (!canForward(nav)) return;
    const next = forwardNav(withScroll());
    commit(next);
    history.pushState({ scope: 'shell', index: next.index, mirrored: mirrored.current }, '');
  };
  // The link "Releases" of an open feed. It goes back when the Releases
  // entry is directly before it, so that the page keeps its scroll.
  const showReleases = (feedId) => {
    const current = withScroll();
    const before = current.index > 0 ? current.entries[current.index - 1] : null;
    if (before && before.name === 'releases') back();
    else go({ name: 'releases', marked: feedId });
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
    const onPop = (event) => {
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

  // The differences between the library folder and the records (ADR 0038
  // items 13 to 15). One scan runs after the feeds load. The Files page
  // runs it again on request and after each action.
  const [differences, setDifferences] = useState([]);
  const [noticeClosed, setNoticeClosed] = useState(false);
  const rescan = useCallback(async () => {
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
  // A different feed closes the Add a track sheet and the import sheet of
  // the feed before it.
  useEffect(() => { setAdding(false); setImporting(false); }, [draftFeedId]);
  const panel = leftPanel({
    view, feeds: feeds || [], model: draft.model, vaultOpen: vault === 'open',
    vaultLocked: serverMode && vault === 'locked', narrow,
  });

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
      onOpen=${(id) => go({ name: 'feed', id })} />`;
  } else if (view.name === 'add-feed') {
    main = html`<${AddFeeds} connectors=${connectors} go=${go}
      onAdded=${async (f) => { await load(); go(f ? { name: 'feed', id: f.id } : { name: 'releases' }); }} />`;
  } else if (selected && view.name === 'outline') {
    main = html`<${OutlinePage} feed=${selected} panel=${panel} draft=${draft} changedFeeds=${changedFeeds} go=${go}
      onAdding=${setAdding} onImporting=${setImporting} onChanged=${() => load()} connectors=${connectors} />`;
  } else if (selected) {
    main = html`<${FeedDetail} feed=${selected} feeds=${feeds} draft=${draft} view=${view} go=${go}
      adding=${adding} onAdding=${setAdding} importing=${importing} onImporting=${setImporting} onChanged=${() => load()}
      onListChanged=${() => load()}
      onRemoved=${async () => { await load(); go({ name: 'releases' }, { replace: true }); }} connectors=${connectors}
      differences=${differences.filter((d) => d.feed_id === selected.id)} onRescan=${rescan} />`;
  } else {
    main = html`<${ReleasesPage} feeds=${feeds} connectors=${connectors} marked=${view.marked} list=${list}
      changedFeeds=${changedFeeds} onQuery=${changeList}
      onOpen=${(id) => go(openFeedView(id, narrow))} onAddFeed=${() => go({ name: 'add-feed' })} />`;
  }

  const navValue = { coarse, canBack: canBack(nav), canForward: canForward(nav), back, forward };
  const panelProps = { panel, draft, changedFeeds, go, showReleases, onAdding: setAdding, onImporting: setImporting, onLock: lock };
  return html`<${NavContext.Provider} value=${navValue}><div class="shell">
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
  </div><//>`;
}

// ---- Sign-in and sign-up on a server (ADR 0055, ADR 0056) ----

// The text of a failed auth request. Studio shows its own text for each
// code of the auth routes.
function AuthNotice({ error }) {
  if (!error) return null;
  const text = error.code === 'offline' || error.code === 'signer' ? error.message : nip46ErrorText(error.code);
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
function Nip46Page({ title, invite, recoveryKey, onDone, onBack }) {
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
    ${link && html`<div class="actions"><a class="button action" href=${link.uri}>Open in the signer app</a></div>`}
    <form class="form" onSubmit=${connectBunker}>
      <${Field} id="nip46-bunker" label="Bunker address" autocomplete="off" spellcheck="false"
        placeholder="bunker://…" value=${bunker} onInput=${(e) => setBunker(e.target.value)} />
      <div class="actions"><button class="button primary" type="submit" disabled=${paste.busy || phase === 'signing' || !isBunkerAddress(bunker)}>Connect</button></div>
    </form>
    ${phase === 'waiting' && html`<p role="status">Waiting for the signer…</p>`}
    ${phase === 'connecting' && html`<p role="status">Connecting to the signer…</p>`}
    ${phase === 'signing' && html`<p role="status">Signing in. Allow the request in your signer app.</p>`}
    <${AuthNotice} error=${paste.error || qr.error} />
    <div class="actions">
      ${(waiting || phase === 'failed') && html`<button class="button" type="button" onClick=${onBack}>${waiting ? 'Stop' : 'Back'}</button>`}
      ${phase === 'failed' && html`<button class="button action" type="button" onClick=${start}>Try again</button>`}
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
    <${Field} id="sign-in-passphrase" label="Passphrase" type="password" autocomplete="current-password" required
      value=${passphrase} onInput=${(e) => setPassphrase(e.target.value)} />
    <${AuthNotice} error=${error} />
    <div class="actions">
      <button class="button primary" type="submit" disabled=${busy || !email.trim() || !passphrase}>${busy ? 'Signing in…' : 'Sign in'}</button>
      ${window.nostr && html`<button class="button action" type="button" disabled=${busy} onClick=${signer}>Sign in with a signer extension</button>`}
      ${hasPasskeys && html`<button class="button action" type="button" disabled=${busy} onClick=${byPasskey}>Sign in with a passkey</button>`}
      <button class="button action" type="button" disabled=${busy} onClick=${() => setRemote(true)}>Sign in with Amber or a bunker</button>
    </div>
    <button class="button link" type="button" onClick=${onSignUp}>Make an account</button>
    <button class="button link" type="button" onClick=${onRecovery}>Sign in with the recovery key</button>
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
    <${Field} id="recovery-key" label="Recovery key" autocomplete="off" spellcheck="false" required
      value=${key} onInput=${(e) => setKey(e.target.value)} />
    ${error && html`<div class="notice error" role="alert">${error.code === 'offline' ? error.message : recoveryErrorText(error.code)}</div>`}
    <div class="actions">
      <button class="button primary" type="submit" disabled=${busy || !account.trim() || !key.trim()}>${busy ? 'Signing in…' : 'Sign in'}</button>
    </div>
    <button class="button link" type="button" onClick=${onBack}>Back to sign in</button>
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
      <${Field} id="add-method-passphrase" label="New passphrase" type="password" autocomplete="new-password" required autofocus
        value=${passphrase} onInput=${(e) => setPassphrase(e.target.value)} error=${passError}>
        <${Meter} strength=${strength} />
      <//>
      <div class="actions">
        <button class="button primary" type="submit" disabled=${busy || !ready}>${busy ? 'Saving…' : 'Save the passphrase'}</button>
      </div>
    </form>`}
    ${choices.includes('nip07') && html`<div class="actions">
      <button class="button action" type="button" disabled=${busy} onClick=${bySigner}>Use the signer extension</button></div>`}
    ${choices.includes('passkey') && html`<div class="actions">
      <button class="button action" type="button" disabled=${busy} onClick=${byPasskey}>Use a passkey</button></div>`}
    ${choices.length === 0 && html`<p class="hint">This account signs in with a Nostr signer. Studio found no signer extension in this browser.</p>`}
    <${AuthNotice} error=${signerError || vaultError.other} />
    <div class="actions"><button class="button" type="button" disabled=${busy} onClick=${signOut}>Sign out</button></div>
  </div></main>`;
}

function SignUpPage({ signup, onDone, onSignIn }) {
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
      <p>Your passphrase opens the vault with the keys of your hosts.</p></div>
    <${Field} id="sign-up-email" label="Email address" type="email" autocomplete="username" required autofocus
      error=${emailError} value=${email} onInput=${(e) => setEmail(e.target.value)} />
    ${needInvite && html`<${Field} id="sign-up-invite" label="Invite code" autocomplete="off" required
      value=${invite} onInput=${(e) => setInvite(e.target.value)} />`}
    <${Field} id="sign-up-passphrase" label="Passphrase" type="password" autocomplete="new-password" required
      value=${passphrase} onInput=${(e) => setPassphrase(e.target.value)} error=${passError}>
      <${Meter} strength=${strength} />
    <//>
    <label class="check"><input type="checkbox" checked=${recovery} onChange=${(e) => setRecovery(e.target.checked)} />
      Make a recovery key, in case I forget the passphrase</label>
    <${AuthNotice} error=${serverError.other && error.code !== 'invalid_email' ? error : null} />
    <div class="actions">
      <button class="button primary" type="submit" disabled=${busy || !ready}>${busy ? 'Making the account…' : 'Make the account'}</button>
      ${window.nostr && html`<button class="button action" type="button" disabled=${busy || (needInvite && invite.trim() === '')}
        onClick=${signer}>Sign up with a signer extension</button>`}
      <button class="button action" type="button" disabled=${busy || (needInvite && invite.trim() === '')}
        onClick=${() => setRemote(true)}>Sign up with Amber or a bunker</button>
    </div>
    <button class="button link" type="button" onClick=${onSignIn}>I have an account</button>
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
      : html`<${SignUpPage} signup=${session.signup} onDone=${done} onSignIn=${() => setPage('sign_in')} />`;
}

// The unlock sheet (ADR 0056 items 7 and 10). `onDone(true)` follows an
// unlock. `onDone(false)` follows a close of the sheet.
function UnlockSheet({ session, onDone }) {
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
  const signerError = error && error.code === 'wrong_key' && methods.includes('nip07') && !methods.includes('passphrase') ? error : null;
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="unlock-title" onClose=${() => finish(false)}>
    <header>
      <h2 id="unlock-title">Unlock Studio</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body">
      <p>This step needs the vault. Your passphrase, your signer or your passkey opens it.</p>
      ${methods.includes('passphrase') && html`<form class="form" onSubmit=${byPassphrase}>
        <${Field} id="unlock-sheet-passphrase" label="Passphrase" type="password" autocomplete="current-password" required autofocus
          error=${passError.field} value=${passphrase} onInput=${(e) => setPassphrase(e.target.value)} />
        <div class="actions"><button class="button primary" type="submit" disabled=${busy || !passphrase}>${busy ? 'Unlocking…' : 'Unlock'}</button></div>
      </form>`}
      ${methods.includes('nip07') && html`<div class="actions">
        <button class="button action" type="button" disabled=${busy} onClick=${bySigner}>Unlock with a signer extension</button></div>`}
      ${methods.includes('nip46') && html`<div class="actions">
        <button class="button action" type="button" disabled=${busy} onClick=${byRemoteSigner}>Use the remote signer</button></div>
        ${busy && html`<p class="hint" role="status">Allow the request in your signer app.</p>`}`}
      ${methods.includes('passkey') && html`<div class="actions">
        <button class="button action" type="button" disabled=${busy} onClick=${byPasskey}>Use a passkey</button></div>`}
      ${methods.length === 0 && html`<p class="hint">This account opens with a Nostr signer. Studio found no signer extension in this browser.</p>`}
      <${AuthNotice} error=${signerError || passError.other} />
    </div>
    <footer><button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button></footer>
  </dialog>`;
}

// The session of the signed-in user, and the function that reloads it.
const SessionContext = createContext(null);

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
    <div class="actions"><button class="button action" type="submit" disabled=${busy}>${busy ? 'Waiting for the passkey…' : 'Add a passkey'}</button></div>
    <${AuthNotice} error=${error} />
  </form>`;
}

// The Account panel of the Hosts page, on a server only (ADR 0055 item 9).
function AccountPanel() {
  const { session, reload } = useContext(SessionContext);
  const [deleting, setDeleting] = useState(false);
  const [typed, setTyped] = useState('');
  const { busy, error, run } = useAction();
  const confirmation = deleteConfirmation(session, typed);
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
  return html`<article class="panel">
    <h2>Account</h2>
    <dl class="facts"><dt>${byEmail ? 'Email address' : 'Nostr key'}</dt><dd class="mono">${accountName(session)}</dd></dl>
    ${hasPasskeys && html`<${PasskeyList} />`}
    ${deleting
      ? html`<form class="form" onSubmit=${remove}>
          <p class="hint">Studio removes your feeds, your vault and your library files from the server. This cannot be undone.</p>
          <${Field} id="delete-confirm" label=${`Type ${byEmail ? 'your email address' : 'your npub'} to confirm`}
            autocomplete="off" value=${typed} onInput=${(e) => setTyped(e.target.value)} />
          <div class="actions">
            <button class="button destructive" type="submit" disabled=${busy || !confirmation}>${busy ? 'Deleting…' : 'Yes, delete the account'}</button>
            <button class="button" type="button" disabled=${busy} onClick=${() => { setDeleting(false); setTyped(''); }}>Cancel</button>
          </div>
        </form>`
      : html`<div class="actions">
          <button class="button" type="button" disabled=${busy} onClick=${signOut}>Sign out</button>
          <button class="button destructive" type="button" disabled=${busy} onClick=${() => setDeleting(true)}>Delete the account</button>
        </div>`}
    <${AuthNotice} error=${error} />
  </article>`;
}

// ---- App ----

// The studio of a user: the desktop, or a signed-in user of a server.
// `epoch` changes after an unlock, so that the studio reads its state again.
function Studio({ epoch }) {
  const [status, setStatus] = useState(null);
  const [library, setLibrary] = useState(null);
  const [skipped, setSkipped] = useState(false);
  const [error, setError] = useState(null);
  const load = useCallback(async () => {
    try {
      const [nextStatus, nextLibrary] = await Promise.all([api('/status'), api('/library')]);
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
    <div class="actions"><button class="button" type="button" onClick=${load}>Try again</button></div></div></main>`;
  if (!status || !library) return html`<p class="boot" role="status">Loading Studio…</p>`;
  // On a server, a locked vault does not stop the start. A route that needs
  // the vault opens the unlock sheet (ADR 0056 item 9).
  if (status.vault === 'locked' && !serverMode) return html`<${Unlock} onUnlocked=${load} />`;
  // Studio needs its library folder before the editor, so that a write has
  // a release folder to keep the hosted file in (ADR 0036 items 1 and 2).
  if (library.root === null) {
    return html`<${LibrarySheet} defaultRoot=${library.default_root} onDone=${load} />`;
  }
  // The first run: an empty collection (ADR 0020). The vault comes when the
  // user first adds a secret. On a server, the vault exists from the sign-up.
  if (status.feeds === 0 && status.connectors === 0 && !skipped) {
    return html`<${Wizard} onFinish=${() => { setSkipped(true); load(); }} />`;
  }
  return html`<${Shell} vault=${status.vault} onLock=${load} />`;
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
  useEffect(() => {
    if (unlockAsk && !(session && session.signed_in)) { unlockAsk.resolve(false); setUnlockAsk(null); }
  }, [session, unlockAsk]);
  useEffect(() => {
    onUnlockNeeded = () => new Promise((resolve) => setUnlockAsk({ resolve }));
    onSessionLost = () => {
      csrfToken = null;
      setSession((current) => (current && current.mode === 'server'
        ? { mode: 'server', signup: current.signup, signed_in: false } : current));
    };
    return () => { onUnlockNeeded = null; onSessionLost = null; };
  }, []);

  if (bootError) {
    return html`<main class="center"><div class="card"><${Brand} /><${ErrorNotice} error=${bootError} />
      <div class="actions"><button class="button" type="button" onClick=${reload}>Try again</button></div></div></main>`;
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
    if (ok) setEpoch((n) => n + 1);
  };
  return html`${body}${unlockAsk && view === 'shell' && html`<${UnlockSheet} session=${session} onDone=${closeUnlock} />`}`;
}

render(html`<${App} />`, document.getElementById('app'));

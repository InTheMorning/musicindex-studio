// Music Index Studio UI (ADR 0016). Preact and htm, with no build step.
import { html, render, useState, useEffect, useCallback } from './vendor/htm-preact-standalone.module.js';
import { AlbumEditor, DraftToolbar, trackKey } from './editor.js';
import { useDraft } from './draft.js';

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

export async function api(path, { method = 'GET', body } = {}) {
  const headers = { Authorization: `Bearer ${token}` };
  if (body !== undefined) headers['Content-Type'] = 'application/json';
  let response;
  try {
    response = await fetch(`/api/v1${path}`, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  } catch (_) {
    throw new ApiError(0, 'offline', 'Studio does not answer. Make sure that it is still running.');
  }
  const data = await response.json().catch(() => null);
  if (!response.ok) {
    const error = data && data.error ? data.error : { code: 'unknown', message: response.statusText };
    throw new ApiError(response.status, error.code, error.message);
  }
  return data;
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
export function Field({ id, label, hint, error, children, ...input }) {
  const described = [hint && `${id}-hint`, error && `${id}-error`].filter(Boolean).join(' ');
  return html`<div class="field">
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

function mediumKind(medium) {
  switch ((medium || '').toLowerCase()) {
    case 'publisher': return { dot: 'publisher', label: 'Publisher' };
    case 'musicl': return { dot: 'playlist', label: 'Playlist' };
    case 'music': return { dot: 'feed', label: 'Album' };
    default: return { dot: 'feed', label: 'Feed' };
  }
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

function BackButton() {
  return html`<button class="button quiet" type="button" onClick=${() => history.back()}>← Back</button>`;
}

// ---- First-run wizard (ADR 0005) ----

const MIN_CHARS = 12;
const STRENGTH_WORDS = ['Very weak', 'Weak', 'Fair', 'Strong', 'Very strong'];

// Ask the API for the strength when the user stops typing (ADR 0018).
function useStrength(passphrase) {
  const [strength, setStrength] = useState(null);
  useEffect(() => {
    setStrength(null);
    if (!passphrase) return undefined;
    let current = true;
    const timer = setTimeout(async () => {
      try {
        const result = await api('/vault/strength', { method: 'POST', body: { passphrase } });
        if (current) setStrength(result);
      } catch (_) {
        // The vault still checks the passphrase when the user submits it.
      }
    }, 300);
    return () => { current = false; clearTimeout(timer); };
  }, [passphrase]);
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
    <div class="actions"><button class="button primary" type="button" disabled=${!kept} onClick=${onDone}>Continue</button></div>
  </div>`;
}

function AddHostForm({ onAdded }) {
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

// Select a feed file from the host, or type its path (ADR 0019).
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
    ${!typing && html`<fieldset class="choices" aria-describedby=${feedError.field ? 'feed-choice-error' : undefined}>
      <legend>Feeds in this folder</legend>
      ${listError && html`<${ErrorNotice} error=${listError} />`}
      ${!found && !listError && html`<p class="empty" role="status">Looking for feed files…</p>`}
      ${found && found.feeds.length === 0 && html`<p class="empty">Studio found no feed files in this folder.</p>`}
      ${found && found.feeds.map((f) => {
        const kind = mediumKind(f.medium);
        return html`<label class=${`choice${f.in_collection ? ' taken' : ''}`}>
          <input type="radio" name="feed" value=${f.path} disabled=${f.in_collection}
            checked=${selected === f.path} onChange=${() => setSelected(f.path)} />
          <span class=${`dot ${kind.dot}`}></span>
          <span class="text"><span class="title">${f.title || f.path}</span>
            <span class="sub">${kind.label}${f.blocked ? ' · blocked' : ''} · <span class="mono">${f.path}</span>${f.in_collection ? ' · in your collection' : ''}</span></span>
        </label>`;
      })}
      ${found && found.truncated && html`<p class="hint">The folder has more files than Studio lists. Type the path of a feed that is not in the list.</p>`}
      ${feedError.field && html`<span class="field-error" id="feed-choice-error">${feedError.field}</span>`}
    </fieldset>`}
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

  const read = async (event) => {
    setFileError(null);
    const parsed = [];
    for (const file of event.target.files) {
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
      <div class="actions"><button class="button primary" type="button" onClick=${() => onAdded(result)}>Continue</button></div>
    </div>`;
  }
  return html`<div class="form">
    <div class="field">
      <label for="msp-files">MSP 2.0 backup files</label>
      <span class="hint" id="msp-files-hint">The .json files that MSP 2.0 saved for your hosted feeds. You can select more than one.</span>
      <input id="msp-files" type="file" accept=".json,application/json" multiple aria-describedby="msp-files-hint"
        aria-invalid=${fileError ? 'true' : undefined} onChange=${read} />
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
      <p>A coming version of Studio makes new feeds. Today, Studio reads and keeps feeds that exist.</p></div>
      <div class="actions"><${BackButton} /><button class="button primary" type="button" onClick=${onFinish}>Open Studio</button></div></div>`;
  } else {
    // A reload lost the wizard memory. Start again at the question.
    body = html`<div class="form"><div><h1>Do you already have a feed?</h1></div>
      <div class="actions"><button class="button primary" type="button" onClick=${() => go('find')}>Yes, I have feeds</button>
        <button class="button" type="button" onClick=${() => go('new')}>No, I am new</button></div></div>`;
  }
  return html`<main class="center"><div class="card"><${Brand} />${body}</div></main>`;
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

// Is this feed an album that the editor can open (ADR 0029 item 4)?
function isEditable(feed) {
  return Boolean(feed.connector_id) && mediumKind(feed.medium).label === 'Album';
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
    <div class="bar"><span class="bar-title">${feed.title || 'Untitled feed'}</span></div>
    <section class="hero" style=${{ '--glow': glow }}>
      <div class="art" aria-hidden="true"></div>
      <div>
        <span class="eyebrow"><span class=${`dot ${kind.dot}`}></span>${kind.label}${feed.connector_id ? '' : ' · read-only'}</span>
        <h1>${feed.title || 'Untitled feed'}</h1>
        <p class="summary">${feed.public_url || 'This feed has no public address yet.'}</p>
        <div class="actions">
          <button class="button primary" type="button" disabled=${busy || !(feed.connector_id || feed.public_url)} onClick=${refresh}>
            ${busy ? 'Reading…' : 'Read again'}</button>
          <button class="button" type="button" disabled=${busy} onClick=${remove}>Remove from collection</button>
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
const PAGE_NAMES = { credits: 'Credits and links', recipients: 'Recipients' };

// The toolbar title: the feed title on the album page, or the feed title in
// the muted color, then the name of the open page (review of task 014c,
// defect 5).
function Crumb({ feed, view, draft }) {
  const page = view.page || 'album';
  const name = feed.title || 'Untitled feed';
  if (page === 'album') return html`<span class="bar-title">${name}</span>`;
  let label = PAGE_NAMES[page];
  if (page === 'track') {
    const item = draft.model && draft.model.items.find((i) => trackKey(i) === view.track);
    label = item ? item.title || 'Untitled track' : 'Track';
  }
  return html`<span class="bar-title"><span class="muted">${name}</span> › ${label}</span>`;
}

// An editable album: the compact header, not the hero (ADR 0029 item 10).
// The toolbar stays in view at each scroll position (ADR 0029 item 1).
function EditableFeed({ feed, draft, view, go, adding, onAdding, onChanged, onRemoved }) {
  const { busy, error, run } = useAction();
  const kind = mediumKind(feed.medium);
  const refresh = () => run(async () => onChanged(await api(`/feeds/${feed.id}/refresh`, { method: 'POST' })));
  const glow = `var(--c-${kind.dot === 'playlist' ? 'playlist' : kind.dot === 'publisher' ? 'publisher' : 'feed'})`;
  return html`<div>
    <div class="bar">
      <${Crumb} feed=${feed} view=${view} draft=${draft} />
      <${DraftToolbar} key=${feed.id} feedId=${feed.id} draft=${draft} onWritten=${onChanged} />
    </div>
    <div class="feed-head">
      <header class="head">
        <div class="cover"><span class="mark" style=${{ '--glow': glow }}></span></div>
        <div class="text">
          <span class="eyebrow">${kind.label}</span>
          <h1>${feed.title || 'Untitled feed'}</h1>
          <p class="sub mono">${feed.public_url || 'This feed has no public address yet.'}</p>
        </div>
        <button class="button" type="button" disabled=${busy} onClick=${refresh}>${busy ? 'Reading…' : 'Read again'}</button>
      </header>
      <${ErrorNotice} error=${error} />
      <${AddressDifferences} feed=${feed} />
    </div>
    <${AlbumEditor} key=${feed.id} feed=${feed} draft=${draft} view=${view} go=${go}
      adding=${adding} onAdding=${onAdding} onRemoved=${onRemoved} />
  </div>`;
}

// The feed open in the main pane: the editor for an editable album, or the
// read-only view for every other feed (ADR 0029 item 10).
function FeedDetail({ feed, draft, view, go, adding, onAdding, onChanged, onRemoved }) {
  if (isEditable(feed)) {
    return html`<${EditableFeed} feed=${feed} draft=${draft} view=${view} go=${go}
      adding=${adding} onAdding=${onAdding} onChanged=${onChanged} onRemoved=${onRemoved} />`;
  }
  return html`<${ReadOnlyFeed} feed=${feed} onChanged=${onChanged} onRemoved=${onRemoved} />`;
}

// The sidebar tree: each feed, and the pages of the selected feed (ADR 0029
// items 5 and 6). A click on a different feed opens its pages. A click on
// the open feed opens or closes its pages. Only the selected feed shows its
// pages, and only after its draft loads.
function FeedTree({ feeds, view, draft, go, onAdding }) {
  const selectedId = view.name === 'feed' ? view.id : null;
  const [expanded, setExpanded] = useState(selectedId);
  useEffect(() => { setExpanded(selectedId); }, [selectedId]);
  const toggleFeed = (feed, canEdit) => {
    if (feed.id === selectedId) {
      if (canEdit) setExpanded((current) => (current === feed.id ? null : feed.id));
    } else {
      go({ name: 'feed', id: feed.id });
      setExpanded(feed.id);
    }
  };
  const openPage = (feed, page) => go({ name: 'feed', id: feed.id, ...page });
  // Is this tree item the page that the view shows now?
  const isCurrent = (page) => {
    if (selectedId === null) return false;
    const target = page.page || 'album';
    if ((view.page || 'album') !== target) return false;
    return target !== 'track' || view.track === page.track;
  };
  return html`<ul class="tree">
    ${feeds.map((feed) => {
      const kind = mediumKind(feed.medium);
      const canEdit = isEditable(feed);
      const isOpen = canEdit && feed.id === selectedId && expanded === feed.id && draft.model !== null;
      return html`<li key=${feed.id}>
        <button class="node feed" type="button" aria-current=${feed.id === selectedId ? 'true' : 'false'}
          aria-expanded=${canEdit ? (isOpen ? 'true' : 'false') : undefined}
          onClick=${() => toggleFeed(feed, canEdit)}>
          ${canEdit && html`<span class="chev" aria-hidden="true">▸</span>`}
          <span class=${`dot ${kind.dot}`}></span>
          <span class="label">${feed.title || feed.path || 'Untitled feed'}</span>
        </button>
        ${isOpen && html`<ul>
          <li><button class="node" type="button" aria-current=${isCurrent({}) ? 'page' : 'false'}
            onClick=${() => openPage(feed, {})}><span class="label">Album</span></button></li>
          <li><button class="node" type="button" aria-current=${isCurrent({ page: 'credits' }) ? 'page' : 'false'}
            onClick=${() => openPage(feed, { page: 'credits' })}><span class="label">Credits and links</span></button></li>
          <li><button class="node" type="button" aria-current=${isCurrent({ page: 'recipients' }) ? 'page' : 'false'}
            onClick=${() => openPage(feed, { page: 'recipients' })}><span class="label">Recipients</span></button></li>
          <li class="tree-sub">Tracks</li>
          ${draft.model.items.map((item, i) => {
            const key = trackKey(item);
            const bad = draft.bad.some((b) => b.field.startsWith(`items/${i}/`));
            return html`<li key=${key}><button class="node" type="button"
              aria-current=${isCurrent({ page: 'track', track: key }) ? 'page' : 'false'}
              onClick=${() => openPage(feed, { page: 'track', track: key })}>
              <span class="num">${i + 1}</span><span class="label">${item.title || 'Untitled track'}</span>
              ${bad && html`<span class="field-error">Address not correct</span>`}
            </button></li>`;
          })}
          <li><button class="node add" type="button" onClick=${() => { openPage(feed, {}); onAdding(true); }}>
            <span class="num">＋</span><span class="label">Add a track</span></button></li>
        </ul>`}
      </li>`;
    })}
  </ul>`;
}

// The feed files in a folder that are not in the collection. Studio shows
// them after a folder match, so that a feed that no index lists, such as a
// blocked test feed, is not missed (ADR 0019).
function FolderFeeds({ connector, onAdded }) {
  const [found, setFound] = useState(null);
  const [chosen, setChosen] = useState([]);
  const [result, setResult] = useState(null);
  const { busy, error, run } = useAction();
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
  return html`<div class="panel form">
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

function Hosts({ connectors, feeds, onChanged, onOpen }) {
  const { busy, error, run } = useAction();
  const [linked, setLinked] = useState(null);
  const [adding, setAdding] = useState(false);
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
    <div class="bar">Hosts</div>
    <section class="section">
      ${error && error.code === 'in_use'
        ? html`<div class="notice error" role="alert"><strong>Feeds use this host.</strong>Remove those feeds from your collection first.</div>`
        : html`<${ErrorNotice} error=${error} />`}
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
      ${connectors.map((c) => {
        const config = JSON.parse(c.config);
        const written = feeds.filter((f) => f.connector_id === c.id);
        const disagreeing = feeds.filter((f) => (c.disagreeing_feeds || []).includes(f.id));
        return html`<article class="panel host">
          <header class="host-head">
            <span class="eyebrow"><span class="dot hosting"></span>Folder on this computer</span>
            <button class="button quiet" type="button" disabled=${busy}
              aria-label=${`Remove the host ${c.name}`} onClick=${() => remove(c)}>Remove</button>
          </header>
          <h2>${c.name}</h2>
          <dl class="facts">
            <dt>Folder</dt><dd class="mono">${config.root}</dd>
            <dt>Public address</dt><dd><span class="mono">${config.base_url || '—'}</span>
              ${config.base_url && config.base_url_inferred && html`<span class="sub"> · Studio found it from the feeds</span>`}</dd>
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
        </article>`;
      })}
      ${showForm
        ? html`<div class="panel"><h2>${connectors.length === 0 ? 'Connect your first host' : 'Add a host'}</h2>
            <${AddHostForm} onAdded=${added} />
            ${connectors.length > 0 && html`<button class="button link" type="button" onClick=${() => setAdding(false)}>Cancel</button>`}</div>`
        : html`<div class="actions"><button class="button" type="button" onClick=${() => setAdding(true)}>＋ Add a host</button></div>`}
    </section>
  </div>`;
}

function AddFeeds({ connectors, onAdded }) {
  const [found, setFound] = useState(null);
  return html`<div><div class="bar">Add feeds</div><section class="section">
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

function Shell({ vault, onLock }) {
  const [feeds, setFeeds] = useState(null);
  const [connectors, setConnectors] = useState([]);
  const [view, go] = useNav('shell', { name: 'feeds' });
  const [error, setError] = useState(null);
  // The Add a track sheet, by the tree and by the album page (ADR 0029 item 7).
  const [adding, setAdding] = useState(false);

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

  const lock = async () => { await api('/vault/lock', { method: 'POST' }).catch(() => {}); onLock(); };
  const selected = view.name === 'feed' && feeds ? feeds.find((f) => f.id === view.id) : null;
  // The shell holds the one draft of the selected feed (ADR 0029 item 4).
  // A feed with no connector, or a feed that is not an album, needs no draft.
  const draftFeedId = selected && isEditable(selected) ? selected.id : null;
  const draft = useDraft(draftFeedId);
  // A different feed closes the Add a track sheet of the feed before it.
  useEffect(() => { setAdding(false); }, [draftFeedId]);

  let main;
  if (error) {
    main = html`<section class="section"><${ErrorNotice} error=${error} /></section>`;
  } else if (feeds === null) {
    main = html`<section class="section"><p class="empty" role="status">Loading your collection…</p></section>`;
  } else if (view.name === 'hosts') {
    main = html`<${Hosts} connectors=${connectors} feeds=${feeds} onChanged=${() => load()}
      onOpen=${(id) => go({ name: 'feed', id })} />`;
  } else if (view.name === 'add-feed') {
    main = html`<${AddFeeds} connectors=${connectors}
      onAdded=${async (f) => { await load(); go(f ? { name: 'feed', id: f.id } : { name: 'feeds' }); }} />`;
  } else if (selected) {
    main = html`<${FeedDetail} feed=${selected} draft=${draft} view=${view} go=${go}
      adding=${adding} onAdding=${setAdding} onChanged=${() => load()}
      onRemoved=${async () => { await load(); go({ name: 'feeds' }, { replace: true }); }} />`;
  } else {
    main = html`<section class="center" style=${{ minHeight: '100%' }}><div class="card">
      <h1>${feeds.length ? 'Select a feed' : 'Your collection is empty'}</h1>
      <p>${feeds.length ? 'Select one in the Feeds list.' : 'Add a feed to start.'}</p>
      <div class="actions"><button class="button primary" type="button" onClick=${() => go({ name: 'add-feed' })}>Add a feed</button></div>
    </div></section>`;
  }

  return html`<div class="shell">
    <nav class="sidebar" aria-label="Collection">
      <header><${Brand} /><${ThemeToggle} /></header>
      <h2>Feeds</h2>
      ${feeds && feeds.length === 0 && html`<p class="empty">No feeds yet.</p>`}
      <${FeedTree} feeds=${feeds || []} view=${view} draft=${draft} go=${go} onAdding=${setAdding} />
      <button class="row" type="button" onClick=${() => go({ name: 'add-feed' })}>＋ Add a feed</button>
      <footer>
        <button class="row" type="button" aria-current=${view.name === 'hosts' ? 'true' : 'false'}
          onClick=${() => go({ name: 'hosts' })}><span class="dot hosting"></span>Hosts</button>
        ${vault === 'open' && html`<button class="row" type="button" onClick=${lock}>Lock Studio</button>`}
      </footer>
    </nav>
    <main class="main">${main}</main>
  </div>`;
}

// ---- App ----

function App() {
  const [status, setStatus] = useState(null);
  const [skipped, setSkipped] = useState(false);
  const [error, setError] = useState(null);
  const load = useCallback(async () => {
    try {
      setStatus(await api('/status'));
      setError(null);
    } catch (e) {
      setError(e);
    }
  }, []);
  useEffect(() => { if (token) load(); }, [load]);

  if (!token) return html`<${NoToken} />`;
  if (error && error.status === 401) return html`<${NoToken} />`;
  if (error) return html`<main class="center"><div class="card"><${Brand} /><${ErrorNotice} error=${error} />
    <div class="actions"><button class="button" type="button" onClick=${load}>Try again</button></div></div></main>`;
  if (!status) return html`<p class="boot" role="status">Loading Studio…</p>`;
  if (status.vault === 'locked') return html`<${Unlock} onUnlocked=${load} />`;
  // The first run: an empty collection (ADR 0020). The vault comes when the
  // user first adds a secret.
  if (status.feeds === 0 && status.connectors === 0 && !skipped) {
    return html`<${Wizard} onFinish=${() => { setSkipped(true); load(); }} />`;
  }
  return html`<${Shell} vault=${status.vault} onLock=${load} />`;
}

render(html`<${App} />`, document.getElementById('app'));

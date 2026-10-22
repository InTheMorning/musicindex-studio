// Music Index Studio UI (ADR 0016). Preact and htm, with no build step.
import { html, render, useState, useEffect, useCallback } from './vendor/htm-preact-standalone.module.js';

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

async function api(path, { method = 'GET', body } = {}) {
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
  return html`<button class="theme-toggle" type="button" onClick=${toggle}
    aria-label=${`Use the ${next} theme`} title=${`Use the ${next} theme`}>
    ${theme === 'dark' ? '☀' : '☾'}</button>`;
}

function Brand() {
  return html`<span class="brand"><span class="mark" aria-hidden="true"></span>Studio</span>`;
}

function ErrorNotice({ error }) {
  if (!error) return null;
  return html`<div class="notice error" role="alert"><strong>${title(error)}</strong>${error.message}</div>`;
}

function title(error) {
  switch (error.code) {
    case 'wrong_key': return 'The passphrase is not correct.';
    case 'passphrase_too_short': return 'The passphrase is too short.';
    case 'invalid_path': return 'That path is not in the folder.';
    case 'not_found': return 'Studio did not find the file.';
    case 'invalid_feed': return 'That file is not a feed that Studio can read.';
    case 'offline': return 'No connection to Studio.';
    default: return 'Something went wrong.';
  }
}

function Field({ id, label, hint, ...input }) {
  return html`<div class="field">
    <label for=${id}>${label}</label>
    ${hint && html`<span class="hint" id=${`${id}-hint`}>${hint}</span>`}
    <input id=${id} aria-describedby=${hint ? `${id}-hint` : undefined} ...${input} />
  </div>`;
}

// Run an API action with a busy state and an error state.
function useAction() {
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

// ---- First-run wizard (ADR 0005) ----

function Steps({ current }) {
  return html`<ol class="steps" aria-label=${`Step ${current} of 3`}>
    ${[1, 2, 3].map((n) => html`<li class=${n < current ? 'done' : n === current ? 'current' : ''}></li>`)}
  </ol>`;
}

function CreateVault({ onDone }) {
  const [passphrase, setPassphrase] = useState('');
  const [confirm, setConfirm] = useState('');
  const [recovery, setRecovery] = useState(true);
  const { busy, error, run } = useAction();
  const mismatch = confirm.length > 0 && confirm !== passphrase;
  const submit = (event) => {
    event.preventDefault();
    run(async () => {
      const result = await api('/vault', { method: 'POST', body: { passphrase, recovery_key: recovery } });
      onDone(result.recovery_key);
    });
  };
  return html`<form class="form" onSubmit=${submit}>
    <${Steps} current=${1} />
    <div><h1>Protect your keys</h1>
      <p>Studio keeps the passwords and keys of your hosts in a vault. Only your passphrase opens it.</p></div>
    <${Field} id="passphrase" label="Passphrase" type="password" autocomplete="new-password" required
      hint="A minimum of 12 characters. A few unrelated words are easy to remember."
      value=${passphrase} onInput=${(e) => setPassphrase(e.target.value)} />
    <${Field} id="confirm" label="Type the passphrase again" type="password" autocomplete="new-password" required
      value=${confirm} onInput=${(e) => setConfirm(e.target.value)} />
    ${mismatch && html`<div class="notice" role="status">The two passphrases are not the same.</div>`}
    <label class="check"><input type="checkbox" checked=${recovery} onChange=${(e) => setRecovery(e.target.checked)} />
      Make a recovery key, in case I forget the passphrase</label>
    <${ErrorNotice} error=${error} />
    <div class="actions">
      <button class="button primary" type="submit"
        disabled=${busy || passphrase.length < 12 || passphrase !== confirm}>${busy ? 'Making the vault…' : 'Make the vault'}</button>
    </div>
  </form>`;
}

function ShowRecoveryKey({ recoveryKey, onDone }) {
  const [kept, setKept] = useState(false);
  return html`<div class="form">
    <${Steps} current=${1} />
    <div><h1>Write down your recovery key</h1>
      <p>Studio shows this key one time only. With it, you can open the vault if you forget your passphrase.</p></div>
    <div class="recovery" aria-label="Recovery key">${recoveryKey}</div>
    <label class="check"><input type="checkbox" checked=${kept} onChange=${(e) => setKept(e.target.checked)} />
      I wrote down the recovery key in a safe place</label>
    <div class="actions"><button class="button primary" type="button" disabled=${!kept} onClick=${onDone}>Continue</button></div>
  </div>`;
}

function AddHostForm({ onAdded, submitLabel = 'Add the folder' }) {
  const [name, setName] = useState('My feeds');
  const [root, setRoot] = useState('');
  const [baseUrl, setBaseUrl] = useState('');
  const { busy, error, run } = useAction();
  const submit = (event) => {
    event.preventDefault();
    run(async () => {
      const connector = await api('/connectors', {
        method: 'POST',
        body: { kind: 'local', name, root, base_url: baseUrl || null },
      });
      onAdded(connector);
    });
  };
  return html`<form class="form" onSubmit=${submit}>
    <${Field} id="host-name" label="Name" required value=${name} onInput=${(e) => setName(e.target.value)} />
    <${Field} id="host-root" label="Folder" required placeholder="/home/me/feeds"
      hint="The full path of the folder that holds your feed files."
      value=${root} onInput=${(e) => setRoot(e.target.value)} />
    <${Field} id="host-url" label="Public address of the folder (optional)" type="url"
      placeholder="https://music.example.com/feeds"
      hint="When a web server publishes this folder, Studio uses this address for each feed."
      value=${baseUrl} onInput=${(e) => setBaseUrl(e.target.value)} />
    <${ErrorNotice} error=${error} />
    <div class="actions"><button class="button primary" type="submit" disabled=${busy || !root}>${busy ? 'Adding…' : submitLabel}</button></div>
  </form>`;
}

function AddFeedForm({ connectors, onAdded }) {
  const [connectorId, setConnectorId] = useState(connectors[0] ? connectors[0].id : null);
  const [path, setPath] = useState('');
  const { busy, error, run } = useAction();
  const submit = (event) => {
    event.preventDefault();
    run(async () => {
      const feed = await api('/feeds', { method: 'POST', body: { connector_id: connectorId, path } });
      const read = await api(`/feeds/${feed.id}/refresh`, { method: 'POST' }).catch((e) => {
        onAdded(feed);
        throw e;
      });
      onAdded(read);
    });
  };
  if (connectors.length === 0) {
    return html`<div class="notice">Add a host first. Studio writes each feed through its host.</div>`;
  }
  return html`<form class="form" onSubmit=${submit}>
    ${connectors.length > 1 && html`<div class="field"><label for="feed-host">Host</label>
      <select id="feed-host" class="button" value=${connectorId}
        onChange=${(e) => setConnectorId(Number(e.target.value))}>
        ${connectors.map((c) => html`<option value=${c.id}>${c.name}</option>`)}
      </select></div>`}
    <${Field} id="feed-path" label="Feed file" required placeholder="album/feed.xml"
      hint="The path of the feed file in the folder of the host."
      value=${path} onInput=${(e) => setPath(e.target.value)} />
    <${ErrorNotice} error=${error} />
    <div class="actions"><button class="button primary" type="submit" disabled=${busy || !path}>${busy ? 'Reading the feed…' : 'Add the feed'}</button></div>
  </form>`;
}

function Wizard({ onFinish }) {
  const [step, setStep] = useState('vault');
  const [recoveryKey, setRecoveryKey] = useState(null);
  const [connector, setConnector] = useState(null);
  const [feed, setFeed] = useState(null);
  let body;
  if (step === 'vault') {
    body = html`<${CreateVault} onDone=${(key) => { setRecoveryKey(key); setStep(key ? 'recovery' : 'host'); }} />`;
  } else if (step === 'recovery') {
    body = html`<${ShowRecoveryKey} recoveryKey=${recoveryKey} onDone=${() => { setRecoveryKey(null); setStep('host'); }} />`;
  } else if (step === 'host') {
    body = html`<div class="form"><${Steps} current=${2} />
      <div><h1>Connect a host</h1><p>A host keeps your feed files. Start with a folder on this computer. Studio can connect other hosts later.</p></div>
      <${AddHostForm} onAdded=${(c) => { setConnector(c); setStep('feed'); }} /></div>`;
  } else if (step === 'feed') {
    body = html`<div class="form"><${Steps} current=${3} />
      <div><h1>Add your first feed</h1><p>Studio reads the feed and remembers it in your collection.</p></div>
      <${AddFeedForm} connectors=${[connector]} onAdded=${(f) => { setFeed(f); setStep('done'); }} />
      <div class="actions"><button class="button quiet" type="button" onClick=${onFinish}>Skip for now</button></div></div>`;
  } else {
    body = html`<div class="form"><${Steps} current=${4} />
      <div><h1>Studio is ready</h1>
        <p>${feed && feed.title ? `“${feed.title}” is in your collection.` : 'Your collection is ready.'}</p></div>
      <div class="actions"><button class="button primary" type="button" onClick=${onFinish}>Open Studio</button></div></div>`;
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
  return html`<main class="center"><form class="card" onSubmit=${submit}>
    <${Brand} />
    <div><h1>Unlock Studio</h1><p>Your passphrase opens the vault with the keys of your hosts.</p></div>
    <${Field} id="unlock" label="Passphrase" type="password" autocomplete="current-password" required autofocus
      value=${passphrase} onInput=${(e) => setPassphrase(e.target.value)} />
    <${ErrorNotice} error=${error} />
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

function FeedDetail({ feed, onChanged }) {
  const { busy, error, run } = useAction();
  const kind = mediumKind(feed.medium);
  const refresh = () => run(async () => onChanged(await api(`/feeds/${feed.id}/refresh`, { method: 'POST' })));
  const glow = `var(--c-${kind.dot === 'playlist' ? 'playlist' : kind.dot === 'publisher' ? 'publisher' : 'feed'})`;
  return html`<div>
    <div class="bar">${feed.title || 'Untitled feed'}</div>
    <section class="hero" style=${{ '--glow': glow }}>
      <div class="art" aria-hidden="true"></div>
      <div>
        <span class="eyebrow"><span class=${`dot ${kind.dot}`}></span>${kind.label}</span>
        <h1>${feed.title || 'Untitled feed'}</h1>
        <p class="summary">${feed.public_url || 'This feed has no public address yet.'}</p>
        <div class="actions">
          <button class="button primary" type="button" disabled=${busy || !feed.connector_id} onClick=${refresh}>
            ${busy ? 'Reading…' : 'Read again'}</button>
        </div>
      </div>
    </section>
    <section class="section">
      <${ErrorNotice} error=${error} />
      <h2>Facts</h2>
      <dl class="facts">
        <dt>Feed GUID</dt><dd class="mono">${feed.podcast_guid || '—'}</dd>
        <dt>File</dt><dd class="mono">${feed.path || 'Read-only: no host'}</dd>
        <dt>Version</dt><dd class="mono">${feed.last_version || 'Not read yet'}</dd>
      </dl>
      <p class="hint">The editor for albums comes next. Today Studio reads and remembers your feeds.</p>
    </section>
  </div>`;
}

function Hosts({ connectors, onAdded }) {
  return html`<div>
    <div class="bar">Hosts</div>
    <section class="section">
      <h2>Your hosts</h2>
      ${connectors.length === 0
        ? html`<p class="empty">No host yet.</p>`
        : html`<ul class="rows">${connectors.map((c) => html`<li class="row">
            <span class="dot hosting"></span>
            <span class="text"><span class="title">${c.name}</span>
              <span class="sub mono">${JSON.parse(c.config).root}</span></span></li>`)}</ul>`}
      <div class="panel"><h2>Add a folder</h2><${AddHostForm} onAdded=${onAdded} /></div>
    </section>
  </div>`;
}

function Shell({ onLock }) {
  const [feeds, setFeeds] = useState(null);
  const [connectors, setConnectors] = useState([]);
  const [view, setView] = useState({ name: 'feeds' });
  const [error, setError] = useState(null);

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

  let main;
  if (error) {
    main = html`<section class="section"><${ErrorNotice} error=${error} /></section>`;
  } else if (feeds === null) {
    main = html`<section class="section"><p class="empty" role="status">Loading your collection…</p></section>`;
  } else if (view.name === 'hosts') {
    main = html`<${Hosts} connectors=${connectors} onAdded=${() => load()} />`;
  } else if (view.name === 'add-feed') {
    main = html`<div><div class="bar">Add a feed</div><section class="section"><div class="panel">
      <${AddFeedForm} connectors=${connectors} onAdded=${async (f) => { await load(); setView({ name: 'feed', id: f.id }); }} />
    </div></section></div>`;
  } else if (selected) {
    main = html`<${FeedDetail} feed=${selected} onChanged=${() => load()} />`;
  } else {
    main = html`<section class="center" style=${{ minHeight: '100%' }}><div class="card">
      <h1>${feeds.length ? 'Select a feed' : 'Your collection is empty'}</h1>
      <p>${feeds.length ? 'Your feeds are in the list at the side.' : 'Add a feed to start.'}</p>
      <div class="actions"><button class="button primary" type="button" onClick=${() => setView({ name: 'add-feed' })}>Add a feed</button></div>
    </div></section>`;
  }

  return html`<div class="shell">
    <nav class="sidebar" aria-label="Collection">
      <header><${Brand} /><${ThemeToggle} /></header>
      <h2>Feeds</h2>
      ${feeds && feeds.length === 0 && html`<p class="empty">No feeds yet.</p>`}
      <ul class="rows">
        ${(feeds || []).map((f) => {
          const kind = mediumKind(f.medium);
          return html`<li><button class="row" type="button" aria-current=${view.name === 'feed' && view.id === f.id ? 'true' : 'false'}
            onClick=${() => setView({ name: 'feed', id: f.id })}>
            <span class=${`dot ${kind.dot}`}></span>
            <span class="text"><span class="title">${f.title || f.path || 'Untitled feed'}</span>
              <span class="sub">${kind.label}${f.connector_id ? '' : ' · read-only'}</span></span>
          </button></li>`;
        })}
      </ul>
      <button class="row" type="button" onClick=${() => setView({ name: 'add-feed' })}>＋ Add a feed</button>
      <footer>
        <button class="row" type="button" aria-current=${view.name === 'hosts' ? 'true' : 'false'}
          onClick=${() => setView({ name: 'hosts' })}><span class="dot hosting"></span>Hosts</button>
        <button class="row" type="button" onClick=${lock}>Lock Studio</button>
      </footer>
    </nav>
    <main class="main">${main}</main>
  </div>`;
}

// ---- App ----

function App() {
  const [status, setStatus] = useState(null);
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
  if (status.vault === 'absent') return html`<${Wizard} onFinish=${load} />`;
  if (status.vault === 'locked') return html`<${Unlock} onUnlocked=${load} />`;
  return html`<${Shell} onLock=${load} />`;
}

render(html`<${App} />`, document.getElementById('app'));

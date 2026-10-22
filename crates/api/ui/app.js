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
    case 'passphrase_too_weak': return 'The passphrase is too easy to guess.';
    case 'invalid_path': return 'That path is not in the folder.';
    case 'not_found': return 'Studio did not find the file.';
    case 'invalid_feed': return 'That file is not a feed that Studio can read.';
    case 'offline': return 'No connection to Studio.';
    default: return 'Something went wrong.';
  }
}

// A labelled input. `error` is a line of red text below the field (ADR 0018).
function Field({ id, label, hint, error, children, ...input }) {
  const described = [hint && `${id}-hint`, error && `${id}-error`].filter(Boolean).join(' ');
  return html`<div class="field">
    <label for=${id}>${label}</label>
    ${hint && html`<span class="hint" id=${`${id}-hint`}>${hint}</span>`}
    <input id=${id} aria-invalid=${error ? 'true' : undefined} aria-describedby=${described || undefined} ...${input} />
    ${error && html`<span class="field-error" id=${`${id}-error`}>${error}</span>`}
    ${children}
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

function Steps({ current }) {
  return html`<ol class="steps" aria-label=${`Step ${current} of 3`}>
    ${[1, 2, 3].map((n) => html`<li class=${n < current ? 'done' : n === current ? 'current' : ''}></li>`)}
  </ol>`;
}

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
  const passError = passphraseError(passphrase, strength);
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
    <${Steps} current=${1} />
    <div><h1>Protect your keys</h1>
      <p>Studio keeps the passwords and keys of your hosts in a vault. Only your passphrase opens it.</p></div>
    <${Field} id="passphrase" label="Passphrase" type="password" autocomplete="new-password" required
      value=${passphrase} onInput=${(e) => setPassphrase(e.target.value)} error=${passError}>
      <${Meter} strength=${strength} />
    <//>
    <${Field} id="confirm" label="Type the passphrase again" type="password" autocomplete="new-password" required
      value=${confirm} onInput=${(e) => setConfirm(e.target.value)} error=${confirmError} />
    <label class="check"><input type="checkbox" checked=${recovery} onChange=${(e) => setRecovery(e.target.checked)} />
      Make a recovery key, in case I forget the passphrase</label>
    <${ErrorNotice} error=${error} />
    <div class="actions">
      <button class="button primary" type="submit" disabled=${busy || !ready}>${busy ? 'Making the vault…' : 'Make the vault'}</button>
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

function AddHostForm({ onAdded, replaces = null, submitLabel = 'Add the folder' }) {
  const config = replaces ? JSON.parse(replaces.config) : {};
  const [name, setName] = useState(replaces ? replaces.name : 'My feeds');
  const [root, setRoot] = useState(config.root || '');
  const [baseUrl, setBaseUrl] = useState(config.base_url || '');
  const { busy, error, run } = useAction();
  const submit = (event) => {
    event.preventDefault();
    run(async () => {
      if (replaces) {
        // A connector that a feed uses stays. The new one is added beside it.
        await api(`/connectors/${replaces.id}`, { method: 'DELETE' }).catch((e) => {
          if (e.code !== 'in_use') throw e;
        });
      }
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
  // The vault step and the recovery key step have no way back: the vault
  // exists after the first, and Studio shows the key one time only.
  const [step, go] = useNav('wizard', 'vault');
  const [recoveryKey, setRecoveryKey] = useState(null);
  const [connector, setConnector] = useState(null);
  const [feed, setFeed] = useState(null);
  let body;
  if (step === 'vault') {
    body = html`<${CreateVault} onDone=${(key) => { setRecoveryKey(key); go(key ? 'recovery' : 'host', { replace: true }); }} />`;
  } else if (step === 'recovery' && recoveryKey) {
    body = html`<${ShowRecoveryKey} recoveryKey=${recoveryKey} onDone=${() => { setRecoveryKey(null); go('host', { replace: true }); }} />`;
  } else if (step === 'host' || step === 'recovery') {
    body = html`<div class="form"><${Steps} current=${2} />
      <div><h1>Connect a host</h1><p>A host keeps your feed files. Start with a folder on this computer. Studio can connect other hosts later.</p></div>
      ${connector && html`<div class="notice" role="status">You added the folder <code>${JSON.parse(connector.config).root}</code>.
        Change it below, or continue with it.</div>
        <div class="actions"><button class="button" type="button" onClick=${() => go('feed')}>Continue with this folder</button></div>`}
      <${AddHostForm} key=${connector ? connector.id : 'new'} replaces=${connector}
        submitLabel=${connector ? 'Use the changed folder' : 'Add the folder'}
        onAdded=${(c) => { setConnector(c); go('feed'); }} /></div>`;
  } else if (step === 'feed' && connector) {
    body = html`<div class="form"><${Steps} current=${3} />
      <div><h1>Add your first feed</h1><p>Studio reads the feed and remembers it in your collection.</p></div>
      <${AddFeedForm} connectors=${[connector]} onAdded=${(f) => { setFeed(f); go('done'); }} />
      <div class="actions"><${BackButton} /><button class="button quiet" type="button" onClick=${onFinish}>Skip for now</button></div></div>`;
  } else if (step === 'done') {
    body = html`<div class="form"><${Steps} current=${4} />
      <div><h1>Studio is ready</h1>
        <p>${feed && feed.title ? `“${feed.title}” is in your collection.` : 'Your collection is ready.'}</p></div>
      <div class="actions"><${BackButton} /><button class="button primary" type="button" onClick=${onFinish}>Open Studio</button></div></div>`;
  } else {
    // A reload lost the wizard memory. The vault exists, so start at the host step.
    body = html`<div class="form"><${Steps} current=${2} />
      <div><h1>Connect a host</h1><p>A host keeps your feed files.</p></div>
      <${AddHostForm} onAdded=${(c) => { setConnector(c); go('feed'); }} /></div>`;
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

function FeedDetail({ feed, onChanged, onRemoved }) {
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
          <button class="button" type="button" disabled=${busy} onClick=${remove}>Remove from collection</button>
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

function Hosts({ connectors, onChanged }) {
  const { busy, error, run } = useAction();
  const remove = (connector) => {
    if (!confirm(`Remove the host “${connector.name}”? The files in it do not change.`)) return;
    run(async () => {
      await api(`/connectors/${connector.id}`, { method: 'DELETE' });
      onChanged();
    });
  };
  return html`<div>
    <div class="bar">Hosts</div>
    <section class="section">
      <h2>Your hosts</h2>
      ${error && error.code === 'in_use'
        ? html`<div class="notice error" role="alert"><strong>Feeds use this host.</strong>Remove those feeds from your collection first.</div>`
        : html`<${ErrorNotice} error=${error} />`}
      ${connectors.length === 0
        ? html`<p class="empty">No host yet.</p>`
        : html`<ul class="rows">${connectors.map((c) => html`<li class="row">
            <span class="dot hosting"></span>
            <span class="text"><span class="title">${c.name}</span>
              <span class="sub mono">${JSON.parse(c.config).root}</span></span>
            <button class="button quiet" type="button" disabled=${busy} style=${{ marginLeft: 'auto' }}
              aria-label=${`Remove the host ${c.name}`} onClick=${() => remove(c)}>Remove</button></li>`)}</ul>`}
      <div class="panel"><h2>Add a folder</h2><${AddHostForm} onAdded=${onChanged} /></div>
    </section>
  </div>`;
}

function Shell({ onLock }) {
  const [feeds, setFeeds] = useState(null);
  const [connectors, setConnectors] = useState([]);
  const [view, setView] = useNav('shell', { name: 'feeds' });
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
    main = html`<${Hosts} connectors=${connectors} onChanged=${() => load()} />`;
  } else if (view.name === 'add-feed') {
    main = html`<div><div class="bar">Add a feed</div><section class="section"><div class="panel">
      <${AddFeedForm} connectors=${connectors} onAdded=${async (f) => { await load(); setView({ name: 'feed', id: f.id }); }} />
    </div></section></div>`;
  } else if (selected) {
    main = html`<${FeedDetail} feed=${selected} onChanged=${() => load()}
      onRemoved=${async () => { await load(); setView({ name: 'feeds' }, { replace: true }); }} />`;
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

// The album editor (task 011, ADR 0022). The UI edits the typed model of the
// draft. Each list entry carries its key, and the draft saves automatically.
import { html, useState, useEffect, useRef, useCallback } from './vendor/htm-preact-standalone.module.js';
import { api, Field, ErrorNotice, useAction } from './app.js';

// ---- Client IDs and keys ----
// Each list entry gets a client ID (`_cid`) for the life of the page. After a
// save, the position of an entry in the sent list is its new key (ADR 0022
// item 5). The client ID finds that position, also when the user changed the
// model during the save.

let nextCid = 1;

function withCids(value) {
  if (Array.isArray(value)) {
    return value.map((entry) => (entry && typeof entry === 'object' ? { ...withCids(entry), _cid: nextCid++ } : entry));
  }
  if (value && typeof value === 'object') {
    const copy = {};
    for (const [name, field] of Object.entries(value)) copy[name] = withCids(field);
    return copy;
  }
  return value;
}

// The model without client IDs, as the API gets it.
function strip(model) {
  return JSON.parse(JSON.stringify(model, (name, value) => (name === '_cid' ? undefined : value)));
}

// The position of each entry in its list, by client ID.
function positions(value, found = new Map()) {
  if (Array.isArray(value)) {
    value.forEach((entry, index) => {
      if (entry && entry._cid) found.set(entry._cid, index);
      positions(entry, found);
    });
  } else if (value && typeof value === 'object') {
    for (const field of Object.values(value)) positions(field, found);
  }
  return found;
}

// Give each entry that the server has the key of its position. An entry that
// the user added during the save has no key yet.
function rekey(value, found) {
  if (Array.isArray(value)) {
    return value.map((entry) => {
      if (!entry || typeof entry !== 'object' || !entry._cid) return entry;
      const next = rekey(entry, found);
      const key = found.get(entry._cid);
      if (key === undefined) delete next.key; else next.key = key;
      return next;
    });
  }
  if (value && typeof value === 'object') {
    const copy = {};
    for (const [name, field] of Object.entries(value)) copy[name] = rekey(field, found);
    return copy;
  }
  return value;
}

// The server sets two fields of a track: the GUID of a new track (ADR 0022
// item 7) and the track number (ADR 0024). Copy them into the local model, or
// the next save would change them back.
function copyServerFields(local, saved, found) {
  return {
    ...local,
    items: local.items.map((item) => {
      const index = found.get(item._cid);
      const from = index === undefined ? null : saved.items[index];
      if (!from) return item;
      return { ...item, guid: item.guid || from.guid, episode: from.episode };
    }),
  };
}

// ---- The draft and its automatic save ----

const SAVE_DELAY = 700;

function useDraft(feedId) {
  const [model, setModel] = useState(null);
  const [changed, setChanged] = useState(false);
  // Each address field that is not correct, by its model path (ADR 0023).
  const [bad, setBad] = useState([]);
  const [status, setStatus] = useState('loading');
  const [error, setError] = useState(null);
  // The number of user edits. After an edit, a diff is not current.
  const [edits, setEdits] = useState(0);
  const current = useRef(null);
  const revision = useRef(null);
  const dirty = useRef(false);
  const inflight = useRef(null);
  const timer = useRef(null);

  const show = (draft) => {
    clearTimeout(timer.current);
    current.current = withCids(draft.model);
    revision.current = draft.revision;
    dirty.current = false;
    setModel(current.current);
    setChanged(draft.changed);
    setBad(draft.bad_addresses);
    setStatus('saved');
    setError(null);
  };

  const load = useCallback(async () => {
    setStatus('loading');
    try {
      show(await api(`/feeds/${feedId}/draft`));
    } catch (e) {
      setError(e);
      setStatus('failed');
    }
  }, [feedId]);
  useEffect(() => { load(); return () => clearTimeout(timer.current); }, [load]);

  const saveOnce = async () => {
    dirty.current = false;
    const sent = current.current;
    setStatus('saving');
    const saved = await api(`/feeds/${feedId}/draft`, {
      method: 'PUT',
      body: { revision: revision.current, model: strip(sent) },
    });
    const found = positions(sent);
    revision.current = saved.revision;
    current.current = copyServerFields(rekey(current.current, found), saved.model, found);
    setModel(current.current);
    setChanged(saved.changed);
    setBad(saved.bad_addresses);
  };

  // Send each change that is not saved. Only one save runs at a time.
  const flush = useCallback(async () => {
    clearTimeout(timer.current);
    while (dirty.current || inflight.current) {
      if (inflight.current) {
        await inflight.current.catch(() => {});
        continue;
      }
      inflight.current = saveOnce();
      try {
        await inflight.current;
      } catch (e) {
        dirty.current = true;
        setError(e);
        setStatus('failed');
        return false;
      } finally {
        inflight.current = null;
      }
    }
    setError(null);
    setStatus('saved');
    return true;
  }, [feedId]);

  const update = useCallback((change) => {
    current.current = change(current.current);
    dirty.current = true;
    setModel(current.current);
    setEdits((n) => n + 1);
    setStatus('waiting');
    clearTimeout(timer.current);
    timer.current = setTimeout(flush, SAVE_DELAY);
  }, [flush]);

  return { model, changed, bad, status, error, edits, update, flush, load, show };
}

// ---- Fields ----

// The text of an optional field. An empty field is `null`, and the writer then
// removes the element (ADR 0022 item 6).
const optional = (text) => (text === '' ? null : text);

const BAD_ADDRESS = 'This is not a full web address. Start it with https://.';

// A text field. An address field gives its model path in `address`. The
// field then shows red text when the core finds the address not correct
// (ADR 0023 item 4).
function TextField({ id, label, value, onChange, required = false, address, bad = [], ...rest }) {
  const error = address && bad.some((b) => b.field === address) ? BAD_ADDRESS : null;
  return html`<${Field} id=${id} label=${label} value=${value ?? ''} error=${error} ...${rest}
    onInput=${(e) => onChange(required ? e.target.value : optional(e.target.value))} />`;
}

function TextArea({ id, label, value, onChange }) {
  return html`<div class="field">
    <label for=${id}>${label}</label>
    <textarea id=${id} value=${value ?? ''} onInput=${(e) => onChange(optional(e.target.value))}></textarea>
  </div>`;
}

// Change one field of an object.
const set = (object, name, value) => ({ ...object, [name]: value });

// Change one entry of a list.
const setAt = (list, index, entry) => list.map((e, i) => (i === index ? entry : e));

function move(list, index, by) {
  const target = index + by;
  if (target < 0 || target >= list.length) return list;
  const copy = [...list];
  [copy[index], copy[target]] = [copy[target], copy[index]];
  return copy;
}

// The buttons that move or remove one entry of a list.
function EntryActions({ list, index, name, onChange }) {
  return html`<div class="actions">
    <button class="button quiet" type="button" disabled=${index === 0}
      aria-label=${`Move ${name} up`} onClick=${() => onChange(move(list, index, -1))}>Move up</button>
    <button class="button quiet" type="button" disabled=${index === list.length - 1}
      aria-label=${`Move ${name} down`} onClick=${() => onChange(move(list, index, 1))}>Move down</button>
    <button class="button quiet" type="button"
      aria-label=${`Remove ${name}`} onClick=${() => onChange(list.filter((_, i) => i !== index))}>Remove</button>
  </div>`;
}

// ---- The channel ----

function ChannelFields({ model, update, bad }) {
  const field = (name) => (value) => update((m) => set(m, name, value));
  return html`<div class="panel form">
    <h2>Album</h2>
    <${TextField} id="ch-title" label="Title" value=${model.title} onChange=${field('title')} />
    <${TextField} id="ch-author" label="Artist" hint="The itunes:author of the feed." value=${model.author} onChange=${field('author')} />
    <${TextArea} id="ch-description" label="Description" value=${model.description} onChange=${field('description')} />
    <${TextField} id="ch-image" label="Cover image address" type="url" address="image" bad=${bad} value=${model.image} onChange=${field('image')} />
    <${TextField} id="ch-link" label="Website" type="url" address="link" bad=${bad} value=${model.link} onChange=${field('link')} />
    <${TextField} id="ch-language" label="Language" hint="For example en or de-AT." value=${model.language} onChange=${field('language')} />
  </div>`;
}

function Persons({ prefix, persons, onChange, bad }) {
  const blank = { name: '', role: null, group: null, img: null, href: null };
  return html`<div class="form">
    <h3>People</h3>
    ${persons.length === 0 && html`<p class="hint">No people yet.</p>`}
    ${persons.map((p, i) => {
      const change = (name) => (value) => onChange(setAt(persons, i, set(p, name, value)));
      const id = `${prefix}-person-${p._cid}`;
      return html`<fieldset class="entry" key=${p._cid}>
        <legend>${p.name || 'New person'}</legend>
        <${TextField} id=${`${id}-name`} label="Name" required value=${p.name} onChange=${change('name')} />
        <${TextField} id=${`${id}-role`} label="Role" hint="For example vocalist or guitarist. Empty means host." value=${p.role} onChange=${change('role')} />
        <${TextField} id=${`${id}-group`} label="Group" hint="For example music. Empty means cast." value=${p.group} onChange=${change('group')} />
        <${TextField} id=${`${id}-img`} label="Picture address" type="url" address=${`persons/${i}/img`} bad=${bad} value=${p.img} onChange=${change('img')} />
        <${TextField} id=${`${id}-href`} label="Information address" type="url" address=${`persons/${i}/href`} bad=${bad} value=${p.href} onChange=${change('href')} />
        <${EntryActions} list=${persons} index=${i} name=${p.name || 'this person'} onChange=${onChange} />
      </fieldset>`;
    })}
    <div class="actions"><button class="button" type="button" onClick=${() => onChange([...persons, withCids([blank])[0]])}>＋ Add a person</button></div>
  </div>`;
}

function Fundings({ fundings, onChange, bad }) {
  const blank = { label: '', url: null };
  return html`<div class="form">
    <h3>Support links</h3>
    ${fundings.length === 0 && html`<p class="hint">No support links yet.</p>`}
    ${fundings.map((f, i) => {
      const change = (name) => (value) => onChange(setAt(fundings, i, set(f, name, value)));
      const id = `funding-${f._cid}`;
      return html`<fieldset class="entry" key=${f._cid}>
        <legend>${f.label || 'New support link'}</legend>
        <${TextField} id=${`${id}-label`} label="Text" required value=${f.label} onChange=${change('label')} />
        <${TextField} id=${`${id}-url`} label="Address" type="url" address=${`fundings/${i}/url`} bad=${bad} value=${f.url} onChange=${change('url')} />
        <${EntryActions} list=${fundings} index=${i} name=${f.label || 'this support link'} onChange=${onChange} />
      </fieldset>`;
    })}
    <div class="actions"><button class="button" type="button" onClick=${() => onChange([...fundings, withCids([blank])[0]])}>＋ Add a support link</button></div>
  </div>`;
}

function LicenseFields({ license, onChange, bad }) {
  const value = license || { identifier: '', url: null };
  // A license with no identifier and no address is no license.
  const change = (name) => (text) => {
    const next = set(value, name, text);
    onChange(next.identifier === '' && next.url === null ? null : next);
  };
  return html`<div class="form">
    <h3>License</h3>
    <${TextField} id="license-id" label="License" hint="For example cc-by-4.0." required value=${value.identifier} onChange=${change('identifier')} />
    <${TextField} id="license-url" label="License text address" type="url" address="license/url" bad=${bad} value=${value.url} onChange=${change('url')} />
  </div>`;
}

// ---- The tracks ----

function Enclosure({ id, path, enclosure, onChange, bad }) {
  const value = enclosure || { url: null, length: null, media_type: null };
  // An enclosure with no field is no enclosure.
  const change = (name) => (text) => {
    const next = set(value, name, text);
    onChange(next.url === null && next.length === null && next.media_type === null ? null : next);
  };
  return html`<${TextField} id=${`${id}-url`} label="Audio file address" type="url" address=${`${path}enclosure/url`} bad=${bad} value=${value.url} onChange=${change('url')} />
    <${TextField} id=${`${id}-length`} label="Audio file size in bytes" inputmode="numeric" value=${value.length} onChange=${change('length')} />
    <${TextField} id=${`${id}-type`} label="Audio file type" hint="For example audio/mpeg." value=${value.media_type} onChange=${change('media_type')} />`;
}

function Track({ items, index, onChange, bad }) {
  const item = items[index];
  const id = `track-${item._cid}`;
  const path = `items/${index}/`;
  const hasBad = bad.some((b) => b.field.startsWith(path));
  const change = (name) => (value) => onChange(setAt(items, index, set(item, name, value)));
  const name = item.title || `track ${index + 1}`;
  // One name for each track, so that the browser keeps only one track open.
  return html`<details class="track" name="tracks" key=${item._cid} open=${hasBad || undefined}>
    <summary><span class="number">${index + 1}</span><span class="title">${item.title || 'Untitled track'}</span>
      ${hasBad && html`<span class="field-error">An address is not correct</span>`}</summary>
    <div class="form">
      <${TextField} id=${`${id}-title`} label="Title" value=${item.title} onChange=${change('title')} />
      <${TextArea} id=${`${id}-description`} label="Description" value=${item.description} onChange=${change('description')} />
      <${Enclosure} id=${id} path=${path} bad=${bad} enclosure=${item.enclosure} onChange=${change('enclosure')} />
      <${TextField} id=${`${id}-duration`} label="Duration" hint="Seconds, or HH:MM:SS." value=${item.duration} onChange=${change('duration')} />
      <${TextField} id=${`${id}-date`} label="Publication date"
        hint="The release date of this track, for example Thu, 22 Oct 2026 12:00:00 GMT. Some apps sort the tracks by this date."
        value=${item.pub_date} onChange=${change('pub_date')} />
      <${TextField} id=${`${id}-image`} label="Track image address" type="url" address=${`${path}image`} bad=${bad} value=${item.image} onChange=${change('image')} />
      <dl class="facts">
        <dt>Track number</dt><dd>${item.episode || '—'}</dd>
        <dt>Track GUID</dt><dd class="mono">${item.guid || 'Studio gives one when it saves'}</dd>
      </dl>
      <${EntryActions} list=${items} index=${index} name=${name} onChange=${onChange} />
    </div>
  </details>`;
}

// A new track needs the address, the size and the type of its audio file
// (ADR 0023 items 5 to 7). Studio asks the host of the file for the size and
// the type. Studio does not host files (ADR 0009).
function AddTrack({ onAdd }) {
  const [title, setTitle] = useState('');
  const [url, setUrl] = useState('');
  const [length, setLength] = useState('');
  const [type, setType] = useState('');
  const [facts, setFacts] = useState({ state: 'none', for: '' });
  const readFacts = async () => {
    const address = url.trim();
    if (!address || facts.for === address) return;
    setFacts({ state: 'reading', for: address });
    try {
      const found = await api('/media/facts', { method: 'POST', body: { url: address } });
      if (found.length !== null) setLength(String(found.length));
      if (found.media_type) setType(found.media_type);
      setFacts({ state: found.length !== null && found.media_type ? 'found' : 'partial', for: address });
    } catch (e) {
      setFacts({ state: e.code === 'invalid_address' ? 'invalid' : 'failed', for: address });
    }
  };
  const ready = url.trim() && /^[0-9]+$/.test(length.trim()) && type.trim() && facts.state !== 'invalid';
  const submit = (event) => {
    event.preventDefault();
    if (!ready) return;
    onAdd({
      title: optional(title.trim()),
      description: null,
      guid: null,
      enclosure: { url: url.trim(), length: length.trim(), media_type: type.trim() },
      pub_date: null,
      duration: null,
      episode: null,
      image: null,
      images: [],
      persons: [],
      fundings: [],
      transcripts: [],
      txts: [],
      license: null,
      values: [],
    });
    setTitle('');
    setUrl('');
    setLength('');
    setType('');
    setFacts({ state: 'none', for: '' });
  };
  const FACTS = {
    reading: 'Studio asks the host for the size and the type of the file…',
    found: 'Studio found the size and the type on the host. You can change them.',
    partial: 'The host did not give all the facts. Type the missing ones.',
    failed: 'Studio could not read the facts from the host. Type the size and the type.',
  };
  return html`<form class="form add-track" onSubmit=${submit}>
    <h3>Add a track</h3>
    <${Field} id="new-track-title" label="Title" value=${title} onInput=${(e) => setTitle(e.target.value)} />
    <${Field} id="new-track-url" label="Audio file address" type="url" required
      hint="Studio does not keep audio files. Give the address where your host keeps the file."
      error=${facts.state === 'invalid' && facts.for === url.trim() ? BAD_ADDRESS : null}
      value=${url} onInput=${(e) => setUrl(e.target.value)} onBlur=${readFacts} />
    ${FACTS[facts.state] && html`<p class="hint" role="status">${FACTS[facts.state]}</p>`}
    <${Field} id="new-track-length" label="Audio file size in bytes" inputmode="numeric" required
      value=${length} onInput=${(e) => setLength(e.target.value)} />
    <${Field} id="new-track-type" label="Audio file type" hint="For example audio/mpeg." required
      value=${type} onInput=${(e) => setType(e.target.value)} />
    <div class="actions"><button class="button" type="submit" disabled=${!ready}>＋ Add the track</button></div>
  </form>`;
}

function Tracks({ model, update, bad }) {
  const onChange = (items) => update((m) => set(m, 'items', items));
  return html`<div class="panel form">
    <h2>Tracks</h2>
    <p class="hint">When you move, add or remove a track, Studio sets the track number (podcast:episode) of each track from this sequence. Studio does not change the publication dates.</p>
    ${model.items.length === 0 && html`<p class="hint">This album has no tracks yet.</p>`}
    <div class="tracks">${model.items.map((_, i) => html`<${Track} key=${model.items[i]._cid} items=${model.items} index=${i} onChange=${onChange} bad=${bad} />`)}</div>
    <${AddTrack} onAdd=${(item) => onChange([...model.items, withCids([item])[0]])} />
  </div>`;
}

// ---- Changes, write and snapshots ----

const STATUS = {
  loading: 'Loading the draft…',
  waiting: 'Changes not saved yet…',
  saving: 'Saving the draft…',
  saved: 'The draft is saved. The hosted file does not change until you write it.',
  failed: 'Studio could not save the draft.',
};

function DiffView({ unified }) {
  const lines = unified.split('\n');
  return html`<pre class="diff" tabindex="0" aria-label="Changes between the hosted file and the draft">${lines.map((line) => {
    const kind = line.startsWith('+') && !line.startsWith('+++') ? 'add'
      : line.startsWith('-') && !line.startsWith('---') ? 'remove' : '';
    return html`<span class=${kind}>${line}\n</span>`;
  })}</pre>`;
}

function Changes({ feedId, draft, onWritten }) {
  const [diff, setDiff] = useState(null);
  const { busy, error, run } = useAction();
  // A diff is for one draft. After a user edit, the diff is not current.
  useEffect(() => { setDiff(null); }, [draft.edits]);
  const examine = () => run(async () => {
    if (!(await draft.flush())) return;
    setDiff(await api(`/feeds/${feedId}/diff`));
  });
  const write = () => run(async () => {
    let feed;
    try {
      feed = await api(`/feeds/${feedId}/write`, { method: 'POST', body: { diff_id: diff.id } });
    } catch (e) {
      // After a conflict, the diff is not against the hosted file. Hide it
      // and its write button.
      if (e.code === 'conflict') setDiff(null);
      throw e;
    }
    setDiff(null);
    await draft.load();
    onWritten(feed);
  });
  const discard = () => {
    if (!confirm('Discard the draft? Each change that you did not write goes out. The hosted file does not change.')) return;
    run(async () => {
      await api(`/feeds/${feedId}/draft`, { method: 'DELETE' });
      setDiff(null);
      await draft.load();
    });
  };
  const startAgain = () => {
    if (!confirm('Start again from the hosted file? Each change in the draft goes out.')) return;
    run(async () => {
      await api(`/feeds/${feedId}/draft`, { method: 'DELETE' });
      setDiff(null);
      await draft.load();
    });
  };
  const conflict = error && error.code === 'conflict';
  const blocked = draft.bad.length > 0;
  const stale = error && error.code === 'diff_not_approved';
  return html`<div class="panel form">
    <h2>Changes</h2>
    <p class="hint" role="status">${STATUS[draft.status]}</p>
    ${draft.error && draft.error.code === 'stale_revision'
      ? html`<div class="notice error" role="alert"><strong>The draft changed in a different window.</strong>
          Load the draft again to continue. The changes in this window since the last save go out.
          <div class="actions"><button class="button" type="button" onClick=${draft.load}>Load the draft again</button></div></div>`
      : html`<${ErrorNotice} error=${draft.error} />`}
    ${conflict
      ? html`<div class="notice error" role="alert"><strong>The hosted file changed after the draft started.</strong>
          Studio did not write. A different tool or a different person changed the file.
          <div class="actions"><button class="button" type="button" disabled=${busy} onClick=${startAgain}>Start again from the hosted file</button></div></div>`
      : stale
        ? html`<div class="notice error" role="alert"><strong>The draft changed after you examined it.</strong>Examine the changes again.</div>`
        : html`<${ErrorNotice} error=${error} />`}
    ${blocked && html`<div class="notice error" role="alert">
      <strong>Studio cannot write while an address is not correct.</strong>
      Correct the fields with red text. Each address starts with https:// or http://.
      <ul class="address-list">${draft.bad.map((b) => html`<li><span class="mono">${b.field}</span>: <span class="mono">${b.value}</span></li>`)}</ul>
    </div>`}
    ${diff && (diff.changed
      ? html`<${DiffView} unified=${diff.unified} />`
      : html`<p>The draft is the same as the hosted file.</p>`)}
    <div class="actions">
      <button class="button" type="button" disabled=${busy || draft.status === 'loading'} onClick=${examine}>Examine the changes</button>
      ${diff && diff.changed && !blocked && html`<button class="button primary" type="button" disabled=${busy} onClick=${write}>
        ${busy ? 'Writing…' : 'Write to the host'}</button>`}
      <button class="button quiet" type="button" disabled=${busy || !draft.changed} onClick=${discard}>Discard the draft</button>
    </div>
  </div>`;
}

function Snapshots({ feedId, draft, version }) {
  const [list, setList] = useState(null);
  const { busy, error, run } = useAction();
  useEffect(() => {
    api(`/feeds/${feedId}/snapshots`).then(setList, () => setList([]));
  }, [feedId, version]);
  const restore = (snapshot) => {
    if (draft.changed && !confirm('Replace the draft with this earlier version? Each change in the draft goes out.')) return;
    run(async () => {
      await draft.flush();
      draft.show(await api(`/feeds/${feedId}/snapshots/${snapshot.id}/draft`, { method: 'POST' }));
    });
  };
  return html`<div class="panel form">
    <h2>Earlier versions</h2>
    <p class="hint">Before each write, Studio keeps the file that the write replaces. To go back, put an earlier version in the draft, examine the changes and write them.</p>
    <${ErrorNotice} error=${error} />
    ${list && list.length === 0 && html`<p class="hint">No earlier versions yet.</p>`}
    ${list && list.length > 0 && html`<ul class="snapshots">${list.map((s) => html`<li key=${s.id}>
      <span><span class="when">${new Date(s.taken_at * 1000).toLocaleString()}</span>
        <span class="mono sub">${s.version.slice(0, 19)}</span></span>
      <button class="button" type="button" disabled=${busy} onClick=${() => restore(s)}>Put in the draft</button>
    </li>`)}</ul>`}
  </div>`;
}

// ---- The editor ----

export function AlbumEditor({ feed, onWritten }) {
  const draft = useDraft(feed.id);
  const { model, update } = draft;
  if (!model) {
    return html`<section class="section"><p class="hint" role="status">${STATUS[draft.status]}</p>
      <${ErrorNotice} error=${draft.error} /></section>`;
  }
  return html`<section class="section editor">
    <${ChannelFields} model=${model} update=${update} bad=${draft.bad} />
    <div class="panel form">
      <h2>Credits and links</h2>
      <${Persons} prefix="ch" persons=${model.persons} bad=${draft.bad} onChange=${(persons) => update((m) => set(m, 'persons', persons))} />
      <${Fundings} fundings=${model.fundings} bad=${draft.bad} onChange=${(fundings) => update((m) => set(m, 'fundings', fundings))} />
      <${LicenseFields} license=${model.license} bad=${draft.bad} onChange=${(license) => update((m) => set(m, 'license', license))} />
    </div>
    <${Tracks} model=${model} update=${update} bad=${draft.bad} />
    <${Changes} feedId=${feed.id} draft=${draft} onWritten=${onWritten} />
    <${Snapshots} feedId=${feed.id} draft=${draft} version=${feed.last_version} />
  </section>`;
}

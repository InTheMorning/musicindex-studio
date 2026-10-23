// The album editor (task 011, ADR 0022) and its toolbar and sheets (task
// 014b, ADR 0029). The UI edits the typed model of the draft. Each list
// entry carries its key, and the draft saves automatically.
import { html, useState, useEffect, useRef } from './vendor/htm-preact-standalone.module.js';
import { api, Field, ErrorNotice, useAction } from './app.js';
import { AlbumPayments, TrackPayments } from './values.js';
import { withCids, STATUS } from './draft.js';

// `values.js` imports `withCids` from this file, so that it does not need
// to know that the draft moved to `draft.js`.
export { withCids } from './draft.js';

// ---- Fields ----

// The text of an optional field. An empty field is `null`, and the writer then
// removes the element (ADR 0022 item 6).
const optional = (text) => (text === '' ? null : text);

const BAD_ADDRESS = 'This is not a full web address. Start it with https://.';

// A text field. An address field gives its model path in `address`. The
// field then shows red text when the core finds the address not correct
// (ADR 0023 item 4).
export function TextField({ id, label, value, onChange, required = false, address, bad = [], ...rest }) {
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
export const set = (object, name, value) => ({ ...object, [name]: value });

// Change one entry of a list.
export const setAt = (list, index, entry) => list.map((e, i) => (i === index ? entry : e));

function move(list, index, by) {
  const target = index + by;
  if (target < 0 || target >= list.length) return list;
  const copy = [...list];
  [copy[index], copy[target]] = [copy[target], copy[index]];
  return copy;
}

// The buttons that move or remove one entry of a list.
export function EntryActions({ list, index, name, onChange }) {
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

// ---- Summary lines (ADR 0028) ----
// Each entry of a list is one summary line that opens. Entries with the same
// group name open one at a time. An entry that the user added starts open.

export const startOpen = new Set();

// A ref that opens a new entry one time, after the browser inserted it. An
// entry that is open at insertion closes when its group has an open entry,
// so the open must come after the insertion. The browser then closes the
// other entry of the group.
export const openOnce = (cid) => (element) => {
  if (element && startOpen.has(cid)) {
    startOpen.delete(cid);
    element.open = true;
  }
};

export function EntryRow({ group, cid, summary, children }) {
  return html`<details class="entry-row" name=${group} key=${cid} ref=${openOnce(cid)}>
    <summary>${summary}</summary>
    <div class="form">${children}</div>
  </details>`;
}

// Add an entry that starts open.
export function addOpen(list, blank) {
  const added = withCids([blank])[0];
  startOpen.add(added._cid);
  return [...list, added];
}

function Persons({ prefix, persons, onChange, bad }) {
  const blank = { name: '', role: null, group: null, img: null, href: null };
  return html`<div class="form">
    <h3>People</h3>
    ${persons.length === 0 && html`<p class="hint">No people yet.</p>`}
    ${persons.map((p, i) => {
      const change = (name) => (value) => onChange(setAt(persons, i, set(p, name, value)));
      const id = `${prefix}-person-${p._cid}`;
      const summary = html`<span class="title">${p.name || 'New person'}</span>
        <span class="sub">${p.role || 'host'} · ${p.group || 'cast'}</span>`;
      return html`<${EntryRow} key=${p._cid} group=${`${prefix}-persons`} cid=${p._cid} summary=${summary}>
        <${TextField} id=${`${id}-name`} label="Name" required value=${p.name} onChange=${change('name')} />
        <div class="grid2">
          <${TextField} id=${`${id}-role`} label="Role" hint="For example vocalist. Empty means host." value=${p.role} onChange=${change('role')} />
          <${TextField} id=${`${id}-group`} label="Group" hint="For example music. Empty means cast." value=${p.group} onChange=${change('group')} />
        </div>
        <${TextField} id=${`${id}-img`} label="Picture address" type="url" address=${`persons/${i}/img`} bad=${bad} value=${p.img} onChange=${change('img')} />
        <${TextField} id=${`${id}-href`} label="Information address" type="url" address=${`persons/${i}/href`} bad=${bad} value=${p.href} onChange=${change('href')} />
        <${EntryActions} list=${persons} index=${i} name=${p.name || 'this person'} onChange=${onChange} />
      <//>`;
    })}
    <div class="actions"><button class="button" type="button" onClick=${() => onChange(addOpen(persons, blank))}>＋ Add a person</button></div>
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
      const summary = html`<span class="title">${f.label || 'New support link'}</span>
        <span class="sub mono">${f.url || 'No address'}</span>`;
      return html`<${EntryRow} key=${f._cid} group="fundings" cid=${f._cid} summary=${summary}>
        <${TextField} id=${`${id}-label`} label="Text" required value=${f.label} onChange=${change('label')} />
        <${TextField} id=${`${id}-url`} label="Address" type="url" address=${`fundings/${i}/url`} bad=${bad} value=${f.url} onChange=${change('url')} />
        <${EntryActions} list=${fundings} index=${i} name=${f.label || 'this support link'} onChange=${onChange} />
      <//>`;
    })}
    <div class="actions"><button class="button" type="button" onClick=${() => onChange(addOpen(fundings, blank))}>＋ Add a support link</button></div>
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

function Track({ items, index, onChange, bad, albumBlocks }) {
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
      <${TrackPayments} id=${id} item=${item} albumBlocks=${albumBlocks} onChange=${change('values')} />
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
    <div class="tracks">${model.items.map((_, i) => html`<${Track} key=${model.items[i]._cid} items=${model.items} index=${i} onChange=${onChange} bad=${bad} albumBlocks=${model.values} />`)}</div>
    <${AddTrack} onAdd=${(item) => onChange([...model.items, withCids([item])[0]])} />
  </div>`;
}

// ---- The diff, the write and the snapshots, in sheets (ADR 0029) ----

function DiffView({ unified }) {
  const lines = unified.split('\n');
  return html`<pre class="diff" tabindex="0" aria-label="Changes between the hosted file and the draft">${lines.map((line) => {
    const kind = line.startsWith('+') && !line.startsWith('+++') ? 'add'
      : line.startsWith('-') && !line.startsWith('---') ? 'remove' : '';
    return html`<span class=${kind}>${line}\n</span>`;
  })}</pre>`;
}

// A sheet opens with showModal() when `open` turns true, and closes with
// close() when `open` turns false. The close event covers the Escape key,
// and each caller passes its handler as `onClose` (ADR 0029 item 5 and 6).
function useSheet(open) {
  const ref = useRef(null);
  useEffect(() => {
    const dialog = ref.current;
    if (!dialog) return;
    if (open) dialog.showModal();
    else if (dialog.open) dialog.close();
  }, [open]);
  return ref;
}

// The review sheet: the draft status, the diff and the write (ADR 0029 item
// 2). It is the same content as the former Changes panel.
function ReviewSheet({ feedId, draft, open, onWritten, onClose }) {
  const ref = useSheet(open);
  const [diff, setDiff] = useState(null);
  const { busy, error, run } = useAction();
  // A diff is for one draft. After a user edit, the diff is not current.
  useEffect(() => { setDiff(null); }, [draft.edits]);
  const examine = () => run(async () => {
    if (!(await draft.flush())) return;
    setDiff(await api(`/feeds/${feedId}/diff`));
  });
  // The sheet examines the draft each time that it opens.
  useEffect(() => { if (open) examine(); }, [open]);
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
    ref.current.close();
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
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="review-title" onClose=${onClose}>
    <header>
      <h2 id="review-title">Review changes</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body">
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
      </div>
    </div>
    <footer>
      <button class="button quiet" type="button" disabled=${busy || !draft.changed} onClick=${discard}>Discard the draft</button>
      ${diff && diff.changed && !blocked && html`<button class="button primary" type="button" disabled=${busy} onClick=${write}>
        ${busy ? 'Writing…' : 'Write to the host'}</button>`}
    </footer>
  </dialog>`;
}

// The versions sheet: the snapshots that each write kept (ADR 0029 item 3).
// It is the same content as the former Snapshots panel, with no details
// element: the sheet itself is the one open view.
function VersionsSheet({ feedId, draft, open, onClose }) {
  const ref = useSheet(open);
  const [list, setList] = useState(null);
  const { busy, error, run } = useAction();
  useEffect(() => {
    if (!open) return;
    setList(null);
    api(`/feeds/${feedId}/snapshots`).then(setList, () => setList([]));
  }, [feedId, open]);
  const restore = (snapshot) => {
    if (draft.changed && !confirm('Replace the draft with this earlier version? Each change in the draft goes out.')) return;
    run(async () => {
      await draft.flush();
      draft.show(await api(`/feeds/${feedId}/snapshots/${snapshot.id}/draft`, { method: 'POST' }));
      ref.current.close();
    });
  };
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="versions-title" onClose=${onClose}>
    <header>
      <h2 id="versions-title">Earlier versions</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body">
      <p class="hint">Before each write, Studio keeps the file that the write replaces. To go back, put an earlier version in the draft, examine the changes and write them.</p>
      <${ErrorNotice} error=${error} />
      ${list && list.length === 0 && html`<p class="hint">No earlier versions yet.</p>`}
      ${list && list.length > 0 && html`<div class="group"><ul class="snapshots">${list.map((s) => html`<li key=${s.id}>
        <span><span class="when">${new Date(s.taken_at * 1000).toLocaleString()}</span>
          <span class="mono sub">${s.version.slice(0, 19)}</span></span>
        <button class="button" type="button" disabled=${busy} onClick=${() => restore(s)}>Put in the draft</button>
      </li>`)}</ul></div>`}
    </div>
  </dialog>`;
}

// The short status of the toolbar (ADR 0029 item 1). It differs from the
// long status of the review sheet.
function toolbarStatus(draft) {
  if (draft.status === 'idle' || draft.status === 'loading') return { text: 'Loading…' };
  if (draft.status === 'waiting' || draft.status === 'saving') return { text: 'Saving…' };
  if (draft.status === 'failed') return { text: 'Not saved', error: true };
  return draft.changed ? { text: 'Saved, not written' } : { text: 'No changes' };
}

// The toolbar of the editor: the draft status, and the buttons that open
// the review sheet and the versions sheet (ADR 0029 items 1 to 3).
export function DraftToolbar({ feedId, draft, onWritten }) {
  // null, 'review' or 'versions': the sheet that is open now.
  const [open, setOpen] = useState(null);
  const status = toolbarStatus(draft);
  return html`<div class="draft-toolbar">
    <span class=${`draft-status${status.error ? ' error' : ''}`} role="status">${status.text}</span>
    <button class="button" type="button" onClick=${() => setOpen('versions')}>Versions</button>
    <button class="button primary" type="button" onClick=${() => setOpen('review')}>Review</button>
    <${ReviewSheet} feedId=${feedId} draft=${draft} open=${open === 'review'}
      onWritten=${onWritten} onClose=${() => setOpen(null)} />
    <${VersionsSheet} feedId=${feedId} draft=${draft} open=${open === 'versions'} onClose=${() => setOpen(null)} />
  </div>`;
}

// ---- The editor ----

// The summary line of the credits: the people, the support links and the
// license.
function creditSummary(model) {
  const people = model.persons.length;
  const names = model.persons.slice(0, 3).map((p) => p.name).filter(Boolean).join(', ');
  const links = model.fundings.length;
  return [
    people === 0 ? 'No people' : `${people} ${people === 1 ? 'person' : 'people'}: ${names}${people > 3 ? ', …' : ''}`,
    `${links} support link${links === 1 ? '' : 's'}`,
    model.license ? model.license.identifier || 'license' : 'No license',
  ].join(' · ');
}

// The album editor shows the typed model of the draft. The shell loads the
// draft and shows its toolbar; this component does not call useDraft, and
// has no Changes panel and no Earlier versions panel (ADR 0029 item 4).
export function AlbumEditor({ draft }) {
  const { model, update } = draft;
  if (!model) {
    return html`<section class="section"><p class="hint" role="status">${STATUS[draft.status]}</p>
      <${ErrorNotice} error=${draft.error} /></section>`;
  }
  return html`<section class="section editor">
    <${ChannelFields} model=${model} update=${update} bad=${draft.bad} />
    <details class="panel disclosure">
      <summary><h2>Credits and links</h2><span class="sub">${creditSummary(model)}</span></summary>
      <div class="form">
      <${Persons} prefix="ch" persons=${model.persons} bad=${draft.bad} onChange=${(persons) => update((m) => set(m, 'persons', persons))} />
      <${Fundings} fundings=${model.fundings} bad=${draft.bad} onChange=${(fundings) => update((m) => set(m, 'fundings', fundings))} />
      <${LicenseFields} license=${model.license} bad=${draft.bad} onChange=${(license) => update((m) => set(m, 'license', license))} />
      </div>
    </details>
    <${AlbumPayments} model=${model} update=${update} />
    <${Tracks} model=${model} update=${update} bad=${draft.bad} />
  </section>`;
}

// The album editor (task 011, ADR 0022), its pages (task 014c, ADR 0029
// items 5 to 10) and its toolbar and sheets (task 014b, ADR 0029). The UI
// edits the typed model of the draft. Each list entry carries its key, and
// the draft saves automatically.
import { html, useState, useEffect, useRef } from './vendor/htm-preact-standalone.module.js';
import { api, Field, ErrorNotice, useAction, addressSource } from './app.js';
import { AlbumPayments, TrackPayments, ShareStack, recipientSummary } from './values.js';
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
    <div class="field-body"><textarea id=${id} value=${value ?? ''} onInput=${(e) => onChange(optional(e.target.value))}></textarea></div>
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

// The key of a track: its GUID, or its client ID when it has no GUID yet
// (task 014c). The sidebar tree and the album page use this key to open the
// track page of one track.
export function trackKey(item) {
  return item.guid ? item.guid : `cid-${item._cid}`;
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

// People, support links and the license are each a section of the credits
// page (ADR 0029 item 5). The page gives the heading, so each function here
// returns its group content only, with no heading of its own.

// Persons and Fundings return their rows with no wrapper div, so that each
// entry and the add row sit as direct rows of the group, each with the
// group's own inset separator (review of task 014c, defect 3).
function Persons({ prefix, persons, onChange, bad }) {
  const blank = { name: '', role: null, group: null, img: null, href: null };
  return html`${persons.length === 0 && html`<p class="hint">No people yet.</p>`}
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
    <button class="link-row add" type="button" onClick=${() => onChange(addOpen(persons, blank))}>
      <span class="num">＋</span><span class="title">Add a person</span>
    </button>`;
}

function Fundings({ fundings, onChange, bad }) {
  const blank = { label: '', url: null };
  return html`${fundings.length === 0 && html`<p class="hint">No support links yet.</p>`}
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
    <button class="link-row add" type="button" onClick=${() => onChange(addOpen(fundings, blank))}>
      <span class="num">＋</span><span class="title">Add a support link</span>
    </button>`;
}

// The license fields, returned with no wrapper, so that they sit as two
// rows of the License group directly (task 014c step 8).
function LicenseFields({ license, onChange, bad }) {
  const value = license || { identifier: '', url: null };
  // A license with no identifier and no address is no license.
  const change = (name) => (text) => {
    const next = set(value, name, text);
    onChange(next.identifier === '' && next.url === null ? null : next);
  };
  return html`<${TextField} id="license-id" label="License" hint="For example cc-by-4.0." required value=${value.identifier} onChange=${change('identifier')} />
    <${TextField} id="license-url" label="License text address" type="url" address="license/url" bad=${bad} value=${value.url} onChange=${change('url')} />`;
}

// ---- The audio file ----

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

// A new track needs the address, the size and the type of its audio file
// (ADR 0023 items 6 and 7). Studio asks the host of the file for the size
// and the type. Studio does not host files (ADR 0009).
function AddTrackSheet({ open, onClose, onAdd }) {
  const ref = useSheet(open);
  const [title, setTitle] = useState('');
  const [url, setUrl] = useState('');
  const [length, setLength] = useState('');
  const [type, setType] = useState('');
  const [facts, setFacts] = useState({ state: 'none', for: '' });
  // Clear the sheet each time that it opens.
  useEffect(() => {
    if (!open) return;
    setTitle('');
    setUrl('');
    setLength('');
    setType('');
    setFacts({ state: 'none', for: '' });
  }, [open]);
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
  };
  const FACTS = {
    reading: 'Studio asks the host for the size and the type of the file…',
    found: 'Studio found the size and the type on the host. You can change them.',
    partial: 'The host did not give all the facts. Type the missing ones.',
    failed: 'Studio could not read the facts from the host. Type the size and the type.',
  };
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="add-track-title" onClose=${onClose}>
    <header>
      <h2 id="add-track-title">Add a track</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <form class="body" id="add-track-form" onSubmit=${submit}>
      <div class="group">
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
      </div>
    </form>
    <footer>
      <button class="button quiet" type="button" onClick=${() => ref.current.close()}>Cancel</button>
      <button class="button primary" type="submit" form="add-track-form" disabled=${!ready}>＋ Add the track</button>
    </footer>
  </dialog>`;
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

// ---- The editor pages (ADR 0029 items 5 to 10) ----

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

// The Feed facts row of the album page (task 014c step 5): the facts that
// the former Facts section showed, and the Remove from collection button.
function FeedFacts({ feed, onRemoved }) {
  const { busy, error, run } = useAction();
  const remove = () => {
    const name = feed.title || feed.path || 'this feed';
    if (!confirm(`Remove “${name}” from your collection? The feed file on the host does not change.`)) return;
    run(async () => {
      await api(`/feeds/${feed.id}`, { method: 'DELETE' });
      onRemoved();
    });
  };
  return html`<details class="disclosure">
    <summary class="link-row"><span class="label">Feed facts</span><span class="summary">GUID, file and version</span></summary>
    <div class="form">
      <dl class="facts">
        <dt>Address source</dt><dd>${feed.public_url ? addressSource(feed.address_source) : '—'}</dd>
        <dt>Feed GUID</dt><dd class="mono">${feed.podcast_guid || '—'}</dd>
        <dt>File</dt><dd class="mono">${feed.path || 'Read-only. Connect the folder that holds this file in Hosts.'}</dd>
        <dt>Version</dt><dd class="mono">${feed.last_version || 'Not read yet'}</dd>
      </dl>
      <${ErrorNotice} error=${error} />
      <div class="actions"><button class="button quiet" type="button" disabled=${busy} onClick=${remove}>Remove from collection</button></div>
    </div>
  </details>`;
}

// The album page: the album fields, the album details and the tracks
// (task 014c step 3).
function AlbumPage({ feed, model, update, bad, go, onAdding, onRemoved }) {
  const field = (name) => (value) => update((m) => set(m, name, value));
  const openPage = (page) => go({ name: 'feed', id: feed.id, ...page });
  // No wrapper: each section is a direct row of .page, with the gap of
  // .page between sections (review of task 014c, defect 2).
  return html`<section class="page-section"><h2>Album</h2>
      <div class="group">
        <${TextField} id="ch-title" label="Title" value=${model.title} onChange=${field('title')} />
        <${TextField} id="ch-author" label="Artist" hint="The itunes:author of the feed." value=${model.author} onChange=${field('author')} />
        <${TextArea} id="ch-description" label="Description" value=${model.description} onChange=${field('description')} />
        <${TextField} id="ch-image" label="Cover image address" type="url" address="image" bad=${bad} value=${model.image} onChange=${field('image')} />
        <${TextField} id="ch-link" label="Website" type="url" address="link" bad=${bad} value=${model.link} onChange=${field('link')} />
        <${TextField} id="ch-language" label="Language" hint="For example en or de-AT." value=${model.language} onChange=${field('language')} />
      </div>
    </section>

    <section class="page-section"><h2>Album details</h2>
      <div class="group">
        <button class="link-row" type="button" onClick=${() => openPage({ page: 'credits' })}>
          <span class="label">Credits and links</span><span class="summary">${creditSummary(model)}</span>
          <span class="go" aria-hidden="true">›</span>
        </button>
        <button class="link-row" type="button" onClick=${() => openPage({ page: 'recipients' })}>
          <span class="label">Recipients</span><span class="summary">${recipientSummary(model.values)}</span>
          ${model.values.length > 0 && html`<${ShareStack} recipients=${model.values[0].recipients} />`}
          <span class="go" aria-hidden="true">›</span>
        </button>
        <${FeedFacts} feed=${feed} onRemoved=${onRemoved} />
      </div>
    </section>

    <section class="page-section"><h2>Tracks</h2>
      <div class="group">
        ${model.items.map((item, i) => html`<button key=${trackKey(item)} class="link-row" type="button"
          onClick=${() => openPage({ page: 'track', track: trackKey(item) })}>
          <span class="num">${i + 1}</span>
          <span class="title">${item.title || 'Untitled track'}</span>
          ${item.duration && html`<span class="dur">${item.duration}</span>`}
          <span class="go" aria-hidden="true">›</span>
        </button>`)}
        <button class="link-row add" type="button" onClick=${() => onAdding(true)}>
          <span class="num">＋</span><span class="title">Add a track</span>
        </button>
      </div>
      <p class="note">Studio sets the track numbers (podcast:episode) from this sequence. It does not change the publication dates.</p>
    </section>`;
}

// The credits and links page: people, support links and the license, each
// as a section (task 014c step 3). No wrapper: each section is a direct row
// of .page (review of task 014c, defect 2).
function CreditsPage({ model, update, bad }) {
  return html`<section class="page-section"><h2>People</h2>
      <div class="group"><${Persons} prefix="ch" persons=${model.persons} bad=${bad}
        onChange=${(persons) => update((m) => set(m, 'persons', persons))} /></div>
    </section>
    <section class="page-section"><h2>Support links</h2>
      <div class="group"><${Fundings} fundings=${model.fundings} bad=${bad}
        onChange=${(fundings) => update((m) => set(m, 'fundings', fundings))} /></div>
    </section>
    <section class="page-section"><h2>License</h2>
      <div class="group"><${LicenseFields} license=${model.license} bad=${bad}
        onChange=${(license) => update((m) => set(m, 'license', license))} /></div>
    </section>`;
}

// The recipients page: the album recipients, with no details around them
// (task 014c step 3, ADR 0029 item 9).
function RecipientsPage({ model, update }) {
  return html`<${AlbumPayments} model=${model} update=${update} />`;
}

// The track page: the track before it and the track after it, then its
// fields, its audio file, its recipients and its other facts (task 014c
// step 3). The current editor has no credits for a track.
function TrackPage({ feed, model, update, bad, trackIndex, go }) {
  const items = model.items;
  const item = items[trackIndex];
  const id = `track-${item._cid}`;
  const path = `items/${trackIndex}/`;
  const change = (name) => (value) => update((m) => set(m, 'items', setAt(m.items, trackIndex, set(item, name, value))));
  const changeItems = (nextItems) => update((m) => set(m, 'items', nextItems));
  const openTrack = (i) => go({ name: 'feed', id: feed.id, page: 'track', track: trackKey(items[i]) });
  const removeTrack = () => {
    changeItems(items.filter((_, i) => i !== trackIndex));
    go({ name: 'feed', id: feed.id });
  };
  // No wrapper: the head and each section are direct rows of .page, with
  // the gap of .page between them (review of task 014c, defect 2).
  return html`<header class="head">
      <div class="text"><span class="eyebrow">Track ${trackIndex + 1} of ${items.length}</span>
        <h1>${item.title || 'Untitled track'}</h1></div>
      <div class="pager">
        <button class="button icon-button" type="button" aria-label="The track before this one"
          disabled=${trackIndex === 0} onClick=${() => openTrack(trackIndex - 1)}>‹</button>
        <button class="button icon-button" type="button" aria-label="The track after this one"
          disabled=${trackIndex === items.length - 1} onClick=${() => openTrack(trackIndex + 1)}>›</button>
      </div>
    </header>

    <section class="page-section"><h2>Track</h2>
      <div class="group">
        <${TextField} id=${`${id}-title`} label="Title" value=${item.title} onChange=${change('title')} />
        <${TextArea} id=${`${id}-description`} label="Description" value=${item.description} onChange=${change('description')} />
        <${TextField} id=${`${id}-date`} label="Publication date"
          hint="For example Thu, 22 Oct 2026 12:00:00 GMT. Some apps sort the tracks by this date."
          value=${item.pub_date} onChange=${change('pub_date')} />
        <${TextField} id=${`${id}-duration`} label="Duration" hint="Seconds, or HH:MM:SS." value=${item.duration} onChange=${change('duration')} />
      </div>
    </section>

    <section class="page-section"><h2>Audio file</h2>
      <div class="group"><${Enclosure} id=${id} path=${path} bad=${bad} enclosure=${item.enclosure} onChange=${change('enclosure')} /></div>
    </section>

    <${TrackPayments} id=${id} item=${item} albumBlocks=${model.values} onChange=${change('values')} />

    <section class="page-section"><h2>More</h2>
      <div class="group">
        <details class="disclosure">
          <summary class="link-row"><span class="label">Track image</span>
            <span class="summary">${item.image ? 'Has an image' : 'Uses the album cover'}</span></summary>
          <div class="form"><${TextField} id=${`${id}-image`} label="Track image address" type="url" address=${`${path}image`} bad=${bad} value=${item.image} onChange=${change('image')} /></div>
        </details>
        <details class="disclosure">
          <summary class="link-row"><span class="label">Identity</span>
            <span class="summary">Number ${item.episode || '—'}</span></summary>
          <div class="form">
            <dl class="facts">
              <dt>Track number</dt><dd>${item.episode || '—'}</dd>
              <dt>Track GUID</dt><dd class="mono">${item.guid || 'Studio gives one when it saves'}</dd>
            </dl>
          </div>
        </details>
      </div>
    </section>

    <div class="actions">
      <button class="button quiet" type="button" disabled=${trackIndex === 0} onClick=${() => changeItems(move(items, trackIndex, -1))}>Move up</button>
      <button class="button quiet" type="button" disabled=${trackIndex === items.length - 1} onClick=${() => changeItems(move(items, trackIndex, 1))}>Move down</button>
      <button class="button quiet" type="button" onClick=${removeTrack}>Remove the track</button>
    </div>`;
}

// The album editor shows one page of the typed model of the draft: the
// album, the credits, the recipients or one track (task 014c step 3). The
// shell loads the draft and shows its toolbar. This component does not call
// useDraft (ADR 0029 item 4).
export function AlbumEditor({ feed, draft, view, go, adding, onAdding, onRemoved }) {
  const { model, update } = draft;
  if (!model) {
    return html`<div class="page"><p class="hint" role="status">${STATUS[draft.status]}</p>
      <${ErrorNotice} error=${draft.error} /></div>`;
  }
  const page = view.page || 'album';
  const trackIndex = page === 'track' ? model.items.findIndex((item) => trackKey(item) === view.track) : -1;
  let body;
  // When the track key of the view is not in the model, for example after a
  // write that gave each item a new client ID, the editor shows the album
  // page (task 014c, Architecture).
  if (page === 'track' && trackIndex !== -1) {
    body = html`<${TrackPage} feed=${feed} model=${model} update=${update} bad=${draft.bad} trackIndex=${trackIndex} go=${go} />`;
  } else if (page === 'credits') {
    body = html`<${CreditsPage} model=${model} update=${update} bad=${draft.bad} />`;
  } else if (page === 'recipients') {
    body = html`<${RecipientsPage} model=${model} update=${update} />`;
  } else {
    body = html`<${AlbumPage} feed=${feed} model=${model} update=${update} bad=${draft.bad} go=${go} onAdding=${onAdding} onRemoved=${onRemoved} />`;
  }
  const onAdd = (item) => {
    const [added] = withCids([item]);
    update((m) => set(m, 'items', [...m.items, added]));
    onAdding(false);
    go({ name: 'feed', id: feed.id, page: 'track', track: trackKey(added) });
  };
  return html`<div class="page">
    ${body}
    <${AddTrackSheet} open=${adding} onClose=${() => onAdding(false)} onAdd=${onAdd} />
  </div>`;
}

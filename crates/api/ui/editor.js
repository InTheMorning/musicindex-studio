// The album editor (task 011, ADR 0022), its pages (task 014c, ADR 0029
// items 5 to 10) and its toolbar and sheets (task 014b, ADR 0029). The UI
// edits the typed model of the draft. Each list entry carries its key, and
// the draft saves automatically.
import { html, useState, useEffect, useRef } from './vendor/htm-preact-standalone.module.js';
import { api, Field, ErrorNotice, useAction, addressSource, ImageThumb } from './app.js';
import { AlbumPayments, TrackPayments, ShareStack, recipientSummary } from './values.js';
import { withCids, STATUS } from './draft.js';
import { trackKey, isTrack, isPublisher } from './shell.js';
import { announceLines, announceStatus, isWaiting } from './announce.js';
import { eventsToSign, signAll, signErrorText, signSummary, signProgressLabel, SignError } from './signing.js';

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

// The buttons that move or remove one entry of a list (ADR 0037 item 1).
// "Move up" and "Move down" change no value, so each is a normal button.
// "Remove" takes an entry out of the list, so it is destructive.
export function EntryActions({ list, index, name, onChange }) {
  return html`<div class="actions">
    <button class="button" type="button" disabled=${index === 0}
      aria-label=${`Move ${name} up`} onClick=${() => onChange(move(list, index, -1))}>Move up</button>
    <button class="button" type="button" disabled=${index === list.length - 1}
      aria-label=${`Move ${name} down`} onClick=${() => onChange(move(list, index, 1))}>Move down</button>
    <button class="button destructive" type="button"
      aria-label=${`Remove ${name}`} onClick=${() => onChange(list.filter((_, i) => i !== index))}>Remove</button>
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

// The alternate enclosures of a track, each as one summary line with a
// Remove button: its title or its type, and its size. There is no form to
// edit an alternate enclosure (ADR 0033 item 8).
function AlternateEnclosures({ list, onChange }) {
  return list.map((enc, i) => {
    const label = enc.title || enc.media_type || 'Alternate file';
    return html`<div class="alt-enclosure" key=${enc._cid}>
      <span class="title">${label}</span>
      <span class="sub">${enc.length ? fileSize(Number(enc.length)) : '—'}</span>
      <button class="button destructive" type="button"
        aria-label=${`Remove ${label}`} onClick=${() => onChange(list.filter((_, j) => j !== i))}>Remove</button>
    </div>`;
  });
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
      alternate_enclosures: [],
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
      <button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>
      <button class="button primary" type="submit" form="add-track-form" disabled=${!ready}>＋ Add the track</button>
    </footer>
  </dialog>`;
}

// ---- The replacement of the audio of a track (ADR 0040) ----

// The track with the facts of its new audio file. Only the enclosure, the
// alternative enclosures with an audio type, and the duration change. The
// GUID, the title and the track number stay (ADR 0040 items 4 and 5).
function withNewAudio(item, facts) {
  const isAudio = (enc) => (enc.media_type || '').startsWith('audio/');
  return {
    ...item,
    enclosure: { url: facts.url, length: String(facts.size), media_type: facts.media_type },
    alternate_enclosures: [
      ...item.alternate_enclosures.filter((enc) => !isAudio(enc)),
      ...withCids(alternateEnclosures(facts)),
    ],
    duration: facts.duration || item.duration,
  };
}

// Each address of the audio of a track: its enclosure and the sources of
// its alternative enclosures with an audio type.
function audioAddresses(item) {
  const found = item.enclosure && item.enclosure.url ? [item.enclosure.url] : [];
  for (const enc of item.alternate_enclosures) {
    if (!(enc.media_type || '').startsWith('audio/')) continue;
    for (const source of enc.sources) if (source.uri) found.push(source.uri);
  }
  return found;
}

// "Replace the Audio File" (ADR 0040 items 1 to 6). The new file goes into
// the release folder, with the hash name when a different file has its
// name. The draft then gets the new addresses, and each copy from before
// that the draft does not use gets a deletion.
function ReplaceAudio({ feedId, item, update, draft, onOpsChanged }) {
  const { busy, error, run } = useAction();
  const [done, setDone] = useState(false);
  const inputId = `replace-${item._cid}`;
  const replace = (file) => run(async () => {
    setDone(false);
    if (!(await draft.flush())) return;
    const facts = await api(`/feeds/${feedId}/files?name=${encodeURIComponent(file.name)}&replace=true`, {
      method: 'POST',
      body: file,
    });
    if (facts.kind !== 'audio') {
      if (!facts.on_host) {
        api(`/feeds/${feedId}/files?file=${facts.library_file_id}`, { method: 'DELETE' }).catch(() => {});
      }
      throw Object.assign(new Error('Select an audio file.'), { code: 'not_audio' });
    }
    const before = audioAddresses(item);
    update((m) => set(m, 'items', m.items.map((i) => (i._cid === item._cid ? withNewAudio(i, facts) : i))));
    if (!(await draft.flush())) return;
    await api(`/feeds/${feedId}/operations`, { method: 'POST', body: { op: 'delete_unused', urls: before } });
    onOpsChanged();
    setDone(true);
  });
  return html`<div class="import-pick">
    <input id=${inputId} class="visually-hidden" type="file" disabled=${busy}
      aria-describedby=${`${inputId}-hint`}
      accept="audio/*,.wav,.flac,.m4a,.ogg,.opus"
      onChange=${(e) => { const [file] = e.target.files; e.target.value = ''; if (file) replace(file); }} />
    <label for=${inputId} class="button action">${busy ? 'Replacing…' : 'Replace the Audio File'}</label>
    <p class="hint" id=${`${inputId}-hint`} role="status">${done
      ? 'Studio replaced the audio file. The review lists the deletion of the file from before.'
      : 'The track keeps its GUID and its other values. Studio deletes the file from before on the host when you write.'}</p>
    ${error && html`<p class="field-error" role="alert">${importErrorText(error)}</p>`}
  </div>`;
}

// ---- The import sheet (task 018d, ADR 0026, ADR 0032) ----
// The user selects music files and image files. The sheet sends each file to
// the draft, one file at a time, and shows its state and its facts. The add
// puts the audio files in the album, in the sequence of their track numbers.

// The text of the WAV notice and the FLAC notice (ADR 0033 item 9). The
// conversion is not an error, so each notice does not use the --error color.
const WAV_NOTICE = 'Studio converts this WAV file to MP3 and FLAC. Each app plays the MP3 file. An app that offers a choice can also play the lossless FLAC file.';
const FLAC_NOTICE = 'Studio makes an MP3 file from this FLAC file. Each app plays the MP3 file. An app that offers a choice can also play the lossless FLAC file.';

// Is this a WAV file, by its name? Studio shows the WAV notice for this file
// until the user removes the row, also after the file sends (ADR 0033 item
// 9, task architecture).
function isWavFile(file) {
  return /\.wav$/i.test(file.name);
}

// Is this a FLAC file, by its name? Studio shows the FLAC notice for this
// file until the user removes the row, also after the file sends (ADR 0033
// item 9, task architecture).
function isFlacFile(file) {
  return /\.flac$/i.test(file.name);
}

// Is this a GIF image, by its name? Studio accepts it, and tells the user
// that Apple Podcasts asks for JPEG or PNG art (ADR 0032 item 1).
function isGifFile(file) {
  return /\.gif$/i.test(file.name);
}

const GIF_NOTICE = 'Apple Podcasts asks for JPEG or PNG art. Other apps show this GIF image.';

const AUDIO_EXTENSIONS = /\.(mp3|m4a|flac|ogg|opus|wav)$/i;

// The kind of a file before Studio reads it: a guess from its name, for the
// list of audio files of an image row. Each row shows its real kind, from
// the server, once it is ready.
function guessKind(file) {
  if (AUDIO_EXTENSIONS.test(file.name)) return 'audio';
  return 'image';
}

// The size of a file, in a short and plain unit.
function fileSize(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

// The `alternate_enclosures` of a new track, from the facts of its sent
// file. A track with no lossless file gets an empty list. A lossless track
// gets the MP3 file and the FLAC file, in that sequence, with each other
// attribute null (ADR 0033 items 5 and 6).
function alternateEnclosures(facts) {
  if (!facts.lossless) return [];
  return [
    {
      media_type: 'audio/mpeg',
      length: String(facts.size),
      bitrate: null,
      title: 'Standard (MP3)',
      rel: null,
      default: 'true',
      sources: [{ uri: facts.url, content_type: null }],
    },
    {
      media_type: 'audio/flac',
      length: String(facts.lossless.size),
      bitrate: null,
      title: 'Maximum fidelity (lossless)',
      rel: null,
      default: null,
      sources: [{ uri: facts.lossless.url, content_type: null }],
    },
  ];
}

const IMPORT_ERRORS = {
  unsupported_media: 'Studio does not read this kind of file.',
  file_pending: 'A track or an image of this draft uses a file with this name. Remove it from the draft first.',
  file_exists: 'A file with this name exists on the host already.',
  no_public_address: 'This host has no public address. Give it one in Hosts.',
  invalid_name: 'This is not a file name that Studio can use.',
};

function importErrorText(error) {
  return IMPORT_ERRORS[error.code] || error.message || 'Studio could not import this file.';
}

function importStateText(row, kind) {
  if (row.state === 'waiting') return 'Waiting to send…';
  if (row.state === 'sending') return 'Sending…';
  if (row.state === 'failed') return importErrorText(row.error);
  return kind === 'audio' ? 'Track ready' : 'Image ready';
}

// One row of the import sheet: the name of the file, its state, the WAV
// notice when it is a WAV file, and its facts when it is ready. An image row
// also has the list of its use (task architecture).
// The short format name of a media type, for the badge of a file row.
function formatLabel(mediaType) {
  const known = { 'audio/mpeg': 'MP3', 'audio/flac': 'FLAC', 'audio/mp4': 'M4A', 'audio/ogg': 'OGG', 'image/jpeg': 'JPEG' };
  return known[mediaType] || (mediaType || '').split('/').pop().toUpperCase();
}

// The files that Studio keeps for one import row. Each file is one row with
// its format, its name, its role and its size. A lossless import has two
// files (ADR 0033 item 1). A file that the import uses from the host gets
// a note of this fact (ADR 0034 item 5).
function FileRows({ facts, role }) {
  const files = [{ file: facts, role }];
  if (facts.lossless) files.push({ file: facts.lossless, role: 'Maximum fidelity (lossless)' });
  return html`<ul class="import-files" aria-label="Files">
    ${files.map(({ file, role: text }) => html`<li class="import-file">
      <span class="badge">${formatLabel(file.media_type)}</span>
      <span class="name">
        <span class="file" title=${file.file_name}>${file.file_name}</span>
        <span class="role">${text}${file.on_host ? ' · On the host already' : ''}</span>
      </span>
      <span class="size">${fileSize(file.size)}</span>
    </li>`)}
  </ul>`;
}

function ImportRow({ row, audioOptions, onRemove, onImageUseChange }) {
  const kind = row.facts ? row.facts.kind : guessKind(row.file);
  const failed = row.state === 'failed';
  return html`<div class="import-row">
    <div class="import-head">
      <span class="title">${row.file.name}</span>
      <span class=${`state${failed ? ' failed' : ''}`} role=${failed ? 'alert' : undefined}>
        ${importStateText(row, kind)}
      </span>
      <button class="button destructive" type="button" disabled=${row.state === 'sending'}
        aria-label=${`Remove ${row.file.name}`} onClick=${() => onRemove(row)}>Remove</button>
    </div>
    ${row.isWav && html`<div class="notice"><strong>${row.file.name}</strong>${WAV_NOTICE}</div>`}
    ${row.isFlac && html`<div class="notice"><strong>${row.file.name}</strong>${FLAC_NOTICE}</div>`}
    ${row.isGif && html`<div class="notice"><strong>${row.file.name}</strong>${GIF_NOTICE}</div>`}
    ${row.state === 'ready' && kind === 'audio' && html`<dl class="facts">
      <dt>Title</dt><dd>${row.facts.title || '—'}</dd>
      <dt>Track number</dt><dd>${row.facts.track_number ?? '—'}</dd>
      <dt>Duration</dt><dd>${row.facts.duration || '—'}</dd>
    </dl>
    <${FileRows} facts=${row.facts} role=${row.facts.lossless ? 'Standard: plays in each app' : 'Track file'} />`}
    ${row.state === 'ready' && kind === 'image' && html`<${FileRows} facts=${row.facts} role="Image" />
    <div class="import-dest">
      <label for=${`import-dest-${row.id}`}>Use for</label>
      <select id=${`import-dest-${row.id}`} value=${row.imageUse}
        onChange=${(e) => onImageUseChange(row, e.target.value)}>
        <option value="album">Album art</option>
        ${audioOptions.map((a) => html`<option key=${a.id} value=${`audio-${a.id}`}>${a.file.name}</option>`)}
        <option value="not-used">Not used</option>
      </select>
    </div>`}
  </div>`;
}

// The next client-only ID of a row of the import sheet. It is not a client
// ID of the draft model (ADR 0022): the sheet sends each row to the server
// before it joins the model.
let nextImportId = 1;

// The import sheet: a file field, one row for each selected file, and the
// add (task architecture). `feedId` sends each file to the draft of this
// feed. `hasAlbumImage` tells the sheet whether the album has an image
// already, for the default use of the first image. The browser file field
// is hidden. Its label is a button of the app, and the field keeps the
// focus, so the button works from the keyboard.
function ImportSheet({ feedId, open, hasAlbumImage, onClose, onAdd }) {
  const ref = useSheet(open);
  const [rows, setRows] = useState([]);
  // A copy of `rows`, for the parts of this component that read the current
  // rows between two turns of the event loop, where the state of a hook is
  // not yet the state of its last `set` call.
  const rowsRef = useRef([]);
  const setAllRows = (updater) => {
    const next = typeof updater === 'function' ? updater(rowsRef.current) : updater;
    rowsRef.current = next;
    setRows(next);
  };
  // The sheet keeps the files that it sent, so that it can remove them from
  // the host when the user cancels (ADR 0026 item 8). `true` after the add,
  // so that the close of the sheet does not remove the files that are now in
  // the album.
  const addedRef = useRef(false);
  const sendingRef = useRef(false);

  useEffect(() => {
    if (!open) return;
    rowsRef.current = [];
    setRows([]);
    addedRef.current = false;
  }, [open]);

  const updateRow = (id, patch) => setAllRows((rs) => rs.map((r) => (r.id === id ? { ...r, ...patch } : r)));

  // Send each waiting row, one at a time, in the sequence of selection
  // (task architecture). A file that a later call to `addFiles` adds joins
  // the queue, because each turn of this loop reads `rowsRef` again.
  const sendQueue = async () => {
    if (sendingRef.current) return;
    sendingRef.current = true;
    try {
      for (;;) {
        const index = rowsRef.current.findIndex((r) => r.state === 'waiting');
        if (index === -1) break;
        const row = rowsRef.current[index];
        updateRow(row.id, { state: 'sending' });
        try {
          const facts = await api(`/feeds/${feedId}/files?name=${encodeURIComponent(row.file.name)}`, {
            method: 'POST',
            body: row.file,
          });
          // The first image becomes the album art when the album has no
          // image of its own yet (task architecture). "First" means no
          // earlier-selected row is a ready image already.
          const imageUse = facts.kind !== 'image' ? null
            : !hasAlbumImage && !rowsRef.current.some((r) => r.id < row.id && r.facts && r.facts.kind === 'image')
              ? 'album' : 'not-used';
          updateRow(row.id, { state: 'ready', facts, imageUse, error: null });
        } catch (e) {
          updateRow(row.id, { state: 'failed', error: e });
        }
      }
    } finally {
      sendingRef.current = false;
    }
  };

  const addFiles = (fileList) => {
    const added = Array.from(fileList).map((file) => ({
      id: nextImportId++,
      file,
      isWav: isWavFile(file),
      isFlac: isFlacFile(file),
      isGif: isGifFile(file),
      state: 'waiting',
      error: null,
      facts: null,
      imageUse: null,
    }));
    setAllRows((rs) => [...rs, ...added]);
    sendQueue();
  };

  // Remove the MP3 file of a sent row from the host, and the FLAC file too
  // when the row has one (ADR 0033 items 1 and 3, task architecture). A
  // file that the import used from the host already is not a pending
  // file, so this skips its removal (ADR 0034 item 5).
  const removeFiles = (facts) => {
    if (!facts.on_host) {
      api(`/feeds/${feedId}/files?file=${facts.library_file_id}`, { method: 'DELETE' }).catch(() => {});
    }
    if (facts.lossless && !facts.lossless.on_host) {
      api(`/feeds/${feedId}/files?file=${facts.lossless.library_file_id}`, { method: 'DELETE' }).catch(() => {});
    }
  };

  // Remove a row. When the sheet already sent the file, remove it from the
  // host too ("Cancel", and a row that the user removes", task
  // architecture). An image row that used this row as its track loses that
  // use.
  const removeRow = (row) => {
    setAllRows((rs) => rs.filter((r) => r.id !== row.id)
      .map((r) => (r.imageUse === `audio-${row.id}` ? { ...r, imageUse: 'not-used' } : r)));
    if (row.state === 'ready' && row.facts) removeFiles(row.facts);
  };

  const changeImageUse = (row, value) => updateRow(row.id, { imageUse: value });

  // The close of the sheet, from Cancel, the ✕ button or the Escape key.
  // Remove each file that the sheet sent, unless the user added the files
  // to the album (task architecture).
  const close = () => {
    if (!addedRef.current) {
      for (const row of rows) {
        if (row.state === 'ready' && row.facts) removeFiles(row.facts);
      }
    }
    onClose();
  };

  const readyAudio = rows.filter((r) => r.state === 'ready' && r.facts.kind === 'audio');
  const audioOptions = rows.filter(
    (r) => r.state !== 'failed' && (r.facts ? r.facts.kind === 'audio' : guessKind(r.file) === 'audio'),
  );
  const stillGoing = rows.some((r) => r.state === 'waiting' || r.state === 'sending');
  const albumArt = rows.find((r) => r.state === 'ready' && r.facts.kind === 'image' && r.imageUse === 'album');
  // An import of the album art alone is also an add.
  const canAdd = !stillGoing && (readyAudio.length > 0 || Boolean(albumArt));
  const addLabel = readyAudio.length > 0 || !albumArt
    ? `＋ Add ${readyAudio.length} track${readyAudio.length === 1 ? '' : 's'}`
    : '＋ Add the album art';

  // The add: the audio files in the sequence of their track numbers, then
  // the files with no number in the sequence of selection (ADR 0026 item 3).
  // Studio sets the final track numbers when it saves the draft (ADR 0024).
  const add = () => {
    const numbered = readyAudio.filter((r) => r.facts.track_number !== null)
      .sort((a, b) => a.facts.track_number - b.facts.track_number);
    const unnumbered = readyAudio.filter((r) => r.facts.track_number === null);
    const imageFor = (rowId) => {
      const found = rows.find(
        (r) => r.state === 'ready' && r.facts.kind === 'image' && r.imageUse === `audio-${rowId}`,
      );
      return found ? found.facts.url : null;
    };
    const albumImage = albumArt;
    const tracks = [...numbered, ...unnumbered].map((row) => ({
      title: row.facts.title || null,
      description: null,
      guid: null,
      enclosure: { url: row.facts.url, length: String(row.facts.size), media_type: row.facts.media_type },
      alternate_enclosures: alternateEnclosures(row.facts),
      pub_date: null,
      duration: row.facts.duration || null,
      episode: null,
      image: imageFor(row.id),
      images: [],
      persons: [],
      fundings: [],
      transcripts: [],
      txts: [],
      license: null,
      values: [],
    }));
    // `onAdd` sets `open` to false. That closes the dialog (`useSheet`), and
    // its close event then runs `close` above, which does not remove these
    // files because `addedRef` is now true.
    addedRef.current = true;
    onAdd(tracks, albumImage ? albumImage.facts.url : null);
  };

  return html`<dialog ref=${ref} class="sheet" aria-labelledby="import-title" onClose=${close}>
    <header>
      <h2 id="import-title">Import files</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body">
      <div class="import-pick">
        <p class="hint" id="import-input-hint">
          Studio reads the facts of each file, and makes one track for each audio file.
        </p>
        <input id="import-input" class="visually-hidden" type="file" multiple
          aria-describedby="import-input-hint"
          accept="audio/*,image/*,.wav,.flac,.m4a,.ogg,.opus"
          onChange=${(e) => { addFiles(e.target.files); e.target.value = ''; }} />
        <label for="import-input" class="button action">＋ Choose music and image files</label>
      </div>
      ${rows.length === 0 && html`<p class="hint">Select the files to import.</p>`}
      ${rows.length > 0 && html`<div class="group">
        ${rows.map((row) => html`<${ImportRow} key=${row.id} row=${row} audioOptions=${audioOptions}
          onRemove=${removeRow} onImageUseChange=${changeImageUse} />`)}
      </div>`}
    </div>
    <footer>
      <button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>
      <button class="button primary" type="button" disabled=${!canAdd} onClick=${add}>
        ${addLabel}
      </button>
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

// ---- Host operations, in the diff sheet and the Files page (task 021g,
// ADR 0036 items 10 to 14) ----

// The name of a connected host, by its ID. A host that the list does not
// have yet, for example right after a page load, falls back to a plain
// word.
function connectorName(connectors, id) {
  const found = connectors.find((c) => c.id === id);
  return found ? found.name : 'Host';
}

function operationKindLabel(kind) {
  switch (kind) {
    case 'upload': return 'Upload';
    case 'copy': return 'Copy';
    case 'delete': return 'Delete';
    case 'confirm': return 'Confirm';
    default: return kind;
  }
}

// The second line of an operation row: what the operation means for the
// feed (task 022b, review of milestone 7a).
const OPERATION_NOTE = {
  upload: 'The feed uses this file.',
  copy: 'A spare copy. The feed does not use it.',
  delete: 'Studio deletes it after the feed is written.',
  confirm: 'Studio makes sure that the host gives this file.',
};

// One row of a host operation of the diff: its kind, its host, its path
// and its size (ADR 0036 item 14). An upload has no draft operation of its
// own, so it has no Remove button. "Remove" takes the operation out of the
// draft, so it is destructive (ADR 0037 item 1).
function HostOperationRow({ op, connectors, onRemove, busy }) {
  return html`<div class="host-op-row">
    <span class="kind">${operationKindLabel(op.kind)}</span>
    <span class="text">
      <span class="title">${connectorName(connectors, op.connector_id)}</span>
      <span class="sub mono">${op.path}</span>
      <span class="sub">${OPERATION_NOTE[op.kind] || ''}</span>
      ${op.waiting && html`<span class="sub waiting">Waiting for the download. Studio writes it after the download.</span>`}
    </span>
    <span class="size">${fileSize(op.size)}</span>
    ${op.operation_id !== null && html`<button class="button" type="button" disabled=${busy}
      onClick=${() => onRemove(op.operation_id)}>Undo</button>`}
  </div>`;
}

// The second line of the feed move row: what the address from before gets
// (ADR 0038 items 16 to 18).
function moveNote(method) {
  return method === 'redirect'
    ? 'The address from before gets a redirect to the new address.'
    : 'The address from before gets a pointer file with the new address.';
}

// The move of the feed XML to a different host (ADR 0038 item 16). It
// is the first row of the operations. "Undo" puts the feed XML back on the
// host of the feed.
function FeedMoveRow({ move, connectors, onUndo, busy }) {
  return html`<div class="host-op-row">
    <span class="kind">Move</span>
    <span class="text">
      <span class="title">Move the feed</span>
      ${move.from_url && html`<span class="sub">From <span class="mono">${move.from_url}</span></span>`}
      <span class="sub">To <span class="mono">${move.url}</span></span>
      <span class="sub">${moveNote(move.method)}</span>
      <span class="sub">${connectorName(connectors, move.connector_id)}</span>
    </span>
    <button class="button" type="button" disabled=${busy} onClick=${onUndo}>Undo</button>
  </div>`;
}

// A deletion that Studio refuses because a feed of the library refers to
// the file (ADR 0036 item 13). The row uses the error color, not an
// entity color (ADR 0018).
function BlockedDeleteRow({ blocked, connectors, onRemove, busy }) {
  return html`<div class="host-op-row error" role="alert">
    <span class="kind">Delete</span>
    <span class="text">
      <span class="title">${connectorName(connectors, blocked.connector_id)}</span>
      <span class="sub mono">${blocked.path}</span>
      <span class="sub">${blocked.pointer
        ? 'Studio keeps this file. It tells the apps the new address of the feed.'
        : `Studio cannot delete this file. These feeds refer to it: ${blocked.feed_titles.join(', ')}`}</span>
    </span>
    <button class="button" type="button" disabled=${busy}
      onClick=${() => onRemove(blocked.operation_id)}>Undo</button>
  </div>`;
}

// Each host operation of the diff, above the XML diff (ADR 0036 item 14).
function HostOperations({ diff, connectors, onRemove, onUndoMove, busy }) {
  if (diff.operations.length === 0 && diff.blocked.length === 0 && !diff.feed_move) return null;
  return html`<div class="group host-ops">
    ${diff.feed_move && html`<${FeedMoveRow} move=${diff.feed_move} connectors=${connectors} onUndo=${onUndoMove} busy=${busy} />`}
    ${diff.operations.map((op) => html`<${HostOperationRow}
      key=${`${op.kind}-${op.connector_id}-${op.path}`} op=${op} connectors=${connectors} onRemove=${onRemove} busy=${busy} />`)}
    ${diff.blocked.map((b) => html`<${BlockedDeleteRow}
      key=${`blocked-${b.operation_id}`} blocked=${b} connectors=${connectors} onRemove=${onRemove} busy=${busy} />`)}
  </div>`;
}

// The announcements of a write (ADR 0052 item 11). The lines show when the
// sheet can write, also when the feed text did not change. Each line has the
// form of a host operation row, with no button.
function AnnounceLines({ diff }) {
  const lines = announceLines(diff);
  if (lines.length === 0) return null;
  return html`<div class="group announce-lines">
    ${lines.map((line) => html`<div class="host-op-row" key=${line}>
      <span class="kind">Announce</span>
      <span class="text">${line}</span>
    </div>`)}
  </div>`;
}

// The name of the event that tells the feed page to load the announcements
// again, such as after a write.
const ANNOUNCE_RELOAD = 'studio-announce-reload';

// A sheet opens with showModal() when `open` turns true, and closes with
// close() when `open` turns false. The close event covers the Escape key,
// and each caller passes its handler as `onClose` (ADR 0029 item 5 and 6).
export function useSheet(open) {
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
function ReviewSheet({ feedId, feedConnectorId, draft, open, onWritten, onClose, connectors, onOpsChanged, onOpsCleared }) {
  const ref = useSheet(open);
  const [diff, setDiff] = useState(null);
  const { busy, error, run } = useAction();
  // The progress of the signer while a write signs the planned events.
  const [progress, setProgress] = useState(null);
  // A diff is for one draft. After a user edit, the diff is not current.
  useEffect(() => { setDiff(null); }, [draft.edits]);
  const examine = () => run(async () => {
    if (!(await draft.flush())) return;
    setDiff(await api(`/feeds/${feedId}/diff`));
  });
  // The sheet examines the draft each time that it opens.
  useEffect(() => { if (open) examine(); }, [open]);
  const removeOperation = (operationId) => run(async () => {
    await api(`/feeds/${feedId}/operations/${operationId}`, { method: 'DELETE' });
    onOpsChanged();
    setDiff(await api(`/feeds/${feedId}/diff`));
  });
  // "Undo" of a feed move sets the host of the feed again. The route
  // changes the self link in the draft, so the shell loads the draft again.
  const undoMove = () => run(async () => {
    setDiff(await api(`/feeds/${feedId}/feed-host`, { method: 'PUT', body: { connector_id: feedConnectorId } }));
    onOpsChanged();
  });
  const toSign = diff ? eventsToSign(diff) : [];
  const write = () => run(async () => {
    let feed;
    let signedEvents = [];
    try {
      if (toSign.length > 0) {
        // The signer of the user signs the planned events before the write.
        // With no signer, nothing goes to the host.
        if (!window.nostr) throw new SignError('no_signer', signErrorText('no_signer'));
        setProgress({ done: 0, total: toSign.length });
        signedEvents = await signAll(toSign, window.nostr, (done, total) => setProgress({ done, total }));
      }
      feed = await api(`/feeds/${feedId}/write`, { method: 'POST', body: { diff_id: diff.id, signed_events: signedEvents } });
    } catch (e) {
      // After a conflict, the diff is not against the hosted file. Hide it
      // and its write button.
      if (e.code === 'conflict') setDiff(null);
      // After an expired plan, the new diff has a new plan.
      if (e.code === 'plan_expired') setDiff(await api(`/feeds/${feedId}/diff`));
      throw e;
    } finally {
      setProgress(null);
    }
    setDiff(null);
    await draft.load();
    onOpsCleared();
    onWritten(feed);
    // The feed page loads the new announcements.
    window.dispatchEvent(new Event(ANNOUNCE_RELOAD));
    ref.current.close();
  });
  const discard = () => {
    if (!confirm('Discard the draft? Each change that you did not write goes out. The hosted file does not change.')) return;
    run(async () => {
      await api(`/feeds/${feedId}/draft`, { method: 'DELETE' });
      setDiff(null);
      onOpsCleared();
      await draft.load();
    });
  };
  const startAgain = () => {
    if (!confirm('Start again from the hosted file? Each change in the draft goes out.')) return;
    run(async () => {
      await api(`/feeds/${feedId}/draft`, { method: 'DELETE' });
      setDiff(null);
      onOpsCleared();
      await draft.load();
    });
  };
  // An operation waits for a download of its file (ADR 0038 item 12). The
  // write refuses the draft until each download is complete.
  const waiting = !!diff && diff.operations.some((op) => op.waiting);
  // While an operation waits, the sheet examines the draft again each
  // 1.5 seconds, so that the mark goes out when the download is complete.
  useEffect(() => {
    if (!open || !waiting) return undefined;
    const timer = setInterval(async () => {
      try { setDiff(await api(`/feeds/${feedId}/diff`)); } catch (_) { /* The next pass tries again. */ }
    }, 1500);
    return () => clearInterval(timer);
  }, [open, waiting, feedId]);
  const conflict = error && error.code === 'conflict';
  // Studio cannot write while an address of the draft is not correct. A
  // host operation of `diff` does not block the write on its own: Studio
  // skips a deletion that `diff.blocked` names, and the user sees it here.
  const addressesBlocked = draft.bad.length > 0;
  const stale = error && error.code === 'diff_not_approved';
  const hasWork = diff && (diff.changed || diff.operations.length > 0 || diff.blocked.length > 0 || !!diff.feed_move);
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
          : signErrorText(error && error.code)
            ? html`<div class="notice error" role="alert"><strong>${signErrorText(error.code)}</strong></div>`
            : html`<${ErrorNotice} error=${error} />`}
      ${addressesBlocked && html`<div class="notice error" role="alert">
        <strong>Studio cannot write while an address is not correct.</strong>
        Correct the fields with red text. Each address starts with https:// or http://.
        <ul class="address-list">${draft.bad.map((b) => html`<li><span class="mono">${b.field}</span>: <span class="mono">${b.value}</span></li>`)}</ul>
      </div>`}
      ${diff && html`<${HostOperations} diff=${diff} connectors=${connectors} onRemove=${removeOperation} onUndoMove=${undoMove} busy=${busy} />`}
      ${hasWork && html`<${AnnounceLines} diff=${diff} />`}
      ${diff && (diff.changed
        ? html`<${DiffView} unified=${diff.unified} />`
        : !hasWork && html`<p>The draft is the same as the hosted file.</p>`)}
      ${waiting && html`<p class="hint" role="status">Wait for each download, then write.</p>`}
      ${diff && toSign.length > 0 && html`<p class="hint" role="status">${signSummary(toSign.length)}</p>`}
      <div class="actions">
        <button class="button" type="button" disabled=${busy || draft.status === 'loading'} onClick=${examine}>Examine the changes</button>
      </div>
    </div>
    <footer>
      <button class="button destructive" type="button" disabled=${busy || !draft.changed} onClick=${discard}>Discard the draft</button>
      ${diff && hasWork && !addressesBlocked && html`<button class="button primary" type="button" disabled=${busy || waiting} onClick=${write}>
        ${progress ? signProgressLabel(progress) : busy ? 'Writing…' : 'Write to the host'}</button>`}
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
// long status of the review sheet. `draft.changed` from the server also
// counts the pending host operations (ADR 0036 item 14).
function toolbarStatus(draft) {
  if (draft.status === 'idle' || draft.status === 'loading') return { text: 'Loading…' };
  if (draft.status === 'waiting' || draft.status === 'saving') return { text: 'Saving…' };
  if (draft.status === 'failed') return { text: 'Not saved', error: true };
  return draft.changed ? { text: 'Saved, not written' } : { text: 'No changes' };
}

// The toolbar of the editor: the draft status, and the buttons that open
// the review sheet and the versions sheet (ADR 0029 items 1 to 3).
export function DraftToolbar({ feedId, feedConnectorId, draft, onWritten, connectors, onOpsChanged, onOpsCleared }) {
  // null, 'review' or 'versions': the sheet that is open now.
  const [open, setOpen] = useState(null);
  const status = toolbarStatus(draft);
  return html`<div class="draft-toolbar">
    <span class=${`draft-status${status.error ? ' error' : ''}`} role="status">${status.text}</span>
    <button class="button" type="button" onClick=${() => setOpen('versions')}>Versions</button>
    <button class="button primary" type="button" onClick=${() => setOpen('review')}>Review</button>
    <${ReviewSheet} feedId=${feedId} feedConnectorId=${feedConnectorId} draft=${draft} open=${open === 'review'}
      onWritten=${onWritten} onClose=${() => setOpen(null)} connectors=${connectors}
      onOpsChanged=${onOpsChanged} onOpsCleared=${onOpsCleared} />
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
  // The announcements of the feed, the newest first (ADR 0052 item 20). The
  // list loads when the page opens, after each write, and each 5 seconds
  // while the newest one waits.
  const [announcements, setAnnouncements] = useState(null);
  const loadAnnouncements = () => api(`/feeds/${feed.id}/announcements`).then(setAnnouncements, () => {});
  useEffect(() => {
    setAnnouncements(null);
    loadAnnouncements();
    window.addEventListener(ANNOUNCE_RELOAD, loadAnnouncements);
    return () => window.removeEventListener(ANNOUNCE_RELOAD, loadAnnouncements);
  }, [feed.id]);
  const waiting = isWaiting(announcements);
  useEffect(() => {
    if (!waiting) return undefined;
    const timer = setInterval(loadAnnouncements, 5000);
    return () => clearInterval(timer);
  }, [waiting, feed.id]);
  const announcementText = announcements
    ? announceStatus(announcements, (seconds) => new Date(seconds * 1000).toLocaleString())
    : 'Not loaded yet.';
  const remove = () => {
    const name = feed.title || feed.path || 'this feed';
    if (!confirm(`Remove “${name}” from your collection? The feed file on the host does not change.`)) return;
    run(async () => {
      await api(`/feeds/${feed.id}`, { method: 'DELETE' });
      onRemoved();
    });
  };
  return html`<details class="disclosure">
    <summary class="link-row"><span class="label">Feed facts</span><span class="summary">GUID, file, version and announcement</span></summary>
    <div class="form">
      <dl class="facts">
        <dt>Address source</dt><dd>${feed.public_url ? addressSource(feed.address_source) : '—'}</dd>
        <dt>Feed GUID</dt><dd class="mono">${feed.podcast_guid || '—'}</dd>
        <dt>File</dt><dd class="mono">${feed.path || 'Read-only. Connect the folder that holds this file in Hosts.'}</dd>
        <dt>Version</dt><dd class="mono">${feed.last_version || 'Not read yet'}</dd>
        <dt>Announcement</dt><dd>${announcementText.split('\n').map((line, i) => html`${i > 0 && html`<br />`}${line}`)}</dd>
      </dl>
      <${ErrorNotice} error=${error} />
      <div class="actions"><button class="button destructive" type="button" disabled=${busy} onClick=${remove}>Remove from collection</button></div>
    </div>
  </details>`;
}

// The album page: the album fields, the album details and the tracks
// (task 014c step 3).
// The label of a role in the `rel` of a link (ADR 0041 item 1).
const ROLE_LABEL = { artist: 'Artist', label: 'Label' };

// The second line of a publisher row on the album page: the role, the link
// state and the primary mark (ADR 0041 items 6 and 9).
function publisherLine(entry, match, index) {
  const parts = [ROLE_LABEL[entry.rel] || 'No role'];
  if (!match) parts.push('Not in the collection');
  if (index === 0) parts.push('Primary');
  return parts.join(' · ');
}

function AlbumPage({ feed, feeds, model, update, bad, draft, go, onAdding, onImporting, onRemoved, onListChanged }) {
  const field = (name) => (value) => update((m) => set(m, name, value));
  const openPage = (page) => go({ name: 'feed', id: feed.id, ...page });
  const linkAction = useAction();
  // The feeds whose draft the last link change altered.
  const [changed, setChanged] = useState([]);
  const [chosen, setChosen] = useState('');
  // The publisher feeds of this release, in the sequence of
  // `podcast:publisher` (ADR 0041 item 9). The first one is the primary.
  const publishers = model.publisher ? model.publisher.remote_items : [];
  const rows = publishers.map((entry, index) => ({ index, entry, match: linkedFeed(feeds, entry) }));
  const linkedIds = new Set(rows.filter((row) => row.match).map((row) => row.match.id));
  const candidates = feeds.filter((f) => f.connector_id && mediumOf(f) === 'publisher' && !linkedIds.has(f.id));
  const twoDrafts = (action, call) => changeTwoDrafts(action, { feedId: feed.id, draft, onListChanged, setChanged, call });
  // Swap this publisher feed with the one above or below it. No other feed
  // changes (ADR 0041 item 9).
  const movePublisher = (index, by) => update((m) => set(m, 'publisher',
    set(m.publisher, 'remote_items', move(m.publisher.remote_items, index, by))));
  const removeRow = (row) => {
    if (!row.match) {
      // Studio keeps no link to a feed that is not in the collection. The
      // entry leaves this release only. Without entries, the element goes
      // (ADR 0041 item 13).
      update((m) => {
        const rest = m.publisher.remote_items.filter((_, i) => i !== row.index);
        return set(m, 'publisher', rest.length === 0 ? null : set(m.publisher, 'remote_items', rest));
      });
      return;
    }
    twoDrafts(linkAction, async () => (await api(`/feeds/${row.match.id}/links/${feed.id}`, { method: 'DELETE' })).changed);
  };
  const addPublisher = async () => {
    const done = await twoDrafts(linkAction, async () => (await api(`/feeds/${chosen}/links`, {
      method: 'POST', body: { release: feed.id },
    })).changed);
    if (done) setChosen('');
  };
  // No wrapper: each section is a direct row of .page, with the gap of
  // .page between sections (review of task 014c, defect 2).
  return html`<section class="page-section"><h2>Album</h2>
      <div class="group">
        <${TextField} id="ch-title" label="Title" value=${model.title} onChange=${field('title')} />
        <${TextField} id="ch-author" label="Artist" hint="The itunes:author of the feed." value=${model.author} onChange=${field('author')} />
        <${TextArea} id="ch-description" label="Description" value=${model.description} onChange=${field('description')} />
        <${TextField} id="ch-image" label="Cover image address" type="url" address="image" bad=${bad} value=${model.image} onChange=${field('image')}>
          <${ImageThumb} url=${model.image} className="field-cover" />
        <//>
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
          <span class="label">Value routes</span><span class="summary">${recipientSummary(model.values)}</span>
          ${model.values.length > 0 && html`<${ShareStack} recipients=${model.values[0].recipients} focusable=${false} />`}
          <span class="go" aria-hidden="true">›</span>
        </button>
        <${FeedFacts} feed=${feed} onRemoved=${onRemoved} />
      </div>
    </section>

    <section class="page-section"><h2>Publisher feeds</h2>
      <div class="group">
        <${ChangedNotices} changed=${changed} go=${go} />
        ${rows.length === 0 && html`<p class="hint">No publisher feeds yet.</p>`}
        ${rows.map((row) => html`<div class="release-row" key=${row.entry._cid}>
          <span class="title">${row.entry.title || row.entry.feed_url}</span>
          <span class="sub">${publisherLine(row.entry, row.match, row.index)}</span>
          <div class="actions">
            <button class="button" type="button" disabled=${row.index === 0 || linkAction.busy}
              aria-label=${`Move ${row.entry.title || 'this publisher feed'} up`}
              onClick=${() => movePublisher(row.index, -1)}>Move up</button>
            <button class="button" type="button" disabled=${row.index === rows.length - 1 || linkAction.busy}
              aria-label=${`Move ${row.entry.title || 'this publisher feed'} down`}
              onClick=${() => movePublisher(row.index, 1)}>Move down</button>
            <button class="button destructive" type="button" disabled=${linkAction.busy}
              aria-label=${`Remove ${row.entry.title || 'this publisher feed'}`}
              onClick=${() => removeRow(row)}>Remove</button>
          </div>
        </div>`)}
        ${candidates.length > 0 && html`<div class="field">
          <label for="album-publisher">Publisher feed</label>
          <div class="field-body">
            <select id="album-publisher" value=${chosen} onChange=${(e) => setChosen(e.target.value)}>
              <option value="">Select a publisher feed</option>
              ${candidates.map((f) => html`<option value=${f.id}>${f.title || f.path || 'Untitled feed'}</option>`)}
            </select>
          </div>
        </div>
        <div class="actions"><button class="button action" type="button" disabled=${!chosen || linkAction.busy}
          onClick=${addPublisher}>Add the Publisher Feed</button></div>`}
        ${candidates.length === 0 && html`<p class="hint">Studio has no other publisher feed in the collection to add.</p>`}
        <${ErrorNotice} error=${publisherError(linkAction.error, 'Open the publisher feed and select its role first.')} />
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
        <button class="link-row add" type="button" onClick=${() => onImporting(true)}>
          <span class="num">＋</span><span class="title">Import files</span>
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

// The Value routes page: the album recipients, with no details around them
// (task 014c step 3, ADR 0029 item 9). The page is at most 560 px wide
// (ADR 0039 item 12).
function RecipientsPage({ model, update }) {
  return html`<div class="value-routes"><${AlbumPayments} model=${model} update=${update} /></div>`;
}

// The track page: the track before it and the track after it, then its
// fields, its audio file, its recipients and its other facts (task 014c
// step 3). The current editor has no credits for a track.
function TrackPage({ feed, model, update, bad, trackIndex, go, draft, onOpsChanged }) {
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
      <${ImageThumb} url=${item.image || model.image} glow="var(--c-track)" className="cover" always />
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
      <div class="group">
        <${Enclosure} id=${id} path=${path} bad=${bad} enclosure=${item.enclosure} onChange=${change('enclosure')} />
        <${AlternateEnclosures} list=${item.alternate_enclosures} onChange=${change('alternate_enclosures')} />
      </div>
      <${ReplaceAudio} feedId=${feed.id} item=${item} update=${update} draft=${draft} onOpsChanged=${onOpsChanged} />
    </section>

    <${TrackPayments} id=${id} item=${item} albumBlocks=${model.values} onChange=${change('values')} />

    <section class="page-section"><h2>More</h2>
      <div class="group">
        <details class="disclosure">
          <summary class="link-row"><span class="label">Track image</span>
            <span class="summary">${item.image ? 'Has an image' : 'Uses the album cover'}</span></summary>
          <div class="form"><${TextField} id=${`${id}-image`} label="Track image address" type="url" address=${`${path}image`} bad=${bad} value=${item.image} onChange=${change('image')}>
            <${ImageThumb} url=${item.image} glow="var(--c-track)" className="field-cover" />
          <//></div>
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
      <button class="button" type="button" disabled=${trackIndex === 0} onClick=${() => changeItems(move(items, trackIndex, -1))}>Move up</button>
      <button class="button" type="button" disabled=${trackIndex === items.length - 1} onClick=${() => changeItems(move(items, trackIndex, 1))}>Move down</button>
      <button class="button destructive" type="button" onClick=${removeTrack}>Remove the track</button>
    </div>`;
}

// ---- The publisher feed page (task 025c, ADR 0041) ----

// The medium of a feed in lower case, or an empty text.
function mediumOf(feed) {
  return (feed.medium || '').toLowerCase();
}

// The role that the music items give (ADR 0041 item 4). Studio takes the
// role when each music item gives the same `rel`. Otherwise it gives none.
function inferredRole(items) {
  const rels = items.filter((entry) => entry.medium === 'music').map((entry) => entry.rel);
  if (rels.length === 0) return null;
  const [first] = rels;
  return rels.every((rel) => rel === first) && (first === 'artist' || first === 'label') ? first : null;
}

// The feed of the list that a release entry names, by its feed GUID or its
// public address (ADR 0041, the releases of a publisher feed).
function linkedFeed(feeds, entry) {
  return feeds.find((f) => (f.podcast_guid && f.podcast_guid === entry.feed_guid)
    || (f.public_url && f.public_url === entry.feed_url)) || null;
}

// The text of an error of a publisher change. A missing role has its own
// text, and each page gives that text (ADR 0041 item 4).
function publisherError(error, roleText) {
  if (!error || error.code !== 'role_required') return error;
  return Object.assign(new Error(roleText), { code: error.code });
}

// The steps of a change of two drafts (task 025c, ADR 0041 item 8). `call`
// makes the change and returns the `changed` list of the route. The
// notices name each feed of that list, but not the open feed `feedId`.
// The helper returns true when the change worked.
async function changeTwoDrafts(action, { feedId, draft, onListChanged, setChanged, call }) {
  const done = await action.run(async () => {
    setChanged([]);
    if (!(await draft.flush())) return false;
    const named = (await call()).filter((c) => c.id !== feedId);
    await draft.load();
    await onListChanged();
    setChanged(named);
    return true;
  });
  return done === true;
}

// One notice for each feed whose draft a change altered (ADR 0041 item 8).
// Each notice has a button that opens its feed.
function ChangedNotices({ changed, go }) {
  return changed.map((c) => html`<div class="notice" role="status" key=${c.id}>
      <span>Studio also changed the draft of ${c.title || 'Untitled feed'}. Open it to examine and write it.</span>
      <div class="actions"><button class="button" type="button"
        onClick=${() => go({ name: 'feed', id: c.id })}>Open ${c.title || 'Untitled feed'}</button></div>
    </div>`);
}

const PUBLISHER_ROLE_TEXT = 'Select the role of this publisher feed first.';

// The publisher page: the fields, the role and the releases (ADR 0041 item
// 3). Each link changes the draft of this feed and the draft of the release,
// so the page loads both drafts and names each release that changed.
function PublisherPage({ feed, feeds, model, update, bad, draft, go, onListChanged }) {
  const field = (name) => (value) => update((m) => set(m, name, value));
  const roleAction = useAction();
  const linkAction = useAction();
  const imageAction = useAction();
  // The feeds whose draft the last link or role change altered.
  const [changed, setChanged] = useState([]);
  const [chosen, setChosen] = useState('');
  const items = model.remote_items;
  // The releases in the sequence of the model (ADR 0041 item 10).
  const rows = items
    .map((entry, index) => ({ index, entry, match: linkedFeed(feeds, entry) }))
    .filter((row) => row.entry.medium === 'music');
  const linkedIds = new Set(rows.filter((row) => row.match).map((row) => row.match.id));
  const candidates = feeds.filter((f) => f.connector_id && mediumOf(f) === 'music' && !linkedIds.has(f.id));
  const inferred = inferredRole(items);
  const shownRole = feed.publisher_role || inferred || '';

  // The steps of this page for one change of two drafts.
  const twoDrafts = (action, call) => changeTwoDrafts(action, { feedId: feed.id, draft, onListChanged, setChanged, call });
  const setRole = (role) => {
    if (!role) return;
    twoDrafts(roleAction, async () => (await api(`/feeds/${feed.id}/role`, { method: 'PUT', body: { role } })).changed);
  };
  const addRelease = async () => {
    const done = await twoDrafts(linkAction, async () => (await api(`/feeds/${feed.id}/links`, {
      method: 'POST',
      body: { release: Number(chosen) },
    })).changed);
    if (done) setChosen('');
  };
  const removeRow = (row) => {
    if (!row.match) {
      // Studio keeps no link to a feed that is not in the collection. Only
      // this feed changes (ADR 0041 item 13).
      update((m) => set(m, 'remote_items', m.remote_items.filter((e) => e._cid !== row.entry._cid)));
      return;
    }
    twoDrafts(linkAction, async () => (await api(`/feeds/${feed.id}/links/${row.match.id}`, { method: 'DELETE' })).changed);
  };
  // Swap a release with the release above or below it (ADR 0041 item 10).
  // Other entries of the list keep their place.
  const moveRow = (position, by) => {
    const target = position + by;
    if (target < 0 || target >= rows.length) return;
    const a = rows[position].index;
    const b = rows[target].index;
    update((m) => set(m, 'remote_items', m.remote_items.map((e, i) => {
      if (i === a) return m.remote_items[b];
      if (i === b) return m.remote_items[a];
      return e;
    })));
  };
  // Send one image to the release folder, and use its address as the
  // cover image. A file that is not an image is removed again.
  const importImage = (file) => imageAction.run(async () => {
    if (!(await draft.flush())) return;
    const facts = await api(`/feeds/${feed.id}/files?name=${encodeURIComponent(file.name)}`, {
      method: 'POST',
      body: file,
    });
    if (facts.kind !== 'image') {
      if (!facts.on_host) {
        api(`/feeds/${feed.id}/files?file=${facts.library_file_id}`, { method: 'DELETE' }).catch(() => {});
      }
      throw Object.assign(new Error('Select an image file.'), { code: 'not_image' });
    }
    update((m) => set(m, 'image', facts.url));
    await draft.flush();
  });
  return html`<${ChangedNotices} changed=${changed} go=${go} />
    <section class="page-section"><h2>Publisher</h2>
      <div class="group">
        <${TextField} id="pub-title" label="Title" value=${model.title} onChange=${field('title')} />
        <${TextField} id="pub-author" label="Name" hint="The itunes:author of the feed." value=${model.author} onChange=${field('author')} />
        <${TextArea} id="pub-description" label="Description" value=${model.description} onChange=${field('description')} />
        <${TextField} id="pub-image" label="Cover image address" type="url" address="image" bad=${bad} value=${model.image} onChange=${field('image')}>
          <${ImageThumb} url=${model.image} className="field-cover" />
        <//>
        <div class="import-pick">
          <input id="pub-image-input" class="visually-hidden" type="file" disabled=${imageAction.busy}
            accept="image/*"
            onChange=${(e) => { const [file] = e.target.files; e.target.value = ''; if (file) importImage(file); }} />
          <label for="pub-image-input" class="button action">Import Image</label>
          ${imageAction.error && html`<p class="field-error" role="alert">${imageAction.error.message}</p>`}
        </div>
        <${TextField} id="pub-link" label="Website" type="url" address="link" bad=${bad} value=${model.link} onChange=${field('link')} />
      </div>
    </section>

    <section class="page-section"><h2>Role</h2>
      <div class="group">
        <div class="field">
          <label for="pub-role">Role</label>
          <div class="field-body">
            <select id="pub-role" value=${shownRole} disabled=${roleAction.busy} onChange=${(e) => setRole(e.target.value)}>
              ${shownRole === '' && html`<option value="" disabled>Select a role</option>`}
              <option value="artist">Artist</option>
              <option value="label">Label</option>
            </select>
            ${!feed.publisher_role && inferred && html`<span class="hint">Studio found this role in the feed.</span>`}
          </div>
        </div>
        <${ErrorNotice} error=${publisherError(roleAction.error, PUBLISHER_ROLE_TEXT)} />
      </div>
    </section>

    <section class="page-section"><h2>Releases</h2>
      <div class="group">
        ${rows.length === 0 && html`<p class="hint">No releases yet.</p>`}
        ${rows.map((row, position) => html`<div class="release-row" key=${row.entry._cid}>
          <span class="title">${row.entry.title || row.entry.feed_url}</span>
          <span class="sub">${row.match ? 'In the collection' : 'Not in the collection'}</span>
          <div class="actions">
            <button class="button" type="button" disabled=${position === 0 || linkAction.busy}
              aria-label=${`Move ${row.entry.title || 'this release'} up`}
              onClick=${() => moveRow(position, -1)}>Move up</button>
            <button class="button" type="button" disabled=${position === rows.length - 1 || linkAction.busy}
              aria-label=${`Move ${row.entry.title || 'this release'} down`}
              onClick=${() => moveRow(position, 1)}>Move down</button>
            <button class="button destructive" type="button" disabled=${linkAction.busy}
              aria-label=${`Remove ${row.entry.title || 'this release'}`}
              onClick=${() => removeRow(row)}>Remove</button>
          </div>
        </div>`)}
        ${candidates.length > 0 && html`<div class="field">
          <label for="pub-release">Release</label>
          <div class="field-body">
            <select id="pub-release" value=${chosen} onChange=${(e) => setChosen(e.target.value)}>
              <option value="">Select a release</option>
              ${candidates.map((f) => html`<option value=${f.id}>${f.title || f.path || 'Untitled feed'}</option>`)}
            </select>
          </div>
        </div>
        <div class="actions"><button class="button action" type="button" disabled=${!chosen || linkAction.busy}
          onClick=${addRelease}>Add the Release</button></div>`}
        ${candidates.length === 0 && html`<p class="hint">Studio has no other release in the collection to add.</p>`}
        <${ErrorNotice} error=${publisherError(linkAction.error, PUBLISHER_ROLE_TEXT)} />
      </div>
    </section>`;
}

// ---- The Files page (task 021g, ADR 0036 items 4, 7 to 14) ----
// The user sets the host of each file, starts a copy, a move or a
// deletion, and sees each host file of the release folder that is not a
// copy of a library file.

// The summary text of a file that is not in the library (ADR 0038 items
// 6 and 7). `download` is the row of the downloads route, or undefined.
function downloadSummary(download) {
  if (!download) return 'Not in the library';
  if (download.state === 'queued') return 'Queued';
  if (download.state === 'failed') return 'Download failed';
  return `Downloading ${downloadPercent(download)}%`;
}

function downloadPercent(download) {
  if (!download.total) return 0;
  return Math.min(100, Math.floor((download.bytes / download.total) * 100));
}

// The download controls of an open remote entry (ADR 0038 items 6 and 7,
// ADR 0037): "Download" and "Try again" are actions. "Stop" is normal.
function DownloadControls({ download, busy, onDownload, onStop }) {
  if (!download) {
    return html`<div class="actions">
      <button class="button action" type="button" disabled=${busy} onClick=${onDownload}>Download</button>
    </div>`;
  }
  if (download.state === 'failed') {
    return html`<div class="download-state">
      <p class="hint error-text" role="alert">${download.message || 'The download failed.'}</p>
      <div class="actions">
        <button class="button action" type="button" disabled=${busy} onClick=${onDownload}>Try again</button>
      </div>
    </div>`;
  }
  const percent = downloadPercent(download);
  return html`<div class="download-state">
    <div class="download-bar" role="progressbar" aria-label="Download" aria-valuemin="0" aria-valuemax="100"
      aria-valuenow=${percent}><span style=${`width: ${percent}%`}></span></div>
    <p class="hint">${download.state === 'queued' ? 'Queued' : `${fileSize(download.bytes)}${download.total ? ` of ${fileSize(download.total)}` : ''}`}</p>
    <div class="actions">
      <button class="button" type="button" disabled=${busy} onClick=${onStop}>Stop</button>
    </div>
  </div>`;
}

// One copy of a library file on a host (ADR 0036 items 12 and 13, ADR
// 0037). A pending operation takes over the row with "Undo". The copy
// that the feed uses shows no Reference button and no Delete button: it
// offers "Move to" each other host instead, so the user can move the use
// of the file in one click. A copy that the feed does not use offers
// "Reference", to put the feed on this copy, and "Delete".
function CopyRow({ copy, connectors, busy, onMoveTo, onReference, onDelete, onUndo }) {
  const name = connectorName(connectors, copy.connector_id);
  if (copy.pending) {
    const text = copy.pending.kind === 'delete' ? 'Deletion pending' : 'Copy pending';
    return html`<div class="copy-row">
      <span class="title">${name}</span>
      <span class="sub mono">${copy.path}</span>
      <span class="sub">${text}</span>
      <div class="actions">
        <button class="button" type="button" disabled=${busy} onClick=${() => onUndo(copy.pending.operation_id)}>Undo</button>
      </div>
    </div>`;
  }
  if (copy.in_use) {
    const targets = connectors.filter((c) => c.id !== copy.connector_id);
    return html`<div class="copy-row">
      <span class="title">${name}</span>
      <span class="sub mono">${copy.path}</span>
      ${copy.url && html`<span class="sub mono">${copy.url}</span>`}
      <span class="sub">Used by the feed</span>
      ${targets.length > 0 && html`<div class="actions">
        ${targets.map((c) => html`<button key=${c.id} class="button action" type="button" disabled=${busy}
          onClick=${() => onMoveTo(copy.connector_id, c.id)}>Move to ${c.name}</button>`)}
      </div>`}
    </div>`;
  }
  return html`<div class="copy-row">
    <span class="title">${name}</span>
    <span class="sub mono">${copy.path}</span>
    ${copy.url && html`<span class="sub mono">${copy.url}</span>`}
    <div class="actions">
      <button class="button action" type="button" disabled=${busy} onClick=${() => onReference(copy.connector_id)}>Reference</button>
      <button class="button destructive" type="button" disabled=${busy} onClick=${() => onDelete(copy.connector_id, copy.path)}>Delete</button>
    </div>
  </div>`;
}

// One library file, as a summary line that opens to show each of its
// copies (ADR 0028, ADR 0036 items 11 and 12). The badge reuses the
// format label of `FileRows` above. `.file-rows` holds the open rows with
// no empty space between them (review of milestone 7a).
//
// The entry shows one "Copy to" button for each host that has no copy of
// the file and no pending copy of it yet (ADR 0037). Each pending copy of
// the file, from `pending_copies`, shows its own row with "Undo".
function FileEntry({ feedId, file, defaultHost, connectors, busy, download, onDownload, onStop, onOperation, onUndo, onChanged }) {
  const summary = html`<span class="badge">${formatLabel(file.media_type)}</span>
    <span class="title">${file.path}</span>
    <span class="sub">${file.in_use ? 'In use' : 'Not in use'} · ${file.local
      ? `${file.copies.length} ${file.copies.length === 1 ? 'copy' : 'copies'}`
      : downloadSummary(download)}</span>`;
  const reference = (connectorId) => onOperation({ op: 'reference', file: file.id, connector_id: connectorId, from_url: file.url });
  const del = (connectorId, path) => onOperation({ op: 'delete', connector_id: connectorId, path });
  // A copy or a move of a remote file asks first (ADR 0038 item 12). The
  // size comes from the row.
  const askFirst = () => file.local || confirm(
    `Studio downloads this file first${file.size > 0 ? ` (${fileSize(file.size)})` : ''}. `
    + 'The copy goes into the draft now, and the write waits for the download.');
  const copyTo = (connectorId) => {
    if (askFirst()) onOperation({ op: 'copy', file: file.id, connector_id: connectorId });
  };
  const moveTo = (from, to) => {
    if (askFirst()) onOperation({ op: 'move', file: file.id, from, to });
  };
  const sizeDiffers = file.local && file.length_in_feed != null && file.length_in_feed > 0
    && file.length_in_feed !== file.size;
  const setHost = (value) => onChanged(`/feeds/${feedId}/files/${file.id}/host`, { connector_id: value });
  // A file with no host of its own uses the default host of the release
  // (ADR 0036 item 8).
  const hostValue = file.own_connector_id ?? '';
  const takenHosts = [
    ...file.copies.map((c) => c.connector_id),
    ...file.pending_copies.map((p) => p.connector_id),
  ];
  const copyTargets = connectors.filter((c) => !takenHosts.includes(c.id));
  return html`<${EntryRow} group="library-files" cid=${file.id} summary=${summary}>
    <div class="file-rows">
      ${!file.local && html`<p class="hint">Not in the library. Studio uses the file on its host.</p>`}
      ${!file.local && html`<${DownloadControls} download=${download} busy=${busy}
        onDownload=${() => onDownload(file.id)} onStop=${() => onStop(file.id)} />`}
      ${sizeDiffers && html`<p class="hint">The feed gives ${fileSize(file.length_in_feed)}. The file is ${fileSize(file.size)}.</p>`}
      ${file.local && file.copies.length === 0 && file.pending_copies.length === 0
        && html`<p class="hint">Studio sends this file to its host when you write.</p>`}
      ${file.copies.map((copy) => html`<${CopyRow} key=${`${copy.connector_id}-${copy.path}`} copy=${copy}
        connectors=${connectors} busy=${busy} onMoveTo=${moveTo} onReference=${reference} onDelete=${del} onUndo=${onUndo} />`)}
      ${file.pending_copies.map((pending) => html`<div class="copy-row" key=${`pending-${pending.operation_id}`}>
        <span class="title">Copy to ${connectorName(connectors, pending.connector_id)} pending</span>
        <span class="sub mono">${pending.path}</span>
        <div class="actions">
          <button class="button" type="button" disabled=${busy} onClick=${() => onUndo(pending.operation_id)}>Undo</button>
        </div>
      </div>`)}
      ${copyTargets.length > 0 && html`<div class="actions">
        ${copyTargets.map((c) => html`<button key=${c.id} class="button action" type="button" disabled=${busy}
          onClick=${() => copyTo(c.id)}>Copy to ${c.name}</button>`)}
      </div>`}
      <div class="field">
        <label for=${`file-host-${file.id}`}>Host of this file</label>
        <select id=${`file-host-${file.id}`} class="button" value=${hostValue} disabled=${busy}
          onChange=${(e) => setHost(e.target.value === '' ? null : Number(e.target.value))}>
          <option value="">Use the default host of the release</option>
          ${connectors.map((c) => html`<option value=${c.id}>${c.name}</option>`)}
        </select>
      </div>
    </div>
  <//>`;
}

// A host file of the release folder that is not a copy of a library file
// (ADR 0036 item 11, ADR 0037). A pending operation takes over the row
// with "Undo". A file that the feed uses keeps its "Delete" button, but
// disabled, since the user must take the file out of the feed first.
function OtherFileRow({ file, connectors, busy, onDelete, onUndo }) {
  const base = `${connectorName(connectors, file.connector_id)} · ${fileSize(file.size)}`;
  if (file.pending) {
    const text = file.pending.kind === 'delete' ? 'Deletion pending' : 'Copy pending';
    return html`<div class="alt-enclosure">
      <span class="title">${file.path}</span>
      <span class="sub">${base} · ${text}</span>
      <button class="button" type="button" disabled=${busy} onClick=${() => onUndo(file.pending.operation_id)}>Undo</button>
    </div>`;
  }
  if (file.pointer) {
    return html`<div class="alt-enclosure">
      <span class="title">${file.path}</span>
      <span class="sub">${base} · Pointer to the new address</span>
    </div>`;
  }
  // The feed XML of a feed of the library is the feed itself. The page
  // does not offer to delete it (ADR 0035 item 17).
  if (file.feed_xml) {
    return html`<div class="alt-enclosure">
      <span class="title">${file.path}</span>
      <span class="sub">${base} · The feed file</span>
    </div>`;
  }
  if (file.in_use) {
    return html`<div class="alt-enclosure">
      <span class="title">${file.path}</span>
      <span class="sub">${base} · Used by the feed · Remove it from the feed first</span>
      <button class="button destructive" type="button" disabled>Delete</button>
    </div>`;
  }
  return html`<div class="alt-enclosure">
    <span class="title">${file.path}</span>
    <span class="sub">${base}</span>
    <button class="button destructive" type="button" disabled=${busy} onClick=${() => onDelete(file.connector_id, file.path)}>Delete</button>
  </div>`;
}

// One difference of the library folder (ADR 0038 items 13 to 15, ADR
// 0037). `file` is the row of the files route for a missing or a changed
// file. `onAct` sends the decision of the user. An error shows in the row.
function ChangeRow({ difference, file, busy, onAct }) {
  const row = useAction();
  const act = (body) => row.run(() => onAct(body));
  const off = busy || row.busy;
  const name = html`<span class="mono">${difference.path}</span>`;
  let text;
  let buttons;
  if (difference.kind === 'new') {
    text = html`New file: ${name} (${fileSize(difference.size)})`;
    buttons = html`<button class="button action" type="button" disabled=${off}
      onClick=${() => act({ action: 'add', path: difference.path })}>Add to the library</button>`;
  } else if (difference.kind === 'missing') {
    const hasSource = file && (file.copies.length > 0 || file.source_url);
    text = html`Missing: ${name}`;
    buttons = html`${hasSource && html`<button class="button action" type="button" disabled=${off}
        onClick=${() => act({ action: 'download', file: difference.file })}>Download again</button>`}
      <button class="button destructive" type="button" disabled=${off}
        onClick=${() => act({ action: 'forget', file: difference.file })}>Forget the file</button>`;
  } else {
    const hasCopy = file && file.copies.length > 0;
    text = html`Changed: ${name} (${fileSize(difference.size)}${file ? `, was ${fileSize(file.size)}` : ''})`;
    buttons = html`<button class="button action" type="button" disabled=${off}
        onClick=${() => act({ action: 'use_new', file: difference.file })}>Use the new file</button>
      <button class="button" type="button" disabled=${off || !hasCopy}
        onClick=${() => act({ action: 'restore', file: difference.file })}>Restore</button>
      ${!hasCopy && html`<span class="sub">No copy on a host</span>`}`;
  }
  return html`<div class="change-row">
    <span class="title">${text}</span>
    <div class="actions">${buttons}</div>
    <${ErrorNotice} error=${row.error} />
  </div>`;
}

// The Files page: the host of the release, each of its library files with
// their copies, and each other host file of its release folder (ADR 0036
// items 4, 7, 9 and 11; task 021g, task 022b). `onOpsChanged` tells the
// shell that the host operations of the draft changed, so that it loads
// the draft again (ADR 0036 item 14). `reloadKey` changes after a write,
// after a discard of the draft and after an "Undo" in the review sheet, so
// the page loads the files route again (review of milestone 7a, visual
// check of task 022).
function FilesPage({ feed, connectors, onOpsChanged, reloadKey, differences, onRescan }) {
  const [data, setData] = useState(null);
  const [error, setError] = useState(null);
  const { busy, error: opError, run } = useAction();
  const scan = useAction();
  // The downloads of this feed, from the downloads route.
  const [downloads, setDownloads] = useState([]);
  const lastActive = useRef(new Set());

  const load = async () => {
    try {
      setData(await api(`/feeds/${feed.id}/files`));
      setError(null);
    } catch (e) {
      setError(e);
    }
  };
  // A different feed clears the list. A reload of the same feed keeps the
  // list, so that the open entry stays open.
  const shownFeed = useRef(null);
  useEffect(() => {
    if (shownFeed.current !== feed.id) setData(null);
    shownFeed.current = feed.id;
    load();
  }, [feed.id, reloadKey]);

  // Read the downloads. A file that was queued or running before, and is
  // not in the list now, is complete or stopped. The page then loads the
  // files route again. A failed download stays in the list.
  const loadDownloads = async () => {
    try {
      const list = (await api(`/feeds/${feed.id}/downloads`)).downloads;
      const active = new Set(list.filter((d) => d.state !== 'failed').map((d) => d.file_id));
      const finished = [...lastActive.current].some((id) => !list.some((d) => d.file_id === id));
      lastActive.current = active;
      setDownloads(list);
      if (finished) await load();
    } catch (_) { /* The next pass tries again. */ }
  };
  useEffect(() => {
    lastActive.current = new Set();
    setDownloads([]);
    loadDownloads();
  }, [feed.id]);
  const activeCount = downloads.filter((d) => d.state !== 'failed').length;
  // While a download is queued or running, read the list each second.
  useEffect(() => {
    if (activeCount === 0) return undefined;
    const timer = setInterval(loadDownloads, 1000);
    return () => clearInterval(timer);
  }, [feed.id, activeCount]);

  const startDownloads = (body) => run(async () => {
    await api(`/feeds/${feed.id}/downloads`, { method: 'POST', body });
    await loadDownloads();
  });
  const stopDownload = (fileId) => run(async () => {
    await api(`/feeds/${feed.id}/downloads/${fileId}`, { method: 'DELETE' });
    await loadDownloads();
    await load();
  });
  const stopAll = () => run(async () => {
    for (const d of downloads.filter((x) => x.state !== 'failed')) {
      await api(`/feeds/${feed.id}/downloads/${d.file_id}`, { method: 'DELETE' });
    }
    await loadDownloads();
    await load();
  });

  // Each host that holds something of this release: its own host, the
  // default host of its files, and each host of a copy or an other file.
  const releaseHosts = () => {
    if (!data) return [];
    const ids = new Set();
    if (feed.connector_id) ids.add(feed.connector_id);
    if (data.files_connector_id) ids.add(data.files_connector_id);
    for (const file of data.files) {
      if (file.connector_id) ids.add(file.connector_id);
      for (const copy of file.copies) ids.add(copy.connector_id);
    }
    for (const extra of data.other_host_files) ids.add(extra.connector_id);
    return [...ids];
  };

  const runScan = () => scan.run(async () => {
    for (const id of releaseHosts()) {
      await api(`/hosts/${id}/scan`, { method: 'POST' });
    }
    await load();
  });

  const runLibraryScan = () => scan.run(async () => { await onRescan(); await load(); });

  // Send the decision of the user for one difference. Then scan again and
  // load the files again. A new file replaces the bytes of the feed, so
  // the shell loads the draft again. A download starts for "Download
  // again" and "Restore".
  const changeLibrary = async (body) => {
    await api(`/feeds/${feed.id}/library-changes`, { method: 'POST', body });
    await onRescan();
    await load();
    if (body.action === 'use_new') onOpsChanged();
    if (body.action === 'download' || body.action === 'restore') await loadDownloads();
  };

  const setDefaultHost = (connectorId) => run(async () => {
    await api(`/feeds/${feed.id}/hosts`, { method: 'PUT', body: { files_connector_id: connectorId } });
    await load();
  });

  // The pending move with its method and its two addresses, from the last
  // route answer. The files route gives only the host and the path, so a
  // page load asks the diff route.
  const [moveInfo, setMoveInfo] = useState(null);
  const moveMethod = moveInfo ? moveInfo.method : 'pointer';
  const pendingMove = data && data.feed_move;
  useEffect(() => {
    if (!pendingMove) return;
    api(`/feeds/${feed.id}/diff`).then(
      (d) => { if (d.feed_move) setMoveInfo(d.feed_move); },
      () => { /* The hint keeps the pointer text with no addresses. */ });
  }, [feed.id, pendingMove && pendingMove.connector_id, reloadKey]);

  // The route changes the self link of the draft, so the shell loads the
  // draft again, as after a host operation.
  const setFeedHost = (connectorId) => run(async () => {
    const diff = await api(`/feeds/${feed.id}/feed-host`, { method: 'PUT', body: { connector_id: connectorId } });
    setMoveInfo(diff.feed_move || null);
    onOpsChanged();
    await load();
  });

  const setFileHost = (path, body) => run(async () => {
    await api(path, { method: 'PUT', body });
    await load();
  });

  // Each operation reloads the files route, so that the page shows the
  // mark of each pending copy and each pending deletion at once (review
  // of milestone 7a).
  const addOperation = (body) => run(async () => {
    await api(`/feeds/${feed.id}/operations`, { method: 'POST', body });
    onOpsChanged();
    await load();
    // The route of a remote file queued its download.
    await loadDownloads();
  });

  // Take a pending operation out of the draft, such as a copy or a
  // deletion that the user no longer wants (task 022b).
  const undoOperation = (operationId) => run(async () => {
    await api(`/feeds/${feed.id}/operations/${operationId}`, { method: 'DELETE' });
    onOpsChanged();
    await load();
  });

  if (error) return html`<section class="page-section"><${ErrorNotice} error=${error} /></section>`;
  if (!data) return html`<section class="page-section"><p class="hint" role="status">Loading the files…</p></section>`;

  const defaultHost = data.files_connector_id ?? feed.connector_id;
  const feedHost = data.feed_move ? data.feed_move.connector_id : feed.connector_id;

  return html`${differences.length > 0 && html`<section class="page-section"><h2>Changes in the library folder</h2>
      <div class="group">
        ${differences.map((difference) => html`<${ChangeRow}
          key=${`${difference.kind}-${difference.file ?? difference.path}`} difference=${difference}
          file=${data.files.find((f) => f.id === difference.file)} busy=${busy} onAct=${changeLibrary} />`)}
      </div>
    </section>`}

    <section class="page-section"><h2>Files</h2>
      <div class="actions">
        <button class="button action" type="button" disabled=${scan.busy} onClick=${runScan}>
          ${scan.busy ? 'Scanning…' : 'Scan the hosts'}</button>
        <button class="button action" type="button" disabled=${scan.busy} onClick=${runLibraryScan}>
          Scan the library folder</button>
      </div>
      <${ErrorNotice} error=${scan.error} />
      <${ErrorNotice} error=${opError} />
    </section>

    <section class="page-section"><h2>Hosts</h2>
      <div class="group">
        <div class="field">
          <label for="files-default-host">Default host of the release files</label>
          <select id="files-default-host" class="button" value=${defaultHost} disabled=${busy}
            onChange=${(e) => setDefaultHost(Number(e.target.value))}>
            ${connectors.map((c) => html`<option value=${c.id}>${c.name}</option>`)}
          </select>
        </div>
        <div class="field">
          <label for="files-feed-host">Host of the feed XML</label>
          <select id="files-feed-host" class="button" value=${feedHost} disabled=${busy}
            onChange=${(e) => setFeedHost(Number(e.target.value))}>
            ${connectors.map((c) => html`<option value=${c.id}>${c.name}</option>`)}
          </select>
          ${data.feed_move && html`<p class="hint">The feed moves to ${connectorName(connectors, data.feed_move.connector_id)} when you write${moveInfo && moveInfo.url ? html`, at <span class="mono">${moveInfo.url}</span>` : ''}.
            The address from before${moveInfo && moveInfo.from_url ? html`, <span class="mono">${moveInfo.from_url}</span>,` : ''} gets ${moveMethod === 'redirect' ? 'a redirect' : 'a pointer file'}.</p>`}
        </div>
      </div>
    </section>

    <section class="page-section"><h2>Library files</h2>
      ${data.remote_count > 0 && html`<div class="notice download-notice">
        <span>${activeCount > 0
          ? `Downloading ${activeCount} ${activeCount === 1 ? 'file' : 'files'}…`
          : `${data.remote_count} ${data.remote_count === 1 ? 'file is' : 'files are'} not in the library (${fileSize(data.remote_bytes)}).`}</span>
        <div class="actions">
          ${activeCount > 0
            ? html`<button class="button" type="button" disabled=${busy} onClick=${stopAll}>Stop all</button>`
            : html`<button class="button action" type="button" disabled=${busy} onClick=${() => startDownloads({ all: true })}>Download all</button>`}
        </div>
      </div>`}
      ${data.files.length === 0 && html`<p class="hint">No files yet.</p>`}
      ${data.files.length > 0 && html`<div class="group">
        ${data.files.map((file) => html`<${FileEntry} key=${file.id} feedId=${feed.id} file=${file} defaultHost=${defaultHost}
          connectors=${connectors} busy=${busy} onOperation=${addOperation} onUndo=${undoOperation}
          download=${downloads.find((d) => d.file_id === file.id)}
          onDownload=${(fileId) => startDownloads({ files: [fileId] })} onStop=${stopDownload}
          onChanged=${(path, body) => setFileHost(path, body)} />`)}
      </div>`}
    </section>

    ${data.other_host_files.length > 0 && html`<section class="page-section"><h2>Other files on the hosts</h2>
      <div class="group">
        ${data.other_host_files.map((extra) => html`<${OtherFileRow}
          key=${`${extra.connector_id}-${extra.path}`} file=${extra} connectors=${connectors} busy=${busy}
          onDelete=${(connectorId, path) => addOperation({ op: 'delete', connector_id: connectorId, path })}
          onUndo=${undoOperation} />`)}
      </div>
    </section>`}`;
}

// The album editor shows one page of the typed model of the draft: the
// album, the credits, the recipients or one track (task 014c step 3). The
// shell loads the draft and shows its toolbar. This component does not call
// useDraft (ADR 0029 item 4).
export function AlbumEditor({ feed, feeds, draft, view, go, adding, onAdding, importing, onImporting, onRemoved, connectors, onOpsChanged, filesReloadKey, differences, onRescan, onListChanged }) {
  const { model, update } = draft;
  const page = view.page || 'album';
  const trackIndex = model && page === 'track' ? model.items.findIndex((item) => isTrack(item, view.track)) : -1;
  // When the track got its GUID, put the new key in the view, so that the
  // tree shows the current track.
  const currentKey = trackIndex === -1 ? null : trackKey(model.items[trackIndex]);
  useEffect(() => {
    if (currentKey && currentKey !== view.track) go({ ...view, track: currentKey }, { replace: true });
  }, [currentKey, view.track]);
  if (!model) {
    return html`<div class="page"><p class="hint" role="status">${STATUS[draft.status]}</p>
      <${ErrorNotice} error=${draft.error} /></div>`;
  }
  // A publisher feed has the publisher page and the Files page only (ADR
  // 0041 item 3).
  const publisher = isPublisher(feed);
  let body;
  // When the track key of the view is not in the model, for example after a
  // write that gave each item a new client ID, the editor shows the album
  // page (task 014c, Architecture).
  if (publisher && page !== 'files') {
    body = html`<${PublisherPage} feed=${feed} feeds=${feeds} model=${model} update=${update} bad=${draft.bad}
      draft=${draft} go=${go} onListChanged=${onListChanged} />`;
  } else if (page === 'track' && trackIndex !== -1) {
    body = html`<${TrackPage} feed=${feed} model=${model} update=${update} bad=${draft.bad} trackIndex=${trackIndex} go=${go}
      draft=${draft} onOpsChanged=${onOpsChanged} />`;
  } else if (page === 'credits') {
    body = html`<${CreditsPage} model=${model} update=${update} bad=${draft.bad} />`;
  } else if (page === 'recipients') {
    body = html`<${RecipientsPage} model=${model} update=${update} />`;
  } else if (page === 'files') {
    body = html`<${FilesPage} feed=${feed} connectors=${connectors} onOpsChanged=${onOpsChanged} reloadKey=${filesReloadKey}
      differences=${differences} onRescan=${onRescan} />`;
  } else {
    body = html`<${AlbumPage} feed=${feed} feeds=${feeds} model=${model} update=${update} bad=${draft.bad} draft=${draft} go=${go}
      onAdding=${onAdding} onImporting=${onImporting} onRemoved=${onRemoved} onListChanged=${onListChanged} />`;
  }
  const onAdd = (item) => {
    const [added] = withCids([item]);
    update((m) => set(m, 'items', [...m.items, added]));
    onAdding(false);
    go({ name: 'feed', id: feed.id, page: 'track', track: trackKey(added) });
  };
  // The import sheet already sent each file to the draft of this feed. Add
  // the tracks that it built, and the image that it chose for the album, to
  // the model (task architecture).
  const onImport = (tracks, albumImageUrl) => {
    const added = withCids(tracks);
    update((m) => {
      const withImage = albumImageUrl ? set(m, 'image', albumImageUrl) : m;
      return set(withImage, 'items', [...withImage.items, ...added]);
    });
    onImporting(false);
    go({ name: 'feed', id: feed.id });
  };
  return html`<div class="page">
    ${body}
    ${!publisher && html`<${AddTrackSheet} open=${adding} onClose=${() => onAdding(false)} onAdd=${onAdd} />`}
    ${!publisher && html`<${ImportSheet} feedId=${feed.id} open=${importing} hasAlbumImage=${Boolean(model.image)}
      onClose=${() => onImporting(false)} onAdd=${onImport} />`}
  </div>`;
}

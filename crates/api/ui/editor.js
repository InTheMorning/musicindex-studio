// The album editor (task 011, ADR 0022), its pages (task 014c, ADR 0029
// items 5 to 10) and its toolbar and sheets (task 014b, ADR 0029). The UI
// edits the typed model of the draft. Each list entry carries its key, and
// the draft saves automatically.
import { html, useState, useEffect, useRef, createContext, useContext } from './vendor/htm-preact-standalone.module.js';
import { api, sendInParts, Field, ErrorNotice, useAction, addressSource, ImageThumb, useImage, SessionContext } from './app.js';
import { AlbumPayments, TrackPayments, ShareStack, recipientSummary } from './values.js';
import { withCids, STATUS } from './draft.js';
import { feedKind, ROLES, FIRST_CLASS_ROLES, roleWord, roleChoice, toggleRole, trackKey, isTrack, isPublisher, isPlaylist, isReadOnly, reviewHost, moveNote } from './shell.js';
import { isLiveItem, liveKey, liveDot, liveProblems, nextAction, canRecord, hasRecording, showSummary, statusLabel, startToIso, isoToLocalInput, grantActive, grantSummary, grantWriteText, grantBoxState, hostUsesSigner } from './live.js';
import { pickFromIndex, remoteItemFromPick, addTrack, hasTrack, removeTrack, trackRow, tracksSummary } from './playlist.js';
import { moveTo, moveAmong } from './order.js';
import { announceLines, announceStatus, isWaiting } from './announce.js';
import { npubOf, eventsToSign, signAll, signErrorText, signSummary, signProgressLabel, SignError, remoteSigner, signerChoice } from './signing.js';
import { SignerConnect } from './signer.js';
import { progressPercent } from './upload.js';
import { coverOf, coverConflict } from './cover.js';
import { platformBlocks } from './listing.js';
import { pageSequence } from './pages.js';
import { isExplicit } from './explicit.js';
import { op3Hint, op3Page } from './op3.js';
import { shortKey, keySummary, feedKey, withFeedKey, sourceText, withPersonKey } from './nostrkey.js';
import { usesAlbumCredits, withTrackCredits, withAlbumCredits } from './credits.js';
import { shortType as formatLabel, propertiesSummary, propertiesOpen, durationFromSeconds, needsMetadataRead, clockText, DURATION_NOTICE } from './audio.js';
import { PURPOSES, purposeTokens, withPurpose, imageFacts, artworkSummary } from './artwork.js';
import { fieldKind, sources, hostTree, canSelect, reasonText, acceptsFile, dropResult, wrongKindText, replacedAddresses, selectedFacts } from './picker.js';

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
// (ADR 0023 item 4). `address` can be a list of model paths.
export function TextField({ id, label, value, onChange, required = false, address, bad = [], ...rest }) {
  const paths = [].concat(address || []);
  const error = bad.some((b) => paths.includes(b.field)) ? BAD_ADDRESS : null;
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

// The position menu of one entry of a list (ADR 0081). A selection moves the
// entry to that position. `name` completes "Position of …".
export function PositionSelect({ id, index, count, name, disabled, onMove }) {
  return html`<select id=${id} class="position" aria-label=${`Position of ${name}`} value=${String(index)}
    disabled=${disabled || count < 2} onChange=${(e) => onMove(Number(e.target.value))}>
    ${Array.from({ length: count }, (_, i) => html`<option value=${String(i)}>${i + 1}</option>`)}
  </select>`;
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
// `path` is the model path of the list, for the address fields. A track gives
// `items/${index}/persons`.
const BLANK_PERSON = { name: '', role: null, group: null, img: null, href: null, npub: null };

function Persons({ prefix, persons, onChange, bad, path = 'persons' }) {
  const blank = BLANK_PERSON;
  const { feedId } = useContext(PickerContext) || {};
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
        <${TextField} id=${`${id}-img`} label="Picture address" type="url" address=${`${path}/${i}/img`} bad=${bad} value=${p.img} onChange=${change('img')} />
        <${TextField} id=${`${id}-href`} label="Information address" type="url" address=${`${path}/${i}/href`} bad=${bad} value=${p.href} onChange=${change('href')} />
        <${NostrRow} label="Nostr" summary=${keySummary(p.name, p.npub)} value=${p.npub} feedId=${feedId}
          bad=${(bad || []).some((b) => b.field === `${path}/${i}/npub`)}
          person=${{ name: p.name, img: p.img }}
          onSet=${(npub, picked) => onChange(setAt(persons, i, withPersonKey(p, npub, picked)))} />
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

function Enclosure({ id, path, enclosure, onChange, bad, pick }) {
  const value = enclosure || { url: null, length: null, media_type: null };
  // An enclosure with no field is no enclosure.
  const change = (name) => (text) => {
    const next = set(value, name, text);
    onChange(next.url === null && next.length === null && next.media_type === null ? null : next);
  };
  // "Properties" opens by itself when a value is empty or has an error. It
  // stays open until the user closes it (ADR 0062 items 24 and 25).
  const errors = ['length', 'media_type'].filter((name) => (bad || []).some((b) => b.field === `${path}enclosure/${name}`));
  const mustOpen = propertiesOpen(enclosure, errors);
  const [opened, setOpened] = useState(mustOpen);
  useEffect(() => { if (mustOpen) setOpened(true); }, [mustOpen]);
  return html`<${TextField} id=${`${id}-url`} label="Audio file address" type="url" address=${`${path}enclosure/url`} bad=${bad} value=${value.url} onChange=${change('url')}
      rootProps=${pick.rootProps}><${ChooseFile} pick=${pick} /><//>
    <${AudioPlayer} id=${id} url=${value.url} pick=${pick} />
    <details class="disclosure" open=${opened} onToggle=${(e) => setOpened(e.currentTarget.open)}>
      <summary class="link-row"><span class="label">Properties</span>
        <span class="summary">${propertiesSummary(enclosure)}</span></summary>
      <div class="form">
        <${TextField} id=${`${id}-length`} label="Audio file size in bytes" inputmode="numeric" value=${value.length} onChange=${change('length')} />
        <${TextField} id=${`${id}-type`} label="Audio file type" hint="For example audio/mpeg." value=${value.media_type} onChange=${change('media_type')} />
      </div>
    </details>`;
}

// The player of an audio file (ADR 0062 items 26 to 29). It plays through
// Studio with a media ticket and reads no audio before the user presses
// play. The one exception is the read of the metadata after a selection that
// `needsMetadataRead` marks. The element then reads the duration, and
// `pick.metadataDone` puts it in the track.
const TICKET_RENEW_MS = 50 * 60 * 1000;
const PLAYER_ERROR = 'Studio cannot play this file.';

function AudioPlayer({ id, url, pick }) {
  const audio = useRef(null);
  // The address that the element holds, with the time of its ticket.
  const held = useRef(null);
  // The position to restore after a new ticket.
  const resume = useRef(0);
  const [playing, setPlaying] = useState(false);
  const [position, setPosition] = useState(0);
  const [length, setLength] = useState(NaN);
  const [error, setError] = useState(null);
  const usable = typeof url === 'string' && /^https?:\/\/\S+$/i.test(url);
  const reading = Boolean(pick.metadataFor) && pick.metadataFor === url;
  // A ticket is for one address, so a new address needs a new ticket.
  const load = async () => {
    const { ticket } = await api('/media/tickets', { method: 'POST', body: { url } });
    audio.current.preload = reading ? 'metadata' : 'none';
    audio.current.src = `/media/audio?ticket=${ticket}`;
    held.current = { url, at: Date.now() };
  };
  useEffect(() => {
    const element = audio.current;
    element.pause();
    element.removeAttribute('src');
    element.load();
    held.current = null;
    resume.current = 0;
    setPlaying(false);
    setPosition(0);
    setLength(NaN);
    setError(null);
  }, [url]);
  useEffect(() => {
    if (!reading || !usable) return;
    load().catch(() => pick.metadataDone(null));
  }, [url, pick.metadataFor]);
  const toggle = async () => {
    const element = audio.current;
    if (!element.paused) { element.pause(); return; }
    setError(null);
    const fresh = held.current && held.current.url === url && Date.now() - held.current.at < TICKET_RENEW_MS;
    try {
      if (!fresh) {
        resume.current = held.current && held.current.url === url ? element.currentTime : 0;
        await load();
      }
      await element.play();
    } catch (e) {
      setError(PLAYER_ERROR);
    }
  };
  const onLoadedMetadata = (e) => {
    const element = e.currentTarget;
    setLength(element.duration);
    if (resume.current > 0) { element.currentTime = resume.current; resume.current = 0; }
    if (reading) pick.metadataDone(durationFromSeconds(element.duration));
  };
  const onError = (e) => {
    // An element with no address gives no error of its own.
    if (!e.currentTarget.getAttribute('src')) return;
    setPlaying(false);
    setError(PLAYER_ERROR);
    if (reading) pick.metadataDone(null);
  };
  const known = Number.isFinite(length) && length > 0;
  return html`<div class="field">
    <label for=${`${id}-play`}>Player</label>
    <div class="field-body">
      <div class="player">
        <button id=${`${id}-play`} class="button icon-button" type="button" disabled=${!usable}
          aria-label=${playing ? 'Pause' : 'Play'} onClick=${toggle}>${playing ? '❚❚' : '▶'}</button>
        <input class="player-bar" type="range" min="0" max=${known ? length : 0} step="any" value=${position}
          aria-label="Position" disabled=${!known}
          onInput=${(e) => { audio.current.currentTime = Number(e.currentTarget.value); setPosition(Number(e.currentTarget.value)); }} />
        <span class="player-time">${clockText(position)} / ${clockText(length)}</span>
      </div>
      ${error && html`<span class="field-error" role="alert">${error}</span>`}
      <audio ref=${audio} preload="none" onLoadedMetadata=${onLoadedMetadata} onDurationChange=${(e) => setLength(e.currentTarget.duration)}
        onTimeUpdate=${(e) => setPosition(e.currentTarget.currentTime)} onPlay=${() => setPlaying(true)} onPause=${() => setPlaying(false)}
        onEnded=${() => setPlaying(false)} onError=${onError}></audio>
    </div>
  </div>`;
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
function AddTrackSheet({ open, onClose, onAdd, onFiles }) {
  const ref = useSheet(open);
  const [title, setTitle] = useState('');
  const [url, setUrl] = useState('');
  const [length, setLength] = useState('');
  const [type, setType] = useState('');
  const [facts, setFacts] = useState({ state: 'none', for: '' });
  const [opened, setOpened] = useState(false);
  // "Properties" opens by itself when the lookup finds fewer than two facts.
  // It stays closed while the lookup is not done. The user can open it at
  // each time (ADR 0073 item 14).
  const mustOpen = (facts.state === 'partial' || facts.state === 'failed')
    && propertiesOpen({ length, media_type: type }, []);
  useEffect(() => { if (mustOpen) setOpened(true); }, [mustOpen]);
  // Clear the sheet each time that it opens.
  useEffect(() => {
    if (!open) return;
    setTitle('');
    setUrl('');
    setLength('');
    setType('');
    setFacts({ state: 'none', for: '' });
    setOpened(false);
  }, [open]);
  const readFacts = async () => {
    const address = url.trim();
    if (!address || facts.for === address) return;
    // A new address clears the facts of the address before it, so that a
    // failed lookup does not keep the size and the type of a different file.
    setLength('');
    setType('');
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
      itunes_episode: null,
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
      <h2 id="add-track-title">Add a track from an address</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <form class="body" id="add-track-form" onSubmit=${submit}>
      <div class="group rows">
        <${Field} id="new-track-title" label="Title" value=${title} onInput=${(e) => setTitle(e.target.value)} />
        <${Field} id="new-track-url" label="Audio file address" type="url" required
          hint="Studio does not keep audio files. Give the address where your host keeps the file."
          error=${facts.state === 'invalid' && facts.for === url.trim() ? BAD_ADDRESS : null}
          value=${url} onInput=${(e) => setUrl(e.target.value)} onBlur=${readFacts} />
        ${FACTS[facts.state] && html`<p class="hint" role="status">${FACTS[facts.state]}</p>`}
        <details class="disclosure" open=${opened} onToggle=${(e) => setOpened(e.currentTarget.open)}>
          <summary class="link-row"><span class="label">Properties</span>
            <span class="summary">${propertiesSummary({ length, media_type: type })}</span></summary>
          <div class="form">
            <${Field} id="new-track-length" label="Audio file size in bytes" inputmode="numeric" required
              value=${length} onInput=${(e) => setLength(e.target.value)} />
            <${Field} id="new-track-type" label="Audio file type" hint="For example audio/mpeg." required
              value=${type} onInput=${(e) => setType(e.target.value)} />
          </div>
        </details>
      </div>
      <button class="button link" type="button" onClick=${onFiles}>Choose files from this computer</button>
    </form>
    <footer>
      <button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>
      <button class="button primary" type="submit" form="add-track-form" disabled=${!ready}>＋ Add the track</button>
    </footer>
  </dialog>`;
}

// ---- The Nostr key rows and the sheet "Find a Nostr profile" (ADR 0074 items 18 to 25) ----

// The row of a Nostr key. It opens the sheet. `person` gives the name and the
// picture of a person entry, or it is missing for the key of a feed. `bad` is
// true when the core found the key not correct.
function NostrRow({ label, summary, value, feedId, bad, person, onSet }) {
  const [open, setOpen] = useState(false);
  return html`<button class="link-row" type="button" onClick=${() => setOpen(true)}>
      <span class="label">${label}</span>
      <span class=${bad ? 'summary field-error' : 'summary'}>${bad ? 'Not a Nostr key' : summary}</span>
      <span class="go" aria-hidden="true">›</span>
    </button>
    <${NostrSheet} open=${open} value=${value} feedId=${feedId} person=${person} onSet=${onSet} onClose=${() => setOpen(false)} />`;
}

const FAILED_TEXT = {
  collection: 'Studio could not read your feeds.',
  musicindex: 'musicindex did not answer.',
  relay: 'The search relay did not answer.',
};

// One profile of the answer of `GET /nostr/profiles`. The picture comes
// through the image route (ADR 0016).
function ProfileBody({ profile }) {
  const { src } = useImage(profile.picture);
  const domain = profile.nip05 && profile.nip05.includes('@') ? profile.nip05.split('@').pop() : '';
  return html`<span style="flex:none;width:40px;height:40px;border-radius:8px;overflow:hidden;display:block">
      ${src && html`<img src=${src} alt="" width="40" height="40" />`}</span>
    <span class="text">
      <span class="title">${profile.name || shortKey(profile.npub)}</span>
      ${profile.nip05 && html`<span class="summary">${profile.nip05}${profile.nip05_ok ? ' ✓' : ''}</span>`}
      <span class="summary mono">${shortKey(profile.npub)}</span>
      <span class="summary">${profile.sources.map(sourceText).join(' · ')}</span>
      ${profile.domain_match && html`<span class="summary">Confirmed at ${domain}</span>`}
    </span>`;
}

// `value` is the key of the row, or `null`. A person entry gives `person`
// with its name and picture. A selection calls `onSet(npub, picked)`. The key
// `null` removes the key. `picked` has the profile name and picture that the
// user chose to use.
function NostrSheet({ open, value, feedId, person, onSet, onClose }) {
  const ref = useSheet(open);
  const session = useContext(SessionContext);
  const mine = session && session.session && session.session.pubkey ? npubOf(session.session.pubkey) : null;
  const [changing, setChanging] = useState(false);
  const [query, setQuery] = useState('');
  const [found, setFound] = useState(null);
  const [card, setCard] = useState(null);
  const [useName, setUseName] = useState(false);
  const [usePicture, setUsePicture] = useState(false);
  const search = useAction();
  const suffix = feedId ? `&feed=${feedId}` : '';
  useEffect(() => {
    if (!open) return undefined;
    setChanging(false);
    setQuery('');
    setFound(null);
    setCard(null);
    setUseName(!!person && !person.name);
    setUsePicture(!!person && !person.img);
    if (!value) return undefined;
    let current = true;
    api(`/nostr/profiles?q=${encodeURIComponent(value)}${suffix}`).then(
      (answer) => { if (current) setCard({ profile: answer.profiles.find((p) => p.npub === value) || null }); },
      () => { if (current) setCard({ profile: null }); },
    );
    return () => { current = false; };
  }, [open]);
  const choose = (npub, profile) => {
    onSet(npub, profile ? { name: useName ? profile.name : null, picture: usePicture ? profile.picture : null } : {});
    ref.current.close();
  };
  const runSearch = (event) => {
    event.preventDefault();
    const text = query.trim();
    if (!text) return;
    search.run(async () => setFound(await api(`/nostr/profiles?q=${encodeURIComponent(text)}${suffix}`)));
  };
  const showCard = !!value && !changing;
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="nostr-sheet-title" onClose=${onClose}>
    <header>
      <h2 id="nostr-sheet-title">Find a Nostr profile</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body">
      ${showCard && html`<div class="group rows">
        ${card && card.profile
          ? html`<div class="link-row"><${ProfileBody} profile=${card.profile} /></div>`
          : html`<div class="link-row"><span class="text"><span class="summary mono">${value}</span>
              ${card && html`<span class="summary">Studio found no profile for this key</span>`}</span></div>`}
        <div class="actions">
          <button class="button" type="button" onClick=${() => setChanging(true)}>Change</button>
          <button class="button destructive" type="button" onClick=${() => choose(null, null)}>Remove</button>
        </div>
      </div>`}
      ${!showCard && html`<form class="form" role="search" onSubmit=${runSearch}>
        <div class="field">
          <label for="nostr-query">npub, NIP-05 address or name</label>
          <div class="inline"><input id="nostr-query" type="search" value=${query} onInput=${(e) => setQuery(e.target.value)} />
            <button class="button" type="submit" disabled=${search.busy || !query.trim()}>${search.busy ? 'Searching…' : 'Search'}</button></div>
        </div>
        ${mine && html`<div class="actions"><button class="button" type="button" onClick=${() => choose(mine, null)}>Use my key</button></div>`}
        <${ErrorNotice} error=${search.error} />
        ${person && html`<div class="group">
          <label class="check"><input type="checkbox" checked=${useName} onChange=${(e) => setUseName(e.target.checked)} />
            <span>Use the profile name</span></label>
          <label class="check"><input type="checkbox" checked=${usePicture} onChange=${(e) => setUsePicture(e.target.checked)} />
            <span>Use the profile picture</span></label>
        </div>`}
        ${found && found.failed.length > 0 && html`<p class="hint" role="status">${found.failed.map((f) => FAILED_TEXT[f] || 'A source did not answer.').join(' ')}</p>`}
        ${found && found.profiles.length === 0 && html`<p class="empty">Studio found no profile.</p>`}
        ${found && found.profiles.length > 0 && html`<div class="group rows">
          ${found.profiles.map((p) => html`<button class="link-row" type="button" key=${p.npub} onClick=${() => choose(p.npub, p)}>
            <${ProfileBody} profile=${p} /></button>`)}
        </div>`}
      </form>`}
    </div>
    <footer>
      <button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>
    </footer>
  </dialog>`;
}

// ---- The replacement of the audio of a track (ADR 0040) ----

// The track with the facts of its new audio file. Only the enclosure, the
// alternative enclosures with an audio type, and the duration change. The
// GUID, the title and the track number stay (ADR 0040 items 4 and 5).
// `readsMetadata` is true after a selection that `needsMetadataRead` marks.
function withNewAudio(item, facts, readsMetadata) {
  const isAudio = (enc) => (enc.media_type || '').startsWith('audio/');
  return {
    ...item,
    enclosure: { url: facts.url, length: facts.size == null ? '' : String(facts.size), media_type: facts.media_type },
    alternate_enclosures: [
      ...item.alternate_enclosures.filter((enc) => !isAudio(enc)),
      ...withCids(facts.alternate_enclosures || []),
    ],
    // A file that only the host holds has no duration yet. The player reads
    // it from the metadata (ADR 0062 item 16), so the old duration goes.
    duration: readsMetadata ? null : facts.duration || item.duration,
  };
}

// ---- The file picker (ADR 0062) ----
// Each image field and each audio field has "Choose…", a sheet with three
// sources, and a drop. `picker.js` has the rules. The code here shows the
// sheet and does the requests. `AlbumEditor` gives the context.

const PickerContext = createContext(null);

// The title of the sheet for each kind of file.
const PICKER_TITLE = {
  image: 'Choose an image',
  audio: 'Choose an audio file',
  lyrics: 'Choose a lyrics file',
};

// The name of the thing in each file field, for the label of its button.
const FIELD_NOUN = {
  cover: 'the cover image',
  'track-image': 'the track image',
  'live-image': 'the image of the show',
  artwork: 'the image',
  'track-audio': 'the audio file',
  recording: 'the recording',
  lyrics: 'the lyrics file',
};

// Does the drag carry a file?
function hasFiles(event) {
  return Array.from((event.dataTransfer && event.dataTransfer.types) || []).includes('Files');
}

// The state and the actions of one file field. `subject` is the channel
// model of a cover field, else the item. It gives the addresses from before.
// `apply(address, facts, source)` puts the new file in the model. A selection
// or an upload calls it, and then the UI sends `delete_unused` for the files
// from before (ADR 0062 items 7, 10, 11 and 17). `source` is `release`,
// `hosts` or `computer`. After a selection that `needsMetadataRead` marks, the
// player reads the duration, and `onDuration(duration)` puts it in the model
// (ADR 0062 item 16).
function useFilePick(field, subject, apply, onDuration) {
  const { feedId, draft, onOpsChanged } = useContext(PickerContext);
  const kind = fieldKind(field);
  const { busy, error, run } = useAction();
  const [open, setOpen] = useState(false);
  const [over, setOver] = useState(false);
  const [dropError, setDropError] = useState(null);
  const [sent, setSent] = useState(0);
  const [size, setSize] = useState(0);
  // The address whose metadata the player reads now, and the result of a
  // read that did not give a duration.
  const [metadataFor, setMetadataFor] = useState(null);
  const [metadataFailed, setMetadataFailed] = useState(false);
  const metadataDone = (duration) => {
    setMetadataFor(null);
    setMetadataFailed(!duration);
    if (duration) onDuration(duration);
  };
  const finish = async (address, facts, source) => {
    const before = replacedAddresses(field, subject, address);
    apply(address, facts, source);
    setMetadataFailed(false);
    setMetadataFor(kind === 'audio' && needsMetadataRead(source) ? address : null);
    if (!(await draft.flush())) return;
    if (before.length === 0) return;
    await api(`/feeds/${feedId}/operations`, { method: 'POST', body: { op: 'delete_unused', urls: before } });
    onOpsChanged();
  };
  // A file of the list. The sheet closes at once.
  const choose = (file, source) => {
    setOpen(false);
    setDropError(null);
    run(() => finish(file.address, selectedFacts(file), source));
  };
  // A file from this computer: the button of the sheet and the drop. A file
  // of the wrong kind is removed again.
  const upload = async (file) => {
    setDropError(null);
    setSent(0);
    setSize(file.size);
    const done = await run(async () => {
      if (!(await draft.flush())) return false;
      const facts = await sendInParts(feedId, file, { replace: kind === 'audio', onProgress: setSent });
      if (!acceptsFile(field, facts)) {
        if (!facts.on_host) {
          api(`/feeds/${feedId}/files?file=${facts.library_file_id}`, { method: 'DELETE' }).catch(() => {});
        }
        throw Object.assign(new Error(wrongKindText(field)), { code: 'wrong_kind' });
      }
      await finish(facts.url, facts, 'computer');
      return true;
    });
    if (done) setOpen(false);
  };
  const drop = (files) => {
    const result = dropResult(field, files);
    if (result.action === 'upload') upload(result.file);
    else if (result.action === 'error') setDropError(result.text);
  };
  // The properties of the `.field` element of the address: the drop state
  // and the drop (ADR 0062 items 20 and 21).
  const rootProps = {
    class: over ? 'field drop-over' : 'field',
    onDragOver: (e) => { if (hasFiles(e) && !busy) { e.preventDefault(); if (!over) setOver(true); } },
    onDragLeave: (e) => { if (!e.currentTarget.contains(e.relatedTarget)) setOver(false); },
    onDrop: (e) => {
      if (!hasFiles(e)) return;
      e.preventDefault();
      setOver(false);
      if (!busy) drop(Array.from(e.dataTransfer.files));
    },
  };
  const errorText = dropError || (error ? importErrorText(error) : null);
  return {
    field, kind, noun: FIELD_NOUN[field], open, busy, over, sent, size, errorText, rootProps, choose, upload,
    metadataFor, metadataFailed, metadataDone,
    openSheet: () => { setDropError(null); setOpen(true); },
    closeSheet: () => setOpen(false),
  };
}

// The list of a picker source. A `null` path gives an empty list.
function usePickerList(path) {
  const empty = { loading: false, error: null, files: [], truncated: false };
  const [state, setState] = useState(empty);
  useEffect(() => {
    if (!path) { setState(empty); return undefined; }
    let current = true;
    setState({ ...empty, loading: true });
    api(path).then(
      (result) => { if (current) setState({ ...empty, files: result.files, truncated: Boolean(result.truncated) }); },
      (error) => { if (current) setState({ ...empty, error }); },
    );
    return () => { current = false; };
  }, [path]);
  return state;
}

// One file of a list. A file with no public address shows its reason, and
// its row is disabled (ADR 0062 item 9).
function PickerFile({ file, kind, onSelect }) {
  const ok = canSelect(file);
  const facts = [file.size > 0 ? fileSize(file.size) : null, file.media_type ? formatLabel(file.media_type) : null, ok ? null : reasonText(file)];
  return html`<button class="link-row picker-file" type="button" disabled=${!ok} onClick=${() => onSelect(file)}>
    ${kind === 'image' && html`<${ImageThumb} url=${ok ? file.address : null} glow="var(--c-track)" className="picker-thumb" always />`}
    <span class="title">${file.name}</span>
    <span class="summary">${facts.filter(Boolean).join(' · ')}</span>
  </button>`;
}

// A folder of files, with the folders in it. A list with no folders is one
// flat level.
function PickerTree({ node, kind, onSelect }) {
  return html`${node.folders.map((folder) => html`<details class="disclosure picker-folder" key=${folder.path}>
      <summary class="link-row"><span class="label">${folder.name}</span></summary>
      <div class="picker-children"><${PickerTree} node=${folder} kind=${kind} onSelect=${onSelect} /></div>
    </details>`)}
    ${node.files.map((file) => html`<${PickerFile} key=${file.path} file=${file} kind=${kind} onSelect=${onSelect} />`)}`;
}

// The files of "This release" or of one host.
function PickerList({ path, kind, byName, onSelect }) {
  const list = usePickerList(path);
  if (list.loading) return html`<p class="hint" role="status">Studio reads the list…</p>`;
  if (list.error) return html`<${ErrorNotice} error=${list.error} />`;
  const files = byName ? list.files.map((f) => ({ ...f, path: f.name })) : list.files;
  if (files.length === 0) return html`<p class="hint">No file of this kind is here.</p>`;
  return html`<div class="group"><${PickerTree} node=${hostTree(files)} kind=${kind} onSelect=${onSelect} /></div>
    ${list.truncated && html`<p class="hint">The list stops at the limit of Studio. The host has more files.</p>`}`;
}

// The sheet of the picker (ADR 0062 item 2). It has the three sources as
// segments.
function FilePicker({ pick }) {
  const { feedId, connectors } = useContext(PickerContext);
  const ref = useSheet(pick.open);
  const [source, setSource] = useState('release');
  const [hostId, setHostId] = useState(null);
  const list = sources(pick.field, connectors);
  const hosts = list[1].hosts;
  const host = hosts.find((h) => h.connectorId === hostId) || hosts[0] || null;
  useEffect(() => { if (pick.open) setSource('release'); }, [pick.open]);
  const inputId = `pick-input-${pick.field}`;
  const title = PICKER_TITLE[pick.kind] || 'Choose a file';
  let path = null;
  if (pick.open && source === 'release') path = `/feeds/${feedId}/picker/release?kind=${pick.kind}`;
  if (pick.open && source === 'hosts' && host) path = `/connectors/${host.connectorId}/picker?kind=${pick.kind}`;
  // A drop on the sheet must not reach the field below it.
  const stop = (e) => { e.preventDefault(); e.stopPropagation(); };
  return html`<dialog ref=${ref} class="sheet" aria-labelledby=${`${inputId}-title`} onClose=${pick.closeSheet}
      onDragOver=${stop} onDrop=${stop}>
    <header>
      <h2 id=${`${inputId}-title`}>${title}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body">
      <div class="actions picker-tabs" role="group" aria-label="Where the file comes from">
        ${list.map((s) => html`<button class="button" type="button" key=${s.id} aria-pressed=${source === s.id}
          onClick=${() => setSource(s.id)}>${s.label}</button>`)}
      </div>
      ${source === 'release' && html`<${PickerList} path=${path} kind=${pick.kind} byName onSelect=${(file) => pick.choose(file, 'release')} />`}
      ${source === 'hosts' && hosts.length === 0 && html`<p class="hint">No host is connected.</p>`}
      ${source === 'hosts' && host && html`<div class="field">
          <label for=${`${inputId}-host`}>Host</label>
          <div class="field-body"><select id=${`${inputId}-host`} value=${host.connectorId} onChange=${(e) => setHostId(Number(e.target.value))}>
            ${hosts.map((h) => html`<option value=${h.connectorId}>${h.name}</option>`)}
          </select></div>
        </div>
        <${PickerList} key=${host.connectorId} path=${path} kind=${pick.kind} onSelect=${(file) => pick.choose(file, 'hosts')} />`}
      ${source === 'computer' && html`<div class="group">
          <div class="import-pick">
            <input id=${inputId} class="visually-hidden" type="file" disabled=${pick.busy} accept=${list[2].accept}
              onChange=${(e) => { const [file] = e.target.files; e.target.value = ''; if (file) pick.upload(file); }} />
            <label for=${inputId} class="button action">${pick.busy ? `Sending… ${progressPercent(pick.sent, pick.size)}%` : 'Select a file'}</label>
          </div>
          <p class="hint">Studio sends the file to the release folder. The host gets the file when you write.</p>
        </div>`}
      ${pick.errorText && html`<p class="field-error" role="alert">${pick.errorText}</p>`}
    </div>
    <footer>
      <button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>
    </footer>
  </dialog>`;
}

// The button "Choose…" with the sheet, the drop hint and the error below
// the field (ADR 0062 items 1, 5 and 21). `children` go before the button.
function ChooseFile({ pick, children }) {
  return html`<div class="file-choose">
      ${children}
      <button class="button action" type="button" id=${`choose-${pick.field}`} disabled=${pick.busy} aria-label=${`Choose… ${pick.noun}`}
        onClick=${pick.openSheet}>${pick.busy ? 'Sending…' : 'Choose…'}</button>
    </div>
    ${pick.over && html`<span class="hint" role="status">Drop the file here to send it.</span>`}
    ${!pick.open && pick.errorText && html`<span class="field-error" role="alert">${pick.errorText}</span>`}
    <${FilePicker} pick=${pick} />`;
}

// An image address field with its preview, "Choose…" and the drop.
function PickedImageField({ field, subject, apply, id, label, address, bad, value, onChange, glow, children }) {
  const pick = useFilePick(field, subject, apply);
  return html`<${TextField} id=${id} label=${label} type="url" address=${address} bad=${bad} value=${value} onChange=${onChange}
      rootProps=${pick.rootProps}>
    <${ChooseFile} pick=${pick}><${ImageThumb} url=${value} glow=${glow} className="field-cover" /><//>
    ${children}
  <//>`;
}

// A file address field with "Choose…" and the drop, and no image. The lyrics
// address uses it.
function PickedFileField({ field, subject, apply, id, label, address, bad, value, onChange }) {
  const pick = useFilePick(field, subject, apply);
  return html`<${TextField} id=${id} label=${label} type="url" address=${address} bad=${bad} value=${value} onChange=${onChange}
      rootProps=${pick.rootProps}><${ChooseFile} pick=${pick} /><//>`;
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
  if (row.state === 'sending') {
    if (row.retrying) return 'A part failed. Trying again…';
    return row.sent ? `Sending… ${progressPercent(row.sent, row.file.size)}%` : 'Sending…';
  }
  if (row.state === 'failed') return importErrorText(row.error);
  return kind === 'audio' ? 'Track ready' : 'Image ready';
}

// One row of the import sheet: the name of the file, its state, the WAV
// notice when it is a WAV file, and its facts when it is ready. An image row
// also has the list of its use (task architecture).

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
    ${row.state === 'sending' && html`<progress class="import-progress" max=${row.file.size} value=${row.sent || 0}
      aria-label=${`Progress of ${row.file.name}`}></progress>`}
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
function ImportSheet({ feedId, open, hasAlbumImage, onClose, onAdd, onAddress }) {
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
          const facts = await sendInParts(feedId, row.file, {
            onProgress: (sent) => updateRow(row.id, { sent }),
            onRetry: (retrying) => updateRow(row.id, { retrying }),
          });
          // The import sheet takes audio files and image files only. The
          // upload route also accepts a lyrics file, so remove it and show
          // the error of an unsupported file (ADR 0064 item 9).
          if (facts.kind !== 'audio' && facts.kind !== 'image') {
            removeFiles(facts);
            throw { code: 'unsupported_media' };
          }
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
      alternate_enclosures: row.facts.alternate_enclosures || [],
      pub_date: null,
      duration: row.facts.duration || null,
      episode: null,
      itunes_episode: null,
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
      <h2 id="import-title">Add tracks</h2>
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
      ${rows.length === 0 && html`<button class="button link" type="button" onClick=${onAddress}>The audio is already online</button>`}
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

// The grant check box of each pending show of the draft (ADR 0054 item 11).
// The check box asks the grant route and takes the new diff from its
// response, so the sheet needs no second request. `diff.grants` gives the
// GUID of each show that has a grant request.
// A host with "My signer" cannot do a granted write (ADR 0058 item 14).
// `signerHost` is true for that host. The box is then disabled, with the
// reason, unless the draft already has the grant. The user can remove it.
function GrantChecks({ feedId, model, diff, apps, busy, signerHost, onDiff, run }) {
  const box = grantBoxState(signerHost);
  const shows = ((model && model.live_items) || []).filter((l) => l.status === 'pending' && l.item && l.item.guid);
  if (shows.length === 0) return null;
  const change = (guid, on) => run(async () => {
    onDiff(await api(`/feeds/${feedId}/grants/${encodeURIComponent(guid)}`, { method: on ? 'PUT' : 'DELETE' }));
  });
  return html`<div class="group grant-checks">
    ${shows.map((l) => html`<div key=${l.item.guid}>
      <label class="check"><input type="checkbox" disabled=${busy || (box.disabled && !diff.grants.includes(l.item.guid))}
        checked=${diff.grants.includes(l.item.guid)} onChange=${(e) => change(l.item.guid, e.target.checked)} />
        <span>Let connected apps start and end this show, and set its live value${shows.length > 1 ? `: ${l.item.title || 'Untitled show'}` : ''}</span></label>
    </div>`)}
    ${box.note && html`<p class="hint">${box.note}</p>`}
    ${apps && apps.length === 0 && html`<p class="hint">No app is connected. Connect an app in Settings.</p>`}
  </div>`;
}

// The name of the event that tells the feed page to load the announcements
// again, such as after a write.
const ANNOUNCE_RELOAD = 'studio-announce-reload';

// A page sends this window event to open the review sheet, for example after
// "Go live". `DraftToolbar` listens to it.
const OPEN_REVIEW = 'studio-open-review';

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
function ReviewSheet({ feedId, feedConnectorId, draft, open, onWritten, onClose, connectors, onOpenHosts, onOpsChanged, onOpsCleared }) {
  const ref = useSheet(open);
  const [diff, setDiff] = useState(null);
  const { busy, error, run } = useAction();
  // The connected apps, for the hint of the grant check box.
  const [apps, setApps] = useState(null);
  useEffect(() => {
    if (open) api('/apps').then(setApps, () => setApps(null));
  }, [open]);
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
  // A read-only feed has no host to go back to, so "Undo" removes the choice.
  const readOnly = isReadOnly({ connector_id: feedConnectorId });
  const setHost = (connectorId) => run(async () => {
    setDiff(await api(`/feeds/${feedId}/feed-host`, { method: 'PUT', body: { connector_id: connectorId } }));
    onOpsChanged();
  });
  const undoMove = () => setHost(readOnly ? null : feedConnectorId);
  // The hosts that can take the file of a read-only feed. The core refuses
  // a host with no public address, and Review shows its error.
  const targets = connectors || [];
  const [target, setTarget] = useState(null);
  const targetId = targets.some((c) => c.id === target) ? target : (targets[0] ? targets[0].id : null);
  const host = reviewHost({ hasHost: draft.hasHost, isReadOnly: readOnly, feedMove: diff && diff.feed_move });
  const choosing = !!diff && host.kind === 'choose';
  const toSign = diff ? eventsToSign(diff) : [];
  // The host of the feed XML after the write: the target of a move, or the
  // host of the feed now.
  const hostId = diff && diff.feed_move ? diff.feed_move.connector_id : feedConnectorId;
  const signerHost = hostUsesSigner((connectors || []).find((c) => c.id === hostId));
  // The signer of the write: the NIP-07 signer of the browser, or else the
  // NIP-46 signer of the workspace (ADR 0056 items 17 and 18).
  const [remote, setRemote] = useState(null);
  const [connecting, setConnecting] = useState(false);
  const hasNip07 = !!window.nostr;
  const loadRemote = () => api('/signer').then(setRemote, () => setRemote({ state: 'none', pubkey: null }));
  useEffect(() => { if (open && !hasNip07) loadRemote(); }, [open]);
  const choice = signerChoice({ hasNip07, remote, events: toSign });
  const needsSigner = choice.kind === 'connect' || choice.kind === 'differs' || choice.kind === 'loading';
  const connected = () => { setConnecting(false); loadRemote(); };
  const write = () => run(async () => {
    let feed;
    let signedEvents = [];
    try {
      if (toSign.length > 0) {
        // The signer of the user signs the planned events before the write.
        // With no signer, nothing goes to the host.
        let signer = window.nostr;
        if (!signer && choice.kind === 'remote') {
          signer = remoteSigner((event) => api('/signer/sign', { method: 'POST', body: { event } }));
        }
        if (!signer) throw new SignError('no_signer', signErrorText('no_signer'));
        setProgress({ done: 0, total: toSign.length });
        signedEvents = await signAll(toSign, signer, (done, total) => setProgress({ done, total }));
      }
      feed = await api(`/feeds/${feedId}/write`, { method: 'POST', body: { diff_id: diff.id, signed_events: signedEvents } });
    } catch (e) {
      // After a conflict, the diff is not against the hosted file. Hide it
      // and its write button.
      if (e.code === 'conflict') setDiff(null);
      // After an expired plan, the new diff has a new plan.
      if (e.code === 'plan_expired') setDiff(await api(`/feeds/${feedId}/diff`));
      // A signer that fails may be gone. Ask for its state again.
      if (!hasNip07 && e.code === 'signer_refused') loadRemote();
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
  const hasWork = diff && (diff.changed || diff.operations.length > 0 || diff.blocked.length > 0 || !!diff.feed_move || diff.grants.length > 0);
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
      ${choosing && html`<div class="group rows">
        <p class="hint">Studio cannot write to the host of this feed. Connect that host, or move the feed to a host that you connected.</p>
        <div class="actions">
          <button class="button action" type="button" disabled=${busy} onClick=${() => { ref.current.close(); onOpenHosts(); }}>Connect the Host of This File</button>
        </div>
        <div class="field"><label for="move-host">Move to a host</label>
          ${targets.length === 0
            ? html`<p class="hint">No host is connected yet. Connect a host in Hosts first.</p>`
            : html`<select id="move-host" value=${targetId} onChange=${(e) => setTarget(Number(e.target.value))}>
                ${targets.map((c) => html`<option value=${c.id}>${c.name}</option>`)}
              </select>`}
        </div>
        <div class="actions">
          <button class="button action" type="button" disabled=${busy || targetId === null} onClick=${() => setHost(targetId)}>Move to This Host</button>
        </div>
      </div>`}
      ${diff && host.kind === 'move' && html`<div class="notice" role="status">
        The file at the address before the move stays and does not change. Apps that read that address continue to get the version from before the move.</div>`}
      ${hasWork && draft.model && draft.hidden && html`<div class="notice" role="status">
        <strong>This feed is hidden from apps.</strong> Apps and directories do not list it after the write.</div>`}
      ${hasWork && html`<${AnnounceLines} diff=${diff} />`}
      ${diff && html`<${GrantChecks} feedId=${feedId} model=${draft.model} diff=${diff} apps=${apps} busy=${busy} signerHost=${signerHost} onDiff=${setDiff} run=${run} />`}
      ${diff && (diff.changed
        ? html`<${DiffView} unified=${diff.unified} />`
        : !hasWork && html`<p>The draft is the same as the hosted file.</p>`)}
      ${waiting && html`<p class="hint" role="status">Wait for each download, then write.</p>`}
      ${diff && toSign.length > 0 && html`<p class="hint" role="status">${signSummary(toSign.length)}</p>`}
      ${diff && choice.kind === 'differs' && html`<div class="notice error" role="alert"><strong>${signErrorText('signer_key_differs')}</strong></div>`}
      ${diff && needsSigner && choice.kind !== 'loading' && !connecting && html`<div class="actions">
        <button class="button action" type="button" disabled=${busy} onClick=${() => setConnecting(true)}>
          ${choice.kind === 'differs' ? 'Connect a different signer' : 'Connect a signer'}</button>
      </div>`}
      ${diff && connecting && html`<${SignerConnect} onConnected=${connected} onCancel=${() => setConnecting(false)} />`}
      <div class="actions">
        <button class="button" type="button" disabled=${busy || draft.status === 'loading'} onClick=${examine}>Examine the changes</button>
      </div>
    </div>
    <footer>
      <button class="button destructive" type="button" disabled=${busy || !draft.changed} onClick=${discard}>Discard the draft</button>
      ${diff && hasWork && !addressesBlocked && !choosing && html`<button class="button primary" type="button" disabled=${busy || waiting || needsSigner} onClick=${write}>
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
export function DraftToolbar({ feedId, feedConnectorId, draft, onWritten, connectors, onOpenHosts, onOpsChanged, onOpsCleared }) {
  // null, 'review' or 'versions': the sheet that is open now.
  const [open, setOpen] = useState(null);
  useEffect(() => {
    const openReview = () => setOpen('review');
    window.addEventListener(OPEN_REVIEW, openReview);
    return () => window.removeEventListener(OPEN_REVIEW, openReview);
  }, []);
  const status = toolbarStatus(draft);
  return html`<div class="draft-toolbar">
    <span class=${`draft-status${status.error ? ' error' : ''}`} role="status">${status.text}</span>
    <button class="button" type="button" onClick=${() => setOpen('versions')}>Versions</button>
    <button class="button primary" type="button" onClick=${() => setOpen('review')}>Review</button>
    <${ReviewSheet} feedId=${feedId} feedConnectorId=${feedConnectorId} draft=${draft} open=${open === 'review'}
      onWritten=${onWritten} onClose=${() => setOpen(null)} connectors=${connectors} onOpenHosts=${onOpenHosts}
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

// The cover field of a channel (ADR 0060). It shows the cover of the model
// and keeps the form of the feed on a change. When the two cover tags have
// different addresses, it shows the second address with an action that
// puts one address in both. A write puts the field address in both
// (ADR 0060 item 8).
function CoverField({ id, model, update, bad }) {
  const conflict = coverConflict(model);
  const change = (address) => update((m) => ({ ...m, image: address ?? '' }));
  return html`<${PickedImageField} field="cover" subject=${model} apply=${change} id=${id} label="Cover image address"
          address=${['image', 'rss_image/url']} bad=${bad} value=${coverOf(model)} onChange=${change}>
          ${conflict && html`<div class="field-note">
            <span>The RSS image tag has a different address: <span class="mono">${conflict}</span>. A write gives it the address of this field.</span>
            <button class="button action" type="button" onClick=${() => change(conflict)}>Use this address</button>
          </div>`}
        <//>`;
}

// The row "Artwork" of a page. It opens the Artwork sheet (ADR 0070 item 9).
function ArtworkRow({ summary, onOpen }) {
  return html`<button class="link-row" type="button" onClick=${onOpen}>
      <span class="label">Artwork</span><span class="summary">${summary}</span>
      <span class="go" aria-hidden="true">›</span>
    </button>`;
}

// The channel fields that the album page and the playlist page share. The
// list `fields` gives the sequence of the page (ADR 0068). The language is
// on the page "Feed settings".
// The row "Nostr profile" of the album, the publisher feed and the playlist:
// the feed key, a `podcast:txt` with `purpose="npub"` (ADR 0074).
function FeedKeyRow({ feedId, model, update, bad }) {
  const key = feedKey(model);
  const path = `txts/${(model.txts || []).findIndex((t) => t.purpose === 'npub')}/value`;
  return html`<${NostrRow} label="Nostr profile" summary=${keySummary('', key)} value=${key} feedId=${feedId}
    bad=${(bad || []).some((b) => b.field === path)}
    onSet=${(npub) => update((m) => withFeedKey(m, npub))} />`;
}

function ChannelFields({ feedId, draft, model, update, bad, artistLabel, onArtwork, fields }) {
  const field = (name) => (value) => update((m) => set(m, name, value));
  const shown = {
    title: () => html`<${TextField} id="ch-title" label="Title" value=${model.title} onChange=${field('title')} />`,
    author: () => html`<${TextField} id="ch-author" label=${artistLabel} hint="The itunes:author of the feed." value=${model.author} onChange=${field('author')} />`,
    nostr: () => html`<${FeedKeyRow} feedId=${feedId} model=${model} update=${update} bad=${bad} />`,
    description: () => html`<${TextArea} id="ch-description" label="Description" value=${model.description} onChange=${field('description')} />`,
    artwork: () => html`<${ArtworkRow} summary=${artworkSummary(model, 'feed')} onOpen=${() => onArtwork({ kind: 'feed' })} />`,
    link: () => html`<${TextField} id="ch-link" label="Website" type="url" address="link" bad=${bad} value=${model.link} onChange=${field('link')} />`,
    listing: () => html`<${ListingField} id="ch-listing" feedId=${feedId} draft=${draft} model=${model} />`,
    explicit: () => html`<${ExplicitField} id="ch-explicit" model=${model} update=${update} />`,
  };
  return fields.map((id) => shown[id]());
}

// Show the sections of a page in the sequence of `list`. `shown` maps each
// section ID to a function that gives the section.
const showSections = (list, shown) => list.map((section) => shown[section.id](section));

// Show the fields or the rows of a section. `shown` maps each ID to a
// function that gives its markup.
const showEach = (ids, shown) => ids.map((id) => shown[id]());

// The explicit switch of a channel (ADR 0064 item 4, ADR 0065). The switch
// shows off when the feed has no `itunes:explicit`, and Studio sets no value
// until the user changes it. A change sends `true` or `false`. The core
// keeps the form of the feed at the draft save.
function ExplicitField({ id, model, update }) {
  const explicit = isExplicit(model.explicit);
  return html`<div class="field">
      <label for=${id}>Explicit</label>
      <div class="field-body">
        <label class="check"><input id=${id} type="checkbox" checked=${explicit}
          onChange=${(e) => update((m) => set(m, 'explicit', e.target.checked ? 'true' : 'false'))} /></label>
        <span class="hint">Apple asks for this tag. Off means clean. Studio adds the tag only when you set the switch.</span>
      </div>
    </div>`;
}

// Does the channel lock say "yes"? This is a display value only.
const isLocked = (locked) => !!locked && (locked.value || '').trim().toLowerCase() === 'yes';

// The summary line of the page "Feed settings": the values that are set.
function settingsSummary(model) {
  const owner = model.owner && (model.owner.name || model.owner.email);
  const parts = [
    model.language && `Language ${model.language}`,
    owner && 'Owner set',
    model.keywords && 'Keywords set',
    isLocked(model.locked) && 'Locked',
    model.managing_editor && 'Editor set',
    model.web_master && 'Webmaster set',
  ].filter(Boolean);
  return parts.length === 0 ? 'No settings' : parts.join(' · ');
}

// The row of a details section that opens the page "Feed settings".
function SettingsRow({ model, onOpen }) {
  return html`<button class="link-row" type="button" onClick=${onOpen}>
      <span class="label">Feed settings</span><span class="summary">${settingsSummary(model)}</span>
      <span class="go" aria-hidden="true">›</span>
    </button>`;
}

// The page "Feed settings": the language, the owner, the keywords, the lock
// and the two editor fields (ADR 0064 items 3 to 7, ADR 0068 item 8). An empty field gives `null`, so that
// Studio writes no empty element. No field refuses a value that is not an
// email address. The core removes the space at the ends of each text.
function SettingsPage({ model, update, album, op3Tracks }) {
  const field = (name) => (value) => update((m) => set(m, name, value));
  const owner = model.owner || { name: null, email: null };
  const setOwner = (name, value) => update((m) => {
    const next = { ...(m.owner || { name: null, email: null }), [name]: value };
    return set(m, 'owner', next.name === null && next.email === null ? null : next);
  });
  const locked = model.locked;
  const setLocked = (on) => update((m) => set(m, 'locked', on
    ? { ...(m.locked || { owner: null }), value: 'yes' }
    : { ...m.locked, value: 'no' }));
  // The UI sends only the value of the switch. The core changes the
  // addresses (ADR 0076 item 12).
  const setOp3 = (on) => update((m) => set(m, 'op3', on));
  const hint = op3Hint(op3Tracks);
  const setLockOwner = (value) => update((m) => set(m, 'locked', { ...m.locked, owner: value }));
  const shown = {
    language: () => html`<section class="page-section"><h2>Language</h2>
      <div class="group">
        <${TextField} id="ch-language" label="Language" hint="For example en or de-AT." value=${model.language} onChange=${field('language')} />
      </div>
    </section>`,
    owner: () => html`<section class="page-section"><h2>Owner</h2>
      <div class="group">
        <${TextField} id="set-owner-name" label="Owner name" value=${owner.name} onChange=${(v) => setOwner('name', v)} />
        <${TextField} id="set-owner-email" label="Owner email" value=${owner.email} onChange=${(v) => setOwner('email', v)} />
      </div>
    </section>`,
    keywords: () => html`<section class="page-section"><h2>Keywords</h2>
      <div class="group">
        <${TextField} id="set-keywords" label="Keywords" hint="Words that apps can search for, with a comma between them."
          value=${model.keywords} onChange=${field('keywords')} />
      </div>
    </section>`,
    locked: () => html`<section class="page-section"><h2>Lock against import</h2>
      <div class="group">
        <div class="field">
          <label for="set-locked">Lock against import</label>
          <div class="field-body">
            <label class="check"><input id="set-locked" type="checkbox" checked=${isLocked(locked)}
              onChange=${(e) => setLocked(e.target.checked)} /></label>
          </div>
        </div>
        ${locked && html`<${TextField} id="set-lock-owner" label="Lock owner email" value=${locked.owner} onChange=${setLockOwner} />`}
      </div>
    </section>`,
    op3: () => html`<section class="page-section"><h2>OP3 statistics</h2>
      <div class="group">
        <div class="field">
          <label for="set-op3">OP3 statistics</label>
          <div class="field-body">
            <label class="check"><input id="set-op3" type="checkbox" checked=${!!model.op3}
              onChange=${(e) => setOp3(e.target.checked)} /></label>
            <span class="hint">OP3 counts the downloads of each track. Studio adds the OP3 address to each audio address.</span>
            ${hint && html`<span class="hint" id="set-op3-count">${hint}</span>`}
            ${model.op3 && model.podcast_guid && html`<div class="actions"><a class="button action" href=${op3Page(model.podcast_guid)} target="_blank" rel="noopener">OP3 statistics</a></div>`}
          </div>
        </div>
      </div>
    </section>`,
    editors: () => html`<section class="page-section"><h2>Editors</h2>
      <div class="group">
        <${TextField} id="set-managing-editor" label="Managing editor" value=${model.managing_editor} onChange=${field('managing_editor')} />
        <${TextField} id="set-web-master" label="Webmaster" value=${model.web_master} onChange=${field('web_master')} />
      </div>
    </section>`,
  };
  return showSections(pageSequence('settings', { album }), shown);
}

// The listing control of a channel (ADR 0063, ADR 0065). One switch hides the
// feed from apps, or shows it again. A block for one platform is in
// "Advanced", which shows only when the feed has such a block. The core
// applies the rule: each change calls a core route and shows the new draft.
function ListingField({ id, feedId, draft, model }) {
  const action = useAction();
  const hidden = draft.hidden;
  const platforms = platformBlocks(model);
  const change = (path, value) => action.run(async () => {
    // Send each edit first, so that the route works on the draft that the
    // user sees.
    if (!(await draft.flush())) return;
    draft.show(await api(`/feeds/${feedId}/listing${path}`, { method: 'POST', body: { hidden: value } }));
  });
  return html`<div class="field">
      <label for=${`${id}-hidden`}>Hide from apps</label>
      <div class="field-body">
        <label class="check"><input id=${`${id}-hidden`} type="checkbox" checked=${hidden} disabled=${action.busy}
          onChange=${(e) => change('', e.target.checked)} /></label>
        <span class="hint">Apps and directories do not list a hidden feed. The feed stays at its address.</span>
        ${action.error && html`<${ErrorNotice} error=${action.error} />`}
      </div>
    </div>
    ${platforms.length > 0 && html`<details class="disclosure">
      <summary class="link-row"><span class="label">Advanced</span>
        <span class="summary">Rules for one platform: ${platforms.map((row) => row.id).join(', ')}</span></summary>
      <div class="form">
        ${platforms.map((row) => html`<label class="check" key=${row.block._cid}>
          <input type="checkbox" checked=${row.hidden} disabled=${action.busy}
            onChange=${(e) => change(`/${encodeURIComponent(row.id)}`, e.target.checked)} />
          <span>Hide from <span class="mono">${row.id}</span></span></label>`)}
      </div>
    </details>`}`;
}

// The rows "Value routes" and "Credits and links" of the section Details.
// The album page and the publisher page both show them (ADR 0075 item 2).
// `openPage({ page })` opens the page of a row.
function detailRows(rows, model, openPage) {
  return showEach(rows, {
    recipients: () => html`<button class="link-row" type="button" onClick=${() => openPage({ page: 'recipients' })}>
      <span class="label">Value routes</span><span class="summary">${recipientSummary(model.values)}</span>
      ${model.values.length > 0 && html`<${ShareStack} recipients=${model.values[0].recipients} focusable=${false} />`}
      <span class="go" aria-hidden="true">›</span>
    </button>`,
    credits: () => html`<button class="link-row" type="button" onClick=${() => openPage({ page: 'credits' })}>
      <span class="label">Credits and links</span><span class="summary">${creditSummary(model)}</span>
      <span class="go" aria-hidden="true">›</span>
    </button>`,
  });
}

function AlbumPage({ feed, feeds, model, update, bad, draft, go, onAdding, onImporting, onRemoved, onListChanged, onArtwork }) {
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
  const shown = {
    album: (s) => html`<section class="page-section"><h2>Album</h2>
      <div class="group">
        <${ChannelFields} feedId=${feed.id} draft=${draft} model=${model} update=${update} bad=${bad} artistLabel="Artist" onArtwork=${onArtwork} fields=${s.fields} />
      </div>
    </section>`,
    details: (s) => html`<section class="page-section"><h2>Album details</h2>
      <div class="group">
        ${detailRows(s.rows, model, openPage)}
      </div>
    </section>`,
    publishers: () => html`<section class="page-section"><h2>Artists and labels</h2>
      <div class="group">
        <${ChangedNotices} changed=${changed} go=${go} />
        ${rows.length === 0 && html`<p class="hint">No artist or label yet.</p>`}
        ${rows.map((row) => html`<div class="release-row" key=${row.entry._cid}>
          <span class="title">${row.entry.title || row.entry.feed_url}</span>
          <span class="sub">${publisherLine(row.entry, row.match, row.index)}</span>
          <div class="actions">
            <button class="button" type="button" disabled=${row.index === 0 || linkAction.busy}
              aria-label=${`Move ${row.entry.title || 'this feed'} up`}
              onClick=${() => movePublisher(row.index, -1)}>Move up</button>
            <button class="button" type="button" disabled=${row.index === rows.length - 1 || linkAction.busy}
              aria-label=${`Move ${row.entry.title || 'this feed'} down`}
              onClick=${() => movePublisher(row.index, 1)}>Move down</button>
            <button class="button destructive" type="button" disabled=${linkAction.busy}
              aria-label=${`Remove ${row.entry.title || 'this feed'}`}
              onClick=${() => removeRow(row)}>Remove</button>
          </div>
        </div>`)}
        ${candidates.length > 0 && html`<div class="field">
          <label for="album-publisher">Artist or label</label>
          <div class="field-body">
            <select id="album-publisher" value=${chosen} onChange=${(e) => setChosen(e.target.value)}>
              <option value="">Select an artist or a label</option>
              ${candidates.map((f) => html`<option value=${f.id}>${f.title || f.path || 'Untitled feed'}</option>`)}
            </select>
          </div>
        </div>
        <div class="actions"><button class="button action" type="button" disabled=${!chosen || linkAction.busy}
          onClick=${addPublisher}>Add the Feed</button></div>`}
        ${candidates.length === 0 && html`<p class="hint">Studio has no other artist feed or label feed in the collection to add.</p>`}
        <${ErrorNotice} error=${publisherError(linkAction.error, 'Open that feed and select its role first.')} />
      </div>
    </section>`,
    feed: (s) => html`<section class="page-section"><h2>Feed</h2>
      <div class="group">
        ${showEach(s.rows, {
          settings: () => html`<${SettingsRow} model=${model} onOpen=${() => openPage({ page: 'settings' })} />`,
          facts: () => html`<${FeedFacts} feed=${feed} onRemoved=${onRemoved} />`,
        })}
      </div>
    </section>`,
    tracks: () => html`<section class="page-section"><h2>Tracks</h2>
      <div class="group">
        ${model.items.map((item, i) => html`<div key=${trackKey(item)} class="order-row">
          <${PositionSelect} id=${`track-position-${i}`} index=${i} count=${model.items.length} name=${item.title || 'Untitled track'}
            onMove=${(to) => update((m) => set(m, 'items', moveTo(m.items, i, to)))} />
          <button class="link-row" type="button" onClick=${() => openPage({ page: 'track', track: trackKey(item) })}>
            <span class="title">${item.title || 'Untitled track'}</span>
            ${item.duration && html`<span class="dur">${item.duration}</span>`}
            <span class="go" aria-hidden="true">›</span>
          </button>
        </div>`)}
        <button class="link-row add" type="button" onClick=${() => onImporting(true)}>
          <span class="num">＋</span><span class="title">Add tracks</span>
        </button>
      </div>
      <p class="note">Studio sets the track numbers (podcast:episode and itunes:episode) from this sequence. It does not change the publication dates.</p>
    </section>`,
  };
  return showSections(pageSequence('album'), shown);
}

// The playlist page (ADR 0053 item 4): the channel fields and the feed facts.
function PlaylistPage({ feed, draft, model, update, bad, go, onRemoved, onArtwork }) {
  const shown = {
    channel: (s) => html`<section class="page-section"><h2>Playlist</h2>
      <div class="group">
        <${ChannelFields} feedId=${feed.id} draft=${draft} model=${model} update=${update} bad=${bad} artistLabel="Author" onArtwork=${onArtwork} fields=${s.fields} />
      </div>
    </section>`,
    feed: (s) => html`<section class="page-section"><h2>Feed</h2>
      <div class="group">
        ${showEach(s.rows, {
          settings: () => html`<${SettingsRow} model=${model} onOpen=${() => go({ name: 'feed', id: feed.id, page: 'settings' })} />`,
          facts: () => html`<${FeedFacts} feed=${feed} onRemoved=${onRemoved} />`,
        })}
      </div>
    </section>`,
  };
  return showSections(pageSequence('playlist'), shown);
}

// The Tracks page of a playlist: one row for each remote item, with the
// buttons to move it and to remove it, then "Add a track" (ADR 0053 item 4).
function PlaylistTracksPage({ feed, model, update }) {
  const [picking, setPicking] = useState(false);
  const items = model.remote_items || [];
  // Each change applies to the current model, as `update` gives it.
  const change = (step) => update((m) => set(m, 'remote_items', step(m.remote_items || [])));
  return html`<section class="page-section"><h2>Tracks</h2>
      <p class="hint" role="status">${tracksSummary(items)}</p>
      <div class="group">
        ${items.map((item, i) => {
          const row = trackRow(item);
          return html`<div class="release-row" key=${item._cid}>
            <div class="order-line">
              <${PositionSelect} id=${`playlist-position-${i}`} index=${i} count=${items.length} name=${row.title}
                onMove=${(to) => change((list) => moveTo(list, i, to))} />
              <span class="title">${row.title}</span>
            </div>
            <span class="sub mono">${row.sub}</span>
            <div class="actions">
              <button class="button destructive" type="button"
                aria-label=${`Remove ${row.title}`} onClick=${() => change((list) => removeTrack(list, i))}>Remove</button>
            </div>
          </div>`;
        })}
        <div class="actions"><button class="button action" type="button" onClick=${() => setPicking(true)}>Add a track</button></div>
      </div>
      <${PlaylistPickerSheet} feedId=${feed.id} open=${picking} remoteItems=${items}
        onClose=${() => setPicking(false)} onAdd=${(pick) => change((list) => addTrack(list, withCids([remoteItemFromPick(pick)])[0]))} />
    </section>`;
}

// One track row of the picker: the title and "Add", or "In the playlist".
function PickerTrack({ pick, remoteItems, onAdd }) {
  const taken = hasTrack(remoteItems, pick);
  return html`<div class="link-row playlist-pick">
    <span class="text"><span class="title">${pick.title || 'Untitled track'}</span></span>
    ${taken
      ? html`<span class="summary">In the playlist</span>`
      : html`<button class="button action" type="button" aria-label=${`Add ${pick.title || 'this track'}`}
          onClick=${() => onAdd(pick)}>Add</button>`}
  </div>`;
}

// The picker sheet of a playlist (ADR 0053 item 4): a search of musicindex
// and the releases of the collection. "Add" keeps the sheet open.
function PlaylistPickerSheet({ open, remoteItems, onAdd, onClose }) {
  const ref = useSheet(open);
  const [tab, setTab] = useState('index');
  const [query, setQuery] = useState('');
  const [hits, setHits] = useState(null);
  const [opened, setOpened] = useState(null);
  const [mine, setMine] = useState(null);
  const search = useAction();
  const load = useAction();
  const own = useAction();
  useEffect(() => {
    if (!open) return;
    setTab('index');
    setQuery('');
    setHits(null);
    setOpened(null);
    setMine(null);
  }, [open]);
  useEffect(() => {
    if (open && tab === 'mine' && mine === null) own.run(async () => setMine(await api('/playlist/tracks')));
  }, [open, tab]);
  const runSearch = (event) => {
    event.preventDefault();
    if (!query.trim()) return;
    search.run(async () => {
      setOpened(null);
      setHits(await api(`/search?q=${encodeURIComponent(query.trim())}`));
    });
  };
  const openFeed = (guid) => {
    if (opened && opened.guid === guid) { setOpened(null); return; }
    load.run(async () => setOpened({ guid, ...(await api(`/musicindex/feeds/${encodeURIComponent(guid)}/tracks`)) }));
  };
  // The tracks of the collection, grouped by the title of their feed.
  const groups = [];
  for (const t of mine || []) {
    const title = t.feed_title || 'Untitled feed';
    let group = groups.find((g) => g.key === t.feed_guid);
    if (!group) { group = { key: t.feed_guid, title, tracks: [] }; groups.push(group); }
    group.tracks.push({ feed_guid: t.feed_guid, item_guid: t.item_guid, feed_url: t.feed_url, title: t.title });
  }
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="playlist-picker-title" onClose=${onClose}>
    <header>
      <h2 id="playlist-picker-title">Add tracks</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body">
      <div class="actions playlist-tabs" role="group" aria-label="Where the tracks come from">
        <button class="button" type="button" aria-pressed=${tab === 'index'} onClick=${() => setTab('index')}>Search musicindex</button>
        <button class="button" type="button" aria-pressed=${tab === 'mine'} onClick=${() => setTab('mine')}>My releases</button>
      </div>
      ${tab === 'index' && html`<section class="page-section">
        <form class="form" onSubmit=${runSearch} role="search">
          <div class="field">
            <label for="playlist-search">Search musicindex</label>
            <div class="inline">
              <input id="playlist-search" type="search" value=${query} placeholder="Artist or feed name"
                onInput=${(e) => setQuery(e.target.value)} />
              <button class="button" type="submit" disabled=${search.busy || !query.trim()}>
                ${search.busy ? 'Searching…' : 'Search'}</button>
            </div>
          </div>
        </form>
        <${ErrorNotice} error=${search.error} />
        <${ErrorNotice} error=${load.error} />
        ${hits && hits.length === 0 && html`<p class="hint">musicindex found no feed with that name.</p>`}
        ${hits && hits.length > 0 && html`<div class="group">
          ${hits.map((h) => html`<div key=${h.feed_guid}>
            <button class="link-row playlist-feed" type="button" disabled=${load.busy} aria-expanded=${opened && opened.guid === h.feed_guid ? 'true' : 'false'}
              onClick=${() => openFeed(h.feed_guid)}>
              <span class="dot feed"></span><span class="title">${h.title || h.feed_guid}</span>
              <span class=${opened && opened.guid === h.feed_guid ? 'go open' : 'go'} aria-hidden="true">›</span>
            </button>
            ${opened && opened.guid === h.feed_guid && html`<div>
              ${opened.tracks.length === 0 && html`<p class="hint">This feed has no tracks.</p>`}
              ${opened.tracks.map((t) => html`<${PickerTrack} key=${t.track_guid} pick=${pickFromIndex(opened, t)}
                remoteItems=${remoteItems} onAdd=${onAdd} />`)}
            </div>`}
          </div>`)}
        </div>`}
      </section>`}
      ${tab === 'mine' && html`<section class="page-section">
        <${ErrorNotice} error=${own.error} />
        ${mine === null && !own.error && html`<p class="hint" role="status">Loading your releases…</p>`}
        ${mine && mine.length === 0 && html`<p class="hint">Your collection has no release with tracks.</p>`}
        ${groups.map((g) => html`<div class="group" key=${g.key}>
          <h3 class="pick-head">${g.title}</h3>
          ${g.tracks.map((pick) => html`<${PickerTrack} key=${pick.item_guid} pick=${pick} remoteItems=${remoteItems} onAdd=${onAdd} />`)}
        </div>`)}
      </section>`}
    </div>
    <footer class="pick-footer">
      <div class="pick-footer-row"><span class="spacer"></span>
        <button class="button primary" type="button" onClick=${() => ref.current.close()}>Done</button></div>
    </footer>
  </dialog>`;
}

// ---- The artwork list (ADR 0064 items 10 to 12) ----

const BLANK_IMAGE = { href: '', alt: null, aspect_ratio: null, width: null, height: null, media_type: null, purpose: null };

// The image at an address, read through `/images` (ADR 0025). It gives the
// size of the image and the type of the response. A failure gives `null`.
async function readImageFacts(url) {
  try {
    const blob = await api(`/images?url=${encodeURIComponent(url)}`, { raw: true });
    const src = await new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(reader.result);
      reader.onerror = reject;
      reader.readAsDataURL(blob);
    });
    const size = await new Promise((resolve, reject) => {
      const image = new Image();
      image.onload = () => resolve({ width: image.naturalWidth, height: image.naturalHeight });
      image.onerror = reject;
      image.src = src;
    });
    if (!(size.width > 0 && size.height > 0)) return null;
    return imageFacts(size.width, size.height, blob.type);
  } catch (_) {
    return null;
  }
}

// One image of an artwork list. `edit(change)` changes the image list of the
// parent, also after a long load. `address` is the model path of the entry.
function ArtworkEntry({ id, list, index, onChange, edit, bad, address }) {
  const t = list[index];
  const same = (x) => x._cid === t._cid;
  const change = (name) => (value) => edit((all) => all.map((x) => (same(x) ? set(x, name, value) : x)));
  const apply = (href) => change('href')(href);
  // The facts fill only after a new address. An entry that Studio read from
  // the feed keeps its values.
  const handled = useRef(t.href);
  const [unreadable, setUnreadable] = useState(false);
  useEffect(() => {
    const href = t.href;
    if (href === handled.current) return undefined;
    if (!href) {
      handled.current = href;
      setUnreadable(false);
      return undefined;
    }
    let current = true;
    // The wait lets the user finish a typed address.
    const timer = setTimeout(async () => {
      handled.current = href;
      const facts = await readImageFacts(href);
      if (!current) return;
      setUnreadable(facts === null);
      // A fact with no value does not change the value of the entry.
      const found = facts && Object.fromEntries(Object.entries(facts).filter(([, v]) => v !== null));
      if (found) edit((all) => all.map((x) => (same(x) && x.href === href ? { ...x, ...found } : x)));
    }, 400);
    return () => { current = false; clearTimeout(timer); };
  }, [t.href]);
  // The purpose shows as RSS tokens in code form, also in the summary line
  // (musicindex ADR 0033 R-WORD-4 item 2).
  const tokens = purposeTokens(t.purpose);
  const unknown = tokens.filter((token) => !PURPOSES.includes(token));
  const size = t.width && t.height ? `${t.width} × ${t.height}` : '';
  const summary = html`<span class="title">${tokens.length > 0 ? html`<code>${tokens.join(' ')}</code>` : 'No purpose'}</span>
    <span class="sub">${size}</span>`;
  return html`<${EntryRow} group=${`${id}-images`} cid=${t._cid} summary=${summary}>
    <${PickedImageField} field="artwork" subject=${t} apply=${apply} id=${`${id}-href`} label="Image address"
      address=${`${address}/href`} bad=${bad} value=${t.href} onChange=${change('href')} glow="var(--c-feed)">
      ${unreadable && html`<span class="hint">Studio cannot read this image. Type the dimensions.</span>`}
    <//>
    <${TextField} id=${`${id}-alt`} label="Description" hint="A short text for people who cannot see the image." value=${t.alt} onChange=${change('alt')} />
    <div class="field">
      <label for=${`${id}-purpose-${PURPOSES[0]}`}>Purpose</label>
      <div class="field-body">
        ${PURPOSES.map((token) => html`<label class="check"><input id=${`${id}-purpose-${token}`} type="checkbox" checked=${tokens.includes(token)}
          onChange=${(e) => change('purpose')(withPurpose(t.purpose, token, e.target.checked))} /><code>${token}</code></label>`)}
        ${unknown.length > 0 && html`<p class="hint">Other purpose in the feed: <code>${unknown.join(' ')}</code>. Studio keeps it.</p>`}
      </div>
    </div>
    <div class="grid2">
      <${TextField} id=${`${id}-width`} label="Width" inputmode="numeric" value=${t.width} onChange=${change('width')} />
      <${TextField} id=${`${id}-height`} label="Height" inputmode="numeric" value=${t.height} onChange=${change('height')} />
      <${TextField} id=${`${id}-ratio`} label="Ratio" value=${t.aspect_ratio} onChange=${change('aspect_ratio')} />
      <${TextField} id=${`${id}-type`} label="Type" hint="For example image/png." value=${t.media_type} onChange=${change('media_type')} />
    </div>
    <${EntryActions} list=${list} index=${index} name="this image" onChange=${onChange} />
  <//>`;
}

// The `podcast:image` entries of one parent, the album or a track. `edit`
// changes the image list of the parent. `path` is the model path of the list.
function ArtworkList({ id, images, edit, bad, path }) {
  return html`${images.length === 0 && html`<p class="hint">No more images yet.</p>`}
    ${images.map((t, i) => html`<${ArtworkEntry} key=${t._cid} id=${`${id}-image-${t._cid}`} list=${images} index=${i}
      onChange=${(list) => edit(() => list)} edit=${edit} bad=${bad} address=${`${path}/${i}`} />`)}
    <button class="link-row add" type="button" onClick=${() => edit((list) => addOpen(list, BLANK_IMAGE))}>
      <span class="num">＋</span><span class="title">Add an image</span>
    </button>`;
}

// The Artwork sheet of a feed or a track (ADR 0070 items 5 to 7). It holds
// the image field, the note about the RSS image, the button "Use the Album
// Cover" and the list "More artwork". Each change goes into the draft at
// once, as a change of a field on a page. `artwork` is `{ kind: 'feed' }` or
// `{ kind: 'track', key }`. `trackIndex` is the index of the track, or -1.
// A playlist and a publisher feed have no list of other images.
function ArtworkSheet({ feed, model, update, bad, artwork, trackIndex, onClose }) {
  const ref = useSheet(true);
  const isTrackSheet = artwork.kind === 'track';
  const item = isTrackSheet ? model.items[trackIndex] : null;
  const path = `items/${trackIndex}/`;
  const editItem = (edit) => update((m) => set(m, 'items', m.items.map((i) => (i._cid === item._cid ? edit(i) : i))));
  const title = isTrackSheet ? item.title || 'Untitled track' : model.title || feed.title || 'Untitled feed';
  const hasList = isTrackSheet || (!isPublisher(feed) && !isPlaylist(feed));
  const preview = isTrackSheet ? item.image || coverOf(model) : coverOf(model);
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="artwork-title" onClose=${onClose}>
    <header>
      <h2 id="artwork-title">Artwork · ${title}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body">
      <${ImageThumb} url=${preview} glow=${isTrackSheet ? 'var(--c-track)' : 'var(--c-feed)'} className="artwork-preview" always />
      ${isTrackSheet && !item.image && html`<p class="hint">Uses the album cover</p>`}
      <div class="group rows">
        ${isTrackSheet
          ? html`<${PickedImageField} field="track-image" subject=${item} apply=${(address) => editItem((i) => set(i, 'image', address))}
              id="artwork-image" label="Track image address" address=${`${path}image`} bad=${bad} value=${item.image}
              onChange=${(address) => editItem((i) => set(i, 'image', address))} glow="var(--c-track)" />`
          : html`<${CoverField} id="artwork-image" model=${model} update=${update} bad=${bad} />`}
      </div>
      ${isTrackSheet && item.image && html`<div class="actions">
        <button class="button" type="button" onClick=${() => editItem((i) => set(i, 'image', null))}>Use the Album Cover</button>
      </div>`}
      ${hasList && html`<section class="page-section"><h2>More artwork</h2>
        <div class="group rows">${isTrackSheet
          ? html`<${ArtworkList} id="artwork-track" images=${item.images} bad=${bad} path=${`${path}images`}
              edit=${(change) => editItem((i) => set(i, 'images', change(i.images)))} />`
          : html`<${ArtworkList} id="artwork-album" images=${model.images} bad=${bad} path="images"
              edit=${(change) => update((m) => set(m, 'images', change(m.images)))} />`}
        </div>
      </section>`}
    </div>
    <footer>
      <button class="button primary" type="button" onClick=${() => ref.current.close()}>Done</button>
    </footer>
  </dialog>`;
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

// The media types of a lyrics file. A feed can have another value, and the
// menu then offers it too.
const LYRICS_TYPES = ['text/plain', 'text/html', 'text/vtt', 'application/json', 'application/srt'];

// The file name of an address, for the summary line. This is a display value.
const addressName = (url) => (url || '').split(/[?#]/)[0].split('/').filter(Boolean).pop() || '';

// One lyrics file of a track. `onChange` sets the list. `edit(change)` changes the transcript list of
// the item, also after a long upload. `address` is the model path of the entry.
function LyricsEntry({ id, list, index, onChange, edit, bad, address }) {
  const t = list[index];
  const same = (x) => x._cid === t._cid;
  const change = (name) => (value) => edit((list) => list.map((x) => (same(x) ? set(x, name, value) : x)));
  const apply = (url, facts) => edit((list) => list.map((x) => (same(x) ? { ...x, url, media_type: (facts && facts.media_type) || x.media_type } : x)));
  const types = LYRICS_TYPES.includes(t.media_type) || !t.media_type ? LYRICS_TYPES : [...LYRICS_TYPES, t.media_type];
  const summary = html`<span class="title">${addressName(t.url) || 'New lyrics file'}</span>
    <span class="sub">${[t.media_type, t.language].filter(Boolean).join(' · ')}</span>`;
  return html`<${EntryRow} group=${`${id}-lyrics`} cid=${t._cid} summary=${summary}>
    <${PickedFileField} field="lyrics" subject=${t} apply=${apply} id=${`${id}-url`} label="Lyrics file address"
      address=${`${address}/url`} bad=${bad} value=${t.url} onChange=${change('url')} />
    <div class="field">
      <label for=${`${id}-type`}>Type</label>
      <div class="field-body"><select id=${`${id}-type`} value=${t.media_type || 'text/plain'}
        onChange=${(e) => change('media_type')(e.target.value)}>
        ${types.map((type) => html`<option value=${type}>${type}</option>`)}
      </select></div>
    </div>
    <${TextField} id=${`${id}-language`} label="Language" hint="For example en or de-AT." value=${t.language} onChange=${change('language')} />
    <div class="field">
      <label for=${`${id}-captions`}>Captions</label>
      <div class="field-body"><label class="check"><input id=${`${id}-captions`} type="checkbox" checked=${t.rel === 'captions'}
        onChange=${(e) => change('rel')(e.target.checked ? 'captions' : null)} />
        <span>These are captions</span></label>
        <span class="hint">Apps show the lines at their times while the track plays. Use this for a timed file: .srt or .vtt.</span></div>
    </div>
    <${EntryActions} list=${list} index=${index} name=${addressName(t.url) || 'this lyrics file'} onChange=${onChange} />
  <//>`;
}

// The track page: the track before it and the track after it, then its
// fields, its audio file, its recipients and its other facts (task 014c
// step 3).
function TrackPage({ feed, model, update, bad, trackIndex, go, onArtwork }) {
  const items = model.items;
  const item = items[trackIndex];
  const id = `track-${item._cid}`;
  const path = `items/${trackIndex}/`;
  const change = (name) => (value) => update((m) => set(m, 'items', setAt(m.items, trackIndex, set(item, name, value))));
  const changeItems = (nextItems) => update((m) => set(m, 'items', nextItems));
  // A change that works on the current model, also after a long upload.
  const editItem = (edit) => update((m) => set(m, 'items', m.items.map((i) => (i._cid === item._cid ? edit(i) : i))));
  const pickAudio = useFilePick(
    'track-audio',
    item,
    (_address, facts, source) => editItem((i) => withNewAudio(i, facts, needsMetadataRead(source))),
    (duration) => editItem((i) => set(i, 'duration', duration)),
  );
  // The season is `null` when both fields are empty.
  // The season number is a text in the model, so an empty number is an
  // empty text. The season is `null` when both fields are empty.
  const changeSeason = (name) => (value) => {
    const next = { value: '', name: null, ...item.season, [name]: value };
    const empty = !next.value && next.name === null;
    change('season')(empty ? null : { ...next, value: next.value || '' });
  };
  const editTranscripts = (edit) => editItem((i) => set(i, 'transcripts', edit(i.transcripts)));
  const blankLyrics = { url: '', media_type: 'text/plain', language: null, rel: null };
  const ownCredits = !usesAlbumCredits(item);
  // A change to the credits. The copy gets its client IDs here. An album
  // with no persons gives one empty person as a start value.
  const useTrackCredits = () => editItem((i) => {
    const next = withTrackCredits(i, model.persons);
    return set(next, 'persons', next.persons.length > 0 ? withCids(next.persons) : addOpen([], BLANK_PERSON));
  });
  const changeCredits = (e) => {
    if (e.target.value === 'track') { useTrackCredits(); return; }
    if (confirm('Remove the credits of this track? The track then uses the album credits.')) {
      editItem(withAlbumCredits);
    } else {
      e.target.value = 'track';
    }
  };
  const openTrack = (i) => go({ name: 'feed', id: feed.id, page: 'track', track: trackKey(items[i]) });
  const removeTrack = () => {
    changeItems(items.filter((_, i) => i !== trackIndex));
    go({ name: 'feed', id: feed.id });
  };
  // No wrapper: the head and each section are direct rows of .page, with
  // the gap of .page between them (review of task 014c, defect 2).
  const shown = {
    track: (s) => html`<section class="page-section"><h2>Track</h2>
      <div class="group">
        ${showEach(s.fields, {
          title: () => html`<${TextField} id=${`${id}-title`} label="Title" value=${item.title} onChange=${change('title')} />`,
          audio: () => html`<${Enclosure} id=${id} path=${path} bad=${bad} enclosure=${item.enclosure} onChange=${change('enclosure')} pick=${pickAudio} />`,
          duration: () => html`<${TextField} id=${`${id}-duration`} label="Duration" hint=${pickAudio.metadataFailed && !item.duration ? DURATION_NOTICE : 'Seconds, or HH:MM:SS.'}
            value=${item.duration} onChange=${change('duration')} />`,
          alternates: () => html`<${AlternateEnclosures} list=${item.alternate_enclosures} onChange=${change('alternate_enclosures')} />`,
          artwork: () => html`<${ArtworkRow} summary=${artworkSummary(item, 'track')} onOpen=${() => onArtwork({ kind: 'track', key: trackKey(item) })} />`,
          description: () => html`<${TextArea} id=${`${id}-description`} label="Description" value=${item.description} onChange=${change('description')} />`,
        })}
      </div>
    </section>`,
    credits: () => html`<section class="page-section"><h2>Credits</h2>
      <div class="group">
        <div class="field">
          <label for=${`${id}-credits`}>Credits</label>
          <div class="field-body"><select id=${`${id}-credits`} value=${ownCredits ? 'track' : 'album'} onChange=${changeCredits}>
            <option value="album">Use the album credits</option>
            <option value="track">Credits for this track</option>
          </select></div>
        </div>
        ${ownCredits
          ? html`<${Persons} prefix=${id} persons=${item.persons} bad=${bad} path=${`${path}persons`}
              onChange=${(persons) => editItem((i) => set(i, 'persons', persons))} />`
          : html`<p class="hint">This track uses the album credits.</p>`}
      </div>
    </section>`,
    lyrics: () => html`<section class="page-section"><h2>Lyrics</h2>
      <div class="group">
        ${item.transcripts.length === 0 && html`<p class="hint">No lyrics files yet.</p>`}
        ${item.transcripts.map((t, i) => html`<${LyricsEntry} key=${t._cid} id=${`${id}-lyrics-${t._cid}`} list=${item.transcripts} index=${i}
          onChange=${(list) => editTranscripts(() => list)} edit=${editTranscripts} bad=${bad} address=${`${path}transcripts/${i}`} />`)}
        <button class="link-row add" type="button" onClick=${() => editTranscripts((list) => addOpen(list, blankLyrics))}>
          <span class="num">＋</span><span class="title">Add a lyrics file</span>
        </button>
      </div>
    </section>`,
    payments: () => html`<${TrackPayments} id=${id} item=${item} albumBlocks=${model.values} onChange=${change('values')} />`,
    more: (s) => html`<section class="page-section"><h2>More</h2>
      <div class="group">
        ${showEach(s.rows, {
          pub_date: () => html`<${TextField} id=${`${id}-date`} label="Publication date"
            hint="For example Thu, 22 Oct 2026 12:00:00 GMT. Some apps sort the tracks by this date."
            value=${item.pub_date} onChange=${change('pub_date')} />`,
          explicit: () => html`<${ExplicitField} id=${`${id}-explicit`} model=${item} update=${editItem} />`,
          identity: () => html`<details class="disclosure">
          <summary class="link-row"><span class="label">Identity</span>
            <span class="summary">Track ${item.episode || item.itunes_episode || '—'}${item.season ? ` · Disc ${item.season.value || item.season.name}` : ''}</span></summary>
          <div class="form">
            <dl class="facts">
              <dt>Track number</dt><dd>${item.episode || item.itunes_episode || '—'}</dd>
              <dt>Track GUID</dt><dd class="mono">${item.guid || 'Studio gives one when it saves'}</dd>
            </dl>
            <details class="disclosure">
              <summary class="link-row"><span class="label">Disc <span class="badge-experimental">Experimental</span></span>
                <span class="summary">${item.season ? item.season.value || item.season.name : 'Not set'}</span></summary>
              <div class="form">
                <p class="hint">Studio writes this as <code>podcast:season</code>. The Podcasting 2.0 namespace defines it for the seasons of a podcast, so apps can show it as a season.</p>
                <div class="grid2">
                  <${TextField} id=${`${id}-season-number`} label="Disc number" inputmode="numeric" value=${item.season && item.season.value} onChange=${changeSeason('value')} />
                  <${TextField} id=${`${id}-season-name`} label="Disc name" value=${item.season && item.season.name} onChange=${changeSeason('name')} />
                </div>
              </div>
            </details>
          </div>
        </details>`,
        })}
      </div>
    </section>`,
  };
  return html`<header class="head">
      <${ImageThumb} url=${item.image || coverOf(model)} glow="var(--c-track)" className="cover" always
        onOpen=${() => onArtwork({ kind: 'track', key: trackKey(item) })} label=${`Artwork of ${item.title || 'Untitled track'}`} />
      <div class="text"><span class="eyebrow"><span class="dot track"></span>Track ${trackIndex + 1} of ${items.length}</span>
        <h1>${item.title || 'Untitled track'}</h1></div>
      <div class="pager">
        <button class="button icon-button" type="button" aria-label="The track before this one"
          disabled=${trackIndex === 0} onClick=${() => openTrack(trackIndex - 1)}>‹</button>
        <button class="button icon-button" type="button" aria-label="The track after this one"
          disabled=${trackIndex === items.length - 1} onClick=${() => openTrack(trackIndex + 1)}>›</button>
      </div>
    </header>

    ${showSections(pageSequence('track'), shown)}

    <div class="actions">
      <label class="position-field" for="track-position">Position
        <${PositionSelect} id="track-position" index=${trackIndex} count=${items.length} name=${item.title || 'Untitled track'}
          onMove=${(to) => changeItems(moveTo(items, trackIndex, to))} /></label>
      <button class="button destructive" type="button" onClick=${removeTrack}>Remove the track</button>
    </div>`;
}

// ---- The live item page (task 033b, ADR 0054) ----

// The start time of a live item in the language of the user, for the
// summary line.
const timeText = (text) => {
  const time = new Date(text);
  return text && !Number.isNaN(time.getTime()) ? time.toLocaleString() : '';
};

// A field for a time. The field shows local time. The model keeps ISO 8601
// text.
function TimeField({ id, label, value, onChange }) {
  return html`<${Field} id=${id} label=${label} type="datetime-local" value=${isoToLocalInput(value)}
    onInput=${(e) => onChange(startToIso(e.target.value))} />`;
}

// A small sheet that asks for the title and the start time of a new live
// show (ADR 0054 item 2). `onAdd` gets the title and the start as ISO text.
function AddLiveSheet({ open, onClose, onAdd, busy, error }) {
  const ref = useSheet(open);
  const [title, setTitle] = useState('');
  const [start, setStart] = useState('');
  // Clear the sheet each time that it opens.
  useEffect(() => {
    if (!open) return;
    setTitle('');
    setStart('');
  }, [open]);
  const iso = startToIso(start);
  const ready = title.trim() && iso && !busy;
  const submit = (event) => {
    event.preventDefault();
    if (ready) onAdd({ title: title.trim(), start: iso });
  };
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="add-live-title" onClose=${onClose}>
    <header>
      <h2 id="add-live-title">Add a live show</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <form class="body" id="add-live-form" onSubmit=${submit}>
      <div class="group rows">
        <${Field} id="new-live-title" label="Title" required value=${title} onInput=${(e) => setTitle(e.target.value)} />
        <${Field} id="new-live-start" label="Start time" type="datetime-local" required
          value=${start} onInput=${(e) => setStart(e.target.value)} />
        <${ErrorNotice} error=${error} />
      </div>
    </form>
    <footer>
      <button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>
      <button class="button primary" type="submit" form="add-live-form" disabled=${!ready}>＋ Add the show</button>
    </footer>
  </dialog>`;
}

// The content links of a live item, each as one entry row (ADR 0028). A
// show needs one link before Studio writes it (ADR 0054 item 4).
function ContentLinks({ id, links, onChange }) {
  const blank = { href: null, label: '' };
  return html`${links.length === 0 && html`<p class="hint">A show needs one content link. Studio does not write the show without it.</p>`}
    ${links.map((link, i) => {
      const change = (name) => (value) => onChange(setAt(links, i, set(link, name, value)));
      const rowId = `${id}-link-${link._cid}`;
      const summary = html`<span class="title">${link.label || 'New content link'}</span>
        <span class="sub mono">${link.href || 'No address'}</span>`;
      return html`<${EntryRow} key=${link._cid} group=${`${id}-links`} cid=${link._cid} summary=${summary}>
        <${TextField} id=${`${rowId}-href`} label="Link address" type="url" value=${link.href} onChange=${change('href')} />
        <${TextField} id=${`${rowId}-label`} label="Link text" value=${link.label} onChange=${change('label')} />
        <${EntryActions} list=${links} index=${i} name=${link.label || 'this content link'} onChange=${onChange} />
      <//>`;
    })}
    <button class="link-row add" type="button" onClick=${() => onChange(addOpen(links, blank))}>
      <span class="num">＋</span><span class="title">Add a content link</span>
    </button>`;
}

// The page of one live item. It edits the show, the stream, the content
// links, the live value and the value routes. "Go live" and "End the show"
// change the draft and open the review sheet (ADR 0054 items 3 and 5).
function LivePage({ feed, model, update, bad, liveIndex, draft }) {
  const lives = model.live_items;
  const live = lives[liveIndex];
  const id = `live-${live._cid}`;
  const action = useAction();
  const changeLive = (next) => update((m) => set(m, 'live_items', setAt(m.live_items, liveIndex, next)));
  const field = (name) => (value) => changeLive(set(live, name, value));
  const itemField = (name) => (value) => changeLive(set(live, 'item', set(live.item, name, value)));
  // A change that works on the current model, also after a long upload.
  const editLive = (edit) => update((m) => set(m, 'live_items', m.live_items.map((l) => (l._cid === live._cid ? edit(l) : l))));
  // Before the first recording, the show has no audio file from before. The
  // enclosure is then the stream (ADR 0054).
  const recordingBefore = hasRecording(live) ? live.item : { ...live.item, enclosure: null, alternate_enclosures: [] };
  const pickRecording = useFilePick(
    'recording',
    recordingBefore,
    (_address, facts, source) => editLive((l) => set(l, 'item', withNewAudio(l.item, facts, needsMetadataRead(source)))),
    (duration) => editLive((l) => set(l, 'item', set(l.item, 'duration', duration))),
  );
  const stream = live.item.enclosure || { url: null, length: null, media_type: null };
  const streamField = (name) => (text) => {
    const next = set(stream, name, text);
    itemField('enclosure')(next.url === null && next.length === null && next.media_type === null ? null : next);
  };
  const value = live.live_value || { uri: null, protocol: null };
  const valueField = (name) => (text) => {
    const next = set(value, name, text);
    field('live_value')(next.uri === null && next.protocol === null ? null : next);
  };
  const links = live.content_links;
  const next = nextAction(live.status);
  const problems = liveProblems(live);
  const start = timeText(live.start);
  // The grant of this show and its granted writes. The list loads when the
  // page opens and after each write.
  const guid = live.item.guid;
  const [grants, setGrants] = useState(null);
  const loadGrants = () => api(`/feeds/${feed.id}/grants`).then(setGrants, () => {});
  useEffect(() => {
    setGrants(null);
    loadGrants();
    window.addEventListener(ANNOUNCE_RELOAD, loadGrants);
    return () => window.removeEventListener(ANNOUNCE_RELOAD, loadGrants);
  }, [feed.id, guid]);
  const grant = (grants || []).find((g) => g.item_guid === guid) || null;
  const nowSeconds = Date.now() / 1000;
  const grantText = (seconds) => new Date(seconds * 1000).toLocaleString();
  const stopGrant = () => {
    if (!confirm('Stop the grant? Connected apps can no longer change this show.')) return;
    action.run(async () => {
      await api(`/feeds/${feed.id}/grants/${encodeURIComponent(guid)}/end`, { method: 'POST' });
      await loadGrants();
    });
  };
  const changeStatus = (status) => action.run(async () => {
    // Send each edit first, so that the status route works on the draft that
    // the user sees.
    if (!(await draft.flush())) return;
    const response = await api(`/feeds/${feed.id}/live-items/${encodeURIComponent(live.item.guid)}/status`, {
      method: 'POST',
      body: { status },
    });
    draft.show(response);
    window.dispatchEvent(new Event(OPEN_REVIEW));
  });
  return html`<header class="head">
      <${ImageThumb} url=${live.item.image || coverOf(model)} glow="var(--c-track)" className="cover" always />
      <div class="text"><span class="eyebrow"><span class=${`dot ${liveDot(live.status)}`}></span>Live show</span>
        <h1>${live.item.title || 'Untitled show'}</h1>
        <p class="sub">${showSummary(live, start)}</p></div>
    </header>

    <section class="page-section"><h2>Status</h2>
      <div class="group">
        <p class="live-line">${statusLabel(live.status)}</p>
        ${problems.map((text) => html`<p class="live-line field-error" key=${text}>${text}</p>`)}
        <${ErrorNotice} error=${action.error} />
        ${next && html`<div class="actions"><button class="button primary" type="button"
          disabled=${action.busy || !live.item.guid} onClick=${() => changeStatus(next.status)}>${next.label}</button></div>`}
        ${next && html`<p class="hint">The button changes the draft and opens the review sheet. The host does not change until you write.</p>`}
      </div>
    </section>

    ${grants && guid && html`<section class="page-section"><h2>Grant</h2>
      <div class="group">
        <p class="live-line">${grantSummary(grant, live, nowSeconds, grantText)}</p>
        ${grant && grant.writes.length > 0 && html`<ul class="grant-writes">
          ${grant.writes.map((write, index) => html`<li key=${index}>${grantWriteText(write, grantText)}</li>`)}
        </ul>`}
        ${grantActive(grant, live, nowSeconds) && html`<div class="actions"><button class="button destructive" type="button"
          disabled=${action.busy} onClick=${stopGrant}>Stop the grant</button></div>`}
        ${!grant && live.status === 'pending' && html`<p class="hint">Ask for a grant in the review sheet.</p>`}
      </div>
    </section>`}

    <section class="page-section"><h2>Show</h2>
      <div class="group">
        <${TextField} id=${`${id}-title`} label="Title" value=${live.item.title} onChange=${itemField('title')} />
        <${TextArea} id=${`${id}-description`} label="Description" value=${live.item.description} onChange=${itemField('description')} />
        <${TimeField} id=${`${id}-start`} label="Start time" value=${live.start} onChange=${field('start')} />
        <${TimeField} id=${`${id}-end`} label="End time" value=${live.end} onChange=${field('end')} />
        <${PickedImageField} field="live-image" subject=${live.item} apply=${(address) => editLive((l) => set(l, 'item', set(l.item, 'image', address)))}
          id=${`${id}-image`} label="Image address" address=${`live_items/${liveIndex}/item/image`} bad=${bad}
          value=${live.item.image} onChange=${itemField('image')} glow="var(--c-track)" />
      </div>
    </section>

    ${!canRecord(live) && html`<section class="page-section"><h2>Stream</h2>
      <div class="group">
        <${TextField} id=${`${id}-stream-url`} label="Stream address" type="url" address=${`live_items/${liveIndex}/item/enclosure/url`} bad=${bad}
          value=${stream.url} onChange=${streamField('url')} />
        <${TextField} id=${`${id}-stream-type`} label="Stream type" hint="For example application/x-mpegURL." value=${stream.media_type} onChange=${streamField('media_type')} />
      </div>
    </section>`}

    ${canRecord(live) && html`<section class="page-section"><h2>Recording</h2>
      <div class="group">
        ${hasRecording(live)
          ? html`<${Enclosure} id=${`${id}-recording`} path=${`live_items/${liveIndex}/item/`} bad=${bad} enclosure=${live.item.enclosure}
              onChange=${itemField('enclosure')} pick=${pickRecording} />
            <${AlternateEnclosures} list=${live.item.alternate_enclosures} onChange=${itemField('alternate_enclosures')} />`
          : html`<div class="field" ...${pickRecording.rootProps}><label for="choose-recording">Recording</label>
              <div class="field-body"><${ChooseFile} pick=${pickRecording} /></div></div>`}
      </div>
      <p class="hint">Studio puts the audio file in the enclosure of the show. The stream address goes out of the draft.</p>
    </section>`}

    <section class="page-section"><h2>Content links</h2>
      <div class="group">
        <${ContentLinks} id=${id} links=${links} onChange=${field('content_links')} />
      </div>
    </section>

    <section class="page-section"><h2>Live value</h2>
      <div class="group">
        <${TextField} id=${`${id}-value-uri`} label="Live value address" value=${value.uri} onChange=${valueField('uri')} />
        <${TextField} id=${`${id}-value-protocol`} label="Live value protocol" value=${value.protocol} onChange=${valueField('protocol')} />
        ${live.live_value && html`<div class="actions"><button class="button destructive" type="button"
          onClick=${() => field('live_value')(null)}>Remove the live value</button></div>`}
      </div>
    </section>

    <${TrackPayments} id=${id} item=${live.item} albumBlocks=${model.values} onChange=${itemField('values')} noun="show" />

    <section class="page-section"><h2>More</h2>
      <div class="group">
        <details class="disclosure">
          <summary class="link-row"><span class="label">Identity</span>
            <span class="summary">${live.item.guid ? 'Has a GUID' : 'No GUID yet'}</span></summary>
          <div class="form">
            <dl class="facts">
              <dt>Show GUID</dt><dd class="mono">${live.item.guid || 'Studio gives one when it saves'}</dd>
            </dl>
          </div>
        </details>
      </div>
    </section>`;
}

// ---- The publisher feed page (task 025c, ADR 0041) ----

// The medium of a feed in lower case, or an empty text.
function mediumOf(feed) {
  return (feed.medium || '').toLowerCase();
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

const PUBLISHER_ROLE_TEXT = 'Select the role of this feed first.';

// The publisher page: the fields, the role and the releases (ADR 0041 item
// 3). Each link changes the draft of this feed and the draft of the release,
// so the page loads both drafts and names each release that changed.
function PublisherPage({ feed, feeds, model, update, bad, draft, go, onListChanged, onArtwork, onRemoved }) {
  const field = (name) => (value) => update((m) => set(m, name, value));
  const roleAction = useAction();
  const linkAction = useAction();
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
  const roles = feed.publisher_roles || [];
  // "Custom" shows the check boxes also before the first change (ADR 0082
  // item 7).
  const [custom, setCustom] = useState(false);
  const choice = custom ? 'custom' : roleChoice(roles);
  // A role change writes the links, so it needs the host of the feed, as a
  // link does (ADR 0041 item 5).
  const readOnly = !feed.connector_id;

  // The steps of this page for one change of two drafts.
  const twoDrafts = (action, call) => changeTwoDrafts(action, { feedId: feed.id, draft, onListChanged, setChanged, call });
  // Save and confirm a set of roles (ADR 0082 items 5 and 8).
  const setRoles = (next) => {
    if (next.length === 0) return;
    twoDrafts(roleAction, async () => (await api(`/feeds/${feed.id}/role`, { method: 'PUT', body: { roles: next } })).changed);
  };
  const choose = (value) => {
    setCustom(value === 'custom');
    if (FIRST_CLASS_ROLES.includes(value)) setRoles([value]);
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
  // Move a release to a position among the releases (ADR 0041 item 10,
  // ADR 0081 item 5). The other entries of the list keep their index.
  const moveRow = (position, to) => {
    const slots = rows.map((row) => row.index);
    update((m) => set(m, 'remote_items', moveAmong(m.remote_items, slots, position, to)));
  };
  const openPage = (page) => go({ name: 'feed', id: feed.id, ...page });
  const shown = {
    channel: (s) => html`<section class="page-section"><h2>${feedKind(feed).label}</h2>
      <div class="group">
        ${showEach(s.fields, {
          title: () => html`<${TextField} id="pub-title" label="Title" value=${model.title} onChange=${field('title')} />`,
          author: () => html`<${TextField} id="pub-author" label="Name" hint="The itunes:author of the feed." value=${model.author} onChange=${field('author')} />`,
          nostr: () => html`<${FeedKeyRow} feedId=${feed.id} model=${model} update=${update} bad=${bad} />`,
          description: () => html`<${TextArea} id="pub-description" label="Description" value=${model.description} onChange=${field('description')} />`,
          artwork: () => html`<${ArtworkRow} summary=${artworkSummary(model, 'feed')} onOpen=${() => onArtwork({ kind: 'feed' })} />`,
          link: () => html`<${TextField} id="pub-link" label="Website" type="url" address="link" bad=${bad} value=${model.link} onChange=${field('link')} />`,
          listing: () => html`<${ListingField} id="pub-listing" feedId=${feed.id} draft=${draft} model=${model} />`,
          explicit: () => html`<${ExplicitField} id="pub-explicit" model=${model} update=${update} />`,
        })}
      </div>
    </section>`,
    details: (s) => html`<section class="page-section"><h2>Details</h2>
      <div class="group">
        ${detailRows(s.rows, model, openPage)}
      </div>
    </section>`,
    role: () => html`<section class="page-section"><h2>Role</h2>
      <div class="group">
        <div class="field">
          <label for="pub-role">Role</label>
          <div class="field-body">
            <select id="pub-role" value=${choice} disabled=${roleAction.busy || readOnly} onChange=${(e) => choose(e.target.value)}>
              ${choice === '' && html`<option value="" disabled>Select a role</option>`}
              ${FIRST_CLASS_ROLES.map((role) => html`<option value=${role}>${roleWord(role)}</option>`)}
              <option value="custom">Custom</option>
            </select>
            ${readOnly && html`<span class="hint">To change the roles, connect the folder that holds this feed in Hosts.</span>`}
            ${roles.length > 0 && !feed.roles_confirmed && html`<span class="hint">Studio found ${roles.length === 1 ? 'this role' : 'these roles'} in the feed: ${roles.map(roleWord).join(', ')}.${readOnly ? '' : ' Select the roles to confirm them.'}</span>`}
            ${choice === 'custom' && html`<div class="role-choices" role="group" aria-label="Roles">
              ${ROLES.map((role) => html`<label class="check"><input type="checkbox" id=${`pub-role-${role}`}
                checked=${roles.includes(role)} disabled=${roleAction.busy || readOnly || (roles.length === 1 && roles[0] === role)}
                onChange=${(e) => setRoles(toggleRole(roles, role, e.target.checked))} /> ${roleWord(role)}</label>`)}
            </div>
            <span class="hint">The first role in this list gives the type of the feed. A feed has one or more roles.</span>`}
          </div>
        </div>
        <${ErrorNotice} error=${publisherError(roleAction.error, PUBLISHER_ROLE_TEXT)} />
      </div>
    </section>`,
    releases: () => html`<section class="page-section"><h2>Releases</h2>
      <div class="group">
        ${rows.length === 0 && html`<p class="hint">No releases yet.</p>`}
        ${rows.map((row, position) => html`<div class="release-row" key=${row.entry._cid}>
          <div class="order-line">
            <${PositionSelect} id=${`release-position-${position}`} index=${position} count=${rows.length}
              name=${row.entry.title || 'this release'} disabled=${linkAction.busy} onMove=${(to) => moveRow(position, to)} />
            <span class="title">${row.entry.title || row.entry.feed_url}</span>
          </div>
          <span class="sub">${row.match ? 'In the collection' : 'Not in the collection'}</span>
          <div class="actions">
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
    </section>`,
    feed: (s) => html`<section class="page-section"><h2>Feed</h2>
      <div class="group">
        ${showEach(s.rows, {
          settings: () => html`<${SettingsRow} model=${model} onOpen=${() => go({ name: 'feed', id: feed.id, page: 'settings' })} />`,
          facts: () => html`<${FeedFacts} feed=${feed} onRemoved=${onRemoved} />`,
        })}
      </div>
    </section>`,
  };
  return html`<${ChangedNotices} changed=${changed} go=${go} />
    ${showSections(pageSequence('publisher'), shown)}`;
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
export function AlbumEditor({ feed, feeds, draft, view, go, adding, onAdding, importing, onImporting, onRemoved, connectors, onOpsChanged, filesReloadKey, differences, onRescan, onListChanged, artwork, onArtwork }) {
  const { model, update } = draft;
  const page = view.page || 'album';
  const trackIndex = model && page === 'track' ? model.items.findIndex((item) => isTrack(item, view.track)) : -1;
  const liveIndex = model && page === 'live' ? (model.live_items || []).findIndex((live) => isLiveItem(live, view.track)) : -1;
  const addLive = useAction();
  // When the track got its GUID, put the new key in the view, so that the
  // tree shows the current track.
  const currentKey = trackIndex === -1 ? null : trackKey(model.items[trackIndex]);
  useEffect(() => {
    if (currentKey && currentKey !== view.track) go({ ...view, track: currentKey }, { replace: true });
  }, [currentKey, view.track]);
  // The track of an open track sheet. A key that no track has closes the
  // sheet (ADR 0070).
  const artworkTrack = model && artwork && artwork.kind === 'track'
    ? model.items.findIndex((item) => isTrack(item, artwork.key)) : -1;
  const staleArtwork = Boolean(model && artwork && artwork.kind === 'track' && artworkTrack === -1);
  useEffect(() => { if (staleArtwork) onArtwork(null); }, [staleArtwork]);
  if (!model) {
    return html`<div class="page"><p class="hint" role="status">${STATUS[draft.status]}</p>
      <${ErrorNotice} error=${draft.error} /></div>`;
  }
  // A publisher feed has the publisher page and the Files page only (ADR
  // 0041 item 3).
  const publisher = isPublisher(feed);
  const playlist = isPlaylist(feed);
  let body;
  // When the track key of the view is not in the model, for example after a
  // write that gave each item a new client ID, the editor shows the album
  // page (task 014c, Architecture).
  if (page === 'settings') {
    body = html`<${SettingsPage} model=${model} update=${update} album=${!publisher && !playlist} op3Tracks=${draft.op3Tracks} />`;
  } else if (publisher && page !== 'files' && page !== 'credits' && page !== 'recipients') {
    body = html`<${PublisherPage} feed=${feed} feeds=${feeds} model=${model} update=${update} bad=${draft.bad}
      draft=${draft} go=${go} onListChanged=${onListChanged} onArtwork=${onArtwork} onRemoved=${onRemoved} />`;
  } else if (playlist && page === 'tracks') {
    body = html`<${PlaylistTracksPage} feed=${feed} model=${model} update=${update} />`;
  } else if (playlist && page !== 'recipients' && page !== 'files') {
    body = html`<${PlaylistPage} feed=${feed} draft=${draft} model=${model} update=${update} bad=${draft.bad} go=${go} onRemoved=${onRemoved} onArtwork=${onArtwork} />`;
  } else if (page === 'track' && trackIndex !== -1) {
    // A key for each entry, so that one track or show does not keep the
    // state of the one before it, for example an open "Properties".
    body = html`<${TrackPage} key=${model.items[trackIndex]._cid} feed=${feed} model=${model} update=${update} bad=${draft.bad} trackIndex=${trackIndex} go=${go} onArtwork=${onArtwork} />`;
  } else if (page === 'live' && liveIndex !== -1 && !publisher && !playlist) {
    body = html`<${LivePage} key=${model.live_items[liveIndex]._cid} feed=${feed} model=${model} update=${update} bad=${draft.bad} liveIndex=${liveIndex} draft=${draft} />`;
  } else if (page === 'credits') {
    body = html`<${CreditsPage} model=${model} update=${update} bad=${draft.bad} />`;
  } else if (page === 'recipients') {
    body = html`<${RecipientsPage} model=${model} update=${update} />`;
  } else if (page === 'files') {
    body = html`<${FilesPage} feed=${feed} connectors=${connectors} onOpsChanged=${onOpsChanged} reloadKey=${filesReloadKey}
      differences=${differences} onRescan=${onRescan} />`;
  } else {
    body = html`<${AlbumPage} feed=${feed} feeds=${feeds} model=${model} update=${update} bad=${draft.bad} draft=${draft} go=${go}
      onAdding=${onAdding} onImporting=${onImporting} onRemoved=${onRemoved} onListChanged=${onListChanged} onArtwork=${onArtwork} />`;
  }
  const onAdd = (item) => {
    const [added] = withCids([item]);
    update((m) => set(m, 'items', [...m.items, added]));
    onAdding(false);
    go({ name: 'feed', id: feed.id, page: 'track', track: trackKey(added) });
  };
  // The route adds the live item to the draft. Then the page shows the
  // draft of the response and opens the page of the new show.
  const onAddLive = ({ title, start }) => addLive.run(async () => {
    if (!(await draft.flush())) return;
    const response = await api(`/feeds/${feed.id}/live-items`, { method: 'POST', body: { title, start } });
    draft.show(response);
    onAdding(false);
    const added = response.model.live_items[response.model.live_items.length - 1];
    go({ name: 'feed', id: feed.id, page: 'live', track: liveKey(added) });
  });
  // The import sheet already sent each file to the draft of this feed. Add
  // the tracks that it built, and the image that it chose for the album, to
  // the model (task architecture).
  const onImport = (tracks, albumImageUrl) => {
    const added = withCids(tracks);
    update((m) => {
      const withImage = albumImageUrl ? { ...m, image: albumImageUrl } : m;
      return set(withImage, 'items', [...withImage.items, ...added]);
    });
    onImporting(false);
    go({ name: 'feed', id: feed.id });
  };
  return html`<${PickerContext.Provider} value=${{ feedId: feed.id, connectors, draft, onOpsChanged }}><div class="page">
    ${body}
    ${artwork && !staleArtwork && html`<${ArtworkSheet} feed=${feed} model=${model} update=${update} bad=${draft.bad} artwork=${artwork}
      trackIndex=${artworkTrack} onClose=${() => onArtwork(null)} />`}
    ${!publisher && !playlist && html`<${AddTrackSheet} open=${adding === true} onClose=${() => onAdding(false)} onAdd=${onAdd}
      onFiles=${() => { onAdding(false); onImporting(true); }} />`}
    ${!publisher && !playlist && html`<${AddLiveSheet} open=${adding === 'live'} onClose=${() => onAdding(false)}
      onAdd=${onAddLive} busy=${addLive.busy} error=${addLive.error} />`}
    ${!publisher && !playlist && html`<${ImportSheet} feedId=${feed.id} open=${importing} hasAlbumImage=${Boolean(coverOf(model))}
      onClose=${() => onImporting(false)} onAdd=${onImport}
      onAddress=${() => { onImporting(false); onAdding(true); }} />`}
  </div><//>`;
}

// The album editor (task 011, ADR 0022), its pages (task 014c, ADR 0029
// items 5 to 10) and its toolbar and sheets (task 014b, ADR 0029). The UI
// edits the typed model of the draft. Each list entry carries its key, and
// the draft saves automatically.
import { html, useState, useEffect, useRef, createContext, useContext } from './vendor/htm-preact-standalone.module.js';
import { CONFIRMS } from './confirms.js';
import { VIEWS, VIEW_LABELS, VIEW_HINTS, normalizeView, shownLines, localTimeNote, runs, commonIndent, stripIndent, moreLabel, fieldRows, guardRow, changeKeys, MERGE_TITLE, MERGE_INTRO, MERGE_HEADINGS, mergeRows, mergeReady } from './review.js';
import { onSegmentKey, dropAreaClass, useFileDrop, api, saveTicket, useMedia, assetMode, askConfirm, CopyButton, RoleMenu, useRoleList, sendInParts, DateTimeField, DurationField, Field, ErrorNotice, useAction, addressSource, ImageThumb, useImage, useHeldFiles, heldFilesNow, SessionContext, Icon, AddFeedRowSheet } from './app.js';
import { AlbumPayments, TrackPayments } from './values.js';
import { PickerLayout, PickFillRow, FindOneSheet, PickFold, TakenTag, useFieldList } from './pickers.js';
import { PersonArt, useProfiles } from './peoplepages.js';
import { creditPickRows, supportPickRows, creditChoices, fundingChoices, nostrChoices, withPersonValues, filterRows, nostrPickRows, feedWord, memberRowLine, memberSectionNote } from './people.js';
import { withCids, STATUS } from './draft.js';
import { roleSummary } from './roles.js';
import { feedKind, ROLES, FIRST_CLASS_ROLES, roleWord, relLabel, roleChoice, toggleRole, trackKey, isTrack, isPublisher, isPlaylist, isReadOnly, reviewHost, moveNote, byName } from './shell.js';
import { isLiveItem, liveKey, liveDot, nextAction, canRecord, hasRecording, showSummary, statusLabel, grantActive, grantSummary, grantWriteText, grantBoxState, hostUsesSigner, OTHER_TYPE, typeIsSelected, typeMenuState, filledType, checkLine, checkTypeWarning } from './live.js';
import { grantControl, draftBlockedNotice } from './apps.js';
import { buildDateNotices } from './builddate.js';
import { versionLabel, LOCK_HELP, hasSpace, spaceNotice, ROW_SPACE_NOTE, newFeedAddress } from './feedfacts.js';
import { refusalView, refusalOfError, noticeViews, blocksWrite, repairRoute, returnWarning, REPLACES_POINTER_NOTE } from './movelink.js';
import { pickFromIndex, remoteItemFromPick, addTrack, hasTrack, removeTrack, trackRow, tracksSummary } from './playlist.js';
import { moveTo, moveAmong } from './order.js';
import { announceLines, announceStatus, isWaiting } from './announce.js';
import { npubOf, eventsToSign, signAll, signErrorText, signSummary, signProgressLabel, SignError, remoteSigner, signerChoice } from './signing.js';
import { SignerConnect } from './signer.js';
import { progressPercent } from './upload.js';
import { coverOf, coverConflict } from './cover.js';
import { platformBlocks } from './listing.js';
import { pageSequence } from './pages.js';
import { nextStep, firstStart, FIRST_STEPS, HOST_STEP } from './welcome.js';
import {
  fileState, fileStatus, canDownloadToDevice, canCopyOrMove, missingCount, missingNotice, reviewNotice, dayText,
  SERVER_NOTICE, ADD_TRACKS_NOTICE, ENTRY_TEXT,
} from './assets.js';
import { isExplicit } from './explicit.js';
import { op3Hint, op3Page } from './op3.js';
import { shortKey, keyRowView, feedKey, withFeedKey, sourceText, withPersonKey } from './nostrkey.js';
import { usesAlbumCredits, withTrackCredits, withAlbumCredits } from './credits.js';
import { readDuration, durationText } from './duration.js';
import { mediaSource, playerNotice, canPlay, playerError, isPlaceholderAddress, PLAYER_STARTING, PLAYER_STARTING_DELAY_MS, playerMark, playWasReplaced, playerProgress } from './media.js';
import { shortType as formatLabel, propertiesSummary, propertiesOpen, propertyRows, propertiesNotice, needsMetadataRead, clockText } from './audio.js';
import { planRows, stayRows, deleteLabel, deleteAllLabel, DELETE_NOTE_ON, DELETE_NOTE_OFF, waitLabel, waitingView, readyNotice, partMoveNotice } from './releasemove.js';
import { PURPOSES, purposeTokens, withPurpose, imageFacts, artworkSummary, artworkPreview } from './artwork.js';
import { fieldKind, sources, hostTree, canSelect, reasonText, acceptsFile, dropResult, addsAsTrackFile, refusedText, wrongKindText, replacedAddresses, selectedFacts } from './picker.js';

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
      aria-label=${`Move ${name} up`} onClick=${() => onChange(move(list, index, -1))}>Move Up</button>
    <button class="button" type="button" disabled=${index === list.length - 1}
      aria-label=${`Move ${name} down`} onClick=${() => onChange(move(list, index, 1))}>Move Down</button>
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

// The picture of a person in an open entry. It shows only when the image
// loads.
function PersonPicture({ url }) {
  const { src } = useImage(url || null);
  return src ? html`<div class="person-picture"><img src=${src} alt="" /></div>` : null;
}

// The sheet "Add a Credit" (ADR 0145). A row of a person or a feed of musicindex
// fills the form, and the user adds the values of the form.
function CreditSheet({ open, persons, feedId, onAdd, onClose }) {
  const ref = useSheet(open);
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="credit-title" onClose=${onClose}>
    ${open && html`<${CreditForm} persons=${persons} feedId=${feedId} onCancel=${() => ref.current.close()}
      onAdd=${(values) => { onAdd(values); ref.current.close(); }} />`}
  </dialog>`;
}

function CreditForm({ persons, feedId, onAdd, onCancel }) {
  const [values, setValues] = useState(BLANK_PERSON);
  const [people, setPeople] = useState(null);
  const [finding, setFinding] = useState(false);
  const [keyOpen, setKeyOpen] = useState(false);
  const loading = useAction();
  useEffect(() => { loading.run(async () => setPeople(await api('/people'))); }, []);
  const change = (name) => (value) => setValues((current) => set(current, name, value));
  const ready = String(values.name ?? '').trim() !== '';
  const rows = creditPickRows(people || [], persons).map((r) => html`<${PickFillRow} key=${r.id} title=${r.name}
    line=${r.line} taken=${r.taken} lead=${html`<${PersonArt} person=${r.person} />`}
    selected=${values.name === r.credit.name} onSelect=${() => setValues((current) => ({ ...current, ...r.credit }))} />`);
  return html`<header>
      <h2 id="credit-title">Add a Credit</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${onCancel}>✕</button>
    </header>
    <${PickerLayout} id="credit-form" onSubmit=${(e) => { e.preventDefault(); if (ready) onAdd(values); }}
      fields=${html`<div class="group rows">
        <${TextField} id="credit-name" label="Name" required value=${values.name} onChange=${change('name')} />
        <${RoleMenu} id="credit-role" role=${values.role} group=${values.group}
          onChange=${(role, group) => setValues((current) => ({ ...current, role: role || null, group: group || null }))} />
        <${TextField} id="credit-img" label="Picture address" type="url" value=${values.img} onChange=${change('img')} />
        <${TextField} id="credit-href" label="Information address" type="url" value=${values.href} onChange=${change('href')} />
        <${NostrKeyRow} label="Nostr" name=${values.name} value=${values.npub} onOpen=${() => setKeyOpen(true)} />
      </div>`}
      notice=${html`<${ErrorNotice} error=${loading.error} />`}
      musicindex=${{ title: 'Find on Music Index', summary: 'Fill the form from a feed.', onOpen: () => setFinding(true) }}
      lists=${[{ title: 'Your people', rows }]} />
    <footer><button class="button primary" type="submit" form="credit-form" disabled=${!ready}>Add the Credit</button></footer>
    <${NostrSheet} open=${keyOpen} value=${values.npub} feedId=${feedId} person=${{ name: values.name, img: values.img }}
      onSet=${(npub, picked) => setValues((current) => withPersonKey(current, npub, picked))} onClose=${() => setKeyOpen(false)} />
    <${FindOneSheet} open=${finding} start=${values.name} choices=${creditChoices} credits=${true} empty="Music Index has no values of this feed."
      onPick=${(found) => setValues((current) => ({ ...current, ...found }))} onClose=${() => setFinding(false)} />`;
}

// The sheet "Add a Support Link" (ADR 0145). The API of musicindex has no support
// link, so its row gives only the website of a feed.
function FundingSheet({ open, fundings, onAdd, onClose }) {
  const ref = useSheet(open);
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="funding-title" onClose=${onClose}>
    ${open && html`<${FundingForm} fundings=${fundings} onCancel=${() => ref.current.close()}
      onAdd=${(values) => { onAdd(values); ref.current.close(); }} />`}
  </dialog>`;
}

function FundingForm({ fundings, onAdd, onCancel }) {
  const [values, setValues] = useState({ label: '', url: null });
  const [people, setPeople] = useState(null);
  const [finding, setFinding] = useState(false);
  const loading = useAction();
  useEffect(() => { loading.run(async () => setPeople(await api('/people'))); }, []);
  const change = (name) => (value) => setValues((current) => set(current, name, value));
  const ready = String(values.url ?? '').trim() !== '';
  const rows = supportPickRows(people || [], fundings).map((r) => html`<${PickFillRow} key=${r.id} title=${r.name}
    line=${r.line} taken=${r.taken}
    selected=${values.url === r.funding.url} onSelect=${() => setValues(r.funding)} />`);
  return html`<header>
      <h2 id="funding-title">Add a Support Link</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${onCancel}>✕</button>
    </header>
    <${PickerLayout} id="funding-form" onSubmit=${(e) => { e.preventDefault(); if (ready) onAdd(values); }}
      fields=${html`<div class="group rows">
        <${TextField} id="funding-label" label="Text" required value=${values.label} onChange=${change('label')} />
        <${TextField} id="funding-url" label="Address" type="url" value=${values.url} onChange=${change('url')} />
      </div>`}
      notice=${html`<${ErrorNotice} error=${loading.error} />`}
      musicindex=${{ title: 'Find on Music Index', summary: 'Fill the address from the website of a feed.', onOpen: () => setFinding(true) }}
      lists=${[{ title: 'Your people', rows }]} />
    <footer><button class="button primary" type="submit" form="funding-form" disabled=${!ready}>Add the Support Link</button></footer>
    <${FindOneSheet} open=${finding} choices=${fundingChoices} empty="This feed has no website."
      onPick=${(found) => setValues((current) => ({ ...current, ...found }))} onClose=${() => setFinding(false)} />`;
}

function Persons({ prefix, persons, onChange, bad, path = 'persons' }) {
  const { feedId } = useContext(PickerContext) || {};
  const [adding, setAdding] = useState(false);
  const roles = useRoleList();
  return html`${persons.length === 0 && html`<p class="hint">No people yet.</p>`}
    ${persons.map((p, i) => {
      const change = (name) => (value) => onChange(setAt(persons, i, set(p, name, value)));
      const id = `${prefix}-person-${p._cid}`;
      const summary = html`<span class="title">${p.name || 'New person'}</span>
        <span class="sub">${roleSummary(p.role, p.group, roles || []) || 'host · cast'}</span>`;
      return html`<${EntryRow} key=${p._cid} group=${`${prefix}-persons`} cid=${p._cid} summary=${summary}>
        <${TextField} id=${`${id}-name`} label="Name" required value=${p.name} onChange=${change('name')} />
        <${RoleMenu} id=${`${id}-role`} role=${p.role} group=${p.group}
          onChange=${(role, group) => onChange(setAt(persons, i, { ...p, role: role || null, group: group || null }))} />
        <${PersonPicture} url=${p.img} />
        <${TextField} id=${`${id}-img`} label="Picture address" type="url" address=${`${path}/${i}/img`} bad=${bad} value=${p.img} onChange=${change('img')} />
        <${TextField} id=${`${id}-href`} label="Information address" type="url" address=${`${path}/${i}/href`} bad=${bad} value=${p.href} onChange=${change('href')} />
        <${NostrRow} label="Nostr" name=${p.name} value=${p.npub} feedId=${feedId}
          bad=${(bad || []).some((b) => b.field === `${path}/${i}/npub`)}
          person=${{ name: p.name, img: p.img }}
          onSet=${(npub, picked) => onChange(setAt(persons, i, withPersonKey(p, npub, picked)))} />
        <${EntryActions} list=${persons} index=${i} name=${p.name || 'this person'} onChange=${onChange} />
      <//>`;
    })}
    <button class="link-row add" type="button" onClick=${() => setAdding(true)}>
      <span class="num">＋</span><span class="title">Add a credit</span>
    </button>
    <${CreditSheet} open=${adding} persons=${persons} feedId=${feedId} onAdd=${(values) => onChange(addOpen(persons, values))}
      onClose=${() => setAdding(false)} />`;
}

function Fundings({ fundings, onChange, bad }) {
  const [adding, setAdding] = useState(false);
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
    <button class="link-row add" type="button" onClick=${() => setAdding(true)}>
      <span class="num">＋</span><span class="title">Add a support link</span>
    </button>
    <${FundingSheet} open=${adding} fundings=${fundings} onAdd=${(values) => onChange(addOpen(fundings, values))}
      onClose=${() => setAdding(false)} />`;
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

// "Properties" of an audio file (ADR 0130). The values show as text. The boxes
// are the children, and they show after "Advanced" and its warning. They stay
// until the user selects "Lock Values" or leaves the page. A lock changes no
// value.
function AudioProperties({ enclosure, duration, errors = [], mustOpen, children }) {
  const [opened, setOpened] = useState(mustOpen);
  const [unlocked, setUnlocked] = useState(false);
  useEffect(() => { if (mustOpen) setOpened(true); }, [mustOpen]);
  const unlock = async () => {
    if (await askConfirm(CONFIRMS.editAudioValues())) setUnlocked(true);
  };
  const notice = propertiesNotice(enclosure, duration, errors);
  return html`<details class="disclosure" open=${opened} onToggle=${(e) => setOpened(e.currentTarget.open)}>
    <summary class="link-row"><span class="label">Properties</span>
      <span class="summary">${propertiesSummary(enclosure, duration)}</span></summary>
    <div class="form">
      ${unlocked ? html`${children}
        <div class="audio-values"><div class="actions"><button class="button" type="button" onClick=${() => setUnlocked(false)}>Lock Values</button></div></div>` : html`<div class="audio-values">
        <dl class="facts">${propertyRows(enclosure, duration).map((row) => html`<dt>${row.label}</dt><dd>${row.value}</dd>`)}</dl>
        ${notice && html`<p class="hint" role="status">${notice}</p>`}
        <div class="actions"><button class="button" type="button" onClick=${unlock}>Advanced</button></div>
      </div>`}
    </div>
  </details>`;
}

function Enclosure({ id, path, enclosure, onChange, bad, pick, duration }) {
  const value = enclosure || { url: null, length: null, media_type: null };
  // An enclosure with no field is no enclosure.
  const change = (name) => (text) => {
    const next = set(value, name, text);
    onChange(next.url === null && next.length === null && next.media_type === null ? null : next);
  };
  // "Properties" opens by itself when a value is empty or has an error, or
  // after a read of the duration that failed (ADR 0130 item 4).
  const errors = ['length', 'media_type'].filter((name) => (bad || []).some((b) => b.field === `${path}enclosure/${name}`));
  const facts = duration && {
    seconds: duration.value,
    text: duration.sourceText,
    failed: Boolean(pick.metadataFailed),
  };
  const mustOpen = propertiesOpen(enclosure, errors) || Boolean(facts && facts.failed && facts.seconds == null);
  // A placeholder address has the text of ADR 0122 item 14 (task 086d).
  const placeholder = isPlaceholderAddress(value.url) ? HOST_STEP.placeholderAddress : null;
  return html`<${TextField} id=${`${id}-url`} label="Audio file address" type="url" address=${`${path}enclosure/url`} bad=${bad} value=${value.url} onChange=${change('url')}
      hint=${placeholder} rootProps=${pick.rootProps}><${ChooseFile} pick=${pick} /><//>
    <${AudioPlayer} id=${id} url=${value.url} pick=${pick} />
    <${AudioProperties} enclosure=${enclosure} duration=${facts} errors=${errors} mustOpen=${mustOpen}>
      <${TextField} id=${`${id}-length`} label="Audio file size in bytes" inputmode="numeric" value=${value.length} onChange=${change('length')} />
      <${TextField} id=${`${id}-type`} label="Audio file type" hint="For example audio/mpeg." value=${value.media_type} onChange=${change('media_type')} />
      ${duration && html`<${DurationField} id=${`${id}-duration`} value=${duration.value} sourceText=${duration.sourceText} onChange=${duration.onChange} />`}
    <//>`;
}

// The player of an audio file (ADR 0062 items 26 to 29, ADR 0109). It plays
// from the `https` address of the file on its host, and reads no audio before
// the user presses play. A file that only Studio holds plays through Studio
// with a media ticket. An `http` address does not play. The one exception is the read of the metadata after a selection that
// `needsMetadataRead` marks. The element then reads the duration, and
// `pick.metadataDone` puts it in the track.
const TICKET_RENEW_MS = 50 * 60 * 1000;

// Wait until the browser has painted the current state.
const nextPaint = () => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));

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
  const [starting, setStarting] = useState(false);
  const [slowStart, setSlowStart] = useState(false);
  const heldFiles = useHeldFiles(url);
  const source = mediaSource(url, heldFiles);
  const usable = canPlay(source);
  const notice = playerNotice(source);
  const reading = Boolean(pick.metadataFor) && pick.metadataFor === url;
  // A file from its host needs no ticket. A held file needs a ticket for its
  // library file ID.
  const load = async () => {
    let src;
    if (source.kind === 'held') {
      const { ticket } = await api('/media/tickets', { method: 'POST', body: { file: source.file } });
      src = `/media/audio?ticket=${ticket}`;
    } else {
      src = source.src;
    }
    audio.current.preload = reading ? 'metadata' : 'none';
    audio.current.src = src;
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
    setStarting(false);
  }, [url]);
  useEffect(() => {
    if (!reading || !usable) return;
    load().catch(() => pick.metadataDone(null));
  }, [url, pick.metadataFor, usable]);
  useEffect(() => {
    if (!starting) { setSlowStart(false); return undefined; }
    const timer = setTimeout(() => setSlowStart(true), PLAYER_STARTING_DELAY_MS);
    return () => clearTimeout(timer);
  }, [starting]);
  const toggle = async () => {
    const element = audio.current;
    if (!element.paused) { element.pause(); setStarting(false); return; }
    setError(null);
    setStarting(true);
    // The start of playback can hold the page, so the state paints first. A
    // touch browser needs the call of play() directly in the gesture.
    if (!matchMedia('(pointer: coarse)').matches) await nextPaint();
    const fresh = held.current && held.current.url === url
      && (source.kind === 'direct' || Date.now() - held.current.at < TICKET_RENEW_MS);
    try {
      if (!fresh) {
        resume.current = held.current && held.current.url === url ? element.currentTime : 0;
        await load();
      }
      await element.play();
    } catch (e) {
      if (playWasReplaced(e)) return;
      setStarting(false);
      setError(playerError(source));
    }
  };
  const onLoadedMetadata = (e) => {
    const element = e.currentTarget;
    setLength(element.duration);
    if (resume.current > 0) { element.currentTime = resume.current; resume.current = 0; }
    if (reading) pick.metadataDone(Number.isFinite(element.duration) ? Math.floor(element.duration) : null);
  };
  const onError = (e) => {
    // An element with no address gives no error of its own.
    if (!e.currentTarget.getAttribute('src')) return;
    setPlaying(false);
    setStarting(false);
    setError(playerError(source));
    if (reading) pick.metadataDone(null);
  };
  const known = Number.isFinite(length) && length > 0;
  const mark = playerMark({ starting, slowStart, playing });
  return html`<div class="field">
    <label for=${`${id}-play`}>Player</label>
    <div class="field-body">
      <div class="player">
        <button id=${`${id}-play`} class="button icon-button" type="button" disabled=${!usable}
          aria-busy=${starting ? 'true' : undefined}
          aria-label=${starting ? PLAYER_STARTING : playing ? 'Pause' : 'Play'} onClick=${toggle}>${mark === 'wait'
            ? html`<span class="player-wait" aria-hidden="true"></span>`
            : html`<${Icon} name=${mark} />`}</button>
        <input class="player-bar" type="range" min="0" max=${known ? length : 0} step="any" value=${position}
          style=${{ '--progress': playerProgress(position, length) }}
          aria-label="Position" disabled=${!known}
          onInput=${(e) => { audio.current.currentTime = Number(e.currentTarget.value); setPosition(Number(e.currentTarget.value)); }} />
        ${slowStart
          ? html`<span class="player-time" role="status">${PLAYER_STARTING}</span>`
          : html`<span class="player-time">${clockText(position)} / ${clockText(length)}</span>`}
      </div>
      ${error && html`<span class="field-error" role="alert">${error}</span>`}
      ${notice && html`<span class="hint" id=${`${id}-notice`} role="status">${notice}</span>`}
      <audio ref=${audio} preload="none" onLoadedMetadata=${onLoadedMetadata} onDurationChange=${(e) => setLength(e.currentTarget.duration)}
        onTimeUpdate=${(e) => setPosition(e.currentTarget.currentTime)} onPlay=${() => setPlaying(true)} onPlaying=${() => { setStarting(false); setError(null); }} onPause=${() => setPlaying(false)}
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
  // Each opening of the sheet starts locked (ADR 0130 item 2).
  const [session, setSession] = useState(0);
  // "Properties" opens by itself when the lookup finds fewer than two facts.
  // It stays closed while the lookup is not done. The user can open it at
  // each time (ADR 0073 item 14).
  const mustOpen = (facts.state === 'partial' || facts.state === 'failed')
    && propertiesOpen({ length, media_type: type }, []);
  // Clear the sheet each time that it opens.
  useEffect(() => {
    if (!open) return;
    setTitle('');
    setUrl('');
    setLength('');
    setType('');
    setFacts({ state: 'none', for: '' });
    setSession((n) => n + 1);
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
      pub_date_time: null,
      duration: null,
      duration_seconds: null,
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
    reading: 'Studio asks the file host for the size and the type of the file…',
    found: 'Studio found the size and the type on the file host.',
    partial: 'The file host did not give all the facts. To type them, open Properties and select Advanced.',
    failed: 'Studio could not read the facts from the file host. To type them, open Properties and select Advanced.',
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
          hint="Studio does not keep audio files. Give the address where your file host keeps the file."
          error=${facts.state === 'invalid' && facts.for === url.trim() ? BAD_ADDRESS : null}
          value=${url} onInput=${(e) => setUrl(e.target.value)} onBlur=${readFacts} />
        ${FACTS[facts.state] && html`<p class="hint" role="status">${FACTS[facts.state]}</p>`}
        <${AudioProperties} key=${session} enclosure=${{ length, media_type: type }} mustOpen=${mustOpen}>
          <${Field} id="new-track-length" label="Audio file size in bytes" inputmode="numeric" required
            value=${length} onInput=${(e) => setLength(e.target.value)} />
          <${Field} id="new-track-type" label="Audio file type" hint="For example audio/mpeg." required
            value=${type} onInput=${(e) => setType(e.target.value)} />
        <//>
      </div>
      <button class="button link" type="button" onClick=${onFiles}>Choose Files from This Computer</button>
    </form>
    <footer>
      <button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>
      <button class="button primary" type="submit" form="add-track-form" disabled=${!ready}>＋ Add the Track</button>
    </footer>
  </dialog>`;
}

// ---- The Nostr key rows and the sheet "Find a Nostr profile" (ADR 0074 items 18 to 25) ----

// The row of a Nostr key. It opens the sheet. `person` gives the name and the
// picture of a person entry, or it is missing for the key of a feed. `bad` is
// true when the core found the key not correct.
export function NostrRow({ label, name, value, feedId, bad, person, onSet }) {
  const [open, setOpen] = useState(false);
  return html`<${NostrKeyRow} label=${label} name=${name} value=${value} bad=${bad} onOpen=${() => setOpen(true)} />
    <${NostrSheet} open=${open} value=${value} feedId=${feedId} person=${person} onSet=${onSet} onClose=${() => setOpen(false)} />`;
}

// The row of a Nostr key, with no sheet. A form puts the row in its group
// and `NostrSheet` after its end, because the sheet has its own form
// (ADR 0123). The row asks for the profile of its key (ADR 0147 items 3 and 4).
export function NostrKeyRow({ label, name, value, bad, onOpen }) {
  const profiles = useProfiles(value && !bad ? [value] : []);
  const { summary, art } = keyRowView(name, value, profiles);
  const row = html`<button class="link-row" type="button" onClick=${onOpen}>
      ${art && !bad && html`<${PersonArt} person=${art} small />`}
      <span class="label">${label}</span>
      <span class=${bad ? 'summary field-error' : 'summary'}>${bad ? 'Not a Nostr key' : summary}</span>
      <span class="go" aria-hidden="true">›</span>
    </button>`;
  return value ? html`<div class="key-row">${row}<${CopyButton} text=${value} label="the Nostr key" /></div>` : row;
}

const FAILED_TEXT = {
  collection: 'Studio could not read your feeds.',
  musicindex: 'Music Index did not answer.',
  relay: 'The search relay did not answer.',
};

// One profile of the answer of `GET /nostr/profiles`. The picture comes
// through the image route (ADR 0016).
function ProfileBody({ profile }) {
  const { src } = useImage(profile.picture);
  const inFeeds = profile.sources.some((s) => s.kind === 'collection');
  const sources = profile.sources.filter((s) => s.kind !== 'collection');
  const domain = profile.nip05 && profile.nip05.includes('@') ? profile.nip05.split('@').pop() : '';
  return html`<span style="flex:none;width:40px;height:40px;border-radius:8px;overflow:hidden;display:block">
      ${src && html`<img src=${src} alt="" width="40" height="40" />`}</span>
    <span class="text">
      <span class="pick-name"><span class="title">${profile.name || shortKey(profile.npub)}</span>${inFeeds && html`<${TakenTag} text="In your feeds" />`}</span>
      ${profile.nip05 && html`<span class="summary">${profile.nip05}${profile.nip05_ok ? ' ✓' : ''}</span>`}
      <span class="summary mono">${shortKey(profile.npub)}</span>
      ${sources.length > 0 && html`<span class="summary">${sources.map(sourceText).join(' · ')}</span>`}
      ${profile.domain_match && html`<span class="summary">Confirmed at ${domain}</span>`}
    </span>`;
}

// `value` is the key of the row, or `null`. A person entry gives `person`
// with its name and picture. A selection calls `onSet(npub, picked)`. The key
// `null` removes the key. `picked` has the profile name and picture that the
// user chose to use. The page of a person sets `offerPeople` to false, because
// a key of a different person is a merge (ADR 0137 item 3).
export function NostrSheet({ open, value, feedId, person, offerPeople = true, onSet, onClose }) {
  const ref = useSheet(open);
  const session = useContext(SessionContext);
  const mine = session && session.session && session.session.pubkey ? npubOf(session.session.pubkey) : null;
  const [changing, setChanging] = useState(false);
  const [query, setQuery] = useState('');
  const [found, setFound] = useState(null);
  const [people, setPeople] = useState(null);
  const [card, setCard] = useState(null);
  const [useName, setUseName] = useState(false);
  const [usePicture, setUsePicture] = useState(false);
  const [finding, setFinding] = useState(false);
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
    setPeople(null);
    if (offerPeople) api('/people').then(setPeople, () => setPeople(null));
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
  const peopleRows = filterRows(nostrPickRows(people, value), query);
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
          <label for="nostr-query">Name, npub or NIP-05 address</label>
          <div class="inline"><input id="nostr-query" type="search" value=${query} onInput=${(e) => setQuery(e.target.value)} />
            <button class="button" type="submit" disabled=${search.busy || !query.trim()}>${search.busy ? 'Searching…' : 'Search'}</button></div>
        </div>
        ${mine && html`<div class="actions"><button class="button" type="button" onClick=${() => choose(mine, null)}>Use My Key</button></div>`}
        <${ErrorNotice} error=${search.error} />
        ${person && html`<div class="group">
          <label class="check"><input type="checkbox" checked=${useName} onChange=${(e) => setUseName(e.target.checked)} />
            <span>Use the profile name</span></label>
          <label class="check"><input type="checkbox" checked=${usePicture} onChange=${(e) => setUsePicture(e.target.checked)} />
            <span>Use the profile picture</span></label>
        </div>`}
        <section class="page-section"><h2>From Music Index</h2>
          <div class="group"><button class="link-row" type="button" onClick=${() => setFinding(true)}>
            <span class="text"><span class="label">Find on Music Index</span>
              <span class="summary">Use the Nostr key of a feed. Few feeds have one.</span></span>
            <span class="go" aria-hidden="true">›</span></button></div></section>
        ${peopleRows.length > 0 && html`<${PickFold} title="Your people" rows=${peopleRows.map((r) => html`<button class="link-row" type="button" key=${r.id} onClick=${() => choose(r.npub, { name: r.name, picture: r.person.img })}>
            <${PersonArt} person=${r.person} /><span class="text"><span class="title">${r.name}</span>
              <span class="summary mono">${shortKey(r.npub)}</span></span></button>`)} />`}
        ${found && found.failed.length > 0 && html`<p class="hint" role="status">${found.failed.map((f) => FAILED_TEXT[f] || 'A source did not answer.').join(' ')}</p>`}
        ${found && found.profiles.length === 0 && html`<p class="empty">Studio found no profile.</p>`}
        ${found && found.profiles.length > 0 && html`<section class="page-section"><h2>Search results</h2><div class="group rows">
          ${found.profiles.map((found1) => withPersonValues(found1, people)).map((p) => html`<button class=${p.sources.some((s) => s.kind === 'collection') ? 'link-row pick-taken' : 'link-row'} type="button" key=${p.npub} onClick=${() => choose(p.npub, p)}>
            <${ProfileBody} profile=${p} /></button>`)}
        </div></section>`}
      </form>`}
    </div>
    <footer>
      <button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>
    </footer>
    <${FindOneSheet} open=${finding} start=${query} choices=${nostrChoices} empty="This feed has no Nostr key."
      onPick=${(found) => choose(found.npub, found)} onClose=${() => setFinding(false)} />
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
    // The core ignores `duration` (ADR 0094 item 2).
    duration_seconds: readsMetadata ? null : readDuration(facts.duration) ?? item.duration_seconds,
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
  // and the drop (ADR 0062 items 20 and 21, ADR 0125 item 3).
  const { over, handlers } = useFileDrop({ busy, onFiles: drop });
  const areaDrop = useFileDrop({ busy, onFiles: drop, stop: true });
  const rootProps = {
    class: `field${busy ? '' : ' drop-target'}${over ? ' drop-over' : ''}`,
    ...handlers,
  };
  const errorText = dropError || (error ? importErrorText(error) : null);
  return {
    field, kind, noun: FIELD_NOUN[field], open, busy, over, sent, size, errorText, rootProps, areaDrop, choose, upload,
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
    ${list.truncated && html`<p class="hint">The list stops at the limit of Studio. The file host has more files.</p>`}`;
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
  // A drop on the sheet must not reach the field below it. The drop area of
  // "This computer" takes a file before this handler.
  const stop = (e) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'none';
  };
  return html`<dialog ref=${ref} class="sheet" aria-labelledby=${`${inputId}-title`} onClose=${pick.closeSheet}
      onDragOver=${stop} onDrop=${stop}>
    <header>
      <h2 id=${`${inputId}-title`}>${title}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body">
      <div class="segmented picker-tabs" role="group" aria-label="Where the file comes from" onKeyDown=${onSegmentKey}>
        ${list.map((s) => html`<button class="segment" type="button" key=${s.id} aria-pressed=${source === s.id} tabIndex=${source === s.id ? 0 : -1}
          onClick=${() => setSource(s.id)}>${s.label}</button>`)}
      </div>
      ${source === 'release' && html`<${PickerList} path=${path} kind=${pick.kind} byName onSelect=${(file) => pick.choose(file, 'release')} />`}
      ${source === 'hosts' && hosts.length === 0 && html`<p class="hint">No file host is connected.</p>`}
      ${source === 'hosts' && host && html`<div class="field">
          <label for=${`${inputId}-host`}>File host</label>
          <div class="field-body"><select id=${`${inputId}-host`} value=${host.connectorId} onChange=${(e) => setHostId(Number(e.target.value))}>
            ${hosts.map((h) => html`<option value=${h.connectorId}>${h.name}</option>`)}
          </select></div>
        </div>
        <${PickerList} key=${host.connectorId} path=${path} kind=${pick.kind} onSelect=${(file) => pick.choose(file, 'hosts')} />`}
      ${source === 'computer' && html`<div class="group">
          <div class=${dropAreaClass(pick.areaDrop.over, pick.busy)} ...${pick.areaDrop.handlers}>
            <p class="hint drop-text">Drop a file here.</p>
            <input id=${inputId} class="visually-hidden" type="file" disabled=${pick.busy} accept=${list[2].accept}
              onChange=${(e) => { const [file] = e.target.files; e.target.value = ''; if (file) pick.upload(file); }} />
            <label for=${inputId} class="button action">${pick.busy ? `Sending… ${progressPercent(pick.sent, pick.size)}%` : 'Select a File'}</label>
          </div>
          <p class="hint">Studio sends the file to the release folder. The file host gets the file when you write.</p>
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
      ${!pick.busy && !pick.over && html`<span class="hint drop-text">or drop a file here</span>`}
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
  file_exists: 'A file with this name exists on the file host already.',
  no_public_address: 'This file host has no public address. Give it one in File hosts.',
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
        <span class="role">${text}${file.on_host ? ' · On the file host already' : ''}</span>
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

  const [refused, setRefused] = useState([]);
  const dropFiles = (files) => {
    const useful = files.filter(addsAsTrackFile);
    setRefused(files.filter((file) => !addsAsTrackFile(file)).map((file) => file.name));
    if (useful.length > 0) addFiles(useful);
  };
  const tracksDrop = useFileDrop({ busy: false, onFiles: dropFiles, stop: true });
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
    ? `＋ Add ${readyAudio.length} Track${readyAudio.length === 1 ? '' : 's'}`
    : '＋ Add the Album Art';

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
      pub_date_time: null,
      duration: null,
      duration_seconds: readDuration(row.facts.duration),
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
      <div class=${dropAreaClass(tracksDrop.over, false)} data-drop-many="" ...${tracksDrop.handlers}>
        <p class="hint drop-text">Drop music and image files here.</p>
        <p class="hint" id="import-input-hint">
          Studio reads the facts of each file, and makes one track for each audio file.
        </p>
        <input id="import-input" class="visually-hidden" type="file" multiple
          aria-describedby="import-input-hint"
          accept="audio/*,image/*,.wav,.flac,.m4a,.ogg,.opus"
          onChange=${(e) => { addFiles(e.target.files); e.target.value = ''; }} />
        <label for="import-input" class="button action">＋ Choose Music and Image Files</label>
        ${refused.length > 0 && html`<p class="field-error" role="alert">${refusedText(refused)}</p>`}
      </div>
      ${!assetMode().holdAssets && html`<div class="notice" role="status">${ADD_TRACKS_NOTICE}</div>`}
      ${rows.length === 0 && html`<button class="button link" type="button" onClick=${onAddress}>The Audio Is Already Online</button>`}
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

const VIEW_KEY = 'studio.review.view';
const VIEW_EVENT = 'studio-review-view';
let viewCounter = 0;

// The last view of the user, kept on this device only (ADR 0114 item 8).
function storedView() {
  try {
    return normalizeView(localStorage.getItem(VIEW_KEY));
  } catch {
    return 'simplified';
  }
}

function useReviewView() {
  const [view, setView] = useState(storedView);
  useEffect(() => {
    const onChange = () => setView(storedView());
    window.addEventListener(VIEW_EVENT, onChange);
    return () => window.removeEventListener(VIEW_EVENT, onChange);
  }, []);
  const choose = (next) => {
    try {
      localStorage.setItem(VIEW_KEY, next);
    } catch {
      // The choice then lasts for this page only.
    }
    setView(next);
    window.dispatchEvent(new Event(VIEW_EVENT));
  };
  return [view, choose];
}

// One row of the field view. The warning of a key shows in the error color.
function FieldRow({ row }) {
  return html`<div class="link-row static"><span class="text"><span class="label">${row.label}</span>
    <span class="summary">${row.text}</span>
    ${row.warning && html`<span class="summary field-error">${row.warning}</span>`}</span></div>`;
}

// "What changes": the field changes of the core in words (ADR 0114 items 1 to
// 5). The core gives the changes and the count of the lines that it does not
// explain.
function FieldView({ diff }) {
  const profiles = useProfiles(changeKeys(diff.fields));
  const groups = fieldRows(diff.fields, profiles);
  const guard = guardRow(diff.uncovered);
  if (groups.length === 0 && !guard) return null;
  // Adjacent groups with no label are rows of one channel field, and they
  // share one group. The guard row joins the last such group (mockup screen 4).
  const cards = [];
  groups.forEach((g) => {
    const last = cards[cards.length - 1];
    if (!g.label && last && last.plain) last.parts.push(g);
    else cards.push({ plain: !g.label, parts: [g] });
  });
  const guardView = guard && html`<div class="link-row static"><span class="text"><span class="label">${guard.title}</span>
    <span class="summary">${guard.text}</span></span></div>`;
  const lastPlain = cards.length > 0 && cards[cards.length - 1].plain;
  const part = (g, i) => html`
    ${g.label && html`<div class="link-row static" key=${`l${i}`}><span class="text"><span class="label">${g.label}</span>
      ${g.summary && html`<span class="summary">${g.summary}</span>`}</span></div>`}
    ${g.rows.map((row, j) => html`<${FieldRow} row=${row} key=${`${i}-${j}`} />`)}
    ${g.more && html`<details class="disclosure" key=${`m${i}`}><summary class="link-row"><span class="summary">${g.more.summary}</span></summary>
      <div class="form">${g.more.rows.map((row, j) => html`<${FieldRow} row=${row} key=${j} />`)}</div></details>`}`;
  return html`<section class="page-section fieldview"><h2>What changes</h2>
    ${cards.map((card, c) => html`<div class="group" key=${c}>
      ${card.parts.map(part)}
      ${lastPlain && c === cards.length - 1 && guardView}
    </div>`)}
    ${guard && !lastPlain && html`<div class="group">${guardView}</div>`}
  </section>`;
}

// The lines of a review: the view menu, the lines with the changed parts
// marked, and the links that open the hidden lines (ADR 0114 items 8 to 13).
// `diff` is the answer of the diff route.
export function DiffView({ diff }) {
  const [view, chooseView] = useReviewView();
  const [whole, setWhole] = useState(false);
  const [opened, setOpened] = useState(() => new Set());
  const narrow = useMedia('(max-width: 760px)');
  const idRef = useRef(null);
  if (idRef.current === null) {
    viewCounter += 1;
    idRef.current = `review-view-${viewCounter}`;
  }
  const rows = shownLines(diff.lines || []);
  const segments = runs(rows, whole);
  const open = (id) => setOpened((prev) => new Set(prev).add(id));
  const shown = segments.flatMap((s) => (s.hidden ? (opened.has(s.id) ? s.rows : []) : [s.row]));
  const indent = narrow ? commonIndent(shown) : 0;
  const line = (row, key) => {
    const note = localTimeNote(row);
    return html`<span key=${key} class=${`line ${row.kind === 'same' ? '' : row.kind}`}>${row.sign} ${
      (indent ? stripIndent(row.parts, indent) : row.parts).map((part, i) => (part.changed
        ? html`<mark key=${i} class=${row.kind === 'remove' ? 'del' : 'ins'}>${part.text}</mark>`
        : part.text))}\n</span>${note && html`<span class="line time-note">  ${note}\n</span>`}`;
  };
  return html`<div class="review-lines">
    <div class="viewbar">
      <div class="field">
        <label for=${idRef.current}>View</label>
        <div class="field-body">
          <span class="hint" id=${`${idRef.current}-hint`}>${VIEW_HINTS[view]}</span>
          <select id=${idRef.current} value=${view} aria-describedby=${`${idRef.current}-hint`}
            onChange=${(e) => chooseView(normalizeView(e.target.value))}>
            ${VIEWS.map((v) => html`<option value=${v}>${VIEW_LABELS[v]}</option>`)}
          </select>
        </div>
      </div>
      ${view === 'detailed' && html`<button class="button" type="button" onClick=${() => setWhole(!whole)}>${whole ? 'Show Only the Changes' : 'Show the Whole File'}</button>`}
    </div>
    ${view === 'simplified' && html`<${FieldView} diff=${diff} />`}
    ${view === 'detailed' && html`<section class="page-section">
    <pre class="diff" tabindex="0" aria-label="Changes between the hosted file and the draft">${segments.map((s, index) => {
      if (!s.hidden) return line(s.row, `r${index}`);
      if (opened.has(s.id)) return s.rows.map((row, i) => line(row, `o${s.id}-${i}`));
      return html`<span key=${`h${s.id}`} class="line"><a class="more" href=${`#diff-run-${s.id}`}
        onClick=${(e) => { e.preventDefault(); open(s.id); }}><span aria-hidden="true">⋯ </span>${moreLabel(s.count)}</a>\n</span>`;
    })}</pre></section>`}
  </div>`;
}

// The merge of the hosted file into the draft (ADR 0129). The core gives the
// plan. A field that both sides changed has a select menu with no start value.
function MergePanel({ merge, busy, onChoose, onApply }) {
  const rows = mergeRows(merge.plan);
  const list = (title, items, text) => items.length > 0 && html`<section class="page-section"><h2>${title}</h2>
    <div class="group">${items.map((row) => html`<div class="link-row static" key=${row.id}><span class="text">
      <span class="label">${row.label}</span><span class="summary">${text(row)}</span></span></div>`)}</div></section>`;
  return html`<div class="merge-panel">
    <div class="notice" role="status"><strong>${MERGE_TITLE}</strong>${MERGE_INTRO}</div>
    ${list(MERGE_HEADINGS.kept, rows.kept, (row) => row.mine)}
    ${list(MERGE_HEADINGS.incoming, rows.incoming, (row) => row.hosted)}
    ${rows.conflicts.length > 0 && html`<section class="page-section"><h2>${MERGE_HEADINGS.conflicts}</h2>
      <div class="group">${rows.conflicts.map((row) => html`<div class="field" key=${row.id}>
        <label for=${`merge-${row.id}`}>${row.label}</label>
        <div class="field-body"><select id=${`merge-${row.id}`} value=${merge.choices[row.id] || ''}
          onChange=${(e) => onChoose(row.id, e.target.value)}>
          <option value="">Select one</option>
          <option value="mine">Your edit: ${row.mine}</option>
          <option value="hosted">Hosted file: ${row.hosted}</option>
        </select></div></div>`)}</div></section>`}
    <div class="actions"><button class="button primary" type="button"
      disabled=${busy || !mergeReady(merge.plan, merge.choices)} onClick=${onApply}>Apply to the Draft</button></div>
  </div>`;
}

// ---- Host operations, in the diff sheet and the Files page (task 021g,
// ADR 0036 items 10 to 14) ----

// The name of a connected host, by its ID. A host that the list does not
// have yet, for example right after a page load, falls back to a plain
// word.
function connectorName(connectors, id) {
  const found = connectors.find((c) => c.id === id);
  return found ? found.name : 'File host';
}

function operationKindLabel(kind) {
  switch (kind) {
    case 'upload': return 'Upload';
    case 'copy': return 'Copy';
    case 'delete': return 'Delete';
    case 'confirm': return 'Confirm';
    case 'hold': return 'Keep';
    default: return kind;
  }
}

// The second line of an operation row: what the operation means for the
// feed (task 022b, review of milestone 7a).
const OPERATION_NOTE = {
  upload: 'The feed uses this file.',
  copy: 'A spare copy. The feed does not use it.',
  delete: 'Studio deletes it after the feed is written.',
  confirm: 'Studio makes sure that the file host gives this file.',
  hold: 'The old copy stays. Studio asks you before it deletes this file.',
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
      ${move.replaces_pointer && html`<span class="sub">${REPLACES_POINTER_NOTE}</span>`}
      <span class="sub">${connectorName(connectors, move.connector_id)}</span>
    </span>
    <button class="button" type="button" disabled=${busy} onClick=${onUndo}>Undo</button>
  </div>`;
}

// The refusal of a write by the core, with each hop and the repair buttons
// (ADR 0126 items 10 and 20). The core gives the condition, the hops and the
// repairs. This function only shows them.
function LinkRefusal({ refusal, busy, onRepair }) {
  const view = refusalView(refusal);
  if (!view) return null;
  return html`<div class="notice error" role="alert"><strong>${view.title}</strong>
    <p>${view.source} ${view.text} ${view.danger}</p>
    ${view.hops.length > 0 && html`<ol class="hops">${view.hops.map((hop, i) => html`<li key=${i}>
      <span class="mono">${hop.url}</span>
      ${hop.finalUrl && html`<span class="why">Redirects to <span class="mono">${hop.finalUrl}</span></span>`}
      <span class="why">${hop.why}</span></li>`)}</ol>`}
    <p>${view.state}</p>
    <p>${view.next}</p>
    ${view.repairs.length > 0 && html`<div class="actions">${view.repairs.map((r) => html`<button class="button" type="button"
      disabled=${busy} onClick=${() => onRepair(r.id)}>${r.label}</button>`)}</div>`}
  </div>`;
}

// The notice of a self link that differs from the address of the feed
// (ADR 0126 item 6). The write stays possible.
function LinkNotices({ report, busy, onRepair }) {
  return noticeViews(report).map((view) => html`<div class="notice" role="status"><strong>${view.title}</strong>The feed names${' '}<span class="mono">${view.selfUrl}</span>. Its address is <span class="mono">${view.draftUrl}</span>. ${view.why} ${view.end}
    ${view.repairs.length > 0 && html`<div class="actions">${view.repairs.map((r) => html`<button class="button" type="button"
      disabled=${busy} onClick=${() => onRepair(r.id)}>${r.label}</button>`)}</div>`}
  </div>`);
}

// The warning of a return (ADR 0128 items 7 to 9). It does not stop the write.
function ReturnWarning({ move }) {
  const view = returnWarning(move);
  return view && html`<div class="notice" role="status"><strong>${view.title}</strong>${view.text}</div>`;
}

// The notices of the build date (ADR 0127 items 8, 11 and 12). The core gives
// each state, and no notice stops a write.
function BuildDateNotices({ state }) {
  return buildDateNotices(state).map((view) => html`<div class="notice" role="status"><strong>${view.title}</strong>${view.text}
    ${view.value && html`${' '}<span class="mono">${view.value}</span>`}</div>`);
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

// The one announcement choice of a write (ADR 0136). Studio remembers it for
// the next write. `onChange` gets the diff again after a change.
export function AnnounceChoice({ onChange }) {
  const [enabled, setEnabled] = useState(null);
  const { busy, error, run } = useAction();
  useEffect(() => { api('/settings/announcements').then((result) => setEnabled(result.enabled), () => {}); }, []);
  if (enabled === null) return null;
  const change = (next) => run(async () => {
    const result = await api('/settings/announcements', { method: 'PUT', body: { enabled: next } });
    setEnabled(result.enabled);
    if (onChange) await onChange();
  });
  return html`<div class="announce-choice">
    <label class="check"><input type="checkbox" checked=${enabled} disabled=${busy} onChange=${(e) => change(e.target.checked)} />
      <span>Tell Podcast Index and send a podping</span></label>
    <p class="hint">${enabled ? 'Studio sends it 30 seconds after the write.' : 'No index learns about this write until it reads the feed by itself.'}</p>
    <${ErrorNotice} error=${error} />
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

// The grant control of each pending show of the draft (ADR 0054 item 11,
// ADR 0110 item 19). `grantControl` of `apps.js` chooses the control: a
// check box for one app, a select menu for two or more apps and a disabled
// menu for no app. The control asks the grant route and takes the new diff
// from its response, so the sheet needs no second request. `diff.grants`
// gives the GUID of each show that has a grant request, and `diff.apps` and
// `diff.grant_apps` give the active apps and the app of each grant.
// A host with "My signer" cannot do a granted write (ADR 0058 item 14).
// `signerHost` is true for that host. The control is then disabled, with the
// reason, unless the draft already has the grant. The user can remove it.
function GrantChecks({ feedId, model, diff, busy, signerHost, onDiff, run }) {
  const shows = ((model && model.live_items) || []).filter((l) => l.status === 'pending' && l.item && l.item.guid);
  if (shows.length === 0) return null;
  const change = (guid, appId) => run(async () => {
    const path = `/feeds/${feedId}/grants/${encodeURIComponent(guid)}`;
    onDiff(await api(path, appId === null ? { method: 'DELETE' } : { method: 'PUT', body: { app_id: appId } }));
  });
  return html`<div class="group grant-checks">
    ${shows.map((l) => {
      const guid = l.item.guid;
      const c = grantControl({
        apps: diff.apps || [], grantApps: diff.grant_apps || {}, guid,
        signerNote: grantBoxState(signerHost).note,
        showTitle: shows.length > 1 ? l.item.title : undefined,
      });
      const id = `grant-${guid}`;
      return html`<div key=${guid}>
        ${c.kind === 'check'
          ? html`<label class="check"><input type="checkbox" disabled=${busy || c.disabled} checked=${c.checked}
              onChange=${(e) => change(guid, e.target.checked ? c.appId : null)} />
              <span>${c.label}</span></label>`
          : html`<div class="field"><label for=${id}>${c.label}</label>
              <div class="field-body"><select id=${id} disabled=${busy || c.disabled} value=${c.value}
                onChange=${(e) => change(guid, e.target.value === '' ? null : Number(e.target.value))}>
                ${c.options.map((o) => html`<option value=${o.value}>${o.text}</option>`)}
              </select></div></div>`}
        ${c.notes.map((note) => html`<p class="hint" key=${note}>${note}</p>`)}
      </div>`;
    })}
  </div>`;
}

// The name of the event that tells the feed page to load the announcements
// again, such as after a write.
export const ANNOUNCE_RELOAD = 'studio-announce-reload';

// A page sends this window event to open the review sheet, for example after
// "Go live". `DraftToolbar` listens to it.
export const OPEN_REVIEW = 'studio-open-review';
// Opens the Review sheet on the merge of the hosted file (ADR 0150 item 4).
export const OPEN_MERGE = 'studio-open-merge';

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

// Sign the planned events of a diff with the signer of the user. `choice` is
// the result of `signerChoice`. With no signer, nothing goes to the host. The
// review sheet and the write of a change set use this function.
export async function signPlanned(toSign, choice, onProgress) {
  if (toSign.length === 0) return [];
  if (choice.kind === 'differs') throw new SignError('signer_key_differs', signErrorText('signer_key_differs'));
  let signer = window.nostr;
  if (!signer && choice.kind === 'remote') {
    signer = remoteSigner((event) => api('/signer/sign', { method: 'POST', body: { event } }));
  }
  if (!signer) throw new SignError('no_signer', signErrorText('no_signer'));
  return signAll(toSign, signer, onProgress);
}

// The signer choice for the planned events of a diff, with the state of the
// NIP-46 signer from the API when the browser has no NIP-07 signer.
export async function currentSignerChoice(toSign) {
  const hasNip07 = !!window.nostr;
  const remote = hasNip07 || toSign.length === 0
    ? null
    : await api('/signer').catch(() => ({ state: 'none', pubkey: null }));
  return signerChoice({ hasNip07, remote, events: toSign });
}

// The review sheet: the draft status, the diff and the write (ADR 0029 item
// 2). It is the same content as the former Changes panel.
function ReviewSheet({ feedId, feedConnectorId, draft, open, mergeRequest = 0, onWritten, onClose, connectors, onOpenHosts, onOpsChanged, onOpsCleared }) {
  const ref = useSheet(open);
  const [diff, setDiff] = useState(null);
  const { busy, error, run } = useAction();
  // The progress of the signer while a write signs the planned events.
  const [progress, setProgress] = useState(null);
  // A diff is for one draft. After a user edit, the diff is not current.
  useEffect(() => { setDiff(null); setLinks(null); }, [draft.edits]);
  // The check of the links of the draft (ADR 0126 item 13). A request that
  // fails gives no block: the write runs the same check again in the core.
  const [links, setLinks] = useState(null);
  const checkLinks = async () => {
    try { setLinks(await api(`/feeds/${feedId}/link-check`)); } catch (_) { setLinks(null); }
  };
  const examine = () => run(async () => {
    if (!(await draft.flush())) return;
    setDiff(await api(`/feeds/${feedId}/diff`));
    await checkLinks();
  });
  const refreshDiff = async () => setDiff(await api(`/feeds/${feedId}/diff`));
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
    await checkLinks();
  });
  // A repair of ADR 0126 item 20 changes the draft in the core. The sheet
  // loads the draft again and examines the diff again.
  const repair = (id) => run(async () => {
    await draft.flush();
    const current = await api(`/feeds/${feedId}/draft`);
    draft.show(await api(`/feeds/${feedId}/draft/${repairRoute(id)}`, { method: 'POST', body: { revision: current.revision } }));
    setDiff(await api(`/feeds/${feedId}/diff`));
    await checkLinks();
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
      if (toSign.length > 0) setProgress({ done: 0, total: toSign.length });
      signedEvents = await signPlanned(toSign, choice, (done, total) => setProgress({ done, total }));
      feed = await api(`/feeds/${feedId}/write`, { method: 'POST', body: { diff_id: diff.id, signed_events: signedEvents } });
    } catch (e) {
      // After a conflict, the diff is not against the hosted file. Hide it
      // and its write button.
      if (e.code === 'conflict') setDiff(null);
      // After an expired plan, the new diff has a new plan.
      if (e.code === 'plan_expired') setDiff(await api(`/feeds/${feedId}/diff`));
      // After a refusal of stage 2, the diff is not the same: the move
      // has a placed file (ADR 0126 item 19). Examine again.
      if (e.code === 'bad_move_link') {
        try { setDiff(await api(`/feeds/${feedId}/diff`)); } catch (_) { /* The user can examine again. */ }
        await checkLinks();
      }
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
  const discard = async () => {
    if (!await askConfirm(CONFIRMS.discardDraft())) return;
    run(async () => {
      await api(`/feeds/${feedId}/draft`, { method: 'DELETE' });
      setDiff(null);
      onOpsCleared();
      await draft.load();
    });
  };
  // A merge keeps each edit of the draft (ADR 0129). The Review sheet then
  // examines the new draft.
  const [merge, setMerge] = useState(null);
  const startMerge = () => run(async () => {
    await draft.flush();
    const found = await api(`/feeds/${feedId}/draft/merge`);
    setMerge({ ...found, choices: {} });
  });
  useEffect(() => { if (mergeRequest > 0) startMerge(); }, [mergeRequest]);
  const applyMerge = () => run(async () => {
    try {
      draft.show(await api(`/feeds/${feedId}/draft/merge`, { method: 'POST', body: { version: merge.version, choices: merge.choices } }));
    } catch (e) {
      // The hosted file changed again, so this plan is old.
      if (e.code === 'conflict') setMerge(null);
      throw e;
    }
    setMerge(null);
    setDiff(await api(`/feeds/${feedId}/diff`));
    await checkLinks();
  });
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
  // The refusal of the check, else the refusal of the last write.
  const refusal = (links && links.refusal) || refusalOfError(error);
  // Studio cannot write while an address of the draft is not correct. A
  // host operation of `diff` does not block the write on its own: Studio
  // skips a deletion that `diff.blocked` names, and the user sees it here.
  const addressesBlocked = draft.bad.length > 0;
  const stale = error && error.code === 'diff_not_approved';
  // The files that the write sends. On a server that holds no assets,
  // Studio then removes its copies (ADR 0105 item 19).
  const sentCount = diff && !assetMode().holdAssets ? diff.operations.filter((op) => op.kind === 'upload').length : 0;
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
            <div class="actions"><button class="button" type="button" onClick=${draft.load}>Load the Draft Again</button></div></div>`
        : html`<${ErrorNotice} error=${draft.error} />`}
      ${merge
        ? html`<${MergePanel} merge=${merge} busy=${busy} onApply=${applyMerge}
            onChoose=${(id, side) => setMerge((m) => {
              const choices = { ...m.choices };
              if (side) choices[id] = side; else delete choices[id];
              return { ...m, choices };
            })} />`
        : conflict
        ? html`<div class="notice error" role="alert"><strong>The hosted file changed after the draft started.</strong>
            Studio did not write. A different tool or a different person changed the file.
            <div class="actions"><button class="button" type="button" disabled=${busy} onClick=${startMerge}>Apply My Edits to the New File</button></div></div>`
        : stale
          ? html`<div class="notice error" role="alert"><strong>The draft changed after you examined it.</strong>Examine the changes again.</div>`
          : refusalOfError(error)
          ? null
          : signErrorText(error && error.code)
            ? html`<div class="notice error" role="alert"><strong>${signErrorText(error.code)}</strong></div>`
            : html`<${ErrorNotice} error=${error} />`}
      ${addressesBlocked && html`<div class="notice error" role="alert">
        <strong>Studio cannot write while an address is not correct.</strong>
        Correct the fields with red text. Each address starts with https:// or http://.
        <ul class="address-list">${draft.bad.map((b) => html`<li><span class="mono">${b.field}</span>: <span class="mono">${b.value}</span></li>`)}</ul>
      </div>`}
      ${refusal && html`<${LinkRefusal} refusal=${refusal} busy=${busy} onRepair=${repair} />`}
      ${diff && html`<${HostOperations} diff=${diff} connectors=${connectors} onRemove=${removeOperation} onUndoMove=${undoMove} busy=${busy} />`}
      ${diff && html`<${ReturnWarning} move=${diff.feed_move} />`}
      ${diff && html`<${LinkNotices} report=${links} busy=${busy} onRepair=${repair} />`}
      ${diff && html`<${BuildDateNotices} state=${diff.build_date} />`}
      ${sentCount > 0 && html`<div class="notice" role="status">${reviewNotice(sentCount)}</div>`}
      ${choosing && html`<div class="group rows">
        <p class="hint">Studio cannot write to the file host of this feed. Connect that file host, or move the feed to a file host that you connected.</p>
        <div class="actions">
          <button class="button action" type="button" disabled=${busy} onClick=${() => { ref.current.close(); onOpenHosts(); }}>Connect the File Host of This File</button>
        </div>
        <div class="field"><label for="move-host">Move to a file host</label>
          ${targets.length === 0
            ? html`<p class="hint">No file host is connected yet. Connect a file host in File hosts first.</p>`
            : html`<select id="move-host" value=${targetId} onChange=${(e) => setTarget(Number(e.target.value))}>
                ${targets.map((c) => html`<option value=${c.id}>${c.name}</option>`)}
              </select>`}
        </div>
        <div class="actions">
          <button class="button action" type="button" disabled=${busy || targetId === null} onClick=${() => setHost(targetId)}>Move to This File Host</button>
        </div>
      </div>`}
      ${diff && host.kind === 'move' && html`<div class="notice" role="status">
        The file at the address before the move stays and does not change. Apps that read that address continue to get the version from before the move.</div>`}
      ${hasWork && draft.model && draft.hidden && html`<div class="notice" role="status">
        <strong>This feed is hidden from apps.</strong> Apps and directories do not list it after the write.</div>`}
      ${hasWork && html`<${AnnounceLines} diff=${diff} />`}
      ${diff && html`<${GrantChecks} feedId=${feedId} model=${draft.model} diff=${diff} busy=${busy} signerHost=${signerHost} onDiff=${setDiff} run=${run} />`}
      ${diff && (diff.changed
        ? html`<${DiffView} diff=${diff} />`
        : !hasWork && html`<p>The draft is the same as the hosted file.</p>`)}
      ${waiting && html`<p class="hint" role="status">Wait for each download, then write.</p>`}
      ${diff && toSign.length > 0 && html`<p class="hint" role="status">${signSummary(toSign.length)}</p>`}
      ${diff && choice.kind === 'differs' && html`<div class="notice error" role="alert"><strong>${signErrorText('signer_key_differs')}</strong></div>`}
      ${diff && needsSigner && choice.kind !== 'loading' && !connecting && html`<div class="actions">
        <button class="button action" type="button" disabled=${busy} onClick=${() => setConnecting(true)}>
          ${choice.kind === 'differs' ? 'Connect a Different Signer' : 'Connect a Signer'}</button>
      </div>`}
      ${diff && connecting && html`<${SignerConnect} onConnected=${connected} onCancel=${() => setConnecting(false)} />`}
      ${!merge && html`<div class="actions">
        <button class="button" type="button" disabled=${busy || draft.status === 'loading'} onClick=${examine}>Examine the Changes</button>
      </div>`}
      ${diff && hasWork && !addressesBlocked && !choosing && diff.announce_skip !== 'no_address' && html`<${AnnounceChoice} onChange=${refreshDiff} />`}
    </div>
    <footer>
      <button class="button destructive" type="button" disabled=${busy || !draft.changed} onClick=${discard}>Discard the Draft</button>
      ${diff && hasWork && !addressesBlocked && !choosing && html`<button class="button primary" type="button" disabled=${busy || waiting || needsSigner || blocksWrite(links)} onClick=${write}>
        ${progress ? signProgressLabel(progress) : busy ? 'Writing…' : 'Write to the File Host'}</button>`}
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
  const restore = async (snapshot) => {
    if (draft.changed && !await askConfirm(CONFIRMS.replaceDraft())) return;
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
          <span class="mono sub">${versionLabel(s.version)}</span></span>
        <button class="button" type="button" disabled=${busy} onClick=${() => restore(s)}>Put in the Draft</button>
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
  const [mergeRequest, setMergeRequest] = useState(0);
  useEffect(() => {
    const openReview = () => setOpen('review');
    window.addEventListener(OPEN_REVIEW, openReview);
    const openMerge = () => { setOpen('review'); setMergeRequest((n) => n + 1); };
    window.addEventListener(OPEN_MERGE, openMerge);
    return () => {
      window.removeEventListener(OPEN_REVIEW, openReview);
      window.removeEventListener(OPEN_MERGE, openMerge);
    };
  }, []);
  const status = toolbarStatus(draft);
  return html`<div class="draft-toolbar">
    <span class=${`draft-status${status.error ? ' error' : ''}`} role="status">${status.text}</span>
    <button class="button" type="button" onClick=${() => setOpen('versions')}>Versions</button>
    <button class="button primary" type="button" onClick=${() => setOpen('review')}>Review</button>
    <${ReviewSheet} feedId=${feedId} feedConnectorId=${feedConnectorId} draft=${draft} open=${open === 'review'} mergeRequest=${mergeRequest}
      onWritten=${onWritten} onClose=${() => setOpen(null)} connectors=${connectors} onOpenHosts=${onOpenHosts}
      onOpsChanged=${onOpsChanged} onOpsCleared=${onOpsCleared} />
    <${VersionsSheet} feedId=${feedId} draft=${draft} open=${open === 'versions'} onClose=${() => setOpen(null)} />
  </div>`;
}

// ---- The editor pages (ADR 0029 items 5 to 10) ----

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
  const remove = async () => {
    const name = feed.title || feed.path || 'this feed';
    if (!await askConfirm(CONFIRMS.removeFeed(name))) return;
    run(async () => {
      await api(`/feeds/${feed.id}`, { method: 'DELETE' });
      onRemoved();
    });
  };
  const tag = newFeedAddress(feed);
  // "Remove from Collection" is a row of its own below the facts, so that it
  // shows with the facts closed.
  return html`<details class="disclosure">
    <summary class="link-row"><span class="label">Feed facts</span><span class="summary">GUID, file, version and announcement</span></summary>
    <div class="form">
      <dl class="facts">
        <dt>Address source</dt><dd>${feed.public_url ? addressSource(feed.address_source) : '—'}</dd>
        ${tag && html`<dt>New feed address</dt><dd class="mono">${tag.url}</dd>`}
        ${tag && tag.note && html`<dd class="facts-note">${tag.note}</dd>`}
        <dt>Feed GUID</dt><dd class="mono">${feed.podcast_guid || '—'}</dd>
        ${feed.placeholder_guid && html`<dd class="facts-note">${HOST_STEP.placeholderGuid}</dd>`}
        <dt>File</dt><dd class="mono">${feed.path || 'Read-only. Connect the folder that holds this file in File hosts.'}</dd>
        <dt>Version</dt><dd class="mono">${feed.last_version ? versionLabel(feed.last_version) : 'Not read yet'}</dd>
        <dt>Announcement</dt><dd>${announcementText.split('\n').map((line, i) => html`${i > 0 && html`<br />`}${line}`)}</dd>
      </dl>
    </div>
  </details>
  <div class="actions"><button class="button destructive" type="button" disabled=${busy} onClick=${remove}>Remove from Collection</button></div>
  <${ErrorNotice} error=${error} />`;
}

// The section "Artists and labels" of an album or a playlist: its publisher
// feeds, in the sequence of `podcast:publisher` (ADR 0041 item 9, ADR 0086
// item 4). Each link changes this draft and the draft of the other feed.
function ArtistsAndLabels({ feed, feeds, model, update, draft, go, onListChanged }) {
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
  return html`<section class="page-section"><h2>Linked artist and label pages</h2>
    <div class="group">
      <${ChangedNotices} changed=${changed} go=${go} />
      ${rows.length === 0 && html`<p class="hint">Not linked to an artist or label page yet.</p>`}
      ${rows.map((row) => html`<div class="release-row" key=${row.entry._cid}>
        <span class="title">${row.entry.title || row.entry.feed_url}</span>
        <span class="sub">${publisherLine(row.entry, row.match, row.index)}</span>
        <div class="actions">
          <button class="button" type="button" disabled=${row.index === 0 || linkAction.busy}
            aria-label=${`Move ${row.entry.title || 'this feed'} up`}
            onClick=${() => movePublisher(row.index, -1)}>Move Up</button>
          <button class="button" type="button" disabled=${row.index === rows.length - 1 || linkAction.busy}
            aria-label=${`Move ${row.entry.title || 'this feed'} down`}
            onClick=${() => movePublisher(row.index, 1)}>Move Down</button>
          <button class="button destructive" type="button" disabled=${linkAction.busy}
            aria-label=${`Remove ${row.entry.title || 'this feed'}`}
            onClick=${() => removeRow(row)}>Remove</button>
        </div>
      </div>`)}
      ${candidates.length > 0 && html`<div class="field">
        <label for="album-publisher">Link to an artist or label page</label>
        <div class="field-body">
          <select id="album-publisher" value=${chosen} onChange=${(e) => setChosen(e.target.value)}>
            <option value="">Select a page</option>
            ${candidates.map((f) => html`<option value=${f.id}>${f.title || f.path || 'Untitled feed'}</option>`)}
          </select>
        </div>
      </div>
      <div class="actions"><button class="button action" type="button" disabled=${!chosen || linkAction.busy}
        onClick=${addPublisher}>Link to This Page</button></div>`}
      ${candidates.length === 0 && html`<p class="hint">Studio has no other artist or label page to link to.</p>`}
      <${ErrorNotice} error=${publisherError(linkAction.error, 'Open that feed and select its role first.')} />
    </div>
  </section>`;
}

// The album page: the album fields, the album details and the tracks
// (task 014c step 3).
// The second line of a publisher row on the album page: the role, the link
// state and the primary mark (ADR 0041 items 6 and 9).
function publisherLine(entry, match, index) {
  const parts = [relLabel(entry.rel)];
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
            <button class="button action" type="button" onClick=${() => change(conflict)}>Use This Address</button>
          </div>`}
        <//>`;
}

// The row "Artwork" of a page. It opens the Artwork sheet (ADR 0070 item 9)
// and shows the image of the page (ADR 0147 item 2).
function ArtworkRow({ summary, image, onOpen }) {
  const { src } = useImage(image || null);
  return html`<button class="link-row" type="button" onClick=${onOpen}>
      ${src && html`<img class="row-preview" src=${src} alt="" />`}
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
  return html`<${NostrRow} label="Nostr profile" value=${key} feedId=${feedId}
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
    artwork: () => html`<${ArtworkRow} summary=${artworkSummary(model, 'feed')} image=${artworkPreview(model, 'feed')} onOpen=${() => onArtwork({ kind: 'feed' })} />`,
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
            <span class="hint">${LOCK_HELP}</span>
          </div>
        </div>
        ${locked && html`<${TextField} id="set-lock-owner" label="Lock owner email" hint="Optional. A platform can use this address to confirm that you own the feed. The address is public in the feed." value=${locked.owner} onChange=${setLockOwner} />`}
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
            ${model.op3 && model.podcast_guid && html`<div class="actions"><a class="button action" href=${op3Page(model.podcast_guid)} target="_blank" rel="noopener">OP3 Statistics</a></div>`}
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

function AlbumPage({ feed, feeds, model, update, bad, draft, go, onAdding, onImporting, onRemoved, onListChanged, onArtwork }) {
  const openPage = (page) => go({ name: 'feed', id: feed.id, ...page });
  // No wrapper: each section is a direct row of .page, with the gap of
  // .page between sections (review of task 014c, defect 2).
  const shown = {
    album: (s) => html`<section class="page-section"><h2>Album</h2>
      <div class="group">
        <${ChannelFields} feedId=${feed.id} draft=${draft} model=${model} update=${update} bad=${bad} artistLabel="Artist" onArtwork=${onArtwork} fields=${s.fields} />
      </div>
    </section>`,
    publishers: () => html`<${ArtistsAndLabels} feed=${feed} feeds=${feeds} model=${model} update=${update} draft=${draft} go=${go} onListChanged=${onListChanged} />`,
    feed: (s) => html`<section class="page-section"><h2>Feed</h2>
      <div class="group">
        ${showEach(s.rows, {
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
            ${(item.duration_seconds != null ? durationText(item.duration_seconds) : item.duration) && html`<span class="dur">${item.duration_seconds != null ? durationText(item.duration_seconds) : item.duration}</span>`}
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
function PlaylistPage({ feed, feeds, draft, model, update, bad, go, onRemoved, onListChanged, onArtwork }) {
  const shown = {
    publishers: () => html`<${ArtistsAndLabels} feed=${feed} feeds=${feeds} model=${model} update=${update} draft=${draft} go=${go} onListChanged=${onListChanged} />`,
    channel: (s) => html`<section class="page-section"><h2>Playlist</h2>
      <div class="group">
        <${ChannelFields} feedId=${feed.id} draft=${draft} model=${model} update=${update} bad=${bad} artistLabel="Author" onArtwork=${onArtwork} fields=${s.fields} />
      </div>
    </section>`,
    feed: (s) => html`<section class="page-section"><h2>Feed</h2>
      <div class="group">
        ${showEach(s.rows, {
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
        <div class="actions"><button class="button action" type="button" onClick=${() => setPicking(true)}>Add a Track</button></div>
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
  const fields = useFieldList({ id: 'playlist-search', value: query, setValue: setQuery });
  const [note, setNote] = useState(null);
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
    setNote(null);
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
      setHits(null);
      setNote(null);
      const found = await api(`/search?q=${encodeURIComponent(query.trim())}`);
      setHits(found.feeds);
      setNote(found.note);
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
  return html`<dialog ref=${ref} class="sheet search-sheet" aria-labelledby="playlist-picker-title" onClose=${onClose}>
    <header>
      <h2 id="playlist-picker-title">Add tracks</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body">
      <div class="segmented playlist-tabs" role="group" aria-label="Where the tracks come from" onKeyDown=${onSegmentKey}>
        <button class="segment" type="button" aria-pressed=${tab === 'index'} tabIndex=${tab === 'index' ? 0 : -1} onClick=${() => setTab('index')}>Search Music Index</button>
        <button class="segment" type="button" aria-pressed=${tab === 'mine'} tabIndex=${tab === 'mine' ? 0 : -1} onClick=${() => setTab('mine')}>My Releases</button>
      </div>
      ${tab === 'index' && html`<section class="page-section pinned-form">
        <form class="form" onSubmit=${runSearch} role="search">
          <div class="field">
            <label for="playlist-search">Search Music Index</label>
            <div class="inline">
              <input id="playlist-search" type="search" value=${query} placeholder="Artist or feed name"
                ...${fields.input} />
              <button class="button" type="submit" disabled=${search.busy || !query.trim()}>
                ${search.busy ? 'Searching…' : 'Search'}</button>
            </div>
            ${fields.list}
          </div>
        </form></section>
      <section class="page-section">
        <${ErrorNotice} error=${search.error} />
        <${ErrorNotice} error=${load.error} />
        ${note && html`<p class="hint" role="status">${note.message}</p>`}
        ${hits && hits.length === 0 && !note && html`<p class="hint">Music Index found no feed with that name.</p>`}
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

// The facts of the image at an address (ADR 0064 items 10 to 12, ADR 0109).
// The browser reads the size of the image from its host. The core reads the
// type of the response (item 7 of the ADR). A held file comes through
// Studio. A failure gives `null`.
async function readImageFacts(url) {
  try {
    const source = mediaSource(url, await heldFilesNow());
    let src;
    let type = null;
    if (source.kind === 'held') {
      const blob = await api(`/images?file=${source.file}`, { raw: true });
      type = blob.type;
      src = await new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(reader.result);
        reader.onerror = reject;
        reader.readAsDataURL(blob);
      });
    } else if (source.kind === 'direct') {
      src = source.src;
      const facts = await api('/media/facts', { method: 'POST', body: { url: source.src } }).catch(() => null);
      type = facts && facts.media_type;
    } else {
      return null;
    }
    const size = await new Promise((resolve, reject) => {
      const image = new Image();
      image.onload = () => resolve({ width: image.naturalWidth, height: image.naturalHeight });
      image.onerror = reject;
      image.src = src;
    });
    if (!(size.width > 0 && size.height > 0)) return null;
    return imageFacts(size.width, size.height, type);
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
    (duration) => editItem((i) => set(i, 'duration_seconds', duration)),
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
  const changeCredits = async (e) => {
    // The event target is not valid after the wait, so keep the select.
    const select = e.target;
    if (select.value === 'track') { useTrackCredits(); return; }
    if (await askConfirm(CONFIRMS.removeTrackCredits())) {
      editItem(withAlbumCredits);
    } else {
      select.value = 'track';
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
          audio: () => html`<${Enclosure} id=${id} path=${path} bad=${bad} enclosure=${item.enclosure} onChange=${change('enclosure')} pick=${pickAudio}
            duration=${{ value: item.duration_seconds, sourceText: item.duration, onChange: change('duration_seconds') }} />`,
          alternates: () => html`<${AlternateEnclosures} list=${item.alternate_enclosures} onChange=${change('alternate_enclosures')} />`,
          artwork: () => html`<${ArtworkRow} summary=${artworkSummary(item, 'track')} image=${artworkPreview(item, 'track', model)} onOpen=${() => onArtwork({ kind: 'track', key: trackKey(item) })} />`,
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
          pub_date: () => html`<${DateTimeField} id=${`${id}-date`} label="Publication date"
              value=${item.pub_date_time} sourceText=${item.pub_date} onChange=${change('pub_date_time')}
              note="Some apps sort the tracks by this date." />`,
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
      <button class="button destructive" type="button" onClick=${removeTrack}>Remove the Track</button>
    </div>`;
}

// ---- The live item page (task 033b, ADR 0054) ----

// The start time of a live item in the language of the user, for the
// summary line.
const timeText = (text) => {
  const time = new Date(text);
  return text && !Number.isNaN(time.getTime()) ? time.toLocaleString() : '';
};

// A small sheet that asks for the title and the start time of a new live
// show (ADR 0054 item 2). `onAdd` gets the title and the start as RFC 3339 text.
function AddLiveSheet({ open, onClose, onAdd, busy, error }) {
  const ref = useSheet(open);
  const [title, setTitle] = useState('');
  const [start, setStart] = useState(null);
  // Clear the sheet each time that it opens.
  useEffect(() => {
    if (!open) return;
    setTitle('');
    setStart(null);
  }, [open]);
  const ready = title.trim() && start && !busy;
  const submit = (event) => {
    event.preventDefault();
    if (ready) onAdd({ title: title.trim(), start });
  };
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="add-live-title" onClose=${onClose}>
    <header>
      <h2 id="add-live-title">Add a live show</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <form class="body" id="add-live-form" onSubmit=${submit}>
      <div class="group rows">
        <${Field} id="new-live-title" label="Title" required value=${title} onInput=${(e) => setTitle(e.target.value)} />
        <${DateTimeField} id="new-live-start" label="Start time" required value=${start} onChange=${setStart} />
        <${ErrorNotice} error=${error} />
      </div>
    </form>
    <footer>
      <button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>
      <button class="button primary" type="submit" form="add-live-form" disabled=${!ready}>＋ Add the Show</button>
    </footer>
  </dialog>`;
}

// The content links of a live item, each as one entry row (ADR 0028). A
// show needs one link before Studio writes it (ADR 0054 item 4).
function ContentLinks({ id, links, onChange }) {
  const blank = { href: null, label: '' };
  return html`${links.length === 0 && html`<p class="hint">A content link is recommended. It gives a page for a person, for example a web player.</p>`}
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
// The tab "Live Shows" of an album (ADR 0135 item 5): one row for each show,
// then the button that opens the sheet "Add a live show".
function LiveShowsPage({ feed, model, go, onAdding }) {
  const lives = model.live_items || [];
  return html`<section class="page-section"><h2>Live shows</h2>
    ${lives.length === 0
      ? html`<p class="hint">No live show yet.</p>`
      : html`<div class="group">${lives.map((live) => html`<button key=${live._cid} class="link-row" type="button"
          onClick=${() => go({ name: 'feed', id: feed.id, page: 'live', track: liveKey(live) })}>
          <span class="label">${live.item.title || 'Untitled show'}</span>
          <span class="summary">${statusLabel(live.status)}</span>
          <span class="go" aria-hidden="true">›</span>
        </button>`)}</div>`}
    <div class="actions"><button class="button action" type="button" onClick=${() => onAdding('live')}>＋ Add a Live Show</button></div>
  </section>`;
}

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
    (duration) => editLive((l) => set(l, 'item', set(l.item, 'duration_seconds', duration))),
  );
  const stream = live.item.enclosure || { url: null, length: null, media_type: null };
  // A change of the stream. `make(current)` gives the fields to change. It
  // works on the current model, so that two changes in a row, such as the
  // address and the type, do not lose one another.
  const editStream = (make) => editLive((l) => {
    const current = l.item.enclosure || { url: null, length: null, media_type: null };
    const next = { ...current, ...make(current) };
    const empty = next.url === null && next.length === null && next.media_type === null;
    return set(l, 'item', set(l.item, 'enclosure', empty ? null : next));
  });
  // Whether the user selected a type while this page is open. A type that is
  // not blank at page load counts as selected (ADR 0162 item 5). The page
  // keeps the flag only here.
  const selectedType = useRef(typeIsSelected(stream.media_type));
  const [otherPicked, setOtherPicked] = useState(false);
  const streamUrl = useRef(stream.url);
  streamUrl.current = stream.url;
  const [check, setCheck] = useState(null);
  const [checking, setChecking] = useState(false);
  const [checkError, setCheckError] = useState(null);
  const fillType = (candidate) => editStream((current) => ({ media_type: filledType(current.media_type, selectedType.current, candidate) }));
  // The core gives the type for the extension of the address (ADR 0162
  // item 5). The page has no table of extensions.
  const changeAddress = (text) => {
    streamUrl.current = text;
    setCheck(null);
    setCheckError(null);
    editStream(() => ({ url: text }));
    if (selectedType.current || !text) return;
    api('/stream/type', { method: 'POST', body: { url: text } }).then((answer) => {
      if (streamUrl.current === text) fillType(answer.media_type);
    }, () => {});
  };
  // A change of the type ends the warning of the last check (ADR 0162
  // item 9). The warning compared the type before the change.
  const endTypeWarning = () => setCheck((c) => c && { ...c, typeWarning: null });
  const pickType = (choice) => {
    selectedType.current = true;
    endTypeWarning();
    setOtherPicked(choice === OTHER_TYPE);
    if (choice !== OTHER_TYPE) editStream(() => ({ media_type: choice || null }));
  };
  const typeText = (text) => {
    selectedType.current = true;
    endTypeWarning();
    editStream(() => ({ media_type: optional(text) }));
  };
  const runCheck = async () => {
    const url = streamUrl.current;
    setChecking(true);
    setCheckError(null);
    try {
      const result = await api('/stream/check', { method: 'POST', body: { url, media_type: stream.media_type } });
      if (streamUrl.current !== url) return;
      setCheck({
        result,
        at: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
        typeWarning: checkTypeWarning(result, selectedType.current),
      });
      fillType(result.fill_type);
    } catch (e) {
      if (streamUrl.current === url) setCheckError(e);
    } finally {
      setChecking(false);
    }
  };
  const typeMenu = typeMenuState(stream.media_type, draft.streamTypes || [], otherPicked);
  const checkResult = check && checkLine(check.result, check.at);
  const value = live.live_value || { uri: null, protocol: null };
  const valueField = (name) => (text) => {
    const next = set(value, name, text);
    field('live_value')(next.uri === null && next.protocol === null ? null : next);
  };
  const links = live.content_links;
  const next = nextAction(live.status);
  const problems = (draft.liveProblems || []).filter((p) => p.guid === live.item.guid);
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
  // The name of the app of the grant.
  const [appList, setAppList] = useState([]);
  useEffect(() => { api('/apps').then(setAppList, () => {}); }, [feed.id, guid]);
  const grantApp = grant && appList.find((a) => a.id === grant.app_id);
  const nowSeconds = Date.now() / 1000;
  const grantText = (seconds) => new Date(seconds * 1000).toLocaleString();
  const stopGrant = async () => {
    if (!await askConfirm(CONFIRMS.stopGrant())) return;
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
        ${problems.map((p) => html`<p class=${`live-line ${p.level === 'refusal' ? 'field-error' : 'field-warning'}`} key=${p.field + p.text}>${p.text}</p>`)}
        <${ErrorNotice} error=${action.error} />
        ${next && html`<div class="actions"><button class="button primary" type="button"
          disabled=${action.busy || !live.item.guid} onClick=${() => changeStatus(next.status)}>${next.label}</button></div>`}
        ${next && html`<p class="hint">The button changes the draft and opens the review sheet. The file host does not change until you write.</p>`}
      </div>
    </section>

    ${grants && guid && html`<section class="page-section"><h2>Grant</h2>
      <div class="group">
        <p class="live-line">${grantSummary(grant, live, nowSeconds, grantText, grantApp ? grantApp.name : null)}</p>
        ${grant && grant.writes.length > 0 && html`<ul class="grant-writes">
          ${grant.writes.map((write, index) => html`<li key=${index}>${grantWriteText(write, grantText)}</li>`)}
        </ul>`}
        ${grantActive(grant, live, nowSeconds) && html`<div class="actions"><button class="button destructive" type="button"
          disabled=${action.busy} onClick=${stopGrant}>Stop the Grant</button></div>`}
        ${!grant && live.status === 'pending' && html`<p class="hint">Ask for a grant in the review sheet.</p>`}
      </div>
    </section>`}

    <section class="page-section"><h2>Show</h2>
      <div class="group">
        <${TextField} id=${`${id}-title`} label="Title" value=${live.item.title} onChange=${itemField('title')} />
        <${TextArea} id=${`${id}-description`} label="Description" value=${live.item.description} onChange=${itemField('description')} />
        <${DateTimeField} id=${`${id}-start`} label="Start time" required value=${live.start} onChange=${field('start')} />
        <${DateTimeField} id=${`${id}-end`} label="End time" value=${live.end} onChange=${field('end')} />
        <${PickedImageField} field="live-image" subject=${live.item} apply=${(address) => editLive((l) => set(l, 'item', set(l.item, 'image', address)))}
          id=${`${id}-image`} label="Image address" address=${`live_items/${liveIndex}/item/image`} bad=${bad}
          value=${live.item.image} onChange=${itemField('image')} glow="var(--c-track)" />
      </div>
    </section>

    ${!canRecord(live) && html`<section class="page-section"><h2>Stream</h2>
      <div class="group">
        <${TextField} id=${`${id}-stream-url`} label="Stream address" type="url" address=${`live_items/${liveIndex}/item/enclosure/url`} bad=${bad}
          value=${stream.url} onChange=${changeAddress} />
        <div class="field">
          <label for=${`${id}-stream-type-menu`}>Stream type</label>
          <div class="field-body">
            <select id=${`${id}-stream-type-menu`} value=${typeMenu.value} onChange=${(e) => pickType(e.target.value)}>
              <option value="">Choose a type</option>
              ${(draft.streamTypes || []).map((option) => html`<option key=${option.media_type} value=${option.media_type}>${option.label}</option>`)}
              <option value=${OTHER_TYPE}>Other…</option>
            </select>
            ${typeMenu.showText && html`<input id=${`${id}-stream-type`} type="text" aria-label="Other stream type"
              value=${typeMenu.text} onInput=${(e) => typeText(e.target.value)} />
              <span class="hint">For example application/x-mpegURL.</span>`}
          </div>
        </div>
        <div class="field">
          <label for=${`${id}-stream-check`}>Check Stream</label>
          <div class="field-body">
            <div class="actions"><button id=${`${id}-stream-check`} class="button" type="button"
              disabled=${checking || !stream.url} onClick=${runCheck}>${checking ? 'Checking…' : 'Check'}</button></div>
            ${checkResult && html`<span class=${checkResult.warning ? 'field-warning' : 'hint'} role="status">${checkResult.text}</span>`}
            ${check && check.typeWarning && html`<span class="field-warning" role="status">${check.typeWarning.text}</span>
              <div class="actions"><button class="button" type="button" onClick=${() => pickType(check.typeWarning.fillType)}>Use ${check.typeWarning.fillLabel}</button></div>`}
            ${checkError && html`<span class="field-warning" role="status">${checkError.message}</span>`}
          </div>
        </div>
      </div>
    </section>`}

    ${canRecord(live) && html`<section class="page-section"><h2>Recording</h2>
      <div class="group">
        ${hasRecording(live)
          ? html`<${Enclosure} id=${`${id}-recording`} path=${`live_items/${liveIndex}/item/`} bad=${bad} enclosure=${live.item.enclosure}
              onChange=${itemField('enclosure')} pick=${pickRecording}
              duration=${{ value: live.item.duration_seconds, sourceText: live.item.duration, onChange: itemField('duration_seconds') }} />
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
          onClick=${() => field('live_value')(null)}>Remove the Live Value</button></div>`}
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
function PublisherPage({ feed, feeds, model, update, bad, draft, go, onListChanged, onArtwork, onRemoved, connectors }) {
  const field = (name) => (value) => update((m) => set(m, name, value));
  const roleAction = useAction();
  const linkAction = useAction();
  // The feeds whose draft the last link or role change altered.
  const [changed, setChanged] = useState([]);
  const [chosen, setChosen] = useState('');
  // The row of the Add feeds page that is open, from the first steps (ADR 0119).
  const [addOpen, setAddOpen] = useState(null);
  // The members of this feed, from `GET /feeds/{id}/members` (ADR 0118 item 9).
  const [members, setMembers] = useState([]);
  useEffect(() => {
    let live = true;
    api(`/feeds/${feed.id}/members`).then((r) => { if (live) setMembers(r.members || []); }, () => { if (live) setMembers([]); });
    return () => { live = false; };
  }, [feed.id]);
  const items = model.remote_items;
  // The releases in the sequence of the model (ADR 0041 item 10).
  const rows = items
    .map((entry, index) => ({ index, entry, match: linkedFeed(feeds, entry) }))
    .filter((row) => row.entry.medium === 'music' || row.entry.medium === 'musicL');
  const linkedIds = new Set(rows.filter((row) => row.match).map((row) => row.match.id));
  const spaced = rows.filter((row) => hasSpace(row.entry.feed_url)).length;
  // The core corrects each address in the draft, and Review shows the change
  // (ADR 0023 item 9).
  const correctSpaces = () => linkAction.run(async () => {
    if (!await draft.flush()) return;
    const current = await api(`/feeds/${feed.id}/draft`);
    draft.show(await api(`/feeds/${feed.id}/draft/encode-link-spaces`, { method: 'POST', body: { revision: current.revision } }));
  });
  // A release is an album or a playlist (ADR 0086 item 1).
  const candidates = feeds.filter((f) => f.connector_id && ['music', 'musicL'].includes(mediumOf(f)) && !linkedIds.has(f.id));
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
          artwork: () => html`<${ArtworkRow} summary=${artworkSummary(model, 'feed')} image=${artworkPreview(model, 'feed')} onOpen=${() => onArtwork({ kind: 'feed' })} />`,
          link: () => html`<${TextField} id="pub-link" label="Website" type="url" address="link" bad=${bad} value=${model.link} onChange=${field('link')} />`,
          listing: () => html`<${ListingField} id="pub-listing" feedId=${feed.id} draft=${draft} model=${model} />`,
          explicit: () => html`<${ExplicitField} id="pub-explicit" model=${model} update=${update} />`,
        })}
      </div>
    </section>`,
    members: () => members.length > 0 && html`<section class="page-section"><h2>People</h2>
      <p class="hint">${memberSectionNote(feedWord(feed))}</p>
      <div class="group">${members.map((m) => html`<button class="link-row" type="button" key=${m.person_id}
        onClick=${() => go({ name: 'person', id: m.person_id })}>
        <${PersonArt} person=${m} />
        <span class="text"><span class="label">${m.name}</span><span class="summary">${memberRowLine(m)}</span></span><span class="go">›</span></button>`)}</div>
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
            ${readOnly && html`<span class="hint">To change the roles, connect the folder that holds this feed in File hosts.</span>`}
            ${roles.length > 0 && !feed.roles_confirmed && feed.roles_linked && html`<span class="hint">The feed and its releases give the same role.</span>`}
            ${roles.length > 0 && !feed.roles_confirmed && !feed.roles_linked && html`<span class="hint">Studio found ${roles.length === 1 ? 'this role' : 'these roles'} in the feed: ${roles.map(roleWord).join(', ')}.${readOnly ? '' : ' Select the roles to confirm them.'}</span>`}
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
        ${spaced > 0 && html`<div class="release-row">
          <span class="sub">${spaceNotice(spaced).text}</span>
          <div class="actions"><button class="button" type="button" disabled=${linkAction.busy} onClick=${correctSpaces}>${spaceNotice(spaced).action}</button></div>
        </div>`}
        ${rows.map((row, position) => html`<div class="release-row" key=${row.entry._cid}>
          <div class="order-line">
            <${PositionSelect} id=${`release-position-${position}`} index=${position} count=${rows.length}
              name=${row.entry.title || 'this release'} disabled=${linkAction.busy} onMove=${(to) => moveRow(position, to)} />
            <span class="title">${row.entry.title || row.entry.feed_url}</span>
          </div>
          <span class="sub">${row.match ? 'In the collection' : 'Not in the collection'}</span>
          ${hasSpace(row.entry.feed_url) && html`<span class="sub">${ROW_SPACE_NOTE}</span>`}
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
          facts: () => html`<${FeedFacts} feed=${feed} onRemoved=${onRemoved} />`,
        })}
      </div>
    </section>`,
  };
  // The first steps of a new publisher feed with no release (ADR 0119 item 8).
  // Each row opens a sheet of the Add feeds page.
  const addedFeed = async (f) => { await onListChanged(); go(f ? { name: 'feed', id: f.id } : { name: 'releases' }); };
  const first = nextStep(feed);
  return html`<${ChangedNotices} changed=${changed} go=${go} />
    ${first === 'album' && html`<div class="group first-step">
      <div class="link-row static first-step-row">
        <span class="step-icon" aria-hidden="true"><${Icon} name=${FIRST_STEPS.album.icon} /></span>
        <span class="text"><span class="label">${FIRST_STEPS.album.title}</span><span class="summary">${FIRST_STEPS.album.line}</span></span>
        <button class="button primary" type="button" onClick=${() => setAddOpen('album')}>${FIRST_STEPS.album.action}</button>
      </div>
    </div>`}
    ${first === 'label' && html`<div class="group">${FIRST_STEPS.label.map((step) => html`<button class="link-row" type="button" key=${step.open}
      onClick=${() => setAddOpen(step.open)}><span class="step-icon" aria-hidden="true"><${Icon} name=${step.icon} /></span>
      <span class="text"><span class="label">${step.title}</span><span class="summary">${step.line}</span></span><span class="go" aria-hidden="true">›</span></button>`)}</div>`}
    ${addOpen && html`<${AddFeedRowSheet} open=${addOpen} connectors=${connectors} feeds=${feeds}
      start=${firstStart(first, addOpen, model)} onAdded=${addedFeed} onClose=${() => setAddOpen(null)} />`}
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
        <button class="button action" type="button" disabled=${busy} onClick=${onDownload}>Try Again</button>
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

// The text of a pending operation on a file.
const PENDING_TEXT = { delete: 'Deletion pending', hold: 'Stays after the write' };

// One copy of a library file on a host (ADR 0036 items 12 and 13, ADR
// 0037). A pending operation takes over the row with "Undo". The copy
// that the feed uses shows no Reference button and no Delete button: it
// offers "Move to" each other host instead, so the user can move the use
// of the file in one click. A copy that the feed does not use offers
// "Reference", to put the feed on this copy, and "Delete".
function CopyRow({ copy, connectors, busy, onMoveTo, onReference, onDelete, onUndo }) {
  const name = connectorName(connectors, copy.connector_id);
  if (copy.pending) {
    const text = PENDING_TEXT[copy.pending.kind] || 'Copy pending';
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
function FileEntry({ feedId, file, defaultHost, connectors, busy, download, onDownload, onStop, onOperation, onUndo, onChanged, onAgain, onSave }) {
  // On a server that holds no assets, the status line names the state of
  // the file (ADR 0105). `mode.holdAssets` is true on the desktop.
  const mode = assetMode();
  const state = fileState(file, mode);
  const status = fileStatus(file, mode) ?? (file.local
    ? `${file.copies.length} ${file.copies.length === 1 ? 'copy' : 'copies'}`
    : downloadSummary(download));
  const summary = html`<span class="badge">${formatLabel(file.media_type)}</span>
    <span class="title">${file.path}</span>
    <span class=${state === null ? 'sub' : 'sub file-status'}>${file.in_use ? 'In use' : 'Not in use'} · ${status}</span>`;
  const hostName = connectorName(connectors, file.own_connector_id ?? defaultHost);
  const reference = (connectorId) => onOperation({ op: 'reference', file: file.id, connector_id: connectorId, from_url: file.url });
  const del = (connectorId, path) => onOperation({ op: 'delete', connector_id: connectorId, path });
  // A copy or a move of a remote file asks first (ADR 0038 item 12). The
  // size comes from the row.
  const askFirst = async () => file.local || !mode.holdAssets
    || await askConfirm(CONFIRMS.downloadFirst(file.size > 0 ? fileSize(file.size) : null));
  const copyTo = async (connectorId) => {
    if (await askFirst()) onOperation({ op: 'copy', file: file.id, connector_id: connectorId });
  };
  const moveTo = async (from, to) => {
    if (await askFirst()) onOperation({ op: 'move', file: file.id, from, to });
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
  const copyTargets = canCopyOrMove(file) ? connectors.filter((c) => !takenHosts.includes(c.id)) : [];
  return html`<${EntryRow} group="library-files" cid=${file.id} summary=${summary}>
    <div class="file-rows">
      ${state === 'removed' && html`<div class="notice error" role="alert"><strong>${ENTRY_TEXT.removed().strong}</strong> ${ENTRY_TEXT.removed().text}</div>
        <${AddFileAgain} file=${file} busy=${busy} onAgain=${onAgain} />`}
      ${state === 'pending' && ENTRY_TEXT.pending(dayText(file.expires_at), hostName).map((text) => html`<p class="hint" key=${text}>${text}</p>`)}
      ${state === 'on_host' && html`<p class="hint">${ENTRY_TEXT.on_host(connectorName(connectors, file.copies[0].connector_id))}</p>`}
      ${state === 'server_only' && html`<div class="notice error" role="alert"><strong>${ENTRY_TEXT.server_only(hostName).strong}</strong> ${ENTRY_TEXT.server_only(hostName).text}</div>`}
      ${!file.local && mode.holdAssets && html`<p class="hint">Not in the library. Studio uses the file on its file host.</p>`}
      ${!file.local && mode.holdAssets && html`<${DownloadControls} download=${download} busy=${busy}
        onDownload=${() => onDownload(file.id)} onStop=${() => onStop(file.id)} />`}
      ${sizeDiffers && html`<p class="hint">The feed gives ${fileSize(file.length_in_feed)}. The file is ${fileSize(file.size)}.</p>`}
      ${file.local && file.copies.length === 0 && file.pending_copies.length === 0 && state === null
        && html`<p class="hint">Studio sends this file to its file host when you write.</p>`}
      ${canDownloadToDevice(file, mode) && html`<div class="actions">
        <button class="button action" type="button" disabled=${busy} onClick=${() => onSave(file.id)}>Download to This Device</button>
      </div>`}
      ${canCopyOrMove(file) && file.copies.map((copy) => html`<${CopyRow} key=${`${copy.connector_id}-${copy.path}`} copy=${copy}
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
        <label for=${`file-host-${file.id}`}>File host of this file</label>
        <select id=${`file-host-${file.id}`} class="button" value=${hostValue} disabled=${busy}
          onChange=${(e) => setHost(e.target.value === '' ? null : Number(e.target.value))}>
          <option value="">Use the default file host of the release</option>
          ${connectors.map((c) => html`<option value=${c.id}>${c.name}</option>`)}
        </select>
      </div>
    </div>
  <//>`;
}

// "Add the File Again" for a removed file (ADR 0105 item 10). The user
// selects the same file. The upload route checks the name and the bytes.
function AddFileAgain({ file, busy, onAgain }) {
  const input = useRef(null);
  const [sent, setSent] = useState(null);
  const again = useAction();
  const choose = (e) => {
    const [picked] = e.target.files;
    e.target.value = '';
    if (!picked) return;
    setSent(0);
    again.run(() => onAgain(file, picked, setSent)).finally(() => setSent(null));
  };
  return html`<div class="actions">
    <input ref=${input} class="visually-hidden" type="file" tabindex="-1" aria-label=${`Select ${file.path} again`}
      accept=${file.media_type || undefined} onChange=${choose} />
    <button class="button action" type="button" disabled=${busy || again.busy} onClick=${() => input.current.click()}>
      ${again.busy ? `Adding… ${progressPercent(sent || 0, file.size)}%` : 'Add the File Again'}</button>
  </div>
  <${ErrorNotice} error=${again.error} />`;
}

// A host file of the release folder that is not a copy of a library file
// (ADR 0036 item 11, ADR 0037). A pending operation takes over the row
// with "Undo". A file that the feed uses keeps its "Delete" button, but
// disabled, since the user must take the file out of the feed first.
function OtherFileRow({ file, connectors, busy, onDelete, onUndo }) {
  const base = `${connectorName(connectors, file.connector_id)} · ${fileSize(file.size)}`;
  if (file.pending) {
    const text = PENDING_TEXT[file.pending.kind] || 'Copy pending';
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
      onClick=${() => act({ action: 'add', path: difference.path })}>Add to the Library</button>`;
  } else if (difference.kind === 'missing') {
    const hasSource = file && (file.copies.length > 0 || file.source_url);
    text = html`Missing: ${name}`;
    buttons = html`${hasSource && html`<button class="button action" type="button" disabled=${off}
        onClick=${() => act({ action: 'download', file: difference.file })}>Download Again</button>`}
      <button class="button destructive" type="button" disabled=${off}
        onClick=${() => act({ action: 'forget', file: difference.file })}>Forget the File</button>`;
  } else {
    const hasCopy = file && file.copies.length > 0;
    text = html`Changed: ${name} (${fileSize(difference.size)}${file ? `, was ${fileSize(file.size)}` : ''})`;
    buttons = html`<button class="button action" type="button" disabled=${off}
        onClick=${() => act({ action: 'use_new', file: difference.file })}>Use the New File</button>
      <button class="button" type="button" disabled=${off || !hasCopy}
        onClick=${() => act({ action: 'restore', file: difference.file })}>Restore</button>
      ${!hasCopy && html`<span class="sub">No copy on a file host</span>`}`;
  }
  return html`<div class="change-row">
    <span class="title">${text}</span>
    <div class="actions">${buttons}</div>
    <${ErrorNotice} error=${row.error} />
  </div>`;
}

// The sheet "Move This Release…" (ADR 0148 items 1 to 6). The core gives the
// plan of each file host. `start` is the first file host of the select.
function MoveReleaseSheet({ open, feed, connectors, start, onDone, onClose }) {
  const ref = useSheet(open);
  const [target, setTarget] = useState(null);
  const [plan, setPlan] = useState(null);
  const [deleteLater, setDeleteLater] = useState(true);
  const [wait, setWait] = useState(30);
  const { busy, error, run } = useAction();
  const other = connectors.find((c) => c.id !== feed.connector_id);
  const chosen = target ?? start ?? (other ? other.id : null);
  useEffect(() => {
    if (!open || chosen === null) { setPlan(null); return undefined; }
    let live = true;
    setPlan(null);
    run(async () => {
      const found = await api(`/feeds/${feed.id}/move/plan?connector_id=${chosen}`);
      if (live) { setPlan(found); setWait(found.default_wait_days); }
    });
    return () => { live = false; };
  }, [open, chosen, feed.id]);
  const add = () => run(async () => {
    await api(`/feeds/${feed.id}/move`, { method: 'POST', body: { connector_id: chosen, delete_later: deleteLater, wait_days: wait } });
    await onDone();
    ref.current.close();
  });
  const stays = plan ? stayRows(plan) : [];
  const hasOld = plan && plan.from_connector_ids.length > 0;
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="move-release-title" onClose=${() => { setTarget(null); onClose(); }}>
    <header>
      <h2 id="move-release-title">Move ${feed.title || 'This Release'}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body form">
      <div class="group move-group">
        <div class="field"><label for="move-release-host">To this file host</label>
          <select id="move-release-host" class="button" value=${chosen} disabled=${busy}
            onChange=${(e) => setTarget(Number(e.target.value))}>
            ${connectors.map((c) => html`<option value=${c.id}>${c.name}</option>`)}
          </select></div>
      </div>
      <${ErrorNotice} error=${error} />
      ${plan && html`<section class="page-section"><h2>The plan</h2>
        <div class="group move-group">
          ${planRows(plan, connectors).map((row) => html`<div class="link-row static" key=${row.label}>
            <span class="label">${row.label}</span><span class="summary">${row.value}</span></div>`)}
          ${stays.map((row) => html`<div class="link-row static" key=${row.name}>
            <span class="label mono key-text">${row.name}</span><span class="summary">${row.text}</span></div>`)}
        </div>
        ${plan.nothing_to_move && html`<p class="hint">Each file of this release is already on this file host.</p>`}
      </section>`}
      ${hasOld && html`<div class="group move-group">
        <div class="move-check"><label class="check"><input type="checkbox" checked=${deleteLater}
            onChange=${(e) => setDeleteLater(e.target.checked)} /> ${deleteLabel(plan, connectors)}</label>
          <p class="hint">${deleteLater ? DELETE_NOTE_ON : DELETE_NOTE_OFF}</p></div>
        <div class="field"><label for="move-release-wait">Keep the old files for</label>
          <select id="move-release-wait" class="button" value=${wait} disabled=${!deleteLater}
            onChange=${(e) => setWait(Number(e.target.value))}>
            ${plan.wait_choices.map((days) => html`<option value=${days}>${waitLabel(days)}</option>`)}
          </select></div>
      </div>`}
    </div>
    <footer>
      <button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>
      <button class="button primary" type="button" disabled=${busy || !plan || plan.nothing_to_move} onClick=${add}>Add to the Draft</button>
    </footer>
  </dialog>`;
}

// The sheet "Move All Releases…" of a file host (ADR 0148 items 16 to 19). The
// result is a change set, so the shell opens its review.
export function MoveAllSheet({ open, connector, connectors, onMoved, onClose }) {
  const ref = useSheet(open);
  const others = connectors.filter((c) => c.id !== connector.id);
  const [target, setTarget] = useState(null);
  const [deleteLater, setDeleteLater] = useState(true);
  const [wait, setWait] = useState(30);
  const { busy, error, run } = useAction();
  const chosen = target ?? (others[0] ? others[0].id : null);
  const add = () => run(async () => {
    const made = await api('/change-sets/move', {
      method: 'POST',
      body: { from: connector.id, to: chosen, delete_later: deleteLater, wait_days: wait },
    });
    ref.current.close();
    await onMoved(made.id);
  });
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby=${`move-all-title-${connector.id}`} onClose=${onClose}>
    <header>
      <h2 id=${`move-all-title-${connector.id}`}>Move All Releases from ${connector.name}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body form">
      <div class="group move-group">
        <div class="field"><label for=${`move-all-host-${connector.id}`}>To this file host</label>
          <select id=${`move-all-host-${connector.id}`} class="button" value=${chosen} disabled=${busy}
            onChange=${(e) => setTarget(Number(e.target.value))}>
            ${others.map((c) => html`<option value=${c.id}>${c.name}</option>`)}
          </select></div>
      </div>
      <div class="group move-group">
        <div class="move-check"><label class="check"><input type="checkbox" checked=${deleteLater}
            onChange=${(e) => setDeleteLater(e.target.checked)} /> ${deleteAllLabel(connector.name)}</label>
          <p class="hint">${deleteLater ? DELETE_NOTE_ON : DELETE_NOTE_OFF}</p></div>
        <div class="field"><label for=${`move-all-wait-${connector.id}`}>Keep the old files for</label>
          <select id=${`move-all-wait-${connector.id}`} class="button" value=${wait} disabled=${!deleteLater}
            onChange=${(e) => setWait(Number(e.target.value))}>
            ${[7, 30, 90].map((days) => html`<option value=${days}>${waitLabel(days)}</option>`)}
          </select></div>
      </div>
      <p class="hint">The next page shows the changes of each release. Nothing changes on a file host until you write.</p>
      <${ErrorNotice} error=${error} />
    </div>
    <footer>
      <button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>
      <button class="button primary" type="button" disabled=${busy || chosen === null} onClick=${add}>Review the Changes</button>
    </footer>
  </dialog>`;
}

// One move that waits, or that is ready (ADR 0148 items 8, 10 and 11). The core
// gives its state.
function WaitingMoveEntry({ move, connectors, busy, onDelete, onKeep, onDeleteNow }) {
  if (move.ready) {
    const notice = readyNotice(move, connectors);
    return html`<div class="notice download-notice" role="status">
      <span><strong>${notice.strong}</strong> ${notice.text}</span>
      <div class="actions">
        <button class="button action" type="button" disabled=${busy} onClick=${() => onDelete(move)}>Review the Deletion</button>
        <button class="button" type="button" disabled=${busy} onClick=${() => onKeep(move)}>Keep the Files</button>
      </div>
    </div>`;
  }
  const view = waitingView(move, connectors);
  return html`<div class="group">
      <div class="link-row static"><span class="label">${view.title}</span><span class="summary">${view.summary}</span></div>
      ${view.rows.map((row) => html`<div class="link-row static" key=${row.label}>
        <span class="label">${row.label}</span><span class="summary">${row.value}</span></div>`)}
    </div>
    <div class="actions">
      <button class="button destructive" type="button" disabled=${busy} onClick=${() => onDeleteNow(move)}>Delete Now…</button>
      <button class="button" type="button" disabled=${busy} onClick=${() => onKeep(move)}>Keep the Files</button>
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
  // The moves that wait (ADR 0148 item 8), and the sheet of a release move.
  const [moves, setMoves] = useState([]);
  const [moving, setMoving] = useState(null);
  const loadMoves = () => api(`/feeds/${feed.id}/moves`).then((r) => setMoves(r.moves), () => setMoves([]));
  useEffect(() => { loadMoves(); }, [feed.id, reloadKey]);
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
    // A server that holds no assets has no downloads (ADR 0105 item 14).
    if (!assetMode().holdAssets) return;
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

  // "Add the File Again": the upload routes take the path of the removed
  // file in `replace` (ADR 0105 item 10). The core checks the name and the
  // bytes. The shell loads the draft again, since the row has its bytes now.
  const addAgain = async (file, picked, onProgress) => {
    await sendInParts(feed.id, picked, { replace: file.path, onProgress });
    await load();
    onOpsChanged();
  };

  // "Download to This Device": get a ticket for the file, and save it
  // with `saveTicket` (ADR 0105 item 15).
  const saveToDevice = (fileId) => run(async () => {
    const { ticket } = await api('/media/tickets', { method: 'POST', body: { file: fileId } });
    saveTicket(ticket);
  });

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

  // "Review the Deletion" puts a deletion of each old copy in the draft, and
  // Review shows them (ADR 0148 items 10 and 12).
  const deleteOldFiles = (move) => run(async () => {
    await api(`/feeds/${feed.id}/moves/delete`, { method: 'POST', body: { records: move.records } });
    onOpsChanged();
    await load();
    window.dispatchEvent(new Event(OPEN_REVIEW));
  });
  const deleteOldFilesNow = async (move) => {
    const host = connectorName(connectors, move.connector_id);
    if (await askConfirm(CONFIRMS.deleteOldFilesNow(move.records.length, host))) deleteOldFiles(move);
  };
  const keepOldFiles = (move) => run(async () => {
    await api(`/feeds/${feed.id}/moves/keep`, { method: 'POST', body: { records: move.records } });
    await loadMoves();
  });
  const afterReleaseMove = async () => {
    onOpsChanged();
    await load();
    await loadDownloads();
  };

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
  const mode = assetMode();
  const missing = mode.holdAssets ? 0 : missingCount(data.files);
  const feedHost = data.feed_move ? data.feed_move.connector_id : feed.connector_id;
  const tag = newFeedAddress(feed);

  const partMove = data.feed_move && data.files_stay > 0
    ? partMoveNotice(data.files_stay, data.feed_move.connector_id, feed.connector_id, connectors) : null;

  return html`${!mode.holdAssets && html`<div class="notice" role="status"><strong>${SERVER_NOTICE.strong}</strong> ${SERVER_NOTICE.text}</div>`}
    ${partMove && html`<div class="notice download-notice" role="status">
      <span><strong>${partMove.strong}</strong> ${partMove.text}</span>
      <div class="actions"><button class="button action" type="button" disabled=${busy}
        onClick=${() => setMoving({ start: data.feed_move.connector_id })}>Move the Files Too</button></div>
    </div>`}
    ${missing > 0 && html`<div class="notice error" role="alert"><strong>${missingNotice(missing).strong}</strong> ${missingNotice(missing).text}</div>`}
    ${differences.length > 0 && html`<section class="page-section"><h2>Changes in the library folder</h2>
      <div class="group">
        ${differences.map((difference) => html`<${ChangeRow}
          key=${`${difference.kind}-${difference.file ?? difference.path}`} difference=${difference}
          file=${data.files.find((f) => f.id === difference.file)} busy=${busy} onAct=${changeLibrary} />`)}
      </div>
    </section>`}

    <section class="page-section"><h2>Files</h2>
      <div class="actions">
        <button class="button action" type="button" disabled=${scan.busy} onClick=${runScan}>
          ${scan.busy ? 'Scanning…' : 'Scan the File Hosts'}</button>
        ${mode.holdAssets && html`<button class="button action" type="button" disabled=${scan.busy} onClick=${runLibraryScan}>
          Scan the Library Folder</button>`}
        ${connectors.length > 1 && html`<button class="button action" type="button" disabled=${busy}
          onClick=${() => setMoving({ start: null })}>Move This Release…</button>`}
      </div>
      <${ErrorNotice} error=${scan.error} />
      <${ErrorNotice} error=${opError} />
    </section>

    <section class="page-section"><h2>File hosts</h2>
      <div class="group">
        <div class="field">
          <label for="files-default-host">Default file host of the release files</label>
          <select id="files-default-host" class="button" value=${defaultHost} disabled=${busy}
            onChange=${(e) => setDefaultHost(Number(e.target.value))}>
            ${connectors.map((c) => html`<option value=${c.id}>${c.name}</option>`)}
          </select>
        </div>
        <div class="field">
          <label for="files-feed-host">File host of the feed XML</label>
          <select id="files-feed-host" class="button" value=${feedHost} disabled=${busy}
            onChange=${(e) => setFeedHost(Number(e.target.value))}>
            ${connectors.filter((c) => c.kind !== 'blossom').map((c) => html`<option value=${c.id}>${c.name}</option>`)}
          </select>
          ${data.feed_move && html`<p class="hint">The feed moves to ${connectorName(connectors, data.feed_move.connector_id)} when you write${moveInfo && moveInfo.url ? html`, at <span class="mono">${moveInfo.url}</span>` : ''}.
            The address from before${moveInfo && moveInfo.from_url ? html`, <span class="mono">${moveInfo.from_url}</span>,` : ''} gets ${moveMethod === 'redirect' ? 'a redirect' : 'a pointer file'}.</p>`}
          ${tag && html`<p class="hint">The feed file names the new feed address <span class="mono">${tag.url}</span>.</p>`}
        </div>
      </div>
    </section>

    ${moves.length > 0 && html`<section class="page-section"><h2>Moves that wait</h2>
      ${moves.map((move) => html`<${WaitingMoveEntry} key=${move.records[0]} move=${move} connectors=${connectors} busy=${busy}
        onDelete=${deleteOldFiles} onDeleteNow=${deleteOldFilesNow} onKeep=${keepOldFiles} />`)}
    </section>`}

    <${MoveReleaseSheet} open=${moving !== null} feed=${feed} connectors=${connectors} start=${moving && moving.start}
      onDone=${afterReleaseMove} onClose=${() => setMoving(null)} />

    <section class="page-section"><h2>Library files</h2>
      ${mode.holdAssets && data.remote_count > 0 && html`<div class="notice download-notice">
        <span>${activeCount > 0
          ? `Downloading ${activeCount} ${activeCount === 1 ? 'file' : 'files'}…`
          : `${data.remote_count} ${data.remote_count === 1 ? 'file is' : 'files are'} not in the library (${fileSize(data.remote_bytes)}).`}</span>
        <div class="actions">
          ${activeCount > 0
            ? html`<button class="button" type="button" disabled=${busy} onClick=${stopAll}>Stop All</button>`
            : html`<button class="button action" type="button" disabled=${busy} onClick=${() => startDownloads({ all: true })}>Download All</button>`}
        </div>
      </div>`}
      ${data.files.length === 0 && html`<p class="hint">No files yet.</p>`}
      ${data.files.length > 0 && html`<div class="group">
        ${byName(data.files, (file) => file.path).map((file) => html`<${FileEntry} key=${file.id} feedId=${feed.id} file=${file} defaultHost=${defaultHost}
          connectors=${connectors} busy=${busy} onOperation=${addOperation} onUndo=${undoOperation}
          download=${downloads.find((d) => d.file_id === file.id)}
          onDownload=${(fileId) => startDownloads({ files: [fileId] })} onStop=${stopDownload}
          onAgain=${addAgain} onSave=${saveToDevice}
          onChanged=${(path, body) => setFileHost(path, body)} />`)}
      </div>`}
    </section>

    ${data.other_host_files.length > 0 && html`<section class="page-section"><h2>Other files on the file hosts</h2>
      <div class="group">
        ${byName(data.other_host_files, (extra) => extra.path).map((extra) => html`<${OtherFileRow}
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
      draft=${draft} go=${go} onListChanged=${onListChanged} onArtwork=${onArtwork} onRemoved=${onRemoved} connectors=${connectors} />`;
  } else if (playlist && page === 'tracks') {
    body = html`<${PlaylistTracksPage} feed=${feed} model=${model} update=${update} />`;
  } else if (playlist && page !== 'recipients' && page !== 'files') {
    body = html`<${PlaylistPage} feed=${feed} feeds=${feeds} draft=${draft} model=${model} update=${update} bad=${draft.bad} go=${go} onRemoved=${onRemoved} onListChanged=${onListChanged} onArtwork=${onArtwork} />`;
  } else if (page === 'track' && trackIndex !== -1) {
    // A key for each entry, so that one track or show does not keep the
    // state of the one before it, for example an open "Properties".
    body = html`<${TrackPage} key=${model.items[trackIndex]._cid} feed=${feed} model=${model} update=${update} bad=${draft.bad} trackIndex=${trackIndex} go=${go} onArtwork=${onArtwork} />`;
  } else if (page === 'live' && liveIndex !== -1 && !publisher && !playlist) {
    body = html`<${LivePage} key=${model.live_items[liveIndex]._cid} feed=${feed} model=${model} update=${update} bad=${draft.bad} liveIndex=${liveIndex} draft=${draft} />`;
  } else if (page === 'lives' && !publisher && !playlist) {
    body = html`<${LiveShowsPage} feed=${feed} model=${model} go=${go} onAdding=${onAdding} />`;
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
    ${draft.grantBlocked && (() => {
      const n = draftBlockedNotice(draft.grantBlocked, draft.grantBlocked.start ? timeText(draft.grantBlocked.start) : null);
      return html`<div class="notice" role="status"><strong>${n.title}</strong> ${n.text}</div>`;
    })()}
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

// The value editor (task 012): the payment recipients of the album and of
// each track. The editor does not change a value time split.
import { html, useState, useEffect, useRef } from './vendor/htm-preact-standalone.module.js';
import { TextField, set, setAt, withCids, startOpen, openOnce, useSheet } from './editor.js';
import { CONFIRMS } from './confirms.js';
import { lookupHint, WALLET_TYPES, CUSTOM_KEY_HINT, percentages, isFee, shares, shareParts, splitOrder, recipientFields, recipientFromFields } from './recipients.js';
import { recipientPickRows, recipientChoices, recipientIdentity, personKeys, memberLine, memberText, addRecipientsLabel, trackHeading } from './people.js';
import { PickerLayout, PickFillRow, FindOneSheet, PickFold, SearchResults, TakenTag } from './pickers.js';
import { PersonArt, useLookup } from './peoplepages.js';
import { api, askConfirm, Field, ErrorNotice, useAction } from './app.js';

const percentText = (p) => (p === null ? '—' : `${p.toFixed(p < 1 && p > 0 ? 2 : 1)} %`);

// ---- Share bars ----
// One hue: the accent. A meter shows one recipient against 100 %, on a pale
// track of the same hue. A stack shows all recipients as parts of the whole,
// with a 2 px gap between parts. A fee part has stripes, and the "Other" part
// is gray. A legend after the stack names the parts, and each part shows its
// label on hover.

function Meter({ percent }) {
  const width = percent === null ? 0 : Math.min(100, Math.max(0, percent));
  return html`<span class="share-meter" aria-hidden="true"><span style=${{ width: `${width}%` }}></span></span>`;
}

// A short name for the legend: a Lightning address shows its user part.
const shortName = (name) => (name.includes('@') ? name.split('@')[0] : name);

// The tooltip of one part has the name, the percentage and the raw share.
// A fee part also has the word "fee".
const partDetail = (p) => `${p.name} ${percentText(p.percent)} (share ${p.share}${p.fee ? ', fee' : ''})`;

// The stacked bar and its legend. A segment takes the focus from the
// keyboard, unless the bar is inside a button (`focusable` false).
export function ShareStack({ recipients, focusable = true }) {
  const { all, shown, legend } = shareParts(recipients);
  if (shown.length === 0) return null;
  const label = all.map((p) => `${p.name} ${percentText(p.percent)}${p.fee ? ' (fee)' : ''}`).join(', ');
  const tip = (p) => (p.other ? p.members.map(partDetail).join(', ') : partDetail(p));
  return html`<span class="share-preview">
    <span class="share-stack" role="group" aria-label=${`Shares: ${label}`}>
      ${shown.map((p) => html`<span class=${p.other ? 'other' : p.fee ? 'fee' : ''} style=${{ flexGrow: p.percent }}
        data-tip=${tip(p)} tabindex=${focusable ? '0' : null} role="img" aria-label=${tip(p)}></span>`)}
    </span>
    <span class="share-legend" aria-hidden="true">
      ${legend.map((p, i) => html`${i > 0 ? ' · ' : ''}<span class="part">${p.more ? `${p.more} more` : shortName(p.name)} ${percentText(p.percent)}</span>`)}
    </span>
  </span>`;
}

// A copy of entries with no key and no client ID, so that the core adds them
// as new entries (ADR 0022 item 3).
function fresh(value) {
  return JSON.parse(JSON.stringify(value, (name, v) => (name === 'key' || name === '_cid' ? undefined : v)));
}

// ---- Recipients ----
// The editor shows the recipients of one payment block for the album and for
// each track. It does not show the payment type or the suggested amount.
// Studio keeps the values of the fields that the editor does not show.

const TYPES = [
  { value: 'node', label: WALLET_TYPES[0][1], hint: 'The public key of the Lightning node, 66 hexadecimal characters.' },
  { value: 'lnaddress', label: WALLET_TYPES[1][1], hint: 'An LNURL address, for example name@example.com.' },
];

// One load of `GET /recipients` for the page. Each `Recipients` shares it. The
// map goes from the lowercase address to its keysend lookup. A failed load
// gives an empty map, and the editor makes no lookup (ADR 0095).
let recipientsPromise = null;
function loadRecipients() {
  if (!recipientsPromise) recipientsPromise = api('/recipients').catch(() => []);
  return recipientsPromise;
}
function loadLookups() {
  return loadRecipients().then((list) => {
    const map = new Map();
    for (const entry of list || []) {
      for (const m of entry.addresses || []) {
        if (m.address) map.set(m.address.toLowerCase(), m.lookup || null);
      }
    }
    return map;
  });
}

function Choice({ id, label, value, options, onChange }) {
  const known = options.some((o) => o.value === value);
  return html`<div class="field">
    <label for=${id}>${label}</label>
    <div class="field-body">
      <select id=${id} value=${value || ''} onChange=${(e) => onChange(e.target.value || null)}>
        ${!value && html`<option value="">Not set</option>`}
        ${value && !known && html`<option value=${value}>${value}</option>`}
        ${options.map((o) => html`<option value=${o.value}>${o.label}</option>`)}
      </select>
    </div>
  </div>`;
}

// Recipients returns its rows with no wrapper div, so that each recipient
// and the add row sit as direct rows of the group, each with the group's
// own inset separator (review of task 014c, defect 3). The rows form two
// groups: "Recipients", then "Fees" (ADR 0039 item 12). A recipient with
// fee "true" is in "Fees". The screen sorts each group by split (ADR 0144).
function Recipients({ id, recipients, defaultType, onChange }) {
  const percent = percentages(recipients);
  // The sheet of a new recipient or of one address: null when closed, else
  // `{ cid }` with the `_cid` of the row to change, or `{}` for a new row.
  const [sheet, setSheet] = useState(null);
  const [copying, setCopying] = useState(false);
  const blank = { name: null, recipient_type: defaultType, address: null, split: '', custom_key: null, custom_value: null, fee: null };
  const changing = sheet && recipients.find((r) => r._cid === sheet.cid);
  const save = (values) => {
    if (changing) {
      onChange(recipients.map((r) => (r._cid === changing._cid ? { ...r, ...values } : r)));
    } else {
      const added = withCids([{ ...blank, ...values }])[0];
      startOpen.add(added._cid);
      onChange([...recipients, added]);
    }
  };
  const addCopied = (added) => {
    onChange([...recipients, ...added]);
    setCopying(false);
  };
  // The rows stay where they are while the focus is in a row. They sort when
  // the focus leaves it, and when a row comes or goes (ADR 0144 item 4). The
  // delay lets a click finish before the row under the pointer moves.
  const [sortAt, setSortAt] = useState(0);
  const order = useRef({ at: -1, cids: [] });
  const same = order.current.cids.length === recipients.length && recipients.every((r) => order.current.cids.includes(r._cid));
  if (order.current.at !== sortAt || !same) {
    order.current = { at: sortAt, cids: splitOrder(recipients).map((i) => recipients[i]._cid) };
  }
  const row = (r, i) => {
      const change = (name) => (value) => onChange(setAt(recipients, i, set(r, name, value)));
      const rid = `${id}-r-${r._cid}`;
      const line = r.address ? memberLine(r, true) : { label: 'No address', address: null, account: null };
      return html`<details class="recipient" name=${`${id}-recipients`} key=${r._cid} ref=${openOnce(r._cid)}
        onfocusout=${(e) => { if (!e.currentTarget.contains(e.relatedTarget)) setTimeout(() => setSortAt((n) => n + 1), 200); }}>
        <summary>
          <span class="title">${r.name || r.address || 'New recipient'}</span>
          <${Meter} percent=${percent[i]} />
          <span class="share">${percentText(percent[i])}${isFee(r) ? ' fee' : ''}</span>
          <span class="mono address">${r.address || 'No address'}</span>
        </summary>
        <div class="form">
        <${TextField} id=${`${rid}-split`} label=${isFee(r) ? 'Split (percent)' : 'Split (shares)'} inputmode="numeric" value=${r.split} onChange=${change('split')} />
        <label class="check"><input type="checkbox" checked=${isFee(r)}
          onChange=${(e) => change('fee')(e.target.checked ? 'true' : null)} />
          Fee, paid first</label>
        <div class="link-row static address-row"><span class="text"><span class="label">${line.label}</span>
            ${line.address && html`<span class="summary mono address-text">${line.address}</span>`}
            ${line.account && html`<span class="summary">Account ${line.account}</span>`}</span>
          <${SaveRecipientButton} recipient=${r} />
          <button class="button" type="button" aria-label=${`Change the address of ${r.name || r.address || 'this recipient'}`} onClick=${() => setSheet({ cid: r._cid })}>Change…</button>
        </div>
        <div class="actions"><button class="button destructive" type="button"
          aria-label=${`Remove ${r.name || r.address || 'this recipient'}`} onClick=${() => onChange(recipients.filter((_, k) => k !== i))}>Remove</button></div>
        </div>
      </details>`;
  };
  // A group holds the rows of one kind. Each row keeps its index in the list.
  const ofGroup = (fee) => order.current.cids.map((cid) => {
    const i = recipients.findIndex((r) => r._cid === cid);
    return i >= 0 && isFee(recipients[i]) === fee ? row(recipients[i], i) : null;
  });
  return html`<h3 class="value-group">Recipients</h3>
    ${ofGroup(false)}
    ${!recipients.some((r) => !isFee(r)) && html`<p class="hint">No recipients.</p>`}
    <button class="link-row add" type="button" onClick=${() => setSheet({})}><span class="num">＋</span><span class="title">Add a recipient</span></button>
    <h3 class="value-group">Fees</h3>
    ${ofGroup(true)}
    ${!recipients.some(isFee) && html`<p class="hint">No fees.</p>`}
    <${RecipientSheet} open=${sheet !== null} recipient=${changing || null} defaultType=${defaultType} recipients=${recipients}
      onSave=${save} onCopy=${() => { setSheet(null); setCopying(true); }} onClose=${() => setSheet(null)} />
    <${PickerSheet} open=${copying} recipients=${recipients} onAdd=${addCopied} onClose=${() => setCopying(false)} />`;
}

// The sheet "Add a Recipient", or "Change the Address" of a row (ADR 0144). It
// has the layout of each sheet that adds a value (ADR 0145).
function RecipientSheet({ open, recipient, defaultType, recipients, onSave, onCopy, onClose }) {
  const ref = useSheet(open);
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="recipient-title" onClose=${onClose}>
    ${open && html`<${RecipientForm} key=${recipient ? recipient._cid : 'new'} recipient=${recipient} defaultType=${defaultType}
      recipients=${recipients} onSave=${(values) => { onSave(values); ref.current.close(); }} onCopy=${onCopy} onCancel=${() => ref.current.close()} />`}
  </dialog>`;
}

function RecipientForm({ recipient, defaultType, recipients, onSave, onCopy, onCancel }) {
  const [fields, setFields] = useState(() => recipientFields(recipient || { recipient_type: defaultType }));
  const [list, setList] = useState(null);
  const [lookups, setLookups] = useState(null);
  const [finding, setFinding] = useState(false);
  const loading = useAction();
  useEffect(() => {
    loadLookups().then(setLookups);
    loading.run(async () => {
      const [people, known] = await Promise.all([api('/people'), api('/recipients')]);
      setList(recipientPickRows(people, known));
    });
  }, []);
  const set = (name) => (value) => setFields((current) => ({ ...current, [name]: value ?? '' }));
  const type = TYPES.find((t) => t.value === fields.recipient_type);
  const address = fields.address.trim();
  const others = new Set(recipients.filter((r) => r !== recipient).map(identityOf));
  const values = recipientFromFields(fields);
  const taken = address !== '' && others.has(identityOf(values));
  const pickRow = (r) => {
    const rowKey = recipientIdentity(r.recipient);
    const used = others.has(rowKey);
    return html`<${PickFillRow} key=${rowKey} title=${r.name} line=${r.line} taken=${used}
      lead=${r.person && html`<${PersonArt} person=${r.person} />`}
      selected=${rowKey === recipientIdentity(values)}
      after=${!r.person && html`<${SaveRecipientButton} recipient=${r.recipient} />`}
      onSelect=${() => setFields((current) => ({ ...current, ...recipientFields(r.recipient) }))} />`;
  };
  return html`<header>
      <h2 id="recipient-title">${recipient ? 'Change the Address' : 'Add a Recipient'}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${onCancel}>✕</button>
    </header>
    <${PickerLayout} id="recipient-form" onSubmit=${(e) => { e.preventDefault(); if (address !== '' && !taken) onSave(values); }}
      fields=${html`<div class="group rows">
        <${Choice} id="recipient-type" label="Type" value=${fields.recipient_type} options=${TYPES} onChange=${set('recipient_type')} />
        <${TextField} id="recipient-address" label="Address" required
          hint=${fields.recipient_type === 'lnaddress' && address && lookups && lookups.get(address.toLowerCase()) === 'none'
            ? lookupHint(address) : type ? type.hint : ''}
          value=${fields.address} onChange=${set('address')} />
        <${TextField} id="recipient-name" label="Name" required value=${fields.name} onChange=${set('name')} />
        ${fields.recipient_type !== 'lnaddress' && html`
          <${TextField} id="recipient-key" label="Custom key" required hint=${CUSTOM_KEY_HINT} value=${fields.custom_key} onChange=${set('custom_key')} />
          <${TextField} id="recipient-account" label="Account" required hint="The account on that node." value=${fields.custom_value} onChange=${set('custom_value')} />`}
      </div>${taken && html`<p class="field-error">This address is already in this list.</p>`}`}
      notice=${html`<${ErrorNotice} error=${loading.error} />
        ${list === null && !loading.error && html`<p class="hint" role="status">Loading your people…</p>`}`}
      musicindex=${recipient
        ? { title: 'Find on musicindex', summary: 'Fill the form from a feed.', onOpen: () => setFinding(true) }
        : { title: 'Copy from a Feed', summary: 'Select many recipients of a different feed.', onOpen: onCopy }}
      lists=${list === null ? [] : [{ title: 'Your people', rows: list.people.map(pickRow) }, { title: 'Payees with no person', rows: list.payees.map(pickRow) }]} />
    <footer><button class="button primary" type="submit" form="recipient-form" disabled=${address === '' || taken}>${recipient ? 'Use This Address' : 'Add the Recipient'}</button></footer>
    <${FindOneSheet} open=${finding} start=${fields.name} choices=${recipientChoices} empty="This feed has no payment address."
      onPick=${(route) => setFields((current) => ({ ...current, ...recipientFields(route) }))} onClose=${() => setFinding(false)} />`;
}

// ---- Save a recipient (task 013c, ADR 0027 item 5) ----

// The button of an open recipient that saves it to the user's list. It
// shows "Saved" for 2 seconds after the save.
function SaveRecipientButton({ recipient }) {
  const { busy, error, run } = useAction();
  const [saved, setSaved] = useState(false);
  // A recipient of a person needs no add. `null` while the list loads.
  const [inPeople, setInPeople] = useState(null);
  useEffect(() => {
    let live = true;
    loadRecipients().then((list) => { if (live) setInPeople(personKeys(list).has(recipientIdentity(recipient))); });
    return () => { live = false; };
  }, [recipient.address, recipient.custom_key, recipient.custom_value]);
  const save = () => run(async () => {
    await api('/recipients/saved', {
      method: 'POST',
      body: {
        name: recipient.name,
        recipient_type: recipient.recipient_type,
        address: recipient.address,
        custom_key: recipient.custom_key,
        custom_value: recipient.custom_value,
      },
    });
    setSaved(true);
    // The next load of the list has the new person.
    recipientsPromise = null;
    setTimeout(() => { setSaved(false); setInPeople(true); }, 2000);
  });
  // Only a recipient with an address that no person holds can be added (ADR 0144 item 3).
  if (!recipient.address || (inPeople !== false && !saved)) return null;
  return html`<${ErrorNotice} error=${error} />
    <div class="actions">
      <button class="button action" type="button" disabled=${busy} onClick=${save}>
        ${saved ? 'Added' : busy ? 'Adding…' : 'Add to People'}</button>
    </div>`;
}

// ---- The recipient picker (task 013c, ADR 0027) ----
// The picker adds recipients from the user's list and from musicindex. It
// only adds a recipient. It does not remove or replace a recipient of the
// editor (ADR 0027 invariant).

// The identity of a recipient: its address, its custom key and its custom
// value (ADR 0027 item 3). Two rows with the same identity are the same
// recipient, even with a different name.
const identityOf = recipientIdentity;

// One row of the sheet "Copy from a Feed": a check box, the name, the line of
// the address and the share in the source. A row that the list has already
// has no check box (ADR 0145 item 10, ADR 0146 item 3).
function PickRow({ name, line, share, note, checked, taken, onToggle }) {
  return html`<div class=${taken ? 'link-row pick-row pick-taken' : 'link-row pick-row'}>
    ${taken ? html`<span class="pick-box"></span>` : html`<label class="pick-box"><input type="checkbox" checked=${checked}
      aria-label=${name} onChange=${onToggle} /></label>`}
    <span class="text"><span class="pick-name"><span class="title">${name}</span>${taken && html`<${TakenTag} />`}</span><span class="sub">${line}</span>
      ${note && html`<span class="sub">${note}</span>`}</span>
    ${share && html`<span class="summary">${share}</span>`}
  </div>`;
}

// The rows of the recipients of one feed or track, with the share of each in
// that source (ADR 0145 item 11).
function copyRows(recipients, taken, selected, toggle, from) {
  const percent = percentages(recipients);
  return recipients.map((r, i) => {
    const key = identityOf(r);
    return html`<${PickRow} key=${key} name=${r.name || r.address || 'Recipient'} line=${memberText(r)}
      share=${`${percentText(percent[i])}${isFee(r) ? ' fee' : ''}`}
      checked=${selected.has(key)} taken=${taken.has(key)} onToggle=${() => toggle(r, from)} />`;
  });
}

const COPY_NOTE = 'A percent is the share in this source. A new recipient gets no split, and you give it in the editor.';

// The sheet that copies many recipients of a feed on musicindex (ADR 0027
// items 5 to 7, ADR 0030, ADR 0145 items 9 to 13).
function PickerSheet({ open, recipients, onAdd, onClose }) {
  const ref = useSheet(open);
  const taken = new Set(recipients.map(identityOf));

  // ---- From musicindex ----
  const [query, setQuery] = useState('');
  const search = useLookup(query, '/musicindex/search');
  const [view, setView] = useState('hits'); // 'hits', 'feed' or 'track'
  const [feedData, setFeedData] = useState(null);
  const [trackData, setTrackData] = useState(null);
  const [indexSelected, setIndexSelected] = useState(new Map());
  const indexLoad = useAction();
  const openFeed = (guid) => indexLoad.run(async () => {
    const data = await api(`/musicindex/feeds/${encodeURIComponent(guid)}/recipients`);
    setFeedData({ guid, ...data });
    setView('feed');
    setMoved((n) => n + 1);
  });
  const openTrack = (trackGuid) => indexLoad.run(async () => {
    const data = await api(
      `/musicindex/feeds/${encodeURIComponent(feedData.guid)}/tracks/${encodeURIComponent(trackGuid)}/recipients`,
    );
    setTrackData({ trackGuid, ...data });
    setView('track');
    setMoved((n) => n + 1);
  });
  const backToHits = () => { setFeedData(null); setView('hits'); setMoved((n) => n + 1); };
  const backToFeed = () => { setTrackData(null); setView('feed'); setMoved((n) => n + 1); };
  // After each move between the hits, a feed and a track, the sheet scrolls to
  // its top. The user then sees the page that opened.
  const [moved, setMoved] = useState(0);
  const bodyRef = useRef(null);
  useEffect(() => {
    if (moved > 0 && bodyRef.current) bodyRef.current.scrollTop = 0;
  }, [moved]);
  // The list of the selections opens from a handle in the footer, so that
  // it does not move the parts of the sheet.
  const [showSelected, setShowSelected] = useState(false);
  // Each musicindex selection keeps the name of its feed or track, so that
  // the Selected part can show it after the user goes to a different page.
  const toggleIndex = (r, from) => setIndexSelected((selected) => {
    const key = identityOf(r);
    const next = new Map(selected);
    if (next.has(key)) next.delete(key); else next.set(key, { recipient: r, from });
    return next;
  });
  const feedName = feedData ? feedData.title || 'Feed' : '';
  const trackName = trackData ? trackData.title || 'Track' : '';
  // A single has one track with the name of its feed. Show that name one time.
  const trackPlace = trackName === feedName ? trackName : `${trackName} · ${feedName}`;

  // The sheet starts fresh each time that it opens.
  useEffect(() => {
    if (!open) return;
    setShowSelected(false);
    setMoved(0);
    setQuery('');
    setView('hits');
    setFeedData(null);
    setTrackData(null);
    setIndexSelected(new Map());
  }, [open]);

  const total = indexSelected.size;
  const selectedRows = [
    ...[...indexSelected].map(([key, entry]) => ({ key, recipient: entry.recipient, from: entry.from,
      clear: () => toggleIndex(entry.recipient) })),
  ];
  const add = () => {
    const combined = new Map();
    // An imported recipient keeps its fee mark but gets no split. The user
    // gives each new recipient its split (ADR 0030).
    for (const [key, { recipient: { provider, ...recipient } }] of indexSelected) combined.set(key, { ...recipient, split: '' });
    onAdd(withCids(fresh([...combined.values()])));
  };

  return html`<dialog ref=${ref} class="sheet" aria-labelledby="picker-title" onClose=${onClose}>
    <header>
      <h2 id="picker-title">Copy from a Feed on musicindex</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body" ref=${bodyRef}>
      <section class="page-section pinned-form"><div class="group rows">
        <${Field} id="picker-search" label="Name" type="search" value=${query} placeholder="Artist or feed name"
          onInput=${(e) => { setQuery(e.target.value); if (view !== 'hits') { setView('hits'); setMoved((n) => n + 1); } }} />
      </div></section>
      <${ErrorNotice} error=${indexLoad.error} />
      ${view === 'hits' && html`<${SearchResults} query=${query} search=${search} disabled=${indexLoad.busy} onPick=${(row) => openFeed(row.feed_guid)} />`}
      ${view === 'feed' && feedData && html`<div class="group">
          <button class="link-row" type="button" onClick=${backToHits}>
            <span class="go" aria-hidden="true">‹</span><span class="title">Search results</span></button>
        </div>
        <${PickFold} title="Tracks" rows=${feedData.tracks.map((t) => html`<button class="link-row" type="button" key=${t.track_guid} disabled=${indexLoad.busy}
          onClick=${() => openTrack(t.track_guid)}>
          <span class="title">${t.title}</span><span class="go" aria-hidden="true">›</span>
        </button>`)} />
        <section class="page-section"><h2>${feedName}</h2>
          ${feedData.recipients.length === 0 && html`<p class="hint">This feed has no recipients.</p>`}
          ${feedData.recipients.length > 0 && html`<div class="group">${copyRows(feedData.recipients, taken, indexSelected, toggleIndex, feedName)}</div>`}
          <p class="hint">${COPY_NOTE}</p></section>`}
      ${view === 'track' && trackData && html`<div class="group">
          <button class="link-row" type="button" onClick=${backToFeed}>
            <span class="go" aria-hidden="true">‹</span><span class="title">${feedName}</span></button>
        </div>
        <section class="page-section"><h2>${trackHeading(trackName, feedName)}</h2>
          ${trackData.recipients.length === 0 && html`<p class="hint">This track has no recipients.</p>`}
          ${trackData.recipients.length > 0 && html`<div class="group">${copyRows(trackData.recipients, taken, indexSelected, toggleIndex, trackPlace)}</div>`}
          <p class="hint">${COPY_NOTE}</p></section>`}
    </div>
    <footer class="pick-footer">
      ${showSelected && selectedRows.length > 0 && html`<div class="group pick-selected" id="picker-selected">
        ${selectedRows.map((row) => html`<${PickRow} key=${`s-${row.key}`}
          name=${row.recipient.name || row.recipient.address || 'Recipient'} line=${memberText(row.recipient)}
          note=${row.from} checked=${true} taken=${false} onToggle=${row.clear} />`)}
      </div>`}
      <div class="pick-footer-row">
        ${total > 0 && html`<button class="button" type="button"
          aria-expanded=${showSelected ? 'true' : 'false'} aria-controls="picker-selected"
          onClick=${() => setShowSelected(!showSelected)}>
          ${total} Selected <span class="pick-arrow" aria-hidden="true">${showSelected ? '▼' : '▲'}</span></button>`}
        <span class="spacer"></span>
        <button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>
        <button class="button primary" type="button" disabled=${total === 0} onClick=${add}>
          ${addRecipientsLabel(total)}</button>
      </div>
    </footer>
  </dialog>`;
}

// ---- Payment blocks ----

const BLANK_BLOCK = { value_type: 'lightning', method: 'keysend', suggested: null, recipients: [], time_splits: [] };

// The recipients of the first payment block. The editor does not show the
// method of the block: an existing block keeps its method, and a new block
// gets keysend or the method of the album block. Studio keeps each other
// block, and each value time split, with no change.
// No wrapper: Recipients already returns its rows with no wrapper, and the
// hints here join them as direct rows of the group (review of task 014c,
// defect 2 and 3).
function Payments({ id, blocks, onChange }) {
  const block = blocks[0];
  const change = (name) => (value) => onChange(setAt(blocks, 0, set(block, name, value)));
  const splits = block.time_splits.length;
  return html`<${Recipients} id=${id} recipients=${block.recipients}
      defaultType=${block.method === 'lnaddress' ? 'lnaddress' : 'node'} onChange=${change('recipients')} />
    ${splits > 0 && html`<p class="hint">This payment block has ${splits} value time split${splits === 1 ? '' : 's'}. Studio keeps ${splits === 1 ? 'it' : 'them'} with no change.</p>`}
    ${blocks.length > 1 && html`<p class="hint">This feed has ${blocks.length - 1} more payment block${blocks.length === 2 ? '' : 's'} here. Studio keeps ${blocks.length === 2 ? 'it' : 'them'} with no change.</p>`}`;
}

// The Value routes page of an album or a publisher feed: a section, not a
// disclosure group (ADR 0029 item 9). `editor.js` puts this behind the Value
// routes link row. A publisher feed has no tracks, so it shows no track hint
// (ADR 0075).
export function AlbumPayments({ model, update }) {
  const onChange = (values) => update((m) => set(m, 'values', values));
  return html`<section class="page-section">
    <div class="section-head"><h2>Value routes</h2>
      ${model.values.length > 0 && html`<${ShareStack} recipients=${model.values[0].recipients} />`}</div>
    <div class="group">
      ${model.items.length > 0 && html`<p class="hint">These recipients get the payments for each track that has no recipients of its own.</p>`}
      ${model.values.length === 0
        ? html`<p class="hint">This feed has no recipients.</p>
          <button class="link-row add" type="button"
            onClick=${() => onChange(withCids([BLANK_BLOCK]))}><span class="num">＋</span><span class="title">Add recipients</span></button>`
        : html`<${Payments} id="ch-pay" blocks=${model.values} onChange=${onChange} />`}
    </div>
  </section>`;
}

// The recipients of one track, as a section of the track page, not a
// disclosure group (ADR 0029 item 9). They replace the album recipients for
// that track.
// `noun` names the item in the texts: "track", or "show" for a live item.
export function TrackPayments({ id, item, albumBlocks, onChange, noun = 'track' }) {
  const blocks = item.values;
  const shown = blocks.length > 0 ? blocks : albumBlocks;
  const copy = () => onChange(withCids(fresh(albumBlocks.length > 0 ? [albumBlocks[0]] : [BLANK_BLOCK]).map((b) => ({ ...b, time_splits: [] }))));
  const useAlbum = async () => {
    if (await askConfirm(CONFIRMS.removeRecipients(noun))) onChange([]);
  };
  return html`<section class="page-section">
    <div class="section-head"><h2>Value routes</h2>
      ${shown.length > 0 && html`<${ShareStack} recipients=${shown[0].recipients} />`}</div>
    <div class="group">
      ${blocks.length === 0
        ? html`<p class="hint">This ${noun} uses the album recipients.</p>
          <div class="actions"><button class="button" type="button" onClick=${copy}>
            Give This ${noun.charAt(0).toUpperCase() + noun.slice(1)} Its Own Recipients</button></div>`
        : html`<div class="actions"><button class="button" type="button" onClick=${useAlbum}>Use the Album Recipients</button></div>
          <p class="hint">These recipients replace the album recipients for this ${noun} only.</p>
          <${Payments} id=${`${id}-pay`} blocks=${blocks} onChange=${onChange} />`}
    </div>
  </section>`;
}

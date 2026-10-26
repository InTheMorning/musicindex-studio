// The value editor (task 012): the payment recipients of the album and of
// each track. The editor does not change a value time split.
import { html, useState, useEffect, useRef } from './vendor/htm-preact-standalone.module.js';
import { TextField, EntryActions, set, setAt, withCids, startOpen, openOnce, useSheet } from './editor.js';
import { api, Field, ErrorNotice, useAction } from './app.js';

// ---- Shares and percentages ----
// A split is a number of shares, not a percentage. A fee recipient
// (fee="true") takes its split as a percentage first. The other recipients
// share the rest in proportion to their shares. The editor shows the
// percentage as information and does not change a split.

const isFee = (r) => (r.fee || '').toLowerCase() === 'true';
const shares = (r) => {
  const n = Number(r.split);
  return r.split !== null && r.split !== '' && Number.isFinite(n) && n >= 0 ? n : null;
};

export function percentages(recipients) {
  const fees = recipients.filter(isFee).reduce((sum, r) => sum + (shares(r) ?? 0), 0);
  const total = recipients.filter((r) => !isFee(r)).reduce((sum, r) => sum + (shares(r) ?? 0), 0);
  const rest = Math.max(0, 100 - fees);
  return recipients.map((r) => {
    const n = shares(r);
    if (n === null) return null;
    if (isFee(r)) return n;
    return total > 0 ? (n / total) * rest : null;
  });
}

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

// A part under this percentage folds into "Other". The legend names at most
// this number of parts (dataviz guidance: fold the tail into "Other").
const SMALL_PART = 5;
const NAMED_PARTS = 3;

// A short name for the legend: a Lightning address shows its user part.
const shortName = (name) => (name.includes('@') ? name.split('@')[0] : name);

function shareParts(recipients) {
  const percent = percentages(recipients);
  const parts = recipients
    .map((r, i) => ({ name: r.name || r.address || 'Recipient', percent: percent[i], share: shares(r), fee: isFee(r) }))
    .filter((p) => p.percent !== null && p.percent > 0);
  const named = new Set(
    [...parts].filter((p) => p.percent >= SMALL_PART).sort((a, b) => b.percent - a.percent).slice(0, NAMED_PARTS),
  );
  const shown = parts.filter((p) => named.has(p));
  const other = parts.filter((p) => !named.has(p));
  const otherPercent = other.reduce((sum, p) => sum + p.percent, 0);
  if (other.length > 0) {
    shown.push({ name: other.length === 1 ? other[0].name : `${other.length} others`, percent: otherPercent, other: true, members: other });
  }
  return { all: parts, shown };
}

// The tooltip of one part has the name, the percentage and the raw share.
// A fee part also has the word "fee".
const partDetail = (p) => `${p.name} ${percentText(p.percent)} (share ${p.share}${p.fee ? ', fee' : ''})`;

// The stacked bar and its legend. The legend names the biggest parts and
// "Other", in the sequence of the bar. A segment takes the focus from the
// keyboard, unless the bar is inside a button (`focusable` false).
export function ShareStack({ recipients, focusable = true }) {
  const { all, shown } = shareParts(recipients);
  if (shown.length === 0) return null;
  const label = all.map((p) => `${p.name} ${percentText(p.percent)}${p.fee ? ' (fee)' : ''}`).join(', ');
  const tip = (p) => (p.other ? p.members.map(partDetail).join(', ') : partDetail(p));
  return html`<span class="share-preview">
    <span class="share-stack" role="group" aria-label=${`Shares: ${label}`}>
      ${shown.map((p) => html`<span class=${p.other ? 'other' : p.fee ? 'fee' : ''} style=${{ flexGrow: p.percent }}
        data-tip=${tip(p)} tabindex=${focusable ? '0' : null} role="img" aria-label=${tip(p)}></span>`)}
    </span>
    <span class="share-legend" aria-hidden="true">
      ${shown.map((p, i) => html`${i > 0 ? ' · ' : ''}<span class="part">${p.other ? 'Other' : shortName(p.name)} ${percentText(p.percent)}</span>`)}
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
  { value: 'node', label: 'node (keysend)', hint: 'The public key of the Lightning node, 66 hexadecimal characters.' },
  { value: 'lnaddress', label: 'lnaddress', hint: 'A Lightning address, for example name@example.com.' },
];

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
// fee "true" is in "Fees". The model keeps the sequence of the recipients.
function Recipients({ id, recipients, defaultType, onChange }) {
  const percent = percentages(recipients);
  const blank = { name: null, recipient_type: defaultType, address: null, split: '', custom_key: null, custom_value: null, fee: null };
  // The picker sheet (task 013c, ADR 0027). It only adds a recipient. It
  // does not replace or remove a recipient of this list.
  const [picking, setPicking] = useState(false);
  const addPicked = (added) => {
    onChange([...recipients, ...added]);
    setPicking(false);
  };
  const row = (r, i) => {
      const change = (name) => (value) => onChange(setAt(recipients, i, set(r, name, value)));
      const rid = `${id}-r-${r._cid}`;
      const name = r.name || r.address || 'this recipient';
      const type = TYPES.find((t) => t.value === r.recipient_type);
      return html`<details class="recipient" name=${`${id}-recipients`} key=${r._cid} ref=${openOnce(r._cid)}>
        <summary>
          <span class="title">${r.name || 'New recipient'}</span>
          <${Meter} percent=${percent[i]} />
          <span class="share">${percentText(percent[i])}${isFee(r) ? ' fee' : ''}</span>
          <span class="mono address">${r.address || 'No address'}</span>
        </summary>
        <div class="form">
        <div class="grid2">
          <${TextField} id=${`${rid}-name`} label="Name" value=${r.name} onChange=${change('name')} />
          <${Choice} id=${`${rid}-type`} label="Type" value=${r.recipient_type} options=${TYPES} onChange=${change('recipient_type')} />
        </div>
        <${TextField} id=${`${rid}-address`} label="Address" hint=${type ? type.hint : 'A node public key, or a Lightning address.'}
          value=${r.address} onChange=${change('address')} />
        ${r.recipient_type !== 'lnaddress' && html`<div class="grid2">
          <${TextField} id=${`${rid}-key`} label="Custom key" hint="For a node that many people share." value=${r.custom_key} onChange=${change('custom_key')} />
          <${TextField} id=${`${rid}-value`} label="Custom value" hint="The account on that node." value=${r.custom_value} onChange=${change('custom_value')} />
        </div>`}
        <div class="grid2">
          <${TextField} id=${`${rid}-split`} label=${isFee(r) ? 'Split (percent)' : 'Split (shares)'} inputmode="numeric" value=${r.split} onChange=${change('split')} />
        </div>
        <label class="check"><input type="checkbox" checked=${isFee(r)}
          onChange=${(e) => change('fee')(e.target.checked ? 'true' : null)} />
          Fee: this recipient gets its split as a percentage before the others share the rest</label>
        <${SaveRecipientButton} recipient=${r} />
        <${EntryActions} list=${recipients} index=${i} name=${name} onChange=${onChange} />
        </div>
      </details>`;
  };
  // A group holds the rows of one kind. Each row keeps its index in the list.
  const ofGroup = (fee) => recipients.map((r, i) => (isFee(r) === fee ? row(r, i) : null));
  return html`<h3 class="value-group">Recipients</h3>
    ${ofGroup(false)}
    ${!recipients.some((r) => !isFee(r)) && html`<p class="hint">No recipients.</p>`}
    <button class="link-row add" type="button"
      onClick=${() => {
        const added = withCids([blank])[0];
        startOpen.add(added._cid);
        onChange([...recipients, added]);
      }}><span class="num">＋</span><span class="title">Add a recipient</span></button>
    <button class="link-row add" type="button" onClick=${() => setPicking(true)}>
      <span class="num">＋</span><span class="title">Pick from your recipients</span></button>
    <h3 class="value-group">Fees</h3>
    ${ofGroup(true)}
    ${!recipients.some(isFee) && html`<p class="hint">No fees.</p>`}
    <${PickerSheet} open=${picking} recipients=${recipients} onAdd=${addPicked} onClose=${() => setPicking(false)} />`;
}

// ---- Save a recipient (task 013c, ADR 0027 item 5) ----

// The button of an open recipient that saves it to the user's list. It
// shows "Saved" for 2 seconds after the save.
function SaveRecipientButton({ recipient }) {
  const { busy, error, run } = useAction();
  const [saved, setSaved] = useState(false);
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
    setTimeout(() => setSaved(false), 2000);
  });
  return html`<${ErrorNotice} error=${error} />
    <div class="actions">
      <button class="button action" type="button" disabled=${busy} onClick=${save}>
        ${saved ? 'Saved' : busy ? 'Saving…' : 'Save to your recipients'}</button>
    </div>`;
}

// ---- The recipient picker (task 013c, ADR 0027) ----
// The picker adds recipients from the user's list and from musicindex. It
// only adds a recipient. It does not remove or replace a recipient of the
// editor (ADR 0027 invariant).

// The identity of a recipient: its address, its custom key and its custom
// value (ADR 0027 item 3). Two rows with the same identity are the same
// recipient, even with a different name.
function identityOf(r) {
  return `${r.address || ''}\u0000${r.custom_key || ''}\u0000${r.custom_value || ''}`;
}

// One row of a part of the picker: a checkbox, the name, the address and a
// status word. The checkbox sits in its own label of one target size, so
// that a Remove button can share the row (musicindex ADR 0029 R-SCALE-3).
function PickRow({ name, address, status, checked, taken, onToggle, remove }) {
  const label = taken ? 'Already in this list' : status;
  return html`<div class="link-row pick-row">
    <label class="pick-box"><input type="checkbox" checked=${checked || taken} disabled=${taken}
      aria-label=${name} onChange=${onToggle} /></label>
    <span class="text"><span class="title">${name}</span><span class="sub mono">${address}</span></span>
    <span class="summary">${label || ''}</span>
    <span class="pick-action">${remove || ''}</span>
  </div>`;
}

// The picker sheet: musicindex, then the user's recipients (ADR 0027 items
// 1, 5 to 7, and ADR 0030). `Recipients` opens it from the album and from each track.
function PickerSheet({ open, recipients, onAdd, onClose }) {
  const ref = useSheet(open);
  const taken = new Set(recipients.map(identityOf));

  // ---- Your recipients ----
  const [list, setList] = useState(null);
  const [filter, setFilter] = useState('');
  const [yoursSelected, setYoursSelected] = useState(new Set());
  const yours = useAction();
  const toggleYours = (key) => setYoursSelected((selected) => {
    const next = new Set(selected);
    if (next.has(key)) next.delete(key); else next.add(key);
    return next;
  });
  const removeSaved = (savedId) => yours.run(async () => {
    await api(`/recipients/saved/${savedId}`, { method: 'DELETE' });
    setList(await api('/recipients'));
  });
  const q = filter.trim().toLowerCase();
  const shownYours = (list || []).filter(
    (r) => !q || `${r.name || ''} ${r.address || ''}`.toLowerCase().includes(q),
  );

  // ---- From musicindex ----
  const [query, setQuery] = useState('');
  const [hits, setHits] = useState(null);
  const [view, setView] = useState('hits'); // 'hits', 'feed' or 'track'
  const [feedData, setFeedData] = useState(null);
  const [trackData, setTrackData] = useState(null);
  const [indexSelected, setIndexSelected] = useState(new Map());
  const search = useAction();
  const indexLoad = useAction();
  const runSearch = (event) => {
    event.preventDefault();
    if (!query.trim()) return;
    search.run(async () => setHits(await api(`/search?q=${encodeURIComponent(query.trim())}`)));
  };
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
  // After each move between the hits, a feed and a track, the sheet
  // scrolls to the top of the musicindex results. The user then sees the
  // recipients of the page that opened.
  const [moved, setMoved] = useState(0);
  const resultsRef = useRef(null);
  useEffect(() => {
    if (moved > 0 && resultsRef.current) resultsRef.current.scrollIntoView({ block: 'start' });
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
    setFilter('');
    setYoursSelected(new Set());
    setShowSelected(false);
    setMoved(0);
    setQuery('');
    setHits(null);
    setView('hits');
    setFeedData(null);
    setTrackData(null);
    setIndexSelected(new Map());
    setList(null);
    yours.run(async () => setList(await api('/recipients')));
  }, [open]);

  const total = yoursSelected.size + indexSelected.size;
  const selectedRows = [
    ...(list || []).filter((r) => yoursSelected.has(identityOf(r)))
      .map((r) => ({ key: identityOf(r), recipient: r, from: 'Your recipients', clear: () => toggleYours(identityOf(r)) })),
    ...[...indexSelected].map(([key, entry]) => ({ key, recipient: entry.recipient, from: entry.from,
      clear: () => toggleIndex(entry.recipient) })),
  ];
  const add = () => {
    const combined = new Map();
    for (const r of list || []) {
      if (yoursSelected.has(identityOf(r))) {
        combined.set(identityOf(r), {
          name: r.name, recipient_type: r.recipient_type, address: r.address, split: '',
          custom_key: r.custom_key, custom_value: r.custom_value, fee: null,
        });
      }
    }
    // An imported recipient keeps its fee mark but gets no split. The user
    // gives each new recipient its split (ADR 0030).
    for (const [key, entry] of indexSelected) combined.set(key, { ...entry.recipient, split: '' });
    onAdd(withCids(fresh([...combined.values()])));
  };

  return html`<dialog ref=${ref} class="sheet" aria-labelledby="picker-title" onClose=${onClose}>
    <header>
      <h2 id="picker-title">Pick recipients</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    <div class="body">
      <section class="page-section">
        <h2>From musicindex</h2>
        <form class="form" onSubmit=${runSearch} role="search">
          <div class="field">
            <label for="picker-search">Search musicindex</label>
            <div class="inline">
              <input id="picker-search" type="search" value=${query} placeholder="Artist or feed name"
                onInput=${(e) => setQuery(e.target.value)} />
              <button class="button" type="submit" disabled=${search.busy || !query.trim()}>
                ${search.busy ? 'Searching…' : 'Search'}</button>
            </div>
          </div>
        </form>
        <span class="pick-anchor" ref=${resultsRef}></span>
        <${ErrorNotice} error=${search.error} />
        <${ErrorNotice} error=${indexLoad.error} />
        ${view === 'hits' && hits && hits.length === 0 && html`<p class="hint">musicindex found no feed with that name.</p>`}
        ${view === 'hits' && hits && hits.length > 0 && html`<div class="group">
          ${hits.map((h) => html`<button class="link-row" type="button" key=${h.feed_guid} disabled=${indexLoad.busy}
            onClick=${() => openFeed(h.feed_guid)}>
            <span class="dot feed"></span><span class="title">${h.title || h.feed_guid}</span>
            <span class="go" aria-hidden="true">›</span>
          </button>`)}
        </div>`}
        ${view === 'feed' && feedData && html`<div class="group">
          <button class="link-row" type="button" onClick=${backToHits}>
            <span class="go" aria-hidden="true">‹</span><span class="title">Search results</span></button>
          <h3 class="pick-head"><span class="dot feed"></span>${feedName}</h3>
          ${feedData.recipients.map((r) => {
            const key = identityOf(r);
            return html`<${PickRow} key=${`f-${key}`} name=${r.name || r.address || 'Recipient'} address=${r.address}
              status=${null} checked=${indexSelected.has(key)} taken=${taken.has(key)} onToggle=${() => toggleIndex(r, feedName)} />`;
          })}
          ${feedData.tracks.length > 0 && html`<h3 class="pick-head sub">Tracks</h3>`}
          ${feedData.tracks.map((t) => html`<button class="link-row" type="button" key=${t.track_guid} disabled=${indexLoad.busy}
            onClick=${() => openTrack(t.track_guid)}>
            <span class="title">${t.title}</span><span class="go" aria-hidden="true">›</span>
          </button>`)}
        </div>`}
        ${view === 'track' && trackData && html`<div class="group">
          <button class="link-row" type="button" onClick=${backToFeed}>
            <span class="go" aria-hidden="true">‹</span><span class="title">${feedName}</span></button>
          <h3 class="pick-head">${trackName}${trackName !== feedName && html`<span class="muted"> · ${feedName}</span>`}</h3>
          ${trackData.recipients.length === 0 && html`<p class="hint">This track has no recipients.</p>`}
          ${trackData.recipients.map((r) => {
            const key = identityOf(r);
            return html`<${PickRow} key=${`t-${key}`} name=${r.name || r.address || 'Recipient'} address=${r.address}
              status=${null} checked=${indexSelected.has(key)} taken=${taken.has(key)} onToggle=${() => toggleIndex(r, trackPlace)} />`;
          })}
        </div>`}
      </section>

      <section class="page-section">
        <h2>Your recipients</h2>
        <${Field} id="picker-filter" label="Filter" value=${filter} placeholder="Name or address"
          onInput=${(e) => setFilter(e.target.value)} />
        <${ErrorNotice} error=${yours.error} />
        ${list === null
          ? html`<p class="hint" role="status">Loading your recipients…</p>`
          : html`<div class="group">
              ${shownYours.length === 0 && html`<p class="hint">${list.length === 0
                ? 'You have no recipients yet.' : 'No recipient matches this filter.'}</p>`}
              ${shownYours.map((r) => {
                const key = identityOf(r);
                return html`<${PickRow} key=${key} name=${r.name || r.address || 'Recipient'} address=${r.address}
                  status=${r.saved_id !== null ? 'Saved' : `In ${r.feeds} feed${r.feeds === 1 ? '' : 's'}`}
                  checked=${yoursSelected.has(key)} taken=${taken.has(key)} onToggle=${() => toggleYours(key)}
                  remove=${r.saved_id !== null && html`<button class="button destructive" type="button" disabled=${yours.busy}
                    onClick=${() => removeSaved(r.saved_id)}>Remove</button>`} />`;
              })}
            </div>`}
      </section>
    </div>
    <footer class="pick-footer">
      ${showSelected && selectedRows.length > 0 && html`<div class="group pick-selected" id="picker-selected">
        ${selectedRows.map((row) => html`<${PickRow} key=${`s-${row.key}`}
          name=${row.recipient.name || row.recipient.address || 'Recipient'} address=${row.recipient.address}
          status=${row.from} checked=${true} taken=${false} onToggle=${row.clear} />`)}
      </div>`}
      <div class="pick-footer-row">
        <button class="button" type="button" disabled=${total === 0}
          aria-expanded=${showSelected && total > 0 ? 'true' : 'false'} aria-controls="picker-selected"
          onClick=${() => setShowSelected(!showSelected)}>
          ${total} selected <span class="pick-arrow" aria-hidden="true">${showSelected && total > 0 ? '▼' : '▲'}</span></button>
        <span class="spacer"></span>
        <button class="button" type="button" onClick=${() => ref.current.close()}>Cancel</button>
        <button class="button primary" type="button" disabled=${total === 0} onClick=${add}>
          Add ${total} recipient${total === 1 ? '' : 's'}</button>
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

// The recipients of the album.
// A short summary of the recipients of a block list. `editor.js` uses this
// for the Recipients link row of the album page.
export function recipientSummary(blocks) {
  const count = blocks.length > 0 ? blocks[0].recipients.length : 0;
  return `${count} recipient${count === 1 ? '' : 's'}`;
}

// The Value routes page of the album: a section, not a disclosure group (ADR
// 0029 item 9). `editor.js` puts this behind the Value routes link row of the
// album page.
export function AlbumPayments({ model, update }) {
  const onChange = (values) => update((m) => set(m, 'values', values));
  return html`<section class="page-section">
    <div class="section-head"><h2>Value routes</h2>
      ${model.values.length > 0 && html`<${ShareStack} recipients=${model.values[0].recipients} />`}</div>
    <div class="group">
      <p class="hint">These recipients get the payments for each track that has no recipients of its own.</p>
      ${model.values.length === 0
        ? html`<p class="hint">This album has no recipients.</p>
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
  const useAlbum = () => {
    if (confirm(`Remove the recipients of this ${noun}? The ${noun} then uses the album recipients.`)) onChange([]);
  };
  return html`<section class="page-section">
    <div class="section-head"><h2>Value routes</h2>
      ${shown.length > 0 && html`<${ShareStack} recipients=${shown[0].recipients} />`}</div>
    <div class="group">
      ${blocks.length === 0
        ? html`<p class="hint">This ${noun} uses the album recipients.</p>
          <div class="actions"><button class="button" type="button" onClick=${copy}>
            Give this ${noun} its own recipients</button></div>`
        : html`<div class="actions"><button class="button" type="button" onClick=${useAlbum}>Use the album recipients</button></div>
          <p class="hint">These recipients replace the album recipients for this ${noun} only.</p>
          <${Payments} id=${`${id}-pay`} blocks=${blocks} onChange=${onChange} />`}
    </div>
  </section>`;
}

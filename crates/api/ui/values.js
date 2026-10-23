// The value editor (task 012): the payment recipients of the album and of
// each track. The editor does not change a value time split.
import { html } from './vendor/htm-preact-standalone.module.js';
import { TextField, EntryActions, set, setAt, withCids, startOpen, openOnce } from './editor.js';

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
    .map((r, i) => ({ name: r.name || r.address || 'Recipient', percent: percent[i], fee: isFee(r) }))
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

// The stacked bar and its legend. The legend names the biggest parts and
// "Other", in the sequence of the bar.
export function ShareStack({ recipients }) {
  const { all, shown } = shareParts(recipients);
  if (shown.length === 0) return null;
  const label = all.map((p) => `${p.name} ${percentText(p.percent)}${p.fee ? ' (fee)' : ''}`).join(', ');
  const tip = (p) => (p.other
    ? p.members.map((m) => `${m.name} ${percentText(m.percent)}`).join(', ')
    : `${p.name} ${percentText(p.percent)}${p.fee ? ' (fee)' : ''}`);
  return html`<span class="share-preview">
    <span class="share-stack" role="img" aria-label=${`Shares: ${label}`}>
      ${shown.map((p) => html`<span class=${p.other ? 'other' : p.fee ? 'fee' : ''} style=${{ flexGrow: p.percent }}
        data-tip=${tip(p)}></span>`)}
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
// own inset separator (review of task 014c, defect 3).
function Recipients({ id, recipients, defaultType, onChange }) {
  const percent = percentages(recipients);
  const blank = { name: null, recipient_type: defaultType, address: null, split: '', custom_key: null, custom_value: null, fee: null };
  return html`${recipients.length === 0 && html`<p class="hint">No recipients yet.</p>`}
    ${recipients.map((r, i) => {
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
        <${EntryActions} list=${recipients} index=${i} name=${name} onChange=${onChange} />
        </div>
      </details>`;
    })}
    <button class="link-row add" type="button"
      onClick=${() => {
        const added = withCids([blank])[0];
        startOpen.add(added._cid);
        onChange([...recipients, added]);
      }}><span class="num">＋</span><span class="title">Add a recipient</span></button>`;
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

// The recipients page of the album: a section, not a disclosure group (ADR
// 0029 item 9). `editor.js` puts this behind the Recipients link row of the
// album page.
export function AlbumPayments({ model, update }) {
  const onChange = (values) => update((m) => set(m, 'values', values));
  return html`<section class="page-section">
    <div class="section-head"><h2>${recipientSummary(model.values)}</h2>
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
export function TrackPayments({ id, item, albumBlocks, onChange }) {
  const blocks = item.values;
  const shown = blocks.length > 0 ? blocks : albumBlocks;
  const copy = () => onChange(withCids(fresh(albumBlocks.length > 0 ? [albumBlocks[0]] : [BLANK_BLOCK]).map((b) => ({ ...b, time_splits: [] }))));
  const useAlbum = () => {
    if (confirm('Remove the recipients of this track? The track then uses the album recipients.')) onChange([]);
  };
  return html`<section class="page-section">
    <div class="section-head"><h2>Recipients</h2>
      ${shown.length > 0 && html`<${ShareStack} recipients=${shown[0].recipients} />`}</div>
    <div class="group">
      ${blocks.length === 0
        ? html`<p class="hint">This track uses the album recipients.</p>
          <div class="actions"><button class="button" type="button" onClick=${copy}>
            Give this track its own recipients</button></div>`
        : html`<div class="actions"><button class="button" type="button" onClick=${useAlbum}>Use the album recipients</button></div>
          <p class="hint">These recipients replace the album recipients for this track only.</p>
          <${Payments} id=${`${id}-pay`} blocks=${blocks} onChange=${onChange} />`}
    </div>
  </section>`;
}

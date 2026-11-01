// The screens of the People area (ADR 0099, task 065). The components show
// the pages and call the API. The functions of `people.js` give each text
// and each state, and the core applies each rule (ADR 0042, ADR 0065).
import { html, useState, useEffect } from './vendor/htm-preact-standalone.module.js';
import { useSheet } from './editor.js';
import { api, Bar, ChoiceSelect, ErrorNotice, FeedCheckRow, Field, useAction, useImage } from './app.js';
import { pageSequence } from './pages.js';
import { shortKey } from './nostrkey.js';
import {
  recipientKey, recipientRow, breakParts, addressLines, lnaddressOf, suggestionField, OTHER_ADDRESS, walletActions, WALLET_TYPES,
  CUSTOM_KEY_HINT, removeWalletHint, walletSheet, walletSheetStart, walletTypeChange, walletDetailsLine, walletSheetBody,
  walletSheetReady, recipientTitle,
} from './recipients.js';
import {
  peopleSections, PEOPLE_EMPTY, suggestionRow, personRow, makePersonStart, makePersonBody, initials,
  filterPeople, filterPayees, payeesWithNoPerson, openChanges, changeRow, memberChoices, makeSheetTitle,
  makeSheetHint, creditItemRow, payeeItemRow, keyItemRow, toggledItem, dismissBody, addPersonBody,
  checkedIds, toggledRow, reviewLabel, addResultLine, keyValue, keyFromValue, memberLine, walletGroups, addressOptions,
  usedText, keyEvidenceText, whereRow, feedRows, levelRows, feedsHeading, SHEET_TITLES, addIntro, useHint, renameHint,
  removeHint, sheetStart, sheetBody, sheetReady, supportChoices, BUSY_REVIEW, LINK_FORMS, linkFields, linkStart, linkBody,
  linkReady, personFieldsBody, profileLine,
} from './people.js';

// The round picture of a person. A person with no picture, or a picture that
// Studio cannot read, shows the initials of the name.
export function PersonArt({ person, large = false }) {
  const { src } = useImage(person.img);
  return html`<span class=${large ? 'person-art large' : 'person-art'} aria-hidden="true">${src ? html`<img src=${src} alt="" />` : initials(person.name)}</span>`;
}

// One row of a list: a button with the lines of its model.
function ListRow({ title, lines, count, lead, onClick }) {
  return html`<button class="link-row" type="button" onClick=${onClick}>${lead}
    <span class="text"><span class="label">${title}</span>${lines.map((line) => html`<span class="summary" key=${line}>${line}</span>`)}</span>
    ${count && html`<span class="count">${count}</span>`}<span class="go">›</span></button>`;
}

// Download the file of `GET /people/export`. The route needs the access
// token, so the page reads the file and gives it to the browser.
async function downloadPeople() {
  const blob = await api('/people/export', { raw: true });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = 'studio-people.json';
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}

// The People page (ADR 0099 item 2).
export function PeoplePage({ feeds, go }) {
  const [data, setData] = useState(null);
  const [query, setQuery] = useState('');
  const [making, setMaking] = useState(null);
  const [adding, setAdding] = useState(false);
  const { busy, error, run } = useAction();
  const load = () => run(async () => {
    const [people, suggestions, sets, recipients] = await Promise.all([
      api('/people'), api('/people/suggestions'), api('/change-sets'), api('/recipients'),
    ]);
    setData({ people, suggestions, changes: openChanges(sets), recipients });
  });
  useEffect(() => { load(); }, []);

  const titles = new Map((feeds || []).map((f) => [f.id, f.title || 'Untitled feed']));
  const people = data ? filterPeople(data.people, query) : [];
  const payees = data ? filterPayees(payeesWithNoPerson(data.recipients), query) : [];
  const sections = data ? peopleSections({ people, suggestions: data.suggestions, changes: data.changes, payees }) : [];
  const sectionViews = {
    search: () => html`<div class="field search-field"><div class="field-body">
      <input type="search" aria-label="Search people, addresses and keys" placeholder="Search people, addresses and keys"
        value=${query} onInput=${(e) => setQuery(e.target.value)} /></div></div>`,
    actions: () => html`<div class="actions">
      <button class="button action" type="button" onClick=${() => setAdding(true)}>Add a Person</button>
      <button class="button" type="button" disabled=${busy} onClick=${() => run(downloadPeople)}>Export People</button></div>`,
    changes: () => html`<section class="page-section"><h2>Changes to write</h2><div class="group">${data.changes.map((set) => {
      const row = changeRow(set, data.people, payeesWithNoPerson(data.recipients));
      return html`<${ListRow} key=${set.id} title=${row.title} lines=${[row.text]} onClick=${() => go({ name: 'change-set', id: set.id })} />`;
    })}</div></section>`,
    suggestions: () => html`<section class="page-section"><h2>Suggestions from your feeds</h2>
      <p class="note">Studio found these names in your feeds. Check each one, then make a person.</p>
      <div class="group">${data.suggestions.map((suggestion) => {
        const row = suggestionRow(suggestion);
        return html`<${ListRow} key=${suggestion.identity} title=${row.title} lines=${row.lines} count="Check"
          lead=${html`<${PersonArt} person=${{ name: suggestion.name, img: (suggestion.credits.find((c) => c.img) || {}).img || '' }} />`}
          onClick=${() => setMaking(suggestion)} />`;
      })}</div></section>`,
    people: () => html`<section class="page-section"><h2>People</h2>
      ${data.people.length === 0 && html`<p class="empty">${PEOPLE_EMPTY}</p>`}
      ${data.people.length > 0 && people.length === 0 && html`<p class="empty">No person matches the search.</p>`}
      ${people.length > 0 && html`<div class="group">${people.map((person) => {
        const row = personRow(person, titles);
        return html`<${ListRow} key=${person.id} title=${row.title} lines=${row.lines} count=${row.count}
          lead=${html`<${PersonArt} person=${person} />`} onClick=${() => go({ name: 'person', id: person.id })} />`;
      })}</div>`}</section>`,
    payees: () => html`<section class="page-section"><h2>Payees with no person</h2><div class="group">${payees.map((recipient) => {
      const row = recipientRow(recipient);
      return html`<${ListRow} key=${JSON.stringify(recipientKey(recipient))} title=${row.title} lines=${row.lines} count=${row.count}
        onClick=${() => go({ name: 'wallet', recipient })} />`;
    })}</div></section>`,
  };
  return html`<div><${Bar}><span class="bar-title">People</span><//><section class="section">
    <${ErrorNotice} error=${error} />
    ${data === null && !error && html`<p class="empty" role="status">Loading the people…</p>`}
    ${sections.map((id) => html`<${Fragment} key=${id}>${sectionViews[id]()}<//>`)}
  </section>
  <${MakePersonSheet} suggestion=${making} feeds=${feeds} titles=${titles} go=${go}
    onClose=${() => setMaking(null)} onDismissed=${() => { setMaking(null); load(); }} />
  <${AddPersonSheet} open=${adding} go=${go} onClose=${() => setAdding(false)} /></div>`;
}

const Fragment = ({ children }) => children;

// The sheet "Make a Person: {name}" (ADR 0099 item 3). The sheet changes no
// feed. The core makes the person, and a 409 gives its message here.
function MakePersonSheet({ suggestion, feeds, titles, go, onClose, onDismissed }) {
  const ref = useSheet(suggestion !== null);
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="make-person-title" onClose=${onClose}>
    ${suggestion && html`<${MakePersonForm} key=${suggestion.identity} suggestion=${suggestion} feeds=${feeds} titles=${titles}
      go=${go} onCancel=${() => ref.current.close()} onDismissed=${onDismissed} />`}
  </dialog>`;
}

function MakePersonForm({ suggestion, feeds, titles, go, onCancel, onDismissed }) {
  const [state, setState] = useState(() => makePersonStart(suggestion));
  const { busy, error, run } = useAction();
  const make = () => run(async () => {
    const made = await api('/people/suggestions/make', { method: 'POST', body: makePersonBody(state) });
    go({ name: 'person', id: made.id });
  });
  const dismiss = () => run(async () => {
    await api('/people/suggestions/dismiss', { method: 'POST', body: dismissBody(suggestion) });
    onDismissed();
  });
  const group = (title, name, rows, rowOf) => rows.length > 0 && html`<section class="page-section"><h2>${title}</h2>
    <div class="group rows">${rows.map((r, i) => {
      const row = rowOf(r.item);
      return html`<${FeedCheckRow} key=${i} title=${row.title} text=${row.lines.join(' · ')} checked=${r.checked}
        onChange=${() => setState(toggledItem(state, name, i))} />`;
    })}</div></section>`;
  return html`<header>
      <h2 id="make-person-title">${makeSheetTitle(suggestion.name)}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${onCancel}>✕</button>
    </header>
    <div class="body">
      <p class="hint">${makeSheetHint(suggestion.name)}</p>
      ${group('Credits', 'credits', state.credits, (item) => creditItemRow(item, titles))}
      ${group('Payment addresses', 'recipients', state.recipients, payeeItemRow)}
      ${group('Nostr key', 'keys', state.keys, keyItemRow)}
      <section class="page-section"><h2>Artist or label</h2><div class="group rows">
        <${ChoiceSelect} id="make-member" label="Member of" value=${state.memberOf} placeholder="None"
          hint="The artist or label feed where this person is a member." options=${memberChoices(feeds)}
          onChange=${(memberOf) => setState({ ...state, memberOf })} /></div></section>
      <${ErrorNotice} error=${error} />
    </div>
    <footer>
      <button class="button" type="button" disabled=${busy} onClick=${dismiss}>Not This Person</button>
      <button class="button primary" type="button" disabled=${busy} onClick=${make}>${busy ? 'Making the Person…' : 'Make the Person'}</button>
    </footer>`;
}

// The sheet "Add a Person" with the field "Name" (ADR 0099 item 2).
function AddPersonSheet({ open, go, onClose }) {
  const ref = useSheet(open);
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="add-person-title" onClose=${onClose}>
    ${open && html`<${AddPersonForm} go=${go} onCancel=${() => ref.current.close()} />`}
  </dialog>`;
}

function AddPersonForm({ go, onCancel }) {
  const [name, setName] = useState('');
  const { busy, error, run } = useAction();
  const body = addPersonBody(name);
  const add = (e) => {
    e.preventDefault();
    if (body.name === '') return;
    run(async () => {
      const added = await api('/people', { method: 'POST', body });
      go({ name: 'person', id: added.id });
    });
  };
  return html`<header>
      <h2 id="add-person-title">Add a Person</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${onCancel}>✕</button>
    </header>
    <form class="body" id="add-person-form" onSubmit=${add}>
      <div class="group rows"><${Field} id="add-person-name" label="Name" required value=${name}
        onInput=${(e) => setName(e.target.value)} /></div>
      <${ErrorNotice} error=${error} />
    </form>
    <footer><button class="button primary" type="submit" form="add-person-form" disabled=${busy || body.name === ''}>${busy ? 'Adding…' : 'Add the Person'}</button></footer>`;
}

// ---- The page of a person (ADR 0099 items 4 to 12) ----

// One address row of a wallet: its line, its feeds and an optional note.
function AddressRow({ member, count, note, action }) {
  const { label, address, account } = memberLine(member, true);
  return html`<div class="link-row static"><span class="text"><span class="label">${label}</span>
      ${address && html`<span class="summary mono address-text">${address}</span>`}
      ${account && html`<span class="summary">Account ${account}</span>`}
      ${note && html`<span class="summary">${note}</span>`}</span>
    ${count && html`<span class="count">${count}</span>`}${action}</div>`;
}

// A row that adds something: a plus sign and a title.
const AddRow = ({ title, onClick }) => html`<button class="link-row add" type="button" onClick=${onClick}>
  <span class="num">＋</span><span class="title">${title}</span></button>`;

// The four fields of a person. A change of one field sends the four
// fields with `PATCH /people/{id}` when the field loses the focus.
function PersonFields({ person, onSave }) {
  const [values, setValues] = useState(() => personFieldsBody(person));
  useEffect(() => { setValues(personFieldsBody(person)); }, [person.name, person.usual_role, person.img, person.href]);
  const field = (id, label, key, hint) => html`<${Field} id=${id} label=${label} hint=${hint} value=${values[key]}
    onInput=${(e) => setValues({ ...values, [key]: e.target.value })}
    onChange=${(e) => { if (e.target.value !== person[key]) onSave(personFieldsBody(person, { [key]: e.target.value })); }} />`;
  return html`<div class="group rows">
    ${field('person-name', 'Name', 'name')}
    ${field('person-role', 'Usual role', 'usual_role', 'The start value of a new credit.')}
    ${field('person-img', 'Picture address', 'img')}
    ${field('person-href', 'Information address', 'href')}</div>`;
}

// The sheet that adds a link to a person (ADR 0099 item 4).
function LinkSheet({ kind, personId, onClose, onAdded }) {
  const ref = useSheet(kind !== null);
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="link-title" onClose=${onClose}>
    ${kind && html`<${LinkForm} key=${kind} kind=${kind} personId=${personId} onCancel=${() => ref.current.close()}
      onAdded=${() => { ref.current.close(); onAdded(); }} />`}
  </dialog>`;
}

function LinkForm({ kind, personId, onCancel, onAdded }) {
  const form = LINK_FORMS[kind];
  const [values, setValues] = useState(() => linkStart(kind));
  const { busy, error, run } = useAction();
  const set = (id, value) => setValues({ ...values, [id]: value });
  const add = (e) => {
    e.preventDefault();
    if (!linkReady(kind, values)) return;
    run(async () => {
      await api(`/people/${personId}/${kind}`, { method: 'POST', body: linkBody(kind, values) });
      onAdded();
    });
  };
  return html`<header>
      <h2 id="link-title">${form.title}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${onCancel}>✕</button>
    </header>
    <form class="body" id="link-form" onSubmit=${add}>
      <div class="group rows">${linkFields(kind, values).map((f) => (f.select
        ? html`<${ChoiceSelect} key=${f.id} id=${`link-${f.id}`} label=${f.label} value=${values[f.id]} options=${f.select}
            onChange=${(v) => set(f.id, v)} />`
        : html`<${Field} key=${f.id} id=${`link-${f.id}`} label=${f.label} hint=${f.hint} required=${!!f.required}
            value=${values[f.id]} onInput=${(e) => set(f.id, e.target.value)} />`))}</div>
      <${ErrorNotice} error=${error} />
    </form>
    <footer><button class="button primary" type="submit" form="link-form" disabled=${busy || !linkReady(kind, values)}>${busy ? 'Adding…' : form.button}</button></footer>`;
}

// The sheet of one operation of a person (ADR 0099 items 7 to 12). The
// sheet sends `POST /people/{id}/changes` and opens the review. The core
// makes each change and gives each outcome.
function OperationSheet({ kind, person, groups, rows, feeds, go, onClose }) {
  const ref = useSheet(kind !== null);
  return html`<dialog ref=${ref} class=${`sheet${kind === 'add_to_releases' ? ' small' : ''}`} aria-labelledby="operation-title" onClose=${onClose}>
    <header>
      <h2 id="operation-title">${kind ? SHEET_TITLES[kind] : ''}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    ${kind && html`<${OperationForm} key=${kind} kind=${kind} person=${person} groups=${groups} rows=${rows} feeds=${feeds} go=${go} />`}
  </dialog>`;
}

// The feeds of a sheet with a check box for each. The sheets of a person and
// of a wallet use it (ADR 0099 item 7).
function FeedChecks({ rows, setRows, empty }) {
  if (!rows) return null;
  if (rows.length === 0) return html`<p class="empty">${empty}</p>`;
  return html`<section class="page-section"><h2>${feedsHeading(rows)}</h2><div class="group rows">${rows.map((row) =>
    html`<${FeedCheckRow} key=${row.id} title=${row.title} text=${row.lines.join(' · ')} checked=${row.checked}
      onChange=${() => setRows(toggledRow(rows, row.id))} />`)}</div></section>`;
}

function OperationForm({ kind, person, groups, rows: startRows, feeds, go }) {
  const adding = kind === 'add_to_releases';
  const [fields, setFields] = useState(() => sheetStart(kind, person, groups));
  const [rows, setRows] = useState(adding ? null : startRows);
  const [line, setLine] = useState('');
  const [previewError, setPreviewError] = useState(null);
  const { busy, error, run } = useAction();
  const set = (patch) => setFields({ ...fields, ...patch });
  const ids = rows ? checkedIds(rows) : [];
  const ready = sheetReady(kind, fields, ids);
  const owners = memberChoices(feeds);
  // The core lists the releases of the artist or label. Each starts selected.
  const chooseOwner = (owner) => {
    set({ owner });
    setRows(null);
    if (owner === '') return;
    run(async () => {
      const level = await api(`/feeds/${owner}/level`);
      setRows(levelRows(level.feed_ids, feeds));
    });
  };
  // After each change of a field, the core gives the outcome of each release.
  useEffect(() => {
    if (!adding || !ready) { setLine(''); setPreviewError(null); return undefined; }
    let live = true;
    const timer = setTimeout(() => {
      api(`/people/${person.id}/changes/preview`, { method: 'POST', body: { operation: sheetBody(kind, fields), feed_ids: ids } })
        .then((result) => { if (live) { setLine(addResultLine(result.feeds.map((f) => f.outcome))); setPreviewError(null); } },
          (e) => { if (live) { setLine(''); setPreviewError(e); } });
    }, 250);
    return () => { live = false; clearTimeout(timer); };
  }, [adding, ready, JSON.stringify(fields), ids.join(',')]);
  const submit = () => run(async () => {
    const made = await api(`/people/${person.id}/changes`, { method: 'POST', body: { operation: sheetBody(kind, fields), feed_ids: ids } });
    go({ name: 'change-set', id: made.id });
  });
  const check = (id, title, text) => html`<${FeedCheckRow} title=${title} text=${text} checked=${fields[id]}
    onChange=${() => set({ [id]: !fields[id] })} />`;
  return html`<div class="body">
      ${adding && html`<p class="hint">${addIntro(person.name)}</p>
        <div class="group rows"><${ChoiceSelect} id="op-owner" label="Artist or label" value=${fields.owner} onChange=${chooseOwner}
          placeholder="Select an artist or label" options=${owners} /></div>
        <div class="group rows">
          ${check('payee', 'As a payee')}
          ${fields.payee && html`<${Field} id="op-percent" label="Percentage" type="number" min="1" max="99" step="1"
            hint="The share of the payee in each release, from 1 to 99." value=${fields.percent} onInput=${(e) => set({ percent: e.target.value })} />`}
          ${check('credit', 'As a credit', person.usual_role || undefined)}
          ${html`<${FeedCheckRow} title="Support link" checked=${fields.supportLink !== ''}
            onChange=${() => set({ supportLink: fields.supportLink === '' ? (supportChoices(person)[0] || [''])[0] : '' })} />`}
          ${fields.supportLink !== '' && html`<${ChoiceSelect} id="op-support" label="Link" value=${fields.supportLink}
            options=${supportChoices(person)} onChange=${(v) => set({ supportLink: v })} />`}
        </div>
        ${line && html`<div class="group rows"><p class="hint" role="status">${line}</p></div>`}
        <${ErrorNotice} error=${previewError} />`}
      ${kind === 'use_address' && html`<div class="group rows"><${ChoiceSelect} id="op-address" label="Address" value=${fields.key}
        options=${addressOptions(groups)} hint=${useHint(person.name)} onChange=${(key) => set({ key })} /></div>`}
      ${kind === 'rename' && html`<div class="group rows"><${Field} id="op-name" label="Name" value=${fields.name}
          onInput=${(e) => set({ name: e.target.value })} /></div>
        <div class="group rows">${check('payment', 'Payment name')}${check('credit', 'Credit name')}</div>`}
      ${kind === 'remove' && html`<p class="hint">${removeHint(person.name)}</p>
        <div class="group rows">${check('payee', 'Payee')}${check('credit', 'Credit')}</div>`}
      <${FeedChecks} rows=${rows} setRows=${setRows} empty=${adding ? 'This artist or label has no release.' : 'No feed uses this person.'} />
      ${kind === 'rename' && html`<p class="hint">${renameHint}</p>`}
      <${ErrorNotice} error=${error} />
    </div>
    <footer>
      <button class="button primary" type="button" disabled=${busy || !ready} onClick=${submit}>${busy ? BUSY_REVIEW : reviewLabel(ids.length)}</button>
    </footer>`;
}

// Load a person, with the list of wallets that gives the addresses of each
// wallet. `reload` reads both again, with no loss of the error of an action.
function usePerson(id) {
  const [data, setData] = useState(null);
  const { error, run } = useAction();
  const read = async () => {
    const [person, recipients] = await Promise.all([api(`/people/${id}`), api('/recipients')]);
    setData({ person, recipients });
  };
  useEffect(() => { run(read); }, [id]);
  return { data, error, run, reload: () => read().catch(() => {}) };
}

// The page of a person (ADR 0099 item 4). The sections come from
// `pageSequence('person')`.
export function PersonPage({ id, feeds, go }) {
  const { data, error, run, reload } = usePerson(id);
  const [suggestions, setSuggestions] = useState([]);
  const [sheet, setSheet] = useState(null);
  const [linking, setLinking] = useState(null);
  const loadSuggestions = () => api(`/people/${id}/profile-addresses`).then((r) => setSuggestions(r.suggestions || []), () => setSuggestions([]));
  useEffect(() => { loadSuggestions(); }, [id]);
  // Each change of a link reads the person again (task 065 item 5).
  const act = (work) => run(async () => { try { await work(); } finally { await reload(); } });
  if (!data) {
    return html`<div><${Bar}><span class="bar-title">Person</span><//><section class="section">
      <${ErrorNotice} error=${error} />${!error && html`<p class="empty" role="status">Loading the person…</p>`}</section></div>`;
  }
  const { person, recipients } = data;
  const titles = new Map((feeds || []).map((f) => [f.id, f.title || 'Untitled feed']));
  const groups = walletGroups(person, recipients);
  const profile = profileLine(person, titles);
  const where = whereRow(person);
  const remove = (kind, linkId) => act(() => api(`/people/${id}/${kind}/${linkId}`, { method: 'DELETE' }));
  const removeButton = (kind, linkId, label) => html`<button class="button" type="button" aria-label=${label}
    onClick=${() => remove(kind, linkId)}>Remove</button>`;
  const open = (kind) => setSheet(kind);
  const sections = {
    profile: () => html`<header class="person-head"><${PersonArt} person=${person} large />
      <div class="text"><h1>${person.name}</h1>
        ${(profile.role || profile.members.length > 0) && html`<p class="hint">${profile.role}${profile.role && profile.members.length > 0 && ' · '}${profile.members.map((m, i) =>
          html`${i > 0 && ', '}Member of <button class="button link" type="button" onClick=${() => go({ name: 'feed', id: m.id })}>${m.title}</button>`)}</p>`}
        ${person.keys.map((k) => html`<p class="hint" key=${k.id}>Nostr <span class="mono">${shortKey(k.npub)}</span> · ${keyEvidenceText(k.evidence)}</p>`)}
      </div></header>`,
    actions: () => html`<div class="actions recipient-actions">
      <button class="button action" type="button" onClick=${() => open('use_address')}>Use This Address</button>
      <button class="button action" type="button" onClick=${() => open('rename')}>Rename</button>
      <button class="button action" type="button" onClick=${() => open('add_to_releases')}>Add to All Releases</button>
      <button class="button destructive" type="button" onClick=${() => open('remove')}>Remove</button></div>`,
    where: () => html`<div class="group"><${ListRow} title=${where.title} lines=${where.lines} count=${where.count}
      onClick=${() => go({ name: 'where-used', id: person.id })} /></div>`,
    payment: () => html`<section class="page-section"><h2>Payment</h2>
      ${groups.length > 0 && html`<div class="group rows"><${ChoiceSelect} id="person-default" label="Default address"
        value=${person.default_address ? keyValue(person.default_address) : ''} options=${addressOptions(groups)}
        hint="The picker adds this address for this person."
        onChange=${(value) => act(() => api(`/people/${id}/default-address`, { method: 'PUT', body: value === '' ? null : keyFromValue(value) }))} /></div>`}
      ${groups.length === 0 && html`<p class="empty">No wallet yet. Add a wallet to give this person a payment address.</p>`}
      ${groups.map((g) => html`<div class="group" key=${g.id}>${g.members.map((m) => html`<${AddressRow} key=${JSON.stringify(m.key)}
          member=${m.member} count=${usedText(m.count)} />`)}
        <div class="actions"><button class="button" type="button" onClick=${() => remove('wallets', g.id)}>Remove Wallet</button></div></div>`)}
      ${suggestions.map((s) => html`<div class="group" key=${s.address}><${AddressRow} member=${{ address: s.address, recipient_type: 'lnaddress' }}
        note="Suggested from the Nostr profile · keysend checked"
        action=${html`<button class="button action" type="button" onClick=${() => act(async () => {
          await api(`/people/${id}/profile-addresses`, { method: 'POST', body: { address: s.address } });
          await loadSuggestions();
        })}>Add to Wallet</button>`} /></div>`)}
      <div class="group"><${AddRow} title="Add a Wallet" onClick=${() => setLinking('wallets')} /></div></section>`,
    credits: () => html`<section class="page-section"><h2>Credits</h2>
      <${PersonFields} person=${person} onSave=${(body) => act(() => api(`/people/${id}`, { method: 'PATCH', body }))} />
      ${(person.credit_names.length > 0 || person.keys.length > 0) && html`<div class="group">
        ${person.credit_names.map((c) => html`<div class="link-row static" key=${`c${c.id}`}><span class="text"><span class="label">Credit name</span>
          ${c.name && html`<span class="summary">${c.name}</span>`}${c.href && html`<span class="summary">${c.href}</span>`}${c.npub && html`<span class="summary mono">${shortKey(c.npub)}</span>`}</span>
          ${removeButton('credit-names', c.id, `Remove the credit name ${c.name}`)}</div>`)}
        ${person.keys.map((k) => html`<div class="link-row static" key=${`k${k.id}`}><span class="text"><span class="label">Nostr key</span>
          <span class="summary mono">${shortKey(k.npub)}</span><span class="summary">${keyEvidenceText(k.evidence)}</span></span>
          ${removeButton('keys', k.id, `Remove the Nostr key ${shortKey(k.npub)}`)}</div>`)}</div>`}
      <div class="group"><${AddRow} title="Add a Credit Name" onClick=${() => setLinking('credit-names')} />
        <${AddRow} title="Add a Nostr Key" onClick=${() => setLinking('keys')} /></div></section>`,
    support: () => html`<section class="page-section"><h2>Support links</h2>
      <div class="group">${person.support_links.map((l) => html`<div class="link-row static" key=${l.id}><span class="text">
          <span class="label">${l.label || 'Support link'}</span><span class="summary address-text">${l.url}</span></span>
          ${removeButton('support-links', l.id, `Remove the support link ${l.label || l.url}`)}</div>`)}
        <${AddRow} title="Add a Support Link" onClick=${() => setLinking('support-links')} /></div></section>`,
  };
  return html`<div><${Bar}><span class="bar-title">${person.name}</span><//><section class="section">
    <${ErrorNotice} error=${error} />
    ${pageSequence('person').map((section) => html`<${Fragment} key=${section.id}>${sections[section.id]()}<//>`)}
  </section>
  <${OperationSheet} kind=${sheet} person=${person} groups=${groups} rows=${feedRows(person.places, feeds)} feeds=${feeds}
    go=${go} onClose=${() => setSheet(null)} />
  <${LinkSheet} kind=${linking} personId=${person.id} onClose=${() => setLinking(null)}
    onAdded=${() => { reload(); loadSuggestions(); }} /></div>`;
}

// The page "Where used" (ADR 0099 item 5): each feed of the person, with no
// check box. A row opens the feed in the editor.
export function WhereUsedPage({ id, feeds, go }) {
  const [person, setPerson] = useState(null);
  const { error, run } = useAction();
  useEffect(() => { run(async () => setPerson(await api(`/people/${id}`))); }, [id]);
  const rows = person ? feedRows(person.places, feeds) : [];
  return html`<div><${Bar}><span class="bar-title">Where used</span><//><section class="section">
    <${ErrorNotice} error=${error} />
    ${person === null && !error && html`<p class="empty" role="status">Loading the feeds…</p>`}
    ${person && html`<header class="recipient-head"><h1>Where used</h1>
        <p class="hint">${person.name} · ${rows.length === 1 ? '1 feed' : `${rows.length} feeds`}</p></header>
      ${rows.length === 0 && html`<p class="empty">No feed uses this person.</p>`}
      ${rows.length > 0 && html`<div class="group">${rows.map((row) => html`<${ListRow} key=${row.id} title=${row.title} lines=${row.lines}
        onClick=${() => go({ name: 'feed', id: row.id })} />`)}</div>`}
      <p class="hint">${`A feed opens in the editor. To change ${person.name} in many feeds, use the buttons on the page of the person.`}</p>`}
  </section></div>`;
}

// ---- The page of a wallet with no person (ADR 0099 items 7 and 10, task 065d) ----

// The sheet of one operation of a wallet. The sheet lists the feeds of the
// wallet with a check box for each, and sends `POST /change-sets`.
function WalletSheet({ kind, recipient, uses, feeds, go, onClose }) {
  const ref = useSheet(kind !== null);
  return html`<dialog ref=${ref} class=${`sheet${kind === 'add' ? ' small' : ''}`} aria-labelledby="operation-title" onClose=${onClose}>
    <header>
      <h2 id="operation-title">${kind ? walletSheet(kind, uses).title : ''}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${() => ref.current.close()}>✕</button>
    </header>
    ${kind && html`<${WalletForm} key=${kind} kind=${kind} recipient=${recipient} uses=${uses} feeds=${feeds} go=${go} />`}
  </dialog>`;
}

function WalletForm({ kind, recipient, uses, feeds, go }) {
  const key = recipientKey(recipient);
  const adding = kind === 'add';
  const [fields, setFields] = useState(() => walletSheetStart(kind, recipient));
  const [rows, setRows] = useState(adding ? null : walletSheet(kind, uses).rows);
  const set = (patch) => setFields((current) => ({ ...current, ...patch }));
  const { busy, error, run } = useAction();
  // A wallet with no Lightning address asks the core for suggestions
  // (ADR 0097 item 5). `null` while the request runs.
  const asks = kind === 'use_lnaddress' && !lnaddressOf(recipient);
  const [field, setField] = useState(null);
  const [choice, setChoice] = useState(OTHER_ADDRESS);
  useEffect(() => {
    if (!asks) return undefined;
    let live = true;
    api('/recipients/lnaddress-suggestions', { method: 'POST', body: key })
      .then((result) => result.suggestions, () => [])
      .then((list) => {
        if (!live) return;
        const next = suggestionField(list);
        setField(next);
        setChoice(next.start);
        set({ address: next.start });
      });
    return () => { live = false; };
  }, []);
  const choose = (value) => {
    setChoice(value);
    set({ address: value });
  };
  // The core lists the releases of the artist or label. Each starts selected.
  const chooseOwner = (owner) => {
    set({ owner });
    setRows(null);
    if (owner === '') return;
    run(async () => {
      const level = await api(`/feeds/${owner}/level`);
      setRows(levelRows(level.feed_ids, feeds));
    });
  };
  const ids = rows ? checkedIds(rows) : [];
  const ready = walletSheetReady(kind, fields, ids);
  const submit = () => run(async () => {
    const made = await api('/change-sets', { method: 'POST', body: { recipient: key, operation: walletSheetBody(kind, fields), feed_ids: ids } });
    go({ name: 'change-set', id: made.id });
  });
  const typeMenu = html`<${ChoiceSelect} id="op-type" label="Type" value=${fields.type}
    onChange=${(next) => setFields(walletTypeChange(recipient, fields, next))} options=${WALLET_TYPES} />`;
  const addressFields = html`<${Field} id="op-address" label=${fields.type === 'lnaddress' ? 'Lightning address' : 'Lightning node key'} required
      value=${fields.address} onInput=${(e) => set({ address: e.target.value })} />
    ${fields.type !== 'lnaddress' && html`<${Field} id="op-custom-key" label="Custom key" hint=${CUSTOM_KEY_HINT}
        value=${fields.custom_key} onInput=${(e) => set({ custom_key: e.target.value })} />
      <${Field} id="op-custom-value" label="Account" value=${fields.custom_value} onInput=${(e) => set({ custom_value: e.target.value })} />`}`;
  return html`<div class="body">
      ${kind === 'remove' && html`<p class="hint">${removeWalletHint(recipientTitle(recipient))}</p>`}
      ${kind === 'rename' && html`<div class="group rows"><${Field} id="op-name" label="Name" value=${fields.name}
        onInput=${(e) => set({ name: e.target.value })} /></div>`}
      ${kind === 'use_lnaddress' && html`<div class="group rows">
        ${asks && field === null && html`<p class="hint" role="status">Studio looks for Lightning addresses of this wallet…</p>`}
        ${field && field.menu && html`<${ChoiceSelect} id="op-suggestion" label="Lightning address" value=${choice}
          options=${field.options} hint=${field.hints[choice]} onChange=${choose} />`}
        ${!(field && field.menu && choice !== OTHER_ADDRESS) && html`<${Field} id="op-address" label=${field && field.menu ? 'Other address' : 'Lightning address'} required
          hint=${field && field.hint ? `${field.hint} The address must pay the same wallet. Studio checks it.` : 'The address must pay the same wallet. Studio checks it.'}
          value=${fields.address} onInput=${(e) => set({ address: e.target.value })} />`}
      </div>`}
      ${kind === 'change_address' && html`<div class="group rows">${typeMenu}${addressFields}</div>`}
      ${adding && html`<div class="group rows"><${ChoiceSelect} id="op-owner" label="Artist or label" value=${fields.owner} onChange=${chooseOwner}
          placeholder="Select an artist or label" options=${memberChoices(feeds)} />
        <${Field} id="op-percent" label="Percentage" type="number" min="1" max="99" step="1" required
          hint="The share of the new recipient, from 1 to 99." value=${fields.percent} onInput=${(e) => set({ percent: e.target.value })} /></div>
        <div class="group"><details class="disclosure">
          <summary class="link-row"><span class="label">Recipient</span><span class="summary">${walletDetailsLine(fields)}</span></summary>
          <div class="form">${typeMenu}
            <${Field} id="op-name" label="Name" value=${fields.name} onInput=${(e) => set({ name: e.target.value })} />
            ${addressFields}</div>
        </details></div>`}
      <${FeedChecks} rows=${rows} setRows=${setRows} empty=${adding ? 'This artist or label has no release.' : 'No feed uses this wallet.'} />
      <${ErrorNotice} error=${error} />
    </div>
    <footer>
      <button class="button primary" type="button" disabled=${busy || !ready} onClick=${submit}>${busy ? BUSY_REVIEW : reviewLabel(ids.length)}</button>
    </footer>`;
}

// The page of a wallet with no person (ADR 0099 item 7). It has the five
// operations and no list of feeds: each sheet lists the feeds.
export function WalletPage({ recipient, feeds, go }) {
  const [uses, setUses] = useState(null);
  const [sheet, setSheet] = useState(null);
  const { error, run } = useAction();
  useEffect(() => {
    run(async () => {
      const result = await api('/recipients/uses', { method: 'POST', body: recipientKey(recipient) });
      setUses(result.feeds);
    });
  }, []);
  const used = uses !== null && uses.length > 0;
  const row = recipientRow(recipient);
  return html`<div><${Bar}><span class="bar-title">${row.title}</span><//><section class="section">
    <${ErrorNotice} error=${error} />
    <header class="recipient-head"><h1>${breakParts(row.title).map((part, i) => html`${i > 0 && html`<wbr />`}${part}`)}</h1>
      ${addressLines(recipient).map((line) => html`<p class="hint" key=${line.address || line.label}>${line.label}${line.address && html` <span class="mono">${line.address}</span>`}${line.account && ` · account ${line.account}`}</p>`)}</header>
    <div class="actions recipient-actions">${walletActions(recipient).map((action) => html`<button key=${action.kind}
      class=${`button ${action.role}`} type="button" disabled=${action.kind !== 'add' && !used}
      onClick=${() => setSheet(action.kind)}>${action.label}</button>`)}</div>
    ${uses === null && !error && html`<p class="empty" role="status">Loading the feeds…</p>`}
    ${uses !== null && uses.length === 0 && html`<p class="empty">No feed uses this wallet. Add to All Releases puts it in feeds.</p>`}
    <${WalletSheet} kind=${sheet} recipient=${recipient} uses=${uses || []} feeds=${feeds} go=${go} onClose=${() => setSheet(null)} />
  </section></div>`;
}

// The screens of the People area (ADR 0099, task 065). The components show
// the pages and call the API. The functions of `people.js` give each text
// and each state, and the core applies each rule (ADR 0042, ADR 0065).
import { html, useState, useEffect, useRef } from './vendor/htm-preact-standalone.module.js';
import { useSheet, NostrKeyRow, NostrSheet } from './editor.js';
import { PickerLayout, FindOneSheet, PickFold, TakenTag } from './pickers.js';
import { api, assetMode, askConfirm, Bar, useLeaveGuard, ChoiceSelect, CopyButton, Icon, RoleMenu, ErrorNotice, FeedCheckRow, Field, useAction, useImage } from './app.js';
import { CONFIRMS } from './confirms.js';
import { pageSequence } from './pages.js';
import { shortKey } from './nostrkey.js';
import {
  recipientKey, recipientRow, addWalletSections, walletBodyOf, PERSON_SECTION_TITLE, PERSON_SECTION_LINE, breakParts, addressLines, lnaddressOf, suggestionField, OTHER_ADDRESS, walletActions, WALLET_TYPES,
  CUSTOM_KEY_HINT, removeWalletHint, walletSheet, walletSheetStart, walletTypeChange, walletDetailsLine, walletSheetBody,
  walletSheetReady, recipientTitle,
} from './recipients.js';
import {
  peopleSections, PEOPLE_EMPTY, PEOPLE_SERVER_NOTE, suggestionRow, personRow, makePersonBody, initials,
  filterPeople, filterPayees, payeesWithNoPerson, openChanges, changeRow, memberChoices,
  toggledItem, dismissBody, addPersonBody, moreAddLabel,
  checkedIds, toggledRow, reviewLabel, addResultLine, keyValue, keyFromValue, memberLine, walletGroups, addressOptions, isDefaultAddress, DEFAULT_HINT,
  defaultFirst, walletValues, walletSelected, walletEdit, walletEditNote, creditNameEdit, supportLinkEdit, creditNameRows, keyOnlyRow, keysWithNoCreditName, creditFoldSummary, CREDIT_FOLD_HINT,
  usedText, keyEvidenceText, checkRow, feedRows, levelRows, feedsHeading, SHEET_TITLES, addIntro, useHint, renameHint,
  removeHint, sheetStart, sheetBody, sheetReady, supportChoices, BUSY_REVIEW, LINK_FORMS, linkFields, linkStart, linkBody,
  changeSetView, linkReady, profileLine, resolveBody, profileMap, withProfiles, keyRow, profileAddressNote,
  personChoices, addToTitle, ADD_TO_HINT, addToRows, addItemsBody, MORE_HINT,
  NEW_MEMBER_HINT, NEW_PERSON_HINT, checkSheetTitle, checkSheetStart, checkSheetChoices, checkSheetHint, checkSheetPerson,
  checkSheetLabel, checkSheetReady, NEW_PERSON, WALLET_TO_PERSON_HINT, walletToPersonStart, walletToPersonReady,
  walletToPersonLabel, correctCount, correctButtonLabel, correctTitle, correctFeedIds, correctStart, correctKeyStart, correctOptions,
  correctKeyChoices, correctBody, correctAnySelected, correctRows, CORRECT_OMITTED, CORRECT_EMPTY,
  moreRows, NO_OTHER_TEXT, mergeTitle, mergeHint, mergeLinks, mergeResult, mergeBody, afterMissing,
  detailList, LINK_HINT, LINK_EMPTY, linkChoices, linkRows, creditPosition, afterCredits, savedValues, pendingKeys, pendingStatus, isPending, saveBody, leaveText, checkGroups, checkFeedLines, checkKeys, checkIntro, CHANGE_ROWS, CHANGE_NOTE,
  CHECK_TITLE, checkSummary, differenceBlocks, openLabel, useValuesLabel,
  recipientChoices, creditChoices, fundingChoices, ALREADY_VALUE, FIND_HINT, FIND_EMPTY, FIND_WAIT, FILL_NOTE, lookupGroups, lookupKind, hasLine, fillRows, fillLabel, fillPatch, fillSource, musicindexFeedUrl,
  creditOffers, creditAddTitle, creditRemoveTitle, creditAddNote, creditPublicNote, creditRemoveNote, creditOperation, REMOVE_CREDIT_OPERATION,
  lookupAddress, showFindEmpty,
} from './people.js';

// The merges of this page load: the ID of each merged person, to the ID of
// the person that kept its items.
const merged = new Map();

// The round picture of a person. A person with no picture, or a picture that
// Studio cannot read, shows the initials of the name.
export function PersonArt({ person, large = false, small = false }) {
  const { src } = useImage(person.img);
  return html`<span class=${large ? 'person-art large' : small ? 'person-art small' : 'person-art'} aria-hidden="true">${src ? html`<img src=${src} alt="" />` : initials(person.name)}</span>`;
}

// The profiles of Nostr keys, from `POST /nostr/resolve`. The value is null
// while the request runs and when it fails, so that a row shows the short key
// only (task 067d item 3).
export function useProfiles(npubs) {
  const [profiles, setProfiles] = useState(null);
  const list = [...new Set(npubs.filter(Boolean))];
  const signature = list.join(',');
  useEffect(() => {
    if (list.length === 0) { setProfiles(null); return undefined; }
    let live = true;
    api('/nostr/resolve', { method: 'POST', body: resolveBody(list) })
      .then((r) => { if (live) setProfiles(profileMap(r.profiles)); }, () => { if (live) setProfiles(null); });
    return () => { live = false; };
  }, [signature]);
  return profiles;
}

// The text of a key row: the profile name or the short key, the other lines
// and the warning.
function KeyText({ row }) {
  return html`<span class="text"><span class=${row.mono ? 'label mono key-text' : 'label'}>${row.title}</span>
    ${row.lines.map((line, i) => html`<span class=${line === row.short ? 'summary key-text' : 'summary'} key=${i}>${line}</span>`)}
    ${row.warning && html`<span class="summary field-error">${row.warning}</span>`}</span>`;
}

// A row of the fold "Credited as": each value has an icon and a label (ADR 0140).
function CreditRow({ row, removeLabel, onRemove, onEdit }) {
  return html`<div class="link-row static key-line more-row credit-row">
    <${PersonArt} person=${{ img: row.picture, name: row.title }} />
    <div class="text"><span class="label">${row.title}</span>
      <dl class="attributes">${row.attributes.map((a) => html`<div key=${a.id} class="attribute">
        <dt><${Icon} name=${a.icon} />${a.label}</dt>
        <dd><span class=${a.mono ? 'mono key-text' : ''}>${a.value}</span>
          ${a.sub.map((line) => html`<span class=${line.startsWith('npub') ? 'attribute-sub mono key-text' : 'attribute-sub'} key=${line}>${line}</span>`)}
          ${a.id === 'nostr' && html`<${CopyButton} text=${row.npub} label="the Nostr key" />`}</dd></div>`)}</dl>
      ${row.warning && html`<span class="summary field-error">${row.warning}</span>`}</div>
    <div class="actions more-actions">
      ${onEdit && html`<button class="button action" type="button" aria-label=${`Edit ${row.title}`} onClick=${onEdit}>Edit</button>`}
      <button class="button destructive" type="button" aria-label=${removeLabel} onClick=${onRemove}>Remove</button></div></div>`;
}

// A Nostr key with its profile, a copy button and an optional action.
function KeyRowView({ npub, profiles, name, action, note }) {
  const row = keyRow(npub, profiles, name);
  return html`<div class="link-row static key-line"><${PersonArt} person=${{ img: row.picture, name: row.title }} />
    <${KeyText} row=${{ ...row, lines: note ? [...row.lines, note] : row.lines }} />
    <${CopyButton} text=${npub} label="the Nostr key" />${action}</div>`;
}

// One row of a list: a button with the lines of its model. A line is text, or
// `{ text, error }`. An error line shows in the error color, with the warning
// icon.
function ListRow({ title, lines, count, lead, onClick, disabled = false, chevron = true, selected = false }) {
  const line = (l, i) => (typeof l === 'string'
    ? html`<span class="summary" key=${i}>${l}</span>`
    : html`<span class=${l.error ? 'summary check-error' : 'summary'} key=${i}>${l.error && html`<${Icon} name="alert" />`}${l.text}</span>`);
  return html`<button class="link-row" type="button" disabled=${disabled} aria-current=${selected ? 'true' : undefined} onClick=${onClick}>${lead}
    <span class="text"><span class="label">${title}</span>${lines.map(line)}</span>
    ${count && html`<span class="count">${count}</span>`}${chevron && html`<span class="go">›</span>`}</button>`;
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
  const sections = data ? peopleSections({ people, suggestions: data.suggestions, changes: data.changes, payees, server: assetMode().server }) : [];
  const sectionViews = {
    note: () => html`<p class="hint">${PEOPLE_SERVER_NOTE}</p>`,
    search: () => html`<div class="field search-field"><div class="field-body">
      <input type="search" aria-label="Search people, addresses and keys" placeholder="Search people, addresses and keys"
        value=${query} onInput=${(e) => setQuery(e.target.value)} /></div></div>`,
    actions: () => html`<div class="actions">
      <button class="button action" type="button" onClick=${() => setAdding(true)}>Add a Person</button>
      <button class="button" type="button" disabled=${busy} onClick=${() => run(downloadPeople)}>Export People</button></div>`,
    changes: () => html`<section class="page-section"><h2>Changes to write</h2><div class="group">${data.changes.map((set) => {
      const row = changeRow(set, data.people, payeesWithNoPerson(data.recipients));
      return html`<${ListRow} key=${set.id} title=${row.title} lines=${[row.text]} onClick=${() => go(changeSetView(set))} />`;
    })}</div></section>`,
    // Each list folds (ADR 0146 item 10). A search opens the lists that it filters.
    suggestions: () => html`<${PickFold} id="people-suggestions" title="Suggestions from your feeds"
      note="Studio found these names in your feeds. Check each one, then make a person."
      rows=${data.suggestions.map((suggestion) => {
        const row = suggestionRow(suggestion);
        return html`<${ListRow} key=${suggestion.identity} title=${row.title} lines=${row.lines} count="Check"
          lead=${html`<${PersonArt} person=${{ name: suggestion.name, img: (suggestion.credits.find((c) => c.img) || {}).img || '' }} />`}
          onClick=${() => setMaking(suggestion)} />`;
      })} />`,
    people: () => (people.length > 0
      ? html`<${PickFold} id="people-people" title="People" startOpen=${true} forceOpen=${query.trim() !== ''} rows=${people.map((person) => {
        const row = personRow(person, titles);
        return html`<${ListRow} key=${person.id} title=${row.title} lines=${row.lines} count=${row.count}
          lead=${html`<${PersonArt} person=${person} />`} onClick=${() => go({ name: 'person', id: person.id })} />`;
      })} />`
      : html`<section class="page-section"><h2>People</h2>
        ${data.people.length === 0 && html`<p class="empty">${PEOPLE_EMPTY}</p>`}
        ${data.people.length > 0 && html`<p class="empty">No person matches the search.</p>`}</section>`),
    payees: () => html`<${PickFold} id="people-payees" title="Payees with no person" forceOpen=${query.trim() !== ''} rows=${payees.map((recipient) => {
      const row = recipientRow(recipient);
      return html`<${ListRow} key=${JSON.stringify(recipientKey(recipient))} title=${row.title} lines=${row.lines} count=${row.count}
        onClick=${() => go({ name: 'wallet', recipient })} />`;
    })} />`,
  };
  return html`<div><${Bar}><span class="bar-title">People</span><//><section class="section">
    <${ErrorNotice} error=${error} />
    ${data === null && !error && html`<p class="empty" role="status">Loading the people…</p>`}
    ${sections.map((id) => html`<${Fragment} key=${id}>${sectionViews[id]()}<//>`)}
  </section>
  <${CheckSuggestionSheet} suggestion=${making} people=${data ? data.people : []} feeds=${feeds} titles=${titles} go=${go}
    onClose=${() => setMaking(null)} onDismissed=${() => { setMaking(null); load(); }} />
  <${AddPersonSheet} open=${adding} go=${go} onClose=${() => setAdding(false)} /></div>`;
}

const Fragment = ({ children }) => children;

// The sheet "Check: {name}" (ADR 0111 items 5 to 7). The menu "Person" picks
// a new person or a person that exists. The sheet changes no feed. The core
// makes the person or adds the items, and a 409 gives its message here.
function CheckSuggestionSheet({ suggestion, people, feeds, titles, go, onClose, onDismissed }) {
  const ref = useSheet(suggestion !== null);
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="check-suggestion-title" onClose=${onClose}>
    ${suggestion && html`<${CheckSuggestionForm} key=${suggestion.identity} suggestion=${suggestion} people=${people} feeds=${feeds} titles=${titles}
      go=${go} onCancel=${() => ref.current.close()} onDismissed=${onDismissed} />`}
  </dialog>`;
}

function CheckSuggestionForm({ suggestion, people, feeds, titles, go, onCancel, onDismissed }) {
  const [state, setState] = useState(() => checkSheetStart(suggestion));
  const profiles = useProfiles(state.keys.map((r) => r.item));
  useEffect(() => { if (profiles) setState((current) => withProfiles(current, profiles)); }, [profiles]);
  const { busy, error, run } = useAction();
  const person = checkSheetPerson(state, people);
  const groups = addToRows(state, person, titles, profiles);
  const submit = () => run(async () => {
    if (person) {
      await api(`/people/${person.id}/items`, { method: 'POST', body: addItemsBody(state, person) });
      go({ name: 'person', id: person.id });
    } else {
      const made = await api('/people/suggestions/make', { method: 'POST', body: makePersonBody(state) });
      go({ name: 'person', id: made.id });
    }
  });
  const dismiss = () => run(async () => {
    await api('/people/suggestions/dismiss', { method: 'POST', body: dismissBody(suggestion) });
    onDismissed();
  });
  const rowView = (row) => {
    // A row that the person has already keeps its content and has a tag (ADR 0146 item 3).
    const lines = row.already ? row.lines.slice(0, -1) : row.lines;
    const text = html`<span class="text"><span class="pick-name"><span class=${row.mono ? 'label mono key-text' : 'label'}>${row.title}</span>
        ${row.already && html`<${TakenTag} text=${row.lines.at(-1)} />`}</span>
      ${lines.map((line, i) => html`<span class=${line === row.short ? 'summary key-text' : 'summary'} key=${i}>${line}</span>`)}
      ${row.warning && html`<span class="summary field-error">${row.warning}</span>`}</span>`;
    const lead = row.group === 'keys' ? html`<${PersonArt} person=${{ img: row.picture, name: row.title }} />` : null;
    const copy = row.group === 'keys' ? html`<${CopyButton} text=${state.keys[row.index].item} label="the Nostr key" />` : null;
    const key = `${row.group}${row.index}`;
    if (row.already) return html`<div class=${copy ? 'key-row' : ''} key=${key}><div class="link-row static pick-taken">${lead}${text}</div>${copy}</div>`;
    return html`<div class=${copy ? 'key-row' : ''} key=${key}><label class="link-row use-row">
      <input type="checkbox" checked=${row.checked} onChange=${() => setState(toggledItem(state, row.group, row.index))} />${lead}${text}</label>${copy}</div>`;
  };
  return html`<header>
      <h2 id="check-suggestion-title">${checkSheetTitle(suggestion.name)}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${onCancel}>✕</button>
    </header>
    <div class="body">
      <div class="group rows">
        <${ChoiceSelect} id="check-person" label="Person" value=${state.personId} options=${checkSheetChoices(people)}
          hint=${checkSheetHint(suggestion, state.personId)} onChange=${(personId) => setState({ ...state, personId })} />
        ${!person && html`<${Field} id="check-name" label="Name" required value=${state.name}
          onInput=${(e) => setState({ ...state, name: e.target.value })} />
          <${ChoiceSelect} id="check-member" label="Link to an artist or label page" value=${state.memberOf} placeholder="None"
            hint=${NEW_MEMBER_HINT} options=${linkChoices(feeds)} onChange=${(memberOf) => setState({ ...state, memberOf })} />`}
      </div>
      ${groups.map((g) => html`<section class="page-section" key=${g.group}><h2>${g.title}</h2>
        <div class="group rows">${g.rows.map(rowView)}</div></section>`)}
      <p class="hint">${person ? ADD_TO_HINT : NEW_PERSON_HINT}</p>
      <${ErrorNotice} error=${error} />
    </div>
    <footer>
      <button class="button" type="button" disabled=${busy} onClick=${dismiss}>Not This Person</button>
      <button class="button primary" type="button" disabled=${busy || !checkSheetReady(state, person)} onClick=${submit}>${busy ? (person ? 'Adding the Items…' : 'Making the Person…') : checkSheetLabel(state, person)}</button>
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
function AddressRow({ member, count, note, tag, mark, pending, action }) {
  const { label, address, account } = memberLine(member, true);
  return html`<div class=${`link-row static address-row${pending ? ' pending' : ''}`}><span class="text"><span class="label">${label}${tag}</span>
      ${address && html`<span class="summary mono address-text">${address}</span>`}
      ${account && html`<span class="summary">Account ${account}</span>`}
      ${note && html`<span class="summary">${note}</span>`}</span>
    ${count && html`<span class="count">${count}</span>`}${mark}
    ${member.address && html`<${CopyButton} text=${member.address} label="the address" />`}${action}</div>`;
}

// A row that adds something: a plus sign and a title.
const AddRow = ({ title, onClick }) => html`<button class="link-row add" type="button" onClick=${onClick}>
  <span class="num">＋</span><span class="title">${title}</span></button>`;

// The fields of a person: the name, the usual role, and the picture and the
// information address. The values are pending changes of the page (ADR 0116
// item 1). `update` changes them, and the page saves them. A pending control
// has a mark. "Use" of a suggestion changes the value only (ADR 0113 item 8).
function PersonFields({ values, saved, update, detail, profiles, titles, feeds, sources = {} }) {
  // A filled field shows the line of its source until Save or Discard (ADR 0117 item 14).
  const field = (id, label, key) => html`<${Field} id=${id} label=${label} value=${values[key]} hint=${sources[key]}
    class=${isPending(saved, values, key) ? 'pending' : undefined}
    onInput=${(e) => update({ [key]: e.target.value })} />`;
  // The suggestions of the core for one field.
  const suggestions = (key) => {
    const list = detailList(key, values[key], detail, profiles, titles, feeds);
    if (list.state === 'none') return null;
    const rows = list.rows.map((row) => html`<div class="link-row static" key=${row.value}>
      ${key === 'img' && html`<${PersonArt} person=${{ img: row.value, name: '' }} />`}
      <span class="text"><span class="summary mono address-text">${row.value}</span><span class="summary">${row.line}</span></span>
      <button class="button action" type="button" aria-label=${`Use ${row.value}`}
        onClick=${() => update({ [key]: row.value })}>Use</button></div>`);
    // A collapsed list has no heading: its summary line names the values
    // (ADR 0113 item 6).
    if (list.state === 'collapsed') {
      return html`<div class="group suggest-gap"><details class="disclosure">
        <summary class="link-row"><span class="summary">${list.collapsedLine}</span></summary>
        <div class="form">${rows}</div></details></div>`;
    }
    return html`<p class="hint suggest-heading">${list.heading}</p><div class="group">${rows}</div>`;
  };
  return html`<div class="group rows">
    ${field('person-name', 'Name', 'name')}
    <${RoleMenu} id="person-role" label="Usual role" role=${values.usual_role} group=${values.usual_group} hint="The start value of a new credit."
      pending=${isPending(saved, values, 'usual_role')}
      onChange=${(role, group) => update({ usual_role: role, usual_group: group })} />
    ${field('person-img', 'Picture address', 'img')}</div>
    ${suggestions('img')}
    <div class="group rows suggest-gap">${field('person-href', 'Information address', 'href')}</div>
    ${suggestions('href')}`;
}

// The row of musicindex of each link form (ADR 0145 items 3 and 14).
const LINK_FINDS = {
  wallets: { choices: recipientChoices, empty: 'This feed has no payment address.', summary: 'Fill the form from a feed.' },
  'credit-names': { choices: creditChoices, empty: 'Music Index has no values of this feed.', summary: 'Fill the form from a feed.' },
  'support-links': { choices: fundingChoices, empty: 'This feed has no website.', summary: 'Fill the address from the website of a feed.' },
};

// The sheet that adds a link to a person (ADR 0099 item 4).
function LinkSheet({ kind, personId, edit = null, onClose, onAdded }) {
  const ref = useSheet(kind !== null);
  return html`<dialog ref=${ref} class=${kind === 'wallets' ? 'sheet' : 'sheet small'} aria-labelledby="link-title" onClose=${onClose}>
    ${kind && html`<${LinkForm} key=${edit ? `edit-${edit.walletId ?? edit.linkId}-${edit.start.address ?? ''}` : kind} kind=${kind} personId=${personId} edit=${edit} onCancel=${() => ref.current.close()}
      onAdded=${() => { ref.current.close(); onAdded(); }} />`}
  </dialog>`;
}

function LinkForm({ kind, personId, edit = null, onCancel, onAdded }) {
  const form = LINK_FORMS[kind];
  const find = LINK_FINDS[kind];
  const [values, setValues] = useState(() => (edit ? edit.start : linkStart(kind)));
  // The payees with no person of the sheet "Add a Wallet" (ADR 0141). `null`
  // while the list loads.
  const [recipients, setRecipients] = useState(kind === 'wallets' ? null : []);
  const { busy, error, run } = useAction();
  useEffect(() => {
    if (kind !== 'wallets') return;
    api('/recipients').then(setRecipients, () => setRecipients([]));
  }, []);
  // A functional update keeps the value of each field that changed last.
  const set = (id, value) => setValues((current) => ({ ...current, [id]: value }));
  // The sheet "Find a Nostr profile" of a form with a Nostr key (ADR 0123).
  const [keyOpen, setKeyOpen] = useState(false);
  const [finding, setFinding] = useState(false);
  const add = (e) => {
    e.preventDefault();
    if (!linkReady(kind, values)) return;
    run(async () => {
      await api(edit ? `/people/${personId}/${kind}/${edit.walletId ?? edit.linkId}` : `/people/${personId}/${kind}`,
        { method: edit ? 'PUT' : 'POST', body: linkBody(kind, values) });
      onAdded();
    });
  };
  const fields = html`<div class="group rows">${linkFields(kind, values).map((f) => (f.select
    ? html`<${ChoiceSelect} key=${f.id} id=${`link-${f.id}`} label=${f.label} value=${values[f.id]} options=${f.select}
        onChange=${(v) => set(f.id, v)} />`
    : html`<${Field} key=${f.id} id=${`link-${f.id}`} label=${f.label} hint=${f.hint} required=${!!f.required}
        value=${values[f.id]} onInput=${(e) => set(f.id, e.target.value)} />`))}
    ${form.nostrKey && html`<${NostrKeyRow} label="Nostr key" name=${values.name} value=${values.npub}
      onOpen=${() => setKeyOpen(true)} />`}</div>
    ${edit && (edit.note || walletEditNote(edit.others)) && html`<p class="hint" role="status">${edit.note || walletEditNote(edit.others)}</p>`}`;
  const payees = addWalletSections(recipients || []).filter((section) => section.id === 'payees')
    .flatMap((section) => section.rows).map((row) => html`<${ListRow} key=${JSON.stringify(row.body)} title=${row.title} lines=${row.lines} count=${row.count}
      chevron=${false} selected=${walletSelected(values, row.body)} onClick=${() => setValues((current) => ({ ...current, ...walletValues(row.body) }))} />`);
  return html`<header>
      <h2 id="link-title">${edit ? form.editTitle : form.title}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${onCancel}>✕</button>
    </header>
    <${PickerLayout} id="link-form" onSubmit=${add} fields=${fields} notice=${html`<${ErrorNotice} error=${error} />`}
      musicindex=${{ title: 'Find on Music Index', summary: find.summary, onOpen: () => setFinding(true) }}
      lists=${[{ title: 'Payees with no person', rows: payees }]} />
    <${FindOneSheet} open=${finding} start=${values.name} choices=${find.choices} credits=${kind === 'credit-names'} empty=${find.empty}
      onPick=${(found) => setValues((current) => ({ ...current, ...(kind === 'wallets' ? walletValues(found) : found) }))} onClose=${() => setFinding(false)} />
    ${form.nostrKey && html`<${NostrSheet} open=${keyOpen} value=${values.npub} offerPeople=${false} onSet=${(npub) => set('npub', npub)}
      onClose=${() => setKeyOpen(false)} />`}
    <footer><button class="button primary" type="submit" form="link-form" disabled=${busy || !linkReady(kind, values)}>${edit ? (busy ? 'Saving…' : form.editButton) : (busy ? 'Adding…' : form.button)}</button></footer>`;
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
// With `separate`, each line of a row shows on its own line.
function FeedChecks({ rows, setRows, empty, separate = false }) {
  if (!rows) return null;
  if (rows.length === 0) return html`<p class="empty">${empty}</p>`;
  return html`<section class="page-section"><h2>${feedsHeading(rows)}</h2><div class="group rows">${rows.map((row) =>
    html`<${FeedCheckRow} key=${row.id} title=${row.title} text=${separate ? undefined : row.lines.join(' · ')} lines=${separate ? row.lines : undefined} checked=${row.checked}
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

// The sheet "Find on Music Index: {name}" (ADR 0117 items 2 to 4). It is open
// while `query` is not null. Its search runs when it opens, and 300 ms after
// each change of the text.
function FindSheet({ open, personId, name, query, onQuery, onClose, onPick }) {
  const ref = useSheet(open);
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="find-title" onClose=${onClose}>
    ${open && html`<${FindForm} personId=${personId} name=${name} query=${query} onQuery=${onQuery}
      onCancel=${() => ref.current.close()} onPick=${onPick} />`}
  </dialog>`;
}

// The rows of a search. It runs at the start, and 300 ms after each change of
// the text. `rows` is null while the first search runs.
export function useLookup(query, path) {
  const [rows, setRows] = useState(null);
  const [feeds, setFeeds] = useState([]);
  const [note, setNote] = useState(null);
  const [searched, setSearched] = useState(false);
  const [error, setError] = useState(null);
  const first = useRef(true);
  // A start text is plain words until the person edits the field (ADR 0151
  // item 3).
  const startText = useRef(query);
  const edited = useRef(false);
  if (query !== startText.current) edited.current = true;
  useEffect(() => {
    const text = query.trim();
    const wait = first.current ? 0 : 300;
    first.current = false;
    if (text === '') { setRows([]); setFeeds([]); setNote(null); setSearched(false); setError(null); return undefined; }
    let live = true;
    const load = () => api(lookupAddress(path, text, !edited.current)).then(
      (result) => { if (live) { setRows(result.rows || []); setFeeds(result.feeds || []); setNote(result.note || null); setSearched(result.searched === true); setError(null); } },
      (e) => { if (live) { setRows([]); setFeeds([]); setNote(null); setSearched(false); setError(e); } },
    );
    const timer = setTimeout(load, wait);
    return () => { live = false; clearTimeout(timer); };
  }, [query]);
  return { rows, feeds, note, searched, error };
}

function FindForm({ personId, name, query, onQuery, onCancel, onPick }) {
  const { rows, note, searched, error } = useLookup(query, `/people/${personId}/musicindex`);
  return html`<header><h2 id="find-title">${`Find on Music Index: ${name}`}</h2></header>
    <div class="body">
      <div class="group rows"><${Field} id="find-query" label="Name" type="search" value=${query}
        onInput=${(e) => onQuery(e.target.value)} /></div>
      <p class="hint">${FIND_HINT}</p>
      <${ErrorNotice} error=${error} />
      ${rows === null && html`<p class="empty" role="status">${FIND_WAIT}</p>`}
      ${note && html`<p class="hint" role="status">${note.message}</p>`}
      ${showFindEmpty({ rows, error, searched }) && html`<p class="empty">${FIND_EMPTY}</p>`}
      ${rows !== null && lookupGroups(rows).map((group) => html`<section class="page-section" key=${group.level}>
        <h2>${group.heading}</h2><p class="hint">${group.note}</p>
        <div class="group">${group.rows.map((row) => html`<${LookupRow} key=${row.feed_guid} row=${row} onClick=${() => onPick(row)} />`)}</div>
      </section>`)}
    </div>
    <footer><button class="button" type="button" onClick=${onCancel}>Cancel</button></footer>`;
}

// A row of a search: the picture, the title, the type line with its dot, and
// the line of the values (ADR 0117 items 4 and 7).
export function LookupRow({ row, onClick }) {
  const kind = lookupKind(row);
  const detail = hasLine(row);
  return html`<button class="link-row" type="button" onClick=${onClick}>
    <${PersonArt} person=${{ img: row.image_url, name: row.title || '' }} />
    <span class="text"><span class="label">${row.title || 'Untitled feed'}</span>
      <span class="summary"><span class=${`dot ${kind.dot}`} aria-hidden="true"></span>${kind.text}</span>
      ${detail && html`<span class="summary">${detail}</span>`}</span>
    <span class="go">›</span></button>`;
}

// The sheet "From Music Index: {title}" (ADR 0117 items 8 to 14). It is open
// while `row` is not null.
function ValuesSheet({ personId, row, shown, onBack, onClose, onFill }) {
  const ref = useSheet(row !== null);
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="values-title" onClose=${onClose}>
    ${row && html`<${ValuesForm} key=${row.feed_guid} personId=${personId} row=${row} shown=${shown} onBack=${onBack} onFill=${onFill} />`}
  </dialog>`;
}

function ValuesForm({ personId, row, shown, onBack, onFill }) {
  const [values, setValues] = useState(null);
  const [checked, setChecked] = useState([]);
  const [error, setError] = useState(null);
  useEffect(() => {
    let live = true;
    api(`/people/${personId}/musicindex/${encodeURIComponent(row.feed_guid)}`).then((result) => {
      if (!live) return;
      setValues(result);
      setChecked(fillRows(result, shown).filter((r) => r.selected).map((r) => r.id));
    }, (e) => { if (live) setError(e); });
    return () => { live = false; };
  }, [row.feed_guid]);
  const rows = values ? fillRows(values, shown) : [];
  const count = rows.filter((r) => r.selectable && checked.includes(r.id)).length;
  const toggle = (id) => setChecked((current) => (current.includes(id) ? current.filter((c) => c !== id) : [...current, id]));
  const kind = lookupKind(row);
  const title = row.title || 'Untitled feed';
  return html`<header><h2 id="values-title">${`From Music Index: ${title}`}</h2></header>
    <div class="body">
      <div class="group"><div class="link-row static"><${PersonArt} person=${{ img: row.image_url, name: title }} />
        <span class="text"><span class="label">${title}</span>
          <span class="summary"><span class=${`dot ${kind.dot}`} aria-hidden="true"></span>${kind.text}</span>
          <span class="summary"><a href=${musicindexFeedUrl(row.feed_guid)} target="_blank" rel="noopener">Open on musicindex.org</a></span></span></div></div>
      <${ErrorNotice} error=${error} />
      ${values && html`<section class="page-section"><h2>Fill</h2>
        <div class="group rows">${rows.map((r) => html`<${FillRow} key=${r.id} row=${r} checked=${checked.includes(r.id)}
          onToggle=${() => toggle(r.id)} />`)}</div>
        <p class="hint">${FILL_NOTE}</p></section>`}
    </div>
    <footer><button class="button" type="button" onClick=${onBack}>Back</button>
      <button class="button primary" type="button" disabled=${count === 0} onClick=${() => onFill(fillPatch(rows, checked, shown), fillSource(row))}>${fillLabel(count)}</button></footer>`;
}

// A row of the Fill section. A row that cannot be selected has no check box.
function FillRow({ row, checked, onToggle }) {
  const lead = row.art ? html`<${PersonArt} person=${{ img: row.art, name: row.title }} />` : null;
  // A value that the person has already keeps its content and has a tag (ADR 0146 item 3).
  const taken = row.lines.includes(ALREADY_VALUE);
  const text = html`<span class="text"><span class="pick-name"><span class="label">${row.title}</span>${taken && html`<${TakenTag} text=${ALREADY_VALUE} />`}</span>
    ${row.value && html`<span class="summary mono address-text">${row.value}</span>`}
    ${row.lines.filter((line) => line !== ALREADY_VALUE).map((line) => html`<span class="summary" key=${line}>${line}</span>`)}</span>`;
  if (!row.selectable) return html`<div class=${taken ? 'link-row static pick-taken' : 'link-row static'}>${lead}${text}</div>`;
  return html`<label class="link-row use-row"><input type="checkbox" checked=${checked} onChange=${onToggle} />${lead}${text}</label>`;
}

// The page of a person (ADR 0099 item 4). The sections come from
// `pageSequence('person')`.
export function PersonPage({ id, feeds, go }) {
  const { data, error, run, reload } = usePerson(id);
  const [suggestions, setSuggestions] = useState([]);
  const [sheet, setSheet] = useState(null);
  const [linking, setLinking] = useState(null);
  // The value of an edit sheet, from `walletEdit`, `creditNameEdit` or
  // `supportLinkEdit` (ADR 0141, ADR 0146).
  const [editing, setEditing] = useState(null);
  const [more, setMore] = useState([]);
  const [merging, setMerging] = useState(false);
  // The search of "Find on musicindex" (its text, or null when closed), and the
  // row that the values sheet shows (ADR 0117 items 2 and 8).
  const [finding, setFinding] = useState(null);
  const [picked, setPicked] = useState(null);
  // The credit offers of the last Save that changed the links (ADR 0132 items
  // 11 and 12). The first offer has its sheet open. `made` holds the ID of each
  // change set of these sheets.
  const [offers, setOffers] = useState([]);
  const [offerTotal, setOfferTotal] = useState(0);
  const made = useRef([]);
  const loadMore = () => api(`/people/${id}/more`).then((r) => setMore(r.items || []), () => setMore([]));
  // "Add a Nostr Key" adds the key that the sheet "Find a Nostr profile" picks (ADR 0123).
  const addKey = (npub) => act(async () => {
    await api(`/people/${id}/keys`, { method: 'POST', body: { npub } });
    loadSuggestions();
    loadMore();
  });
  useEffect(() => { loadMore(); }, [id]);
  const loadSuggestions = () => api(`/people/${id}/profile-addresses`).then((r) => setSuggestions(r.suggestions || []), () => setSuggestions([]));
  useEffect(() => { loadSuggestions(); }, [id]);
  // The suggestions of the picture and the information address (ADR 0113).
  // They load after the person and again after each save.
  const [detail, setDetail] = useState(null);
  useEffect(() => {
    if (!data) return undefined;
    let live = true;
    api(`/people/${id}/detail-suggestions`).then((r) => { if (live) setDetail(r); }, () => { if (live) setDetail(null); });
    return () => { live = false; };
  }, [data]);
  const known = data ? data.person : null;
  const profiles = useProfiles([
    ...(known ? known.keys.map((k) => k.npub) : []),
    ...(known ? known.credit_names.map((c) => c.npub) : []),
    ...suggestions.map((s) => s.npub),
    ...more.map((m) => m.npub),
    ...((detail && [...detail.img, ...detail.href]) || []).flatMap((d) => d.sources.map((x) => x.npub)),
  ]);
  // Each change of a link reads the person again (task 065 item 5).
  const act = (work) => run(async () => { try { await work(); } finally { await reload(); } });
  // The pending changes of the fields, the default address and the links
  // (ADR 0116 items 1 to 6). The saved values come from each read of the person.
  const [values, setValues] = useState(null);
  const [saving, setSaving] = useState(false);
  const [saveFailure, setSaveFailure] = useState(null);
  // The line of each filled field. Save, Discard and a new read clear it.
  const [sources, setSources] = useState({});
  useEffect(() => { if (data) { setValues(savedValues(data.person)); setSaveFailure(null); setSources({}); } }, [data]);
  const saved = data ? savedValues(data.person) : null;
  const shown = values || saved;
  const keys = saved ? pendingKeys(saved, shown) : [];
  const dirty = keys.length > 0;
  const update = (patch) => { setSaveFailure(null); setValues((current) => ({ ...(current || saved), ...patch })); };
  const discard = () => { setValues(saved); setSaveFailure(null); setSources({}); };
  // A fill makes pending changes, and closes the values sheet (ADR 0117 item 14).
  const fill = (patch, source) => {
    update(patch);
    setSources((current) => ({
      ...current,
      ...(patch.img !== undefined ? { img: source } : {}),
      ...(patch.href !== undefined ? { href: source } : {}),
    }));
    setPicked(null);
  };
  // Save sends each pending change in one request (ADR 0116 item 5). A failure
  // keeps the pending changes, and shows the error.
  const save = async () => {
    if (!dirty) return;
    const before = saved.members;
    const after = shown.members;
    const linksChanged = keys.includes('member');
    setSaving(true);
    setSaveFailure(null);
    let saveDone = false;
    try {
      await api(`/people/${id}/save`, { method: 'POST', body: saveBody(saved, shown) });
      saveDone = true;
      await reload();
    } catch (e) {
      setSaveFailure((e && e.message) || 'Studio could not save this person');
    } finally {
      setSaving(false);
    }
    if (saveDone && linksChanged) await offerCredits(before, after);
  };
  // The offers of a credit for changed links (ADR 0118 item 3, ADR 0132 item
  // 11). A failed read of the members gives no offer for that feed.
  const offerCredits = async (before, after) => {
    const feedIds = [...new Set([...before, ...after])];
    const lists = await Promise.all(feedIds.map((feedId) => api(`/feeds/${feedId}/members`).then((r) => r.members || [], () => [])));
    const members = new Map(feedIds.map((feedId, i) => [feedId, lists[i]]));
    const list = creditOffers(before, after, feeds, members, Number(id));
    made.current = [];
    setOfferTotal(list.length);
    setOffers(list);
  };
  // The next sheet. After the last sheet, the view of ADR 0132 item 12 opens.
  const nextOffer = () => {
    const rest = offers.slice(1);
    setOffers(rest);
    if (rest.length > 0) return;
    const view = afterCredits(made.current);
    made.current = [];
    if (view) go(view);
  };
  // Asks before the page leaves with a pending change (ADR 0116 items 8 and 11).
  // "Discard Changes" puts back the saved values, and gives true.
  const askLeave = async () => {
    if (!dirty) return true;
    const leave = await askConfirm(CONFIRMS.leavePerson(leaveText(data.person.name, keys)));
    if (leave) setValues(saved);
    return leave;
  };
  useLeaveGuard(dirty, askLeave);
  // An action that would lose the pending changes waits for the answer (ADR 0116 item 13).
  const withLeave = (work) => askLeave().then((leave) => { if (leave) work(); });
  // The browser asks before it closes the page with a pending change (ADR 0116 item 10).
  useEffect(() => {
    if (!dirty) return undefined;
    const warn = (event) => { event.preventDefault(); event.returnValue = ''; };
    addEventListener('beforeunload', warn);
    return () => removeEventListener('beforeunload', warn);
  }, [dirty]);
  // Back can open a person that a merge or a delete removed.
  const missing = !data && error && error.status === 404;
  useEffect(() => { if (missing) go(afterMissing(id, merged), { replace: true }); }, [missing]);
  if (!data) {
    return html`<div><${Bar}><span class="bar-title">Person</span><//><section class="section">
      <${ErrorNotice} error=${error} />${!error && html`<p class="empty" role="status">Loading the person…</p>`}</section></div>`;
  }
  const { person, recipients } = data;
  const titles = new Map((feeds || []).map((f) => [f.id, f.title || 'Untitled feed']));
  const groups = walletGroups(person, recipients);
  const profile = profileLine(person, titles, feeds);
  const check = checkRow(person);
  const remove = (kind, linkId) => withLeave(() => act(() => api(`/people/${id}/${kind}/${linkId}`, { method: 'DELETE' })));
  const removeButton = (kind, linkId, label) => html`<button class="button destructive" type="button" aria-label=${label}
    onClick=${() => remove(kind, linkId)}>Remove</button>`;
  const open = (kind) => withLeave(() => setSheet(kind));
  // The delete changes no feed (ADR 0102). Then the People page opens.
  const deletePerson = () => run(async () => {
    if (!await askConfirm(CONFIRMS.deletePerson(person.name))) return;
    await api(`/people/${id}`, { method: 'DELETE' });
    go({ name: 'people' });
  });
  const sections = {
    profile: () => html`<${Fragment}><header class="person-head"><${PersonArt} person=${person} large />
      <div class="text"><h1>${person.name}</h1>
        ${(profile.role || profile.members.length > 0) && html`<p class="hint">${profile.role}${profile.role && profile.members.length > 0 && ' · '}${profile.members.map((m, i) =>
          html`${i > 0 && ', '}<span class=${`dot ${m.dot || 'feed'}`} aria-hidden="true"></span><button class="button link" type="button" onClick=${() => go({ name: 'feed', id: m.id })}>${m.title}</button>${m.role && ` · ${m.role}`}`)}</p>`}
        ${person.keys.map((k) => html`<p class="hint key-hint" key=${k.id}><span class="key-hint-text">Nostr <span class=${keyRow(k.npub, profiles, person.name).mono ? 'mono key-text' : ''}>${keyRow(k.npub, profiles, person.name).title}</span> · ${keyEvidenceText(k.evidence)}</span>
          <${CopyButton} text=${k.npub} label="the Nostr key" /></p>`)}
      </div></header>
      <div class="actions"><button class="button action" type="button" onClick=${() => setFinding(person.name)}>Find on Music Index</button></div><//>`,
    check: () => html`<div class="group"><${ListRow} title=${check.title} lines=${check.lines} count=${check.count}
      onClick=${() => go({ name: 'check', id: person.id })} /></div>`,
    more: () => more.length > 0 && html`<section class="page-section"><h2>More from your feeds</h2>
      <p class="hint">${MORE_HINT}</p>
      <div class="group">${moreRows(more, titles, profiles, person.name).map((row) => html`<div class="link-row static key-line more-row" key=${row.key}>
          ${row.npub && html`<${PersonArt} person=${{ img: row.picture, name: row.title }} />`}<${KeyText} row=${row} />
          ${row.npub && html`<${CopyButton} text=${row.npub} label="the Nostr key" />`}
          <div class="actions more-actions">
            ${row.warning && html`<button class="button action" type="button" onClick=${() => go({ name: 'check', id: person.id })}>Correct the Feeds</button>`}
            <button class="button action" type="button" aria-label=${`${moreAddLabel(row)}: ${row.title}`}
              onClick=${() => withLeave(() => act(async () => { await api(`/people/${id}/items`, { method: 'POST', body: row.body }); await loadMore(); }))}>${moreAddLabel(row)}</button>
          </div></div>`)}</div></section>`,
    payment: () => html`<section class="page-section"><h2>Payment</h2>
      ${groups.length > 0 && html`<p class="hint">${DEFAULT_HINT}</p>`}
      ${groups.length === 0 && html`<p class="empty">No wallet yet. Add a wallet to give this person a payment address.</p>`}
      ${defaultFirst(groups, shown.default_address).map((g) => html`<div class="group" key=${g.id}>${g.members.map((m) => {
        const isDefault = isDefaultAddress(m.key, shown.default_address);
        return html`<${AddressRow} key=${JSON.stringify(m.key)} member=${m.member} count=${usedText(m.count)}
          pending=${isDefault && isPending(saved, shown, 'default_address')}
          tag=${isDefault && html` <span class="default-tag"><${Icon} name="check" />Default</span>`}
          mark=${!isDefault && html`<button class="button action" type="button" aria-label=${`Make Default: ${m.member.address}`}
            onClick=${() => update({ default_address: keyValue(m.key) })}>Make Default</button>`}
          action=${html`<button class="button" type="button" aria-label=${`Edit ${m.member.address}`}
            onClick=${() => withLeave(() => setEditing(walletEdit(g, m)))}>Edit</button>`} />`;
      })}
        <div class="actions"><button class="button destructive" type="button" onClick=${() => remove('wallets', g.id)}>Remove Wallet</button></div></div>`)}
      ${shown.wallets.length > 0 && html`<div class="group">${shown.wallets.map((w) => html`<div class="link-row static" key=${`p${keyValue(w.key)}`}>
        <span class="text"><span class="label">Wallet</span><span class="summary mono address-text">${w.key.address}</span><span class="summary">Not saved yet</span></span></div>`)}</div>`}
      ${suggestions.map((s) => html`<div class="group" key=${s.address}><${AddressRow} member=${{ address: s.address, recipient_type: 'lnaddress' }}
        note=${profileAddressNote(s.npub, profiles)} />
        <div class="actions"><button class="button action" type="button" onClick=${() => act(async () => {
          await api(`/people/${id}/profile-addresses`, { method: 'POST', body: { address: s.address } });
          await loadSuggestions();
        })}>Add to Wallet</button>
        <button class="button" type="button" onClick=${() => act(async () => {
          // "Dismiss" hides the address for this person for good (ADR 0098 item 18).
          await api(`/people/${id}/profile-addresses/dismiss`, { method: 'POST', body: { address: s.address } });
          await loadSuggestions();
        })}>Dismiss</button></div></div>`)}
      <div class="group"><${AddRow} title="Add a Wallet" onClick=${() => withLeave(() => setLinking('wallets'))} /></div></section>`,
    credits: () => html`<section class="page-section"><h2>Credits</h2>
      <${PersonFields} values=${shown} saved=${saved} update=${update} detail=${detail} profiles=${profiles} titles=${titles} feeds=${feeds} sources=${sources} />
      ${(person.credit_names.length > 0 || person.keys.length > 0) && html`<details class="disclosure group credit-fold">
        <summary class="link-row"><span class="text"><span class="label">${creditFoldSummary(person).title}</span>
          <span class="summary">${creditFoldSummary(person).line}</span></span></summary>
        <p class="hint">${CREDIT_FOLD_HINT}</p>
        ${creditNameRows(person, profiles).map((row) => html`<${CreditRow} key=${row.key} row=${row} removeLabel=${`Remove ${row.title}`}
          onEdit=${() => withLeave(() => setEditing(creditNameEdit(person.credit_names.find((c) => c.id === row.creditId))))}
          onRemove=${() => withLeave(() => act(async () => {
            await api(`/people/${id}/credit-names/${row.creditId}`, { method: 'DELETE' });
            if (row.keyId !== null) await api(`/people/${id}/keys/${row.keyId}`, { method: 'DELETE' });
          }))} />`)}
        ${keysWithNoCreditName(person).map((k) => keyOnlyRow(k, profiles, person.name)).map((row) => html`<${CreditRow} key=${row.key} row=${row}
          removeLabel=${`Remove the Nostr key ${shortKey(row.npub)}`} onRemove=${() => remove('keys', row.keyId)} />`)}</details>`}
      ${shown.keys.length > 0 && html`<div class="group">${shown.keys.map((npub) => html`<div class="link-row static" key=${`p${npub}`}>
        <span class="text"><span class="label">Nostr key</span><span class="summary mono">${shortKey(npub)}</span><span class="summary">Not saved yet</span></span></div>`)}</div>`}
      <div class="group"><${AddRow} title="Add a Credit Name" onClick=${() => withLeave(() => setLinking('credit-names'))} />
        <${AddRow} title="Add a Nostr Key" onClick=${() => withLeave(() => setLinking('keys'))} /></div></section>`,
    support: () => html`<section class="page-section"><h2>Support links</h2>
      <div class="group">${person.support_links.map((l) => html`<div class="link-row static more-row" key=${l.id}><span class="text">
          <span class="label">${l.label || 'Support link'}</span><span class="summary address-text">${l.url}</span></span>
          <div class="actions more-actions"><button class="button action" type="button" aria-label=${`Edit the support link ${l.label || l.url}`}
            onClick=${() => withLeave(() => setEditing(supportLinkEdit(l)))}>Edit</button>
          ${removeButton('support-links', l.id, `Remove the support link ${l.label || l.url}`)}</div></div>`)}
        <${AddRow} title="Add a Support Link" onClick=${() => withLeave(() => setLinking('support-links'))} /></div></section>`,
    member: () => {
      const rows = linkRows(shown, saved, feeds, person.linked_credits);
      return html`<section class="page-section"><h2>Linked artist and label pages</h2><p class="hint">${LINK_HINT}</p>
      <div class="group">${rows.length === 0
        ? html`<div class="link-row static"><span class="text"><span class="summary">${LINK_EMPTY}</span></span></div>`
        : rows.map((row) => html`<div class="link-row static" key=${row.id}><span class=${`dot ${row.dot}`} aria-hidden="true"></span>
          <span class="text"><span class="label">${row.title}</span><span class="summary">${row.line}</span></span>
          <button class="button destructive" type="button" aria-label=${`Remove the link to ${row.title}`}
            onClick=${() => update({ members: shown.members.filter((feedId) => feedId !== row.id) })}>Remove</button></div>`)}</div>
      <div class="group rows"><${ChoiceSelect} id="person-link-add" label="Link to an artist or label page" value="" placeholder="Select…"
        options=${linkChoices(feeds, shown.members)} onChange=${(value) => { if (value) update({ members: [...shown.members, value] }); }} /></div></section>`;
    },
    change: () => html`<section class="page-section"><h2>Change your feeds</h2>
      <p class="hint">${CHANGE_NOTE}</p>
      <div class="group">${CHANGE_ROWS.map(([kind, line]) => html`<${ListRow} key=${kind} title=${SHEET_TITLES[kind]} lines=${[line]}
        onClick=${() => open(kind)} />`)}</div></section>`,
    delete: () => html`<section class="page-section"><div class="actions">
      <button class="button" type="button" onClick=${() => withLeave(() => setMerging(true))}>Merge with a Person</button>
      <button class="button destructive" type="button" onClick=${() => withLeave(deletePerson)}>Delete Person</button></div></section>`,
  };
  return html`<div><${Bar}><span class="bar-title">${person.name}</span>
    <div class="draft-toolbar"><span class=${saveFailure ? 'draft-status field-error' : 'draft-status'} role="status">${saveFailure || pendingStatus(keys.length)}</span>
      <button class="button" type="button" disabled=${!dirty || saving} onClick=${discard}>Discard</button>
      <button class="button primary" type="button" disabled=${!dirty || saving} onClick=${save}>Save</button></div><//><section class="section">
    <${ErrorNotice} error=${error} />
    ${pageSequence('person').map((section) => html`<${Fragment} key=${section.id}>${sections[section.id]()}<//>`)}
  </section>
  <${OperationSheet} kind=${sheet} person=${person} groups=${groups} rows=${feedRows(person.places, feeds)} feeds=${feeds}
    go=${go} onClose=${() => setSheet(null)} />
  <${LinkSheet} kind=${editing ? editing.kind || 'wallets' : null} personId=${person.id} edit=${editing} onClose=${() => setEditing(null)}
    onAdded=${() => { reload(); loadSuggestions(); loadMore(); }} />
  <${LinkSheet} kind=${linking === 'keys' ? null : linking} personId=${person.id} onClose=${() => setLinking(null)}
    onAdded=${() => { reload(); loadSuggestions(); loadMore(); }} />
  <${NostrSheet} open=${linking === 'keys'} value=${null} offerPeople=${false} onSet=${(npub) => { if (npub) addKey(npub); }}
    onClose=${() => setLinking(null)} />
  <${MergeSheet} open=${merging} person=${person} onClose=${() => setMerging(false)}
    onMerged=${() => { setMerging(false); reload(); loadMore(); loadSuggestions(); }} />
  <${FindSheet} open=${finding !== null && picked === null} personId=${person.id} name=${person.name} query=${finding || ''}
    onQuery=${setFinding} onClose=${() => setFinding(null)} onPick=${setPicked} />
  <${ValuesSheet} personId=${person.id} row=${picked} shown=${shown}
    onBack=${() => { setPicked(null); setFinding(person.name); }} onClose=${() => setPicked(null)} onFill=${fill} />
  ${offers.length > 0 && html`<${CreditSheet} key=${`${offers[0].kind}${offers[0].feedId}`} offer=${offers[0]} person=${person}
    feeds=${feeds} position=${creditPosition(offerTotal - offers.length, offerTotal)}
    onMade=${(setId) => { made.current.push(setId); }} onNotNow=${nextOffer} />`}</div>`;
}

// The sheets "Add {name} to the Credits of {feed}?" and "Remove {name} from
// the Credits of {feed}?" (ADR 0118 items 4 to 7). The sheet is open while it
// is shown. "Not Now" closes it, and changes no feed. Each close opens the next
// sheet (ADR 0132 item 12).
function CreditSheet({ offer, person, feeds, position, onMade, onNotNow }) {
  const ref = useSheet(true);
  const feed = (feeds || []).find((f) => String(f.id) === offer.feedId);
  const title = feed ? (feed.title || 'Untitled feed') : 'this feed';
  const close = () => ref.current.close();
  const adding = offer.kind === 'add';
  return html`<dialog ref=${ref} class=${adding ? 'sheet' : 'sheet small'} aria-labelledby="credit-title" onClose=${onNotNow}>
    <header>
      <h2 id="credit-title">${adding ? creditAddTitle(person.name, title) : creditRemoveTitle(person.name, title)}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${close}>✕</button>
    </header>
    ${adding
      ? html`<${CreditAddForm} person=${person} feedId=${offer.feedId} title=${title} position=${position} onMade=${onMade} close=${close} />`
      : html`<${CreditRemoveForm} person=${person} feedId=${offer.feedId} position=${position} onMade=${onMade} close=${close} />`}
  </dialog>`;
}

// The body of the sheet that adds a credit. The role starts with the usual
// role of the person (ADR 0118 item 4).
function CreditAddForm({ person, feedId, title, position, onMade, close }) {
  const [role, setRole] = useState(person.usual_role || '');
  const [group, setGroup] = useState(person.usual_group || '');
  const { busy, error, run } = useAction();
  const hasKey = person.keys.length > 0;
  const review = () => run(async () => {
    const made = await api(`/people/${person.id}/changes`, { method: 'POST', body: { operation: creditOperation(role, group), feed_ids: [Number(feedId)] } });
    onMade(made.id);
    close();
  });
  const roleLine = [role, group].filter((text) => text).join(' · ');
  return html`<div class="body">
      <p class="hint">${position && html`<strong>${position}</strong> `}${creditAddNote(person.name, title)}</p>
      <div class="group rows"><${RoleMenu} id="credit-role" label="Role" role=${role} group=${group}
        onChange=${(nextRole, nextGroup) => { setRole(nextRole); setGroup(nextGroup); }} /></div>
      <section class="page-section"><h2>The credit</h2>
        <div class="group"><div class="link-row static key-line">
          <${PersonArt} person=${person} />
          <span class="text"><span class="label">${person.name}</span>
            ${roleLine && html`<span class="summary">${roleLine}</span>`}
            ${person.img && html`<span class="summary address-text">${person.img}</span>`}
            ${person.href && html`<span class="summary address-text">${person.href}</span>`}
            ${hasKey && html`<span class="summary mono">${shortKey(person.keys[0].npub)}</span>`}</span>
        </div></div>
      </section>
      <p class="hint">${creditPublicNote(hasKey)}</p>
      <${ErrorNotice} error=${error} />
    </div>
    <footer>
      <button class="button" type="button" onClick=${close}>Not Now</button>
      <button class="button primary" type="button" disabled=${busy || role === ''} onClick=${review}>Review in 1 Feed</button>
    </footer>`;
}

// The body of the sheet that removes a credit (ADR 0118 item 6).
function CreditRemoveForm({ person, feedId, position, onMade, close }) {
  const { busy, error, run } = useAction();
  const review = () => run(async () => {
    const made = await api(`/people/${person.id}/changes`, { method: 'POST', body: { operation: REMOVE_CREDIT_OPERATION, feed_ids: [Number(feedId)] } });
    onMade(made.id);
    close();
  });
  return html`<div class="body">
      <p class="hint">${position && html`<strong>${position}</strong> `}${creditRemoveNote(person.name)}</p>
      <${ErrorNotice} error=${error} />
    </div>
    <footer>
      <button class="button" type="button" onClick=${close}>Not Now</button>
      <button class="button primary" type="button" disabled=${busy} onClick=${review}>Review in 1 Feed</button>
    </footer>`;
}

// The sheet "Merge with {name}" (ADR 0103 items 14 to 16). The core moves the
// links and deletes the other person. No feed changes.
function MergeSheet({ open, person, onClose, onMerged }) {
  const ref = useSheet(open);
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="merge-title" onClose=${onClose}>
    ${open && html`<${MergeForm} person=${person} onCancel=${() => ref.current.close()} onMerged=${onMerged} />`}
  </dialog>`;
}

function MergeForm({ person, onCancel, onMerged }) {
  const [people, setPeople] = useState(null);
  const [otherId, setOtherId] = useState('');
  const { busy, error, run } = useAction();
  useEffect(() => {
    run(async () => {
      const list = await api('/people');
      setPeople(list);
      const first = personChoices(list, person.id)[0];
      if (first) setOtherId(first[0]);
    });
  }, []);
  const other = (people || []).find((p) => String(p.id) === otherId) || null;
  const profiles = useProfiles(other ? other.keys.map((k) => k.npub) : []);
  const links = mergeLinks(other, profiles);
  const result = other ? mergeResult(person, other) : null;
  const merge = () => run(async () => {
    await api(`/people/${person.id}/merge`, { method: 'POST', body: mergeBody(otherId) });
    merged.set(String(otherId), person.id);
    onMerged();
  });
  return html`<header>
      <h2 id="merge-title">${mergeTitle(person.name)}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${onCancel}>✕</button>
    </header>
    <div class="body">
      ${people && personChoices(people, person.id).length === 0 && html`<p class="empty">${NO_OTHER_TEXT}</p>`}
      ${other && html`<div class="group rows"><${ChoiceSelect} id="merge-person" label="Person to merge" value=${otherId}
        hint=${mergeHint(person.name)} options=${personChoices(people, person.id)} onChange=${setOtherId} /></div>`}
      ${other && html`<section class="page-section"><h2>${`${other.name} has`}</h2>
        ${links.length === 0 && html`<p class="empty">This person has no link.</p>`}
        ${links.length > 0 && html`<div class="group rows">${links.map((l) => html`<div class="link-row static" key=${l.key}><span class="text">
          <span class=${l.mono ? 'label mono key-text' : 'label'}>${l.title}</span>${l.lines.map((line, i) => html`<span class="summary" key=${i}>${line}</span>`)}</span></div>`)}</div>`}</section>`}
      ${result && html`<section class="page-section"><h2>After the merge</h2><div class="group rows"><div class="link-row static"><span class="text">
        <span class="label">${result.title}</span>${result.lines.map((line, i) => html`<span class="summary" key=${i}>${line}</span>`)}</span></div></div></section>`}
      <${ErrorNotice} error=${error} />
    </div>
    <footer>
      <button class="button" type="button" onClick=${onCancel}>Cancel</button>
      <button class="button primary" type="button" disabled=${busy || !other} onClick=${merge}>${busy ? 'Merging the People…' : 'Merge the People'}</button>
    </footer>`;
}

// One feed of the check with a difference. The entry opens in the list and
// shows each difference, then the buttons (ADR 0120 items 1 to 3).
function CheckEntry({ entry, feed, title, person, profiles, open, onToggle, onCorrect, go }) {
  return html`<details class="disclosure" open=${open} onToggle=${(e) => onToggle(e.currentTarget.open)}>
    <summary class="link-row"><span class="text"><span class="label">${title}</span>
      <span class="summary">${checkSummary(entry, feed)}</span></span></summary>
    <div class="change-diff">
      ${differenceBlocks(entry, person.name, feed, profiles, person).map((block, i) => html`<${DifferenceBlock} key=${i} block=${block} />`)}
    </div>
    <div class="actions">
      <button class="button primary" type="button" onClick=${onCorrect}>${useValuesLabel(person.name)}</button>
      <button class="button" type="button" onClick=${() => go({ name: 'feed', id: entry.feed_id })}>${openLabel(feed)}</button>
    </div>
  </details>`;
}

// One difference: the position, the type of the value, then the two values.
// The value of the feed is shown as the old value.
function DifferenceBlock({ block }) {
  return html`<div class="difference">
    <div class="label">${block.place}</div>
    <div class="summary">${block.what}</div>
    <div class="diff">${block.lines.map((line) => html`<span class=${line.old ? 'line remove' : 'line add'}>${line.label} · ${line.value}</span>`)}</div>
  </div>`;
}

// The page "Check your feeds" (ADR 0111 items 9 to 13, ADR 0120). The core
// compares each feed with the person. A feed with a difference opens in the list.
// The button "Correct N Feeds" opens the sheet of the correction (ADR 0111
// item 14). A feed that is correct opens the feed in the editor.
export function CheckPage({ id, feeds, go }) {
  const [person, setPerson] = useState(null);
  const [check, setCheck] = useState(null);
  // The sheet "Correct the Feeds": null when closed. `feedIds` is null for all
  // feeds, or the one feed of an entry.
  const [correcting, setCorrecting] = useState(null);
  // The entry that is open. One entry is open at a time (ADR 0028).
  const [openId, setOpenId] = useState(null);
  const { error, run } = useAction();
  useEffect(() => {
    run(async () => {
      const [p, c] = await Promise.all([api(`/people/${id}`), api(`/people/${id}/check`)]);
      setPerson(p);
      setCheck(c);
    });
  }, [id]);
  const profiles = useProfiles([...checkKeys(check), ...((person && person.keys) || []).map((k) => k.npub)]);
  const rows = new Map((person ? feedRows(person.places, feeds) : []).map((r) => [r.id, r]));
  const titles = new Map((feeds || []).map((f) => [f.id, f.title || 'Untitled feed']));
  const groups = checkGroups(check);
  const feedOf = (feedId) => (feeds || []).find((f) => f.id === feedId);
  return html`<div><${Bar}><span class="bar-title">${CHECK_TITLE}</span><//><section class="section">
    <${ErrorNotice} error=${error} />
    ${(person === null || check === null) && !error && html`<p class="empty" role="status">Checking the feeds…</p>`}
    ${person && check && html`<header class="recipient-head"><h1>${CHECK_TITLE}</h1>
        <p class="hint">${checkIntro(person.name)}</p></header>
      ${correctCount(check) > 0 && html`<div class="actions"><button class="button primary" type="button"
        onClick=${() => setCorrecting({ feedIds: null })}>${correctButtonLabel(correctCount(check))}</button></div>`}
      ${groups.length === 0 && html`<p class="empty">No feed uses this person.</p>`}
      ${groups.map((g) => html`<section class="page-section" key=${g.state}><h2>${g.title}</h2>
        <div class="group">${g.feeds.map((entry) => {
          const row = rows.get(entry.feed_id);
          const title = row ? row.title : titles.get(entry.feed_id) || `Feed ${entry.feed_id}`;
          if (entry.state !== 'correct') {
            return html`<${CheckEntry} key=${entry.feed_id} entry=${entry} feed=${feedOf(entry.feed_id)} title=${title} person=${person}
              profiles=${profiles} open=${openId === entry.feed_id}
              onToggle=${(open) => setOpenId((current) => (open ? entry.feed_id : (current === entry.feed_id ? null : current)))}
              onCorrect=${() => setCorrecting({ feedIds: [entry.feed_id] })} go=${go} />`;
          }
          const place = row ? row.lines : [];
          return html`<${ListRow} key=${entry.feed_id} title=${title}
            lines=${checkFeedLines(entry, place, person.name, profiles)} onClick=${() => go({ name: 'feed', id: entry.feed_id })} />`;
        })}</div></section>`)}`}
  </section>
  <${CorrectSheet} open=${correcting !== null} feedIds=${correcting ? correcting.feedIds : null} person=${person} check=${check}
    titles=${titles} placeRows=${rows} go=${go} onClose=${() => setCorrecting(null)} /></div>`;
}

// The sheet "Correct the Feeds: {name}" (ADR 0111 items 14 to 17). The
// preview of the core lists the feeds that the selected corrections change.
// The primary button makes the change set, and the review follows. With
// `feedIds`, the sheet covers only those feeds (ADR 0120 item 3).
function CorrectSheet({ open, person, check, titles, placeRows, go, onClose, feedIds = null }) {
  const ref = useSheet(open);
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="correct-title" onClose=${onClose}>
    ${open && person && check && html`<${CorrectForm} person=${person} check=${check} titles=${titles} placeRows=${placeRows} go=${go}
      onlyIds=${feedIds} onCancel=${() => ref.current.close()} />`}
  </dialog>`;
}

function CorrectForm({ person, check, titles, placeRows, go, onCancel, onlyIds = null }) {
  const [fields, setFields] = useState(() => correctStart(person));
  const [list, setList] = useState(null);
  const [previewError, setPreviewError] = useState(null);
  const { busy, error, run } = useAction();
  const profiles = useProfiles([...person.keys.map((k) => k.npub), ...checkKeys(check)]);
  // The feeds of the sheet: the one feed of an entry, or each feed that is not correct.
  const feedIds = onlyIds || correctFeedIds(check);
  // When the profiles come, the key menu takes a key with no warning, until
  // the user changes a selection (ADR 0103 item 18).
  const touched = useRef(false);
  const set = (patch) => { touched.current = true; setFields({ ...fields, ...patch }); };
  useEffect(() => {
    if (!touched.current && profiles && profiles.size > 0) setFields((before) => correctKeyStart(person, before, profiles));
  }, [profiles]);
  // After each change of a selection, the core gives the outcome of each feed.
  useEffect(() => {
    if (!correctAnySelected(fields) || feedIds.length === 0) { setList({ rows: [], omitted: 0 }); setPreviewError(null); return undefined; }
    let live = true;
    const timer = setTimeout(() => {
      api(`/people/${person.id}/changes/preview`, { method: 'POST', body: { operation: correctBody(fields), feed_ids: feedIds } })
        .then((result) => { if (live) { setList((before) => correctRows(result, titles, placeRows, before ? before.rows : [], check, fields, person.name, profiles)); setPreviewError(null); } },
          (e) => { if (live) { setList({ rows: [], omitted: 0 }); setPreviewError(e); } });
    }, 250);
    return () => { live = false; clearTimeout(timer); };
  }, [JSON.stringify(fields)]);
  const rows = list ? list.rows : [];
  const ids = checkedIds(rows);
  const submit = () => run(async () => {
    const made = await api(`/people/${person.id}/changes`, { method: 'POST', body: { operation: correctBody(fields), feed_ids: ids } });
    go({ name: 'change-set', id: made.id });
  });
  const choices = correctKeyChoices(person, profiles);
  return html`<header>
      <h2 id="correct-title">${correctTitle(person.name)}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${onCancel}>✕</button>
    </header>
    <div class="body">
      <section class="page-section"><h2>Changes</h2><div class="group rows">
        ${choices.length > 0 && html`<${ChoiceSelect} id="correct-key" label="Key" value=${fields.npub} options=${choices}
          onChange=${(npub) => set({ npub })} />`}
        ${correctOptions(person, fields, profiles, check).map((o) => html`<${FeedCheckRow} key=${o.id} title=${o.label} text=${o.line}
          checked=${fields[o.id]} onChange=${() => set({ [o.id]: !fields[o.id] })} />`)}</div></section>
      ${list === null && !previewError && html`<p class="empty" role="status">Checking the feeds…</p>`}
      ${list !== null && rows.length === 0 && !previewError && html`<p class="empty">${CORRECT_EMPTY}</p>`}
      ${rows.length > 0 && html`<${FeedChecks} rows=${rows} setRows=${(next) => setList({ ...list, rows: next })} empty="" separate />`}
      ${rows.length > 0 && list.omitted > 0 && html`<p class="hint">${CORRECT_OMITTED}</p>`}
      <${ErrorNotice} error=${previewError || error} />
    </div>
    <footer>
      <button class="button primary" type="button" disabled=${busy || ids.length === 0} onClick=${submit}>${busy ? BUSY_REVIEW : reviewLabel(ids.length)}</button>
    </footer>`;
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
  const addressFields = html`<${Field} id="op-address" label=${fields.type === 'lnaddress' ? 'LNURL address' : 'Node address key'} required
      value=${fields.address} onInput=${(e) => set({ address: e.target.value })} />
    ${fields.type !== 'lnaddress' && html`<${Field} id="op-custom-key" label="Custom key" hint=${CUSTOM_KEY_HINT}
        value=${fields.custom_key} onInput=${(e) => set({ custom_key: e.target.value })} />
      <${Field} id="op-custom-value" label="Account" value=${fields.custom_value} onInput=${(e) => set({ custom_value: e.target.value })} />`}`;
  return html`<div class="body">
      ${kind === 'remove' && html`<p class="hint">${removeWalletHint(recipientTitle(recipient))}</p>`}
      ${kind === 'rename' && html`<div class="group rows"><${Field} id="op-name" label="Name" value=${fields.name}
        onInput=${(e) => set({ name: e.target.value })} /></div>`}
      ${kind === 'use_lnaddress' && html`<div class="group rows">
        ${asks && field === null && html`<p class="hint" role="status">Studio looks for LNURL addresses of this wallet…</p>`}
        ${field && field.menu && html`<${ChoiceSelect} id="op-suggestion" label="LNURL address" value=${choice}
          options=${field.options} hint=${field.hints[choice]} onChange=${choose} />`}
        ${!(field && field.menu && choice !== OTHER_ADDRESS) && html`<${Field} id="op-address" label=${field && field.menu ? 'Other address' : 'LNURL address'} required
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

// The page of a wallet with no person (ADR 0099 item 7, ADR 0111 item 8). It
// has the row "Add to a Person", the operations and no list of feeds: each
// sheet lists the feeds.
export function WalletPage({ recipient, feeds, go }) {
  const [uses, setUses] = useState(null);
  const [sheet, setSheet] = useState(null);
  const [personSheet, setPersonSheet] = useState(false);
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
    <section class="page-section"><h2>Person</h2><div class="group"><${ListRow} title=${PERSON_SECTION_TITLE} lines=${[PERSON_SECTION_LINE]}
      onClick=${() => setPersonSheet(true)} /></div></section>
    <section class="page-section"><h2>Change your feeds</h2><div class="group">${walletActions(recipient).map((action) => html`<${ListRow}
      key=${action.kind} title=${action.label} lines=${[action.line]} disabled=${action.kind !== 'add' && !used}
      onClick=${() => setSheet(action.kind)} />`)}</div></section>
    ${uses === null && !error && html`<p class="empty" role="status">Loading the feeds…</p>`}
    ${uses !== null && uses.length === 0 && html`<p class="empty">No feed uses this wallet. Add to All Releases puts it in feeds.</p>`}
    <${WalletSheet} kind=${sheet} recipient=${recipient} uses=${uses || []} feeds=${feeds} go=${go} onClose=${() => setSheet(null)} />
    <${WalletPersonSheet} open=${personSheet} recipient=${recipient} go=${go} onClose=${() => setPersonSheet(false)} />
  </section></div>`;
}

// The sheet "Add to a Person: {name}" of a payee with no person (ADR 0111
// item 8). The menu "Person" picks a new person or a person that exists. The
// sheet changes no feed. A new person is made first, then the wallet is added.
function WalletPersonSheet({ open, recipient, go, onClose }) {
  const ref = useSheet(open);
  return html`<dialog ref=${ref} class="sheet small" aria-labelledby="wallet-person-title" onClose=${onClose}>
    ${open && html`<${WalletPersonForm} recipient=${recipient} go=${go} onCancel=${() => ref.current.close()} />`}
  </dialog>`;
}

function WalletPersonForm({ recipient, go, onCancel }) {
  const title = recipientTitle(recipient);
  const [state, setState] = useState(() => walletToPersonStart(title));
  const [people, setPeople] = useState(null);
  const { busy, error, run } = useAction();
  useEffect(() => { run(async () => { setPeople(await api('/people')); }); }, []);
  const submit = () => run(async () => {
    let id = Number(state.personId);
    if (state.personId === NEW_PERSON) id = (await api('/people', { method: 'POST', body: addPersonBody(state.name) })).id;
    await api(`/people/${id}/wallets`, { method: 'POST', body: walletBodyOf(recipient) });
    go({ name: 'person', id });
  });
  const row = recipientRow(recipient);
  return html`<header>
      <h2 id="wallet-person-title">${addToTitle(title)}</h2>
      <button class="button quiet icon-button" type="button" aria-label="Close" onClick=${onCancel}>✕</button>
    </header>
    <div class="body">
      <div class="group rows"><${ChoiceSelect} id="wallet-person" label="Person" value=${state.personId}
          options=${checkSheetChoices(people || [])} onChange=${(personId) => setState({ ...state, personId })} />
        ${state.personId === NEW_PERSON && html`<${Field} id="wallet-person-name" label="Name" required value=${state.name}
          onInput=${(e) => setState({ ...state, name: e.target.value })} />`}</div>
      <div class="group">${row.lines.map((line, i) => html`<div class="link-row static" key=${i}><span class="text">
        <span class="label">${line}</span>${i === 0 && html`<span class="summary">${row.count}</span>`}</span></div>`)}</div>
      <p class="hint">${WALLET_TO_PERSON_HINT}</p>
      <${ErrorNotice} error=${error} />
    </div>
    <footer>
      <button class="button" type="button" onClick=${onCancel}>Cancel</button>
      <button class="button primary" type="button" disabled=${busy || people === null || !walletToPersonReady(state)} onClick=${submit}>${busy ? 'Working…' : walletToPersonLabel(state)}</button>
    </footer>`;
}

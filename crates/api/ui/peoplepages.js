// The screens of the People area (ADR 0099, task 065). The components show
// the pages and call the API. The functions of `people.js` give each text
// and each state, and the core applies each rule (ADR 0042, ADR 0065).
import { html, useState, useEffect } from './vendor/htm-preact-standalone.module.js';
import { useSheet } from './editor.js';
import { api, Bar, ChoiceSelect, ErrorNotice, FeedCheckRow, Field, useAction, useImage } from './app.js';
import { recipientKey, recipientRow } from './recipients.js';
import {
  peopleSections, PEOPLE_EMPTY, suggestionRow, personRow, makePersonStart, makePersonBody, initials,
  filterPeople, filterPayees, payeesWithNoPerson, openChanges, changeRow, memberChoices, makeSheetTitle,
  makeSheetHint, creditItemRow, payeeItemRow, keyItemRow, toggledItem, dismissBody, addPersonBody,
} from './people.js';

// The round picture of a person. A person with no picture, or a picture that
// Studio cannot read, shows the initials of the name.
function PersonArt({ person }) {
  const { src } = useImage(person.img);
  return html`<span class="person-art" aria-hidden="true">${src ? html`<img src=${src} alt="" />` : initials(person.name)}</span>`;
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

// The page of a person, until packet 065c gives it.
export function PersonStub({ id }) {
  const [person, setPerson] = useState(null);
  const { error, run } = useAction();
  useEffect(() => { run(async () => setPerson(await api(`/people/${id}`))); }, [id]);
  return html`<div><${Bar}><span class="bar-title">${person ? person.name : 'Person'}</span><//><section class="section">
    <${ErrorNotice} error=${error} />
    ${person && html`<p class="empty">Studio shows this page in the next version.</p>`}
  </section></div>`;
}

// The one layout of the sheets that add a value (ADR 0145): the form, the row
// of musicindex under its heading, then the lists that fill the form.

import { html, useState, useRef } from './vendor/htm-preact-standalone.module.js';
import { useSheet } from './editor.js';
import { api, ErrorNotice, Field, Icon } from './app.js';
import { LookupRow, PersonArt, useLookup } from './peoplepages.js';
import { foldStartsOpen, lookupGroups, FIND_ONE_EMPTY, FIND_WAIT, FIND_ONE_HINT } from './people.js';

// The folds that the user opened or closed, by `id`, for this visit. A fold keeps
// its state after the user opens a page from it and comes back.
const foldStates = new Map();

// A list that folds, with its count. A long list starts closed, unless `startOpen`
// says different. With `forceOpen`, it stays open, for a search of its rows.
export function PickFold({ id, title, note, rows, startOpen = foldStartsOpen(rows.length), forceOpen = false, onClick }) {
  const open = forceOpen || (foldStates.has(id) ? foldStates.get(id) : startOpen);
  const remember = (e) => { if (id && !forceOpen) foldStates.set(id, e.currentTarget.open); };
  return html`<details class="disclosure pick-fold" open=${open} onClick=${onClick} onToggle=${remember}>
    <summary><span class="pick-fold-title">${title}</span><span class="pick-fold-count">${rows.length}</span></summary>
    ${note && html`<p class="note">${note}</p>`}
    <div class="group">${rows}</div>
  </details>`;
}

// `fields` is the content of the form. `musicindex` gives the title, the summary
// and `onOpen` of its row. Each entry of `lists` has a title and rows.
export function PickerLayout({ id, onSubmit, fields, notice, musicindex, lists }) {
  const form = useRef(null);
  // A selection fills the form, so the form must be in view. At narrow width it
  // scrolls with the list.
  const showForm = (e) => {
    if (e.target.closest('button.link-row:not(:disabled)')) form.current.scrollIntoView({ block: 'nearest' });
  };
  return html`<form class="body" id=${id} onSubmit=${onSubmit}>
    <section class="page-section pinned-form" ref=${form}>${fields}</section>
    ${notice}
    <section class="page-section"><h2>From musicindex</h2>
      <div class="group"><button class="link-row" type="button" onClick=${musicindex.onOpen}>
        <span class="text"><span class="label">${musicindex.title}</span>
          <span class="summary">${musicindex.summary}</span></span>
        <span class="go" aria-hidden="true">›</span></button></div></section>
    ${lists.filter((list) => list.rows.length > 0).map((list) => html`<${PickFold} key=${list.title} title=${list.title} rows=${list.rows} onClick=${showForm} />`)}
  </form>`;
}

// The tag of a value that is already there (ADR 0146 item 3).
export function TakenTag({ text = 'Already in this list' }) {
  return html`<span class="taken-tag"><${Icon} name="check" />${text}</span>`;
}

// A row of a list that fills the form. `after` is a control beside the row. A
// row that the list has already keeps its content and has a tag.
export function PickFillRow({ title, line, lead, after, taken, selected, onSelect }) {
  return html`<div class=${taken ? 'link-row static pick-fill pick-taken' : 'link-row static pick-fill'}>
    <button class="link-row" type="button" disabled=${taken} aria-current=${selected ? 'true' : undefined} onClick=${onSelect}>
      ${lead}<span class="text"><span class="pick-name"><span class="label">${title}</span>${taken && html`<${TakenTag} />`}</span>
        <span class="summary">${line}</span></span>
    </button>${after}</div>`;
}

// The results of the one search (ADR 0145 item 3): the artist and label feeds,
// then the other feeds. `onPick` gets the row, and for a row of the other feeds
// it gets `true` as its second argument.
export function SearchResults({ query, search, disabled, onPick }) {
  const { rows, feeds, note, error } = search;
  const searching = query.trim() !== '';
  return html`<${ErrorNotice} error=${error} />
    ${searching && rows === null && html`<p class="empty" role="status">${FIND_WAIT}</p>`}
    ${note && html`<p class="hint" role="status">${note.message}</p>`}
    ${searching && rows !== null && rows.length === 0 && feeds.length === 0 && !error && !note && html`<p class="empty">${FIND_ONE_EMPTY}</p>`}
    ${rows !== null && lookupGroups(rows).map((group) => html`<section class="page-section" key=${group.level}>
      <h2>${group.heading}</h2><p class="hint">${group.note}</p>
      <div class="group">${group.rows.map((row) => html`<${LookupRow} key=${row.feed_guid} row=${row} onClick=${() => onPick(row, false)} />`)}</div>
    </section>`)}
    ${feeds.length > 0 && html`<section class="page-section"><h2>Feeds</h2>
      <div class="group">${feeds.map((h) => html`<button class="link-row" type="button" key=${h.feed_guid} disabled=${disabled}
        onClick=${() => onPick(h, true)}>
        <${PersonArt} person=${{ img: h.image_url, name: h.title || '' }} />
        <span class="text"><span class="label">${h.title || h.feed_guid}</span>
          <span class="summary"><span class="dot feed" aria-hidden="true"></span>${h.artist ? `Feed · ${h.artist}` : 'Feed'}</span></span>
        <span class="go" aria-hidden="true">›</span>
      </button>`)}</div></section>`}`;
}

// The sheet "Find on musicindex": one selection fills the form (ADR 0145 item 3).
// `choices` turns the values of a feed into the choices of that feed, each with
// `values`. With many choices, the sheet asks for one of them. With `credits`,
// the values also hold the credits of the feed and its tracks (ADR 0146 item 8).
export function FindOneSheet({ open, start, choices, empty, credits = false, onPick, onClose }) {
  const ref = useSheet(open);
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="find-one-title" onClose=${onClose}>
    ${open && html`<${FindOneForm} start=${start} choices=${choices} empty=${empty} credits=${credits} onCancel=${() => ref.current.close()}
      onPick=${(values) => { onPick(values); ref.current.close(); }} />`}
  </dialog>`;
}

function FindOneForm({ start, choices, empty, credits, onPick, onCancel }) {
  const [query, setQuery] = useState(start || '');
  const search = useLookup(query, '/musicindex/search');
  const [options, setOptions] = useState(null);
  const [problem, setProblem] = useState(null);
  const [none, setNone] = useState(false);
  const open = async (row, isFeed) => {
    setProblem(null);
    setNone(false);
    try {
      const list = choices(await api(`/musicindex/values/${encodeURIComponent(row.feed_guid)}${credits ? '?credits=true' : ''}`), isFeed ? row : null);
      if (list.length === 1) onPick(list[0].values);
      else if (list.length === 0) setNone(true);
      else setOptions(list);
    } catch (e) {
      setProblem(e);
    }
  };
  return html`<header><h2 id="find-one-title">Find on musicindex</h2></header>
    <div class="body">
      ${options === null && html`<div class="group rows"><${Field} id="find-one-query" label="Name" type="search" value=${query}
        onInput=${(e) => setQuery(e.target.value)} /></div>
      <p class="hint">${FIND_ONE_HINT}</p>`}
      <${ErrorNotice} error=${problem} />
      ${none && html`<p class="hint" role="status">${empty}</p>`}
      ${options === null && html`<${SearchResults} query=${query} search=${search} onPick=${open} />`}
      ${options !== null && html`<div class="group">${options.map((option) => html`<button class="link-row" type="button" key=${option.id} onClick=${() => onPick(option.values)}>
        <span class="text"><span class="label">${option.title}</span><span class="summary">${option.line}</span></span></button>`)}</div>`}
    </div>
    <footer>${options !== null && html`<button class="button" type="button" onClick=${() => setOptions(null)}>Back</button>`}
      <button class="button" type="button" onClick=${onCancel}>Cancel</button></footer>`;
}

// The one layout of the sheets that add a value (ADR 0145): the form, the row
// of musicindex under its heading, then the lists that fill the form.

import { html, useState, useRef } from './vendor/htm-preact-standalone.module.js';
import { useSheet } from './editor.js';
import { api, ErrorNotice, Field } from './app.js';
import { LookupRow, useLookup } from './peoplepages.js';
import { foldStartsOpen, lookupGroups, FIND_EMPTY, FIND_WAIT, FIND_ONE_HINT } from './people.js';

// A list of a sheet. A long list starts closed, with its count.
function PickFold({ title, rows, onClick }) {
  return html`<details class="disclosure pick-fold" open=${foldStartsOpen(rows.length)} onClick=${onClick}>
    <summary><span class="pick-fold-title">${title}</span><span class="pick-fold-count">${rows.length}</span></summary>
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

// A row of a list that fills the form. `after` is a control beside the row.
export function PickFillRow({ title, line, lead, after, disabled, selected, onSelect }) {
  return html`<div class="link-row static pick-fill">
    <button class="link-row" type="button" disabled=${disabled} aria-current=${selected ? 'true' : undefined} onClick=${onSelect}>
      ${lead}<span class="text"><span class="label">${title}</span><span class="summary">${line}</span></span>
    </button>${after}</div>`;
}

// The sheet "Find on musicindex": one selection fills the form (ADR 0145 item 3).
// `choices` turns the values of a feed into the choices of that feed, each with
// `values`. With many choices, the sheet asks for one of them.
export function FindOneSheet({ open, start, choices, empty, onPick, onClose }) {
  const ref = useSheet(open);
  return html`<dialog ref=${ref} class="sheet" aria-labelledby="find-one-title" onClose=${onClose}>
    ${open && html`<${FindOneForm} start=${start} choices=${choices} empty=${empty} onCancel=${() => ref.current.close()}
      onPick=${(values) => { onPick(values); ref.current.close(); }} />`}
  </dialog>`;
}

function FindOneForm({ start, choices, empty, onPick, onCancel }) {
  const [query, setQuery] = useState(start || '');
  const { rows, error } = useLookup(query, '/musicindex/publishers');
  const [options, setOptions] = useState(null);
  const [problem, setProblem] = useState(null);
  const [none, setNone] = useState(false);
  const open = async (row) => {
    setProblem(null);
    setNone(false);
    try {
      const list = choices(await api(`/musicindex/publishers/${encodeURIComponent(row.feed_guid)}`));
      if (list.length === 1) onPick(list[0].values);
      else if (list.length === 0) setNone(true);
      else setOptions(list);
    } catch (e) {
      setProblem(e);
    }
  };
  const searching = query.trim() !== '';
  return html`<header><h2 id="find-one-title">Find on musicindex</h2></header>
    <div class="body">
      ${options === null && html`<div class="group rows"><${Field} id="find-one-query" label="Name" type="search" value=${query}
        onInput=${(e) => setQuery(e.target.value)} /></div>
      <p class="hint">${FIND_ONE_HINT}</p>`}
      <${ErrorNotice} error=${error || problem} />
      ${none && html`<p class="hint" role="status">${empty}</p>`}
      ${options === null && searching && rows === null && html`<p class="empty" role="status">${FIND_WAIT}</p>`}
      ${options === null && searching && rows !== null && rows.length === 0 && !error && html`<p class="empty">${FIND_EMPTY}</p>`}
      ${options === null && rows !== null && lookupGroups(rows).map((group) => html`<section class="page-section" key=${group.level}>
        <h2>${group.heading}</h2><p class="hint">${group.note}</p>
        <div class="group">${group.rows.map((row) => html`<${LookupRow} key=${row.feed_guid} row=${row} onClick=${() => open(row)} />`)}</div>
      </section>`)}
      ${options !== null && html`<div class="group">${options.map((option) => html`<button class="link-row" type="button" key=${option.id} onClick=${() => onPick(option.values)}>
        <span class="text"><span class="label">${option.title}</span><span class="summary">${option.line}</span></span></button>`)}</div>`}
    </div>
    <footer>${options !== null && html`<button class="button" type="button" onClick=${() => setOptions(null)}>Back</button>`}
      <button class="button" type="button" onClick=${onCancel}>Cancel</button></footer>`;
}

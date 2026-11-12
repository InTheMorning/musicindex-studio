// The one layout of the sheets that add a value (ADR 0145): the form, the row
// of musicindex under its heading, then the lists that fill the form.

import { html, useState, useRef, useEffect, useLayoutEffect } from './vendor/htm-preact-standalone.module.js';
import { useSheet } from './editor.js';
import { api, ErrorNotice, Field, Icon } from './app.js';
import { LookupRow, PersonArt, useLookup } from './peoplepages.js';
import { foldStartsOpen, lookupGroups, FIND_ONE_EMPTY, FIND_WAIT, FIND_ONE_HINT, showFindEmpty } from './people.js';
import { listRows, namePrefix, applyRow, listKey, NAME_PAUSE } from './fieldlist.js';
import { onPageListKey } from './listkeys.js';

// A list of rows that open a page or select a value, with one Tab stop
// (ADR 0172 item 2). A row is a `button.link-row`, or the button of a
// `PickFillRow`. `mode` 'grid' gives the keys of the tiles view (item 3).
const ROWS = ':scope > button.link-row, :scope > .link-row.static > button.link-row';
export function RowList({ children, className = 'group', selector = ROWS, mode = 'list', as = 'div' }) {
  const [current, setCurrent] = useState(0);
  const ref = useRef(null);
  const sync = (node) => {
    ref.current = node;
    if (!node) return;
    const rows = Array.from(node.querySelectorAll(selector));
    const next = rows.length ? Math.min(current, rows.length - 1) : 0;
    if (next !== current) setCurrent((value) => value === next ? value : next);
    rows.forEach((row, index) => { row.tabIndex = index === next ? 0 : -1; });
  };
  useEffect(() => { sync(ref.current); });
  const focus = (event) => {
    const rows = Array.from(event.currentTarget.querySelectorAll(selector));
    const index = rows.indexOf(event.target);
    if (index >= 0) setCurrent((value) => value === index ? value : index);
  };
  const Tag = as;
  return html`<${Tag} class=${className} ref=${sync} onfocusin=${focus}
    onKeyDown=${(event) => onPageListKey(event, current, setCurrent, selector, mode)}>${children}<//>`;
}

// The field list of a search field (ADR 0154, musicindex ADR 0052 R-QUERY-7).
// `id` is the ID of the input, `value` its text and `setValue` sets the text.
// `scope` is 'publisher' for a search of artist and label feeds only. The hook
// gives `input`, the properties of the input, and `list`, the list to show
// under it. The list opens on a click or a typed key, not on the focus alone.
// The list is a floating layer at the field, as on musicindex.org. It does not
// change the height of a sheet, and a sheet does not cut it.
export function useFieldList({ id, value, setValue, scope = null }) {
  const [open, setOpen] = useState(false);
  const [selected, setSelected] = useState(null);
  const [cursor, setCursor] = useState(null);
  const [answer, setAnswer] = useState(null);
  const [names, setNames] = useState([]);
  const text = String(value ?? '');
  const at = cursor === null ? text.length : Math.min(cursor, text.length);
  // An Escape with no list lets a search field clear its text. That clear is
  // not a typed key, so it does not open the list.
  const escaped = useRef(false);
  useEffect(() => {
    if (!open) return undefined;
    let live = true;
    const scopePart = scope ? `&scope=${scope}` : '';
    api(`/musicindex/fields?q=${encodeURIComponent(text)}&cursor=${at}${scopePart}`).then(
      (result) => { if (live) setAnswer(result); },
      () => { if (live) setAnswer(null); },
    );
    return () => { live = false; };
  }, [open, text, at, scope]);
  // The names come after the pause of the search (R-QUERY-7 item 2).
  const prefix = open ? namePrefix(answer) : null;
  useEffect(() => {
    setNames([]);
    if (prefix === null) return undefined;
    let live = true;
    const timer = setTimeout(() => api(`/musicindex/artist-names?from=${encodeURIComponent(prefix)}`).then(
      (result) => { if (live) setNames(result.names || []); },
      () => {},
    ), NAME_PAUSE);
    return () => { live = false; clearTimeout(timer); };
  }, [prefix]);
  const rows = open ? listRows(answer, names) : [];
  const error = open && answer ? answer.error : null;
  const shows = rows.length > 0 || Boolean(error);
  const place = useListPlace(id, shows);
  const readCursor = (e) => setCursor(e.target.selectionStart);
  const put = (row) => {
    const next = applyRow(text, answer && answer.edit, row.insert);
    if (!next) return;
    setValue(next.text);
    setSelected(null);
    setCursor(next.cursor);
    requestAnimationFrame(() => {
      const field = document.getElementById(id);
      if (field) { field.focus(); field.setSelectionRange(next.cursor, next.cursor); }
    });
  };
  const listId = `${id}-list`;
  const input = {
    role: 'combobox',
    autocomplete: 'off',
    'aria-autocomplete': 'list',
    'aria-expanded': shows ? 'true' : 'false',
    'aria-controls': listId,
    'aria-activedescendant': shows && selected !== null ? `${listId}-${selected}` : undefined,
    onInput: (e) => {
      setValue(e.target.value);
      if (!escaped.current) setOpen(true);
      escaped.current = false;
      setSelected(null);
      readCursor(e);
    },
    onClick: (e) => { setOpen(true); setSelected(null); readCursor(e); },
    onKeyUp: (e) => {
      // A move of the cursor clears the selection (ADR 0154 item 8).
      if (['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(e.key)) { setSelected(null); readCursor(e); }
    },
    onKeyDown: (e) => {
      // Escape closes a list with rows or with an error row, and nothing else.
      // With no list, Escape keeps its action: a search field clears its text,
      // and then the sheet closes.
      escaped.current = e.key === 'Escape' && !shows;
      if (e.key === 'Escape') {
        if (!shows) { setOpen(false); return; }
        e.preventDefault();
        e.stopPropagation();
        setOpen(false);
        setSelected(null);
        return;
      }
      const next = listKey({ open: shows && rows.length > 0, selected }, e.key, rows.length);
      if (next.act === 'search') {
        setOpen(false);
        setSelected(null);
        return;
      }
      if (next.act === 'pass') return;
      e.preventDefault();
      e.stopPropagation();
      if (next.act === 'insert') put(rows[selected]);
      setSelected(next.selected);
    },
    onBlur: () => { setOpen(false); setSelected(null); },
  };
  // A row takes no focus, so the field keeps the focus and the cursor.
  const list = shows && html`<div class="field-list" style=${place} onMouseDown=${(e) => e.preventDefault()}>
    ${error && html`<p class="field-list-note">${error}</p>`}
    ${rows.length > 0 && html`<ul id=${listId} role="listbox" aria-label="Search fields">
      ${rows.map((row, i) => html`<li id=${`${listId}-${i}`} key=${`${row.label}-${i}`} role="option"
        aria-selected=${i === selected ? 'true' : 'false'} onClick=${() => put(row)}>
        <span class="label">${row.label}</span>${(row.line || row.placeholder) && html`<span class="summary">${row.line}${row.line && row.placeholder && ' · '}${row.placeholder && html`<span class="field-placeholder">${row.placeholder}</span>`}</span>`}</li>`)}
    </ul>`}
  </div>`;
  return { input, list, open: shows };
}

// The place of the floating field list of the input `id`, while `shows` is
// true. The list opens under the field. When the window has less than
// LIST_ROOM px under the field and more space above it, the list opens above.
// The place changes after each render, a change of the window, a scroll and a
// change of the size of the sheet or the page around the field.
const LIST_ROOM = 200;
const LIST_GAP = 4;
const LIST_EDGE = 16;
function listPlace(id) {
  const field = document.getElementById(id);
  if (!field) return null;
  const box = field.getBoundingClientRect();
  const below = window.innerHeight - box.bottom - LIST_GAP - LIST_EDGE;
  const above = box.top - LIST_GAP - LIST_EDGE;
  const up = below < LIST_ROOM && above > below;
  return {
    left: `${box.left}px`,
    width: `${box.width}px`,
    maxHeight: `${Math.max(up ? above : below, 0)}px`,
    ...(up ? { bottom: `${window.innerHeight - box.top + LIST_GAP}px` } : { top: `${box.bottom + LIST_GAP}px` }),
  };
}

function useListPlace(id, shows) {
  const [place, setPlace] = useState(null);
  const last = useRef('null');
  const measure = useRef(null);
  measure.current = () => {
    const next = shows ? listPlace(id) : null;
    const key = JSON.stringify(next);
    if (key !== last.current) { last.current = key; setPlace(next); }
  };
  useLayoutEffect(() => measure.current());
  useEffect(() => {
    if (!shows) return undefined;
    const run = () => measure.current();
    const field = document.getElementById(id);
    const sizes = new ResizeObserver(run);
    sizes.observe((field && field.closest('dialog')) || document.body);
    window.addEventListener('resize', run);
    window.addEventListener('scroll', run, true);
    return () => { sizes.disconnect(); window.removeEventListener('resize', run); window.removeEventListener('scroll', run, true); };
  }, [id, shows]);
  return place;
}

// The folds that the user opened or closed, by `id`, for this visit. A fold keeps
// its state after the user opens a page from it and comes back.
const foldStates = new Map();

// A list that folds, with its count. A long list starts closed, unless `startOpen`
// says different. With `forceOpen`, it stays open, for a search of its rows.
export function PickFold({ id, title, note, lead, rows, startOpen = foldStartsOpen(rows.length), forceOpen = false, onClick, selector = ROWS, showCount = true }) {
  const open = forceOpen || (foldStates.has(id) ? foldStates.get(id) : startOpen);
  const remember = (e) => { if (id && !forceOpen) foldStates.set(id, e.currentTarget.open); };
  return html`<details class="disclosure pick-fold" open=${open} onClick=${onClick} onToggle=${remember}>
    <summary><span class="pick-fold-title">${title}</span>${showCount && html`<span class="pick-fold-count">${rows.length}</span>`}</summary>
    ${note && html`<p class="note">${note}</p>`}
    ${lead}
    <${RowList} selector=${selector}>${rows}<//>
  </details>`;
}

// `fields` is the content of the form. With no `fields`, as in an open credit
// (ADR 0169 item 1), the layout has no pinned section. `musicindex` gives the
// title, the summary and `onOpen` of its row, and an optional `note` below the
// row. Each entry of `lists` has a
// title and rows.
export function PickerLayout({ id, onSubmit, fields, notice, musicindex, lists }) {
  const form = useRef(null);
  // A selection fills the form, so the form must be in view. At narrow width it
  // scrolls with the list.
  const showForm = (e) => {
    if (form.current && e.target.closest('button.link-row:not(:disabled)')) form.current.scrollIntoView({ block: 'nearest' });
  };
  return html`<form class="body" id=${id} onSubmit=${onSubmit}>
    ${fields && html`<section class="page-section pinned-form" ref=${form}>${fields}</section>`}
    ${notice}
    <section class="page-section"><h2>From Music Index</h2>
      <div class="group"><button class="link-row" type="button" onClick=${musicindex.onOpen}>
        <span class="text"><span class="label">${musicindex.title}</span>
          <span class="summary">${musicindex.summary}</span></span>
        <span class="go" aria-hidden="true">›</span></button></div>
      ${musicindex.note && html`<p class="hint" role="status">${musicindex.note}</p>`}</section>
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
    <button class="link-row" type="button" aria-disabled=${taken ? 'true' : undefined} aria-current=${selected ? 'true' : undefined} onClick=${taken ? undefined : onSelect}>
      ${lead}<span class="text"><span class="pick-name"><span class="label">${title}</span>${taken && html`<${TakenTag} />`}</span>
        <span class="summary">${line}</span></span>
    </button>${after}</div>`;
}

// The results of the one search (ADR 0145 item 3): the artist and label feeds,
// then the other feeds. `onPick` gets the row, and for a row of the other feeds
// it gets `true` as its second argument. `listOpen` is true while the field
// list of the search field is open.
export function SearchResults({ query, search, listOpen, disabled, onPick }) {
  const { rows, feeds, note, error } = search;
  const searching = query.trim() !== '';
  return html`<${ErrorNotice} error=${error} />
    ${searching && rows === null && html`<p class="empty" role="status">${FIND_WAIT}</p>`}
    ${note && html`<p class="hint" role="status">${note.message}</p>`}
    ${feeds.length === 0 && !note && showFindEmpty({ ...search, query, listOpen }) && html`<p class="empty">${FIND_ONE_EMPTY}</p>`}
    ${rows !== null && lookupGroups(rows).map((group) => html`<section class="page-section" key=${group.level}>
      <h2>${group.heading}</h2><p class="hint">${group.note}</p>
      <${RowList}>${group.rows.map((row) => html`<${LookupRow} key=${row.feed_guid} row=${row} onClick=${() => onPick(row, false)} />`)}<//>
    </section>`)}
    ${feeds.length > 0 && html`<section class="page-section"><h2>Feeds</h2>
      <${RowList}>${feeds.map((h) => html`<button class="link-row" type="button" key=${h.feed_guid} aria-disabled=${disabled ? 'true' : undefined}
        onClick=${disabled ? undefined : () => onPick(h, true)}>
        <${PersonArt} person=${{ img: h.image_url, name: h.title || '' }} />
        <span class="text"><span class="label">${h.title || h.feed_guid}</span>
          <span class="summary"><span class="dot feed" aria-hidden="true"></span>${h.artist ? `Feed · ${h.artist}` : 'Feed'}</span></span>
        <span class="go" aria-hidden="true">›</span>
      </button>`) }<//></section>`}`;
}

// The sheet "Find on musicindex": one selection fills the form (ADR 0145 item 3).
// `choices` turns the values of a feed into the choices of that feed, each with
// `values`. With many choices, the sheet asks for one of them. With `credits`,
// the values also hold the credits of the feed and its tracks (ADR 0146 item 8).
export function FindOneSheet({ open, start, choices, empty, credits = false, onPick, onClose }) {
  const ref = useSheet(open);
  return html`<dialog ref=${ref} class="sheet search-sheet" aria-labelledby="find-one-title" onClose=${onClose}>
    ${open && html`<${FindOneForm} start=${start} choices=${choices} empty=${empty} credits=${credits} onCancel=${() => ref.current.close()}
      onPick=${(values) => { onPick(values); ref.current.close(); }} />`}
  </dialog>`;
}

function FindOneForm({ start, choices, empty, credits, onPick, onCancel }) {
  const [query, setQuery] = useState(start || '');
  const search = useLookup(query, '/musicindex/search');
  const fields = useFieldList({ id: 'find-one-query', value: query, setValue: setQuery });
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
  return html`<header><h2 id="find-one-title">Find on Music Index</h2></header>
    <div class="body">
      ${options === null && html`<section class="page-section pinned-form"><div class="group rows"><${Field} id="find-one-query" label="Name" type="search" value=${query}
        ...${fields.input}>${fields.list}<//></div>
      <p class="hint">${FIND_ONE_HINT}</p></section>`}
      <${ErrorNotice} error=${problem} />
      ${none && html`<p class="hint" role="status">${empty}</p>`}
      ${options === null && html`<${SearchResults} query=${query} search=${search} listOpen=${fields.open} onPick=${open} />`}
      ${options !== null && html`<div class="group">${options.map((option) => html`<button class="link-row" type="button" key=${option.id} onClick=${() => onPick(option.values)}>
        <span class="text"><span class="label">${option.title}</span><span class="summary">${option.line}</span></span></button>`)}</div>`}
    </div>
    <footer>${options !== null && html`<button class="button" type="button" onClick=${() => setOptions(null)}>Back</button>`}
      <button class="button" type="button" onClick=${onCancel}>Cancel</button></footer>`;
}

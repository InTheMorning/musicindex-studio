// The key model for lists and trees (ADR 0172, musicindex ADR 0061), and for
// a grid of tiles (musicindex ADR 0078 R-GRIDKEY-1).
// The model has no Preact dependency. The tree handler reads the live DOM.

// Each row has a kind: 'page', 'expandable' or 'tree'. A row can name its
// parent by row index and can set `expanded`, `expandable` and `visible`.
// In mode 'grid', `options.lines` gives the row indexes of each line of tiles.
export function createListModel(rows, current = 0, options = {}) {
  const items = (rows || []).map((row) => ({
    kind: row.kind,
    parent: Number.isInteger(row.parent) ? row.parent : null,
    expanded: Boolean(row.expanded),
    expandable: row.expandable === undefined ? row.kind === 'tree' || row.kind === 'expandable' : Boolean(row.expandable),
    visible: row.visible !== false,
  }));
  const selected = items.length === 0 ? null : Math.max(0, Math.min(current, items.length - 1));
  const lines = options.lines || [items.map((_, index) => index)];
  return { rows: items, current: selected, mode: options.mode || 'list', lines };
}

// Divide tiles into lines from the top of each tile, in the order of the list
// (musicindex ADR 0078 R-GRIDKEY-1 item 5).
export function gridLines(tops) {
  const lines = [];
  let lineTop = null;
  tops.forEach((top, index) => {
    if (lineTop === null || Math.abs(top - lineTop) > 1) {
      lines.push([]);
      lineTop = top;
    }
    lines[lines.length - 1].push(index);
  });
  return lines;
}

// Apply one key. The returned action is 'none', 'page' or 'toggle'.
export function listKey(model, key) {
  const rows = model.rows;
  const current = model.current;
  const row = current === null ? null : rows[current];
  const unchanged = { model, action: { type: 'none' } };

  if (!row) return unchanged;

  const move = (index) => ({
    model: { ...model, current: index },
    action: { type: 'none' },
  });
  const toggle = (expanded) => {
    const nextRows = rows.map((item, index) => index === current ? { ...item, expanded } : item);
    return {
      model: { ...model, rows: nextRows },
      action: { type: 'toggle', row: current, expanded },
    };
  };
  const nextVisible = (start, direction) => {
    let index = start;
    while (index >= 0 && index < rows.length && !rows[index].visible) index += direction;
    return index >= 0 && index < rows.length ? index : current;
  };
  const firstVisible = () => rows.findIndex((item) => item.visible);
  const lastVisible = () => rows.findLastIndex((item) => item.visible);
  // Up and Down go to the same column of the next line, or to its last tile.
  // On the first line and the last line they do not move (R-GRIDKEY-1 item 3).
  const lineMove = (direction) => {
    const at = model.lines.findIndex((line) => line.includes(current));
    const target = at < 0 ? undefined : model.lines[at + direction];
    if (!target || target.length === 0) return unchanged;
    const column = model.lines[at].indexOf(current);
    return move(target[Math.min(column, target.length - 1)]);
  };

  switch (key) {
    case 'ArrowDown':
      if (model.mode === 'grid') return lineMove(1);
      return move(nextVisible(current + 1, 1));
    case 'ArrowUp':
      if (model.mode === 'grid') return lineMove(-1);
      return move(nextVisible(current - 1, -1));
    case 'ArrowRight':
      if (model.mode === 'grid') return move(Math.min(rows.length - 1, current + 1));
      return row.expandable && !row.expanded ? toggle(true) : unchanged;
    case 'ArrowLeft':
      if (model.mode === 'grid') return move(Math.max(0, current - 1));
      if (row.expandable && row.expanded) return toggle(false);
      if (row.parent !== null) return move(row.parent);
      return unchanged;
    case 'Home':
      return move(firstVisible() < 0 ? current : firstVisible());
    case 'End':
      return move(lastVisible() < 0 ? current : lastVisible());
    case 'Enter':
      if (row.expandable) return toggle(!row.expanded);
      return { model, action: { type: 'page', row: current } };
    default:
      return unchanged;
  }
}

// Handle keys for a list of page or value buttons. The current row remains
// selected when focus leaves the list.
export function onPageListKey(event, current, setCurrent, selector = ':scope > button.link-row', mode = 'list') {
  const list = event.currentTarget;
  const elements = Array.from(list.querySelectorAll(selector));
  const indexByElement = new Map(elements.map((element, index) => [element, index]));
  const active = indexByElement.get(document.activeElement);
  if (active === undefined || event.target !== document.activeElement) return;
  const lines = mode === 'grid'
    ? gridLines(elements.map((element) => element.getBoundingClientRect().top))
    : undefined;
  const result = listKey(createListModel(elements.map(() => ({ kind: 'page' })), active, { mode, lines }), event.key);
  const next = result.model.current;
  if (next === active) {
    if (result.action.type === 'page') {
      event.preventDefault();
      if (elements[active]?.getAttribute('aria-disabled') !== 'true') elements[active]?.click();
    }
    return;
  }
  event.preventDefault();
  setCurrent(next);
  elements.forEach((element, index) => { element.tabIndex = index === next ? 0 : -1; });
  elements[next]?.focus();
}

// Handle one tree key from the tree's current DOM state. The caller keeps the
// current row so Tab returns to the same row after focus leaves the tree.
export function onTreeKey(event, current, setCurrent) {
  const tree = event.currentTarget;
  const elements = Array.from(tree.querySelectorAll('[role="treeitem"]'))
    .filter((element) => element.closest('[role="tree"]') === tree);
  const indexByElement = new Map(elements.map((element, index) => [element, index]));
  const rows = elements.map((element) => {
    const parent = Number(element.dataset.listParent);
    let ancestor = element.parentElement;
    let visible = true;
    while (ancestor && ancestor !== tree) {
      if (ancestor.tagName === 'DETAILS' && !ancestor.open
          && ancestor.querySelector(':scope > summary') !== element) visible = false;
      ancestor = ancestor.parentElement;
    }
    return {
      kind: element.dataset.listKind || 'page',
      parent: Number.isInteger(parent) && parent >= 0 ? parent : null,
      expanded: element.getAttribute('aria-expanded') === 'true',
      expandable: element.dataset.listExpandable === 'true',
      visible,
    };
  });
  const active = indexByElement.get(document.activeElement);
  const selected = active === undefined ? current : active;
  const result = listKey(createListModel(rows, selected), event.key);
  const next = result.model.current;
  const moved = next !== selected;
  if (!moved && result.action.type === 'none') return;

  event.preventDefault();
  if (moved) {
    setCurrent(next);
    elements.forEach((element, index) => { element.tabIndex = index === next ? 0 : -1; });
    elements[next]?.focus();
    return;
  }

  const element = elements[result.action.row];
  if (result.action.type === 'toggle') element?.click();
  if (result.action.type === 'page') element?.click();
}

// Handle one list key. The caller keeps the current expandable row so Tab
// returns to the same row after focus leaves the list.
export function onListKey(event, current, setCurrent) {
  const row = event.currentTarget.querySelector(':scope > summary[data-list-row]');
  const list = row?.closest('.group');
  if (!list) return;
  const elements = Array.from(list.querySelectorAll(':scope > details > summary[data-list-row]'));
  if (elements.length === 0) return;
  const indexByElement = new Map(elements.map((element, index) => [element, index]));
  const active = indexByElement.get(document.activeElement);
  if (active === undefined) return;
  const selected = active;
  const rows = elements.map((element) => ({
    kind: 'expandable', expandable: true,
    expanded: element.getAttribute('aria-expanded') === 'true',
  }));
  const result = listKey(createListModel(rows, selected), event.key);
  const next = result.model.current;
  const moved = next !== selected;
  if (!moved && result.action.type === 'none') return;

  event.preventDefault();
  if (moved) {
    setCurrent(next);
    elements.forEach((element, index) => { element.tabIndex = index === next ? 0 : -1; });
    elements[next]?.focus();
    return;
  }

  elements[result.action.row]?.click();
}

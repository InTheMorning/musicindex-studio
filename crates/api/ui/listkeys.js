// The key model for lists and trees (ADR 0172, musicindex ADR 0061).
// The model has no Preact dependency. The tree handler reads the live DOM.

// Each row has a kind: 'page', 'expandable' or 'tree'. A row can name its
// parent by row index and can set `expanded`, `expandable` and `visible`.
export function createListModel(rows, current = 0) {
  const items = (rows || []).map((row) => ({
    kind: row.kind,
    parent: Number.isInteger(row.parent) ? row.parent : null,
    expanded: Boolean(row.expanded),
    expandable: row.expandable === undefined ? row.kind === 'tree' || row.kind === 'expandable' : Boolean(row.expandable),
    visible: row.visible !== false,
  }));
  const selected = items.length === 0 ? null : Math.max(0, Math.min(current, items.length - 1));
  return { rows: items, current: selected };
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

  switch (key) {
    case 'ArrowDown':
      return move(nextVisible(current + 1, 1));
    case 'ArrowUp':
      return move(nextVisible(current - 1, -1));
    case 'Home':
      return move(firstVisible() < 0 ? current : firstVisible());
    case 'End':
      return move(lastVisible() < 0 ? current : lastVisible());
    case 'ArrowRight':
      return row.expandable && !row.expanded
        ? toggle(true)
        : unchanged;
    case 'ArrowLeft':
      if (row.expandable && row.expanded) return toggle(false);
      if (row.parent !== null) return move(row.parent);
      return unchanged;
    case 'Enter':
      if (row.expandable) return toggle(!row.expanded);
      return { model, action: { type: 'page', row: current } };
    default:
      return unchanged;
  }
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

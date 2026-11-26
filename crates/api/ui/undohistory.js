// The undo history of the edits of one page (ADR 0227 items 1, 4, 6 and
// 11). `record` takes the state before an edit, the key of the field or
// null, and the time in ms. Typing in one field is one step until a pause.
// `recordOperation` adds a step for an operation that saves through its own
// route (ADR 0228 items 2, 3 and 7). Such a step holds the functions `undo`
// and `redo`. `undo` and `redo` of the history give that step as it is, and
// the page runs it with `perform`.
//
// A Catalog step has no `redo` (ADR 0254 item 37).
class OperationStep {
  constructor(undo, redo) {
    this.undo = undo;
    this.redo = redo;
  }
}

export const isOperationStep = (value) => value instanceof OperationStep;

export function makeHistory({ limit = 100, pause = 1000 } = {}) {
  const undoSteps = [];
  const redoSteps = [];
  let lastField = null;
  let lastTime = null;
  let canUndo = false;
  let canRedo = false;
  return {
    record(before, field, now) {
      redoSteps.length = 0;
      if (field === null || field !== lastField || lastTime === null || now - lastTime > pause) {
        undoSteps.push(before);
        if (undoSteps.length > limit) undoSteps.shift();
      }
      lastField = field;
      lastTime = now;
      canUndo = undoSteps.length > 0;
      canRedo = false;
    },
    recordOperation({ undo, redo }) {
      redoSteps.length = 0;
      const step = new OperationStep(undo, redo);
      undoSteps.push(step);
      if (undoSteps.length > limit) undoSteps.shift();
      lastField = null;
      lastTime = null;
      canUndo = true;
      canRedo = false;
      return step;
    },
    // Remove an operation that a different control completed.
    forgetOperation(step) {
      for (const steps of [undoSteps, redoSteps]) {
        const at = steps.indexOf(step);
        if (at >= 0) steps.splice(at, 1);
      }
      canUndo = undoSteps.length > 0;
      canRedo = redoSteps.length > 0;
    },
    // Runs one side of an operation step. When the route fails, the step
    // goes from the history and the error goes to the caller (ADR 0228 item 5).
    async perform(step, direction) {
      try {
        await step[direction]();
      } catch (error) {
        for (const steps of [undoSteps, redoSteps]) {
          const at = steps.indexOf(step);
          if (at >= 0) steps.splice(at, 1);
        }
        canUndo = undoSteps.length > 0;
        canRedo = redoSteps.length > 0;
        throw error;
      }
    },
    undo(current) {
      if (!undoSteps.length) return null;
      // The next edit starts a new step, also in the same field.
      lastField = null;
      lastTime = null;
      const previous = undoSteps.pop();
      if (!isOperationStep(previous) || previous.redo) {
        redoSteps.push(isOperationStep(previous) ? previous : current);
      }
      canUndo = undoSteps.length > 0;
      canRedo = redoSteps.length > 0;
      return previous;
    },
    redo(current) {
      if (!redoSteps.length) return null;
      // The next edit starts a new step, also in the same field.
      lastField = null;
      lastTime = null;
      const next = redoSteps.pop();
      undoSteps.push(isOperationStep(next) ? next : current);
      canUndo = true;
      canRedo = redoSteps.length > 0;
      return next;
    },
    clear() {
      undoSteps.length = 0;
      redoSteps.length = 0;
      lastField = null;
      lastTime = null;
      canUndo = false;
      canRedo = false;
    },
    get canUndo() { return canUndo; },
    get canRedo() { return canRedo; },
  };
}

// The action of a key: 'undo', 'redo' or null (ADR 0227 item 3). Cmd on a
// Mac takes the place of Ctrl.
export function undoKey(event) {
  if (event.altKey) return null;
  const key = event.key.toLowerCase();
  if ((event.ctrlKey || event.metaKey) && key === 'z') return event.shiftKey ? 'redo' : 'undo';
  if (event.ctrlKey && key === 'y' && !event.metaKey && !event.shiftKey) return 'redo';
  return null;
}

// The key of the text field that has the focus, or null (ADR 0227 item 4).
// Typing in that field is one step until a pause.
export function focusedField(element = globalThis.document?.activeElement) {
  if (!element || !/^(INPUT|TEXTAREA)$/.test(element.tagName)) return null;
  return element.id || element.name || null;
}

// ---- The keys of list entries (ADR 0022 item 5) ----
// The key of an entry is its position in the draft of the server. Each
// entry also has a client ID (`_cid`) for the life of the page.

// The position of each entry in its list, by client ID. `at` gives the
// value to keep for an entry. The default is its position in `value`.
export function positions(value, found = new Map(), at = (entry, index) => index) {
  if (Array.isArray(value)) {
    value.forEach((entry, index) => {
      if (entry && entry._cid) found.set(entry._cid, at(entry, index));
      positions(entry, found, at);
    });
  } else if (value && typeof value === 'object') {
    for (const field of Object.values(value)) positions(field, found, at);
  }
  return found;
}

// Give each entry that the server has the key of its position. An entry that
// the user added during the save has no key yet.
export function rekey(value, found) {
  if (Array.isArray(value)) {
    return value.map((entry) => {
      if (!entry || typeof entry !== 'object' || !entry._cid) return entry;
      const next = rekey(entry, found);
      const key = found.get(entry._cid);
      if (key === undefined) delete next.key; else next.key = key;
      return next;
    });
  }
  if (value && typeof value === 'object') {
    const copy = {};
    for (const [name, field] of Object.entries(value)) copy[name] = rekey(field, found);
    return copy;
  }
  return value;
}

// The key of each entry in the draft of the server, by client ID. The draft
// hook takes it from each loaded model. After a save, `positions` of the
// sent model gives the new keys (ADR 0022 item 5).
export const keysOf = (model) => positions(model, new Map(), (entry) => entry.key);

// An undo or a redo step holds a model with the keys of an earlier draft.
// After a move or a removal, the same key names a different entry of the
// server draft. Give each entry the key of its client ID in the server draft
// (`serverKeys`). An entry that a save removed (`removed`) gets no key, so
// the core writes it as a new element. An entry with an unknown client ID
// keeps its own key.
export function restoreKeys(value, serverKeys, removed) {
  if (Array.isArray(value)) {
    return value.map((entry) => {
      if (!entry || typeof entry !== 'object' || !entry._cid) return entry;
      const next = restoreKeys(entry, serverKeys, removed);
      if (serverKeys.has(entry._cid)) {
        const key = serverKeys.get(entry._cid);
        if (key === undefined) delete next.key; else next.key = key;
      } else if (removed.has(entry._cid)) {
        delete next.key;
      }
      return next;
    });
  }
  if (value && typeof value === 'object') {
    const copy = {};
    for (const [name, field] of Object.entries(value)) copy[name] = restoreKeys(field, serverKeys, removed);
    return copy;
  }
  return value;
}

// Give each entry of a loaded model the client ID of the entry with the same
// key in `old`. Use it only when `old` holds the same server draft, so that
// the undo steps of an earlier visit match the entries again (ADR 0227 item 7).
export function carryCids(value, old) {
  if (Array.isArray(value)) {
    const byKey = new Map((Array.isArray(old) ? old : [])
      .filter((entry) => entry && typeof entry === 'object' && entry.key !== undefined)
      .map((entry) => [entry.key, entry]));
    return value.map((entry) => {
      if (!entry || typeof entry !== 'object' || !entry._cid) return entry;
      const match = entry.key === undefined ? undefined : byKey.get(entry.key);
      if (!match) return entry;
      return { ...carryCids(entry, match), _cid: match._cid };
    });
  }
  if (value && typeof value === 'object') {
    const copy = {};
    for (const [name, field] of Object.entries(value)) {
      copy[name] = old && typeof old === 'object' ? carryCids(field, old[name]) : field;
    }
    return copy;
  }
  return value;
}

// The content of an entry without its client IDs and keys.
function content(entry) {
  return JSON.stringify(entry, (name, field) => (name === '_cid' || name === 'key' ? undefined : field));
}

// Give each entry of a loaded model the client ID of the entry in `old` with
// the same content. Use it when `old` holds an earlier server draft, for
// example after a route changed one field. An entry with changed content, or
// with content that more than one entry has, keeps its new client ID.
export function carryCidsByContent(value, old) {
  if (Array.isArray(value)) {
    const byContent = new Map();
    for (const entry of Array.isArray(old) ? old : []) {
      if (!entry || typeof entry !== 'object' || !entry._cid) continue;
      const text = content(entry);
      byContent.set(text, byContent.has(text) ? null : entry);
    }
    const seen = new Map();
    for (const entry of value) {
      if (entry && typeof entry === 'object' && entry._cid) {
        const text = content(entry);
        seen.set(text, (seen.get(text) || 0) + 1);
      }
    }
    return value.map((entry) => {
      if (!entry || typeof entry !== 'object' || !entry._cid) return entry;
      const text = content(entry);
      const match = seen.get(text) === 1 ? byContent.get(text) : null;
      if (!match) return carryCidsByContent(entry, null);
      return { ...carryCidsByContent(entry, match), _cid: match._cid };
    });
  }
  if (value && typeof value === 'object') {
    const copy = {};
    for (const [name, field] of Object.entries(value)) {
      copy[name] = carryCidsByContent(field, old && typeof old === 'object' ? old[name] : undefined);
    }
    return copy;
  }
  return value;
}

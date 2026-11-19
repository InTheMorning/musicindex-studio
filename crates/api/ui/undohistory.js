// The undo history of the edits of one page (ADR 0227 items 1, 4, 6 and
// 11). `record` takes the state before an edit, the key of the field or
// null, and the time in ms. Typing in one field is one step until a pause.
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
    undo(current) {
      if (!undoSteps.length) return null;
      // The next edit starts a new step, also in the same field.
      lastField = null;
      lastTime = null;
      redoSteps.push(current);
      const previous = undoSteps.pop();
      canUndo = undoSteps.length > 0;
      canRedo = true;
      return previous;
    },
    redo(current) {
      if (!redoSteps.length) return null;
      // The next edit starts a new step, also in the same field.
      lastField = null;
      lastTime = null;
      undoSteps.push(current);
      const next = redoSteps.pop();
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

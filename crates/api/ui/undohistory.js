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

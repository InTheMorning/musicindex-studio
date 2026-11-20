// The undo step of a delete (ADR 0228 items 3 to 5 and 7). Delete Person and
// Remove from Collection add one step to a history of the app. The page that
// opens after the delete shows that step in its bar, as **Undo** and **Redo**.
import { html, useContext, useRef, useState, createContext } from './vendor/htm-preact-standalone.module.js';
import { makeHistory } from './undohistory.js';

const DeleteUndoContext = createContext(null);

// The status texts of the bar (ADR 0228 item 3).
export const undoNoticeText = Object.freeze({
  personDeleted: (name) => `${name} is deleted.`,
  feedRemoved: (title) => `${title} is removed from your collection.`,
});

// The model of the bar: the status text, and if **Undo** and **Redo** work.
// `changed` gets no argument and runs after each change of the model. `done`
// runs after a successful undo or redo, so that the pages read their data
// again. The step stays while the app is open. A new delete replaces it
// (item 4). When the core fails to undo or redo, the model gives the error
// text of the core, and the step goes (item 5).
export function makeDeleteUndo(api, { changed = () => {}, done = () => {} } = {}) {
  const history = makeHistory({ limit: 1 });
  let text = '';
  let failure = null;
  const model = () => ({
    text: failure !== null ? failure : (history.canUndo ? text : ''),
    error: failure !== null,
    canUndo: history.canUndo,
    canRedo: history.canRedo,
  });
  const run = async (direction) => {
    const step = direction === 'undo' ? history.undo(null) : history.redo(null);
    if (step === null) return;
    failure = null;
    changed();
    try {
      await history.perform(step, direction);
      done();
    } catch (error) {
      failure = (error && error.message) || 'Studio could not undo this action.';
    }
    changed();
  };
  return {
    model,
    // `redo` sends the same delete again. It asks for no confirmation.
    record(statusText, redo) {
      history.clear();
      failure = null;
      text = statusText;
      history.recordOperation({
        undo: () => api('/undo', { method: 'POST' }),
        redo,
      });
      changed();
    },
    // A delete with no core record removes the step.
    clear() {
      history.clear();
      failure = null;
      changed();
    },
    undo: () => run('undo'),
    redo: () => run('redo'),
  };
}

// The `model`, `record`, `undo` and `redo` of the delete step.
export function useDeleteUndo() {
  return useContext(DeleteUndoContext);
}

// The actions that `useDeleteUndo()` gives with `model`. A page calls only
// these. `clear` was missing once, and **Remove from Collection** then failed
// after the remove.
export const DELETE_UNDO_ACTIONS = Object.freeze(['record', 'undo', 'redo', 'clear']);

// Holds the one delete step of the app.
// The window event after a successful undo or redo of a delete step. The
// shell and the pages that show data from the core load it again (ADR 0228
// item 4).
export const UNDO_COMPLETE = 'studio-undo-complete';

export function DeleteUndoProvider({ api, children }) {
  const [model, setModel] = useState({ text: '', error: false, canUndo: false, canRedo: false });
  const control = useRef(null);
  if (control.current === null) {
    control.current = makeDeleteUndo(api, {
      changed: () => setModel(control.current.model()),
      done: () => window.dispatchEvent(new Event(UNDO_COMPLETE)),
    });
  }
  // Each action of the control, made one time, so that a page that calls
  // it gets the same function at each render.
  const actions = useRef(null);
  if (actions.current === null) {
    actions.current = Object.fromEntries(DELETE_UNDO_ACTIONS.map((name) => [name, (...args) => control.current[name](...args)]));
  }
  const value = { model, ...actions.current };
  return html`<${DeleteUndoContext.Provider} value=${value}>${children}<//>`;
}

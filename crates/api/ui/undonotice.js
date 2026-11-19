// The Undo notice after an immediate local operation (ADR 0227 items 13 to
// 19). The shell shows one notice at a time, for 10 seconds.
import { html, useCallback, useContext, useEffect, useRef, useState, createContext } from './vendor/htm-preact-standalone.module.js';

const UndoNoticeContext = createContext(null);

// The texts of the notice (ADR 0227 item 13).
export const undoNoticeText = Object.freeze({
  personDeleted: (name) => `${name} is deleted.`,
  feedRemoved: (title) => `${title} is removed from your collection.`,
  kindChanged: (kind) => `The kind is now ${kind || 'not set'}.`,
  nicknameChanged: (nickname) => `The nickname is now ${nickname || 'not set'}.`,
});

// The `show` and `dismiss` functions of the shell notice.
export function useUndoNotice() {
  return useContext(UndoNoticeContext);
}

// Holds the notice of the shell. A new notice replaces the old one (item 15).
// When the notice ends, the core deletes its undo record (item 16). When an
// undo fails, the notice shows the error and no Undo (item 19).
export function UndoNoticeProvider({ api, children }) {
  const [notice, setNotice] = useState(null);
  const timer = useRef(null);
  const current = useRef(null);
  const clear = useCallback(async (discard = true) => {
    clearTimeout(timer.current);
    timer.current = null;
    current.current = null;
    setNotice(null);
    if (discard) {
      try { await api('/undo', { method: 'DELETE' }); } catch (_) {}
    }
  }, [api]);
  const show = useCallback((text, undo = null, duration = 10000) => {
    clearTimeout(timer.current);
    current.current = { text, undo };
    setNotice(current.current);
    timer.current = duration === null ? null : setTimeout(() => { clear(true); }, duration);
  }, [clear]);
  const dismiss = useCallback(() => {
    clearTimeout(timer.current);
    timer.current = null;
    current.current = null;
    setNotice(null);
  }, []);
  const runUndo = async () => {
    const active = current.current;
    if (!active || !active.undo) return;
    try {
      await active.undo();
      window.dispatchEvent(new Event('studio-undo-complete'));
      await clear(true);
    } catch (error) {
      clearTimeout(timer.current);
      current.current = { text: error.message || 'Studio could not undo this action.', undo: null };
      setNotice(current.current);
      timer.current = setTimeout(() => { clear(true); }, 10000);
    }
  };
  useEffect(() => () => {
    clearTimeout(timer.current);
    if (current.current) api('/undo', { method: 'DELETE' }).catch(() => {});
  }, [api]);
  const value = { show, dismiss };
  return html`<${UndoNoticeContext.Provider} value=${value}>${children}
    ${notice && html`<div class="notice undo-notice" role="status"><span>${notice.text}</span>
      ${notice.undo && html`<button class="button action" type="button" onClick=${runUndo}>Undo</button>`}</div>`}<//>`;
}

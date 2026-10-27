// The draft of an album feed (task 011, ADR 0022; task 014b, ADR 0029). The
// draft holds the typed model of the feed and saves it to the API on each
// change. Each list entry carries a client ID for the life of the page.
import { useState, useEffect, useRef, useCallback } from './vendor/htm-preact-standalone.module.js';
import { api } from './app.js';
import { withSavedCover } from './cover.js';

// ---- Client IDs and keys ----
// Each list entry gets a client ID (`_cid`) for the life of the page. After a
// save, the position of an entry in the sent list is its new key (ADR 0022
// item 5). The client ID finds that position, also when the user changed the
// model during the save.

let nextCid = 1;

export function withCids(value) {
  if (Array.isArray(value)) {
    return value.map((entry) => (entry && typeof entry === 'object' ? { ...withCids(entry), _cid: nextCid++ } : entry));
  }
  if (value && typeof value === 'object') {
    const copy = {};
    for (const [name, field] of Object.entries(value)) copy[name] = withCids(field);
    return copy;
  }
  return value;
}

// The model without client IDs, as the API gets it.
export function strip(model) {
  return JSON.parse(JSON.stringify(model, (name, value) => (name === '_cid' ? undefined : value)));
}

// The position of each entry in its list, by client ID.
export function positions(value, found = new Map()) {
  if (Array.isArray(value)) {
    value.forEach((entry, index) => {
      if (entry && entry._cid) found.set(entry._cid, index);
      positions(entry, found);
    });
  } else if (value && typeof value === 'object') {
    for (const field of Object.values(value)) positions(field, found);
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

// The server sets two fields of a track: the GUID of a new track (ADR 0022
// item 7) and the track number (ADR 0024). Copy them into the local model, or
// the next save would change them back.
export function copyServerFields(local, saved, found) {
  return {
    ...local,
    items: local.items.map((item) => {
      const index = found.get(item._cid);
      const from = index === undefined ? null : saved.items[index];
      if (!from) return item;
      return { ...item, guid: item.guid || from.guid, episode: from.episode };
    }),
  };
}

// ---- The draft and its automatic save ----

export const SAVE_DELAY = 700;

// The long status of the draft, for the review sheet (ADR 0029 item 2). The
// toolbar shows a shorter status of its own.
export const STATUS = {
  loading: 'Loading the draft…',
  waiting: 'Changes not saved yet…',
  saving: 'Saving the draft…',
  saved: 'The draft is saved. The hosted file does not change until you write it.',
  failed: 'Studio could not save the draft.',
};

// The draft of one feed. A feed ID of `null` means that the selected feed
// needs no draft now, for example a feed that is not an album (ADR 0029
// item 4: the shell holds one draft and gives it to the editor).
export function useDraft(feedId) {
  const [model, setModel] = useState(null);
  const [changed, setChanged] = useState(false);
  // Whether the core says that the feed is hidden from apps (ADR 0065 item 9).
  const [hidden, setHidden] = useState(false);
  // Each address field that is not correct, by its model path (ADR 0023).
  const [bad, setBad] = useState([]);
  const [status, setStatus] = useState(feedId === null ? 'idle' : 'loading');
  const [error, setError] = useState(null);
  // The number of user edits. After an edit, a diff is not current.
  const [edits, setEdits] = useState(0);
  const current = useRef(null);
  const revision = useRef(null);
  const dirty = useRef(false);
  const inflight = useRef(null);
  const timer = useRef(null);
  // The feed of the current draft. A load for a different feed that ends
  // late does not show its draft.
  const active = useRef(feedId);
  active.current = feedId;

  const show = (draft) => {
    clearTimeout(timer.current);
    current.current = withCids(draft.model);
    revision.current = draft.revision;
    dirty.current = false;
    setModel(current.current);
    setChanged(draft.changed);
    setHidden(draft.hidden);
    setBad(draft.bad_addresses);
    setStatus('saved');
    setError(null);
  };

  const load = useCallback(async () => {
    if (feedId === null) {
      setStatus('idle');
      return;
    }
    setStatus('loading');
    try {
      const draft = await api(`/feeds/${feedId}/draft`);
      if (active.current === feedId) show(draft);
    } catch (e) {
      if (active.current !== feedId) return;
      setError(e);
      setStatus('failed');
    }
  }, [feedId]);

  useEffect(() => {
    // A new feed ID means a different draft, or no draft at all. Clear the
    // old model first, so that the page never shows the draft of the feed
    // before this one while the new draft loads.
    clearTimeout(timer.current);
    current.current = null;
    revision.current = null;
    dirty.current = false;
    setModel(null);
    setChanged(false);
    setHidden(false);
    setBad([]);
    setError(null);
    setEdits(0);
    load();
    return () => clearTimeout(timer.current);
  }, [load]);

  const saveOnce = async () => {
    dirty.current = false;
    const sent = current.current;
    setStatus('saving');
    const saved = await api(`/feeds/${feedId}/draft`, {
      method: 'PUT',
      body: { revision: revision.current, model: strip(sent) },
    });
    // The server keeps the save. When the user selected a different feed
    // during the save, this hook does not show the result.
    if (active.current !== feedId) return;
    const found = positions(sent);
    revision.current = saved.revision;
    const copied = copyServerFields(rekey(current.current, found), saved.model, found);
    current.current = withSavedCover(copied, sent, saved.model);
    setModel(current.current);
    setChanged(saved.changed);
    setHidden(saved.hidden);
    setBad(saved.bad_addresses);
  };

  // Send each change that is not saved. Only one save runs at a time.
  const flush = useCallback(async () => {
    clearTimeout(timer.current);
    while (dirty.current || inflight.current) {
      if (inflight.current) {
        await inflight.current.catch(() => {});
        continue;
      }
      inflight.current = saveOnce();
      try {
        await inflight.current;
      } catch (e) {
        dirty.current = true;
        setError(e);
        setStatus('failed');
        return false;
      } finally {
        inflight.current = null;
      }
    }
    setError(null);
    setStatus('saved');
    return true;
  }, [feedId]);

  const update = useCallback((change) => {
    current.current = change(current.current);
    dirty.current = true;
    setModel(current.current);
    setEdits((n) => n + 1);
    setStatus('waiting');
    clearTimeout(timer.current);
    timer.current = setTimeout(flush, SAVE_DELAY);
  }, [flush]);

  return { model, changed, hidden, bad, status, error, edits, update, flush, load, show };
}

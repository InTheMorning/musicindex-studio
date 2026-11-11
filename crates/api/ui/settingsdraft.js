// The Settings draft shared by its tabs (ADR 0166 items 7 to 11).
import { pendingStatus } from './people.js';

// The tab of each setting of the draft (ADR 0166 items 1 and 7), for the
// pencil on the tab of a pending change (item 9).
export const SETTING_TABS = {
  scale: 'general', lock: 'general', window: 'general', outside: 'general',
  nostr: 'nostr', libraryMax: 'data', libraryMin: 'data',
};

// A draft with its saved values. A panel adds the saved value of its setting
// at its first change.
export function makeSettingsDraft(saved = {}) {
  return { saved: { ...saved }, values: { ...saved }, didSave: false };
}

export function changeSetting(draft, key, value) {
  return { ...draft, values: { ...draft.values, [key]: value }, didSave: false };
}

// The settings with a change that is not saved.
export function pendingSettings(draft) {
  return Object.keys(draft.values).filter((key) => draft.values[key] !== draft.saved[key]);
}

// The state of the bar and of the marks (ADR 0166 items 8, 9 and 18): the
// texts of the page of a person, Discard and Save, the tabs with a pending
// change and the cause of each value that cannot be saved.
export function settingsDraftView(draft) {
  const pending = pendingSettings(draft);
  const causes = settingCauses(draft);
  const invalid = pending.filter((key) => causes[key]);
  return { pending, count: pending.length, status: pending.length ? pendingStatus(pending.length) : (draft.didSave ? 'Saved' : pendingStatus(0)),
    causes, discardDisabled: pending.length === 0, saveDisabled: pending.length === 0 || invalid.length > 0,
    tabs: [...new Set(pending.map((key) => SETTING_TABS[key]))], rows: pending };
}

// The cause of each pending value that Studio cannot save (ADR 0166 item 17).
export function settingCauses(draft) {
  const causes = {};
  for (const key of ['libraryMax', 'libraryMin']) {
    if (draft.values[key] !== draft.saved[key] && !(Number(draft.values[key]) > 0)) causes[key] = 'Enter a number greater than 0.';
  }
  const relay = draft.values.nostr;
  if (relay !== draft.saved.nostr && relay !== '' && !/^wss?:\/\//i.test(relay.trim())) causes.nostr = 'A relay address starts with wss:// or ws://.';
  return causes;
}

// Save each pending setting through its route (ADR 0166 item 12). The two
// library limits go in one request. A refused setting stays pending with its
// cause, and the other settings keep their save.
export async function saveSettings(draft, routes) {
  const causes = settingCauses(draft);
  const pending = pendingSettings(draft).filter((key) => !causes[key]);
  const errors = { ...causes };
  const libraryPending = pending.filter((key) => key === 'libraryMax' || key === 'libraryMin');
  const groups = pending.filter((key) => !libraryPending.includes(key)).map((key) => [key]);
  if (libraryPending.length) groups.push(libraryPending);
  for (const keys of groups) {
    try {
      if (keys.some((key) => key === 'libraryMax' || key === 'libraryMin')) await routes.library(draft.values.libraryMax, draft.values.libraryMin);
      else await routes[keys[0]](draft.values[keys[0]]);
      for (const key of keys) delete errors[key];
    } catch (error) {
      const message = error?.message || 'Could not save this setting.';
      for (const key of keys) errors[key] = message;
    }
  }
  const saved = { ...draft.saved };
  for (const key of pending) if (!errors[key]) saved[key] = draft.values[key];
  return { draft: { ...draft, saved, didSave: pending.length > 0 && Object.keys(errors).length === 0 }, errors };
}

// Discard: each value goes back to its saved value (ADR 0166 item 13).
export function discardSettings(draft) {
  return { saved: { ...draft.saved }, values: { ...draft.saved }, didSave: false };
}

// Whether Studio asks the leave question (ADR 0166 items 10 and 11). A change
// of the tab does not ask.
export function asksSettingsLeave(draft, destination) {
  return destination !== 'tab' && pendingSettings(draft).length > 0;
}

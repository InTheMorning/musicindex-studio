import test from 'node:test';
import assert from 'node:assert/strict';
import { makeSettingsDraft, changeSetting, settingsDraftView, discardSettings, saveSettings, asksSettingsLeave, SETTING_TABS } from '../ui/settingsdraft.js';

test('settings draft reports no, one, and many changes', () => {
  let draft = makeSettingsDraft({ scale: 'medium', lock: 5 });
  assert.equal(settingsDraftView(draft).status, 'No changes');
  assert.equal(settingsDraftView(draft).saveDisabled, true);
  assert.equal(settingsDraftView(draft).discardDisabled, true);
  draft = changeSetting(draft, 'scale', 'large');
  assert.equal(settingsDraftView(draft).status, '1 change not saved');
  draft = changeSetting(draft, 'lock', 10);
  assert.equal(settingsDraftView(draft).status, '2 changes not saved');
});

test('all seven settings track their owning tab', () => {
  let draft = makeSettingsDraft({ scale: 1, lock: 1, window: 'tray', outside: true, nostr: '', libraryMax: 2, libraryMin: 1 });
  for (const key of Object.keys(SETTING_TABS)) draft = changeSetting(draft, key, `changed-${key}`);
  const view = settingsDraftView(draft);
  assert.deepEqual(view.pending, Object.keys(SETTING_TABS));
  assert.deepEqual(view.tabs, ['general', 'nostr', 'data']);
  assert.deepEqual(view.rows, Object.keys(SETTING_TABS));
});

test('tab changes keep the draft; leaving with changes asks', () => {
  const draft = changeSetting(makeSettingsDraft({ lock: 5 }), 'lock', 10);
  assert.equal(asksSettingsLeave(draft, 'tab'), false);
  assert.equal(asksSettingsLeave(draft, 'page'), true);
});

test('discard restores saved scale and save accepts pending values', async () => {
  let draft = changeSetting(makeSettingsDraft({ scale: 'medium' }), 'scale', 'large');
  assert.equal(discardSettings(draft).values.scale, 'medium');
  ({ draft } = await saveSettings(draft, { scale: async () => {} }));
  assert.equal(draft.saved.scale, 'large');
  assert.equal(settingsDraftView(draft).status, 'Saved');
});

test('invalid library limits and relay values block Save with row causes', () => {
  let draft = makeSettingsDraft({ libraryMax: '10', libraryMin: '2', nostr: 'wss://relay.example' });
  draft = changeSetting(draft, 'libraryMax', '0');
  draft = changeSetting(draft, 'nostr', 'https://relay.example');
  const view = settingsDraftView(draft);
  assert.equal(view.saveDisabled, true);
  assert.equal(view.discardDisabled, false);
  assert.match(view.causes.libraryMax, /greater than 0/);
  assert.match(view.causes.nostr, /wss:\/\//);
  draft = changeSetting(draft, 'nostr', '');
  assert.equal(settingsDraftView(draft).causes.nostr, undefined);
});

test('save attempts every setting and sends both library limits once', async () => {
  const draft = { ...makeSettingsDraft({ scale: 'medium', lock: 5, libraryMax: '10', libraryMin: '2' }),
    values: { scale: 'large', lock: 10, libraryMax: '20', libraryMin: '4' } };
  const calls = [];
  const routes = {
    scale: async () => { calls.push('scale'); },
    lock: async () => { calls.push('lock'); throw new Error('locked'); },
    library: async (max, min) => { calls.push(`library:${max}:${min}`); },
  };
  const result = await saveSettings(draft, routes);
  assert.deepEqual(calls, ['scale', 'lock', 'library:20:4']);
  assert.equal(result.errors.lock, 'locked');
  assert.deepEqual(settingsDraftView(result.draft).pending, ['lock']);
});

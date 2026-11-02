// The connected apps model (ADR 0110, task 073e, ADR 0042).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import {
  appRowLines, pairingPhase, pairingNotice, doneRemovesApp, grantControl, draftBlockedNotice,
} from '../ui/apps.js';

const timeText = (s) => `t${s}`;
const clockText = (s) => `c${s}`;
const active = {
  id: 1, name: 'Live Publisher', state: 'active', short_key: 'npub1lp7w…9k2q',
  live_value_origin: 'wss://live.musicindex.org', last_used_at: null, last_address: null, shows: [],
};

test('a waiting app shows no key and the time when the code ends (screen 7)', () => {
  const lines = appRowLines({ id: 2, name: 'Laptop', state: 'waiting', code_expires_at: 90, shows: [] }, timeText, clockText);
  assert.equal(lines.key, 'No key yet');
  assert.equal(lines.status, 'Waiting for the app. The pairing text works until c90.');
  assert.equal(lines.address, null);
  assert.equal(lines.connectAgain, false);
});

test('an app that paired and has no show shows screen 6c lines', () => {
  const lines = appRowLines(active, timeText, clockText);
  assert.equal(lines.key, 'npub1lp7w…9k2q');
  assert.equal(lines.status, 'No show yet');
  assert.equal(lines.address, 'wss://live.musicindex.org');
});

test('an app with shows gives the count and the last use (screen 7)', () => {
  const used = { ...active, shows: [{ grant_id: 1 }], last_used_at: 5, last_address: '203.0.113.7' };
  assert.equal(appRowLines(used, timeText, clockText).status, '1 show · Last used t5 from 203.0.113.7');
  const two = { ...active, shows: [{ grant_id: 1 }, { grant_id: 2 }], last_used_at: 5 };
  assert.equal(appRowLines(two, timeText, clockText).status, '2 shows · Last used t5');
  assert.equal(appRowLines({ ...active, shows: [{ grant_id: 1 }] }, timeText, clockText).status, '1 show · Not used yet');
});

test('an app with a token shows Connect Again and the text of screen 8', () => {
  const lines = appRowLines({ id: 3, name: 'Old', state: 'token', shows: [] }, timeText, clockText);
  assert.equal(lines.connectAgain, true);
  assert.equal(lines.key, 'Token from before key pairs');
  assert.equal(lines.status, 'Studio no longer accepts tokens.');
});

test('the pairing phase follows the poll', () => {
  assert.equal(pairingPhase({ state: 'waiting' }), 'waiting');
  assert.equal(pairingPhase({ state: 'active' }), 'connected');
  assert.equal(pairingPhase(null), 'ended');
});

test('the pairing text is in the notice only while the app waits (screen 6b)', () => {
  const made = { id: 1, name: 'Live Publisher (studio PC)', pairingText: 'https://s.example#pair=K7M2-Q9XD' };
  const waiting = pairingNotice(made, 'waiting');
  assert.equal(waiting.text, made.pairingText);
  assert.equal(waiting.copy, true);
  assert.match(waiting.hint, /Done before that ends the pairing text\./);
  assert.match(waiting.hint, /15 minutes/);
  assert.equal(doneRemovesApp('waiting'), true);
});

test('after the pairing the notice gives the lines of screen 6c and no pairing text', () => {
  const made = { id: 1, name: 'Live Publisher (studio PC)', pairingText: 'https://s.example#pair=K7M2-Q9XD' };
  const done = pairingNotice(made, 'connected');
  assert.equal(done.title, 'Live Publisher (studio PC) is connected');
  assert.equal(done.hint, 'Compare the key with the key that the app shows. If the app did not connect, select Remove.');
  assert.equal(done.text, null);
  assert.equal(done.copy, false);
  assert.equal(doneRemovesApp('connected'), false);
  const ended = pairingNotice(made, 'ended');
  assert.equal(ended.text, null);
  assert.equal(doneRemovesApp('ended'), false);
  assert.ok(!JSON.stringify([done, ended]).includes('K7M2'));
});

test('the pairing text uses SecretInput and goes to no other place (ADR 0110 item 7)', () => {
  const app = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  const start = app.indexOf('function ConnectedAppsPanel');
  const panel = app.slice(start, app.indexOf('\n}\n', start));
  assert.match(panel, /<\$\{SecretInput\} id="app-pairing-text"/,
    'ADR 0110 item 7: the pairing text must be in a SecretInput. Use the component of app.js.');
  assert.ok(!/document\.title|history\.(push|replace)State|location\.(hash|href)\s*=|new Notification/.test(panel),
    'ADR 0110 item 7: the pairing text must not go to the address, the title or a notification.');
  const writes = panel.match(/clipboard\.writeText/g) || [];
  assert.equal(writes.length, 1, 'ADR 0110 item 7: Studio writes to the clipboard only from Copy.');
  assert.ok(!/has_key|made\.token|newAppToken/.test(panel),
    'ADR 0110 item 37: the app panel keeps no code of the token protocol. Delete it.');
});

const one = [{ id: 1, name: 'Live Publisher', short_key: 'npub1a', live_value_origin: 'wss://live.musicindex.org' }];
const two = [...one, { id: 2, name: 'Test Publisher', short_key: 'npub1b', live_value_origin: 'wss://t.example' }];

test('one app gives a check box with its name and live value address (screen 1)', () => {
  const c = grantControl({ apps: one, grantApps: {}, guid: 'g', signerNote: null });
  assert.equal(c.kind, 'check');
  assert.equal(c.label, 'Let Live Publisher start and end this show, and set its live value');
  assert.deepEqual(c.notes, ['Live value address: wss://live.musicindex.org']);
  assert.equal(c.checked, false);
  assert.equal(c.appId, 1);
  assert.equal(grantControl({ apps: one, grantApps: { g: 1 }, guid: 'g', signerNote: null }).checked, true);
});

test('two or more apps give a select menu with No app first (screen 2)', () => {
  const c = grantControl({ apps: two, grantApps: {}, guid: 'g', signerNote: null });
  assert.equal(c.kind, 'select');
  assert.equal(c.label, 'Connected app for this show');
  assert.deepEqual(c.options.map((o) => o.text), ['No app', 'Live Publisher', 'Test Publisher']);
  assert.equal(c.value, '');
  assert.equal(c.notes[0], 'This app can start and end this show, and set its live value. No other app can.');
  assert.equal(grantControl({ apps: two, grantApps: { g: 2 }, guid: 'g', signerNote: null }).value, '2');
  const same = grantControl({ apps: [one[0], { ...two[1], name: 'Live Publisher' }], grantApps: {}, guid: 'g', signerNote: null });
  assert.equal(same.options[2].text, 'Live Publisher · npub1b');
});

test('no app gives a disabled menu and the note of screen 3', () => {
  const c = grantControl({ apps: [], grantApps: {}, guid: 'g', signerNote: null });
  assert.equal(c.kind, 'none');
  assert.equal(c.disabled, true);
  assert.deepEqual(c.options, [{ value: '', text: 'No app' }]);
  assert.deepEqual(c.notes, ['No app is connected. Connect an app in Settings.']);
});

test('a host with My signer disables the control unless the draft has the grant', () => {
  const note = 'Connected apps cannot write to a file host with My signer.';
  assert.equal(grantControl({ apps: one, grantApps: {}, guid: 'g', signerNote: note }).disabled, true);
  assert.equal(grantControl({ apps: one, grantApps: { g: 1 }, guid: 'g', signerNote: note }).disabled, false);
  assert.equal(grantControl({ apps: two, grantApps: {}, guid: 'g', signerNote: note }).disabled, true);
  assert.ok(grantControl({ apps: one, grantApps: {}, guid: 'g', signerNote: note }).notes.includes(note));
});

test('a show title joins the label when the draft has many shows', () => {
  assert.match(grantControl({ apps: one, grantApps: {}, guid: 'g', signerNote: null, showTitle: 'Night' }).label, /: Night$/);
});

test('the draft notice names the app, the show and the start (screens 9 and 10)', () => {
  const n = draftBlockedNotice({ app_name: 'Live Publisher', show_title: 'Friday Night Signals' }, 'Friday 21:00');
  assert.equal(n.title, 'Live Publisher cannot start Friday Night Signals while this draft has changes');
  assert.equal(n.text, 'Write or discard the draft before Friday 21:00. Select Review to write it.');
  assert.equal(draftBlockedNotice({ app_name: '', show_title: null }, null).text,
    'Write or discard the draft before the show starts. Select Review to write it.');
});

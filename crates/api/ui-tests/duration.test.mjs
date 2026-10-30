// The duration model (ADR 0094, ADR 0042).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { readDuration, durationText, DURATION_ERROR, unreadDurationHint } from '../ui/duration.js';

test('readDuration reads each form', () => {
  for (const text of ['4:21', '04:21', '0:04:21', '261', ' 4:21 ', '261.9']) {
    assert.equal(readDuration(text), 261, text);
  }
  assert.equal(readDuration('1:02:03'), 3723);
});

test('readDuration gives null for other text', () => {
  for (const text of ['4:61', 'four', '', ' ', '1:2:3:4', '-5', '4:', ':21', null]) {
    assert.equal(readDuration(text), null, String(text));
  }
});

test('durationText gives M:SS, H:MM:SS or an empty text', () => {
  assert.equal(durationText(261), '4:21');
  assert.equal(durationText(3723), '1:02:03');
  assert.equal(durationText(5), '0:05');
  assert.equal(durationText(null), '');
});

test('the texts of the field', () => {
  assert.equal(DURATION_ERROR, 'Type the duration as minutes and seconds, for example 4:21.');
  assert.match(unreadDurationHint('soon'), /“soon”/);
});

test('ADR 0094 item 6: only duration.js makes duration text from seconds', () => {
  const dir = new URL('../ui/', import.meta.url);
  for (const name of readdirSync(dir).filter((n) => n.endsWith('.js') && n !== 'duration.js')) {
    assert.ok(!readFileSync(new URL(name, dir), 'utf8').includes('durationFromSeconds'),
      `ADR 0094 item 6: ${name} must not have durationFromSeconds. Use duration_seconds and durationText of duration.js.`);
  }
  const editor = readFileSync(new URL('editor.js', dir), 'utf8');
  assert.ok(!/^\s*duration: (?!null|\(\) =>)/m.test(editor),
    'ADR 0094 item 6: editor.js must not set duration: in a new item. Set duration_seconds.');
});

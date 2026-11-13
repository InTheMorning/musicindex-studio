// The confirmation sheets (task 057, musicindex ADR 0023 R-CONFIRM-1 and
// R-CASE-1). Each test reads the UI modules.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { CONFIRMS, confirmButtons } from '../ui/confirms.js';
import { isTitleCase } from '../ui/casing.js';

const uiDir = new URL('../ui/', import.meta.url);

test('no UI module uses the browser box confirm()', () => {
  const names = readdirSync(uiDir).filter((name) => name.endsWith('.js'));
  for (const name of names) {
    const text = readFileSync(new URL(name, uiDir), 'utf8');
    assert.ok(!/(?<![A-Za-z0-9_$.])confirm\(/.test(text),
      `musicindex ADR 0023 R-CONFIRM-1: ${name} uses the browser box confirm(). Use askConfirm of app.js with an entry of confirms.js.`);
  }
});

test('the leave sheet of a person has the texts of ADR 0116 item 8', () => {
  const entry = CONFIRMS.leavePerson('Text of the sheet');
  assert.equal(entry.title, 'Leave without saving?');
  assert.equal(entry.text, 'Text of the sheet');
  assert.equal(entry.action, 'Discard Changes');
  assert.equal(entry.role, 'destructive');
  assert.equal(entry.cancel, 'Keep Editing');
});

test('restore the hosted copy is destructive and asks first (ADR 0195)', () => {
  assert.equal(typeof CONFIRMS.restoreHostedCopy, 'function',
    'ADR 0195: add restoreHostedCopy to confirms.js.');
  const entry = CONFIRMS.restoreHostedCopy('a.mp3');
  assert.equal(entry.role, 'destructive',
    'ADR 0195: mark Restore the Hosted Copy as destructive in confirms.js.');
  assert.equal(entry.action, 'Restore the Hosted Copy',
    'ADR 0195: name the confirmation action Restore the Hosted Copy.');

  const editor = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  assert.ok(editor.includes('Restore the Hosted Copy…'),
    'ADR 0195: label the changed file button Restore the Hosted Copy….');
  assert.ok(!editor.includes('Restore</button>'),
    'ADR 0195: remove the unconfirmed Restore button label.');
  const confirm = editor.indexOf('CONFIRMS.restoreHostedCopy');
  const restore = editor.indexOf("action: 'restore'", confirm);
  assert.ok(confirm >= 0 && restore > confirm,
    'ADR 0195: call CONFIRMS.restoreHostedCopy through askConfirm before the restore action.');
});

test('each entry gives a title, a text, an action and a role', () => {
  for (const [name, entry] of Object.entries(CONFIRMS)) {
    const { title, text, action, role } = entry('X', 'track');
    assert.ok(title.endsWith('?'), `musicindex ADR 0023 R-CONFIRM-1: the title of ${name} must end with "?".`);
    assert.ok(text.length > 0, `musicindex ADR 0023 R-CONFIRM-1: ${name} needs a text.`);
    assert.ok(isTitleCase(action),
      `musicindex ADR 0023 R-CASE-1: the action of ${name} ("${action}") must be in title case.`);
    assert.ok(role === 'destructive' || role === 'primary',
      `ADR 0037: the role of ${name} must be destructive or primary.`);
  }
});

test('a destructive confirmation shows the destructive button first and focuses the safe button', () => {
  const entries = [
    ...Object.values(CONFIRMS).map((make) => make === CONFIRMS.downloadFirst ? make(null) : make('item')),
  ];
  for (const entry of entries) {
    const buttons = confirmButtons(entry);
    const focused = buttons.find((button) => button.focus);
    if (entry.role === 'destructive') {
      assert.equal(buttons[0].kind, 'action', 'musicindex ADR 0064 R-ACT-8: a destructive button is not first. Put it first in the row.');
      assert.equal(buttons.at(-1).kind, 'cancel', 'musicindex ADR 0064 R-ACT-9: the safe button is not last. Put it last and give it focus.');
      assert.equal(focused, buttons.at(-1), 'musicindex ADR 0064 R-ACT-9: the safe button has no first focus. Focus the safe button.');
    } else {
      assert.equal(focused.kind, 'cancel', 'musicindex ADR 0064 R-ACT-9: the safe button has no first focus. Focus the safe button.');
    }
  }
});

test('no destructive button is a submit button', () => {
  for (const name of readdirSync(uiDir).filter((file) => file.endsWith('.js'))) {
    const text = readFileSync(new URL(name, uiDir), 'utf8');
    assert.ok(!/<button(?=[^>]*class=["'][^"']*destructive)(?=[^>]*type=["']submit)[^>]*>/s.test(text),
      `musicindex ADR 0064 R-ACT-9: ${name} has a destructive submit button. Use type="button" and an onClick handler.`);
  }
});

test('no button comes before a destructive button in its row', () => {
  const names = readdirSync(uiDir).filter((file) => file.endsWith('.js'));
  const startPattern = /buttons = html`|=> html`|<footer\b|class=["'](?:actions|draft-toolbar|app-buttons|alt-enclosure|change-row|link-row)["']|<li\b|<\$Bar\b/g;
  for (const name of names) {
    const text = readFileSync(new URL(name, uiDir), 'utf8');
    const buttonPattern = /<button\b[^>]*>/gs;
    let match;
    while ((match = buttonPattern.exec(text))) {
      if (!/class=["'][^"']*(?:destructive|discard)/s.test(match[0])) continue;
      const before = text.slice(0, match.index);
      const starts = [...before.matchAll(startPattern)];
      if (!starts.length) continue;
      const rowStart = starts.at(-1).index;
      if (text.slice(rowStart, match.index).includes('<button')) {
        assert.fail(`musicindex ADR 0064 R-ACT-8: a button comes before a destructive button in ${name}, at ${match[0].replace(/\s+/g, ' ').slice(0, 90)}. Put the destructive button first and the safe action last.`);
      }
    }
  }
});

test('no row has only destructive buttons', () => {
  const names = readdirSync(uiDir).filter((file) => file.endsWith('.js'));
  const rowEnd = /<\/footer>|<\/div>|<\/li>|<\/\$\{Bar\}>|<\/\/>/;
  for (const name of names) {
    const text = readFileSync(new URL(name, uiDir), 'utf8');
    const buttonPattern = /<button\b[^>]*>/gs;
    let match;
    while ((match = buttonPattern.exec(text))) {
      if (!/class=["'`][^"'`]*(?:destructive|discard)/s.test(match[0])) continue;
      const after = text.slice(match.index + match[0].length);
      const end = after.search(rowEnd);
      const rest = [...after.slice(0, end < 0 ? after.length : end).matchAll(/<button\b[^>]*>/gs)];
      if (rest.length > 0 && rest.every((button) => /destructive|discard/.test(button[0]))) {
        assert.fail(`musicindex ADR 0064 R-ACT-10: a row in ${name} has only destructive buttons, at ${match[0].replace(/\s+/g, ' ').slice(0, 90)}. Show one "Remove…" button that opens a menu with the destructive items.`);
      }
    }
  }
});

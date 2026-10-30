// A secret value stays hidden until the user shows it (ADR 0078). Each test
// reads the UI modules as text.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';

const uiDir = new URL('../ui/', import.meta.url);
const sources = readdirSync(uiDir)
  .filter((name) => name.endsWith('.js'))
  .map((name) => ({ name, text: readFileSync(new URL(name, uiDir), 'utf8') }));

const FIX = 'Make the field with SecretField, SecretInput or SecretText of app.js.';

// The words of a label or an ID that name a secret value.
const SECRET_WORDS = /passphrase|password|secret|private key|recovery key|access key|token|nsec/i;

test('no module makes a password field of its own', () => {
  for (const { name, text } of sources) {
    assert.ok(!/type="password"/.test(text),
      `ADR 0078 item 3: ${name} makes a password field with no eye button. ${FIX}`);
  }
});

test('no Field has the label of a secret value', () => {
  for (const { name, text } of sources) {
    for (const match of text.matchAll(/<\$\{Field\}[^>]*?\slabel=(?:"([^"]*)"|\$\{`([^`]*)`\})/g)) {
      const label = match[1] ?? match[2];
      assert.ok(!SECRET_WORDS.test(label),
        `ADR 0078 item 7: ${name} shows "${label}" in a Field with no eye button. ${FIX}`);
    }
  }
});

test('each known secret value uses a secret component', () => {
  const app = sources.find((s) => s.name === 'app.js').text;
  const ids = ['passphrase', 'confirm', 's3-key', 's3-secret', 'sftp-key', 'sftp-passphrase',
    'sftp-password', '${`${idPrefix}-secret`}', 'unlock', 'app-token', 'sign-in-passphrase',
    'recovery-key', 'recovery-key-shown', 'add-method-passphrase', 'sign-up-passphrase',
    'unlock-sheet-passphrase'];
  for (const id of ids) {
    const quoted = id.startsWith('$') ? id : `"${id}"`;
    const tag = new RegExp(`<\\$\\{Secret(Field|Input|Text)\\} id=${quoted.replace(/[$`{}]/g, '\\$&')}[\\s/>]`);
    assert.match(app, tag, `ADR 0078 item 2: the secret value with the ID ${id} is not hidden. ${FIX}`);
  }
});

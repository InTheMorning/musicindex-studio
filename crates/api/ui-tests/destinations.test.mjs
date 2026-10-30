// The fixed destinations show an icon and no type dot (ADR 0092 items 5 and 9).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { leftPanel } from '../ui/shell.js';

const appText = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');

const panel = (extra) => leftPanel({
  view: { name: 'releases' }, feeds: [], model: null, vaultOpen: false, ...extra,
});

test('each fixed destination of leftPanel has its icon', () => {
  const entries = [
    ...panel({ vaultOpen: true }).top,
    ...panel({ vaultOpen: true }).bottom,
    ...panel({ vaultLocked: true }).bottom,
  ];
  const expected = {
    releases: 'library', hosts: 'hard-drive', settings: 'settings', lock: 'lock', unlock: 'lock-open',
  };
  for (const [id, icon] of Object.entries(expected)) {
    const entry = entries.find((item) => item.id === id);
    assert.ok(entry, `ADR 0092 item 5: leftPanel has no entry "${id}". Add the entry in crates/api/ui/shell.js.`);
    assert.equal(entry.icon, icon,
      `ADR 0092 item 5: the entry "${id}" must have the icon "${icon}". Set icon in leftPanel of crates/api/ui/shell.js.`);
  }
});

test('a destination row of app.js shows no type dot', () => {
  for (const name of ['LeftPanelFooter', 'RootPage']) {
    const start = appText.indexOf(`function ${name}(`);
    assert.ok(start >= 0, `ADR 0092 item 9: app.js has no function ${name}. Update this test.`);
    const end = appText.indexOf('\n}\n', start);
    const source = appText.slice(start, end);
    assert.ok(!/\bdot\b/.test(source),
      `ADR 0092 item 9: a destination row shows no type dot. Use the icon of its leftPanel entry (${name}).`);
  }
});

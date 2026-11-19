import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const source = (name) => readFileSync(new URL(`../ui/${name}`, import.meta.url), 'utf8');

test('feed terms have their task 184 hints', () => {
  const editor = source('editor.js');
  const values = source('values.js');
  const people = source('peoplepages.js');

  for (const hint of [
    'The artist name that podcast apps show.',
    'The name that podcast apps show for this page.',
    'Tells podcast apps that the feed changed, so that they show the change in minutes.',
    'Value Routes tell podcast apps where to send the payments of listeners.',
  ]) {
    assert.ok(editor.includes(hint), `editor.js must include: ${hint}`);
  }
  assert.ok(values.includes('A recipient gets its shares divided by the total shares of the block.'));
  assert.ok(people.includes('Wallets in your payments that no person in People has.'));
  assert.ok(!editor.includes('The itunes:author of the feed.'));
});

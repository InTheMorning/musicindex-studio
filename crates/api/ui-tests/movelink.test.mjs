// The texts of a refused write (ADR 0126, ADR 0042). The test reads the
// variant names from the core, so that a new variant fails here.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { isTitleCase } from '../ui/casing.js';
import {
  CONDITION_TEXT, HOP_TEXT, STAGE_TEXT, REPAIRS, refusalView, refusalOfError, noticeViews, blocksWrite,
  repairButtons, repairRoute, REPLACES_POINTER_NOTE,
} from '../ui/movelink.js';

const source = readFileSync(new URL('../../core/src/move_link.rs', import.meta.url), 'utf8');

// The variants of a public enum of move_link.rs, as snake_case names.
function variants(name) {
  const match = source.match(new RegExp(`pub enum ${name} \\{([\\s\\S]*?)\\n\\}`));
  assert.ok(match, `crates/core/src/move_link.rs has no enum ${name}. Update movelink.test.mjs.`);
  return match[1].split('\n')
    .map((line) => line.match(/^    ([A-Z][A-Za-z]*)[,\s]/))
    .filter(Boolean)
    .map((m) => m[1].replace(/(?<!^)([A-Z])/g, '_$1').toLowerCase());
}

const owner = 'ADR 0126 item 10. Add the text to crates/api/ui/movelink.js.';

test('each condition of the core has a text', () => {
  const names = variants('Condition');
  assert.ok(names.length >= 6);
  for (const name of names) assert.ok(CONDITION_TEXT[name], `Condition ${name} has no text. ${owner}`);
  assert.deepEqual(Object.keys(CONDITION_TEXT).sort(), [...names].sort(), 'movelink.js has a text for a condition that the core does not give.');
});

test('each hop result of the core has a text', () => {
  const names = variants('HopResult');
  assert.ok(names.length >= 14);
  for (const name of names) assert.ok(HOP_TEXT[name], `HopResult ${name} has no text. ${owner}`);
  assert.deepEqual(Object.keys(HOP_TEXT).sort(), [...names].sort());
});

test('each repair of the core has a label and a route', () => {
  const names = variants('Repair');
  assert.deepEqual(Object.keys(REPAIRS).sort(), [...names].sort());
  assert.equal(REPAIRS.use_public_address.label, 'Use the Public Address');
  assert.equal(REPAIRS.remove_tag.label, 'Remove the Tag');
  assert.equal(repairRoute('remove_tag'), 'remove-tag');
  assert.equal(repairRoute('use_public_address'), 'use-public-address');
  for (const { label } of Object.values(REPAIRS)) assert.ok(isTitleCase(label), `${label} is not in title case (musicindex ADR 0023 R-CASE-1).`);
});

test('the stage gives what changed on the hosts', () => {
  assert.equal(STAGE_TEXT[1], 'Nothing changed on a file host.');
  assert.match(STAGE_TEXT[2], /The feed stays at its address\. The new file stays on the new file host, and the next write replaces it\./);
});

test('a refusal gives the title, the condition, the state, each hop and the repairs', () => {
  const view = refusalView({
    condition: 'loop', number: 1, stage: 1, repairs: ['remove_tag', 'unknown'],
    hops: [
      { url: 'https://c.example/a.xml', result: 'feed' },
      { url: 'https://c.example/a.xml', final_url: 'https://d.example/a.xml', result: 'revisited' },
    ],
  });
  assert.equal(view.title, 'Studio refuses this write.');
  assert.equal(view.text, CONDITION_TEXT.loop);
  assert.equal(view.state, STAGE_TEXT[1]);
  assert.deepEqual(view.hops.map((h) => [h.url, h.finalUrl, h.why]), [
    ['https://c.example/a.xml', null, HOP_TEXT.feed],
    ['https://c.example/a.xml', 'https://d.example/a.xml', HOP_TEXT.revisited],
  ]);
  assert.deepEqual(view.repairs, [{ id: 'remove_tag', label: 'Remove the Tag' }], 'a repair that the UI does not know gives no button');
  assert.equal(refusalView(null), null);
});

test('a condition or a hop that the UI does not know still gives a text', () => {
  const view = refusalView({ condition: 'new', stage: 3, hops: [{ url: 'u', result: 'new' }], repairs: [] });
  assert.ok(view.text.length > 0);
  assert.equal(view.state, '');
  assert.ok(view.hops[0].why.length > 0);
});

test('the refusal of a write is in the details of the error bad_move_link', () => {
  const details = { condition: 'loop', stage: 2, hops: [] };
  assert.equal(refusalOfError({ code: 'bad_move_link', details }), details);
  assert.equal(refusalOfError({ code: 'conflict', details }), null);
  assert.equal(refusalOfError({ code: 'bad_move_link' }), null);
  assert.equal(refusalOfError(null), null);
});

test('the self link notice gives both addresses and the repair', () => {
  const report = {
    refusal: null,
    notices: [{ notice: 'self_link_differs', self_url: 'https://old.example/a.xml', draft_url: 'https://b.example/a.xml' }],
    repairs: ['use_public_address'],
  };
  const [view] = noticeViews(report);
  assert.equal(view.title, 'The self link names a different address.');
  assert.equal(view.selfUrl, 'https://old.example/a.xml');
  assert.equal(view.draftUrl, 'https://b.example/a.xml');
  assert.equal(view.end, 'Studio can still write it.');
  assert.deepEqual(view.repairs, [{ id: 'use_public_address', label: 'Use the Public Address' }]);
  assert.deepEqual(noticeViews(null), []);
  assert.deepEqual(noticeViews({ notices: [{ notice: 'other' }] }), []);
});

test('only a refusal of the check blocks the write', () => {
  assert.equal(blocksWrite(null), false);
  assert.equal(blocksWrite({ refusal: null, notices: [{ notice: 'self_link_differs' }] }), false);
  assert.equal(blocksWrite({ refusal: { condition: 'loop' } }), true);
});

test('the move row says that a pointer file becomes the feed', () => {
  assert.equal(REPLACES_POINTER_NOTE, 'The pointer file at the new address becomes the feed.');
  assert.deepEqual(repairButtons(undefined), []);
});

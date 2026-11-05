// The texts of the build date (ADR 0127, ADR 0042). The test reads the fields
// from the core, so that a new field of the state fails here.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import {
  buildDateNotices, NO_SERVER_TITLE, LATER_BASE_TITLE, CLOCK_AHEAD_TITLE,
} from '../ui/builddate.js';
import { keyWord } from '../ui/review.js';

const owner = 'ADR 0127 items 8, 11, 12 and 23. Correct crates/api/ui/builddate.js.';
const source = readFileSync(new URL('../../core/src/pipeline.rs', import.meta.url), 'utf8');

test('the module knows each field of the state of the core', () => {
  const match = source.match(/pub struct BuildDateState \{([\s\S]*?)\n\}/);
  assert.ok(match, `crates/core/src/pipeline.rs has no BuildDateState. ${owner}`);
  const fields = [...match[1].matchAll(/pub (\w+):/g)].map((m) => m[1]).sort();
  assert.deepEqual(fields, ['clock_ahead_seconds', 'later_base', 'no_server', 'set'], `A new field needs a text. ${owner}`);
});

test('each state gives its notice, and a set date gives none', () => {
  const none = { set: false, no_server: false, later_base: null, clock_ahead_seconds: null };
  assert.deepEqual(buildDateNotices(null), [], owner);
  assert.deepEqual(buildDateNotices(none), [], owner);
  assert.deepEqual(buildDateNotices({ ...none, set: true }), [], owner);
  assert.equal(buildDateNotices({ ...none, no_server: true })[0].title, NO_SERVER_TITLE, owner);
  assert.equal(buildDateNotices({ ...none, clock_ahead_seconds: 600 })[0].title, CLOCK_AHEAD_TITLE, owner);
  const later = buildDateNotices({ ...none, later_base: 'Mon, 01 Jan 2099 00:00:00 +0000' })[0];
  assert.equal(later.title, LATER_BASE_TITLE, owner);
  assert.equal(later.value, 'Mon, 01 Jan 2099 00:00:00 +0000', owner);
});

test('the field view names the date', () => {
  assert.equal(keyWord('last_build_date'), 'Build date', owner);
});

test('the notice of a clock gives no calculated time', () => {
  const [notice] = buildDateNotices({ set: true, no_server: false, later_base: null, clock_ahead_seconds: 63072000 });
  assert.doesNotMatch(`${notice.title} ${notice.text}`, /\d/, owner);
});

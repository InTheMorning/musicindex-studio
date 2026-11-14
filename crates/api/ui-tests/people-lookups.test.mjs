import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { refreshLookups } from '../ui/recipients.js';

test('recipient lookups report whether the response checked an address', async () => {
  const calls = [];
  const api = async (...args) => {
    calls.push(args);
    return { checked: 2 };
  };
  assert.equal(await refreshLookups(api), true,
    'ADR 0201: a positive checked count must refresh recipients; check refreshLookups in recipients.js.');
  assert.deepEqual(calls, [['/recipients/lookups', { method: 'POST' }]],
    'ADR 0201: send POST /recipients/lookups; check the request in refreshLookups.');
  assert.equal(await refreshLookups(async () => ({ checked: 0 })), false,
    'ADR 0201: zero checked addresses must not refresh recipients; check the checked count.');
  assert.equal(await refreshLookups(async () => { throw new Error('offline'); }), false,
    'ADR 0201: a failed lookup must not refresh recipients; keep the cached recipient list.');
});

test('People loads cached recipients before it starts lookups', () => {
  const source = readFileSync(new URL('../ui/peoplepages.js', import.meta.url), 'utf8');
  const page = source.slice(source.indexOf('export function PeoplePage'), source.indexOf('const LINKED_NONE'));
  const person = source.slice(source.indexOf('function usePerson'), source.indexOf('// The sheet "Find on Music Index'));
  const initialPageRequests = page.slice(page.indexOf('const [people, suggestions, sets, hidden]'), page.indexOf(']);'));
  assert.ok(!initialPageRequests.includes("api('/recipients')"),
    'ADR 0201: load the People list before recipients; remove GET /recipients from the initial Promise.all.');
  assert.ok(page.includes('refreshLookups(api)'),
    'ADR 0201: the People page must run lookups after GET /recipients; call refreshLookups.');
  const load = page.slice(page.indexOf('const load = () => run('), page.indexOf('const loadRecipients'));
  assert.ok(load.includes('loadRecipients(id, base);') && !load.includes('await loadRecipients'),
    'ADR 0201: the actions of People must not wait for the lookups; start loadRecipients without await inside run.');
  assert.ok(!person.includes('await refreshLookups'),
    'ADR 0201: a read of a person must not wait for the lookups; start refreshLookups without await in usePerson.');
  assert.ok(person.includes('refreshLookups(api)'),
    'ADR 0201: the person page must run lookups after GET /recipients; call refreshLookups.');
});

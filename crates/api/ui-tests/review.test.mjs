// The lines of a review (ADR 0114 items 8 to 12).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { shownLines, localTime, localTimeNote, runs, commonIndent, moreLabel, stripIndent, normalizeView, showsExamine, streamProblemRows } from '../ui/review.js';
import { setLiveStreamType } from '../ui/live.js';

const part = (text, changed = false) => ({ text, changed });
const same = (text) => ({ kind: 'same', parts: [part(text)], pair: null });
const removed = (parts, pair = null) => ({ kind: 'removed', parts, pair });
const added = (parts) => ({ kind: 'added', parts });

test('each changed line is one row with its sign', () => {
  const lines = [
    same('<a>'),
    removed([part('<t>'), part('Old', true)], 2),
    added([part('<t>'), part('New', true)]),
  ];
  assert.deepEqual(shownLines(lines).map((r) => r.sign), [' ', '−', '+']);
});

test('a changed line with a time has the local time, and each other line has none', () => {
  const row = (text, kind = 'add') => ({ kind, parts: [part(text)] });
  assert.equal(
    localTimeNote(row('<lastBuildDate>Thu, 05 Nov 2026 14:20:10 +0000</lastBuildDate>'), 'America/Chicago'),
    `Local time: ${localTime('Thu, 05 Nov 2026 14:20:10 +0000', 'America/Chicago')}`,
    'ADR 0114 item 15: a changed date line has a second line with the local time.',
  );
  assert.equal(localTimeNote(row('<pubDate>Thu, 05 Nov 2026 14:20:10 +0000</pubDate>', 'same')), null);
  assert.equal(localTimeNote(row('<title>start="now"</title>')), null);
  assert.equal(localTimeNote(row('<lastBuildDate>not a time</lastBuildDate>')), null);
  assert.match(localTimeNote(row('<podcast:liveItem start="2026-11-05T14:00:00Z" end="2026-11-05T15:00:00Z">')), /^Local time: .* to /);
});

test('runs keeps three rows of context and counts each hidden run', () => {
  const rows = [];
  for (let i = 0; i < 20; i += 1) rows.push({ kind: i === 10 ? 'add' : 'same', sign: ' ', parts: [part(`l${i}`)] });
  const segments = runs(rows, false);
  const hidden = segments.filter((s) => s.hidden);
  assert.deepEqual(hidden.map((s) => s.count), [7, 6]);
  assert.equal(segments.filter((s) => !s.hidden).length, 7);
  assert.equal(runs(rows, true).some((s) => s.hidden), false);
});

test('commonIndent gives the shared indent and ignores an empty row', () => {
  const row = (text) => ({ parts: [part(text)] });
  assert.equal(commonIndent([row('    <a>'), row(''), row('        <b>'), row('\t\t<c>')]), 2);
  assert.equal(commonIndent([]), 0);
  assert.deepEqual(stripIndent([part('    <a', false), part(' x', true)], 7), [part('x', true)]);
});

test('moreLabel and normalizeView', () => {
  assert.equal(moreLabel(1), 'Show 1 more line');
  assert.equal(moreLabel(12), 'Show 12 more lines');
  assert.equal(normalizeView('x'), 'simplified');
  assert.equal(normalizeView('detailed'), 'detailed');
});

test('Review blocks a refused stream type and allows a stream warning', () => {
  const rows = streamProblemRows([
    { guid: 'show-1', field: 'stream', level: 'refusal', text: 'The feed gives AAC, but the stream server sends MP3. Apps cannot play this stream.', server_type: 'audio/mpeg', server_type_label: 'MP3' },
    { guid: 'show-2', field: 'stream', level: 'warning', text: 'Studio could not check the stream: The stream server sent no type.' },
  ], [
    { item: { guid: 'show-1', title: 'Morning Show' } },
    { item: { guid: 'show-2', title: 'Evening Show' } },
  ]);
  assert.deepEqual(rows.map(({ title, button, blocksWrite }) => ({ title, button, blocksWrite })), [
    { title: 'Morning Show', button: 'Use MP3', blocksWrite: true },
    { title: 'Evening Show', button: null, blocksWrite: false },
  ]);
  assert.equal(rows[0].text, 'The feed gives AAC, but the stream server sends MP3. Apps cannot play this stream.');
  assert.equal(rows[1].text, 'Studio could not check the stream: The stream server sent no type.');
});

test('Use server type changes only the matching live item in the draft model', () => {
  const model = { live_items: [
    { item: { guid: 'show-1', enclosure: { url: 'https://stream.example/live', length: '0', media_type: 'audio/aac' } } },
    { item: { guid: 'show-2', enclosure: { url: 'https://stream.example/other', length: '0', media_type: 'audio/aac' } } },
  ] };
  const changed = setLiveStreamType(model, 'show-1', 'audio/mpeg');
  assert.equal(changed.live_items[0].item.enclosure.media_type, 'audio/mpeg');
  assert.equal(changed.live_items[1].item.enclosure.media_type, 'audio/aac');
  assert.equal(model.live_items[0].item.enclosure.media_type, 'audio/aac');
});

// ---- What changes (task 077c) ----
import { readFileSync } from 'node:fs';
import { fieldRows, guardRow, changeRow, KEY_WORDS, pathWord, inSentence, mergeRows, mergeReady } from '../ui/review.js';
import { shortKey } from '../ui/nostrkey.js';

const KEY = 'npub1' + 'q'.repeat(58);
const credit = (role) => ({ op: 'changed', path: 'npub', entry: role, old: null, new: KEY });
const profiles = new Map([[KEY, { npub: KEY, name: 'City Radio' }]]);

test('six changes of one key give a summary, two rows and a collapsed line', () => {
  const roles = ['musician', 'audio engineer', 'songwriter', 'composer', 'guitarist', 'keyboardist'];
  const [group] = fieldRows([{ kind: 'credits', name: 'City Radio', changes: roles.map(credit) }], profiles);
  assert.equal(group.label, 'Credits · City Radio');
  assert.equal(group.summary, '6 credits get the Nostr key City Radio');
  assert.equal(group.rows.length, 2);
  assert.equal(group.rows[0].label, 'musician');
  assert.match(group.rows[0].text, /Nostr key: none → City Radio \(/);
  assert.equal(group.more.summary, '4 more credits: songwriter, composer, guitarist, keyboardist');
  assert.equal(group.more.rows.length, 4);
});

test('a title, an added entry and a removed entry have a line', () => {
  const title = changeRow({ kind: 'field', name: 'title' }, { op: 'changed', path: '', old: 'A', new: 'B' });
  assert.deepEqual(title, { label: 'Title', text: 'A → B', warning: null });
  const live = changeRow({ kind: 'live', name: 'Show' }, { op: 'changed', path: 'item.title', entry: null, old: 'A', new: 'B' });
  assert.equal(live.label, 'Title');
  assert.equal(pathWord('item.title'), 'Title');
  const add = changeRow({ kind: 'credits', name: 'X' }, { op: 'added', path: '', entry: 'mixer', old: null, new: { name: 'X' } });
  assert.deepEqual([add.label, add.text], ['mixer', 'Added']);
  const gone = changeRow({ kind: 'track', name: 'T', position: 1 }, { op: 'removed', path: 'persons', entry: null, old: { name: 'Zed' }, new: null });
  assert.deepEqual([gone.label, gone.text], ['Credits', 'Removed: Zed']);
});

test('a payment block change shows one row for each recipient (task 182)', () => {
  // The core gives a channel payment list the kind `list`, with an empty path.
  const block = (recipients) => ({ value_type: 'lightning', method: 'keysend', suggested: null, recipients, time_splits: [] });
  const groups = [{ kind: 'list', name: 'values', changes: [{ op: 'added', path: '', new: block([
    { name: 'LN payer', recipient_type: 'lnaddress', address: 'pay@example.com', split: '90' },
    { name: 'Node payer', recipient_type: 'node', address: `02${'ab'.repeat(32)}`, split: '10' },
  ]) }] }];
  const [added] = fieldRows(groups);
  assert.equal(added.label, 'Payments');
  assert.deepEqual(added.rows.map(({ label, text }) => [label, text]), [
    ['Added: LN payer', 'pay@example.com · LNURL Address · 90.0 %'],
    ['Added: Node payer', `${shortKey(`02${'ab'.repeat(32)}`)} · Node Address · 10.0 %`],
  ]);
  // A track block has the path `values` in the track group.
  const [track] = fieldRows([{ kind: 'track', name: 'One', position: 1, changes: [{ op: 'removed', path: 'values', old: block([
    { name: 'Gone', recipient_type: 'lnaddress', address: 'gone@example.com', split: '1', fee: 'true' },
  ]) }] }]);
  assert.deepEqual([track.rows[0].label, track.rows[0].text], ['Removed: Gone', 'gone@example.com · LNURL Address · 100.0 % fee']);
  // A recipient with no name has its address as the title, as on the Value Routes page.
  const [noName] = fieldRows([{ kind: 'list', name: 'values', changes: [{ op: 'added', path: '', new: block([
    { name: '', recipient_type: 'lnaddress', address: 'a@example.com', split: '1' },
  ]) }] }]);
  assert.equal(noName.rows[0].label, 'Added: a@example.com');
  const [empty] = fieldRows([{ kind: 'list', name: 'values', changes: [{ op: 'added', path: '', new: block([]) }] }]);
  assert.deepEqual(empty.rows.map((r) => r.text), ['Added']);
});

test('an added recipient in a block that stays shows its share (task 182)', () => {
  const [group] = fieldRows([{ kind: 'list', name: 'values', changes: [{ op: 'added', path: 'recipients', entry: 'Wallet',
    new: { name: 'Wallet', recipient_type: 'lnaddress', address: 'wallet@example.com', split: '25' } }] }]);
  assert.equal(group.rows[0].label, 'Wallet');
  assert.match(group.rows[0].text, /Added: wallet@example\.com · LNURL Address · share 25$/);
});

test('the Simplified view does not show a placeholder GUID (task 182)', () => {
  const group = { kind: 'field', name: 'podcast_guid' };
  assert.equal(changeRow(group, { op: 'changed', path: '', old: null, new: 'pending-60384c23-abc' }).text, 'Set when the feed has a public address');
  assert.equal(changeRow(group, { op: 'changed', path: '', old: 'pending-60384c23-abc', new: 'guid-1' }).text, 'none → guid-1');
});

test('added and removed live items show their status and local times', async () => {
  const { changeRow } = await import('../ui/review.js');
  const group = { kind: 'live', name: 'Show' };
  const liveItem = { status: 'pending', start: '2026-11-05T14:00:00Z', end: '2026-11-05T15:00:00Z', item: { title: 'Show' } };
  const times = `${localTime(liveItem.start, 'UTC')} to ${localTime(liveItem.end, 'UTC')}`;
  assert.equal(changeRow(group, { op: 'added', path: '', entry: null, new: liveItem, old: null }, null, 'UTC').text, `Added: pending, ${times}`);
  assert.equal(changeRow(group, { op: 'removed', path: '', entry: null, old: liveItem, new: null }, null, 'UTC').text, `Removed: pending, ${times}`);
  const track = changeRow({ kind: 'track', name: 'Show', position: 1 }, { op: 'added', path: '', entry: 'track', new: { title: 'Track' }, old: null });
  assert.equal(track.text, 'Added');
});

test('a key line shows the profile, the short key or the warning', () => {
  const group = { kind: 'credits', name: 'HeyCitizen' };
  const withProfile = changeRow(group, credit('musician'), profiles);
  assert.match(withProfile.text, /City Radio \(npub1/);
  assert.equal(withProfile.warning, 'The profile name is not "HeyCitizen". Check this key.');
  const without = changeRow(group, credit('musician'), null);
  assert.doesNotMatch(without.text, /City Radio/);
  assert.equal(without.warning, null);
  assert.equal(changeRow({ ...group, name: 'City Radio' }, credit('musician'), profiles).warning, null);
});

test('a track credit key shows the warning from the name of the credit', () => {
  const track = { kind: 'track', name: 'Song', position: 1 };
  const change = { ...credit('musician'), path: 'persons.npub', name: 'HeyCitizen' };
  assert.equal(changeRow(track, change, profiles).warning, 'The profile name is not "HeyCitizen". Check this key.');
  assert.equal(changeRow(track, change, null).warning, null);
  assert.equal(changeRow(track, { ...change, name: undefined }, profiles).warning, null);
});

test('the guard row counts the uncovered changes', () => {
  assert.equal(guardRow(0), null);
  assert.equal(guardRow(1).title, '1 more change in the XML');
  assert.match(guardRow(1).text, /does not show it/);
  assert.equal(guardRow(3).title, '3 more changes in the XML');
  assert.match(guardRow(3).text, /does not show them/);
});

test('the word table has a word for each key of the feed model', () => {
  const source = readFileSync(new URL('../../core/src/feed.rs', import.meta.url), 'utf8');
  const keys = new Set();
  for (const block of source.matchAll(/pub struct (\w+)[^{]*\{([\s\S]*?)\n\}/g)) {
    for (const m of block[2].matchAll(/^\s*pub (\w+):/gm)) keys.add(m[1]);
  }
  assert.ok(keys.size > 40);
  const missing = [...keys].filter((k) => !(k in KEY_WORDS));
  assert.deepEqual(missing, []);
});

test('the simplified view names a change of Hide from apps', () => {
  const [group] = fieldRows([{ kind: 'field', name: 'hidden', changes: [{ op: 'changed', path: '', old: 'No', new: 'Yes' }] }], null);
  assert.equal(group.rows[0].label, 'Hide from apps');
  assert.equal(group.rows[0].text, 'No → Yes');
  assert.ok(Object.values(KEY_WORDS).every((word) => !word.includes('Apple Podcasts') && word !== 'Blocks'));
});

test('a word in a summary line keeps sentence case', () => {
  assert.equal(inSentence('Nostr key'), 'Nostr key');
  assert.equal(inSentence('Picture'), 'picture');
  assert.equal(inSentence('Information address'), 'information address');
  const change = (entry) => ({ op: 'changed', path: 'img', entry, old: null, new: 'https://a.example/p.jpg' });
  const [group] = fieldRows([{ kind: 'credits', name: 'Ann', changes: ['a', 'b', 'c'].map(change) }], null);
  assert.equal(group.summary, '3 credits get the picture https://a.example/p.jpg');
});

test('a build date in words shows the local time', () => {
  const old = 'Thu, 05 Nov 2026 14:20:10 +0000';
  const row = changeRow({ kind: 'field', name: 'last_build_date' }, { op: 'changed', path: '', old, new: 'Thu, 05 Nov 2026 14:43:54 +0000' }, {});
  assert.ok(row.text.startsWith(`${localTime(old)} → `), 'ADR 0114 item 14: the Simplified view shows each time in local time.');
  assert.ok(!row.text.includes('+0000'));
  const text = changeRow({ kind: 'field', name: 'title' }, { op: 'changed', path: '', old: 'May 5', new: 'June 6' }, {}).text;
  assert.equal(text, 'May 5 → June 6', 'A title that looks like a date stays as text.');
});

test('a merge plan gives the words of each row and waits for each choice', () => {
  const plan = {
    kept: [{ id: 'description', track: null, key: 'description', mine: 'Mine', hosted: 'Old' }],
    incoming: [{ id: 'items/a/title', track: 'One', key: 'title', mine: 'One', hosted: 'New' }],
    conflicts: [{ id: 'title', track: null, key: 'title', mine: 'A', hosted: 'B' }, { id: 'items/b', track: 'Two', key: '', mine: null, hosted: { title: 'Two' } }],
  };
  const rows = mergeRows(plan);
  assert.equal(rows.kept[0].label, 'Description');
  assert.equal(rows.incoming[0].label, 'One · Title');
  assert.deepEqual([rows.conflicts[1].mine, rows.conflicts[1].hosted], ['No track', 'Two']);
  assert.equal(mergeReady(plan, { title: 'mine' }), false, 'ADR 0129 item 5: each conflict needs a choice.');
  assert.equal(mergeReady(plan, { title: 'mine', 'items/b': 'hosted' }), true);
});

test('a merge row gives a list of credits as names and roles, not as JSON', () => {
  const credits = [{ name: 'Ann', role: 'Vocals', npub: null, href: null }, { name: 'Bo', role: null }];
  const rows = mergeRows({ conflicts: [{ id: 'credits', track: null, key: 'credits', mine: credits, hosted: [] }] });
  assert.equal(rows.conflicts[0].mine, 'Ann (Vocals), Bo');
  assert.equal(rows.conflicts[0].hosted, 'none');
});

test('a merge row gives an empty credit as No name, not as JSON', () => {
  const credits = [{ name: '', role: null, group: null, href: null, img: null, npub: null }, { name: 'Ann', role: 'Vocals' }];
  const rows = mergeRows({ conflicts: [{ id: 'credits', track: null, key: 'credits', mine: credits, hosted: [] }] });
  assert.equal(rows.conflicts[0].mine, 'No name, Ann (Vocals)');
});

test('Examine the Changes shows only with no result or with a result that is not current', () => {
  const diff = { changed: false };
  assert.equal(showsExamine({ merge: null, diff, stale: false }), false);
  assert.equal(showsExamine({ merge: null, diff: null, stale: false }), true);
  assert.equal(showsExamine({ merge: null, diff, stale: true }), true);
  assert.equal(showsExamine({ merge: {}, diff: null, stale: true }), false);
});

test('Review uses the shared segmented control for its view', () => {
  const source = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  const viewbar = source.match(/<div class="viewbar">([\s\S]*?)<\/div>\s*\$\{view === 'simplified'/)?.[1];
  assert.ok(viewbar, 'musicindex ADR 0051 R-SEG-1: keep the Review view bar.');
  assert.doesNotMatch(viewbar, /<select\b/, 'musicindex ADR 0051 R-SEG-1: replace the select with a segmented control.');
  assert.match(viewbar, /class="segmented"/, 'musicindex ADR 0051 R-SEG-1: use the segmented control class.');
  assert.match(viewbar, /VIEWS\.map/, 'musicindex ADR 0051 R-SEG-1: make each view a segment from VIEWS.');
  assert.match(viewbar, /onKeyDown=\$\{onSegmentKey\}/, 'musicindex ADR 0051 R-SEG-1: use onSegmentKey for keyboard focus.');
  assert.match(viewbar, /VIEW_HINTS\[view\]/, 'musicindex ADR 0051 R-SEG-1: keep the hint for the selected view.');
});

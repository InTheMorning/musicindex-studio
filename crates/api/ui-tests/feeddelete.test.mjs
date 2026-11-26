import { test } from 'node:test';
import assert from 'node:assert/strict';
import { deleteChoices, deleteButtonEnabled, deleteWrite, refusedDelete, retireSteps, retireCheckedLine, leftoverLine, operationNote, domainLine, DELETE_POINTS, retireActions, retireBlockShown, deletedNotice } from '../ui/feeddelete.js';
import { readFileSync } from 'node:fs';

test('the choice sheet gives the goals of ADR 0131 items 2 to 4 and the third row only after removal', () => {
  const fix = 'Fix DELETE_CHOICES and deleteChoices in crates/api/ui/feeddelete.js.';
  const all = deleteChoices({ removal_done: true });
  assert.deepEqual(all.map(({ action, title, text, destructive }) => [action, title, text, destructive]), [
    ['move', 'Put the feed on a different file host', 'Apps and Music Index follow. Nothing is lost.', false],
    ['retire', 'Take the feed out of the apps', 'The file stays. You can stop this later.', false],
    ['delete', 'Remove the file', 'Music Index does not hold this feed.', true],
  ], `ADR 0131 items 2 to 4: with removal_done, the sheet has three goal rows. ${fix}`);
  assert.ok(all.every((choice) => !('recommended' in choice)),
    `ADR 0131 item 1: no row has a tag. ${fix}`);
  for (const value of [null, { removal_done: false }, ...[
    'not_yet', 'held_with_other_guid', 'not_known_failed', 'not_known_no_guid', 'not_read',
  ].map((answer) => ({ removal_done: false, removal: { answer } }))]) {
    assert.deepEqual(deleteChoices(value).map(({ action }) => action), ['move', 'retire'],
      `ADR 0131 item 4 and ADR 0200 items 2 and 3: a feed that Music Index may hold has no delete row. ${fix}`);
  }
});

test('the domain line gives the words of ADR 0131 items 15 and 16 or null', () => {
  assert.deepEqual(domainLine('music.example', true), {
    title: 'Keep the domain music.example.',
    text: 'If it expires, its new owner can publish a feed with the same GUID and collect the payments that apps still send to it.',
  }, 'ADR 0131 item 16: a feed with a done delete has these words. Fix domainLine in crates/api/ui/feeddelete.js.');
  assert.deepEqual(domainLine('music.example'), {
    title: 'Keep the domain music.example.',
    text: 'If it expires, its new owner can take over this feed and its payments.',
  }, 'ADR 0131 item 15: the domain line has these words. Fix domainLine in crates/api/ui/feeddelete.js.');
  for (const value of [null, undefined, '']) {
    assert.equal(domainLine(value), null,
      'ADR 0131 item 18: a feed with no host has no domain line. Fix domainLine in crates/api/ui/feeddelete.js.');
  }
});

test('the last sheet gives the three state points of ADR 0131 item 10', () => {
  assert.deepEqual(DELETE_POINTS, [
    { mark: '✓', text: 'Music Index does not hold this feed' },
    { mark: '✓', text: 'Studio keeps a copy of the feed' },
    { mark: '↺', text: 'You can put the file back later with', button: 'Put the File Back', after: 'on this feed' },
  ], 'ADR 0131 item 10: three state points. Fix DELETE_POINTS in crates/api/ui/feeddelete.js.');
});

test('the Retire block gives Put the File Back only for a done delete', () => {
  assert.deepEqual(retireActions({ delete: 'done' }), ['restore', 'check'],
    'ADR 0198 item 9: a done delete shows restore before check. Fix retireActions in crates/api/ui/feeddelete.js.');
  for (const value of ['not_done', 'pending', undefined]) {
    assert.deepEqual(retireActions({ delete: value }), ['check', 'stop'],
      'ADR 0198 item 9: only a done delete shows restore. Fix retireActions in crates/api/ui/feeddelete.js.');
  }
});

test('editor.js has no loss list and uses DomainLine in the four places', () => {
  const editor = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  assert.ok(!editor.includes('You lose these things'),
    'ADR 0131 item 10: the list of losses is gone. Remove it from FeedDelete in crates/api/ui/editor.js.');
  const place = (from, to) => {
    const start = editor.indexOf(from);
    assert.ok(start >= 0, `${from} is not in editor.js`);
    return editor.slice(start, editor.indexOf(to, start));
  };
  const where = {
    'the choice sheet': place('aria-labelledby="delete-host-title"', 'aria-labelledby="delete-feed-title"'),
    'the last sheet': place('aria-labelledby="delete-feed-title"', '</dialog>'),
    'the Retire block': place('<div class="retire-head">', '<dialog ref=${firstRef}'),
    'the move sheet': place('aria-labelledby="move-release-title"', '</dialog>'),
  };
  for (const [name, text] of Object.entries(where)) {
    assert.ok(text.includes('<${DomainLine} domain=${feed.address_host}'),
      `ADR 0131 item 19: ${name} shows DomainLine. Add it in crates/api/ui/editor.js.`);
  }
});

const state = (extra = {}) => ({
  ask: 'written', removal: { answer: 'not_yet' }, removal_done: false, delete: 'not_done', ...extra,
});

test('Walk 2026-11-26 finding 9: the Retire check line uses the given local time', () => {
  const fix = 'Walk 2026-11-26 finding 9: show the last check time. Fix retireCheckedLine in feeddelete.js.';
  assert.equal(retireCheckedLine('11:06:23'), 'Checked at 11:06:23', fix);
  assert.equal(retireCheckedLine('01:06:23 PM'), 'Checked at 01:06:23 PM', fix);
  for (const time of [undefined, null, '']) assert.equal(retireCheckedLine(time), null, fix);
});

test('retire steps show the state of each step', () => {
  const steps = retireSteps(state({ removal: { answer: 'removed' }, removal_done: true, delete: 'done' }));
  assert.deepEqual(steps.map(({ done }) => done), [true, true, true]);
  assert.equal(steps[1].text, 'Music Index does not hold this feed.');
  assert.equal(steps[2].button, null);
});

test('step three has no button before removal and offers Delete the File after removal', () => {
  assert.equal(retireSteps(state())[2].button, null,
    'ADR 0200 item 5: step 3 has no button before removal_done. Fix retireSteps in crates/api/ui/feeddelete.js.');
  assert.equal(retireSteps(state({ removal: { answer: 'removed' }, removal_done: true }))[2].button, 'Delete the File');
});

test('step three has no button when Music Index holds a replaced GUID', () => {
  const result = retireSteps(state({ removal: { answer: 'held_with_other_guid', new_guid: 'new-guid' } }));
  assert.equal(result[1].text, 'Music Index holds this feed with a different GUID.');
  assert.equal(result[2].button, null,
    'ADR 0200 items 3 and 5: a GUID that Music Index replaced gives no delete. Fix retireSteps in crates/api/ui/feeddelete.js.');
});

test('the first sheet reads retire state when it opens and maps deleteChoices', () => {
  const editor = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  assert.ok(editor.includes('api(`/feeds/${feed.id}/retire/state`, { method: \'POST\' })'),
    'ADR 0200 item 4: the sheet reads the retire state when it opens. Fix FeedDelete in crates/api/ui/editor.js.');
  assert.ok(editor.includes('${deleteChoices(deleteState).map('),
    'ADR 0200 item 4: the sheet maps deleteChoices. Fix FeedDelete in crates/api/ui/editor.js.');
});

test('the final button waits for an exact, nonempty confirmation', () => {
  assert.equal(deleteButtonEnabled('Album', 'Album'), true);
  assert.equal(deleteButtonEnabled('Album', 'album'), false);
  assert.equal(deleteButtonEnabled('Album', 'Album '), false);
  assert.equal(deleteButtonEnabled('', ''), false);
  assert.equal(deleteButtonEnabled(null, ''), false);
});

test('operationNote marks only the row of the feed file', () => {
  assert.equal(operationNote({ kind: 'delete', deletes_feed_file: true }), 'This write deletes the feed file.');
  assert.equal(operationNote({ kind: 'delete', deletes_feed_file: false }), 'Studio deletes it after the feed is written.');
  assert.equal(operationNote({ kind: 'upload', deletes_feed_file: false }), 'The feed uses this file.');
});

test('ADR 0198 item 12 gives a destructive write for only deletions', () => {
  assert.deepEqual(deleteWrite({ changed: false, operations: [{ kind: 'delete', deletes_feed_file: false }] }), {
    label: 'Delete 1 File', destructive: true, cancel: null,
  }, 'ADR 0198 item 12: name one file. Fix deleteWrite in crates/api/ui/feeddelete.js.');
  assert.equal(deleteWrite({ changed: false, operations: Array.from({ length: 3 }, () => ({ kind: 'delete', deletes_feed_file: false })) }).label,
    'Delete 3 Files', 'ADR 0198 item 12: count delete operations. Fix deleteWrite in crates/api/ui/feeddelete.js.');
  for (const diff of [
    { operations: [{ kind: 'delete' }], changed: true },
    { changed: false, operations: [{ kind: 'delete' }], feed_move: {} },
    { changed: false, operations: [{ kind: 'upload' }, { kind: 'delete' }] },
    { changed: false, operations: [] },
  ]) assert.equal(deleteWrite(diff), null, 'ADR 0198 item 12: only unchanged feed deletions use this view. Fix deleteWrite in crates/api/ui/feeddelete.js.');
  assert.deepEqual(deleteWrite({ operations: [{ kind: 'delete', deletes_feed_file: true }] }), {
    label: 'Delete the Feed File', destructive: true, cancel: 'Cancel the Delete',
  }, 'ADR 0198 item 1: keep the feed delete review.');
  assert.equal(deleteWrite({ operations: [
    { kind: 'delete', deletes_feed_file: true },
    { kind: 'delete', deletes_feed_file: false },
    { kind: 'delete', deletes_feed_file: false },
  ] }).label, 'Delete 3 Files',
  'ADR 0198 item 1: count every delete operation when one marks the feed file.');
});

test('ADR 0036 item 13 refusedDelete gives warning rows', () => {
  assert.deepEqual(refusedDelete({ pointer: true }), {
    kind: 'Not deleted', tone: 'warning', text: 'Studio does not delete this file. It tells the apps the new address of the feed.',
  }, 'ADR 0036 item 13: name a refused pointer deletion. Fix refusedDelete in crates/api/ui/feeddelete.js.');
  assert.deepEqual(refusedDelete({ pointer: false, feed_titles: ['A', 'B'] }), {
    kind: 'Not deleted', tone: 'warning', text: 'Studio does not delete this file. These feeds use it: A, B',
  }, 'ADR 0036 item 13: name the feeds that use a file. Fix refusedDelete in crates/api/ui/feeddelete.js.');
});

test('ADR 0198 keeps the destructive review button first and guards the write', () => {
  const editor = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  const deleteStart = editor.indexOf('${deletion && diff && hasWork && !addressesBlocked && !choosing && html`<button class="button destructive"');
  const cancel = editor.indexOf('deletion.cancel : reviewRemoveLabel(');
  const usual = editor.indexOf('${!deletion && diff && hasWork && !addressesBlocked && !choosing && html`<button class="button primary"');
  assert.ok(deleteStart >= 0 && cancel > deleteStart,
    'ADR 0198 items 2 and 3: put the guarded destructive delete before Cancel the Delete.');
  assert.ok(usual > cancel && editor.includes("'Write to the File Host'"),
    'ADR 0198 item 4: keep Write to the File Host in the guarded primary path.');
});

test('ADR 0255 item 19 puts the podping choice in the footer and refused deletes are warnings', () => {
  const editor = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  const body = editor.slice(editor.indexOf('<div class="body">', editor.indexOf('function ReviewSheet')), editor.indexOf('<footer>', editor.indexOf('function ReviewSheet')));
  const footer = editor.slice(editor.indexOf('<footer>', editor.indexOf('function ReviewSheet')), editor.indexOf('</footer>', editor.indexOf('function ReviewSheet')));
  assert.ok(!body.includes('<${AnnounceChoice}') && !body.includes('<${AnnounceFooterChoice}'),
    'ADR 0255 item 19: the body of the review sheet has no check box of the announcement. Fix ReviewSheet in crates/api/ui/editor.js.');
  assert.ok(!editor.includes('AnnounceLines') && !editor.includes('announce-lines'),
    'ADR 0255 item 19: the review sheet has no row Announce. Remove it from crates/api/ui/editor.js.');
  assert.ok(footer.includes('skip=${announceSkipLine(diff)}'),
    'ADR 0255 item 19: the footer gives the cause of no podping in the position of the check box. Fix ReviewSheet in crates/api/ui/editor.js.');
  assert.ok(footer.includes('<${AnnounceFooterChoice}') && editor.includes('<span>${ANNOUNCE_FOOTER_LABEL}</span>'),
    'ADR 0255 item 19: put the labeled choice in the footer. Fix ReviewSheet in crates/api/ui/editor.js.');
  const blocked = editor.slice(editor.indexOf('function BlockedDeleteRow'), editor.indexOf('// Each host operation', editor.indexOf('function BlockedDeleteRow')));
  assert.ok(blocked.includes('role="status"') && !blocked.includes('role="alert"') && !blocked.includes('class="host-op-row error"'),
    'ADR 0036 item 13: refused deletion is a warning status row. Fix BlockedDeleteRow in crates/api/ui/editor.js.');
});

test('ADR 0198 hides actions that a deleted feed cannot use', () => {
  const editor = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  assert.ok(editor.includes('retireActions(retire).map(') && editor.includes('onClick=${stopRetire}>Stop Retiring</button>'),
    'ADR 0198 item 6: the Retire block takes its buttons from retireActions, which hides Stop Retiring after the delete. Fix FeedDelete in crates/api/ui/editor.js.');
  assert.ok(!retireActions({ delete: 'done' }).includes('stop'),
    'ADR 0198 item 6: hide Stop Retiring after the delete. Fix retireActions in crates/api/ui/feeddelete.js.');
  assert.ok(editor.includes('!feed.host_deleted && candidates.length > 0'),
    'ADR 0198 item 7: hide the linked page picker after the delete.');
  assert.ok(editor.includes('!feed.host_deleted && candidates.length === 0'),
    'ADR 0198 item 7: hide the linked page hint after the delete.');
  const picker = editor.indexOf('${!feed.host_deleted && candidates.length > 0 && html`<div class="field">');
  const link = editor.indexOf('>Link to This Page</button>', picker);
  assert.ok(picker >= 0 && link > picker,
    'ADR 0198 item 7: put Link to This Page inside the host_deleted guard.');
});

test('ADR 0198 reads the retire state again after a write', () => {
  const editor = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  assert.ok(editor.includes('}, [retiring, feed.id, written]);') && editor.includes('feed.last_version') && editor.includes('feed.host_deleted}`'),
    'ADR 0198 item 6: the Retire block must read its state again after a write. Add the version of the feed and host_deleted to the dependencies of the retire state effect in FeedDelete.');
  assert.ok(editor.includes('if (request === retireRequest.current) showRetire(state);'),
    'ADR 0198 item 6: a late answer must not show the state before the write. Set the retire state only from the answer of the last request.');
  assert.ok(editor.includes('onClick=${loadRetire}>Check Again</button>') && !editor.includes('setRetire(await api('),
    'ADR 0198 item 6: Check Again must use loadRetire. Do not set the retire state directly from an API answer.');
});

test('a leftover copy shows the row of the delete and the line of ADR 0228 item 20', () => {
  const fix = 'Fix leftoverLine and deleteChoices in crates/api/ui/feeddelete.js.';
  const leftover = state({ removal: { answer: 'leftover_copy', index_url: 'https://other.example/a.xml' }, removal_done: true });
  assert.equal(leftoverLine(leftover), 'Music Index reads this feed from https://other.example/a.xml. This file is a leftover copy.',
    `ADR 0228 item 20: the sheet gives this text. ${fix}`);
  assert.deepEqual(deleteChoices(leftover).map(({ action }) => action), ['move', 'retire', 'delete'],
    `ADR 0228 item 20: a leftover copy has the delete row. ${fix}`);
  assert.equal(retireSteps(leftover)[2].button, 'Delete the File');
  for (const value of [null, state(), state({ removal: { answer: 'removed' }, removal_done: true })]) {
    assert.equal(leftoverLine(value), null, `ADR 0228 item 20: only a leftover copy has the line. ${fix}`);
  }
});

test('the first delete sheet does not show the error of an earlier try', () => {
  const source = readFileSync(new URL('../ui/editor.js', import.meta.url), 'utf8');
  assert.match(source, /dismissed\.current = errorNow\.current/,
    'ADR 0261 item 24: the open handler of FeedDelete must keep the error that is there when the first sheet opens. Restore it in crates/api/ui/editor.js.');
  assert.match(source, /error=\$\{action\.error === dismissed\.current \? null : action\.error\}/,
    'ADR 0261 item 24: the first sheet must hide the dismissed error. Restore it in crates/api/ui/editor.js.');
});

test('ADR 0261 items 25 and 26: the sheets say that Music Index does not hold the feed', () => {
  const fix = 'Fix DELETE_CHOICES and DELETE_POINTS in crates/api/ui/feeddelete.js.';
  for (const text of [deleteChoices({ removal_done: true })[2].text, DELETE_POINTS[0].text]) {
    assert.match(text, /^Music Index does not hold this feed\.?$/,
      `ADR 0261 items 25 and 26: a plain 404 is not a removal by Music Index. ${fix}`);
  }
});

test('ADR 0261 item 27: the Retire block shows for a feed with a done delete that is not hidden', () => {
  const fix = 'Fix retireBlockShown in crates/api/ui/feeddelete.js and its use in crates/api/ui/editor.js.';
  assert.equal(retireBlockShown({ ask: 'no', delete: 'done' }, true), true,
    `ADR 0261 item 27: Put the File Back must show for a deleted feed that the draft does not hide. ${fix}`);
  assert.equal(retireBlockShown({ ask: 'no', delete: 'not_done' }, false), false,
    `ADR 0261 item 27: a feed that is not retiring and not deleted has no block. ${fix}`);
  assert.equal(retireBlockShown(null, true), false,
    `ADR 0261 item 27: the block waits for the retire state. ${fix}`);
  assert.deepEqual(retireActions({ ask: 'no', delete: 'done' }), ['restore', 'check'],
    `ADR 0198 item 6 and ADR 0261 item 27: a deleted feed has no Stop Retiring. ${fix}`);
});

test('ADR 0231 item 1: a deleted feed has a notice with two actions, a feed that is not deleted has none', () => {
  const fix = 'Fix deletedNotice in crates/api/ui/feeddelete.js and its use in OutsideNotices in crates/api/ui/app.js.';
  assert.deepEqual(deletedNotice({ host_deleted: true }), {
    text: 'The feed file of this release is deleted from its file host.',
    actions: ['restore', 'check'],
  }, `ADR 0231 item 1: the notice has the text and the actions Put the File Back and Check Again. ${fix}`);
  for (const feed of [{ host_deleted: false }, {}, null, undefined]) {
    assert.equal(deletedNotice(feed), null, `ADR 0231 item 1: a feed that is not deleted has no notice. ${fix}`);
  }
  const app = readFileSync(new URL('../ui/app.js', import.meta.url), 'utf8');
  assert.match(app, /deletedNotice\(feed\)/, `ADR 0231 item 1: the head of the page must show the notice. ${fix}`);
});

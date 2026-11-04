// The Node tests of the file drop (ADR 0125).
import test from 'node:test';
import assert from 'node:assert/strict';
import {
  hasFiles, dropHandlers, watchFileDrags, accountDropResult, FILE_DRAG_ATTRIBUTE,
  ACCOUNT_DROP_ONE, ACCOUNT_DROP_KIND,
} from '../ui/filedrop.js';

function dragEvent(types = ['Files'], files = []) {
  return {
    dataTransfer: { types, files, dropEffect: 'copy' },
    defaultPrevented: false,
    stopped: false,
    preventDefault() { this.defaultPrevented = true; },
    stopPropagation() { this.stopped = true; },
  };
}

function fakeWindow() {
  const listeners = {};
  return {
    timers: [],
    listeners,
    addEventListener(type, fn) { (listeners[type] ||= []).push(fn); },
    setTimeout(fn) { this.timers.push(fn); return this.timers.length; },
    clearTimeout() {},
    send(type, event) { for (const fn of listeners[type] || []) fn(event); },
  };
}

function fakeRoot() {
  const attributes = new Set();
  return {
    has: (name) => attributes.has(name),
    setAttribute: (name) => attributes.add(name),
    removeAttribute: (name) => attributes.delete(name),
  };
}

test('hasFiles needs the type Files', () => {
  assert.equal(hasFiles(dragEvent()), true);
  assert.equal(hasFiles(dragEvent(['text/uri-list'])), false);
  assert.equal(hasFiles({}), false);
});

test('a target takes a file drag, and gives the files of a drop', () => {
  let over = null;
  let got = null;
  const handlers = dropHandlers({ isBusy: () => false, setOver: (v) => { over = v; }, onFiles: (f) => { got = f; } });
  const hover = dragEvent();
  handlers.onDragOver(hover);
  assert.equal(hover.defaultPrevented, true);
  assert.equal(over, true);
  const drop = dragEvent(['Files'], ['a.mp3', 'b.mp3']);
  handlers.onDrop(drop);
  assert.equal(over, false);
  assert.deepEqual(got, ['a.mp3', 'b.mp3']);
});

test('a busy target does not take a drag or the files of a drop', () => {
  let got = null;
  const handlers = dropHandlers({ isBusy: () => true, setOver: () => {}, onFiles: (f) => { got = f; } });
  const hover = dragEvent();
  handlers.onDragOver(hover);
  assert.equal(hover.defaultPrevented, false);
  handlers.onDrop(dragEvent(['Files'], ['a.mp3']));
  assert.equal(got, null);
});

test('a target of a sheet stops the drag, so the field below does not get it', () => {
  const handlers = dropHandlers({ isBusy: () => false, setOver: () => {}, onFiles: () => {}, stop: true });
  const hover = dragEvent();
  handlers.onDragOver(hover);
  assert.equal(hover.stopped, true);
});

test('a target does not take a text drag', () => {
  const handlers = dropHandlers({ isBusy: () => false, setOver: () => {}, onFiles: () => {} });
  const hover = dragEvent(['text/plain']);
  handlers.onDragOver(hover);
  assert.equal(hover.defaultPrevented, false);
});

test('a file drag marks the root until no dragover comes', () => {
  const win = fakeWindow();
  const root = fakeRoot();
  watchFileDrags(win, root);
  win.send('dragover', dragEvent());
  assert.equal(root.has(FILE_DRAG_ATTRIBUTE), true);
  win.timers.at(-1)();
  assert.equal(root.has(FILE_DRAG_ATTRIBUTE), false);
});

test('a drag that leaves the window ends the mark, and a move to a different element does not', () => {
  const win = fakeWindow();
  const root = fakeRoot();
  watchFileDrags(win, root);
  win.send('dragover', dragEvent());
  win.send('dragleave', { relatedTarget: {} });
  assert.equal(root.has(FILE_DRAG_ATTRIBUTE), true);
  win.send('dragleave', { relatedTarget: null });
  assert.equal(root.has(FILE_DRAG_ATTRIBUTE), false, 'ADR 0125 item 3: the targets must not stay lit.');
});

test('a text drag does not mark the root, and the page does not refuse it', () => {
  const win = fakeWindow();
  const root = fakeRoot();
  watchFileDrags(win, root);
  const event = dragEvent(['text/plain']);
  win.send('dragover', event);
  assert.equal(root.has(FILE_DRAG_ATTRIBUTE), false);
  assert.equal(event.defaultPrevented, false);
});

test('a file drag that no target takes shows a refusal and does nothing', () => {
  const win = fakeWindow();
  watchFileDrags(win, fakeRoot());
  const event = dragEvent();
  win.send('dragover', event);
  assert.equal(event.defaultPrevented, true, 'ADR 0125 item 5: the browser must not open the file.');
  assert.equal(event.dataTransfer.dropEffect, 'none');
});

test('a file drag that a target took keeps the effect of the target', () => {
  const win = fakeWindow();
  watchFileDrags(win, fakeRoot());
  const event = dragEvent();
  event.preventDefault();
  win.send('dragover', event);
  assert.equal(event.dataTransfer.dropEffect, 'copy');
});

test('the account drop takes one .tar file', () => {
  const file = { name: 'studio.TAR' };
  assert.deepEqual(accountDropResult([file]), { file });
  assert.deepEqual(accountDropResult([file, file]), { error: ACCOUNT_DROP_ONE });
  assert.deepEqual(accountDropResult([]), { error: ACCOUNT_DROP_ONE });
  assert.deepEqual(accountDropResult([{ name: 'a.zip' }]), { error: ACCOUNT_DROP_KIND });
});

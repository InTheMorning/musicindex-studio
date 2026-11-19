import test from 'node:test';
import assert from 'node:assert/strict';
import { undoNoticeText } from '../ui/undonotice.js';

test('undo notices use the text of ADR 0227 item 13', () => {
  assert.equal(undoNoticeText.personDeleted('Ann'), 'Ann is deleted.');
  assert.equal(undoNoticeText.feedRemoved('Album'), 'Album is removed from your collection.');
  assert.equal(undoNoticeText.kindChanged('EP'), 'The kind is now EP.');
  assert.equal(undoNoticeText.nicknameChanged('Annie'), 'The nickname is now Annie.');
});

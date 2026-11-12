import { test } from 'node:test';
import assert from 'node:assert/strict';
import { libraryFolderRow } from '../ui/feedfacts.js';

test('the library folder row gives only its path, also away from the standard path (ADR 0180 item 4)', () => {
  const current = libraryFolderRow({
    library_folder: '/music/Artist/Album',
    default_library_folder: '/music/Artist/Album',
    library_folder_path: 'Artist/Album',
    default_library_folder_path: 'Artist/Album',
    library_folder_is_default: true,
  });
  assert.deepEqual(current, { path: '/music/Artist/Album' });

  const custom = libraryFolderRow({
    library_folder: '/music/Custom/Album',
    default_library_folder: '/music/Artist/Album',
    library_folder_path: 'Custom/Album',
    default_library_folder_path: 'Artist/Album',
    library_folder_is_default: false,
  });
  assert.deepEqual(custom, { path: '/music/Custom/Album' });
});

test('a server without library assets has no library folder row', () => {
  assert.equal(libraryFolderRow({
    library_folder: null,
    default_library_folder: null,
    library_folder_is_default: null,
    library_folder_path: null,
    default_library_folder_path: null,
  }), null);
});

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { libraryFolderRow } from '../ui/feedfacts.js';

test('the library folder row always gives Change and gives the standard folder only when needed', () => {
  const current = libraryFolderRow({
    library_folder: '/music/Artist/Album',
    default_library_folder: '/music/Artist/Album',
    library_folder_path: 'Artist/Album',
    default_library_folder_path: 'Artist/Album',
    library_folder_is_default: true,
  });
  assert.equal(current.moveToDefault, null);

  const custom = libraryFolderRow({
    library_folder: '/music/Custom/Album',
    default_library_folder: '/music/Artist/Album',
    library_folder_path: 'Custom/Album',
    default_library_folder_path: 'Artist/Album',
    library_folder_is_default: false,
  });
  assert.equal(custom.moveToDefault, 'Artist/Album');
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

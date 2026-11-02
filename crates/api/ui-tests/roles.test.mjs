import test from 'node:test';
import assert from 'node:assert/strict';
import { roleChoice, fromChoice, roleOptions, roleSummary } from '../ui/roles.js';

const LIST = [
  { name: 'Music', roles: ['Band', 'Vocalist', 'Musician'] },
  { name: 'Writing', roles: ['Editor', 'Author'] },
  { name: 'Video Post-Production', roles: ['Editor'] },
];

test('roleChoice gives the entry of the list with no regard to case', () => {
  assert.equal(roleChoice('vocalist', 'music', LIST), 'Music|Vocalist');
  assert.equal(roleChoice('  VOCALIST ', null, LIST), 'Music|Vocalist');
  assert.equal(roleChoice('Editor', 'video post-production', LIST), 'Video Post-Production|Editor');
});

test('roleChoice gives the first group for a group that is not a group of the role', () => {
  assert.equal(roleChoice('Editor', 'Music', LIST), 'Writing|Editor');
  assert.equal(roleChoice('Editor', '', LIST), 'Writing|Editor');
});

test('roleChoice gives other for a role not of the list and empty text for no role', () => {
  assert.equal(roleChoice('Field Recordist', 'music', LIST), 'other');
  assert.equal(roleChoice('', 'music', LIST), '');
  assert.equal(roleChoice(null, null, LIST), '');
  assert.equal(roleChoice('   ', null, LIST), '');
});

test('fromChoice gives the display names, and null for other', () => {
  assert.deepEqual(fromChoice('Music|Vocalist', LIST), { role: 'Vocalist', group: 'Music' });
  assert.deepEqual(fromChoice('Video Post-Production|Editor', LIST), { role: 'Editor', group: 'Video Post-Production' });
  assert.equal(fromChoice('other', LIST), null);
  assert.equal(fromChoice('', LIST), null);
  assert.equal(fromChoice('Music|Nothing', LIST), null);
});

test('roleOptions has a group for each group of the list, then the other entry', () => {
  const options = roleOptions(LIST);
  assert.equal(options.groups.length, 3);
  assert.deepEqual(options.groups[0].options[1], ['Music|Vocalist', 'Vocalist']);
  assert.deepEqual(options.other, ['other', 'Other role…']);
});

test('roleSummary shows the role and the group of the menu', () => {
  assert.equal(roleSummary('vocalist', null, LIST), 'Vocalist · Music');
  assert.equal(roleSummary('Field Recordist', 'Cast', LIST), 'Field Recordist · Cast');
  assert.equal(roleSummary('', '', LIST), '');
});

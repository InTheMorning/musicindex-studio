// The rules of the audio section (ADR 0062 items 16 and 23 to 29, ADR 0042).
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { propertiesSummary, propertiesOpen, needsMetadataRead, clockText, shortType } from '../ui/audio.js';

test('propertiesSummary gives the size in MB with one decimal and the short type', () => {
  assert.equal(propertiesSummary({ length: '3984588', media_type: 'audio/mpeg' }), '3.8 MB · MP3');
  assert.equal(propertiesSummary({ length: 3984588, media_type: 'audio/flac' }), '3.8 MB · FLAC');
  assert.equal(propertiesSummary({ length: '0', media_type: 'audio/mpeg' }), '0.0 MB · MP3');
});

test('propertiesSummary gives a dash for each missing part', () => {
  assert.equal(propertiesSummary({ length: null, media_type: 'audio/mpeg' }), '— · MP3');
  assert.equal(propertiesSummary({ length: '1048576', media_type: null }), '1.0 MB · —');
  assert.equal(propertiesSummary({ length: '', media_type: '' }), '— · —');
  assert.equal(propertiesSummary({ length: 'many', media_type: 'audio/mpeg' }), '— · MP3');
  assert.equal(propertiesSummary(null), '— · —');
});

test('shortType names a known type and shortens an unknown type', () => {
  assert.equal(shortType('audio/mp4'), 'M4A');
  assert.equal(shortType('audio/x-wav'), 'X-WAV');
  assert.equal(shortType(null), '');
});

test('propertiesOpen is false when the size and the type have a value and no error', () => {
  assert.equal(propertiesOpen({ length: '10', media_type: 'audio/mpeg' }, []), false);
  assert.equal(propertiesOpen({ length: 10, media_type: 'audio/mpeg' }), false);
});

test('propertiesOpen is true when the size or the type is empty', () => {
  assert.equal(propertiesOpen({ length: null, media_type: 'audio/mpeg' }, []), true);
  assert.equal(propertiesOpen({ length: '10', media_type: null }, []), true);
  assert.equal(propertiesOpen({ length: '10', media_type: '  ' }, []), true);
  assert.equal(propertiesOpen(null, []), true);
});

test('propertiesOpen is true when the size or the type has an error', () => {
  const full = { length: '10', media_type: 'audio/mpeg' };
  assert.equal(propertiesOpen(full, ['length']), true);
  assert.equal(propertiesOpen(full, ['media_type']), true);
  assert.equal(propertiesOpen(full, ['url']), false);
});

test('needsMetadataRead is true for This release and File hosts, and false for an upload', () => {
  assert.equal(needsMetadataRead('release'), true);
  assert.equal(needsMetadataRead('hosts'), true);
  assert.equal(needsMetadataRead('computer'), false);
  assert.equal(needsMetadataRead(undefined), false);
});

test('clockText gives M:SS or H:MM:SS, and a dash for no value', () => {
  assert.equal(clockText(0), '0:00');
  assert.equal(clockText(65.9), '1:05');
  assert.equal(clockText(3725), '1:02:05');
  assert.equal(clockText(NaN), '—');
  assert.equal(clockText(Infinity), '—');
});

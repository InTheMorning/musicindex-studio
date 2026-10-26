// The signing models (ADR 0042). Each test checks one rule of ADR 0050
// items 10 to 12.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  eventsToSign, signAll, signErrorText, signSummary, signProgressLabel,
  addressLines, invalidConfigField, blossomDefaultName, npubOf, siteAddress,
  siteNameFromGuid, isNsec,
} from '../ui/signing.js';

const ev = (id) => ({ id, pubkey: 'a'.repeat(64), created_at: 1, kind: 1, tags: [], content: id });
const diff = {
  events: [
    { connector_id: 1, user_signs: true, event: ev('e1') },
    { connector_id: 2, user_signs: false, event: ev('e2') },
    { connector_id: 1, user_signs: true, event: ev('e3') },
  ],
};

test('eventsToSign gives the events with user_signs, in their sequence', () => {
  assert.deepEqual(eventsToSign(diff).map((e) => e.id), ['e1', 'e3']);
  assert.deepEqual(eventsToSign({ events: [] }), []);
  assert.deepEqual(eventsToSign({}), []);
});

test('signAll signs each event in sequence and reports the progress', async () => {
  const seen = [];
  const progress = [];
  const signer = { signEvent: async (event) => { seen.push(event.id); return { ...event, sig: 's' }; } };
  const signed = await signAll(eventsToSign(diff), signer, (done, total) => progress.push([done, total]));
  assert.deepEqual(seen, ['e1', 'e3']);
  assert.deepEqual(signed.map((e) => e.id), ['e1', 'e3']);
  assert.deepEqual(progress, [[1, 2], [2, 2]]);
});

test('a signed event with a different ID gives changed_event', async () => {
  const signer = { signEvent: async (event) => ({ ...event, id: 'other', sig: 's' }) };
  await assert.rejects(signAll([ev('e1')], signer), (error) => error.code === 'changed_event');
});

test('a signer that refuses gives signer_refused and stops', async () => {
  let calls = 0;
  const signer = { signEvent: async () => { calls += 1; throw new Error('no'); } };
  await assert.rejects(signAll([ev('e1'), ev('e3')], signer), (error) => error.code === 'signer_refused');
  assert.equal(calls, 1);
});

test('the texts of the signer step', () => {
  assert.equal(signErrorText('changed_event'), 'Your signer changed an event. Studio wrote nothing.');
  assert.equal(signErrorText('signature_invalid'), null);
  assert.equal(signSummary(2), 'This write signs 2 events with your signer.');
  assert.equal(signSummary(1), 'This write signs 1 event with your signer.');
  assert.equal(signProgressLabel({ done: 1, total: 3 }), 'Signing 1 of 3…');
});

test('an address text gives one address for each line', () => {
  assert.deepEqual(addressLines(' wss://a \n\nwss://b\n'), ['wss://a', 'wss://b']);
  assert.deepEqual(addressLines(''), []);
});

test('a message of invalid_config names its field', () => {
  assert.equal(invalidConfigField('the site needs one relay'), 'relays');
  assert.equal(invalidConfigField('the host needs one server'), 'servers');
  assert.equal(invalidConfigField('the gateway must be a host name'), 'gateway');
  assert.equal(invalidConfigField('the site name is not valid'), 'd');
  assert.equal(invalidConfigField('the secret key must not be empty'), 'secret_key');
  assert.equal(invalidConfigField('something else'), null);
});

test('a Blossom host with no name takes the host name of the first server', () => {
  assert.equal(blossomDefaultName(['https://blossom.example.com:3000/x', 'https://b']), 'blossom.example.com:3000');
  assert.equal(blossomDefaultName([]), '');
});

test('npubOf encodes a public key as in NIP-19', () => {
  assert.equal(
    npubOf('3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d'),
    'npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64th6gkwsyjh6w6',
  );
  assert.equal(npubOf('abc'), null);
});

test('siteAddress gives the gateway address with 50 base36 characters', () => {
  const address = siteAddress({ public_key: 'f'.repeat(64), d: 'album', gateway: 'gw.example' });
  assert.match(address, /^https:\/\/[0-9a-z]{50}album\.gw\.example$/);
  assert.ok(siteAddress({ public_key: '0'.repeat(63) + '1', d: 'a' }).startsWith(`https://${'0'.repeat(49)}1a.nsite.lol`));
  assert.equal(siteAddress({ public_key: 'x', d: 'a' }), null);
});

test('siteNameFromGuid gives the site name of the core', () => {
  // The same cases as the core test of site_name_from_guid.
  assert.equal(siteNameFromGuid('ABCDEF12-3456-7890-abcd-ef1234567890'), 'abcdef12');
  assert.equal(siteNameFromGuid('abcdefghijkl-x'), 'abcdefgh');
  assert.equal(siteNameFromGuid('short'), 'short');
});

test('isNsec is true only for an nsec', () => {
  assert.equal(isNsec(' nsec1qyqsqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqsqzz4zq7 '), true);
  // 64 hexadecimal characters go to the lookup as a public key.
  assert.equal(isNsec('01'.repeat(32)), false);
  assert.equal(isNsec('npub1rwzv24nmzfjypx2a8m264ws9vht3uxp5vpypnluuzl67n4waq78suk0wul'), false);
});

// ---- The NIP-46 signer of the workspace (task 035d) ----

import { remoteSigner, signerChoice, signerView } from '../ui/signing.js';

test('remoteSigner works with signAll and calls the API function for each event', async () => {
  const sent = [];
  const signer = remoteSigner(async (event) => { sent.push(event.id); return { ...event, sig: 's' }; });
  const progress = [];
  const signed = await signAll(eventsToSign(diff), signer, (done, total) => progress.push([done, total]));
  assert.deepEqual(sent, ['e1', 'e3']);
  assert.deepEqual(signed.map((e) => e.sig), ['s', 's']);
  assert.deepEqual(progress, [[1, 2], [2, 2]]);
});

test('remoteSigner keeps the checks of signAll', async () => {
  const changed = remoteSigner(async (event) => ({ ...event, id: 'other' }));
  await assert.rejects(signAll([ev('e1')], changed), (error) => error.code === 'changed_event');
  const refused = remoteSigner(async () => { throw new Error('409'); });
  await assert.rejects(signAll([ev('e1')], refused), (error) => error.code === 'signer_refused');
});

test('signerChoice picks the signer of a write', () => {
  const events = [ev('e1')];
  const key = events[0].pubkey;
  const connected = { state: 'connected', pubkey: key };
  assert.equal(signerChoice({ hasNip07: true, remote: null, events }).kind, 'nip07');
  assert.equal(signerChoice({ hasNip07: false, remote: connected, events: [] }).kind, 'none');
  assert.equal(signerChoice({ hasNip07: false, remote: null, events }).kind, 'loading');
  assert.equal(signerChoice({ hasNip07: false, remote: { state: 'none', pubkey: null }, events }).kind, 'connect');
  assert.equal(signerChoice({ hasNip07: false, remote: { state: 'pending', pubkey: null }, events }).kind, 'connect');
  assert.equal(signerChoice({ hasNip07: false, remote: connected, events }).kind, 'remote');
  const other = { state: 'connected', pubkey: 'b'.repeat(64) };
  assert.equal(signerChoice({ hasNip07: false, remote: other, events }).kind, 'differs');
});

test('signerView reads the answer of the signer route', () => {
  assert.equal(signerView({ state: 'pending', pubkey: null }).phase, 'waiting');
  assert.deepEqual(signerView({ state: 'connected', pubkey: 'k' }), { phase: 'connected', pubkey: 'k', error: null });
  assert.deepEqual(signerView({ state: 'none', error: 'signer_timeout' }), { phase: 'failed', pubkey: null, error: 'signer_timeout' });
  assert.equal(signerView({ state: 'none', pubkey: null }).phase, 'idle');
  assert.equal(signerView(null).phase, 'idle');
});

test('the texts of the NIP-46 signer step', () => {
  assert.match(signErrorText('signer_key_differs'), /different key/);
  assert.notEqual(signErrorText('signer_timeout'), null);
});

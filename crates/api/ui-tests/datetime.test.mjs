// The date and time models (ADR 0093, ADR 0042).
process.env.TZ = 'Europe/Berlin';
const { test } = await import('node:test');
const assert = (await import('node:assert/strict')).default;
const { readFileSync, readdirSync } = await import('node:fs');
const { localInputToIso, isoToLocalInput, offsetText, zoneHint, nowInput, unreadDateHint } =
  await import('../ui/datetime.js');

test('the offset comes from the selected date', () => {
  assert.equal(localInputToIso('2026-03-28T12:00'), '2026-03-28T12:00:00+01:00');
  assert.equal(localInputToIso('2026-03-30T12:00'), '2026-03-30T12:00:00+02:00');
  assert.equal(localInputToIso('2026-10-30T20:00'), '2026-10-30T20:00:00+01:00');
});

test('an empty or incorrect value gives null', () => {
  assert.equal(localInputToIso(''), null);
  assert.equal(localInputToIso('not a time'), null);
  assert.equal(localInputToIso(null), null);
  assert.equal(localInputToIso('2026-02-31T10:00'), null);
});

test('the local input and the ISO text convert back', () => {
  assert.equal(isoToLocalInput('2026-10-18T00:00:00.000Z'), '2026-10-18T02:00');
  const v = '2026-10-30T20:00';
  assert.equal(isoToLocalInput(localInputToIso(v)), v);
  assert.equal(isoToLocalInput('x'), '');
  assert.equal(isoToLocalInput(null), '');
});

test('the offset text', () => {
  assert.equal(offsetText(60), 'UTC+01:00');
  assert.equal(offsetText(-330), 'UTC-05:30');
  assert.equal(offsetText(0), 'UTC+00:00');
});

test('the zone hint', () => {
  const now = new Date('2026-01-15T12:00:00Z');
  assert.equal(zoneHint('2026-07-01T12:00:00+02:00', 'Europe/Berlin', now), 'Europe/Berlin, UTC+02:00');
  assert.equal(zoneHint(null, 'Europe/Berlin', now), 'Europe/Berlin, UTC+01:00');
});

test('the value of Now has the precision of one minute', () => {
  assert.equal(nowInput(new Date('2026-10-30T19:00:45Z')), '2026-10-30T20:00');
});

test('the hint of a date that the core cannot read', () => {
  assert.equal(unreadDateHint('last Tuesday'),
    'The feed has the date “last Tuesday”. Studio cannot read it. Select a time to replace it.');
});

test('only DateTimeField of app.js has a datetime-local input', () => {
  const dir = new URL('../ui/', import.meta.url);
  for (const name of readdirSync(dir).filter((n) => n.endsWith('.js'))) {
    const count = (readFileSync(new URL(name, dir), 'utf8').match(/type="datetime-local"/g) || []).length;
    const allowed = name === 'app.js' ? 1 : 0;
    assert.equal(count, allowed,
      `ADR 0093 item 1: ${name} makes a date and time field of its own. Use DateTimeField of app.js.`);
  }
});

// The model of the People area (ADR 0098, ADR 0099, ADR 0042). The functions
// have no Preact and no browser state. `peoplepages.js` shows the pages. The
// functions give texts and states only. The core applies each feed rule and
// gives each outcome (ADR 0065).

import { shortKey } from './nostrkey.js';

const plural = (n, one, many) => `${n} ${n === 1 ? one : many}`;
const feedCount = (n) => plural(n, 'feed', 'feeds');

// The text of the empty list of people (ADR 0099 item 2).
export const PEOPLE_EMPTY = 'No people yet. Check a suggestion above, or add a person.';

// The IDs of the sections of the People page, in the sequence of ADR 0099
// item 2. A section with no data is left out, but `people` always shows,
// because it gives the empty text. With no person, the suggestions come first.
export function peopleSections({ people = [], suggestions = [], changes = [], payees = [] } = {}) {
  const ids = ['search', 'actions'];
  if (changes.length > 0) ids.push('changes');
  const hasSuggestions = suggestions.length > 0;
  if (hasSuggestions && people.length === 0) ids.push('suggestions');
  ids.push('people');
  if (hasSuggestions && people.length > 0) ids.push('suggestions');
  if (payees.length > 0) ids.push('payees');
  return ids;
}

// The title of a feed from a Map or from a plain object, or a fallback.
const titleOf = (titles, id) => {
  const found = titles instanceof Map ? titles.get(id) : (titles || {})[id];
  return found || `Feed ${id}`;
};

// The text of one address. An address with "@" is a Lightning address. The
// other addresses are Lightning node keys (ADR 0099 item 15). This gives a
// display value only. `type` is the recipient type, when the caller has it.
export function addressText(key, type = '') {
  const address = key.address || '';
  const isAddress = type ? type === 'lnaddress' : address.includes('@');
  return isAddress ? `Lightning address ${address}` : `Lightning node ${shortKey(address)}`;
}

// The second line of a suggestion row: the first evidence.
const EVIDENCE = {
  same_npub: 'The same Nostr key is in the credits',
  same_name: 'The same name is in the credits and in the value blocks',
  same_wallet: 'The same wallet is in the value blocks',
};

// The row of a suggestion: the name, the counts and the evidence (ADR 0099
// item 2). A count of 0 is left out.
export function suggestionRow(suggestion) {
  const counts = [];
  if (suggestion.credit_feeds > 0) counts.push(`Credit in ${feedCount(suggestion.credit_feeds)}`);
  if (suggestion.payee_feeds > 0) counts.push(`Payee in ${feedCount(suggestion.payee_feeds)}`);
  const lines = [];
  if (counts.length > 0) lines.push(counts.join(' · '));
  const first = (suggestion.evidence || [])[0];
  if ((suggestion.recipients || []).length === 0 || !EVIDENCE[first]) {
    lines.push('No payment address found');
  } else {
    lines.push(EVIDENCE[first]);
  }
  return { title: suggestion.name, lines };
}

// The recipient type of the default address of a person, from its wallets.
function defaultType(person) {
  const key = person.default_address;
  const found = (person.wallets || []).find((w) => w.key.address === key.address
    && (w.key.custom_key || null) === (key.custom_key || null)
    && (w.key.custom_value || null) === (key.custom_value || null));
  return found ? found.recipient_type : '';
}

// The number of distinct feeds of a person. A list entry gives `feeds`. A
// full entry gives the arrays `pays` and `credited`.
function feedsOf(person) {
  if (typeof person.feeds === 'number') return person.feeds;
  const ids = new Set([...(person.pays || []), ...(person.credited || [])]);
  return ids.size;
}

// The row of a person: the title, the lines and the count (ADR 0099 item 2).
// `feedTitles` maps the ID of a publisher feed to its title.
export function personRow(person, feedTitles) {
  const lines = [];
  const membership = (person.members || []).map((id) => `Member of ${titleOf(feedTitles, id)}`);
  const first = [person.usual_role, ...membership].filter((t) => t && t.trim() !== '');
  if (first.length > 0) lines.push(first.join(' · '));
  const wallets = (person.wallets || []).length;
  if (person.default_address) {
    const more = wallets >= 2 ? ` · ${wallets} wallets` : '';
    lines.push(`Pays ${addressText(person.default_address, defaultType(person))}${more}`);
  } else {
    lines.push('No payment address');
  }
  return { title: person.name, lines, count: feedCount(feedsOf(person)) };
}

// The start state of the sheet "Make a Person" (ADR 0099 item 3). An item
// with a different name starts not selected. The sheet has `memberOf` as text.
export function makePersonStart(suggestion) {
  const npubs = [...new Set((suggestion.credits || []).map((c) => c.npub).filter(Boolean))];
  return {
    name: suggestion.name,
    credits: (suggestion.credits || []).map((item) => ({ item, checked: item.same_name !== false })),
    recipients: (suggestion.recipients || []).map((item) => ({ item, checked: item.same_name !== false })),
    keys: npubs.map((item) => ({ item, checked: true })),
    memberOf: '',
  };
}

// The body of `POST /people/suggestions/make`. Only the checked items go in.
export function makePersonBody(state) {
  const memberOf = String(state.memberOf ?? '').trim();
  return {
    name: state.name,
    credits: state.credits.filter((r) => r.checked).map((r) => [r.item.name, r.item.href, r.item.npub]),
    recipients: state.recipients.filter((r) => r.checked).map((r) => [r.item.key, r.item.recipient_type]),
    keys: state.keys.filter((r) => r.checked).map((r) => r.item),
    member_of: memberOf === '' ? null : Number(memberOf),
  };
}

// The rows of the feeds in a sheet of an operation. Each row starts selected
// (ADR 0099 items 5 and 7). `places` is the list `places` of a person.
export function sheetFeeds(places, titles) {
  return (places || []).map((place) => {
    const lines = (place.pays || []).map((key) => `Pays ${addressText(key)}`);
    for (const role of place.roles || []) lines.push(role === '' ? 'Credited' : `Credited as ${role}`);
    return { id: place.feed_id, title: titleOf(titles, place.feed_id), lines, checked: true };
  });
}

// The IDs of the selected rows.
export const checkedIds = (rows) => rows.filter((r) => r.checked).map((r) => r.id);

// A copy of the rows with the selection of one row turned over.
export const toggledRow = (rows, id) => rows.map((r) => (r.id === id ? { ...r, checked: !r.checked } : r));

// The text of the primary button of a sheet (ADR 0099 item 12).
export const reviewLabel = (n) => `Review in ${plural(n, 'Feed', 'Feeds')}`;

// The `PersonOperation` for the route (task 064). The core checks each value.
export function operationBody(kind, fields) {
  switch (kind) {
    case 'use_address':
      return { kind, key: fields.key };
    case 'rename':
      return { kind, name: fields.name, payment: !!fields.payment, credit: !!fields.credit };
    case 'add_to_releases': {
      const percent = String(fields.percent ?? '').trim();
      return {
        kind,
        payee_percent: fields.payee && percent !== '' ? Number(percent) : null,
        credit: !!fields.credit,
        support_link: fields.supportLink ?? null,
      };
    }
    case 'remove':
      return { kind, payee: !!fields.payee, credit: !!fields.credit };
    default:
      throw new Error(`The operation ${kind} is not an operation of a person. Use use_address, rename, add_to_releases or remove.`);
  }
}

const PART_TEXT = { payee: 'get the payee', credit: 'get the credit', support_link: 'get the support link' };
const PART_ORDER = ['payee', 'credit', 'support_link'];
const ALREADY = ['have it already', 'have both already', 'have all already'];

// The parts of an outcome, from the top level or the `parts` list.
const partsOf = (outcome) => outcome.parts || [];
const causeName = (cause) => (typeof cause === 'string' ? cause : (cause && cause.cause) || '');

// The result line of the sheet "Add to All Releases" (ADR 0099 item 10),
// from the outcomes of the preview. A count of 0 is left out. The core gives
// each outcome. This function only counts.
export function addResultLine(outcomes) {
  const selected = PART_ORDER.filter((p) => outcomes.some((o) => partsOf(o).some((x) => x.part === p)));
  const pieces = [];
  if (outcomes.length > 0) pieces.push(plural(outcomes.length, 'release', 'releases'));
  for (const part of selected) {
    const n = outcomes.filter((o) => partsOf(o).some((x) => x.part === part && x.changed)).length;
    if (n > 0) pieces.push(`${n} ${PART_TEXT[part]}`);
  }
  const already = outcomes.filter((o) => {
    const parts = partsOf(o);
    return selected.every((part) => {
      const x = parts.find((y) => y.part === part);
      return x && !x.changed && causeName(x.cause).startsWith('has_');
    });
  }).length;
  if (selected.length > 0 && already > 0) pieces.push(`${already} ${ALREADY[selected.length - 1]}`);
  return pieces.join(' · ');
}

// ---- The People page (ADR 0099 items 1 to 3 and 12) ----

const lower = (text) => String(text ?? '').toLowerCase();
const matchesAny = (texts, query) => {
  const q = lower(query).trim();
  return q === '' || texts.some((t) => lower(t).includes(q));
};

// The initials of a name, for a person with no picture.
export function initials(name) {
  return String(name ?? '').split(/\s+/).filter(Boolean).map((w) => Array.from(w)[0]).slice(0, 2).join('').toUpperCase();
}

// True when a person matches the text of the search field: the name, an
// address or a Nostr key, with no regard to capital letters. This is a
// display filter only.
export function personMatches(person, query) {
  const texts = [person.name, ...(person.wallets || []).map((w) => w.key.address), ...(person.keys || []).map((k) => k.npub)];
  return matchesAny(texts, query);
}

// True when a wallet with no person matches the text of the search field.
export function payeeMatches(payee, query) {
  const texts = [payee.name, payee.address, ...(payee.addresses || []).map((a) => a.address)];
  return matchesAny(texts, query);
}

export const filterPeople = (people, query) => (people || []).filter((p) => personMatches(p, query));
export const filterPayees = (payees, query) => (payees || []).filter((p) => payeeMatches(p, query));

// The wallets of `GET /recipients` that have no person (ADR 0099 item 2).
export const payeesWithNoPerson = (recipients) => (recipients || []).filter((r) => r.saved_id === null || r.saved_id === undefined);

// The change sets of `GET /change-sets` that are not closed.
export const openChanges = (sets) => (sets || []).filter((s) => s.open);

// The title of the operation of a change set. The page "Review the changes"
// and the list "Changes to write" use it (ADR 0099 items 2 and 12).
export function changeTitle(operation) {
  switch (operation && operation.kind) {
    case 'change_address': return 'Change address';
    case 'rename': return 'Rename';
    case 'remove': return 'Remove';
    case 'add': return 'Add to an artist or label';
    case 'use_lnaddress': return 'Use the Lightning address';
    case 'use_address': return 'Use This Address';
    case 'rename_person': return 'Rename';
    case 'add_to_releases': return 'Add to All Releases';
    case 'remove_person': return 'Remove';
    default: return 'Change';
  }
}

// The number of feeds of a change set that Studio has still to write.
export function feedsToWrite(set) {
  return (set.feeds || []).filter((f) => f.outcome && f.outcome.outcome === 'changed' && f.written_at == null).length;
}

// The row of a change set in "Changes to write". `people` is the list of
// people, and `payees` is the list of wallets, for the name of the subject.
export function changeRow(set, people = [], payees = []) {
  let who = '';
  if (set.person_id != null) {
    const person = people.find((p) => p.id === set.person_id);
    who = person ? person.name : 'A person';
  } else {
    const key = set.recipient || {};
    const same = (a, b) => (a || '') === (b || '');
    const found = payees.find((r) => r.address === key.address && same(r.custom_key, key.custom_key) && same(r.custom_value, key.custom_value));
    who = found && found.name && found.name.trim() !== '' ? found.name : key.address || 'A wallet';
  }
  return { title: `${changeTitle(set.operation)} · ${who}`, text: `${feedCount(feedsToWrite(set))} to write` };
}

// The view that opens after Discard: the page of the person of a person
// change set, else the People page (ADR 0099 item 12).
export const afterDiscard = (set) => (set && set.person_id != null ? { name: 'person', id: set.person_id } : { name: 'people' });

// The choices of "Member of": each publisher feed, A to Z, as
// [value, label] pairs. `None` is the placeholder of the menu.
export function memberChoices(feeds) {
  return (feeds || []).filter((f) => f.medium === 'publisher')
    .sort((a, b) => (a.title || '').localeCompare(b.title || '', undefined, { sensitivity: 'base' }))
    .map((f) => [String(f.id), f.title || 'Untitled feed']);
}

// The title of the sheet "Make a Person".
export const makeSheetTitle = (name) => `Make a Person: ${name}`;

// The hint of the sheet "Make a Person" (ADR 0099 item 3).
export const makeSheetHint = (name) => `Studio found these items with the name "${name}". Select the items that are this person. Studio changes no feed.`;

// The row of a credit in the sheet "Make a Person".
export function creditItemRow(item, titles) {
  const feeds = (item.feeds || []).map((id) => titleOf(titles, id));
  const lines = [];
  if (feeds.length > 0) lines.push(feeds.join(', '));
  if (item.same_name === false) lines.push(`The name in that credit is "${item.name}"`);
  return { title: (item.roles || []).length > 0 ? item.roles.join(', ') : 'Credit', lines };
}

// The row of a payment address in the sheet "Make a Person".
export function payeeItemRow(item) {
  const count = (item.feeds || []).length;
  const lines = [feedCount(count)];
  if (item.same_name === false) lines[0] += ` · the name in that feed is "${item.name}"`;
  return { title: addressText(item.key, item.recipient_type), lines };
}

// The row of a Nostr key in the sheet "Make a Person".
export const keyItemRow = (npub) => ({ title: shortKey(npub), lines: ['In a credit of the feeds'] });

// A copy of the state of the sheet with one item turned over. `group` is
// `credits`, `recipients` or `keys`.
export function toggledItem(state, group, index) {
  return { ...state, [group]: state[group].map((r, i) => (i === index ? { ...r, checked: !r.checked } : r)) };
}

// The body of `POST /people/suggestions/dismiss`.
export const dismissBody = (suggestion) => ({ identity: suggestion.identity, items: suggestion.item_keys || [] });

// The body of `POST /people` from the sheet "Add a Person".
export const addPersonBody = (name) => ({ name: String(name ?? '').trim() });

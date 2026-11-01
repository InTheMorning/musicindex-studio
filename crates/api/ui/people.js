// The model of the People area (ADR 0098, ADR 0099, ADR 0042). The functions
// have no Preact and no browser state. `peoplepages.js` shows the pages. The
// functions give texts and states only. The core applies each feed rule and
// gives each outcome (ADR 0065).

import { shortKey } from './nostrkey.js';
import { feedKind, isPublisher } from './shell.js';

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

// ---- The page of a person and its sheets (ADR 0099 items 4 to 12, task 065c) ----

// The key of an address as the value of a select menu, and back.
export const keyValue = (key) => JSON.stringify([key.address, key.custom_key || '', key.custom_value || '']);
export function keyFromValue(value) {
  const [address, customKey, customValue] = JSON.parse(value);
  return { address, custom_key: customKey === '' ? null : customKey, custom_value: customValue === '' ? null : customValue };
}

// The type of a member of a wallet. An address with "@" is a Lightning
// address when the member gives no type.
const isLnaddress = (m) => (m.recipient_type ? m.recipient_type === 'lnaddress' : (m.address || '').includes('@'));

// The parts of the line of one address (ADR 0099 item 15). `full` shows a
// node key in full. A node of a shared provider names the provider.
export function memberLine(member, full = true) {
  const account = member.custom_value || null;
  if (isLnaddress(member)) return { label: 'Lightning address', address: member.address, account: null };
  if (member.provider) return { label: `${member.provider} node`, address: full ? member.address : null, account };
  // The account names the wallet on a shared node, so the line keeps it.
  return { label: 'Lightning node', address: full ? member.address : shortKey(member.address), account };
}

// One line of text for an address, for a select menu.
export function memberText(member) {
  const { label, address, account } = memberLine(member, false);
  return `${label}${address ? ` ${address}` : ''}${account ? ` · account ${account}` : ''}`;
}

const sameParts = (a, b) => a.address === b.address && (a.custom_key || '') === (b.custom_key || '')
  && (a.custom_value || '') === (b.custom_value || '');

// The wallets of a person, each with its addresses. `recipients` is the list
// of `GET /recipients`, which gives each address of a wallet with its
// provider and its feeds. A wallet that the list does not have gives its own
// key as its one address.
export function walletGroups(person, recipients) {
  return (person.wallets || []).map((wallet) => {
    const entry = (recipients || []).find((r) => (r.addresses || []).some((m) => sameParts(m, wallet.key)));
    const members = entry
      ? entry.addresses.map((m) => ({
        key: { address: m.address, custom_key: m.custom_key ?? null, custom_value: m.custom_value ?? null },
        member: m,
        count: m.feeds || 0,
      }))
      : [{ key: wallet.key, member: { ...wallet.key, recipient_type: wallet.recipient_type }, count: 0 }];
    members.sort((a, b) => isLnaddress(b.member) - isLnaddress(a.member));
    return { id: wallet.id, members };
  });
}

// The menu entries for each address of each wallet, as [value, text].
export const addressOptions = (groups) => groups.flatMap((g) => g.members.map((m) => [keyValue(m.key), memberText(m.member)]));

// The text of the count of feeds of an address.
export const usedText = (count) => (count > 0 ? feedCount(count) : 'Not used');

// The text for the evidence of a Nostr key of a person.
export function keyEvidenceText(evidence) {
  switch (evidence) {
    case 'keysend': return 'Found in a lookup of a Lightning address';
    case 'nostr': return 'Found in a Nostr profile';
    default: return 'Added by you';
  }
}

// The lines of the section "Where used" on the page of a person.
export function whereRow(person) {
  const pays = (person.pays || []).length;
  const credited = (person.credited || []).length;
  return {
    title: 'Where used',
    lines: [`Pays in ${feedCount(pays)} · Credited in ${feedCount(credited)}`],
    count: feedCount(typeof person.feeds === 'number' ? person.feeds : new Set([...(person.pays || []), ...(person.credited || [])]).size),
  };
}

// The rows of the feeds of a person: the type, then the places (ADR 0099
// item 5). `feeds` is the feed list of the shell.
export function feedRows(places, feeds) {
  const byId = new Map((feeds || []).map((f) => [f.id, f]));
  const titles = new Map((feeds || []).map((f) => [f.id, f.title || 'Untitled feed']));
  return sheetFeeds(places, titles).map((row) => {
    const feed = byId.get(row.id);
    return feed ? { ...row, lines: [feedKind(feed).label, ...row.lines] } : row;
  });
}

// The rows of the releases of an artist or label, for the sheet "Add to All
// Releases". Each row starts selected.
export function levelRows(ids, feeds) {
  const byId = new Map((feeds || []).map((f) => [f.id, f]));
  // Add to All Releases acts on the releases of the level, not on its
  // artist or label feeds (ADR 0098 item 20).
  return (ids || []).filter((id) => !(byId.has(id) && isPublisher(byId.get(id)))).map((id) => {
    const feed = byId.get(id);
    return { id, title: feed ? feed.title || 'Untitled feed' : `Feed ${id}`, lines: feed ? [feedKind(feed).label] : [], checked: true };
  });
}

// The heading of the list of feeds in a sheet.
export const feedsHeading = (rows) => `Feeds · ${checkedIds(rows).length} of ${rows.length} selected`;

export const SHEET_TITLES = {
  use_address: 'Use This Address',
  rename: 'Rename',
  add_to_releases: 'Add to All Releases',
  remove: 'Remove',
};

export const addIntro = (name) => `Studio adds ${name} to each release of the artist or label that you select.`;
export const useHint = (name) => `Each address of ${name} in the selected feeds changes to this address. The names and the splits do not change.`;
export const renameHint = 'Studio changes the person and the selected names. You see each feed in the review before Studio writes it.';
export const removeHint = (name) => `Studio removes ${name} from the selected feeds. Studio changes the splits of the other recipients, and their total does not change.`;

// The start values of the fields of a sheet.
export function sheetStart(kind, person, groups) {
  switch (kind) {
    case 'use_address': {
      const options = addressOptions(groups);
      const current = person.default_address ? keyValue(person.default_address) : '';
      return { key: options.some(([v]) => v === current) ? current : (options[0] || [''])[0] };
    }
    case 'rename': return { name: person.name, payment: true, credit: true };
    case 'add_to_releases': return { owner: '', payee: (person.wallets || []).length > 0, percent: '', credit: true, supportLink: '' };
    default: return { payee: true, credit: true };
  }
}

// The body of the route of a change set, from the fields of a sheet.
export function sheetBody(kind, fields) {
  if (kind === 'use_address') return operationBody(kind, { key: keyFromValue(fields.key) });
  if (kind === 'add_to_releases') {
    return operationBody(kind, { ...fields, supportLink: fields.supportLink === '' ? null : Number(fields.supportLink) });
  }
  return operationBody(kind, fields);
}

// True when the primary button of a sheet works: one feed is selected, and
// the sheet has one part (ADR 0099 item 7). The core checks each value.
export function sheetReady(kind, fields, ids) {
  if (ids.length === 0) return false;
  switch (kind) {
    case 'use_address': return fields.key !== '';
    case 'rename': return fields.name.trim() !== '' && (fields.payment || fields.credit);
    case 'add_to_releases': return (fields.payee && String(fields.percent).trim() !== '') || fields.credit || fields.supportLink !== '';
    default: return fields.payee || fields.credit;
  }
}

// The menu of the support links of a person, as [value, text].
export const supportChoices = (person) => (person.support_links || []).map((l) => [String(l.id), l.label || l.url]);

// The text of the primary button while the request runs.
export const BUSY_REVIEW = 'Making the Changes…';

// The forms that add a link to a person (ADR 0099 item 4). `fields` gives
// the id, the label, the kind and the start value of each field.
export const LINK_FORMS = {
  wallets: {
    title: 'Add a Wallet',
    button: 'Add the Wallet',
    fields: [
      { id: 'recipient_type', label: 'Type', select: [['lnaddress', 'Lightning Address'], ['node', 'Lightning Node']], start: 'lnaddress' },
      { id: 'address', label: 'Address', required: true },
      { id: 'custom_key', label: 'Custom key', node: true, hint: 'Some nodes need a custom key and an account to find the wallet.' },
      { id: 'custom_value', label: 'Account', node: true },
    ],
  },
  'credit-names': {
    title: 'Add a Credit Name',
    button: 'Add the Credit Name',
    fields: [
      { id: 'name', label: 'Name', required: true },
      { id: 'href', label: 'Information address' },
      { id: 'npub', label: 'Nostr key' },
    ],
  },
  keys: {
    title: 'Add a Nostr Key',
    button: 'Add the Key',
    fields: [{ id: 'npub', label: 'Nostr key', required: true }],
  },
  'support-links': {
    title: 'Add a Support Link',
    button: 'Add the Link',
    fields: [
      { id: 'label', label: 'Label' },
      { id: 'url', label: 'Address', required: true },
    ],
  },
};

// The fields of a link form that show, from the type of a wallet.
export const linkFields = (kind, values) => LINK_FORMS[kind].fields
  .filter((f) => !f.node || values.recipient_type === 'node');

export const linkStart = (kind) => Object.fromEntries(LINK_FORMS[kind].fields.map((f) => [f.id, f.start ?? '']));

// The body of the POST of a link. An empty custom key or account is none.
export function linkBody(kind, values) {
  const body = {};
  for (const f of linkFields(kind, values)) {
    const text = String(values[f.id] ?? '').trim();
    body[f.id] = f.node && text === '' ? null : text;
  }
  return body;
}

export const linkReady = (kind, values) => linkFields(kind, values).every((f) => !f.required || String(values[f.id] ?? '').trim() !== '');

// The body of `PATCH /people/{id}`: the four fields, with one changed.
export const personFieldsBody = (person, change = {}) => ({
  name: person.name, usual_role: person.usual_role, img: person.img, href: person.href, ...change,
});

// The line of the usual role of a person, and the publisher feeds of which
// the person is a member, as `{ role, members: [{ id, title }] }`.
export function profileLine(person, titles) {
  return {
    role: (person.usual_role || '').trim(),
    members: (person.members || []).map((id) => ({ id, title: titleOf(titles, id) })),
  };
}

const joinList = (items) => (items.length < 2 ? items.join('') : `${items.slice(0, -1).join(', ')} and ${items[items.length - 1]}`);

// The line under the title of the review of a change set of a person
// (ADR 0099 item 12, task 065 item 6). `set` is a change set with a
// `person_id`, and `name` is the name of the person now.
export function changeLine(set, name) {
  const op = set.operation || {};
  const who = name || 'the person';
  const n = (set.feeds || []).length;
  switch (op.kind) {
    case 'use_address': {
      const { label, address } = memberLine({ address: op.address, recipient_type: op.recipient_type, custom_value: op.custom_value }, false);
      const target = op.recipient_type === 'node' && op.custom_value ? `${label} ${address} with account ${op.custom_value}` : `${label} ${address}`;
      return `Use ${target} for ${who}`;
    }
    case 'rename_person': {
      const parts = [op.payment && 'payment name', op.credit && 'credit name'].filter(Boolean);
      const to = who === op.name ? `Rename to ${op.name}` : `Rename ${who} to ${op.name}`;
      return `${to} in ${feedCount(n)}${parts.length > 0 ? ` (${joinList(parts)})` : ''}`;
    }
    case 'add_to_releases': {
      const parts = [];
      if (op.payee) parts.push(`a payee with ${op.payee.percent}%`);
      if (op.credit) parts.push('a credit');
      if (op.support_link) parts.push('a support link');
      return `Add ${who} as ${joinList(parts)} in ${feedCount(n)}`;
    }
    case 'remove_person': {
      const parts = [op.payee && 'a payee', op.credit && 'a credit'].filter(Boolean);
      return `Remove ${who} as ${joinList(parts)} from ${feedCount(n)}`;
    }
    default: return '';
  }
}

// The model of the People area (ADR 0098, ADR 0099, ADR 0042). The functions
// have no Preact and no browser state. `peoplepages.js` shows the pages. The
// functions give texts and states only. The core applies each feed rule and
// gives each outcome (ADR 0065).

import { shortKey } from './nostrkey.js';
import { feedKind, isPublisher } from './shell.js';
import { placesText } from './recipients.js';

const plural = (n, one, many) => `${n} ${n === 1 ? one : many}`;
const feedCount = (n) => plural(n, 'feed', 'feeds');

// The text of the empty list of people (ADR 0099 item 2).
export const PEOPLE_EMPTY = 'No people yet. Check a suggestion above, or add a person.';

// The line at the top of the People page on a server (ADR 0108 item 3).
export const PEOPLE_SERVER_NOTE = 'The operator of this server can read the people that you keep.';

// The IDs of the sections of the People page, in the sequence of ADR 0099
// item 2. A section with no data is left out, but `people` always shows,
// because it gives the empty text. With no person, the suggestions come first.
export function peopleSections({ people = [], suggestions = [], changes = [], payees = [], server = false } = {}) {
  // On a server, the page starts with the line of ADR 0108 item 3.
  const ids = [...(server ? ['note'] : []), 'search', 'actions'];
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

// The start state of the sheet "Check: {name}" (ADR 0111 item 7). An item
// with a different name starts not selected. The sheet has `memberOf` as text.
// `profiles` is the result of `profileMap`, or null before it is known. A key
// whose profile name differs from the name starts not selected (ADR 0103
// item 18).
export function makePersonStart(suggestion, profiles = null) {
  const npubs = [...new Set((suggestion.credits || []).map((c) => c.npub).filter(Boolean))];
  return {
    name: suggestion.name,
    credits: (suggestion.credits || []).map((item) => ({ item, checked: item.same_name !== false })),
    recipients: (suggestion.recipients || []).map((item) => ({ item, checked: item.same_name !== false })),
    keys: npubs.map((item) => ({ item, checked: keyWarning(item, profiles, suggestion.name) === null })),
    memberOf: '',
  };
}

// The state of the sheet after the profiles arrive. A key with a warning is
// not selected. The other keys keep their state.
export const withProfiles = (state, profiles) => ({
  ...state,
  keys: state.keys.map((r) => (keyWarning(r.item, profiles, state.name) === null ? r : { ...r, checked: false })),
});

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
    case 'add': return 'Add to All Releases';
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

// The view in the place of the page of a person that is gone: Back can open
// the page of a person after a merge or a delete. `merged` maps the ID of a
// merged person to the ID of the person that kept its items.
export const afterMissing = (id, merged) => (merged.has(String(id)) ? { name: 'person', id: merged.get(String(id)) } : { name: 'people' });

// The choices of "Member of": each publisher feed, A to Z, as
// [value, label] pairs. A feed with a role shows its first role, for example
// "HeyCitizen · Artist" (ADR 0111 item 3). `None` is the placeholder of the
// menu.
export function memberChoices(feeds) {
  return (feeds || []).filter((f) => f.medium === 'publisher')
    .sort((a, b) => (a.title || '').localeCompare(b.title || '', undefined, { sensitivity: 'base' }))
    .map((f) => {
      const title = f.title || 'Untitled feed';
      return [String(f.id), (f.publisher_roles || []).length > 0 ? `${title} · ${firstRole(f)}` : title];
    });
}

// The first role of a feed, as the label of its type.
const firstRole = (feed) => feedKind(feed).label.split(' · ')[0];

// The change of the membership when the user picks a feed in "Member of": the
// feed to add, and the feeds to remove. An empty pick removes each membership.
export function memberChange(person, picked) {
  const target = picked === '' || picked == null ? null : Number(picked);
  const current = person.members || [];
  return { add: target !== null && !current.includes(target) ? target : null, remove: current.filter((id) => id !== target) };
}

// ---- The profiles of Nostr keys (ADR 0099 item 3, ADR 0103 items 17 and 18) ----

export const NO_PROFILE = 'No Nostr profile found';
export const NIP05_OK = 'NIP-05 checked';

// The body of `POST /nostr/resolve`: each key one time, 50 keys at most.
export const resolveBody = (npubs) => ({ npubs: [...new Set((npubs || []).filter(Boolean))].slice(0, 50) });

// A Map from key to profile, from the answer of `POST /nostr/resolve`.
export const profileMap = (profiles) => new Map((profiles || []).map((p) => [p.npub, p]));

const nameKey = (text) => String(text ?? '').trim().replace(/\s+/g, ' ').toLowerCase();

// The profile of a key from `profileMap`, or null before the answer or when
// the answer failed.
const profileOf = (npub, profiles) => (profiles instanceof Map ? profiles.get(npub) || null : null);

// The profile name of a key, or '' with no profile name.
export const profileName = (npub, profiles) => ((profileOf(npub, profiles) || {}).name || '').trim();

// The warning of ADR 0103 item 18 for a key, or null. A key with no profile
// name has no warning, and a profile name that contains the name has none.
export function keyWarning(npub, profiles, name) {
  const found = profileName(npub, profiles);
  const wanted = nameKey(name);
  if (found === '' || wanted === '' || nameKey(found).includes(wanted)) return null;
  return `The profile name is not "${name}". Check this key.`;
}

// The row of a Nostr key: the profile name and the short key, or the short
// key only while the profiles load or when they failed (ADR 0099 item 3).
// `picture` is the address of the profile picture, or null. `short` is the
// short key, which the row keeps on one line.
export function keyRow(npub, profiles, name = '') {
  const short = shortKey(npub);
  const profile = profileOf(npub, profiles);
  if (!profile) return { title: short, short, mono: true, picture: null, lines: [], warning: null };
  if (!profileName(npub, profiles)) {
    return { title: short, short, mono: true, picture: profile.picture || null, lines: [NO_PROFILE], warning: null };
  }
  const lines = [short];
  if (profile.nip05_ok) lines.push(NIP05_OK);
  return { title: profile.name.trim(), short, mono: false, picture: profile.picture || null, lines, warning: keyWarning(npub, profiles, name) };
}

// The note of a suggested Lightning address of a profile (task 067d item 4).
export function profileAddressNote(npub, profiles) {
  const short = shortKey(npub);
  const name = profileName(npub, profiles);
  return `From the Nostr profile ${name ? `${name} (${short})` : short} · keysend checked`;
}

// The row of a credit in the sheet "Check: {name}".
export function creditItemRow(item, titles) {
  const feeds = (item.feeds || []).map((id) => titleOf(titles, id));
  const lines = [];
  if (feeds.length > 0) lines.push(feeds.join(', '));
  if (item.same_name === false) lines.push(`The name in that credit is "${item.name}"`);
  return { title: (item.roles || []).length > 0 ? item.roles.join(', ') : 'Credit', lines };
}

// The row of a payment address in the sheet "Check: {name}".
export function payeeItemRow(item) {
  const count = (item.feeds || []).length;
  const lines = [feedCount(count)];
  if (item.same_name === false) lines[0] += ` · the name in that feed is "${item.name}"`;
  return { title: addressText(item.key, item.recipient_type), lines };
}

// The row of a Nostr key in the sheet "Check: {name}".
export const keyItemRow = (npub, profiles = null, name = '') => {
  const row = keyRow(npub, profiles, name);
  return { ...row, lines: [...row.lines, 'In a credit of the feeds'] };
};

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
// A line of a payee with " · fee" when `GET /recipients` marks the member
// as a fee in one or more feeds (the mockups of ADR 0099).
export const withFee = (text, member) => `${text}${member && member.fee ? ' · fee' : ''}`;

export function memberText(member) {
  const { label, address, account } = memberLine(member, false);
  return `${label}${address ? ` ${address}` : ''}${account ? ` · account ${account}` : ''}`;
}

const sameParts = (a, b) => a.address === b.address && (a.custom_key || '') === (b.custom_key || '')
  && (a.custom_value || '') === (b.custom_value || '');

// The wallets of a person, each with its addresses. A wallet gives its
// `members` (ADR 0095 item 5). `recipients` is the list of `GET /recipients`.
// It gives the provider and the feed count of an address. A wallet with no
// `members` gives its own key as its one address.
export function walletGroups(person, recipients) {
  const listed = (recipients || []).flatMap((r) => r.addresses || []);
  return (person.wallets || []).map((wallet) => {
    const given = wallet.members && wallet.members.length > 0
      ? wallet.members
      : [{ key: wallet.key, recipient_type: wallet.recipient_type }];
    const members = given.map((m) => {
      const entry = listed.find((l) => sameParts(l, m.key));
      const key = { address: m.key.address, custom_key: m.key.custom_key ?? null, custom_value: m.key.custom_value ?? null };
      const member = { ...key, recipient_type: m.recipient_type };
      if (entry && entry.provider) member.provider = entry.provider;
      if (entry && entry.fee) member.fee = entry.fee;
      return { key, member, count: entry ? entry.feeds || 0 : 0 };
    });
    members.sort((x, y) => isLnaddress(y.member) - isLnaddress(x.member));
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

// The navigation row "Check your feeds" on the page of a person (ADR 0111
// item 12). `needs_correction` is the count from `GET /people/{id}`. A line
// with `error` shows in the error color.
export function checkRow(person) {
  const pays = (person.pays || []).length;
  const credited = (person.credited || []).length;
  const bad = person.needs_correction || 0;
  return {
    title: 'Check your feeds',
    lines: [
      bad > 0 ? { text: `${bad === 1 ? '1 feed needs' : `${bad} feeds need`} a correction`, error: true } : { text: 'Each feed is correct', error: false },
      { text: `Pays in ${feedCount(pays)} · Credited in ${feedCount(credited)}`, error: false },
    ],
    count: feedCount(feedsOf(person)),
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
  remove: 'Remove from Feeds',
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

// The body of `PATCH /people/{id}`: the fields, with some changed.
export const personFieldsBody = (person, change = {}) => ({
  name: person.name, usual_role: person.usual_role, usual_group: person.usual_group ?? '', img: person.img, href: person.href, ...change,
});

// The line of the usual role of a person, and the publisher feeds of which
// the person is a member, as `{ role, members: [{ id, title, dot, role }] }`.
// `feeds` is the feed list of the shell. A member with a feed in the list
// also gets the dot class and the first role of the feed (ADR 0111 item 1).
export function profileLine(person, titles, feeds = []) {
  const byId = new Map((feeds || []).map((f) => [f.id, f]));
  return {
    role: (person.usual_role || '').trim(),
    members: (person.members || []).map((id) => {
      const member = { id, title: titleOf(titles, id) };
      const feed = byId.get(id);
      if (feed) {
        member.dot = feedKind(feed).dot;
        if ((feed.publisher_roles || []).length > 0) member.role = firstRole(feed);
      }
      return member;
    }),
  };
}

// The hint of the first field of a person (ADR 0111 item 2).
export const FIELDS_HINT = 'Studio saves each field when you leave it.';

// The hint of "Member of" (ADR 0111 item 3).
export const MEMBER_HINT = 'The artist or label that this person is in. Studio changes no feed.';

// The status of a field after a save. `save` is null before a save, or
// `{ key, ok: true }`, or `{ key, ok: false, message }` for the field `key`.
// The result is `{ text, error }`. A field that was not saved has no text.
export function saveStatus(save, key) {
  if (!save || save.key !== key) return { text: '', error: false };
  return save.ok ? { text: 'Saved', error: false } : { text: save.message || 'Studio could not save this field', error: true };
}

// ---- The suggestions of the picture and the information address (ADR 0113) ----

export const DETAIL_HEADINGS = {
  img: 'Suggestions for the picture',
  href: 'Suggestions for the information address',
};

// The collapsed line of a list of N values.
export const detailCollapsedLine = (count) => (count === 1
  ? '1 other value in your feeds and profiles'
  : `${count} other values in your feeds and profiles`);

// The line of one source of a suggestion. `field` is `img` or `href`.
// `profiles` is the `profileMap` or null, `titles` maps a feed ID to its
// title, and `feeds` is the feed list. The word of a feed is its first role in
// lowercase, or "feed" with no role.
export function detailSourceText(source, field, profiles, titles, feeds = []) {
  const byId = new Map((feeds || []).map((f) => [f.id, f]));
  if (source.kind === 'profile') {
    return `From the Nostr profile ${profileName(source.npub, profiles) || shortKey(source.npub)}`;
  }
  if (source.kind === 'credit') {
    const ids = source.feed_ids || [];
    const names = ids.map((id) => titleOf(titles, id)).join(', ');
    return ids.length === 1 ? `In the credit of 1 feed: ${names}` : `In the credits of ${ids.length} feeds: ${names}`;
  }
  const feed = byId.get(source.feed_id);
  const role = feed && (feed.publisher_roles || []).length > 0 ? firstRole(feed).toLowerCase() : '';
  const what = field === 'img' ? 'The image' : 'The link';
  return `${what} of the ${role ? `${role} feed` : 'feed'} ${titleOf(titles, source.feed_id)}`;
}

// The source line of a suggestion. Two or more sources join with " · ". A
// source after the first starts with a lowercase letter.
export function detailSourceLine(sources, field, profiles, titles, feeds = []) {
  return (sources || []).map((s, i) => {
    const text = detailSourceText(s, field, profiles, titles, feeds);
    return i === 0 ? text : text.charAt(0).toLowerCase() + text.slice(1);
  }).join(' · ');
}

// The list of suggestions for one field, as `{ state, heading, collapsedLine,
// rows: [{ value, line }] }`. `state` is `open` for an empty field with
// suggestions, `collapsed` for a filled field with suggestions, and `none`
// with no suggestion. `detail` is the answer of the core, or null.
export function detailList(field, current, detail, profiles, titles, feeds = []) {
  const list = (detail && detail[field]) || [];
  const rows = list.map((s) => ({ value: s.value, line: detailSourceLine(s.sources, field, profiles, titles, feeds) }));
  let state = 'none';
  if (rows.length > 0) state = String(current ?? '').trim() === '' ? 'open' : 'collapsed';
  return { state, heading: DETAIL_HEADINGS[field], collapsedLine: detailCollapsedLine(rows.length), rows };
}

// ---- The page "Check your feeds" (ADR 0111 items 9 to 13) ----

export const CHECK_GROUPS = [
  ['needs_correction', 'Needs a correction'],
  ['not_default', 'Not your default address'],
  ['correct', 'Correct'],
];

// The intro of the page.
export const checkIntro = (name) => `Studio compares each feed with ${name}: the Nostr key and the picture of each credit, and the address of each payment.`;

// The groups of the check, in the sequence of ADR 0111 item 11. A group with
// no feed is left out.
export function checkGroups(check) {
  const feeds = (check && check.feeds) || [];
  return CHECK_GROUPS.map(([state, title]) => ({ state, title, feeds: feeds.filter((f) => f.state === state) }))
    .filter((g) => g.feeds.length > 0);
}

// The line of one finding as `{ text, error }`. An error finding shows in the
// error color. `not_default` is the neutral finding. `profiles` is the
// `profileMap` of the shown keys, or null. The core finds each difference.
export function findingLine(finding, name, profiles = null, state = 'needs_correction') {
  switch (finding.kind) {
    case 'credit_key': {
      if (!finding.npub) return { text: 'No Nostr key in the credit', error: true };
      const who = profileName(finding.npub, profiles) || shortKey(finding.npub);
      return { text: `Nostr key of ${who} · not the key of ${name}`, error: true };
    }
    case 'credit_details': {
      const text = [finding.img && 'picture', finding.href && 'information address'].filter(Boolean).join(' and ');
      return { text: `The ${text} in the credit differs from ${name}`, error: true };
    }
    case 'credit_role': return { text: 'No role in the credit', error: true };
    case 'not_default': {
      // In its own group, the line says that the wallet is the same.
      const tail = state === 'not_default' ? 'the same wallet as your default address' : 'not your default address';
      return { text: `Pays ${addressText(finding.key)} · ${tail}`, error: false };
    }
    case 'other_wallet': return { text: `Pays ${addressText(finding.key)} · not a wallet of ${name}`, error: true };
    default: return { text: '', error: false };
  }
}

// The lines of a feed row of the check: the lines of the places, then one line
// for each finding.
export function checkFeedLines(entry, placeLines, name, profiles = null) {
  return [...placeLines.map((text) => ({ text, error: false })),
    ...(entry.findings || []).map((f) => findingLine(f, name, profiles, entry.state))];
}

// The keys of the findings, for the profile lookup.
export const checkKeys = (check) => [...new Set(((check && check.feeds) || [])
  .flatMap((f) => f.findings || []).filter((f) => f.kind === 'credit_key' && f.npub).map((f) => f.npub))];

// The lines of the sections "Change your feeds" (ADR 0111 item 4).
export const CHANGE_ROWS = [
  ['use_address', 'Pay a different address of this person in your feeds.'],
  ['rename', 'Change the name of this person in the payments and the credits.'],
  ['add_to_releases', 'Add this person to each release of an artist or label.'],
  ['remove', 'Remove this person from the payments or the credits of your feeds.'],
];
export const CHANGE_NOTE = 'Each operation shows the feeds and a review before Studio writes.';

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
    case 'correct': {
      const parts = [op.npub && 'the Nostr key', op.payment && 'the payment address', (op.img || op.href) && 'the picture and the information address',
        op.role && 'the role', op.other_payees && 'the payees with this name'].filter(Boolean);
      return `Correct ${joinList(parts)} of ${who} in ${feedCount(n)}`;
    }
    default: return '';
  }
}

// ---- The page "Check your feeds": the correction (ADR 0111 items 14 to 17, task 075c) ----

// The number of feeds that "Correct N Feeds" names: each feed that is not
// correct. The preview of the core leaves out a feed with no change.
export const correctCount = (check) => ((check && check.feeds) || []).filter((f) => f.state !== 'correct').length;

export const correctButtonLabel = (n) => `Correct ${plural(n, 'Feed', 'Feeds')}`;
export const correctTitle = (name) => `Correct the Feeds: ${name}`;
export const CORRECT_OMITTED = 'A feed with no change is not in the list.';
export const CORRECT_EMPTY = 'No feed changes with these corrections.';

// The IDs of the feeds that the preview covers: each feed that is not correct.
export const correctFeedIds = (check) => ((check && check.feeds) || []).filter((f) => f.state !== 'correct').map((f) => f.feed_id);

// The first Nostr key of a person, or an empty text.
const firstNpub = (person) => ((person.keys || [])[0] || {}).npub || '';

// The changes that a person can select. A change that needs a key or a
// wallet shows only when the person has one.
export function correctAvailable(person) {
  const wallets = (person.wallets || []).length > 0;
  return { key: (person.keys || []).length > 0, payment: wallets, details: true, role: true, other_payees: wallets };
}

// The start state of the sheet. Each change starts selected, except the
// payees with this name, because that change moves a payment to a different
// wallet.
export function correctStart(person) {
  const can = correctAvailable(person);
  return {
    key: can.key, npub: firstNpub(person), payment: can.payment, details: true, role: true, other_payees: false,
  };
}

// The check boxes of the sheet: `id`, `label` and `line`.
// The line of the change "other_payees". `count` is the number of the
// findings `other_wallet` of the check, and the address is the default
// address of the person. The texts are display values only.
export function otherPayeesLine(name, defaultAddress, count) {
  const to = defaultAddress ? addressText(defaultAddress).replace(/^Lightning (address|node) /, '') : 'the default address';
  const head = count === 1 ? `1 payment to the name "${name}" goes to an address that is not in this person. Studio replaces it`
    : count > 1 ? `${count} payments to the name "${name}" go to an address that is not in this person. Studio replaces them`
      : `A payment to the name "${name}" can go to an address that is not in this person. Studio replaces it`;
  return `${head} with ${to}. Select this only when that address is yours too.`;
}

// The number of the findings `other_wallet` in the check of a person.
export const otherWalletCount = (check) => ((check && check.feeds) || [])
  .reduce((n, f) => n + (f.findings || []).filter((x) => x.kind === 'other_wallet').length, 0);

export function correctOptions(person, fields, profiles = null, check = null) {
  const can = correctAvailable(person);
  const who = profileName(fields.npub, profiles) || shortKey(fields.npub || '');
  const role = trimmed(person.usual_role);
  const all = [
    ['key', 'Nostr key', `Each credit of ${person.name} gets ${who}`],
    ['payment', 'Payment address', `Each payment of ${person.name} uses the default address`],
    ['details', 'Picture and information address', 'Each credit gets the picture and the information address of this person'],
    ['role', 'Role', role === '' ? 'Each credit with no role gets the usual role' : `Each credit with no role gets ${role}`],
    ['other_payees', `Replace other addresses named "${person.name}"`, otherPayeesLine(person.name, person.default_address, otherWalletCount(check))],
  ];
  return all.filter(([id]) => can[id]).map(([id, label, line]) => ({ id, label, line }));
}

// The choices of the menu "Key", or an empty list for a person with one key.
export const correctKeyChoices = (person, profiles = null) => ((person.keys || []).length < 2 ? []
  : person.keys.map((k) => [k.npub, profileName(k.npub, profiles) || shortKey(k.npub)]));

// The `PersonOperation` of the correction. The core checks each value.
export function correctBody(fields) {
  return {
    kind: 'correct',
    key: fields.key && fields.npub ? fields.npub : null,
    payment: !!fields.payment,
    details: !!fields.details,
    role: !!fields.role,
    other_payees: !!fields.other_payees,
  };
}

// True when the sheet selects one change at least.
export const correctAnySelected = (fields) => !!(fields.key || fields.payment || fields.details || fields.role || fields.other_payees);

// The feeds of the preview that the core changes, as the rows of the sheet.
// A feed with the cause `no_credit`, `no_recipient` or `has_new_address` is
// "Not changed", and the sheet leaves it out (ADR 0111 item 16). Any other
// feed that the core does not change is also left out, and `omitted` counts
// them. `placeRows` maps the ID of a feed to its row with `title` and
// `lines`. `before` is the list of the rows now, so that a selection stays
// after a new preview.
// The change of the sheet "Correct the Feeds" that corrects each kind of
// finding of the check (ADR 0111 items 9 and 15).
const CORRECTED_BY = {
  credit_key: 'key', credit_details: 'details', credit_role: 'role', not_default: 'payment', other_wallet: 'other_payees',
};

// The lines of the findings of one feed that the selected changes correct.
// The findings come from the check of the core. This is a display aid.
export function correctFindingLines(entry, fields, name, profiles = null) {
  return ((entry && entry.findings) || [])
    .filter((f) => fields[CORRECTED_BY[f.kind]])
    .map((f) => findingLine(f, name, profiles, entry.state).text)
    .filter(Boolean);
}

// The rows of the sheet "Correct the Feeds". `check`, `fields`, `name` and
// `profiles` give the findings that the selected changes correct.
export function correctRows(preview, titles, placeRows, before = [], check = null, fields = {}, name = '', profiles = null) {
  const checks = new Map(((check && check.feeds) || []).map((f) => [f.feed_id, f]));
  const kept = new Map(before.map((r) => [r.id, r.checked]));
  const rows = [];
  let omitted = 0;
  for (const entry of (preview && preview.feeds) || []) {
    const outcome = entry.outcome || {};
    if (outcome.outcome !== 'changed') {
      omitted += 1;
      continue;
    }
    const place = placeRows.get(entry.feed_id);
    rows.push({
      id: entry.feed_id,
      title: place ? place.title : titleOf(titles, entry.feed_id),
      checked: kept.has(entry.feed_id) ? kept.get(entry.feed_id) : true,
      lines: [[...(place ? place.lines.slice(0, 1) : []), placesText(outcome)].filter(Boolean).join(' · '),
        ...correctFindingLines(checks.get(entry.feed_id), fields, name, profiles)].filter(Boolean),
    });
  }
  return { rows, omitted };
}

// ---- The recipient picker (ADR 0099 item 19, task 066a) ----

const sameKey = (a, b) => a.address === b.address && (a.custom_key || null) === (b.custom_key || null)
  && (a.custom_value || null) === (b.custom_value || null);

// The identity of a recipient in a list: its address, its custom key and its
// custom value (ADR 0027 item 3).
export const recipientIdentity = (r) => `${r.address || ''}\u0000${r.custom_key || ''}\u0000${r.custom_value || ''}`;

// The new recipient from a person: the name of the person and the values of
// the default address. It is a start value, with no split (ADR 0030).
export function recipientFromPerson(person) {
  const key = person.default_address || {};
  const wallet = (person.wallets || []).find((w) => sameKey(w.key, key));
  // A default address that joins a wallet through the keysend lookup is not
  // a wallet row. Its start type follows the rule of the core: an address
  // with "@" is a Lightning address, else a node key (task 064b).
  const type = wallet && wallet.recipient_type ? wallet.recipient_type : (key.address || '').includes('@') ? 'lnaddress' : 'node';
  return {
    name: person.name, recipient_type: type, address: key.address ?? null, split: '',
    custom_key: key.custom_key ?? null, custom_value: key.custom_value ?? null, fee: null,
  };
}

// The rows of the recipient picker. `people` is the list of `GET /people`
// and `recipients` is the list of `GET /recipients`. A person with no wallet
// or no default address has no row. A payee row is an address with no person.
export function recipientPickRows(people, recipients) {
  const rows = (people || []).filter((p) => p.default_address && (p.wallets || []).length > 0).map((person) => {
    const start = recipientFromPerson(person);
    const found = (recipients || []).flatMap((e) => e.addresses || []).find((m) => sameKey(m, start));
    const member = { ...start, provider: found ? found.provider : null };
    return { id: person.id, name: person.name, img: person.img, line: `Adds ${memberText(member)}`, identity: recipientIdentity(start), recipient: start, person };
  });
  const payees = (recipients || []).filter((e) => e.saved_id === null || e.saved_id === undefined).flatMap((entry) => {
    const members = entry.addresses || [];
    const list = members.length === 0 ? [entry] : members.map((m) => ({ ...m, name: m.name || entry.name }));
    return list.map((m) => ({
      name: m.name || m.address || 'Recipient', line: withFee(memberText(m), m), identity: recipientIdentity(m),
      // A fee payee starts as a fee. The user can change it in the editor.
      recipient: { name: m.name, recipient_type: m.recipient_type, address: m.address, split: '', custom_key: m.custom_key ?? null, custom_value: m.custom_value ?? null, fee: m.fee ? 'true' : null },
    }));
  });
  return { people: rows, payees };
}

// The rows that match the text of the filter field, by name and address. This
// is a display filter only.
export function filterPickRows(rows, query) {
  const q = String(query ?? '').trim().toLowerCase();
  if (q === '') return rows;
  return rows.filter((r) => `${r.name || ''} ${r.recipient.address || ''}`.toLowerCase().includes(q));
}

// ---- The pickers of the credits and the support links (ADR 0099 items 20 and 21) ----

const lowerName = (name) => String(name ?? '').trim().toLowerCase();
const firstKey = (person) => ((person.keys || []).find((k) => k.npub) || {}).npub || null;

// The new credit from a person: a start value. An empty value gives no
// attribute. The group is the usual group of the person (ADR 0112).
export function creditFromPerson(person) {
  const text = (value) => (value && String(value).trim() !== '' ? value : null);
  return { name: person.name, role: text(person.usual_role), group: text(person.usual_group), img: text(person.img), href: text(person.href), npub: firstKey(person) };
}

// The rows of the credits picker. A row is taken when the list has a credit
// with the same Nostr key or the same name.
export function creditPickRows(people, persons) {
  const keys = new Set((persons || []).map((p) => p.npub).filter(Boolean));
  const names = new Set((persons || []).map((p) => lowerName(p.name)).filter((n) => n !== ''));
  return (people || []).map((person) => {
    const credit = creditFromPerson(person);
    const parts = [credit.role, credit.npub ? shortKey(credit.npub) : null].filter(Boolean);
    const taken = (credit.npub !== null && keys.has(credit.npub)) || names.has(lowerName(person.name));
    return { id: person.id, name: person.name, line: parts.join(' · '), taken, credit, person };
  });
}

// The new support link from a link of a person: a start value.
export const fundingFromLink = (link) => ({ label: link.label ?? '', url: link.url ?? null });

// The rows of the support links picker: one row for each support link of
// each person. A row is taken when the list has the same address.
export function supportPickRows(people, fundings) {
  const urls = new Set((fundings || []).map((f) => String(f.url ?? '').trim()).filter((u) => u !== ''));
  return (people || []).flatMap((person) => (person.support_links || []).map((link) => ({
    id: `${person.id}-${link.id ?? link.url}`, name: link.label || link.url, line: `${person.name} · ${link.url}`,
    taken: urls.has(String(link.url ?? '').trim()), funding: fundingFromLink(link), person,
  })));
}

// The footer button of a picker. `one` and `many` are the nouns.
export const addPickLabel = (n, one, many) => `Add ${n} ${n === 1 ? one : many}`;

// The rows that match the filter text, by name and line. A display filter.
export function filterRows(rows, query) {
  const q = lowerName(query);
  return q === '' ? rows : rows.filter((r) => `${r.name || ''} ${r.line || ''}`.toLowerCase().includes(q));
}

// The identity of each address of each wallet that is in a person, from the
// list of `GET /recipients`. The value editor hides "Add to People" for them.
export function personKeys(recipients) {
  const keys = new Set();
  for (const entry of recipients || []) {
    if (entry.saved_id === null || entry.saved_id === undefined) continue;
    const members = (entry.addresses || []).length > 0 ? entry.addresses : [entry];
    for (const m of members) keys.add(recipientIdentity(m));
  }
  return keys;
}

// ---- Add to a person, more from your feeds and merge (ADR 0103 items 8 to 16, task 067e) ----

// The choices of a select menu of people, A to Z. `exceptId` leaves out one
// person, such as the person that keeps the merge.
export function personChoices(people, exceptId = null) {
  return (people || [])
    .filter((p) => p.id !== exceptId)
    .slice()
    .sort((a, b) => String(a.name).localeCompare(String(b.name), undefined, { sensitivity: 'base' }))
    .map((p) => [String(p.id), p.name]);
}

// The title of the sheet that adds a payee with no person to a person
// (ADR 0111 item 8).
export const addToTitle = (name) => `Add to a Person: ${name}`;
// The hint under a person of the sheet "Check: {name}" (ADR 0103 item 9).
export const ADD_TO_HINT = 'Studio adds the selected items to the person. Studio changes no feed.';
export const addItemsLabel = (n) => addPickLabel(n, 'Item', 'Items');

const trimmed = (text) => String(text ?? '').trim();
const sameCreditName = (c, item) => trimmed(c.name).toLowerCase() === trimmed(item.name).toLowerCase()
  && trimmed(c.href) === trimmed(item.href) && trimmed(c.npub).toLowerCase() === trimmed(item.npub).toLowerCase();
const keyParts = (k) => `${k.address || ''}\u0000${k.custom_key || ''}\u0000${k.custom_value || ''}`;

// The keys that a person has for payment: each wallet key and each member
// of each wallet. This is a display check only. The core adds each item once.
const walletKeys = (person) => new Set((person.wallets || []).flatMap((w) => [
  keyParts(w.key || {}),
  ...(w.members || []).map((m) => keyParts(m.key || m)),
]));

// Whether the person has an item of the group `credits`, `recipients` or
// `keys` already.
export function hasItem(person, group, item) {
  if (!person) return false;
  if (group === 'credits') return (person.credit_names || []).some((c) => sameCreditName(c, item));
  if (group === 'keys') return (person.keys || []).some((k) => k.npub === item);
  return walletKeys(person).has(keyParts(item.key || {}));
}

// The rows of the sheet "Add to a Person". Each row has its group and
// its index in the state, `already` with the name of the person, and
// `checked`. A row that the person has already has no check box.
export function addToRows(state, person, titles = null, profiles = null) {
  const status = (group, r, i, row) => {
    const already = hasItem(person, group, r.item);
    const flag = already ? `Already in ${person.name}` : 'New';
    // A new person has no flag: no item is in it yet.
    return { group, index: i, already, checked: !already && r.checked, ...row, lines: person ? [...row.lines, flag] : row.lines };
  };
  return [
    ['credits', 'Credits', (item) => creditItemRow(item, titles)],
    ['recipients', 'Payment addresses', payeeItemRow],
    ['keys', 'Nostr key', (npub) => keyItemRow(npub, profiles, state.name)],
  ].map(([group, title, rowOf]) => ({
    group, title, rows: (state[group] || []).map((r, i) => status(group, r, i, rowOf(r.item))),
  })).filter((g) => g.rows.length > 0);
}

// The number of items that the button adds: the checked items that the
// person does not have.
export const addSelectedCount = (state, person) => ['credits', 'recipients', 'keys']
  .reduce((n, group) => n + state[group].filter((r) => r.checked && !hasItem(person, group, r.item)).length, 0);

// The body of `POST /people/{id}/items` from the sheet "Add to a Person".
export function addItemsBody(state, person) {
  const chosen = (group) => state[group].filter((r) => r.checked && !hasItem(person, group, r.item));
  return {
    credits: chosen('credits').map((r) => [r.item.name, r.item.href, r.item.npub]),
    recipients: chosen('recipients').map((r) => [r.item.key, r.item.recipient_type]),
    keys: chosen('keys').map((r) => r.item),
  };
}

// ---- The sheet "Check: {name}" (ADR 0111 items 5 to 7, task 075c) ----

// The value of the menu entry "A new person".
export const NEW_PERSON = 'new';

export const checkSheetTitle = (name) => `Check: ${name}`;

// The hint of the menu "Person" when it shows the person with the name of
// the suggestion (ADR 0111 item 6).
export const MATCH_HINT = 'You have a person with this name. Studio adds the selected items to it.';

// The hint under the menu "Member of" of a new person.
export const NEW_MEMBER_HINT = 'The artist or label that this person is in. Studio changes no feed.';

// The hint under the items of a new person.
export const NEW_PERSON_HINT = 'Select the items that are this person. Studio changes no feed.';

// The start state of the sheet. The menu starts on `person_id` of the
// suggestion, else on "A new person". The core finds `person_id`.
export function checkSheetStart(suggestion, profiles = null) {
  const personId = suggestion.person_id === null || suggestion.person_id === undefined ? NEW_PERSON : String(suggestion.person_id);
  return { ...makePersonStart(suggestion, profiles), personId };
}

// The choices of the menu "Person": "A new person", then each person A to Z.
export const checkSheetChoices = (people) => [[NEW_PERSON, 'A new person'], ...personChoices(people)];

// The hint of the menu: the text for the person that matches the suggestion.
export const checkSheetHint = (suggestion, personId) => (
  suggestion.person_id !== null && suggestion.person_id !== undefined && String(suggestion.person_id) === personId ? MATCH_HINT : undefined);

// The person of the menu, or null for "A new person".
export const checkSheetPerson = (state, people) => (state.personId === NEW_PERSON ? null
  : (people || []).find((p) => String(p.id) === state.personId) || null);

// The label of the primary button: "Add 1 Item" or "Add N Items" for a person,
// else "Make the Person".
export const checkSheetLabel = (state, person) => (person ? addItemsLabel(addSelectedCount(state, person)) : 'Make the Person');

// True when the primary button works: a person with one new selected item,
// or a new person with a name.
export const checkSheetReady = (state, person) => (person ? addSelectedCount(state, person) > 0 : trimmed(state.name) !== '');

// ---- The sheet "Add to a Person: {name}" of a payee (ADR 0111 item 8) ----

export const WALLET_TO_PERSON_HINT = 'Studio keeps this wallet with the person. Studio changes no feed.';

// The start state: "A new person" with the name of the payee.
export const walletToPersonStart = (name) => ({ personId: NEW_PERSON, name: String(name ?? '') });

// True when the primary button works.
export const walletToPersonReady = (state) => state.personId !== NEW_PERSON || trimmed(state.name) !== '';

// The label of the primary button.
export const walletToPersonLabel = (state) => (state.personId === NEW_PERSON ? 'Make the Person' : 'Add the Wallet');

export const MORE_HINT = 'Studio found these items with the name, a wallet or a Nostr key of this person. Add the items that are this person.';

const CAUSES = {
  same_name: 'The name is the name of this person',
  same_credit_name: 'The name is a credit name of this person',
  same_npub: 'The Nostr key is a key of this person',
  same_wallet: 'The wallet is a wallet of this person',
};
export const causeText = (cause) => CAUSES[cause] || '';

// The rows of the section "More from your feeds" from `GET /people/{id}/more`.
// `key` is the stable key of the row, and `body` is the body of the Add. A
// credit with a Nostr key shows the profile of the key (ADR 0103 item 17),
// with the warning of item 18 for the name `name` of the person.
export function moreRows(items, titles = null, profiles = null, name = '') {
  return (items || []).map((item) => {
    const feeds = (item.feeds || []).map((id) => titleOf(titles, id)).join(', ');
    if (item.kind === 'credit') {
      const roles = (item.roles || []).length > 0 ? item.roles.join(', ') : 'Credit';
      const key = `c\u0000${item.name}\u0000${item.href}\u0000${item.npub}`;
      const nameLine = item.name ? [`The name in that credit is "${item.name}"`] : [];
      if (item.npub) {
        const row = keyRow(item.npub, profiles, name);
        const lines = [...row.lines, `Nostr key in the credit ${roles}`, ...nameLine, feeds || feedCount(0), causeText(item.cause)];
        return { ...row, key, lines, npub: item.npub, body: moreItemBody(item) };
      }
      const lines = [...nameLine, feeds || feedCount(0), causeText(item.cause)];
      return { key, title: roles, lines, npub: null, body: moreItemBody(item) };
    }
    const lines = [...(item.name ? [`The name in that feed is "${item.name}"`] : []), feeds || feedCount(0), causeText(item.cause)];
    return { key: `p${keyParts(item.key)}`, title: addressText(item.key, item.recipient_type), lines, npub: null, body: moreItemBody(item) };
  });
}

// The body of `POST /people/{id}/items` for one item of "More from your feeds".
export function moreItemBody(item) {
  if (item.kind === 'credit') return { credits: [[item.name, item.href, item.npub]], recipients: [], keys: [] };
  return { credits: [], recipients: [[item.key, item.recipient_type]], keys: [] };
}

// ---- The sheet "Merge with a Person" ----

export const mergeTitle = (name) => `Merge with ${name}`;
export const mergeHint = (name) => `Studio moves each item of the selected person to ${name}, then deletes that person. Studio changes no feed.`;
export const NO_OTHER_TEXT = 'No other person yet. Make a second person first.';

// The links of the selected person: wallets, Nostr keys, credit names and
// support links, one row each.
export function mergeLinks(other, profiles = null) {
  if (!other) return [];
  return [
    ...(other.wallets || []).map((w) => ({ key: `w${w.id}`, title: addressText(w.key, w.recipient_type), lines: [] })),
    ...(other.keys || []).map((k) => ({ key: `k${k.id}`, npub: k.npub, title: keyRow(k.npub, profiles).title, mono: keyRow(k.npub, profiles).mono, lines: ['Nostr key'] })),
    ...(other.credit_names || []).map((c) => ({ key: `c${c.id}`, title: c.name || 'Credit name', lines: ['Credit name'] })),
    ...(other.support_links || []).map((l) => ({ key: `s${l.id}`, title: l.label || l.url, lines: ['Support link'] })),
  ];
}

// The result line of the merge: the counts of the links that the person
// has after the merge. This is a display count. The core merges the links.
export function mergeResult(keep, other) {
  const union = (listOf, idOf) => new Set([...listOf(keep), ...listOf(other)].map(idOf)).size;
  const counts = [
    [union((p) => p.keys || [], (k) => k.npub), 'Nostr key', 'Nostr keys'],
    [union((p) => p.wallets || [], (w) => keyParts(w.key || {})), 'wallet', 'wallets'],
    [union((p) => p.credit_names || [], (c) => c.identity), 'credit name', 'credit names'],
    [union((p) => p.support_links || [], (l) => `${l.label}\u0000${l.url}`), 'support link', 'support links'],
  ].filter(([n]) => n > 0).map(([n, one, many]) => plural(n, one, many));
  return {
    title: keep.name,
    lines: [...(counts.length > 0 ? [counts.join(' · ')] : []), `Name, picture and default address stay as in ${keep.name}`],
  };
}

// The body of `POST /people/{id}/merge`.
export const mergeBody = (otherId) => ({ other: Number(otherId) });

// The model of the People area (ADR 0098, ADR 0099, ADR 0042). The functions
// have no Preact and no browser state. `peoplepages.js` shows the pages. The
// functions give texts and states only. The core applies each feed rule and
// gives each outcome (ADR 0065).

import { shortKey } from './nostrkey.js';
import { titleCase } from './casing.js';
import { feedKind, isPublisher, isReadOnly, roleWord } from './shell.js';
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
  return isAddress ? `LNURL address ${address}` : `Node address ${shortKey(address)}`;
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
  const links = (person.members || []).map((id) => titleOf(feedTitles, id));
  const membership = links.length > 0 ? [`Linked to ${links.join(', ')}`] : [];
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
    // Each line names the tracks, or the channel (ADR 0143).
    const where = (places) => ((places || []).length > 0 ? ` · ${placeText(places)}` : '');
    const lines = (place.pays || []).map((key, i) => `Pays ${addressText(key)}${where((place.pay_places || [])[i])}`);
    const roles = (place.roles || []).filter((role) => role !== '');
    if (roles.length > 0) lines.push(`Credited as ${roles.join(', ')}${where(place.role_places)}`);
    else if ((place.roles || []).length > 0) lines.push(`Credited${where(place.role_places)}`);
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
    case 'use_lnaddress': return 'Use the LNURL address';
    case 'use_address': return 'Use This Address';
    case 'rename_person': return 'Rename';
    case 'add_to_releases': return 'Add to All Releases';
    // The credit of a member on its artist or label feed (ADR 0118 item 12).
    case 'credit_feed': return 'Add to the Credits';
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

// Each publisher feed, A to Z, as [value, label] pairs, for the menu "Artist
// or label" of an operation. A feed with a role shows its first role, for
// example "HeyCitizen · Artist" (ADR 0111 item 3).
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
  if (isLnaddress(member)) return { label: 'LNURL address', address: member.address, account: null };
  if (member.provider) return { label: `${member.provider} node`, address: full ? member.address : null, account };
  // The account names the wallet on a shared node, so the line keeps it.
  return { label: 'Node address', address: full ? member.address : shortKey(member.address), account };
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

// The default address of a person shows on its own row (ADR 0139). `value` is
// the value of `keyValue`, or an empty text with no default.
export const isDefaultAddress = (key, value) => keyValue(key) === value;
// The wallets and the addresses with the default address first (ADR 0139 item 7).
export function defaultFirst(groups, value) {
  const rank = (m) => (isDefaultAddress(m.key, value) ? 0 : 1);
  return groups
    .map((g) => ({ ...g, members: [...g.members].sort((x, y) => rank(x) - rank(y)) }))
    .sort((x, y) => rank(x.members[0]) - rank(y.members[0]));
}
export const DEFAULT_HINT = '"Add a Recipient" uses the default address of this person.';

// The roles of a credit name, with the usual role of the person first and in
// brackets, when the credit name has that role (ADR 0140 item 7).
export function roleText(roles, usual) {
  const wanted = lowerName(usual);
  const found = wanted === '' ? undefined : roles.find((role) => lowerName(role) === wanted);
  if (found === undefined) return roles.join(', ');
  return [`[${found}]`, ...roles.filter((role) => role !== found)].join(', ');
}

// The text of an information address, with no scheme and no last slash.
const webText = (address) => String(address).replace(/^https?:\/\//, '').replace(/\/$/, '');

// The labeled row of a Nostr key. `value` is the profile name, or the short key
// with no profile. `sub` holds the other lines.
function nostrAttribute(row, notes = []) {
  const named = !row.mono;
  return { id: 'nostr', icon: 'zap', label: 'Nostr', value: named ? row.title : row.short, mono: !named,
    sub: [...(named ? [row.short] : []), ...row.lines.filter((line) => line !== row.short), ...notes] };
}

// The rows of the fold "Credited as" (ADR 0140). A credit name with a key of
// the person is one row with that key. `keyId` is the ID of that key, for the
// removal of the row. Each value is one labeled attribute.
export function creditNameRows(person, profiles = null) {
  const uses = new Map((person.credit_name_uses || []).map((u) => [u.id, u]));
  return (person.credit_names || []).map((c) => {
    const use = uses.get(c.id) || { feeds: 0, roles: [] };
    const key = c.npub ? keyRow(c.npub, profiles, person.name) : null;
    const found = (person.keys || []).find((k) => c.npub && k.npub === c.npub);
    const attributes = [
      { id: 'feeds', icon: 'disc', label: 'Used in', value: use.feeds > 0 ? feedCount(use.feeds) : 'No feed yet', sub: [] },
      ...(use.roles.length > 0 ? [{ id: 'roles', icon: 'tag', label: 'Roles', value: roleText(use.roles, person.usual_role), sub: [] }] : []),
      ...(c.href ? [{ id: 'website', icon: 'globe', label: 'Website', value: webText(c.href), sub: [] }] : []),
      ...(key ? [nostrAttribute(key)] : []),
    ];
    return {
      key: `c${c.id}`, title: c.name || (key ? key.title : 'Credit name'), attributes,
      picture: key ? key.picture : null, warning: key ? key.warning : null, npub: c.npub || null, creditId: c.id, keyId: found ? found.id : null,
    };
  });
}

// The row of a Nostr key that is not in a credit name.
export function keyOnlyRow(key, profiles, name) {
  const row = keyRow(key.npub, profiles, name);
  return { key: `k${key.id}`, title: row.mono ? 'Nostr key' : row.title, attributes: [nostrAttribute(row, [keyEvidenceText(key.evidence)])],
    picture: row.picture, warning: row.warning, npub: key.npub, keyId: key.id };
}

// The keys of a person that are not in a credit name.
export const keysWithNoCreditName = (person) => (person.keys || []).filter((k) => !(person.credit_names || []).some((c) => c.npub === k.npub));

// The summary line of the fold "Credited as" (ADR 0140 item 1).
export function creditFoldSummary(person) {
  const names = person.credit_names || [];
  const keys = person.keys || [];
  const first = names.find((c) => lowerName(c.name) === lowerName(person.name)) || names.find((c) => c.name);
  const parts = [];
  if (names.length > 0) parts.push(names.length === 1 ? '1 credit name' : `${names.length} credit names`);
  if (keys.length > 0) parts.push(keys.length === 1 ? '1 Nostr key' : `${keys.length} Nostr keys`);
  return { title: `Credited as ${first ? first.name : person.name}`, line: parts.join(' · ') };
}
export const CREDIT_FOLD_HINT = 'How Studio recognizes this person in your feeds. A credit with this key, or this name and website, counts as this person. This changes no feed. Brackets mark the usual role.';

// The text of the count of feeds of an address.
export const usedText = (count) => (count > 0 ? feedCount(count) : 'Not used');

// The text for the evidence of a Nostr key of a person.
export function keyEvidenceText(evidence) {
  switch (evidence) {
    case 'keysend': return 'Found in a lookup of an LNURL address';
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
    editTitle: 'Edit a Wallet',
    editButton: 'Save the Wallet',
    fields: [
      { id: 'recipient_type', label: 'Type', select: [['lnaddress', 'LNURL Address'], ['node', 'Node Address']], start: 'lnaddress' },
      { id: 'address', label: 'Address', required: true },
      { id: 'custom_key', label: 'Custom key', node: true, hint: 'Some nodes need a custom key and an account to find the wallet.' },
      { id: 'custom_value', label: 'Account', node: true },
    ],
  },
  'credit-names': {
    title: 'Add a Credit Name',
    button: 'Add the Credit Name',
    editTitle: 'Edit a Credit Name',
    editButton: 'Save the Credit Name',
    fields: [
      { id: 'name', label: 'Name', required: true },
      { id: 'href', label: 'Information address' },
    ],
    // The Nostr key is a NostrRow of editor.js, not a field (ADR 0123 item 5).
    nostrKey: true,
  },
  'support-links': {
    title: 'Add a Support Link',
    button: 'Add the Link',
    editTitle: 'Edit a Support Link',
    editButton: 'Save the Link',
    fields: [
      { id: 'label', label: 'Label' },
      { id: 'url', label: 'Address', required: true },
    ],
  },
};

// The fields of a link form that show, from the type of a wallet.
export const linkFields = (kind, values) => LINK_FORMS[kind].fields
  .filter((f) => !f.node || values.recipient_type === 'node');

export const linkStart = (kind) => ({
  ...Object.fromEntries(LINK_FORMS[kind].fields.map((f) => [f.id, f.start ?? ''])),
  ...(LINK_FORMS[kind].nostrKey ? { npub: null } : {}),
});

// The body of the POST of a link. An empty custom key or account is none.
// A form with `nostrKey` sends its key, or `null` with no key (ADR 0123).
export function linkBody(kind, values) {
  const body = {};
  for (const f of linkFields(kind, values)) {
    const text = String(values[f.id] ?? '').trim();
    body[f.id] = f.node && text === '' ? null : text;
  }
  if (LINK_FORMS[kind].nostrKey) body.npub = String(values.npub ?? '').trim() || null;
  return body;
}

// The values of the form "Add a Wallet" from a payee row (ADR 0141 item 2). The
// type follows the address when the payee gives none.
export const walletValues = (body) => ({
  recipient_type: body.recipient_type || (String(body.address ?? '').includes('@') ? 'lnaddress' : 'node'),
  address: body.address ?? '', custom_key: body.custom_key ?? '', custom_value: body.custom_value ?? '',
});

// True when the form "Add a Wallet" holds the values of this payee row, so the
// row stays selected. A change of a field ends the selection (ADR 0141 item 5).
export function walletSelected(values, body) {
  const given = walletValues(body);
  return ['address', 'custom_key', 'custom_value'].every((id) => String(values[id] ?? '').trim() === String(given[id]).trim());
}

// The start of the sheet "Edit a Credit Name" (ADR 0146 item 6).
export const creditNameEdit = (credit) => ({
  kind: 'credit-names', linkId: credit.id, note: 'This changes the credits that Studio counts as this person. It changes no feed.',
  start: { name: credit.name ?? '', href: credit.href ?? '', npub: credit.npub || null },
});

// The start of the sheet "Edit a Support Link" (ADR 0146 item 6).
export const supportLinkEdit = (link) => ({
  kind: 'support-links', linkId: link.id, note: 'This changes no feed.', start: { label: link.label ?? '', url: link.url ?? '' },
});

// The start of the sheet "Edit a Wallet" for one address of a wallet (ADR 0141
// item 6). `others` lists each other address that the wallet holds, because
// the change replaces the whole wallet.
export function walletEdit(group, member) {
  return {
    walletId: group.id,
    start: walletValues({ ...member.key, recipient_type: member.member.recipient_type }),
    others: group.members.filter((m) => m !== member).map((m) => m.key.address),
  };
}

// The warning of the sheet "Edit a Wallet" (ADR 0141 item 7).
export const walletEditNote = (others) => (others.length === 0 ? null
  : `This wallet also holds ${others.join(', ')}. A saved change replaces the wallet and removes ${others.length === 1 ? 'that address' : 'those addresses'} from this person. This changes no feed.`);

export const linkReady = (kind, values) => linkFields(kind, values).every((f) => !f.required || String(values[f.id] ?? '').trim() !== '');

// ---- The pending changes of the page of a person (ADR 0116 items 1 to 5) ----

// The values that the page keeps for a person. Each field, the default
// address and the membership are values of the page.
export function savedValues(person) {
  return {
    name: person.name ?? '',
    usual_role: person.usual_role ?? '',
    usual_group: person.usual_group ?? '',
    img: person.img ?? '',
    href: person.href ?? '',
    default_address: person.default_address ? keyValue(person.default_address) : '',
    members: (person.members || []).map(String),
    // The wallets and the Nostr keys that a fill adds (ADR 0117 item 14).
    wallets: [],
    keys: [],
  };
}

// The controls of the page, in their sequence. The usual role and its group
// are one control. The field controls are sent in the `fields` of a save.
const CONTROLS = ['name', 'usual_role', 'img', 'href', 'default_address', 'members', 'wallets', 'keys'];
const FIELD_CONTROLS = ['name', 'usual_role', 'img', 'href'];
// The list controls. Each added item is one pending change.
const LIST_NAMES = { wallets: 'wallet', keys: 'npub' };

const listLength = (list) => (list || []).length;

// The links that one list has and the other does not.
const changedLinks = (saved, values) => [
  ...values.members.filter((id) => !saved.members.includes(id)),
  ...saved.members.filter((id) => !values.members.includes(id)),
];

// Whether one control has a value that differs from the saved value. A list
// control compares its lengths.
export function isPending(saved, values, control) {
  if (control === 'usual_role') return values.usual_role !== saved.usual_role || values.usual_group !== saved.usual_group;
  if (control === 'members') return changedLinks(saved, values).length > 0;
  if (control in LIST_NAMES) return listLength(values[control]) !== listLength(saved[control]);
  return values[control] !== saved[control];
}

// The pending changes, one entry for each control. Each added wallet, each
// added Nostr key and each added or removed link is one entry of its own
// (ADR 0132 item 8).
export const pendingKeys = (saved, values) => CONTROLS.flatMap((control) => {
  if (control === 'members') return changedLinks(saved, values).map(() => 'member');
  if (control in LIST_NAMES) {
    const added = listLength(values[control]) - listLength(saved[control]);
    return Array.from({ length: Math.max(0, added) }, () => LIST_NAMES[control]);
  }
  return isPending(saved, values, control) ? [control] : [];
});

// The status of the toolbar of the page (ADR 0116 item 3).
export const pendingStatus = (count) => (count === 0 ? 'No changes' : plural(count, 'change not saved', 'changes not saved'));

// The body of `POST /people/{id}/save` (packet 079a). It holds only the parts
// with a pending change. It gives null when no control has a change.
export function saveBody(saved, values) {
  const keys = pendingKeys(saved, values);
  if (keys.length === 0) return null;
  const body = {};
  if (keys.some((key) => FIELD_CONTROLS.includes(key))) {
    body.fields = { name: values.name, usual_role: values.usual_role, usual_group: values.usual_group, img: values.img, href: values.href };
  }
  if (keys.includes('default_address')) body.default_address = values.default_address === '' ? null : keyFromValue(values.default_address);
  if (keys.includes('member')) body.members = values.members.map(Number);
  if (keys.includes('wallet')) {
    body.wallets = values.wallets.slice(listLength(saved.wallets)).map((w) => ({ key: w.key, recipient_type: w.recipient_type }));
  }
  if (keys.includes('npub')) body.keys = values.keys.slice(listLength(saved.keys));
  return body;
}

// The names of the controls in the text of the leave sheet.
const LEAVE_NAMES = {
  name: 'the name',
  usual_role: 'the usual role',
  img: 'the picture address',
  href: 'the information address',
  default_address: 'the default address',
  member: 'the artists and labels',
  wallet: 'a wallet',
  npub: 'a Nostr key',
};

// The text of the leave sheet. It names each control with a pending change
// (ADR 0116 item 8).
export function leaveText(name, keys) {
  const changes = plural(keys.length, 'change', 'changes');
  return `${name} has ${changes} that ${keys.length === 1 ? 'is' : 'are'} not saved: ${joinList([...new Set(keys.map((key) => LEAVE_NAMES[key]))])}.`;
}

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

// The hint of the links of a person (ADR 0132 item 2, ADR 0146 item 1).
export const LINK_HINT = 'Add the details of this person to your artist or label page. After Save, you can add a credit to each new feed. The link itself stays in Studio.';

// The row of the section with no link (ADR 0132 item 3).
export const LINK_EMPTY = 'This person is not linked to an artist or label page yet.';

// The choices of the menu "Link to an artist or label page" of a person and of
// a new person (ADR 0132 item 6). `linked` holds the feed IDs as text.
export const linkChoices = (feeds, linked = []) => memberChoices((feeds || []).filter((f) => (
  (f.publisher_roles || [])[0] !== 'hosting' && !linked.includes(String(f.id)))));

// The rows of the links of the page, A to Z (ADR 0132 items 3 and 4).
// `linkedCredits` is the list of the core: the links with a credit of the person.
export function linkRows(values, saved, feeds, linkedCredits = []) {
  const byId = new Map((feeds || []).map((f) => [String(f.id), f]));
  return values.members.map((id) => {
    const feed = byId.get(id);
    const role = feed && (feed.publisher_roles || []).length > 0 ? firstRole(feed) : '';
    const state = !saved.members.includes(id) ? 'Not saved yet'
      : linkedCredits.map(String).includes(id) ? 'In the credits of this feed' : 'Only in Studio';
    return {
      id,
      title: (feed && feed.title) || 'Untitled feed',
      dot: feed ? feedKind(feed).dot : 'feed',
      line: [role, state].filter((text) => text).join(' · '),
    };
  }).sort((x, y) => x.title.localeCompare(y.title, undefined, { sensitivity: 'base' }));
}

// The offers of a credit after a Save that changes the links (ADR 0118 items 3
// and 6, ADR 0132 item 11). `before` and `after` are the links before and
// after the Save, as text IDs. `feeds` is the feed list. `members` is a Map
// from each feed ID to its `GET /feeds/{id}/members` list. Each removal comes
// first, and each offer is `{ kind, feedId }`.
export function creditOffers(before, after, feeds, members, personId) {
  const byId = new Map((feeds || []).map((f) => [String(f.id), f]));
  // The row of the person in the members of a feed, or undefined.
  const rowOf = (feedId) => ((members && members.get(String(feedId))) || []).find((m) => m.person_id === personId);
  const removes = before.filter((feedId) => !after.includes(feedId) && (rowOf(feedId) || {}).in_credits)
    .map((feedId) => ({ kind: 'remove', feedId }));
  // A read-only feed gets no offer, because its change set cannot be written (ADR 0067).
  const adds = after.filter((feedId) => {
    const feed = before.includes(feedId) ? undefined : byId.get(feedId);
    return feed && !isReadOnly(feed) && !(rowOf(feedId) || {}).in_credits;
  }).map((feedId) => ({ kind: 'add', feedId }));
  return [...removes, ...adds];
}

// The position of a credit sheet, with two or more sheets (ADR 0132 item 11).
export const creditPosition = (index, total) => (total >= 2 ? `${index + 1} of ${total}.` : '');

// The view after the last credit sheet (ADR 0132 item 12). `made` holds the ID
// of each change set of the sheets. The People page holds "Changes to write".
export function afterCredits(made) {
  if (made.length === 0) return null;
  return made.length === 1 ? { name: 'change-set', id: made[0] } : { name: 'people' };
}

// The title of the sheet that adds a credit (ADR 0118 item 4).
export const creditAddTitle = (name, feed) => `Add ${name} to the Credits of ${feed}?`;

// The title of the sheet that removes a credit (ADR 0118 item 6).
export const creditRemoveTitle = (name, feed) => `Remove ${name} from the Credits of ${feed}?`;

// The note of the sheet that adds a credit (ADR 0118 item 4).
export const creditAddNote = (name, feed) => `${name} is part of ${feed} in Studio. The credits of the feed can also show it, so that apps and musicindex.org can find ${name}.`;

// The note about the public data of a credit (ADR 0118 item 4). It names the
// Nostr key only when the person has one.
export const creditPublicNote = (hasKey) => (hasKey
  ? 'The feed is public. The name, the picture, the information address and the Nostr key go in it.'
  : 'The feed is public. The name, the picture and the information address go in it.');

// The text of the sheet that removes a credit (ADR 0118 item 6).
export const creditRemoveNote = (name) => `The feed then does not name ${name}. Studio shows the review first.`;

// The body of the operation `credit_feed` of `POST /people/{id}/changes`.
export const creditOperation = (role, group) => ({ kind: 'credit_feed', role, group });

// The body of the operation that removes the credit of a person (ADR 0118 item 6).
export const REMOVE_CREDIT_OPERATION = { kind: 'remove', payee: false, credit: true };

// The word of a feed in the note of its members: the first role in lowercase,
// or "feed" with no role (ADR 0118 item 10).
export const feedWord = (feed) => ((feed.publisher_roles || []).length > 0
  ? feedKind(feed).label.split(' · ')[0].toLowerCase()
  : 'feed');

// The note of the section "People" of a publisher feed (ADR 0132 item 10).
export const memberSectionNote = (word) => `The people that are part of this ${word}. A person that is not in the credits is only in Studio.`;

// The second line of a member row: its usual role, then whether the channel
// of the feed has a credit of the person (ADR 0118 item 9).
export function memberRowLine(member) {
  const state = member.in_credits ? 'In the credits of this feed' : 'Only in Studio';
  const role = (member.usual_role || '').trim();
  return role ? `${role} · ${state}` : state;
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

// ---- The entries of the check (ADR 0120, task 084b) ----

export const CHECK_TITLE = 'Check your feeds';

const capital = (text) => text.charAt(0).toUpperCase() + text.slice(1);

// The word for the type of a feed in the check: "album", "playlist", "artist
// page" or "label page" (ADR 0120 item 4). A feed with no known type is an
// album.
export function releaseWord(feed) {
  if (feedKind(feed || {}).label === 'Playlist') return 'playlist';
  if (!isPublisher(feed || {})) return 'album';
  return (feed.publisher_roles || []).includes('label') ? 'label page' : 'artist page';
}

// The label of the button that opens the feed in the editor (ADR 0120 item 3).
export const openLabel = (feed) => `Open the ${titleCase(releaseWord(feed))}`;

// The label of the button that corrects one feed (ADR 0120 item 3).
export const useValuesLabel = (name) => `Use ${name}'s Values`;

// The type and the count of an entry of the check, for example "Album · 2
// credits differ" (ADR 0120 item 1). The count uses the findings of the core.
export function checkSummary(entry, feed) {
  const findings = (entry && entry.findings) || [];
  const credits = findings.filter((f) => f.kind.startsWith('credit_')).length;
  const payments = findings.filter((f) => f.kind === 'not_default' || f.kind === 'other_wallet').length;
  const counted = [credits > 0 && plural(credits, 'credit', 'credits'), payments > 0 && plural(payments, 'payment', 'payments')].filter(Boolean);
  // One difference takes the singular verb, and more than one takes the plural.
  const verb = counted.length === 1 && credits + payments === 1 ? 'differs' : 'differ';
  const differ = counted.length > 0 ? `${joinList(counted)} ${verb}` : '';
  return [capital(releaseWord(feed)), differ].filter(Boolean).join(' · ');
}

// The position of a credit: "Channel", or the title of each item. More than three
// places give the first two and "and N more" (ADR 0120 item 2).
export function placeText(places) {
  const names = (places || []).map((p) => (p.kind === 'channel' ? 'Channel' : (p.title || '').trim() || 'Untitled item'));
  if (names.length <= 3) return names.join(', ');
  return `${names[0]}, ${names[1]} and ${names.length - 2} more`;
}

// The text of a value, or "None" for a missing value.
const orNone = (text) => (String(text ?? '').trim() === '' ? 'None' : String(text));

// The name of a Nostr key: the name of its profile, or the short key.
const keyName = (npub, profiles) => profileName(npub, profiles) || shortKey(npub);

// The blocks of the differences of one entry of the check (ADR 0120 item 2).
// A credit with a picture and an information address that both differ gives
// one block for each. `person` gives the keys and the usual role of the person.
// The core gives each value. The function only chooses the text.
export function differenceBlocks(entry, name, feed, profiles = null, person = {}) {
  const feedLine = `In the ${releaseWord(feed)}`;
  const blocks = [];
  for (const f of (entry && entry.findings) || []) {
    const role = String(f.role || '').trim();
    const credit = ['Credit', role && capital(role)];
    switch (f.kind) {
      case 'credit_key': {
        const personKeys = (person.keys || []).filter((k) => k.npub).map((k) => keyName(k.npub, profiles)).join(', ');
        blocks.push({
          place: placeText(f.places),
          what: [...credit, 'Nostr key'].filter(Boolean).join(' · '),
          lines: [{ label: feedLine, value: f.npub ? keyName(f.npub, profiles) : 'None', old: true },
            { label: name, value: orNone(personKeys), old: false }],
        });
        break;
      }
      case 'credit_details': {
        const parts = [
          { changed: f.img, part: 'picture', feedValue: f.feed_img, personValue: f.person_img },
          { changed: f.href, part: 'information address', feedValue: f.feed_href, personValue: f.person_href },
        ];
        for (const p of parts.filter((x) => x.changed)) {
          blocks.push({
            place: placeText(f.places),
            what: [...credit, p.part].filter(Boolean).join(' · '),
            lines: [{ label: feedLine, value: orNone(p.feedValue), old: true }, { label: name, value: orNone(p.personValue), old: false }],
          });
        }
        break;
      }
      case 'credit_role':
        blocks.push({
          place: placeText(f.places),
          what: 'Credit · No role',
          lines: [{ label: feedLine, value: 'None', old: true }, { label: name, value: orNone(person.usual_role), old: false }],
        });
        break;
      case 'not_default':
      case 'other_wallet': {
        const what = f.kind === 'other_wallet' ? `Payment · not a wallet of ${name}` : 'Payment · not the default address';
        blocks.push({
          place: 'Payments',
          what,
          lines: [{ label: feedLine, value: addressText(f.key), old: true },
            { label: name, value: person.default_address ? addressText(person.default_address) : 'None', old: false }],
        });
        break;
      }
      default: break;
    }
  }
  return blocks;
}

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
    case 'credit_feed': return `Add ${who} as a credit in ${feedCount(n)}`;
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

// The key of the change "Nostr key" when the profiles are known: the first
// key with no warning of ADR 0103 item 18. When each key has the warning, the
// change starts not selected, so that Studio writes no key of a different
// account by default. `fields` is the state before the profiles came.
export function correctKeyStart(person, fields, profiles) {
  const keys = person.keys || [];
  const good = keys.find((k) => !keyWarning(k.npub, profiles, person.name));
  if (good) return { ...fields, npub: good.npub };
  return { ...fields, key: false };
}

// The check boxes of the sheet: `id`, `label` and `line`.
// The line of the change "other_payees". `count` is the number of the
// findings `other_wallet` of the check, and the address is the default
// address of the person. The texts are display values only.
export function otherPayeesLine(name, defaultAddress, count) {
  const to = defaultAddress ? addressText(defaultAddress).replace(/^(LNURL address|Node address) /, '') : 'the default address';
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
  const profile = profileName(fields.npub, profiles);
  const who = profile ? `the Nostr key of ${profile}` : `the Nostr key ${shortKey(fields.npub || '')}`;
  const role = trimmed(person.usual_role);
  const all = [
    ['key', 'Nostr key', [`Each credit of ${person.name} gets ${who}`, keyWarning(fields.npub, profiles, person.name)].filter(Boolean).join('. ')],
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

// The people list of the sheet "Find a Nostr profile" (ADR 0137 items 1 and 2):
// one row for each Nostr key of each person. The key of the field has no row.
export function nostrPickRows(people, current) {
  return (people || []).flatMap((person) => (person.keys || []).filter((k) => k.npub && k.npub !== current).map((k) => ({
    id: `${person.id}-${k.npub}`, name: person.name, line: k.npub, npub: k.npub, person,
  })));
}

const isEmpty = (value) => value === null || value === undefined || String(value).trim() === '';

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

// A list of a sheet with this number of rows or more starts closed (ADR 0145
// item 2).
const FOLD_AT = 6;
export const foldStartsOpen = (count) => count < FOLD_AT;

const present = (entries) => Object.fromEntries(entries.filter(([, value]) => !isEmpty(value)));

// The choices of a feed of musicindex for each sheet (ADR 0145 items 3 and 5).
// The values come from `GET /musicindex/values/{guid}`, and `hit` is the row of
// a feed that is not an artist or a label. The title of such a feed is not the
// name of a person, and its website and key are not of the artist. Each value
// is a start value.
export function creditChoices(values, hit = null) {
  const start = present(hit
    ? [['name', hit.artist], ['img', values.image_url]]
    : [['name', values.title], ['img', values.image_url], ['href', values.website], ['npub', values.npub]]);
  // The credits of the feed and its tracks (ADR 0146 item 8). A credit with the
  // name of the feed joins its row, so the row keeps the role of that credit.
  const creditValues = (c) => present([['name', c.name], ['role', c.role], ['group', c.group], ['img', c.img], ['href', c.href], ['npub', c.npub]]);
  const credits = (values.credits || []).filter((c) => !isEmpty(c.name));
  const same = credits.find((c) => c.name === start.name);
  const whole = Object.keys(start).length === 0 ? [] : [{
    id: 'feed', title: start.name || 'The feed',
    line: (same && same.role) || (hit ? 'The artist of this feed' : 'This artist or label feed'),
    values: same ? { ...start, ...creditValues(same) } : start,
  }];
  return [...whole, ...credits.filter((c) => c !== same).map((c) => ({
    id: `${c.name} ${c.role ?? ''}`, title: c.name, line: c.role || 'No role', values: creditValues(c),
  }))];
}

// A result of "Find a Nostr profile" with a missing name or picture takes it from
// the person that holds the key (ADR 0146 item 5).
export function withPersonValues(profile, people) {
  const person = (people || []).find((p) => (p.keys || []).some((k) => k.npub === profile.npub));
  if (!person) return profile;
  return { ...profile, name: profile.name || person.name || null, picture: profile.picture || person.img || null };
}

// The Nostr key of a feed. It comes with the name and the picture that the sheet
// "Find a Nostr profile" can use (ADR 0145 item 15).
export const nostrChoices = (values) => (isEmpty(values.npub) ? [] : [{ id: 'key', values: { npub: values.npub, name: values.title ?? null, picture: values.image_url ?? null } }]);

export const fundingChoices = (values) => (isEmpty(values.website) ? [] : [{ id: 'website', values: { url: values.website } }]);

export const recipientChoices = (values) => (values.routes || []).filter((r) => !isEmpty(r.address)).map((r) => {
  return { id: `${r.address} ${r.custom_key ?? ''} ${r.custom_value ?? ''}`, title: r.name || r.address, line: memberText(r), values: r };
});

// The footer button of the sheet "Copy from a Feed" (ADR 0145 item 13).
export const addRecipientsLabel = (n) => (n === 0 ? 'Add Recipients' : addPickLabel(n, 'Recipient', 'Recipients'));

// The heading of the recipients of a track. A single has one track with the
// title of its feed, and the back row already names that title.
export const trackHeading = (track, feed) => (track === feed ? 'Recipients of this track' : `Recipients of ${track}`);


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

// The hint under the menu of the link of a new person (ADR 0132 item 10).
export const NEW_MEMBER_HINT = 'The artist or label page of this person. Studio changes no feed.';

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

// The label of the button that adds a row of "More from your feeds". A row with
// a key warning also has "Correct the Feeds" (ADR 0138 item 1).
export const moreAddLabel = (row) => (row.warning ? 'Add Anyway' : 'Add');

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

// ---- Find on musicindex (ADR 0117, task 080b) ----

// The text of the search sheet (ADR 0117 item 2).
export const FIND_HINT = 'Studio searches the artist and label feeds on musicindex.org. Select the feed of this person.';
export const FIND_ONE_HINT = 'Studio searches the feeds on musicindex.org.';
export const FIND_ONE_EMPTY = 'No feed on musicindex has this name.';
export const FIND_EMPTY = 'No artist or label feed on musicindex has this name.';
export const FIND_WAIT = 'Searching…';
export const FILL_NOTE = 'Studio puts the values in the fields. Select Save to keep them. Studio changes no feed.';

// The levels of the search sheet, in sequence (ADR 0117 items 4 and 6).
export const LOOKUP_LEVELS = [
  { level: 'role', heading: 'Verified role', note: 'The feed and its albums link to each other, and both give the role.' },
  { level: 'two_way', heading: 'Verified two-way links', note: 'The feed and its albums link to each other, but they give no role.' },
  { level: 'name_only', heading: 'Name only', note: 'Only the name matches. No album links back to this feed.' },
];

// The rows of a search in their levels. A level with no row is left out. In
// a level, the rows keep the sequence of the search (ADR 0117 item 4).
export const lookupGroups = (rows) => LOOKUP_LEVELS
  .map((group) => ({ ...group, rows: (rows || []).filter((row) => row.level === group.level) }))
  .filter((group) => group.rows.length > 0);

// The dot class and the type line of a row (ADR 0117 item 4). The dot of a
// role is filled. The dots of the other kinds are hollow.
export function lookupKind(row) {
  const kind = row.kind || {};
  switch (kind.type) {
    case 'role': {
      const roles = kind.roles || [];
      const [one, many] = roles[0] === 'label' ? ['release', 'releases'] : ['album', 'albums'];
      const count = row.albums == null ? '' : ` · ${plural(row.albums, one, many)}`;
      return { dot: roles[0] || 'feed', text: `${roles.map(roleWord).join(' · ')}${count}` };
    }
    case 'inferred_artist': {
      const count = row.albums == null ? '' : ` · ${plural(row.albums, 'album', 'albums')}`;
      return { dot: 'artist hollow', text: `Inferred artist${count}` };
    }
    case 'no_role': return { dot: 'feed hollow', text: 'Feed · two-way links, no role' };
    default: return { dot: 'feed hollow', text: 'Unverified link' };
  }
}

const VALUE_WORDS = [['picture', 'a picture'], ['website', 'a website'], ['wallet', 'a wallet'], ['npub', 'a Nostr key']];

// The host of a row, then the values that it has (ADR 0117 item 7). The
// result is `null` for a row with no detail.
export function hasLine(row) {
  const has = row.has || {};
  const words = VALUE_WORDS.filter(([key]) => has[key]).map(([, word]) => word);
  const parts = [row.host, words.length > 0 ? joinList(words) : null].filter(Boolean);
  return parts.length > 0 ? parts.join(' · ') : null;
}

// The start value, the state and the words of each value of a feed (ADR 0117
// items 9 to 12).
export const ALREADY_VALUE = 'Already the value of this person';
const ALREADY_WALLET = 'Already a wallet of this person';
const NO_KEY = 'This feed gives no Nostr key. Use “Add a Nostr Key” to search the Nostr profiles.';
const NO_ADDRESS = 'This wallet has no address, so Studio cannot save it.';
const PAYS_CHECK = 'A feed can pay a band member, a collaborator or an app. Add this wallet only when this recipient is this person.';
const NO_RECIPIENT_NAME = 'The feed gives no name for this recipient.';
const WALLET_WORDS = { lnaddress: 'LNURL address', node: 'Node address' };
const KEYSEND_WORDS = { found: 'Checked with keysend', no_data: 'No keysend data', failed: 'The keysend check failed' };

// A value of another person has the name of that person (ADR 0098 item 6).
const otherName = (state) => (state && typeof state === 'object' && state.other_person ? state.other_person.name : null);
const isWebAddress = (value) => typeof value === 'string' && /^https?:\/\//i.test(value);

// The rows of the values sheet (ADR 0117 items 9 to 12). `pending` gives the
// values of the page: a picture or an information address starts selected
// when its field is empty, and a wallet or a Nostr key starts selected when a
// fill has added it already.
export function fillRows(values, pending) {
  const rows = [];
  const picture = values.picture;
  if (picture && isWebAddress(picture.value)) {
    const current = picture.state === 'current';
    rows.push({
      id: 'img', title: 'Picture', value: picture.value, lines: current ? [ALREADY_VALUE] : [],
      selectable: !current, selected: !current && pending.img === '', art: picture.value,
    });
  }
  const website = values.website;
  if (website && isWebAddress(website.value)) {
    const current = website.state === 'current';
    rows.push({
      id: 'href', title: 'Information address', value: website.value, lines: [current ? ALREADY_VALUE : 'The link of the feed'],
      selectable: !current, selected: !current && pending.href === '',
    });
  }
  (values.wallets || []).forEach((wallet, index) => {
    const state = wallet.state;
    const owned = state === 'current' || state === 'has_wallet';
    const name = otherName(state);
    const selectable = !!wallet.key && !owned && name === null;
    let status;
    if (owned) status = ALREADY_WALLET;
    else if (name !== null) status = `A wallet of ${name}`;
    else if (!wallet.key) status = NO_ADDRESS;
    const split = /^\d+$/.test(wallet.split || '') ? ` · ${wallet.split}%` : '';
    const lines = [wallet.name ? `Paid to: ${wallet.name}` : NO_RECIPIENT_NAME, `The value block of the feed${split}`];
    if (wallet.lookup) lines.push(KEYSEND_WORDS[wallet.lookup]);
    lines.push(selectable ? PAYS_CHECK : status);
    rows.push({
      id: `wallet:${index}`, title: 'Wallet', value: wallet.value, lines: [WALLET_WORDS[wallet.recipient_type] || 'Wallet', ...lines],
      selectable, selected: selectable && pending.wallets.some((w) => keyValue(w.key) === keyValue(wallet.key)),
      wallet: { key: wallet.key, recipient_type: wallet.recipient_type },
    });
  });
  const npub = values.npub;
  if (!npub) {
    rows.push({ id: 'npub', title: 'Nostr key', value: null, lines: [NO_KEY], selectable: false, selected: false });
  } else {
    const current = npub.state === 'current';
    const name = otherName(npub.state);
    let lines = [];
    if (current) lines = [ALREADY_VALUE];
    else if (name !== null) lines = [`A Nostr key of ${name}`];
    rows.push({
      id: 'npub', title: 'Nostr key', value: npub.value, mono: true, npub: npub.value, lines,
      // A key of a different person has no check box, as a wallet of a
      // different person (ADR 0117 item 12). The core refuses it at the save.
      selectable: !current && name === null, selected: !current && name === null && pending.keys.includes(npub.value),
    });
  }
  return rows;
}

// The text of the Fill button, for the number of selected rows (ADR 0117 item 13).
export const fillLabel = (count) => `Fill ${plural(count, 'Field', 'Fields')}`;

// The rows that a fill adds to the pending values (ADR 0117 item 14). `checked`
// gives the IDs of the checked rows. A picture or an information address
// replaces its field. A wallet or a Nostr key is added to its list, once.
export function fillPatch(rows, checked, pending) {
  const on = rows.filter((row) => row.selectable && checked.includes(row.id));
  const patch = {};
  const picture = on.find((row) => row.id === 'img');
  if (picture) patch.img = picture.value;
  const website = on.find((row) => row.id === 'href');
  if (website) patch.href = website.value;
  const wallets = on.filter((row) => row.wallet)
    .map((row) => row.wallet)
    .filter((wallet) => !pending.wallets.some((w) => keyValue(w.key) === keyValue(wallet.key)));
  if (wallets.length > 0) patch.wallets = [...pending.wallets, ...wallets];
  const npubs = on.filter((row) => row.npub).map((row) => row.npub).filter((npub) => !pending.keys.includes(npub));
  if (npubs.length > 0) patch.keys = [...pending.keys, ...npubs];
  return patch;
}

// The line of a filled field (ADR 0117 item 14). A row with no kind word
// gives the line with the word "feed" only.
export function fillSource(row) {
  const kind = row.kind || {};
  const word = kind.type === 'role' ? ((kind.roles || [])[0] || '').toLowerCase() : (kind.type === 'inferred_artist' ? 'artist' : '');
  const title = row.title || 'Untitled feed';
  return word ? `From musicindex: the ${word} feed ${title}` : `From musicindex: the feed ${title}`;
}

// The link of a feed on musicindex.org (ADR 0117 item 8).
export const musicindexFeedUrl = (guid) => `https://musicindex.org/#feed=${encodeURIComponent(guid)}`;

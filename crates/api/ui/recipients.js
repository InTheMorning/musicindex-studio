// The model of the Recipients pages and of the review of a change set
// (ADR 0091, ADR 0042). The functions have no Preact and no browser state.
// `app.js` shows the pages. They give texts and states only. The core
// calculates each split and finds each place (ADR 0065).

import { shortKey } from './nostrkey.js';
import { memberLine, memberText, withFee, reviewLabel, checkedIds } from './people.js';

const plural = (n, one, many) => `${n} ${n === 1 ? one : many}`;

// The key that the routes use to name a recipient (ADR 0027).
export function recipientKey(recipient) {
  return {
    address: recipient.address,
    custom_key: recipient.custom_key ?? null,
    custom_value: recipient.custom_value ?? null,
  };
}

// The name of a row: the name, or else the address of the first member of the
// wallet, or else the address (ADR 0095).
// The parts of a title for its line breaks: the text up to and with each
// "@", then the rest. The page puts a break point between the parts, so a
// heading wraps an address after its "@", not in its domain.
export function breakParts(text) {
  return String(text || '').split(/(?<=@)/);
}

export function recipientTitle(recipient) {
  if (recipient.name && recipient.name.trim() !== '') return recipient.name;
  const first = (recipient.addresses || [])[0];
  return first ? first.address : recipient.address;
}

// The members of a wallet, the Lightning addresses first (ADR 0095 item 7).
// An entry with no `addresses` is its own single member.
const membersOf = (entry) => ((entry.addresses || []).length > 0 ? entry.addresses : [entry])
  .slice().sort((a, b) => (b.recipient_type === 'lnaddress') - (a.recipient_type === 'lnaddress'));

// One line for each member of a wallet, for the header of its page. The
// lines come from `memberLine` of `people.js`, with a node key in full
// (ADR 0095 items 7 and 8, ADR 0099 item 15).
export function addressLines(entry) {
  return membersOf(entry).map((m) => memberLine(m, true));
}

// The texts of the lines of a row: the Lightning addresses first, then each
// node (ADR 0095 item 7). A node key shows in short form.
export function rowLines(entry) {
  return membersOf(entry).map((m) => withFee(memberText(m), m));
}

const memberOfType = (entry, type) => (entry.addresses || []).find((m) => m.recipient_type === type) || null;

// The first Lightning address of a wallet, or null.
export function lnaddressOf(entry) {
  const found = memberOfType(entry, 'lnaddress');
  return found ? found.address : null;
}

// The first node key of a wallet, or null.
export function nodeOf(entry) {
  const found = memberOfType(entry, 'node');
  return found ? found.address : null;
}

export function hasNode(entry) {
  return nodeOf(entry) !== null;
}

// The start values of the address fields of an operation form for one type:
// the first member of the wallet with that type, or empty texts (ADR 0095
// item 14). The key of the wallet is not a start value: for a Lightning
// address with a found lookup, it is a node key that no feed has. The user
// can change the values (an edit aid only).
export function memberStart(entry, type) {
  const found = memberOfType(entry, type);
  return {
    address: found ? found.address || '' : '',
    custom_key: found ? found.custom_key || '' : '',
    custom_value: found ? found.custom_value || '' : '',
  };
}

// The start type of an operation form. Change address starts at the node
// key when the wallet has one. Add to an artist or label starts at the
// Lightning address when the wallet has one (ADR 0095 items 10 and 14,
// ADR 0096).
export function startType(entry, kind) {
  const [first, second] = kind === 'add' ? ['lnaddress', 'node'] : ['node', 'lnaddress'];
  return memberOfType(entry, first) ? first : second;
}

// The value of the option "Other address" of the suggestion menu.
export const OTHER_ADDRESS = '';

const SOURCE_TEXTS = { collection: 'in your feeds', nostr: 'in a Nostr profile', provider: 'at the node provider' };

// The hint of a suggestion: where Studio found it.
function sourceHint(sources) {
  const from = (sources || []).map((k) => SOURCE_TEXTS[k]).filter(Boolean);
  if (from.length === 0) return 'This address pays the same wallet.';
  const text = from.length === 1 ? from[0] : `${from.slice(0, -1).join(', ')} and ${from[from.length - 1]}`;
  return `Studio found this address ${text}. It pays the same wallet.`;
}

// The Lightning address field of the sheet "Use the Lightning address"
// (ADR 0097 item 5). With suggestions, a menu starts at the first one and
// ends with "Other address". The menu shows only the address, and `hints`
// gives the source of each one. With none, the sheet shows the text field.
// The core examines the address of the operation in each condition.
export function suggestionField(suggestions) {
  const list = suggestions || [];
  if (list.length === 0) {
    return { menu: false, options: [], hints: {}, start: '', hint: 'Studio found no Lightning address for this wallet.' };
  }
  return {
    menu: true,
    options: [...list.map((s) => [s.address, s.address]), [OTHER_ADDRESS, 'Other address']],
    hints: Object.fromEntries(list.map((s) => [s.address, sourceHint(s.sources)])),
    start: list[0].address,
    hint: null,
  };
}

// The hint of a Lightning address that has no keysend data.
export function lookupHint(address) {
  const domain = (address || '').split('@').slice(1).join('@');
  return `Studio found no payment details for this address at ${domain}. Some apps cannot pay it.`;
}

// The row of a recipient: the name, the count of feeds and one line for each
// address of the wallet (ADR 0095 item 7 and ADR 0091 item 2). The count is
// "Saved, not used" for a saved recipient that no feed uses. A recipient that
// is not saved and that no feed model uses now shows "0 feeds".
export function recipientRow(recipient) {
  const unusedSaved = !(recipient.feeds > 0) && recipient.saved_id != null;
  return {
    title: recipientTitle(recipient),
    count: unusedSaved ? 'Saved, not used' : plural(recipient.feeds || 0, 'feed', 'feeds'),
    lines: rowLines(recipient),
  };
}

// The places of a feed in one text, for example "Channel · 12 tracks"
// (ADR 0091 item 3).
export function placesText(use) {
  const parts = [];
  if (use.channel) parts.push('Channel');
  if (use.items > 0) parts.push(plural(use.items, 'track', 'tracks'));
  if (use.live_items > 0) parts.push(plural(use.live_items, 'live item', 'live items'));
  return parts.join(' · ');
}

// The text under the title of a feed on the page of a recipient. A feed that
// Studio cannot write shows it in the same line.
export function useRow(use) {
  const places = placesText(use);
  return {
    title: use.title && use.title.trim() !== '' ? use.title : 'Untitled feed',
    text: use.writable ? places : `${places} · Read-only`,
  };
}

// The text of each cause that a feed gives when the core does not change it.
const CAUSES = {
  no_recipient: 'The feed has no value block with this recipient.',
  has_recipient: 'The value block has this recipient already.',
  has_new_address: 'The value block has the new address already.',
  no_value_block: 'The feed has no value block.',
  only_fees: 'Each split of the value block is a fee.',
  split_not_number: 'A split of the value block is not a number.',
  bad_percent: 'The percentage must be a number from 1 to 99.',
  no_node_recipient: 'Each value block has the Lightning address already.',
  in_open_change_set: 'The feed has other changes that Studio did not write.',
};

export function causeText(cause) {
  const value = cause && cause.cause;
  if (value === 'cannot_start') {
    return cause.reason ? `Studio cannot start the change: ${cause.reason}` : 'Studio cannot start the change.';
  }
  return CAUSES[value] || 'Studio did not change this feed.';
}

// The outcome of a feed in the review: a text and a flag.
export function outcomeText(outcome) {
  if (outcome && outcome.outcome === 'changed') {
    return { changed: true, text: `Changed · ${placesText(outcome)}` };
  }
  return { changed: false, text: `Not changed · ${causeText(outcome && outcome.cause)}` };
}

const BLOCKED = {
  read_only_no_host: 'Studio cannot write this feed. Connect a file host for it first.',
  vault_locked: 'Studio cannot write this feed while the vault is locked. Unlock Studio first.',
};

// The cause that a feed cannot be written now, or null.
export function blockedText(blocked) {
  return blocked ? (BLOCKED[blocked] || 'Studio cannot write this feed now.') : null;
}

// The line under the title of the review.
export function operationLine(operation, recipientName) {
  const who = recipientName || 'the recipient';
  switch (operation && operation.kind) {
    case 'change_address': return `Change the address of ${who} to ${operation.address}.`;
    case 'rename': return `Rename ${who} to ${operation.name}.`;
    case 'remove': return `Remove ${who}.`;
    case 'add': return `Add ${who} with ${operation.percent}%.`;
    case 'use_lnaddress': return `Studio replaces the node key of ${who} with ${operation.address} in the selected feeds.`;
    default: return '';
  }
}

// The name of the recipient of a key in a list of recipients, or the
// address when the list does not have the key.
export function nameOf(list, key) {
  const same = (a, b) => (a || '') === (b || '');
  const found = (list || []).find((r) => r.address === key.address
    && same(r.custom_key, key.custom_key) && same(r.custom_value, key.custom_value));
  return found ? recipientTitle(found) : key.address;
}

// ---- The write of a change set (ADR 0091 items 23 and 24) ----
// `statuses` maps a feed ID to `{ state, error }`. The states are `waiting`,
// `writing`, `written` and `failed`.

// Does the feed need a write? It is changed, not blocked, and not written.
export function isWritable(feed) {
  return !!feed.outcome && feed.outcome.outcome === 'changed' && !feed.blocked && feed.written_at == null;
}

export function writableIds(changeSet) {
  return (changeSet.feeds || []).filter(isWritable).map((feed) => feed.feed_id);
}

// The status of each feed at the start. A feed that is written is
// `written`. A feed that needs a write is `waiting`.
export function initialStatuses(changeSet) {
  const statuses = {};
  for (const feed of changeSet.feeds || []) {
    if (feed.written_at != null) statuses[feed.feed_id] = { state: 'written' };
    else if (isWritable(feed)) statuses[feed.feed_id] = { state: 'waiting' };
  }
  return statuses;
}

// The status of the other feeds does not change.
export function startWrite(statuses, id) {
  return { ...statuses, [id]: { state: 'writing' } };
}

// A failure sets the status of that feed only. The next feed stays `waiting`
// and the write goes on with it.
export function finishWrite(statuses, id, error) {
  return { ...statuses, [id]: error ? { state: 'failed', error } : { state: 'written' } };
}

export function statusText(status) {
  if (!status) return '';
  switch (status.state) {
    case 'waiting': return 'Waiting';
    case 'writing': return 'Writing';
    case 'written': return 'Written';
    default: return 'Not written';
  }
}

// The error text of a feed that Studio did not write, or null. The review
// shows it below the outcome, at the full width of the row.
export function statusError(status) {
  return status && status.state === 'failed' && status.error ? status.error : null;
}

// The next feed to write: the first one that is `waiting`, or null.
export function nextToWrite(statuses, order) {
  const id = (order || []).find((candidate) => statuses[candidate] && statuses[candidate].state === 'waiting');
  return id === undefined ? null : id;
}

// The feeds that still need a write after a pass: the failed ones.
export function retryStatuses(statuses) {
  const next = {};
  for (const [id, status] of Object.entries(statuses)) {
    next[id] = status.state === 'failed' ? { state: 'waiting' } : status;
  }
  return next;
}

export function writeLabel(count) {
  return `Write ${plural(count, 'Feed', 'Feeds')}`;
}

export function hasWrittenFeed(changeSet, statuses) {
  return (changeSet.feeds || []).some((feed) => feed.written_at != null || (statuses[feed.feed_id] || {}).state === 'written');
}

// ---- The sheets of a wallet with no person (ADR 0099 items 7 and 10, task 065d) ----

// The operations of the page of a wallet, in sequence, with the role of each
// button (ADR 0037). A wallet with a node key can use a Lightning address.
const WALLET_TITLES = {
  change_address: 'Change Address',
  rename: 'Rename',
  use_lnaddress: 'Use the Lightning Address',
  add: 'Add to All Releases',
  remove: 'Remove',
};

export function walletActions(entry) {
  return Object.entries(WALLET_TITLES)
    .filter(([kind]) => kind !== 'use_lnaddress' || hasNode(entry))
    .map(([kind, label]) => ({ kind, label, role: kind === 'remove' ? 'destructive' : 'action' }));
}

// The menu of the type of an address (ADR 0099 item 14).
export const WALLET_TYPES = [['node', 'Lightning Node'], ['lnaddress', 'Lightning Address']];

// The hint of the custom key (ADR 0099 item 14).
export const CUSTOM_KEY_HINT = 'Most wallets do not need it. Copy it from your wallet when it gives one.';

export const removeWalletHint = (title) => `Studio removes ${title} from each value block of the selected feeds. Studio changes the splits of the other recipients, and their total does not change.`;

// The rows of the feeds that use a wallet, from `POST /recipients/uses`.
// Each row starts selected (ADR 0099 item 7).
export function walletFeedRows(uses) {
  return (uses || []).map((use) => {
    const row = useRow(use);
    return { id: use.feed_id, title: row.title, lines: row.text ? [row.text] : [], checked: true };
  });
}

// The model of the sheet of one operation: its title, the check boxes of the
// feeds and the text of the primary button. The sheet "Add to All Releases"
// has no feed until the user selects an artist or label.
export function walletSheet(kind, uses) {
  const rows = kind === 'add' ? [] : walletFeedRows(uses);
  return { title: WALLET_TITLES[kind], rows, button: reviewLabel(checkedIds(rows).length) };
}

// The start values of the fields of a sheet. The address fields start at the
// member of the wallet with the type of the menu (ADR 0095 item 14). The
// user can change the values (an edit aid only).
export function walletSheetStart(kind, entry) {
  const type = startType(entry, kind);
  const start = memberStart(entry, type);
  return {
    type,
    address: kind === 'use_lnaddress' ? lnaddressOf(entry) || '' : start.address,
    custom_key: start.custom_key,
    custom_value: start.custom_value,
    name: entry.name || '',
    percent: '',
    owner: '',
  };
}

// A change of the type puts the member of the new type in the fields, when
// the user did not change the address.
export function walletTypeChange(entry, fields, next) {
  if (fields.address !== memberStart(entry, fields.type).address) return { ...fields, type: next };
  const member = memberStart(entry, next);
  return { ...fields, type: next, address: member.address, custom_key: member.custom_key, custom_value: member.custom_value };
}

// The summary line of the collapsed entry with the details of the new
// recipient (ADR 0099 item 10).
export function walletDetailsLine(fields) {
  const type = (WALLET_TYPES.find(([value]) => value === fields.type) || [null, ''])[1];
  const address = fields.address.trim() === '' ? 'No address yet' : fields.type === 'node' ? shortKey(fields.address.trim()) : fields.address.trim();
  return `${type} · ${address}`;
}

const optional = (text) => (String(text ?? '').trim() === '' ? null : String(text).trim());

// The `operation` of `POST /change-sets`. A Lightning address has no custom
// pair (ADR 0095 item 11). The core checks each value.
export function walletSheetBody(kind, fields) {
  const custom = fields.type === 'lnaddress'
    ? { custom_key: null, custom_value: null }
    : { custom_key: optional(fields.custom_key), custom_value: optional(fields.custom_value) };
  switch (kind) {
    case 'change_address':
      return { kind, address: fields.address.trim(), ...custom, recipient_type: fields.type || null };
    case 'rename': return { kind, name: fields.name };
    case 'remove': return { kind };
    case 'use_lnaddress': return { kind, address: fields.address.trim() };
    default:
      return { kind, name: optional(fields.name), recipient_type: fields.type || null, address: fields.address.trim(), ...custom, percent: Number(fields.percent) };
  }
}

// True when the primary button works: one feed is selected and the fields
// that the operation needs have a value.
export function walletSheetReady(kind, fields, ids) {
  return ids.length > 0
    && (kind === 'rename' || kind === 'remove' || fields.address.trim() !== '')
    && (kind !== 'add' || fields.percent !== '');
}


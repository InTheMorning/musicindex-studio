// The model of the Recipients pages and of the review of a change set
// (ADR 0091, ADR 0042). The functions have no Preact and no browser state.
// `app.js` shows the pages. They give texts and states only. The core
// calculates each split and finds each place (ADR 0065).

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
export function recipientTitle(recipient) {
  if (recipient.name && recipient.name.trim() !== '') return recipient.name;
  const first = (recipient.addresses || [])[0];
  return first ? first.address : recipient.address;
}

const TYPE_LABELS = { node: 'node (keysend)', lnaddress: 'lnaddress' };

export function typeLabel(type) {
  return TYPE_LABELS[type] || (type ? type : 'Type not set');
}

// The different types of the members of a wallet, "lnaddress" first.
export function typesText(entry) {
  const members = entry.addresses || [];
  if (members.length === 0) return typeLabel(entry.recipient_type);
  const types = [...new Set(members.map((m) => m.recipient_type))];
  types.sort((a, b) => (b === 'lnaddress') - (a === 'lnaddress'));
  return types.map(typeLabel).join(' · ');
}

// One line for each member of a wallet, for the header of its page.
export function addressLines(entry) {
  const members = entry.addresses || [];
  if (members.length === 0) return [{ address: entry.address, type: typeLabel(entry.recipient_type) }];
  return members.map((m) => ({ address: m.address, type: typeLabel(m.recipient_type) }));
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

// The start value of a type menu. The user can change it (an edit aid only).
export function defaultType(address) {
  return (address || '').includes('@') ? 'lnaddress' : 'node';
}

// One picker row for each member of each wallet. A member with no name gets
// the name of its wallet (ADR 0027 item 3).
export function pickerRows(list) {
  return (list || []).flatMap((entry) => {
    const members = entry.addresses || [];
    if (members.length === 0) return [entry];
    return members.map((m) => ({ ...m, name: m.name || entry.name }));
  });
}

// The hint of a Lightning address that has no keysend data.
export function lookupHint(address) {
  const domain = (address || '').split('@').slice(1).join('@');
  return `Studio found no keysend data at ${domain}. An app that pays only with keysend cannot pay this address.`;
}

// The recipients A to Z by name (ADR 0091 item 2 and task 054c item 2).
export function sortRecipients(list) {
  return (list || []).slice().sort((a, b) => recipientTitle(a).localeCompare(recipientTitle(b), undefined, { sensitivity: 'base' }));
}

// The summary line of a recipient: the name, the type, and "N feeds" or
// "Saved, not used" (ADR 0091 item 2). A recipient that is not saved and that
// no feed model uses now shows "0 feeds".
export function recipientRow(recipient) {
  const unusedSaved = !(recipient.feeds > 0) && recipient.saved_id != null;
  return {
    title: recipientTitle(recipient),
    type: typesText(recipient),
    text: unusedSaved ? 'Saved, not used' : plural(recipient.feeds || 0, 'feed', 'feeds'),
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

// Each feed of a list is selected at the start (ADR 0091 item 3).
export function allSelected(feeds) {
  return new Set((feeds || []).map((feed) => feed.feed_id));
}

// The selected feeds in the sequence of the list, for the body of a request.
export function selectedIds(feeds, selected) {
  return (feeds || []).map((feed) => feed.feed_id).filter((id) => selected.has(id));
}

export function toggled(selected, id) {
  const next = new Set(selected);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  return next;
}

// The text of the sheet button that counts the selected feeds.
export function feedCount(n) {
  return plural(n, 'feed', 'feeds');
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
  in_open_change_set: 'The feed has a change in a different change set that is not written.',
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

// The title of a change set: its operation and its recipient.
export function operationTitle(operation) {
  switch (operation && operation.kind) {
    case 'change_address': return 'Change address';
    case 'rename': return 'Rename';
    case 'remove': return 'Remove';
    case 'add': return 'Add to a level';
    case 'use_lnaddress': return 'Use the Lightning address';
    default: return 'Change set';
  }
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

// The row of an open change set on the Recipients page.
// The name of the recipient of a key in a list of recipients, or the
// address when the list does not have the key.
export function nameOf(list, key) {
  const same = (a, b) => (a || '') === (b || '');
  const found = (list || []).find((r) => r.address === key.address
    && same(r.custom_key, key.custom_key) && same(r.custom_value, key.custom_value));
  return found ? recipientTitle(found) : key.address;
}

export function changeSetRow(set, list = []) {
  const changed = (set.feeds || []).filter((feed) => feed.outcome && feed.outcome.outcome === 'changed').length;
  return {
    title: `${operationTitle(set.operation)} · ${nameOf(list, set.recipient)}`,
    text: `${plural(changed, 'feed', 'feeds')} changed`,
  };
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
  return `Write ${plural(count, 'feed', 'feeds')}`;
}

export function hasWrittenFeed(changeSet, statuses) {
  return (changeSet.feeds || []).some((feed) => feed.written_at != null || (statuses[feed.feed_id] || {}).state === 'written');
}

export function isWriting(statuses) {
  return Object.values(statuses).some((status) => status.state === 'writing');
}

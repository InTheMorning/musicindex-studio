// The model of the export and the import of an account (ADR 0107, ADR 0042).
// This module has no Preact and no browser state. The Node tests in
// `ui-tests/move.test.mjs` examine it. Each function gives the text of a
// screen from the data of the API. The core owns the rules, for example
// `movable`, and the UI only shows them (ADR 0065).
import { sizeText } from './assets.js';
import { npubOf } from './signing.js';

// The count with its noun, such as "1 draft" or "3 drafts".
function counted(count, one, many) {
  return `${count} ${count === 1 ? one : many}`;
}

const MONTHS = ['January', 'February', 'March', 'April', 'May', 'June', 'July',
  'August', 'September', 'October', 'November', 'December'];

// The date of a time in seconds, such as "1 November 2026", in the local time
// of the browser (ADR 0093).
export function fullDay(seconds) {
  const time = new Date(seconds * 1000);
  return `${time.getDate()} ${MONTHS[time.getMonth()]} ${time.getFullYear()}`;
}

// The sheet title: the Account panel of a server, or Settings of the desktop.
export function exportTitle(server) {
  return server ? 'Export Account' : 'Export Studio';
}

// The size of an archive, such as "about 3.4 MB". The numbers are estimates.
export function aboutSize(bytes) {
  return `about ${sizeText(bytes)}`;
}

// The names of the ways that open the vault on a different server.
const VAULT_WAYS = {
  passphrase: 'your passphrase',
  recovery: 'your recovery key',
  nostr: 'your Nostr signer',
};

// The text of the row "Vault". `methods` is the list of `export-facts`.
function vaultText(methods) {
  const ways = methods.filter((m) => Object.hasOwn(VAULT_WAYS, m)).map((m) => VAULT_WAYS[m]);
  if (ways.length === 0) return 'Does not open on a different server';
  const last = ways.pop();
  return `Opens with ${ways.length ? `${ways.join(', ')} and ` : ''}${last}`;
}

export const EXPORT_NOTICE = {
  strong: 'A person who has this file can read your feeds, drafts, versions and People list.',
  text: 'Your passwords and keys stay locked in your vault. Keep this file private.',
};

export const EXPORT_WARNING = {
  strong: 'Your vault cannot open on a different server.',
  text: 'Only your passkeys open it, and a passkey works only on this server. Make a recovery key first. Keep it in a safe place.',
};

export const EXPORT_WARNING_DESKTOP = {
  strong: 'Your vault cannot open in a different Studio.',
  text: 'Only your passkeys open it, and a passkey works only on this server. Make a recovery key first. Keep it in a safe place.',
};

// The view of the export sheet. `facts` is the answer of `GET
// /account/export-facts`. `includeAssets` is the state of the check box. The
// `screen` is `warn` (screen 3 of the mockups), `plain` (screen 1: the
// archive has no choice) or `audio` (screen 2: the user chooses).
export function exportView(facts, { server, includeAssets = true }) {
  const title = exportTitle(server);
  if (!facts.movable) {
    return {
      screen: 'warn',
      title,
      intro: null,
      notice: server ? EXPORT_WARNING : EXPORT_WARNING_DESKTOP,
      error: true,
      rows: [],
      checkbox: null,
      hint: 'Studio shows the recovery key one time, then makes the file.',
      size: null,
      primary: 'Make a Recovery Key',
    };
  }
  const choice = facts.holds_assets;
  const withAssets = choice && includeAssets;
  const bytes = withAssets ? facts.archive_bytes : facts.archive_bytes_without_assets;
  const rows = [
    {
      label: 'Feeds and drafts',
      summary: [counted(facts.feeds, 'feed', 'feeds'), counted(facts.drafts, 'draft', 'drafts'),
        counted(facts.versions, 'version', 'versions')].join(' · '),
    },
    { label: 'People', summary: counted(facts.people, 'person', 'people') },
    { label: 'Vault', summary: vaultText(facts.methods || []) },
    choice
      ? { label: 'Audio and images', summary: facts.asset_bytes > 0 ? aboutSize(facts.asset_bytes) : 'No audio and no images in the library' }
      : { label: 'Audio and images', summary: 'This server keeps no audio and no images. Your file hosts keep them.' },
  ];
  return {
    screen: choice ? 'audio' : 'plain',
    title,
    intro: 'Studio makes one file with your feeds, drafts, versions, people, vault and connected apps.',
    notice: EXPORT_NOTICE,
    error: false,
    rows,
    checkbox: choice ? 'Include the audio and images' : null,
    hint: server
      ? 'The export changes nothing. This account stays on this server until you delete it.'
      : 'The export changes nothing. Your data stays in this Studio.',
    size: `The file is ${aboutSize(bytes)}.`,
    primary: title,
  };
}

// The text of the sheet after the download starts.
export const EXPORT_DONE = {
  strong: 'The download started.',
  text: 'A person who has this file can read your feeds, drafts, versions and People list. Keep this file private.',
};

// The text above the recovery key in the export sheet.
export const RECOVERY_KEY_TEXT = 'Studio shows this key one time only. With it, you can open the vault on a different server or in a different Studio.';

// The notice of the Account panel after an import (ADR 0107 item 14,
// screen 8). `origin` is the address of this server. The session has the
// flag `imported` until the user adds a passkey or closes the notice.
export function importedNotice({ origin, passkeys }) {
  const address = `Each connected app must use the address of this server: ${origin}.`;
  return {
    strong: 'This account came from a file.',
    text: passkeys ? `Add a passkey for this server. ${address}` : address,
  };
}

// True when the Account panel shows the notice of an import.
export function showsImportNotice(session) {
  return !!session && session.imported === true;
}

// The text of the progress row while Studio sends the file.
export function sendingText(sent, size) {
  return `Sending… ${sizeText(sent)} of ${sizeText(size)}`;
}

export const CHECKING_TEXT = 'Checking the file…';

// The facts rows of the page that opens the vault (screen 6). `check` is the
// answer of the check route.
export function importFacts(check) {
  const account = check.account || {};
  const key = (account.nostr_keys || [])[0];
  const name = account.email || (key ? npubOf(key) : null) || 'No email address';
  const counts = check.counts || {};
  const day = fullDay(check.exported_at);
  return [
    { label: 'Account', summary: name },
    { label: 'Exported', summary: `${day} from Studio ${check.studio_version}` },
    {
      label: 'Content',
      summary: [counted(counts.feeds, 'feed', 'feeds'), counted(counts.drafts, 'draft', 'drafts'),
        counted(counts.people, 'person', 'people')].join(' · '),
    },
  ];
}

// The ways to open the vault of the file, in sequence. `check.methods` has
// `passphrase`, `recovery` and `nostr`. The extension needs `hasNip07`. The
// remote signer is only on a server (ADR 0107 item 12).
export function importMethods(check, { server, hasNip07 }) {
  const has = (name) => (check.methods || []).includes(name);
  const methods = [];
  if (has('passphrase')) methods.push('passphrase');
  if (has('recovery')) methods.push('recovery');
  if (has('nostr') && (check.nostr_wrappers || []).length > 0) {
    if (hasNip07) methods.push('nip07');
    if (server) methods.push('nip46');
  }
  return methods;
}

// What the open page shows. `mode` is the method that the user selected, or
// `null` for the first method. The result has the form field of the secret
// (`passphrase` or `recovery`), the email field and the buttons.
export function importOpenView(check, { server, hasNip07, mode = null }) {
  const methods = importMethods(check, { server, hasNip07 });
  const secrets = methods.filter((m) => m === 'passphrase' || m === 'recovery');
  const current = secrets.includes(mode) ? mode : secrets[0] || null;
  const others = methods.filter((m) => m !== current);
  const labels = {
    passphrase: 'Use the Passphrase',
    recovery: 'Use the Recovery Key',
    nip07: 'Use a Signer Extension',
    nip46: 'Use Amber or a Bunker',
  };
  // Only a server needs the email address, and only when the manifest has
  // none and the user opens the vault with a secret.
  const needsEmail = server && check.needs_email === true && current !== null;
  return {
    secret: current,
    needsEmail,
    // The desktop has no account to make. It imports the file.
    intro: server ? 'Studio makes the account when the vault opens.' : 'Studio imports the file when the vault opens.',
    primary: current ? (server ? 'Open and Make the Account' : 'Open and Import') : null,
    others: others.map((id) => ({ id, label: labels[id] })),
    noMethod: methods.length === 0,
  };
}

const IMPORT_CAUSES = {
  unknown_format: 'The file is not an account file of Studio, or it has a format that this Studio does not know.',
  newer_schema: 'The file comes from a newer Studio. Update this Studio, then try again.',
  unsafe_path: 'The file has an entry with an unsafe path. Studio refuses it.',
  link_entry: 'The file has a link. An account file has none.',
  special_entry: 'The file has an entry that is not a file or a folder.',
  unexpected_entry: 'The file has an entry that an account file does not have.',
  duplicate_entry: 'The file has the same entry two times.',
  missing_manifest: 'The file has no manifest. It is not an account file of Studio.',
  invalid_manifest: 'The manifest of the file cannot be read.',
  missing_file: 'The file is not complete. Export the account again.',
  invalid_collection: 'The collection in the file cannot be read.',
  too_many_entries: 'The file has too many entries.',
  too_large: 'The file is larger than this Studio accepts.',
  unreadable: 'Studio cannot read the file. It is damaged or it is not a tar file.',
  no_portable_method: 'The vault in the file has only passkeys. Make a recovery key on the old server, and export again.',
};

// The text of a failed import step. `message` is the message of the API. For
// `archive_invalid` it starts with the cause code.
export function importErrorText(code, message = '') {
  if (code === 'archive_invalid') {
    const cause = String(message).split(/[\s:]/)[0];
    return Object.hasOwn(IMPORT_CAUSES, cause)
      ? IMPORT_CAUSES[cause]
      : 'Studio cannot use this file. Select a file that "Export Account" made.';
  }
  return IMPORT_ERRORS[code] || null;
}

const IMPORT_ERRORS = {
  account_exists: 'This server has an account with this email address or this key. Sign in, or use a different server.',
  storage_full: 'The file is larger than the space of a new account on this server.',
  bad_size: 'The file is empty. Select the file that "Export Account" made.',
  too_many_imports: 'Too many imports. Try again later.',
  upload_incomplete: 'The server did not get the whole file. Select the file again.',
  email_required: 'Give an email address for the new account.',
  invalid_email: 'Give an email address.',
  pubkey_not_in_archive: 'This signer has another key than the file.',
  no_nostr_wrapper: 'The vault in the file has no key for this signer.',
  collection_not_empty: 'This collection has feeds. Import into a new collection or a new data folder.',
  library_not_empty: 'The library folder of this collection has files. Import into a new collection or a new data folder.',
  wrong_key: 'The passphrase or the key does not open the vault of the file.',
  invite_required: 'Sign-up needs an invite code that nobody used.',
};

// True when the user can try the open step again with other input.
export function canRetryOpen(code) {
  return ['wrong_key', 'email_required', 'invalid_email', 'pubkey_not_in_archive', 'no_nostr_wrapper'].includes(code);
}

// The text above the file button of the import page, by mode.
export function importSelectText(server) {
  return server
    ? 'Select the file that "Export Account" made on a different server, or that Studio desktop made.'
    : 'Select the file that "Export Account" made on a server, or that "Export Studio" made on a different computer.';
}

export const IMPORT_HINT = 'The import changes no feed and writes nothing to your file hosts.';

export const DESKTOP_IMPORT_HINT = 'Import the file of "Export Account" from a server, to keep your drafts, people and vault.';

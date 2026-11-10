// The Welcome page of a new user and the first steps of an artist page or a
// label page (ADR 0119, task 083). The functions have no Preact and no browser
// state. `app.js` and `editor.js` show the rows and the sheets.
import { isPublisher } from './shell.js';
import { feedFileStart } from './addfeeds.js';

// The texts of the Welcome page (ADR 0119 items 2 and 3).
export const WELCOME = {
  title: 'Welcome to Studio',
  intro: 'Studio publishes your music so that every music app can play it. What do you want to start with?',
  rows: {
    music: { icon: 'music', title: 'I make music', line: 'Start with your artist page. Your albums go on it.' },
    label: { icon: 'tag', title: 'I run a label', line: 'Start with your label page. Your artists and releases go on it.' },
    online: { icon: 'search', title: 'My music is online already', line: 'Find your artist or albums, and bring them into Studio.' },
  },
  mspBefore: 'Moving from MSP 2.0?',
  mspButton: 'Import the Backups',
  importBefore: 'Moving from a different computer or server?',
  importButton: 'Import an Account',
};

// The texts of the file host step (ADR 0122 items 3, 6 and 10). The text of
// item 10 has no caller yet. Task 086c shows it on an expired feed.
export const HOST_STEP = {
  temporary: 'Kept for 7 days. Pick a real file host to keep your pages.',
  placeholderGuid: 'Studio cannot make the GUID yet. A GUID comes from the public address of the feed, and this file host has none. Studio sets the GUID when the feed gets a public address.',
  expired: 'The temporary area has expired. Move this feed to a real file host.',
  placeholderAddress: 'Studio cannot make the address of this file yet. The address comes from the public address of the feed, and this file host has none. Studio sets the address when the feed gets a public address.',
};

// The texts and the step names of each path (ADR 0119 items 4 to 6).
const PATHS = {
  music: {
    role: 'artist',
    steps: ['Your name', 'Where it lives', 'Your first album'],
    sheet: 'Your artist page',
    nameLabel: 'Artist name',
    hint: 'A band is one artist. Add the members later, in People.',
    page: 'artist page',
  },
  label: {
    role: 'label',
    steps: ['Label name', 'Where it lives', 'Your first artist'],
    sheet: 'Your label page',
    nameLabel: 'Label name',
    hint: null,
    page: 'label page',
  },
};

// The step names of a path, `music` or `label`.
export function welcomeSteps(path) {
  return PATHS[path].steps;
}

// The texts of the sheet of a path: its title, the label of the name, the
// hint of the name and the text of step 2.
export function pathTexts(path) {
  const { sheet, nameLabel, hint, page } = PATHS[path];
  return {
    sheet,
    nameLabel,
    hint,
    where: `Studio does not keep your music. It puts your ${page} and your songs on a file host that you choose. You can move them later.`,
  };
}

// Is the Welcome page shown? It is shown when the collection has no feed (ADR 0119 item 1).
export function showWelcome(feeds) {
  return (feeds || []).length === 0;
}

// The model of step 2 (ADR 0122 items 2 and 4). `menu` holds the connected file
// hosts, with the first one selected, or null with no file host. `rows` are
// the rows of the file host types. `selected` is the type of the first row
// with no file host. With a file host, the menu is selected at the start, so
// `selected` is null and the step has one primary button.
export function hostStepModel(connectors, serverMode) {
  const list = connectors || [];
  const rows = HOST_ROWS(serverMode);
  const menu = list.length > 0 ? { value: String(list[0].id), options: list.map((c) => [String(c.id), c.name]) } : null;
  return { menu, rows, selected: menu ? null : rows[0].type };
}

// The rows of the file host step (ADR 0122 items 2 and 3). The first row is
// the default row. It is selected at the start. Each row gives the type of
// its form. A server has the temporary area as its default row.
export function HOST_ROWS(serverMode) {
  return [
    serverMode
      ? { type: 'temporary', icon: 'hard-drive', primary: true, title: 'Try it here (temporary)', line: HOST_STEP.temporary }
      : { type: 'local', icon: 'hard-drive', primary: true, title: 'Folder on this computer', line: 'A folder on this computer. The public address is optional.' },
    { type: 'nsite', icon: 'upload', title: 'Nostr site (nsite)', line: 'A site on Nostr. Its pages and songs go to Blossom servers, signed with your Nostr key.' },
    { type: 's3', icon: 'cloud', title: 'S3 or Cloudflare R2', line: 'A storage bucket at Amazon S3, Cloudflare R2 or a similar provider.' },
    { type: 'sftp', icon: 'hard-drive', title: 'SFTP server', line: 'Your own SFTP server.' },
  ];
}

// The body of `POST /feeds/new` for the first publisher feed of a path (ADR
// 0119 item 7). The name is the title and the author. `feedFileStart` gives
// the path (ADR 0073 item 8).
export function welcomeBody(path, name, connectorId) {
  return {
    connector_id: connectorId,
    path: feedFileStart(name),
    title: name,
    author: name,
    medium: 'publisher',
    role: PATHS[path].role,
  };
}

// The next step of the page of a publisher feed (ADR 0119 item 8). It is
// `album` for an artist with no release, `label` for a label with no
// release, and `null` for any other feed. `release_ids` holds the releases
// of the feed, and `child_ids` the publisher feeds below it (ADR 0088).
export function nextStep(feed) {
  if (!feed || !isPublisher(feed)) return null;
  if ((feed.release_ids || []).length > 0 || (feed.child_ids || []).length > 0) return null;
  const role = (feed.publisher_roles || [])[0];
  if (role === 'artist') return 'album';
  if (role === 'label') return 'label';
  return null;
}

// The rows of the page of a new publisher feed (ADR 0119 item 8). `open` is
// the row of the Add feeds page that the row opens.
export const FIRST_STEPS = {
  album: {
    icon: 'disc',
    title: 'Add your first album',
    line: 'Drop the songs in. Studio reads their names and order, and makes the album page.',
    action: 'Add an Album',
    open: 'album',
  },
  label: [
    { icon: 'music', title: 'Add an artist', line: 'Make a page for an artist on your label. Their albums go on it.', open: 'publisher' },
    { icon: 'disc', title: 'Add a release', line: 'An album that the label puts out, with its artist.', open: 'album' },
    { icon: 'search', title: 'Find an artist that is online', line: 'Link an artist page that exists already.', open: 'find' },
  ],
};

// The start values of the form that a first-step row opens (ADR 0119 item
// 8). An album of an artist starts with the artist as its author. A new
// artist of a label starts with the role `artist`. These are edit aids, not
// feed rules (ADR 0065).
export function firstStart(first, open, model) {
  if (first === 'album' && open === 'album') return { author: (model && model.title) || '' };
  if (first === 'label' && open === 'publisher') return { role: 'artist' };
  return {};
}

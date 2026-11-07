# ADR 0149: The Desktop App Has Many Collections

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Accepted: 2026-11-06 by the operator, from a mockup and a questionnaire.
- Technical choices: items 3, 4, 10, 12, 13, 16, 18 and 19 are choices of the planning model.
- Changes: [ADR 0107](0107-a-user-can-export-and-import-an-account.md) item 9 on the desktop app. The data folder of [ADR 0124](0124-the-desktop-app-is-a-tauri-window-on-the-local-api.md).
- Source: [findings of local use, findings 12, 13 and 17](../reviews/2026-11-06-local-use-findings.md)

## Context

A test of a new start, an import or an export needs a different data folder. The installed app reads no `--data-dir`, so a user with feeds cannot import an account. A check with real releases also needs the file hosts of the real collection.

## Decision

### Folders

1. The default data folder holds each collection in `collections/{folder}/`, and a file gives the names and the collection that is open.
2. The first start after this change moves the current data into `collections/main/`. The name is "Main collection".
3. The move is one rename in each top level entry, under the lock of the data folder. If it stops, the next start completes it. Each stored absolute path into the data folder changes with it. A failure stops the start with a message, and Studio deletes nothing.
4. `--data-dir` opens that folder as one collection. It has no move and no list of collections.

### Operations

5. Settings has the section "Collections", on the desktop app only. A server has no such route.
6. The section lists each collection with its count of feeds, "Open" and "Delete…". It has "New Collection…", "Copy This Collection…" and "Import an Account File…".
7. A new collection starts on the Welcome page, with its own vault and its own passphrase.
8. A copy holds the feeds, the drafts, the versions, the people, the library and the vault. Its passphrase is the passphrase of its source.
9. The sheet of a copy shows its size. It warns that a write from the copy changes the real feed on the same file host.
10. A copy holds no connected app, no show grant and no queued announcement.
11. An import makes a new collection and opens it. No import replaces data. To overwrite a collection, the user deletes it and imports a file.
12. "Main collection" has no "Delete…", and the open collection has none.
13. A delete needs a confirmation sheet. It removes local data only, and it changes nothing on a file host.

### A Change Of Collection

14. "Open" restarts the app on the selected collection. A write that runs follows the "Quit" rules of ADR 0124 items 16 to 21, and the dialog and the tray menu say "Open After Write" and "Open Now". "Cancel" keeps the open collection selected. The vault of the new collection starts locked.
15. Only the open collection runs. A granted write to a collection that is not open gets an error, and its queued announcements wait until it opens.
16. The UI has no code for Tauri. A route of the local API tells the shell which collection to open. The plain `studio-desktop` shell opens the selected collection at its next start, and it does not restart.

### The Mark

17. When the open collection is not "Main collection", the bottom of the left panel has a row "Collection: {name}" in the accent color. The row opens Settings.

### Each Collection And The App

18. A new collection keeps its library in its own folder, so that two collections share no release folder. It starts on the Welcome page, with no library page.
19. The choice for a close of the window belongs to the app, so it stays in the default data folder.

## Alternatives Considered

- The current folder stays, and only new collections are subfolders. Not chosen by the operator: each collection now has one location.
- A change with no restart. Not chosen by the operator: each background worker must stop and start, and a write can reach the wrong collection.
- One vault for all collections. Not chosen by the operator: a test collection could read the production secrets.
- A mark at all times, or only in Settings. Not chosen by the operator: the first adds a row for a user with one collection, and the second hides a test copy.

## Consequences

- The first start after this change moves the only copy of the user's data. Tests must cover a stop during the move.
- A copy can use much disk space, and two collections with one feed can overwrite each other on a file host. The warning does not stop this.
- A user has more passphrases.

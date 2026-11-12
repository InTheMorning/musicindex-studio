# ADR 0176: The User Controls Each Collection

- Status: Accepted
- Date: 2026-11-11
- Owner: operator
- Accepted: 2026-11-11 by the operator, from the drawings of a questionnaire. The operator added Rename.
- Technical choices: items 4, 6, 8, 9 and 12 are choices of the planning model.
- Changes: [ADR 0149](0149-the-desktop-app-has-many-collections.md) items 2, 6, 11, 12 and 17. [ADR 0107](0107-a-user-can-export-and-import-an-account.md) item 3 and item 9 on the desktop app.
- Source: the human checks of 2026-11-11.

## Context

On 2026-11-11, the operator imported an account file into a new collection. The new collection held all the feeds and the file hosts. The first collection, "Main collection", could not be deleted, and no import could replace it. A run of a newer binary had also migrated the first collection, so the release candidate could not open it. The user must keep one current collection, and Studio stopped that.

The export was in the section "Move Studio", and the import was in the section "Collections". They are the two ends of one task.

## Decision

### No Collection Is Special

1. No collection has a protection that a different collection does not have. The first collection keeps the folder `collections/main/`, and its start name stays "Main collection". The user can rename it and delete it.
2. Each collection that is not open has "Open", "Rename…", "Replace with a File…" and "Delete…".
3. The open collection has "Rename…" and the tag "Open now". It has no "Delete…" and no "Replace with a File…", because its files are in use.
4. A start opens the collection that the list file names as open. A start does not make a new `collections/main/` when the list has no such entry.

### Rename

5. "Rename…" changes only the name in the list file. The folder does not change. The name rules of "New Collection…" apply.

### Delete And Replace

6. A delete and a replace each need a confirmation sheet. The sheet gives the name, the count of feeds and the parts that go: the feeds, the drafts, the versions, the people and the vault. It says that nothing changes on a file host.
7. The sheet has "Export First…", which makes an account file of that collection. The export rules of ADR 0107 items 4 and 5 apply.
8. "Replace with a File…" reads the account file into a new folder first, with the checks of ADR 0107 item 11. After a good read, one write of the list file moves the entry to the new folder. Then Studio removes the previous folder. The name of the collection stays. A read that is not good changes nothing. After a stop, a folder that the list does not name can stay, and Studio does not delete it.
9. The vault of the replaced collection is the vault of the file. The user opens it with a method of the file.

### Export Next To Import

10. On the desktop app, the section "Collections" has "Export This Collection…" adjacent to "Import an Account File…". The section "Move Studio" does not show on the desktop app.
11. A shell with no list of collections, such as `--data-dir`, keeps the section "Move Studio".

### The Mark

12. The row "Collection: {name}" at the bottom of the left panel shows when the list has more than one collection.

## Alternatives Considered

- Delete only, with no replace. Not chosen by the operator: an import and a delete is two steps, and a mistake between them leaves two collections.
- Main collection stays protected. Not chosen by the operator: it keeps data that the user wants to remove.
- A delete of the open collection, with an automatic change to a different one. Not chosen by the planning model: it hides a restart.

## Consequences

- A user can delete the only copy of data. The sheet offers an export first, and a delete changes nothing on a file host.
- A replace uses disk space for two copies until Studio removes the previous folder.
- The folder name of a collection can differ from its first folder name after a replace.

## Acceptance Criteria

Mechanical:

- A core test: a delete of `main` while a different collection is open removes the folder and the list entry. The next start opens the remaining collection, and makes no `collections/main/`.
- A core test: a delete or a replace of the open collection is refused.
- A core test: a rename changes the list file only.
- A core test: a replace with a bad file changes nothing. A replace with a good file keeps the name, and the entry holds the feeds of the file. A stop before and after the write of the list file gives an entry with a complete folder.
- An API test: an export of a collection that is not open gives the same archive as an export of that collection when it is open.
- A Node test: the UI model gives the actions of item 2 for a closed row and of item 3 for the open row.

Visual, by the planning model:

- The Data tab at 1280 px and at 390 px shows the rows and the buttons of items 2, 3 and 10, with no "Move Studio" section.

Human, by the operator. This gate stays open until it runs:

- In the desktop app, delete "Main collection", restart, and continue in the remaining collection.

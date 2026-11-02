# ADR 0108: People Data At Rest Stays Plain

- Status: Accepted
- Date: 2026-11-01
- Owner: operator
- Accepted: 2026-11-01 by the operator.
- Uses: [ADR 0014](0014-collection-in-sqlite.md) (the collection in SQLite), [ADR 0055](0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) items 9 and 14 (account deletion, and the work with a locked vault), [ADR 0098](0098-the-people-module.md) and [ADR 0103](0103-one-person-collects-the-items-of-many-suggestions.md) (the People module).

## Context

- ADR 0014 keeps the collection in SQLite with no encryption. The vault holds each secret. At that time, the collection held feeds, drafts and versions.
- ADR 0098 added the People module. The collection thus holds data about people who are not users of Studio: names, credit names, wallets, Nostr keys, support links, the people of an artist or a label, and dismissals.
- Most of these values are public, because the published feeds give them. The links between them are not always public. A merge of ADR 0103 records that two identities are one person, and the user makes that link.
- On a server, the operator of the computer can read each collection. The security audit of 2026-11-01 found this item (SEC-17).
- The work with a locked vault reads the collection (ADR 0055 item 14). The People page, the pickers and the suggestions also read it while the vault is locked.
- On 2026-11-01, the operator selected plain data at rest, with the reasons recorded.

## Decision

1. The People data stays in the collection with no encryption, as the other data of the collection does (ADR 0014).
2. The People data holds no secret. A field of a person that holds a secret is not permitted. A new field of a person needs a statement in its ADR that the field holds no secret.
3. On a server, the People page has the text "The operator of this server can read the people that you keep." The desktop has no such text.
4. `docs/runbooks/studio-server.md` tells the operator that each collection holds personal data about people who are not users. It also tells the operator that the backups hold the same data.
5. The deletion of an account removes the People data with the user folder (ADR 0055 item 9).
6. This decision is examined again before one of these events, and a new ADR records the result:
   - The server permits sign-up with no invite code.
   - A person gets a field with data that the feeds do not give, such as a note or an email address.

## Alternatives Considered

- Encrypt the People tables with the vault key. Not chosen by the operator. People, the pickers and the suggestions would not be available while the vault is locked. The other data of the collection is available (ADR 0055 item 14).
- Encrypt the whole collection, for example with SQLCipher. Not chosen by the operator. A locked vault could then show nothing, which removes the work of ADR 0055 item 14. It is also a large change of the storage.

## Consequences

- The operator of a server can read the people of each user, and the links that the user made between identities. The People page says this.
- People, the pickers and the suggestions continue to work while the vault is locked.
- A new field of a person needs a statement about secrets (item 2), and some fields need this decision to be examined again (item 6).

## Acceptance Criteria

Mechanical:

- A Node test: the People page model gives the text of item 3 on a server and no text on the desktop.

Visual: the People page on a server, with the text of item 3, at 1100 px and at 390 px with touch.

Manual: the runbook has the text of item 4. A test cannot read the runbook for its meaning.

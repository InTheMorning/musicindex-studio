# ADR 0020: The First Run Asks One Question

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

[ADR 0005](0005-collection-and-first-run-discovery.md) gives the first-run wizard three discovery methods and asks for a connector after each feed.
The first wizard asked for a passphrase first, then a host folder, then a typed path.
On 2026-10-22 the operator examined the wizard. The operator said that the wizard did not know the situation of the user, and that it must not ask many questions.

The users are different. Some host their own feeds. Some use Podhome, Wavlake or Fountain. Some are new to V4V. Many have more than one feed, and some have publisher feeds.
A feed address and the links in a feed tell most of this situation.

## Decision

### One Question

1. The wizard asks one question: "Do you already have a feed?"
   - Yes: the user pastes one or more feed addresses, one on each line, or searches musicindex by name.
   - No: the user starts a new feed. The feed editor comes in Phase 5. Until then, this path explains this and ends.
   - A small link starts an MSP 2.0 import.
2. The wizard asks no other question about the situation of the user.

### Studio Follows The Links

3. For each address, Studio reads the feed. Studio then follows its links one level:
   - For a publisher feed, Studio reads each `podcast:remoteItem` of the channel.
   - For a feed with `podcast:publisher`, Studio reads the publisher feed and its `podcast:remoteItem` list.
4. A `podcast:remoteItem` with no `feedUrl` gets its address from musicindex by its `feedGuid`.
5. Studio follows no link from a feed that it found through a link.
6. Studio reads a maximum of 100 feeds for one wizard run. The result tells the user when this limit stopped it.

### Studio Infers The Host

7. Studio groups the feeds by the host name of their addresses.
8. Studio knows some hosting services by their host names. Studio cannot write to them, so their feeds are read-only. The first list is:
   - `wavlake.com`: Wavlake
   - `feeds.fountain.fm`: Fountain
   - `serve.podhome.fm`: Podhome
9. For each other host name, Studio offers to connect the server or folder that holds the files. The user can also keep these feeds read-only.
10. A connector for a known service removes the service from the read-only list. That change needs the connector ADR, not a change of this ADR.

### One Result Screen

11. The result is one screen with the feeds grouped by host. Each feed is selected at the start. The user clears the feeds that the user does not want, and adds the others with one action.
12. A feed that is already in the collection shows this, and the wizard does not add it again.

### The Vault When First Necessary

13. The wizard does not start with the vault. Studio makes the vault the first time that the user adds a secret, such as the password of a server.
14. A read-only feed and a local folder need no secret, so they need no vault.
15. When a vault exists, Studio asks for its passphrase at start, as before.

## Alternatives Considered

- A question about the situation of the user, with four answers: own host, hosting service, Wavlake or Fountain only, new to V4V. Rejected by the operator: it asks the user what Studio can find.
- The vault as the first step. Rejected by the operator: a user who only reads feeds has no secret to protect.

## Consequences

- This decision replaces ADR 0005 item 2 (the three wizard methods) and item 3 (a connector after each feed).
- The core needs an HTTP client to read public feeds and the musicindex API.
- The list of known services is data in the core. A new entry needs no ADR.
- The server shell has no vault step, because its sign-in opens the vault ([ADR 0004](0004-server-sign-in-with-nostr-or-email.md)).

## Invariants

- The wizard asks one question before the first result.
- Studio does not follow links more than one level from an address that the user gave.

## Open Questions

- Should Studio also ask for the passphrase only when an action needs a secret, and not at start?

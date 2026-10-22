# ADR 0011: AGPL-3.0 License

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

The server shell lets a host operator run a public service ([ADR 0001](0001-one-core-with-desktop-and-server-shells.md)).
The mission of the project is self-hosting.

## Decision

musicindex-studio uses the GNU Affero General Public License, version 3.
The text is in `LICENSE`. It came from `https://www.gnu.org/licenses/agpl-3.0.txt` on 2026-10-22.

## Alternatives Considered

- MIT or Apache-2.0. Rejected: a host could run a changed public server and keep the changes private.

## Consequences

- A host operator that changes the server must give the source of the changes to its users.
- Each dependency must have a license that is compatible with AGPL-3.0.

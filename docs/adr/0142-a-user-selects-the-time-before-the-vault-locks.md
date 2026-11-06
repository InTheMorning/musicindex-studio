# ADR 0142: A User Selects The Time Before The Vault Locks

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Accepted: 2026-11-06 by the operator, who asked for the setting and accepted the proposal of finding 11.
- Technical choices: the times of item 1, the maximum of item 3 and items 4 to 6 are choices of the planning model.
- Changes: [ADR 0055](0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) item 16, the fixed time of the server.
- Source: [findings of local use, finding 11](../reviews/2026-11-06-local-use-findings.md)

## Context

The server locked a vault after a time that only its operator could change. The desktop app did not lock the vault by itself.

## Decision

1. Settings has the section "Lock" with the select menu "Lock Studio after". The times are 5, 10 and 30 minutes and 1 hour.
2. The desktop also has "Never", and it is the start value, so that a new version does not change how the app acts.
3. A server starts at `--vault-idle-minutes`. It has no "Never". `--vault-idle-max-minutes`, with the default 60, limits the times, because a longer time keeps a vault key in the memory of a public server for longer.
4. The collection keeps the time. A stored time that this shell does not offer does not count, and the start value applies. Thus an account import with "Never" still locks on a server.
5. The time counts from the last use of a vault secret, as in ADR 0055 item 16. The desktop checks the time each 30 seconds, and the server task does so for each user.
6. In both shells, the answer 423 `vault_locked` opens the unlock sheet, and the request then runs again.

## Alternatives Considered

- "Never" on a server. Not chosen: the key stays in the memory of a public server until the last session ends.
- A time from the last request. Not chosen: an open page that polls would keep the vault open.

## Consequences

- On the desktop, the edit of a draft does not use a secret, so a short time can lock the vault during an edit. A reload of the status then shows the Unlock page.

## How This Decision Is Checked

Mechanical: `crates/api/src/tests.rs` and `crates/server/src/tests.rs` examine the times of each shell, the refusal of a time and a stored "Never" on a server. `crates/server/src/tests.rs` examines the lock.

Visual: the planning model looks at the section at 1280 px, and at 390 px with touch, and runs a real lock of the desktop shell after 5 minutes.

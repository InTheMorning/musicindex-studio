# ADR 0104: Key Material Stays Off The Disk

- Status: Accepted
- Date: 2026-11-01
- Owner: operator
- Accepted: 2026-11-01 by the operator, from the inspection of the production storage on the same day.
- Uses: [ADR 0013](0013-vault-format-and-cryptography.md) items 10, 13 and 15 (no plain secret in a file, keys only in memory), [ADR 0055](0055-the-server-shell-user-folders-sessions-and-the-vault-window.md) item 2 (one user folder for each account) and [ADR 0066](0066-the-server-ships-as-a-container-image.md) (the server image).

## Context

- On 2026-11-01, an inspection of the production storage found two gaps.
- The runbook says that `LimitCORE=0` stops core dumps. That line is only in the systemd unit of the installation without Docker. `deploy/compose.yml` had no core limit.
- The host can send core dumps to `systemd-coredump`. The kernel does not always apply `RLIMIT_CORE` when `core_pattern` is a pipe. A crash with an open vault window could then write a vault key to the disk of the host.
- The vault file and the data folders got the mode of the umask, usually 0644 and 0755. Each user of the computer could read them.
- The workspace has `unsafe_code = "forbid"`. The safe call `rustix::process::set_dumpable_behavior` sets the dumpable state of the process.

## Decision

1. Each shell process marks itself not dumpable at its start, on Linux, before the first request. The server stops with an error when the call fails. The desktop writes a warning and continues. The server compose file also sets the core limit to 0.
2. On Unix, Studio makes each data folder, each user folder and `grants/` with the mode 0700. It makes each vault file and each grant vault file with the mode 0600. At its start, Studio sets these modes on an existing folder and on an existing `vault.json`. The server also sets `<data>` and `<data>/users`.
3. These items do nothing on Windows. The access control of the user folder of Windows applies.
4. The code uses the safe API of `rustix`. The vault format does not change.

## Alternatives Considered

- A core limit alone. Not sufficient, because a pipe in `core_pattern` can ignore it.
- `libc::prctl`. Not possible, because the workspace forbids `unsafe`.

## Consequences

- A crash writes no vault key to the disk of the host. A debugger cannot attach to the process of the server, and the operator gets no core dump for a crash.
- A user of the computer other than `studio` cannot read a vault file or a data folder.
- A backup tool that runs as a different user needs access to the data folder.

## Acceptance Criteria

Mechanical:

- On Linux, a server test and a desktop test call the start step, and find `NotDumpable`.
- A server test finds `core: 0` in the `ulimits` of the service `studio` in `deploy/compose.yml`.
- A core test, on Unix, saves a vault with the umask 0022, and finds the mode 0600.
- An API test, on Unix, opens a workspace in a new folder, and finds the mode 0700 on the folder and on `grants/`.
- An API test, on Unix, opens a workspace in a folder with the mode 0755 and a `vault.json` with the mode 0644. It finds 0700 and 0600 after the open.
- A server test, on Unix, opens the accounts in a new folder, and finds the mode 0700 on `<data>` and `<data>/users`.

Manual: the runbook gives the two mechanisms against core dumps and the repair of an existing Docker volume. A test cannot read the runbook for its meaning.

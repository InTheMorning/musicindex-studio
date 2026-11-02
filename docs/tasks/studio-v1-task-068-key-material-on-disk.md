# Studio v1 Task 068: Key Material On The Disk

Decision: ADR 0013 items 10, 13 and 15 (no plain secret in a file, keys only in memory), ADR 0055 item 2 (one user folder for each account) and ADR 0066 (the server image). Packet a writes ADR 0104 first. ADR 0104 makes the two guards of this task binding rules.

Status: Complete. The human test on the VPS is open.

## Goal

A vault key does not go to the disk in a core dump. Only the user `studio` can read a vault file or a data folder.

## Context

On 2026-11-01, an inspection of the production storage found two gaps.

1. **Core dumps.** The runbook says that `LimitCORE=0` stops core dumps. That line is in the systemd unit of the installation without Docker. `deploy/compose.yml` has no `ulimits`.
   - The host can send core dumps to `systemd-coredump`. Then a crash with an open vault window can write a vault key to the disk of the host.
   - The kernel does not always apply `RLIMIT_CORE` when `core_pattern` is a pipe. Thus a limit alone is not sufficient.
2. **File modes.** `VaultFile::save` in `crates/core/src/vault/file.rs` makes its temporary file with `fs::File::create`. The file thus gets the mode of the umask, usually 0644. `Workspace::open` in `crates/api/src/workspace.rs` and the `grants/` folder use `create_dir_all`, usually 0755. The `Dockerfile` makes `/data` with `mkdir -p`, thus 0755. The runbook gives 0700 only for a restore.

The workspace has `unsafe_code = "forbid"`. Thus a direct `libc::prctl` call is not permitted. `Cargo.lock` has `rustix` 1.1, and `rustix::process::set_dumpable_behavior` is a safe call.

## Packets

### 068a: The decision

Write ADR 0104 "Key material stays off the disk". It states two rules:

1. Each shell process marks itself not dumpable at its start, on Linux. The server compose file also sets the core limit to 0.
2. On Unix, Studio makes each data folder, each user folder and `grants/` with the mode 0700. It makes each vault file and each grant vault file with the mode 0600.

Add ADR 0104 to `docs/README.md` and to AGENTS.md in the same commit.

### 068b: No core dump

1. Add `rustix` with the feature `process` to `crates/server` and `crates/desktop`, for `target_os = "linux"` only.
2. At the start of `main`, before the first request, call `set_dumpable_behavior(DumpableBehavior::NotDumpable)`. If the call fails, the server stops with an error. The desktop writes a warning and continues.
3. In `deploy/compose.yml`, add `ulimits: { core: 0 }` to the service `studio`.
4. In `docs/runbooks/studio-server.md` section 7, tell the operator that the two installation types prevent core dumps. Give the two mechanisms.
5. In `deploy/compose.yml`, add `--argon2-concurrency ${STUDIO_ARGON2_CONCURRENCY:-2}` to the command. The server default of 2 does not change. Add `STUDIO_ARGON2_CONCURRENCY` to `deploy/env.example`, with the advice to use 1 on a computer with 1 CPU. On 2026-11-01, the production VPS had 1 CPU, 1.9 GiB of memory and a full swap. Two derivations on one CPU give no speed, and they use 128 MiB.

### 068c: Private files

1. `VaultFile::save` opens its temporary file with `OpenOptionsExt::mode(0o600)` on Unix. `connector/blossom_http.rs` line 74 has this pattern.
2. `Workspace::open` makes `data_dir` with `DirBuilderExt::mode(0o700)` on Unix. When the folder exists, it sets the mode to 0700. When `vault.json` exists, it sets the mode to 0600. This repairs the files of an existing installation at the next start.
3. The workspace makes `grants/` with the mode 0700.
4. `studio-server` makes `<data>` and `<data>/users` with the mode 0700 at its start.
5. In the `Dockerfile`, replace `mkdir -p /data && chown studio:studio /data` with `install -d -m 0700 -o studio -g studio /data`.
6. In the runbook, add one step for an existing Docker volume: Docker copies the mode of the image folder only into a new volume. Item 2 repairs the user folders, and item 4 repairs `<data>`.

On Windows, these items do nothing. The access control of the user folder of Windows applies.

## Files

Add: `docs/adr/0104-key-material-stays-off-the-disk.md`.

Change: `crates/core/src/vault/file.rs`, `crates/api/src/workspace.rs`, `crates/server/src/main.rs`, `crates/server/Cargo.toml`, `crates/desktop/src/main.rs`, `crates/desktop/Cargo.toml`, `deploy/compose.yml`, `deploy/env.example`, `Dockerfile`, `docs/runbooks/studio-server.md`, `docs/README.md`, `AGENTS.md`.

## Constraints

- No `unsafe` code. Use the safe API of `rustix`.
- No change to the vault format. A vault file from before this task opens.
- No change to the feed rules or the UI.
- When a new test fails, its message names ADR 0104 and the fix.

## Acceptance Criteria

Mechanical:

- On Linux, a server test calls the start step, then reads `rustix::process::dumpable_behavior()`. It finds `NotDumpable`. A desktop test does the same.
- A test in `crates/server` reads `deploy/compose.yml`, and finds `core: 0` in the `ulimits` of the service `studio`.
- A test in `crates/core`, on Unix, saves a vault with the umask 0022, and finds the mode 0600.
- A test in `crates/api`, on Unix, opens a workspace in a new folder, and finds the mode 0700 on the folder and on `grants/`.
- A test in `crates/api`, on Unix, opens a workspace in a folder with the mode 0755 and a `vault.json` with the mode 0644. It finds 0700 and 0600 after the open.
- Each new test fails when its fix is removed.
- The four commit checks pass.

Visual: none. This task has no UI change.

Human test, on the VPS after the deploy of the next release candidate:

- `docker exec` into the container, and `stat -c %a /data /data/users /data/users/*/vault.json` gives 700, 700 and 600.
- `docker exec` into the container, and `grep core /proc/1/limits` gives a core limit of 0.

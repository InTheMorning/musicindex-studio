# Studio v1 Task 043a: The Dockerfile And The Compose File

Parent: [task 043](studio-v1-task-043-server-image.md). Decision: [ADR 0066](../adr/0066-the-server-ships-as-a-container-image.md) items 1 to 5 and 10.

Status: Not started.

## Goal

`docker build .` makes an image that runs `studio-server`. A compose file runs it with the options of the runbook.

## Files

Inspect: `rust-toolchain.toml`, `Cargo.toml`, `crates/server/src/main.rs`, `crates/server/src/config.rs`, `docs/runbooks/studio-server.md` sections 2 to 4, `../stophammer/Dockerfile` (the model of the stages), `../stophammer/docker-compose.yml` (the service `primary`, the anchor `hardened`).

Change: a new `Dockerfile`, a new `.dockerignore`, a new `deploy/compose.yml`, a new `deploy/env.example`.

Do not change: the crates, the workflows, the runbook, `AGENTS.md`.

## Architecture

These choices are fixed. Do not change them.

- The build stage is `rust:<version>-alpine`, with the version of `rust-toolchain.toml`. It installs `build-base`, `perl`, `cmake`, `linux-headers` and `pkgconf` for the three C libraries. It builds `--release -p studio-server --locked`. Use `cargo-chef` as stophammer does only if it builds. If it does not, use one plain build and say so in the report.
- The runtime stage is `alpine:3.24` pinned by digest, the same digest as `../stophammer/Dockerfile`. It installs `ca-certificates`, makes the system user `studio`, and makes `/data` owned by it.
- The entry point is `studio-server`. The default command is `serve --data-dir /data --listen 0.0.0.0:8080`. The compose file adds the other options.
- The build arguments `STUDIO_REVISION` and `STUDIO_VERSION` set the labels `org.opencontainers.image.revision` and `org.opencontainers.image.version`.
- `.dockerignore` keeps `target/`, `.studio-dev/`, `.git/`, `.claude/` and `docs/` out of the context.
- `deploy/compose.yml` has one service `studio`:
  - `image: ${STUDIO_SERVER_IMAGE:-ghcr.io/inthemorning/studio-server:latest}`.
  - The command adds `--public-origin ${STUDIO_PUBLIC_ORIGIN}` and `--client-address-header X-Real-IP`.
  - `ports: ["127.0.0.1:8080:8080"]`, with a comment that says why it is loopback only.
  - The volume `studio-data:/data`, `read_only: true`, `tmpfs: [/tmp]`, `cap_drop: [ALL]`, `security_opt: [no-new-privileges:true]` and `restart: unless-stopped`.
  - No health check. The server refuses a `Host` that is not the public origin, so a check needs that header. Say this in a comment.
- `deploy/env.example` gives `STUDIO_SERVER_IMAGE` and `STUDIO_PUBLIC_ORIGIN=https://studio.musicindex.org`, with a comment for each.

## Steps

1. Write the four files.
2. Run `docker build -t studio-server:local .` and `docker run --rm studio-server:local --help`.
3. Run the image with `docker run --rm -p 127.0.0.1:18080:8080 studio-server:local serve --data-dir /data --listen 0.0.0.0:8080 --public-origin http://127.0.0.1:18080`, and request `http://127.0.0.1:18080/manifest.webmanifest`. It gives 200.
4. Run the four commit checks.

## Constraints

- Each comment is in ASD-STE100 Simplified Technical English.
- No change to a crate. When the build on musl needs a change to a crate or a different base, stop and report.
- Use the data folder `.studio-dev/` for each local test file, not `/tmp`.

## Acceptance Criteria

Mechanical:

- `docker build .` succeeds on this computer.
- The image starts with `--help`.
- The manifest request of step 3 gives 200.
- The four commit checks pass.

## Test Commands

```sh
docker build -t studio-server:local .
docker run --rm studio-server:local --help
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when a C library does not build on musl.
- Stop and report when `--listen 0.0.0.0:8080` is refused by the server.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-043a-dockerfile.md
- docs/adr/0066-the-server-ships-as-a-container-image.md
- The files that the Files section names.

Goal:
- A Dockerfile, a .dockerignore, deploy/compose.yml and deploy/env.example for studio-server.

Constraints:
- Follow the Architecture section of the task file fully.
- No change to a crate.

Do not touch:
- The crates, the workflows, the runbook, AGENTS.md.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- The commands of the Test Commands section.

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

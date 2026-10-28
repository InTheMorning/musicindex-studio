# Studio v1 Task 043c: The Deploy Script And The Runbook

Parent: [task 043](studio-v1-task-043-server-image.md). Decision: [ADR 0066](../adr/0066-the-server-ships-as-a-container-image.md) items 10 to 13.

Status: Complete on 2026-10-27. The first deployment is open.

## Goal

`tools/deploy-server.sh --release VERSION` runs the candidate image on the VPS and checks it. The runbook gives the container method as its main method.

## Files

Inspect: `deploy/compose.yml` and `deploy/env.example` of packet 043a, `../stophammer/deploy.sh` (the release mode: the pull before a stop, the pin in `.env`, the check of the revision), `docs/runbooks/studio-server.md`, `crates/server/src/main.rs` (the start line `Music Index Studio server <version>`).

Change: a new `tools/deploy-server.sh`, `docs/runbooks/studio-server.md`.

Do not change: the crates, the workflows, the `Dockerfile`, `deploy/compose.yml`.

## Architecture

These choices are fixed. Do not change them.

- The script has the usage `tools/deploy-server.sh --release VERSION [--host USER@HOST] [--dir PATH]`. The default host is `STUDIO_DEPLOY_HOST`, and the default folder is `STUDIO_DEPLOY_DIR`. The script reads them from the environment or from `.studio-dev/deploy.env`, which Git ignores. `--help` gives the usage.
- Before it connects, the script refuses a version with no local tag `vVERSION`, and a version with no hyphen (ADR 0066 item 9).
- It copies `deploy/compose.yml` to the folder on the VPS with `rsync`. It does not copy `.env`. When `.env` does not exist on the VPS, it stops and tells the operator to make it from `deploy/env.example`.
- On the VPS, it pulls `ghcr.io/inthemorning/studio-server:VERSION` first. When the pull fails, it stops and changes nothing.
- Then it sets `STUDIO_SERVER_IMAGE=ghcr.io/inthemorning/studio-server:VERSION` in `.env`, and runs `docker compose up -d`.
- The checks of ADR 0066 item 12, with a wait of a maximum of 60 seconds:
  - The label `org.opencontainers.image.revision` of the running container is the commit of the local tag `vVERSION` (`git rev-list -n 1 vVERSION`).
  - The log of the container has the line `Music Index Studio server VERSION`.
  - A difference stops the script with an error that names the expected value and the found value.
- The script uses `set -euo pipefail`, and it quotes each variable. It makes no account and no invite code.
- The runbook:
  - Section 2 gets the container method first: make `.env` from `deploy/env.example`, run `tools/deploy-server.sh --release VERSION`, and make the first invite code with `docker compose exec studio studio-server invite --data-dir /data`.
  - The build from source stays as "Without Docker", after the container method.
  - Section 3 keeps the line `client_max_body_size 9m;`.
  - Section 6 (backup) names the volume `studio-data` for the container method.
  - Section 9 (update) gives `tools/deploy-server.sh --release VERSION`.

## Steps

1. Write `tools/deploy-server.sh` and make it executable.
2. Change the runbook.
3. Run `bash -n tools/deploy-server.sh` and `shellcheck tools/deploy-server.sh` if it is installed.
4. Run `tools/deploy-server.sh --release 9.9.9-rc.1`. It must stop with the error of a missing tag, and connect to no host.
5. Run the four commit checks.

## Constraints

- Each comment and the runbook are in ASD-STE100 Simplified Technical English. Check the runbook with `~/.agents/skills/asd-ste100/scripts/ste_lint.py`.
- The script never connects to a host in a test.

## Acceptance Criteria

Mechanical:

- `bash -n` passes, and `shellcheck` gives no error when it is installed.
- Step 4 stops with the error of a missing tag.
- The four commit checks pass.

Manual, by the operator:

- The deployment of a candidate with the script, with its two checks.

## Test Commands

```sh
bash -n tools/deploy-server.sh
shellcheck tools/deploy-server.sh
tools/deploy-server.sh --release 9.9.9-rc.1
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when the start line of the server does not give the version.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-043c-deploy-script.md
- docs/adr/0066-the-server-ships-as-a-container-image.md
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section names.

Goal:
- tools/deploy-server.sh and the container method in the runbook.

Constraints:
- Follow the Architecture section of the task file fully.
- The script never connects to a host in a test.

Do not touch:
- The crates, the workflows, the Dockerfile, deploy/compose.yml.
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

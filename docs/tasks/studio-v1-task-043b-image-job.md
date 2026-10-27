# Studio v1 Task 043b: The Image Job

Parent: [task 043](studio-v1-task-043-server-image.md). Decision: [ADR 0066](../adr/0066-the-server-ships-as-a-container-image.md) items 3 and 5 to 9.

Status: Not started.

## Goal

A candidate tag builds the image for `amd64` and `arm64` on native runners, after a passed CI run on its commit. The workflow pushes the image to `ghcr.io/inthemorning/studio-server` with the tag of the version.

## Files

Inspect: `.github/workflows/ci.yml`, `.github/workflows/desktop-release.yml`, `tools/release_desktop.py` (`version`, the command `version --tag`), the `Dockerfile` of packet 043a, `../stophammer/.github/workflows/release.yml` (the jobs `ci-passed`, `build-images` and `merge-images`).

Change: a new `.github/workflows/server-image.yml`, `.github/workflows/desktop-release.yml` (the notes text of the draft only).

Do not change: `ci.yml`, the crates, the `Dockerfile`, the runbook.

## Architecture

These choices are fixed. Do not change them.

- The workflow starts on a pushed tag that matches `v*-*`, and on `workflow_dispatch` with no push. A run with no tag builds both CPU types and pushes nothing.
- Job `ci-passed`: a copy of the stophammer job. It waits for a run of `ci.yml` with `event=push` on `GITHUB_SHA`. The error text names ADR 0066 item 7 and the fix: "Push main before the candidate tag, so that CI tests the commit of the tag."
- Job `version`: runs `python tools/release_desktop.py version --tag "$GITHUB_REF_NAME"`. It gives the version with no `v` as an output.
- Job `build`: a matrix of `amd64` on `ubuntu-latest` and `arm64` on `ubuntu-24.04-arm`. It needs `ci-passed` and `version`. It builds with `docker/build-push-action`, with the build arguments `STUDIO_REVISION=${{ github.sha }}` and `STUDIO_VERSION`. It loads the image, runs it with `--help`, and then pushes it by digest. It uploads the digest as an artifact.
- Job `merge`: makes one manifest list from the two digests with `docker buildx imagetools create`, with the tag of the version. It does not set `latest`.
- `permissions` is `contents: read` at the top. Only `build` and `merge` get `packages: write`.
- Each action is pinned by a commit SHA with the version in a comment, as stophammer does. Use the SHAs of `../stophammer/.github/workflows/release.yml` for the same actions.
- `desktop-release.yml`: the notes text of the draft changes from "The server shell is not ready for public use." to "The server image is `ghcr.io/inthemorning/studio-server:<version>`."

## Steps

1. Write `server-image.yml`.
2. Change the notes text of `desktop-release.yml`.
3. Check both files with `actionlint` if it is installed, else with a YAML parser.
4. Run the four commit checks.

## Constraints

- Each comment is in ASD-STE100 Simplified Technical English.
- No secret other than `GITHUB_TOKEN`.

## Acceptance Criteria

Mechanical:

- `server-image.yml` parses, and `actionlint` gives no error when it is installed.
- The build job runs the image with `--help` before it pushes.
- The four commit checks pass.

Manual, after the operator pushes a candidate tag:

- The workflow pushes `ghcr.io/inthemorning/studio-server:<version>` with two CPU types.

## Test Commands

```sh
actionlint .github/workflows/server-image.yml .github/workflows/desktop-release.yml
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
tools/node --test crates/api/ui-tests/*.test.mjs
```

## Escalation

- Stop and report when `tools/release_desktop.py version --tag` cannot give the version as an output.
- Stop and report when a stophammer action has no pinned SHA to copy.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-043b-image-job.md
- docs/adr/0066-the-server-ships-as-a-container-image.md
- The files that the Files section names.

Goal:
- .github/workflows/server-image.yml, and one text change in desktop-release.yml.

Constraints:
- Follow the Architecture section of the task file fully.

Do not touch:
- ci.yml, the crates, the Dockerfile, the runbook.
- Do not commit. Do not push. Do not make a tag.

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

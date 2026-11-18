# Studio v1 Task 170: The Build Values In One Form

Decision: musicindex ADR 0083 (each app shows its version in one form). Studio has no ADR of its own for this task. This task cites the rules and does not write them again.

Status: Complete on 2026-11-18 for the mechanical checks and the screenshot check. The check of the Settings line by a person (R-VERSION-9 item 2) is open.

## Problem

Studio showed only the semantic version. The `--version` line, the status route and the Settings section had no git revision and no build time.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Build values | reuse | /home/citizen/build/musicindex-lib/build-info/src/lib.rs | musicindex-build-info at rev e6d1111. |
| Copy button | reuse | crates/api/ui/app.js | `CopyButton` copies the line. |
| Version section | extend | crates/api/ui/app.js | `VersionPanel` of task 165 shows the line. |
| Status route | extend | crates/api/src/routes.rs | It gives the build values. |
| Release version checks | extend | tools/release_desktop.py | The smoke test accepts the new line. |

## Changes

1. The workspace pins `musicindex-build-info` at rev `e6d11118d33069c26c01368f7de71524988039c6`.
2. `studio-desktop`, `studio-server` and `studio-desktop-app` have a build script that calls `build::emit("STUDIO")`. The input variables are `STUDIO_GIT_REVISION` and `STUDIO_BUILT_AT`.
3. `--version` and the start message print the version line. The desktop name is `Music Index Studio`. The server name is `Music Index Studio server`.
4. Each binary gives its build values to `studio_api::set_build_info` at its start. The value is one value for the process, because a collection switch opens a new `Workspace`.
5. `GET /api/v1/status` gives `version`, `git_revision`, `built_at` and `version_line`.
6. Settings > General shows the line as its last section, with the copy button.
7. The image has no `.git` folder. The `Dockerfile` gives the first 7 characters of the build argument `STUDIO_REVISION` to the build as `STUDIO_GIT_REVISION`.
8. The image check in `server-image.yml` and the smoke test in `tools/release_desktop.py` accept the line with the git revision and the date.

## Checks

Mechanical:

- `crates/api/tests/build_info.rs`: the status route gives the three build values that the binary sets (R-VERSION-9 item 1).
- `crates/desktop/tests/version.rs` and `crates/server/tests/version.rs`: `--version` gives the version, the git revision and the build date (R-VERSION-9 item 1).
- `tools/gate.sh` passes.

Visual, for a person:

- Settings > General shows the line at its end. The text can be selected, and the copy button copies the line (musicindex ADR 0083 R-VERSION-9 item 2). Open.

## Screenshot Check

`.studio-dev/walk/steps-170-version.json` with `setup-none.py`. The shots are in `.studio-dev/walk/runs/v170/shots/`.

- 1280 px, Light: the line is the last section of General. A selection holds the full line. No edge cut and no overflow.
- 390 px, Dark: the line breaks into two lines. No edge cut and no overflow.

## Open Items

- R-VERSION-7: the operator changes the version in `Cargo.toml` and makes the tag for each release. This task changes no release step.

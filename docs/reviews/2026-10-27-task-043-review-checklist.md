# Task 043 Review Checklist, 2026-10-27

This record holds the orchestration of [task 043](../tasks/studio-v1-task-043-server-image.md). It records each packet and each review result. It states no rule. [ADR 0066](../adr/0066-the-server-ships-as-a-container-image.md) owns the rules.

## Sequence

A coding model implements each packet. The planning model examines the diff and runs the four commit checks. It commits a packet after its review passes, as the operator decided on 2026-10-27.

| Packet | Status | Review |
|---|---|---|
| 043a | Complete | Pass on 2026-10-27. The operator built the image on musl. `--version` gave the version, and the manifest gave 200. The image is 17 MB of content. |
| 043b | Complete | Pass on 2026-10-27. The build job runs the image with `--version` before the push. `actionlint` is not installed, so only a YAML parse examined the files. The first run on a candidate tag is open. |
| 043c | Complete | Pass on 2026-10-27. The two refusals work with no connection. `shellcheck` is not installed. The first deployment is open. |

## Decisions

- The operator decided on 2026-10-27 that GitHub builds a server image, as for stophammer. The operator selected Alpine with musl, the CPU types `amd64` and `arm64`, and a deploy script with `--release`.
- The operator accepted ADR 0066 on 2026-10-27.

### Packet 043a

- The packet said that Docker works on the computer of the planning model. Only the Docker client is in its sandbox, with no daemon. The coding model wrote the four files and stopped before the build. The operator runs the build in the terminal of the operator.
- `studio-server` had no `--help` and no `--version`, so the smoke test of ADR 0066 could not pass. The planning model added both options, as `studio-desktop` has them. `--version` prints the start line of the server.
- The first manifest request came directly after the start, and it got no connection. At start, the server makes its state with an Argon2 derivation, and that takes several seconds. After a wait of 10 seconds, the log gave the start lines and the manifest gave 200. The deploy script of packet 043c waits a maximum of 60 seconds.

### Packet 043b

- The build job runs `--version` in place of `--help`, and compares the output with the version of the tag.
- GHCR makes a new package private. If the VPS pulls with no login, the operator sets the package `studio-server` to public after the first push.

### Packet 043c

- The log check used `docker logs | grep -q` with `pipefail`. When `grep` stops at the first match, `docker logs` can fail, and the check then reports a missing line. The planning model put the log in a variable first.
- The runbook gave `docker volume inspect studio-data`. Compose puts the project name before the volume name, so the planning model changed it to `studio_studio-data`.
- The nginx block of `studio.musicindex.org` got `client_max_body_size 9m` in commit `3af81a2` of `musicindex`.

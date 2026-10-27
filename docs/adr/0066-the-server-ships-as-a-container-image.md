# ADR 0066: The Server Ships As A Container Image

- Status: Accepted
- Date: 2026-10-27
- Owner: operator
- Accepted: 2026-10-27 by the operator.

## Context

Task 037 deploys `studio-server` on `studio.musicindex.org`. The [runbook](../runbooks/studio-server.md) builds the binary from source on the server and runs it with a systemd unit. The release workflow `desktop-release.yml` builds only the desktop archives. No ADR records a release decision of Studio.

The host of musicindex.org also runs stophammer. stophammer builds its images on GitHub from a candidate tag, and pushes them to `ghcr.io/inthemorning` (stophammer ADR 0063 and ADR 0065). On the VPS, `deploy.sh --release VERSION` pulls the images and pins the version. The operator decided on 2026-10-27 that Studio uses the same method.

Studio compiles three C libraries: SQLite (`rusqlite` with `bundled`), OpenSSL (`ssh2` with `vendored-openssl`) and LAME (`mp3lame-encoder`). stophammer builds on Alpine with musl.

## Decision

### The Image

1. GitHub builds the image `ghcr.io/inthemorning/studio-server` from the `Dockerfile` at the repository root. The image holds `studio-server` only.
2. The build stage is `rust:alpine` with musl, with the Rust version of `rust-toolchain.toml`. The runtime stage is an Alpine image pinned by its digest, with `ca-certificates`.
3. The image has two CPU types: `amd64` and `arm64`. Each type builds on a native runner. One manifest list holds both types.
4. The image runs as a user with no privileges. The data folder is the volume `/data`. The process writes only to `/data` and `/tmp`.
5. The image has the label `org.opencontainers.image.revision` with the commit of the build, and `org.opencontainers.image.version` with the version.

### The Candidate Tag

6. A tag `v*` with a hyphen, such as `v0.1.0-rc.2`, is a candidate. It builds the image and the desktop archives. The image gets the tag of the version with no `v`, such as `0.1.0-rc.2`.
7. The image job waits for a passed CI run on the commit of the tag, as stophammer ADR 0065 section 4 does. A CI run that does not pass, or no CI run in 30 minutes, stops the job, and no image is pushed.
8. The tag must agree with the version of `Cargo.toml`. A different version stops the job.
9. A tag with no hyphen builds no image in this ADR. The promotion of a candidate to a release waits for a subsequent decision.

### The Deployment

10. On the VPS, a compose file runs the image with the options of the runbook. The container listens on port 8080. The host publishes it only on `127.0.0.1:8080`, so that each request goes through nginx.
11. `tools/deploy-server.sh --release VERSION` deploys a candidate from this computer. It pulls the image on the VPS first, and stops nothing when the pull fails. Then it pins `VERSION` in the `.env` file of the compose folder and starts the new container.
12. After the start, the script checks two facts. The revision label of the running image is the commit of the local tag `vVERSION`, and the start line of the log gives `VERSION`. A difference stops the script with an error.
13. The script makes no account, no invite code and no change in a user folder.

## Alternatives Considered

- Build from source on the server, as the runbook says now. Not chosen: the VPS needs a Rust toolchain, and Studio and stophammer then use two release methods.
- Debian slim with glibc. Not chosen: it has less risk for the C libraries, but the image is larger and it differs from stophammer. If a C library does not build on musl, the implementation stops and reports.
- `amd64` only. Not chosen: a host operator with an ARM server can then not use the image (ADR 0001).
- A manual `docker compose pull` with no script. Not chosen: the script checks the revision of the running image, and a manual pull does not.

## Consequences

- The runbook gets the container method as its main method. The build from source stays as a second method for a host operator with no Docker.
- The nginx server block of `studio.musicindex.org` in `../musicindex/nginx-site` needs `client_max_body_size 9m` (runbook section 3). That change is a commit in the `musicindex` repository.
- A candidate tag also makes a draft GitHub Release with desktop archives. The text of that draft names the server image.
- The distribution plan gets its first release decision.

## Invariants

- No image is pushed from a commit that did not pass CI.
- The host publishes the port of the container only on a loopback address.
- The image holds no secret, no data folder and no signing key.

## Verification

Mechanical:

- The image job builds both CPU types from a candidate tag, and the image starts with `--help` on each type.
- The job stops when the tag and `Cargo.toml` do not agree.
- `tools/deploy-server.sh` refuses a version with no local tag.

Manual, because CI cannot connect to the VPS:

- The deployment of a candidate on `studio.musicindex.org`, with the checks of item 12.
- The human tests of [task 037](../tasks/studio-v1-task-037-runbook-and-human-tests.md).

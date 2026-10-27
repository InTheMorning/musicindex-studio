# Studio v1 Task 043: The Server Image

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), Phase 12. Decision: [ADR 0066](../adr/0066-the-server-ships-as-a-container-image.md).

Status: Not started. The operator accepted ADR 0066 on 2026-10-27. The [review checklist](../reviews/2026-10-27-task-043-review-checklist.md) records each packet.

## Goal

A candidate tag builds the image `ghcr.io/inthemorning/studio-server` for `amd64` and `arm64`. `tools/deploy-server.sh --release VERSION` runs that image on the VPS behind nginx. Task 037 then deploys with it.

## Packets

The packets go in this sequence. Each one depends on the packet before it.

1. [043a: the Dockerfile and the compose file](studio-v1-task-043a-dockerfile.md).
2. [043b: the image job](studio-v1-task-043b-image-job.md).
3. [043c: the deploy script and the runbook](studio-v1-task-043c-deploy-script.md).

The `client_max_body_size 9m` line of nginx is a commit in `../musicindex`. The planning model makes it after packet 043c, because it is one line in a different repository.

## Acceptance Criteria

Mechanical:

- The mechanical criteria of each packet.
- The four commit checks pass.

Manual, by the operator, because CI cannot connect to the VPS:

- A candidate tag builds the image for both CPU types.
- `tools/deploy-server.sh --release VERSION` runs the image on the VPS, and its two checks pass.
- `https://studio.musicindex.org/` shows the sign-in page.

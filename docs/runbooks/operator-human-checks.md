# Operator Human Checks

This runbook gives each open human check of Studio that is not in the notes of a candidate. It also gives the items that only the operator can supply.

This runbook makes no rule. Each check names its owner:

- [ADR 0159](../adr/0159-a-person-can-link-to-a-page-on-music-index.md), and the [notes of 0.1.0-rc.21](../releases/0.1.0-rc.21.md), step 28.
- The [notes of 0.1.0-rc.22](../releases/0.1.0-rc.22.md), step 9, and the [human test findings of 0.1.0-rc.22](../reviews/2026-11-13-rc22-human-test-findings.md), step 11.

When a check passes, record the result in its owner, and remove the block here. The [human check walk of 2026-11-14](../reviews/2026-11-14-human-check-walk.md) gives the results of checks D, H and E item 2, and the comments of that walk.

These blocks hold only the steps that need the operator: the real AppImage, the server and the network.

## State On 2026-11-14

- The server runs `0.1.0-rc.22`. Local `main` has commits after the tag `v0.1.0-rc.22`. The next candidate holds them.
- The agent environment has no GitHub login and no SSH access to GitHub. The operator does each push, each release and each deploy.

## Check E: Pictures From A Linked Page

Owner: ADR 0159 human criterion, and the notes of 0.1.0-rc.21, step 28. The sandbox of the agents cannot load pictures from outside hosts.

Necessary: a person in the desktop app with a read-only link to a Music Index page that is not in the collection. The picture of that page must change during the check. ADR 0182 and ADR 0196 give the button **Add** for a page value.

1. The suggestions of a person show the picture adjacent to **Add**.
2. Link a person to the page of a collaborator. After the collaborator changes a picture, open the person again. The new picture shows with **Add**.

## Checks For The Next Candidate

These checks need a candidate after `0.1.0-rc.22`:

1. Step 11 of the findings of 0.1.0-rc.22: ADR 0175 in the AppImage. The AppImage of `0.1.0-rc.22` does not have commit 8a23157.
2. Step 9 of the notes of 0.1.0-rc.22, on the server: a person with two linked pages shows the host of each page row and the source of each value. In `0.1.0-rc.22` it failed, as expected. Task 138 corrects it on local `main`.

## What Only The Operator Can Supply

| Necessary item | Why | Supply |
| --- | --- | --- |
| A GitHub login, or the operator, for each push and release | The agent environment has no GitHub login and no SSH access. | Each release candidate |
| The network to outside picture hosts and relays | The sandbox of the agents stops these requests. | Check E |
| A sign-in on the server | `GET /api/status` needs a session and its CSRF header, so the agent cannot read the version of the server. | Each check on the server |
| A macOS or Windows computer | Only a tester with those systems can examine their installers. | None. These tests are blocked. |

## Older Open Human Tests

The notes of 0.1.0-rc.21, "Known gaps", list the older open human tests. They are Phase 12, tasks 065 to 084, and the parts of 0.1.0-rc.10 to 0.1.0-rc.20 that did not pass. Their owners give their steps. This runbook does not give them again.

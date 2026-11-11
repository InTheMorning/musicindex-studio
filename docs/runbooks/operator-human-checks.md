# Operator Human Checks

This runbook gives each open human check of Studio as a command block. It also gives the items that only the operator can supply, and the decisions that wait for the operator.

This runbook makes no rule. Each check names its owner:

- The [notes of 0.1.0-rc.21](../releases/0.1.0-rc.21.md), "What to test".
- [ADR 0128](../adr/0128-a-moved-feed-names-its-own-address.md) and [task 091](../tasks/studio-v1-task-091-own-address-tag.md).
- [ADR 0159](../adr/0159-a-person-can-link-to-a-page-on-music-index.md).

When a check passes, record the result in its owner, and remove the block here. The [human test of 0.1.0-rc.21](../reviews/2026-11-10-rc21-human-test-findings.md) gives the results of 2026-11-10.

These blocks hold only the steps that need the operator: the real AppImage, real file hosts and the network.

## State On 2026-11-10

- The draft release `v0.1.0-rc.21` has its 15 assets, and the server runs rc.21.
- Local `main` has commits after the tag `v0.1.0-rc.21`. The next candidate holds them.
- The agent environment has no GitHub login and no SSH access to GitHub. The operator does each push, each release and each deploy.

## Before The Checks

Run these commands one time. They set the variables that the blocks use.

```sh
cd ~/build/musicindex-studio
export STUDIO_REPO="$PWD"
export STUDIO_VERSION=0.1.0-rc.21
```

## Check A: The First Close Of The Window

Owner: the notes of 0.1.0-rc.21, step 22. The tray passed on 2026-11-10. The question at the first close did not show, because the data had the marker of an earlier run.

```sh
# With the AppImage open:
mv ~/.local/share/musicindex-studio/collections/main/tray-notice-shown ~/tray-notice-shown.bak
```

Close the window. Expect one question. Close the window again. Expect no question.

## Check D: A Release Move And A Feed Return Between Two Real File Hosts

Owner: the notes of 0.1.0-rc.21, steps 24 to 26, ADR 0128, human criterion, and task 091. The agent check of 2026-11-08 sent the bytes that Studio writes to a copy of Stophammer at `398d10f`, and each result was correct. This check uses the live index and two real file hosts.

Necessary: a test release on file host A, and a second real file host B in the same collection. Use the AppImage of the release candidate.

```sh
# Put the GUID of the test feed here. The Files page of the feed shows it.
GUID=put-the-feed-guid-here
index() { curl -s "https://api.musicindex.org/v1/feeds/$GUID" | python3 -c 'import json,sys; print(json.load(sys.stdin)["data"]["feed_url"])'; }

index   # Before: the address on host A.
# In Studio: Files, Move This Release..., host B, 7 days. Review, then Write.
index   # After the move: the address on host B.
# In Studio: Files, Move This Release..., host A. Review shows the return warning. Write.
index   # After the return: the address on host A, with no wait.
```

Look at these items, and expect these results:

1. Review of the return shows the warning below the move row. The first move has no warning.
2. Each `index` call after a write gives the new address.
3. The feed and each file play from the host of each step.
4. The old files stay on the old host until the user deletes them. On a later day, **Delete Now…** on the **Files** page removes only the old files.

If an address does not change, wait 10 minutes and run `index` again. Then send the output of each `index` call.

## Check H: A Change From Outside

Owner: the notes of 0.1.0-rc.21, step 27.

1. Change the feed file of a test release on its host, with a tool other than Studio.
2. Open **Releases**. Expect the mark "Changed outside Studio" on the tile, and the red cloud on the row of its artist or label in the left panel.

## Check E: Pictures And The Network

Owner: ADR 0159 human criterion, and the notes of 0.1.0-rc.21, step 28. The sandbox of the agents cannot load pictures from outside hosts.

Result on 2026-11-10: a result row of **Link to an Artist or Label Page** shows its picture. A release with a real Nostr key shows its picture and name.

Open items:

1. The suggestions of a person show the picture adjacent to **Use**.
2. Stop the network. The page of a release stays usable.
3. Link a person to the page of a collaborator. After the collaborator changes a picture, open the person again. The new picture shows.

## What Only The Operator Can Supply

| Necessary item | Why | Supply |
| --- | --- | --- |
| A GitHub login, or the operator, for each push and release | The agent environment has no GitHub login and no SSH access. | Each release candidate |
| Two real file hosts with secrets | A real move uses the vault and the hosts of the operator. | Check D |
| The network to outside picture hosts and relays | The sandbox of the agents stops these requests. | Check E |
| A macOS or Windows computer | Only a tester with those systems can examine their installers. | None. These tests are blocked. |

## Decisions For The Operator

1. **The test feed and the hosts of check D.** Recommendation: use a test release that no listener uses, on two hosts that you can clean. A real feed gets the wait of Stophammer ADR 0083 after each move.

## Older Open Human Tests

The notes of 0.1.0-rc.21, "Known gaps", list the older open human tests. They are Phase 12, tasks 065 to 084, and the parts of 0.1.0-rc.10 to 0.1.0-rc.20 that did not pass. Their owners give their steps. This runbook does not give them again.

# Studio v1 Task 168: The Schema 43 Upgrade With The AppImage

Decision: ADR 0198 (the `feed_writes` table of schema 43). This task is a check of an existing upgrade, so it has no code change and no new decision.

Status: Complete on 2026-11-17 for the walk data. The upgrade of the data of the operator stays a human check.

## Problem

The known gaps of the rc.24 and rc.25 notes say this: no check opened data of schema 42 with the AppImage of schema 43. The unit tests of the upgrade use the Rust code, not the AppImage.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Walk data | reuse | .studio-dev/v-116a-117/data | The script copies it. It is at schema 36. |
| Xvfb in the sandbox | reuse | .studio-dev/walk/README.md | The same Xvfb flags and the own D-Bus. |
| Check script | new | .studio-dev/task-168/t168.sh | Searched .studio-dev/walk and tools: no script starts an AppImage on Xvfb. |

## Method

1. `.studio-dev/task-168/t168.sh` copies the walk data into a temporary home folder and sets `HOME` and the XDG folders to it.
2. It starts Xvfb on `:99` with a session D-Bus. It does not use the display of the operator.
3. It starts the rc.23 AppImage from the public GitHub release, waits 40 seconds, takes a screenshot and stops the process `studio-desktop-`.
4. It does step 3 again with the rc.24 AppImage.
5. After each AppImage, it reads `pragma user_version` and the row counts of `collection.sqlite`.

## Result

| After | Schema | `feed_writes` table | Feeds | People | Drafts |
| --- | --- | --- | --- | --- | --- |
| Start | 36 | no | 58 | 0 | 1 |
| rc.23 | 42 | no | 58 | 0 | 1 |
| rc.24 | 43 | yes | 58 | 0 | 1 |

- The rc.24 AppImage upgraded the data from schema 42 to schema 43. The rows stayed the same.
- The rc.24 screenshot shows Releases with the track counts.
- rc.25 was not started, because its release is a draft and needs a login. It has the same schema 43.
- The upgrade of the data in `~/.local/share/musicindex-studio` stays a human check. The sandbox cannot read that folder.

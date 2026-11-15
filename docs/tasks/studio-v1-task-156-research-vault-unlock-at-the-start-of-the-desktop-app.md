# Studio v1 Task 156: Research, The Vault Unlock At The Start Of The Desktop App

Decision: none. This is Note 2 of the walk. [ADR 0003](../adr/0003-user-unlocked-secret-vault.md) item 2 owns the rule of today: the passphrase of the artist opens the vault. This task changes no rule and no code. Its result gives the options for a decision of the operator.

Status: Complete on 2026-11-15. The [research note](../research/2026-11-15-vault-unlock-at-the-start-of-the-desktop-app.md) is ready for the operator decision.

## Result

- The delegate was `gpt-6-luna`. The first run, with medium effort, did one web read and left question 3 open. The second run, with high effort, gave the note.
- Gate of the main session: met. The main session compared each cited code location with the code. It read the version, the license, the Rust version and the default feature of `keyring` 4.2.0 from crates.io.
- The main session did not read the other web sources. The note names each item that no source gave.

Source: [walk notes of 2026-11-15](../reviews/2026-11-15-walk-notes.md), Note 2.

## Goal

A research document gives the options that let the desktop app open the vault at its start, with no passphrase from the user. For each option, it gives the evidence, the good points, the bad points and the change to ADR 0003 item 2.

## Facts From The Code

- `crates/desktop/src/lib.rs`, `start`, lines 289 to 304: the desktop shell opens the `Workspace`. It gives no passphrase and does no unlock.
- `crates/api/src/workspace.rs`, `unlock_with_passphrase`, lines 1465 to 1472, and `Workspace::lock`, lines 1610 to 1613.
- `crates/core/src/vault.rs`, `open_derived` and `open_checked`, lines 518 to 540: Argon2id makes a wrapping key from the passphrase, and that key opens the random vault key. `OpenVault`, lines 159 to 163, holds the vault key in memory.
- `crates/core/src/vault.rs`, `derived_wrapper`, lines 400 to 420: one wrapper for each unlock method. The vault file can hold more than one wrapper.
- `crates/api/src/workspace.rs`, lines 50 to 55 and 1195 to 1277: the lock time. The desktop starts with "Never" ([ADR 0142](../adr/0142-a-user-selects-the-time-before-the-vault-locks.md) item 2).
- `crates/api/ui/app.js`, `Unlock`, lines 1799 to 1818: the unlock screen.
- No crate has a dependency for a keyring, the Secret Service or a keychain of the operating system. `crates/desktop-app` has `zbus` for the tray only.
- ADR 0003 names the keychain of the operating system as an alternative. The operator rejected it, because the encrypted file can move between computers.

## Questions

1. Which storage locations does each desktop target give for a key: Linux (Secret Service, kernel keyring, TPM), macOS (Keychain) and Windows (Credential Manager, DPAPI)? Which locations work in an AppImage and under the Tauri shell?
2. Can a second wrapper of the vault key use a key from that storage, while the passphrase wrapper stays? The vault file then still moves between computers, and a different computer asks for the passphrase.
3. Which Rust crates give this storage, with their license, their maintenance state and their behavior when no storage service runs?
4. What can a different program of the same user read in each option? Compare this with the lock time "Never" of today.
5. What does the user see when the storage is not available, or when its key does not open the vault?
6. Which setting gives the user the choice, and what is its start value?
7. Does an option apply to the server shell? The expected answer is no. Give the evidence.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Vault wrappers | reuse | crates/core/src/vault.rs | The research examines `derived_wrapper` and `open_checked`. It changes no code. |
| Desktop start | reuse | crates/desktop/src/lib.rs | The research examines `start`. It changes no code. |
| Lock time setting | reuse | crates/api/src/workspace.rs | The research examines the setting. It changes no code. |

## Do Not Touch

- Each file outside `docs/research/`
- `Cargo.toml` and `Cargo.lock`: add no dependency for a test

## Rules For The Delegate

- Read `/home/citizen/.agents/skills/asd-ste100/SKILL.md` before you write. Write the document in Simplified Technical English.
- Do not commit. Leave the document in the working tree and report it.
- Read `AGENTS.md` of `/home/citizen/build/musicindex-research` first, and the Tauri research in `docs/research/2026-11-03-tauri-desktop-shell-research.md`.
- Give the source for each claim: a file with its lines, or a document of the upstream project with its address and the date of the read. Mark each inference as an inference.
- Do not recommend a decision as a fact. Give one recommendation in its own section, with its conditions.
- Write no ADR text. The main session writes the decision record after the operator selects an option.

## Output

One file: `docs/research/2026-11-15-vault-unlock-at-the-start-of-the-desktop-app.md`, with these sections:

1. Summary, at most ten lines.
2. One section for each option: how it works, the evidence, the good points, the bad points, and the items of ADR 0003 and ADR 0142 that it changes.
3. A table of the options against the three desktop targets.
4. The answers to the seven questions.
5. Recommendation, with its conditions.
6. Each item that the research could not find.

## Acceptance Criteria

Mechanical:

1. The file of the section "Output" exists and has the six sections.
2. `git status --short` shows no other changed file.
3. `python3 ~/.agents/skills/asd-ste100/scripts/ste_lint.py --check --no-heuristics` on the file reports no sentence above the word limit.

By the main session, which reads the document. This gate is open until the main session records it:

1. Each claim has a source, and each code location is correct.
2. Each option names its change to ADR 0003 item 2.

## Test Commands

- `git status --short`
- `python3 ~/.agents/skills/asd-ste100/scripts/ste_lint.py --check --no-heuristics docs/research/2026-11-15-vault-unlock-at-the-start-of-the-desktop-app.md`

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- `docs/tasks/studio-v1-task-156-research-vault-unlock-at-the-start-of-the-desktop-app.md` (this file), each section
- `docs/adr/0003-user-unlocked-secret-vault.md`
- `docs/adr/0142-a-user-selects-the-time-before-the-vault-locks.md`
- `docs/research/2026-11-03-tauri-desktop-shell-research.md`
- The code locations of the section "Facts From The Code"

Goal:
- A research document gives the options that let the desktop app open the vault at its start, with no passphrase from the user.

Constraints:
- The sections "Questions", "Rules For The Delegate" and "Output" of this file.

Do not touch:
- Each file outside `docs/research/`
- `Cargo.toml` and `Cargo.lock`

Acceptance criteria:
- Each mechanical criterion of the section "Acceptance Criteria".

Test commands:
- `git status --short`
- `python3 ~/.agents/skills/asd-ste100/scripts/ste_lint.py --check --no-heuristics docs/research/2026-11-15-vault-unlock-at-the-start-of-the-desktop-app.md`

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

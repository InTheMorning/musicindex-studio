# Distribution documentation review, 2026-10-26

## Scope

This review covers the [distribution plan](../plans/distribution-phase-plan.md), the [release runbook](../runbooks/desktop-release.md), and the [start guide](../runbooks/install-desktop.md).
It also checks their claims against the release workflow and the Studio phase plan.

## Result

The documents identify the current desktop archive as an interim download.
They do not describe the unfinished server shell as a production release.
The plan now makes Phase 12 completion a gate for Tauri work, as the Studio phase plan requires.
No public Release or signing credential was created during this review.

## Release gates

1. **Native results are present.** A manual run of the release workflow on `8e820e6` passed on the five native runners.
   After a change to the workflow, the release tool, or the desktop shell, run the manual workflow again.
   Do this before you push the next version tag.
2. **The archives have no installer or signature.** Users must extract an archive and start the binary.
   Complete Phase 12 before work starts on the Tauri installers in Phase 13.
3. **Installation tests are absent.** The archive smoke test checks startup, API access, and vault data after restart.
   Test native installation, update, removal, and data retention when the installers exist.
4. **Server distribution is gated.** The Phase 12 deployment and human tests remain open.
   Keep server packages and OCI images out of production Releases until those checks pass.

## Checks in this worktree

The four checks in `AGENTS.md` passed on Linux x86-64.
The packaged Linux x86-64 archive passed the startup and restart smoke test.
The STE checker found dictionary rule 1.6 terms in the release documents.
Its paragraph-length finding in the plan was corrected. The checker did not report another structural finding.

This review does not verify a Windows, macOS, Linux ARM64, or installed application artifact.

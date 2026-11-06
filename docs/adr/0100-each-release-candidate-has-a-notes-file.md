# ADR 0100: Each Release Candidate Has A Notes File

- Status: Accepted
- Date: 2026-11-01
- Owner: operator
- Accepted: 2026-11-01 by the operator.
- Uses: [ADR 0066](0066-the-server-ships-as-a-container-image.md) for the candidate tag, the server image and `tools/release-candidate.sh`.

## Context

- A candidate tag starts `desktop-release.yml`, which makes a draft GitHub Release with `--generate-notes` and one fixed sentence.
- GitHub makes its notes from the merged pull requests. Studio commits straight to `main`, so the notes of each draft hold only the fixed sentence and a link to the comparison of the commits.
- The server image of a candidate has no notes. The operator and the testers cannot see what a candidate changes, or what to test.
- On 2026-11-01 the operator selected a notes file for each version, written by hand or by an agent, from three options.

## Decision

1. Each candidate version `VERSION` has the file `docs/releases/VERSION.md`, for example `docs/releases/0.1.0-rc.10.md`. It is in the commit of the tag.
2. The file has three sections: "What changed", "What to test" and "Known gaps". It is written for the operator and the testers, not as a list of commits. It is in ASD-STE100 Simplified Technical English.
3. `tools/release-candidate.sh VERSION` stops before its first change when the file is missing, empty, or not committed. For a tag that exists, the file must be in the commit of the tag. The message names this ADR and the path of the file.
4. `desktop-release.yml` stops before the build when the file of the tag is missing. A manual run with no tag does not need a file.
5. The draft GitHub Release gets the file as its notes, then the fixed sentence about the archives and the server image. When a draft exists already, a new run replaces its notes with the same text. A link to a repository document in the file stays relative. The workflow changes it to an address that names the tag, because a relative link gave 404 on the Release page of `0.1.0-rc.19`.
6. The server image of a candidate uses the notes of the GitHub Release of its tag. The image gets no copy.
7. A candidate from before this ADR, `0.1.0-rc.9` and older, needs no file. A file for `0.1.0-rc.9` is permitted, and the operator can put it on that draft by hand.

## Alternatives Considered

- Notes from the commit messages only. Not chosen by the operator: the list has each document commit and each correction, and it does not tell a tester what to test.
- A file `CHANGELOG.md` at the root. Not chosen: one file for each version keeps the notes of a candidate in the commit of its tag. The workflow then reads one file with no parse.
- Pull requests for each change, so that GitHub makes the notes. Not chosen: Studio commits straight to `main`, and this changes the procedure of work.

## Consequences

- Each candidate needs one more step before the tool runs: the notes file, in a commit on `main`.
- The tool and the workflow are the guards of item 3 and item 4. No Rust or Node test examines them, because CI does not run the release tool.
- `docs/README.md` lists `docs/releases/`. The desktop release runbook and the server runbook name the step.

## Non-Goals

- Notes for a release with no hyphen. ADR 0066 item 9 keeps the promotion of a candidate for a subsequent decision.
- A page of notes in the UI of Studio.

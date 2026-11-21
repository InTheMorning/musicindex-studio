# ADR 0236: The Release Notes Are For Testers, And The Operator Checks Are In A Review File

- Status: Accepted
- Date: 2026-11-21
- Owner: operator
- Accepted: 2026-11-21 by the operator, who selected "Notes public, checks in a review file" and "rc.30 now" from drawings in the questionnaire. The choices were two files, one file with a part for the maintainer at the end, and one list with no names.
- Technical choices: the main session selected the section names and the rules of items 3 to 6.
- Bar: opposite choice by mistake. Without this record, an agent writes the notes for the operator, with the names of the operator's feeds and file hosts, as before.
- Changes: [ADR 0100](0100-each-release-candidate-has-a-notes-file.md) item 2, the three sections and the readers of the notes file.

## Context

The notes of `0.1.0-rc.30` had 2,679 words. Almost each line had an ADR number. The steps named the artist page of the operator, the file hosts of the operator and a test album. A tester who is not the operator cannot do these steps, and must not see the names of the operator.

The operator wants other people to test the candidates. The GitHub Release shows the notes file.

## Decision

1. The notes file `docs/releases/VERSION.md` is for testers who are not the operator. It has the sections "What's new", "Try this" and "Known problems".
2. "What's new" has one short item for each change that a user can see. Small fixes of one area are one item. It names the controls with the words of the app.
3. "Try this" has steps that a tester can do with their own data and their own file host. A step does not name a feed, a person, a file host or a release of the operator.
4. "Known problems" has only the problems that a user can see, with a workaround when one exists.
5. The notes file has no ADR number, no task number, no commit, no schema version and no project word that the app does not show. The file gives the meaning of each term from the feed standard where it first occurs.
6. The checks of the operator for a candidate go in the review file `docs/reviews/DATE-rcN-checks.md`. This file has the full list of changes with their ADRs and the steps on the operator's data. It also has the open steps of earlier candidates and the known gaps for the operator. The notes file does not link to it.
7. Items 1, 3 to 7 of ADR 0100 do not change. The notes file stays in ASD-STE100 Simplified Technical English.

## Item Map

| Earlier item | Change by this ADR |
| --- | --- |
| ADR 0100 item 2 | The notes have the sections "What's new", "Try this" and "Known problems", for testers who are not the operator (items 1 to 5). The checks of the operator are in a review file (item 6). |

## Consequences

- The operator reads two files for a candidate: the notes and the review file.
- The notes of `0.1.0-rc.30` change after its tag. The operator puts the new text on the draft GitHub Release by hand. A new run of `desktop-release.yml` for the tag `v0.1.0-rc.30` puts back the text of the tag commit.
- `docs/runbooks/operator-human-checks.md` names the notes of `0.1.0-rc.21` for the older open tests. That file does not change.

## Alternatives Considered

- One file, with a part for the maintainer at the end. Not chosen by the operator: the testers continue to see the names of the operator and the long part.
- One list as before, with general words in place of the names. Not chosen by the operator: the notes stay long and keep the ADR numbers.

## How This Decision Is Checked

Manual, by the main session before each candidate tag: read the notes file for the names of the operator's feeds, persons and file hosts, for ADR and task numbers, and for the sections of item 1. A test cannot tell a name of the operator from a general word. No incident of a missing section has occurred, so no tool examines the section names.

# ADR Consolidation Plan

- Date: 2026-11-14
- Owner: operator
- Rule: global `CLAUDE.md` §Consolidate A Changed Decision. The operator accepted it on 2026-11-14 (musicindex-research `df7b823`).
- Pattern: [ADR 0203](../adr/0203-people.md).

## Goal

On `e3b0f6c`, `check_adr_links.py --counts .` of the `project-baseline` skill gives 11 failures of the limit check. Eleven current ADRs have more than three "Changed by" lines. This plan puts them into areas and gives one consolidated ADR to each area. The last commit removes `--skip limit` from the gate in `AGENTS.md`.

## How The Areas Are Made

Two ADRs go into one area when one changes the other, or when they have a shared changing ADR and a shared subject.

A shared changing ADR alone does not put two ADRs into one area. ADR 0135, ADR 0071, ADR 0073 and ADR 0092 each change ADRs with different subjects. With that rule, nine of the eleven ADRs make one area. That area has approximately 16,000 words and no shared subject.

A link from one area to a different area stays a link. The consolidated ADR cites the current ADR of the other area.

## The Areas

| Area | ADRs | Words | Connection | New ADR |
| --- | --- | --- | --- | --- |
| Feed files, moves and the server | 0038, 0055 | 3,356 | ADR 0105 changes the two ADRs. The two ADRs own where the files of a user are kept. | 0204 |
| Publisher feeds, announcements and grants | 0041, 0052, 0054 | 4,698 | ADR 0054 changes ADR 0041 and ADR 0052. ADR 0135 changes ADR 0041 and ADR 0054. | 0205 |
| The shell and the editor pages | 0039, 0064, 0068 | 4,574 | ADR 0068 changes ADR 0064. ADR 0071, ADR 0073 and ADR 0135 change ADR 0039 and ADR 0068. | 0206 |
| Recipients | 0027, 0091, 0095 | 5,545 | ADR 0095 changes ADR 0091. ADR 0098 and ADR 0099 change all three. All three own the recipients of a feed. | 0207 |

The links that stay between areas:

- ADR 0091 changes ADR 0039 items 1, 5 and 11. ADR 0207 cites ADR 0206 for the destinations.
- ADR 0092 changes ADR 0039 and ADR 0091. It stays current, and its "Changes:" lines point to the archive.
- ADR 0135 and ADR 0071 change ADRs in two areas. ADR 0071 stays current. ADR 0239 consolidated ADR 0135 on 2026-11-21.

## Steps

1. Add the ADR link guard to the gate with `--skip limit`. Done in `3419713`.
2. Write this plan.
3. Write one consolidated ADR for each area, in the sequence of the table. Each one has these parts:
    - The status Accepted, and an "Accepted:" line that cites musicindex-research `df7b823`.
    - One "- Supersedes:" line for each replaced ADR.
    - Each live rule of the replaced ADRs, with each change from a subsequent ADR. It makes no new decision.
    - An item map from each item of a replaced ADR to its new item.
4. In the commit of each area, move the replaced ADRs to `docs/archive/adr/`, and update `docs/README.md`, `AGENTS.md` and each current citation. Code comments keep their citations of the replaced ADRs. The item map resolves them.
5. An independent review examines each consolidated ADR before its commit.
6. The last commit removes `--skip limit` from the gate.

If a restatement needs a change of a rule, the work stops, and the orchestrator gets a report.

## Status

| Area | New ADR | Condition | Commit |
| --- | --- | --- | --- |
| Gate | — | Done | `3419713` |
| Feed files, moves and the server | 0204 | Done | `b9caddc` |
| Publisher feeds, announcements and grants | 0205 | Done | `7fc3ba6` |
| The shell and the editor pages | 0206 | Done | `ae7f363` |
| Recipients | 0207 | Done | `539f328` |
| Limit check in the gate | — | Done | This commit |

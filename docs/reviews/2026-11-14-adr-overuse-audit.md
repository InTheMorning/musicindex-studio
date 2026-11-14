# ADR Overuse Audit Of 2026-11-14

This review is advisory. It states no rule. It gives findings and proposals for the orchestrator. The owner of a rule change is the global `CLAUDE.md` or the `project-baseline` skill.

## Question

The operator asked if the project makes too many ADRs, and if the ADRs add more complexity than they resolve.

## Answer

Yes. The first ADRs record architecture decisions. At this time, about half of the ADRs do the work of a task file or of a findings review. The reading cost of the decision layer is larger than the value of the decisions.

The format of each ADR is good. Each ADR names the choices of the operator and the choices of the planning model, and each ADR names its checks. The defect is the unit (one ADR for each finding) and no consolidation.

## Evidence

All counts come from `grep` and `git log` on `main` at `2430128`.

| Measure | Value |
| --- | --- |
| ADRs in `docs/adr/` | 200, from 2026-10-22 to 2026-11-14 (24 days, about 8 each day, 17 on 2026-11-06) |
| Words in ADRs | 180,321 |
| Words in task files | 266,579 |
| Words in reviews | 50,263 |
| Archived ADRs | 2 (ADR 0101, ADR 0121) |
| ADRs with a `Changes` line to an earlier ADR | 98 |
| ADRs that cite a finding, a human test or a defect | 63 |
| Change links with no back-link in the changed ADR | 35 of 187 (19%) |
| Words in `AGENTS.md` | 11,360, with links to 189 ADRs |

## Findings

### Finding 1: Patches accumulate and no ADR consolidates them

A new ADR changes some items of an earlier ADR, and the earlier ADR stays current. Twelve subsequent ADRs change ADR 0099 (the People destination): 0102, 0103, 0111, 0123, 0138, 0139, 0140, 0144, 0145, 0158, 0182 and 0193. ADR 0116, ADR 0118, ADR 0132 and ADR 0182 change ADR 0111. To find the current rule for the page of a person, an agent must merge about 15 documents item by item. No document states the current People design.

### Finding 2: The back-links are not complete

A back-link is a "Changed by" line in the earlier ADR. It is a project term. Of 187 change links, 35 have no back-link. ADR 0099 does not name ADR 0123, ADR 0138, ADR 0140, ADR 0144 or ADR 0158. A reader of ADR 0099 gets a rule that is not current, and no warning.

### Finding 3: Many ADRs record a defect correction or one UI choice

- ADR 0168 keeps a search field still while the user types. Its own text says that it corrects "the defect of finding 7".
- ADR 0202 shows an `http` image from its `https` address. It is one operator choice on one field.

Neither ADR limits future work, and an agent cannot make the opposite choice by mistake. A line in the task file or the findings review can hold each choice.

### Finding 4: The global archive rule prevents archiving

The global `CLAUDE.md` says: "A decision with an unenforced rule stays current whatever its status." Most of these ADRs have only a visual check or a human check, so no test enforces them. Thus almost no ADR can archive. With one ADR for each finding, the reading path can only grow. The global rule "The Reading Path Does Not Grow With Age" exists to prevent this result.

### Finding 5: `AGENTS.md` keeps history

The global rule says that `AGENTS.md` describes the present and carries no history. The Studio `AGENTS.md` has 11,360 words of history for each task, for example "Task 060 is complete in four packets" and "Its human test is open". The plan and the task files must hold this history.

## Proposals For Global Remediation

These proposals change the global rules or the shared skills. The operator decides them.

1. Set a bar for a new ADR. Write an ADR only for a choice that crosses modules or that is difficult to change. Also write an ADR for a choice that an agent can make incorrectly without the record. Record a defect correction or a choice on one field as an "Operator choice" line. Put that line in the task file or in the findings review.
2. Consolidate each area. When an area has more than a set number of change links (proposal: three), write one current ADR for the area. That ADR restates each live rule and replaces the chain. The chain moves to `docs/archive/adr/`.
3. Change the archive condition. A decision can also archive when a current ADR restates each of its live rules. This keeps the protection against the loss of a live rule, and it removes the chain from the reading path.
4. Reduce `AGENTS.md` to its four questions in each repository. Move the status history to the plan.

## Proposals For Guards

Each guard below has a known incident in this review.

| Guard | Incident | Failure message (draft) |
| --- | --- | --- |
| Each `Changes` link has a matching "Changed by" line in the changed ADR. | Finding 2: 35 links with no back-link. | `Global §Authority Lives In Decision Records: ADR 0140 changes ADR 0099, but ADR 0099 has no "Changed by" line for ADR 0140. Add the line to ADR 0099.` |
| A current ADR has at most a set number of "Changed by" lines (proposal: three). | Finding 1: ADR 0099 has 12. | `Global §The Reading Path Does Not Grow With Age: ADR 0099 has 12 changes. Write one current ADR for this area, restate each live rule, and archive the chain.` |
| `AGENTS.md` has no task status history (proposal: no "Task NNN is complete" text, and a word limit on the status section only). | Finding 5. | `Global §AGENTS.md Describes The Present: AGENTS.md records the status of task 060. Move it to the plan.` |

The global rules do not permit a line limit on `AGENTS.md`. The third guard limits only the status section, or finds history phrases. The orchestrator must select one form.

## Open Points

- Inference: ADR 0001 to about ADR 0060 are mostly architecture decisions. This inference comes from their titles. The audit did not read each ADR.
- The audit examined only Studio. The other repositories can have the same pattern. The orchestrator must count them.
- The People consolidation (proposal 2) is the first local candidate in Studio. Studio waits for the global decision before it starts.

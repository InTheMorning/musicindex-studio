# Studio v1 Task 234: People Values Before A Write

Decision: [ADR 0245](../adr/0245-review-repairs-short-addresses-and-offers-the-people-values.md) items 5, 9, 10, 12 and 14. Run it after task 233.

Status: Done 2026-11-22. The gate passed (2061 Rust tests, 979 Node tests).

Operator choices of the main session:

- The People findings of a review read the credits and the payees of each draft of the review, and then roll back these rows. Thus the findings go after **Apply to the Drafts**, as the checks of ADR 0245 give. After **Discard**, the findings come back. The API test examines the two states.
- `make_person` copies only a correct address from the credits, and `add_person` refuses an address that is not correct (ADR 0245 items 9 and 14). A person made from the credits of "The Doerfels" thus gets `https://www.doerfelverse.com/` and not `www.doerfelverse.com`.
- Studio refuses the save of a person that has an address that is not correct, also when the user did not change that field. Task 235 shows the error and **Add https://** below the field.

## Problem

People saves an information address that is not full. The check of a person runs only on the page of that person. Thus a review does not tell the user that its feeds differ from People.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Person save | extend | crates/core/src/people.rs | `save_person`, `update_fields` and `replace_person_values` save the fields and the values. |
| Web address rule | reuse | crates/core/src/feed/links.rs | `is_web_address` and `short_address_repair` of task 233. |
| Person check | extend | crates/core/src/people.rs | `check` and `credit_findings` give the findings of ADR 0203 item 64 for each feed. |
| Correct operation | reuse | crates/core/src/change_set.rs | `correct_snapshot`, `correct_credit` and the `Correct` operation write the person values into a feed. |
| Save route | extend | crates/api/src/people.rs | `save_person` near line 280 and `person_error` near line 43. |
| Review data | extend | crates/api/src/routes.rs | `get_diff` gives the data of the review of one feed. |

## Changes

1. Read ADR 0245, ADR 0203 items 64 to 73, and ADR 0207 items 17 to 19 and 27 first. Then read `save_person`, `check`, `credit_findings`, `correct_snapshot` and `get_diff`.
2. ADR 0245 item 9: the core save of a person refuses a picture or an information address that is not empty and does not pass `is_web_address`. This covers the fields `img` and `href`, and the values of `pictures` and `websites`. The error names the field. It gives `short_address_repair` of the text when one exists. The save changes nothing on a refusal. The API gives the code `bad_address` with the field and the repair text.
3. ADR 0245 item 10: add a core function that gives the People findings for a list of feeds. It runs `check` for each person with a credit or a payee in those feeds. It keeps the findings of those feeds only, and drops the neutral finding "Not your default address". It makes no request (ADR 0203 item 70).
4. Add one API route that gives the result of step 3 for a list of feed IDs, grouped by person: the person ID, the name, and the feeds with their findings in the form that the check route of a person gives today. The UI of task 235 calls it for the review of one feed and for the review of a change set.
5. ADR 0245 item 12: add one API route that applies the selected corrections of the operation `Correct` of a person to the drafts of a list of feeds directly. It makes no change set, and it starts a draft for a feed with no draft, as a repair does.
   It also works for a feed of an open change set, and then **Discard** of that change set removes the correction, because Discard puts back the draft from before the change set. Reuse `correct_snapshot` and the change of a feed of the `Correct` operation. Do not copy them.
6. Tests:
   - Rust in `crates/core/src/people.rs`: a save with the information address `www.doerfelverse.com` gives the error with the repair text `https://www.doerfelverse.com`, and the person keeps the old value. A save with `https://www.doerfelverse.com/` works.
   - Rust: the findings of step 3 for one feed of a person with two feeds give only that feed, and give no neutral finding.
   - Rust in `crates/api/src/tests.rs`: a move change set, then the route of step 5 for one of its feeds. The draft has the person value. **Discard** of the move then puts back the draft from before the move. The route of step 4 then gives no finding for the corrected feed.

## Constraints

- Do not change a file in `docs/` or `AGENTS.md`.
- Write comments in ASD-STE100 Simplified Technical English: `/home/citizen/.agents/skills/asd-ste100/SKILL.md`. Each new comment cites ADR 0245 and its item. A test failure message names ADR 0245, the item and the function to fix.
- Follow the `rust-dev` skill at `/home/citizen/.agents/skills/rust-dev/SKILL.md` and the lints of each crate.
- Studio applies no People value without a request from the user (ADR 0245 item 14). The route of step 5 runs only on the request of the button.
- The agent does not commit and does not stage. The main session reads the diff, runs the gate and commits.
- If a step needs a design decision that this task does not give, stop and report it.

## Acceptance Criteria

Mechanical:

- The tests of step 6 pass.
- `tools/gate.sh` passes.

## Report

Give the changed files, the new routes with their request and answer, the new tests, the result of `tools/gate.sh` and `git status --short`.

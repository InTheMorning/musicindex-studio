# ADR 0188: The README Is A Landing Page For An Artist

- Status: Accepted
- Date: 2026-11-12
- Owner: operator
- Accepted: 2026-11-12 by the operator, through the questionnaire. The operator selected the quick start with the installer, and no section "Contribute".
- Technical choices: items 4, 7 and 8 are choices of the planning model.
- Changes: the rule in `AGENTS.md` that `README.md` holds only links to `docs/user-guide/`. That rule named the global documentation rules as its owner, but no global rule has that text.
- Source: P1 of the [research note on the README](../plans/research-readme-pilot-request.md), P1. It applies the README contract of the research repository.

## Context

On 2026-11-12, `README.md` had one sentence and 26 links to the user guide. A visitor on the GitHub page did not see what Studio looks like, or how to start it. The README had no link to `docs/user-guide/feed-check.md`. The guide index `docs/user-guide/README.md` had that link.

The research repository proposes one README contract for each repository of the suite. In less than one minute, a visitor must find three answers. What does the project do? Is it for the visitor? How can the visitor try it?

## Decision

1. The primary reader of `README.md` is an artist or a label that publishes music feeds. A different reader, for example a host operator or a developer, gets one link to the correct document.
2. `README.md` has nine sections, in this sequence: the name and the promise, the image and who it is for. Then the quick start, what it does today and the documentation. Then the suite, the status and the license. It has no section "Contribute" until the repository has a file with contribution rules.
3. The quick start uses the installer of the latest GitHub Release. It has five steps or fewer, and the last step shows a result. One link gives the steps from the source code in `docs/user-guide/start-on-desktop.md`.
4. The image is one screenshot of the release page of the current version, with test data only. The file is `docs/user-guide/images/release-page.png`, in the repository.
5. The section "Documentation" gives the link to `docs/user-guide/README.md` first. Then it gives a short group of links for the needs of the reader: learn, do a task, look up and understand. Only the guide index lists each page of the guide.
6. The architecture, the routes, the ADRs, the steps that compile the code and the repository layout stay out of `README.md`. One link to `docs/README.md` gives them to a developer.
7. A Node test in `crates/api/ui-tests/` enforces two rules. The message of each test error names this ADR and the fix:
    - Each relative link in `README.md` and in `docs/user-guide/` points to a file that exists.
    - `docs/user-guide/README.md` has a link to each page in `docs/user-guide/`.
8. Studio does not use two checks of the contract as tests:
    - A retired name in the current text. Studio has no record of an error that this check finds.
    - The STE checker. The document hook and the manual run of `ste_lint.py` stay. The checker needs the local dictionary, which CI does not have.
9. The quick start has no test, because it needs an installer and a window. The manual checks are the agent check of a subagent with no project context and the human check on a clean computer.

## Alternatives Considered

- The quick start from the source code. Not chosen by the operator: an artist must install Rust first.
- The quick start with a sign-in on the server. Not chosen by the operator.
- A section "Contribute" with a link to the issues. Not chosen by the operator.
- Keep the full list of guide pages in `README.md`. Not chosen: the guide index has the same list, and two lists can be different, as `feed-check.md` showed.

## Consequences

- A change of the release page can make the image incorrect. The visual check of the README examines the image.
- The guide index is the one list of the guide pages, and the test enforces it.

## Acceptance Criteria

Mechanical:

- The Node test of item 7 passes.
- The test of the guide index fails when a page link is removed from `docs/user-guide/README.md`. A run with one link removed proves it.

Agent:

- An agent renders `README.md` at desktop width and at phone width, and examines the image and the tables.
- A subagent with no project context reads `README.md` and does the quick start. The task records each step where it stopped.

Human, by the operator. These gates stay open until the operator does them:

- The README tells a new artist what Studio does in less than one minute.
- The screenshot is easy to read on a phone.
- The quick start works on a clean computer.

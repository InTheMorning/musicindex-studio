# Research Request: The README As A Landing Page

## Status

Open - 2026-11-12. [ADR 0188](../adr/0188-the-readme-is-a-landing-page-for-an-artist.md) proposes P1. Received - 2026-11-12.

This request comes from the research repository at
`../musicindex-research`, commit `ae359aa`. That repository has no Git remote,
and it is private. The request is
`research_notes/user-docs/studio-pilot-request.md`, and the contract is
`research_notes/user-docs/readme-contract.md` in the same folder.

This document binds nothing in Studio. Studio records its decision in
[ADR 0188](../adr/0188-the-readme-is-a-landing-page-for-an-artist.md).

The research examined Studio at commit `ba8efbd`.

## Purpose

Studio is the first repository to apply the README contract (`readme-contract.md` in the research repository).
Its primary reader is an artist or a label that publishes music feeds.
Studio has a complete user guide, so the work is a landing page in front of that guide.

## Findings At Studio `ba8efbd`

- `README.md` has 32 lines: one sentence and 26 guide links.
- The README has no link to `docs/user-guide/feed-check.md`. The guide index `docs/user-guide/README.md` groups the pages by task.
- No page of the user guide has an image.
- `docs/user-guide/start-on-desktop.md` gives the start steps for an archive and for the source code.

## Requested Work

| # | Work | Check |
| --- | --- | --- |
| P1 | Write an ADR that adopts the contract for Studio, with the artist as the primary reader. | The operator accepts the ADR. |
| P2 | Write the new `README.md` in the sequence of the contract. Replace the 26 links with a short group of links and one link to `docs/user-guide/README.md`. | Mechanical: the guards of P4 pass. |
| P3 | Add one screenshot of the release page, made from the current build. Keep the image in the repository. | Agent: render the README at desktop width and at phone width. |
| P4 | Add two guards. The first guard makes sure that each relative link in `README.md` and `docs/user-guide/` points to a file that exists. The second guard makes sure that the guide index has a link to each guide page. Each failure message names the ADR of P1 and the fix. | Mechanical: the tests fail on the `feed-check.md` defect before the fix. |
| P5 | Give the README to a subagent with no project context. It follows the quick start and reports each place where it stopped. | Agent: record the report in the task. |

Use a smaller model for P5. Do not use the subagent for P1.

## Visual Checks For The Operator

These checks stay open until the operator does them:

- The README tells a new artist what Studio does in less than one minute.
- The screenshot is easy to read on a phone.

## Send Back

- The ADR number and the commit of each packet.
- The report of the subagent from P5.
- Each part of the contract that was not correct for Studio. The research updates the contract from these results before the next repository.

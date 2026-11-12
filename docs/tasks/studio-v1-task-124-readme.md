# Studio v1 Task 124: The README Is A Landing Page

Decision: [ADR 0188](../adr/0188-the-readme-is-a-landing-page-for-an-artist.md).
Source: P2 to P5 of the [research request on the README](../plans/research-readme-pilot-request.md).

Status: Complete on 2026-11-12. The mechanical checks and the agent checks passed. The human gates are open.

## Goal

In less than one minute, a new artist on the GitHub page of Studio finds what Studio does. The artist also finds if Studio is for the artist, and how to try it.

## Constraints

- The image uses test data only. It shows no person, artist, address or key from outside the test data.
- Only `docs/user-guide/README.md` lists each guide page.
- The prose is in STE. The button names and the page titles keep their text.

## Work

| # | Work | Check |
| --- | --- | --- |
| P2 | Write `README.md` in the sequence of ADR 0188 item 2. | Mechanical: the Node test `docs-links.test.mjs` passes. |
| P3 | Add the screenshot `docs/user-guide/images/release-page.png` of the release page, from the current version. | Agent: render `README.md` at desktop width and at phone width. |
| P4 | Add the Node test `crates/api/ui-tests/docs-links.test.mjs` of ADR 0188 item 7. | Mechanical: the test fails when one link is removed from the guide index. |
| P5 | Give `README.md` to a subagent with no project context. It does the quick start and reports each step where it stopped. | Agent: this task records the report. |

## Checks

Mechanical:

- `docs-links.test.mjs` passes.
- On 2026-11-12, the test gave an error in a run with no link to `feed-check.md` in `docs/user-guide/README.md`. The message named ADR 0188 item 7 and `feed-check.md`.

Agent:

- The render at desktop width and at phone width. At 1280 px the page is correct. At 390 px the image scales to approximately 360 px, and its text is too small to read. The phone gate of the operator stays open.
- The report of P5, on 2026-11-12. A reader with no project context did the quick start to the download. The README got these corrections:
  - The newest public release was `0.1.0-rc.19`, because `0.1.0-rc.21` is a draft. After the correction, the README names no version.
  - The reader did not know which Linux file to use, or how to open an AppImage. Step 1 and step 2 tell this after the correction.
  - Step 3 had no description of its page. After the correction, it links to the first-run page.
  - No text told that `~/musicindex/pages` is not on the web. After the correction, a note after the steps tells this.
- The reader also asked these questions. The guide pages answer them, and the README does not. The questions are: which file host to use, if Studio needs a passphrase, and what the announcement does. The reader also asked which audio files Studio accepts, and where the data folder is. The install page has no Linux steps for the first start.
- The reader compared the SHA-256 of the AppImage of `0.1.0-rc.19` with `SHA256SUMS.txt`, and the values agreed.

Human, by the operator. These gates stay open until the operator does them:

- The README tells a new artist what Studio does in less than one minute.
- The screenshot is easy to read on a phone.
- The quick start works on a clean computer.

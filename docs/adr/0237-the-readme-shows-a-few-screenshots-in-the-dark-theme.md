# ADR 0237: The README Shows A Few Screenshots In The Dark Theme

- Status: Accepted
- Date: 2026-11-21
- Owner: operator
- Accepted: 2026-11-21 by the operator, who asked for "some screenshots in the readme (dark mode preferred)" and a minimal quick start.
- Technical choices: the main session selected the four pages, the file names and the layout of item 3.
- Bar: opposite choice by mistake. ADR 0188 item 4 permits one image only. Without this record, an agent removes the other images.
- Changes: [ADR 0188](0188-the-readme-is-a-landing-page-for-an-artist.md) item 4.

## Context

On 2026-11-21, `README.md` had one screenshot of the release page in the light theme, from 2026-11-12. The release page changed after that date: the first tab is "Release", and the bar has undo and redo. The image did not show the artist page, the review of a change or the value routes.

## Decision

1. `README.md` has one to four screenshots of the current version. Each screenshot uses test data only, with invented names, addresses and keys.
2. Each screenshot uses the dark theme, at 1280 × 800.
3. The first screenshot is the release page, below the promise. The other screenshots are in one section "See it", after the quick start "Try it". The quick start thus stays near the top of the page. Each has a caption of one line.
4. The files are in `docs/user-guide/images/`. The release page stays `release-page.png`.
5. The images of the current set are the release page, the artist page, the review of a change and the value routes. A change of this set is a change of `README.md` only.
6. Items 1 to 3 and 5 to 9 of ADR 0188 do not change. The section sequence of ADR 0188 item 2 gets "See it" after the quick start.

## Alternatives Considered

- Keep one image. Not chosen: the operator asked for screenshots.
- The light theme. Not chosen: the operator prefers the dark theme.
- One image for each theme. Not chosen: it doubles the images, and the operator selected one theme.

## Consequences

- A change of one of the four pages can make an image incorrect. The visual check of the README examines each image.
- The test data and the drive steps are in `.studio-dev/readme-shots/`, which Git ignores. An agent can make the same data again from its script `mk.py`.

## Acceptance Criteria

Mechanical:

- The Node test of ADR 0188 item 7 passes. It refuses a broken image link.

Agent:

- An agent renders `README.md` at desktop width and at phone width, and examines each image and its caption.

Human, by the operator:

- The screenshots are easy to read on a phone.

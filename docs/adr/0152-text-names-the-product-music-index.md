# ADR 0152: Text Names The Product "Music Index"

- Status: Accepted
- Date: 2026-11-07
- Owner: operator
- Accepted: 2026-11-07 by the operator, through the orchestrator session. The operator decided that text that a person reads says "Music Index", also in the place of the lower-case form.
- Uses: musicindex ADR 0023 R-WORD-8.

## Context

Studio gave the product name as "musicindex" in its buttons, sheets, notes and errors, for example "Find on musicindex".

## Decision

1. Studio follows musicindex ADR 0023 R-WORD-8. Each text that a person reads gives the product name as "Music Index": the UI, the error messages of the API, the user guide and the release notes of the next candidate.
2. A code name keeps its form: a crate, a module, a type, a route, a file, a field, a command option, a domain and a URL. Examples are `musicindex.org`, `--musicindex`, `/musicindex/search` and the type `MusicIndex`.
3. A label that a current record quotes uses the new form. A record that tells what a past version said keeps its words, and so does a quote of a different record.
4. The name of the repository `../musicindex` and its records, for example "musicindex ADR 0023", keep their form, because they name the repository.

## How This Decision Is Checked

Mechanical: the title case test of `crates/api/ui-tests/casing.test.mjs` refuses the lower-case word in a button label, because the list of fixed names no longer holds it.

Visual: the planning model looks at the changed buttons, headings and notes at 1280 px, and at 390 px with touch.

Visual result: passed on 2026-11-08 ([visual checks of 2026-11-08](../reviews/2026-11-08-visual-checks.md)).

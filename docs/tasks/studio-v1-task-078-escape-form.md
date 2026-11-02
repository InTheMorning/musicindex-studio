# Studio v1 Task 078: A Changed Value Keeps Its Escape Form

Plan: [Studio v1 phase plan](../plans/studio-v1-phase-plan.md), "The Review Of A Write".
Decision: [ADR 0115](../adr/0115-a-changed-value-keeps-the-escape-form-of-its-source.md). It uses [ADR 0012](../adr/0012-lossless-tree-with-a-typed-model.md) and [ADR 0114](../adr/0114-the-review-shows-the-changes-in-words-and-marks-the-changed-part-of-a-line.md).

Status: Not started.

## Goal

A change of a value changes only the characters that the user changed. An apostrophe, a double quote in text and a `>` keep the form of the source value: the entity or the literal character.

## Packets

One packet, 078a, in the core. The planning model examines it before its commit.

Visual check, by the planning model, before the commit: examine the review of a changed title with an apostrophe, at 1100 px. Only the changed words have marks. The UI does not change, so one width and one theme are sufficient.

Human test, by the operator, after the deploy:

1. Change a title with an apostrophe in a feed from MSP 2.0.
2. Do the same in a feed with literal apostrophes.
3. In each review, only the changed words have marks.

## Words

- The **form** of a character in a source value is `entity` or `literal`. A named entity (`&apos;`, `&quot;`, `&gt;`) and a character reference (`&#39;`, `&#x27;`) are the entity form (ADR 0115 item 5).
- The **kept characters** are `'`, `"` in text, and `>` (ADR 0115 item 2). The writer always escapes `&` and `<`, and `"` in an attribute value (ADR 0115 item 1).

## Packet 078a: The Escape Form

### Files

Inspect: `crates/core/src/xml.rs` (`Text`, `Text::new`, `Text::new_cdata`, `Attribute`, `Element::set_attribute`, `Element::set_text`, `write_new_start`, the parser near `tag.attributes()`, the tests), `~/.cargo/registry/src/*/quick-xml-0.42*/src/escape.rs` (`escape`, `partial_escape`, `minimal_escape`), each caller of `Text::new` (`grep -rn "Text::new" crates/core/src`), `crates/core/tests/corpus.rs`.

Change: `crates/core/src/xml.rs`, the callers of `Text::new` only where a source text is at hand, the core tests.

Do not change: the UI, the API, the ADRs, `crates/server`, the bytes of a read and a write with no edit.

### Architecture

These choices are fixed. Do not change them.

1. `xml.rs` gets a private `EscapeForm` with one field for each kept character: `apos`, `quot` and `gt`. Each field is `Option<Form>`, where `Form` is `Entity` or `Literal`, and `None` means that the source has no form for that character.
2. `EscapeForm::of_raw(raw: &str)` reads the form from the source text. For each kept character, the first occurrence of the character or of its entity form gives the form.
3. One private function `escape_with(value: &str, form: &EscapeForm, attribute: bool) -> Cow<str>` does each escape of the writer. It always writes `&amp;` and `&lt;`. In an attribute, it always writes `&quot;`. For each other kept character, it writes the entity for `Entity` or `None`, and the literal character for `Literal` (ADR 0115 items 2 and 4).
4. `Attribute` keeps the `EscapeForm` of its source text, from the raw attribute value of the parser. A new attribute and a changed attribute keep the form that the attribute had before, or `EscapeForm::default()` for a new attribute. `write_new_start` uses `escape_with` with the form of each attribute.
5. `Element::set_text` reads the `EscapeForm` of the text that it replaces, from the raw text of the source children, and makes the new text with that form. A CDATA text does not change.
6. `Text::new(value)` keeps its behavior: it uses `EscapeForm::default()`. A new function `Text::with_form` takes the form. A caller with a source text uses `set_text` or `Text::with_form`.
7. `Attribute` derives `PartialEq` with its form. When a test compares two documents and fails only because of the form, the test compares the written bytes.

### Acceptance Criteria

Mechanical:

- A core test: a changed title with the source `HeyCitizen's` writes `'`, and a source `HeyCitizen&apos;s` writes `&apos;`.
- A core test: a change of the `split` of a recipient keeps `name="Joe's"` as it is, and keeps `name="Joe&apos;s"` as it is.
- A core test: a new element writes `&apos;`, and a `"` in an attribute value writes `&quot;`, also when the source attribute was in single quotes.
- A core test: a source `&#39;` gives the entity form, and a source `>` in text keeps the literal `>`.
- The corpus tests pass with no change.
- The four commit checks pass.

### Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/adr/0115-a-changed-value-keeps-the-escape-form-of-its-source.md
- docs/tasks/studio-v1-task-078-escape-form.md (the Goal, the Words section and packet 078a)
- ~/.agents/skills/asd-ste100/SKILL.md
- The files that the Files section of packet 078a names.

Goal:
- A changed value keeps the escape form of its source for `'`, `"` in text and `>`.

Constraints:
- Follow the Architecture section of packet 078a fully.
- A read and a write with no edit give the same bytes. The corpus tests pass with no change.
- Each comment and each test message is in ASD-STE100 Simplified Technical English.
- Follow the Pragmatic Rust Guidelines of the `rust-dev` skill.

Do not touch:
- The UI, the API, the ADRs, `crates/server`.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of packet 078a.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
- tools/node --test crates/api/ui-tests/*.test.mjs

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

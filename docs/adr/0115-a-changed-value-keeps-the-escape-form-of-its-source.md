# ADR 0115: A Changed Value Keeps The Escape Form Of Its Source

- Status: Accepted
- Date: 2026-11-02
- Owner: operator
- Accepted: 2026-11-02 by the operator. The operator asked the planning model to collect the facts and to make the correct choice. This record gives the facts and the choice.
- Uses: [ADR 0008](0008-feed-types-and-feed-identity.md) and [ADR 0012](0012-lossless-tree-with-a-typed-model.md) for the bytes that the writer keeps. [ADR 0114](0114-the-review-shows-the-changes-in-words-and-marks-the-changed-part-of-a-line.md) for the marks of the review.

## Context

- The visual check of task 077b on 2026-11-02 changed the title `HeyCitizen's Lo-Fi Beats`. The writer wrote `HeyCitizen&apos;s Lo-Fi Hip-Hop Beats`. The user did not change the apostrophe, but the review marked it.
- The writer escapes each changed value with `quick_xml::escape::escape` (`crates/core/src/xml.rs`). That function writes `&apos;` for `'`, `&quot;` for `"` and `&gt;` for `>`. A change of one attribute writes the full start tag again, so each other attribute of the element is escaped again too.
- XML 1.0, section 2.4 (https://www.w3.org/TR/xml/): the characters `&` and `<` "MUST NOT appear in their literal form". The character `>` must be escaped only in the string `]]>`. The apostrophe "may be represented as `&apos;`", and the double quote as `&quot;`, so that an attribute value can contain the two quote characters. A `"` in an attribute value in double quotes must be escaped.
- Each reader of Studio decodes the two forms. quick-xml resolves `apos` (`src/escape.rs`). Stophammer reads with roxmltree, and `stophammer-parser/src/html.rs` decodes `&apos;`. HTML5 defines `&apos;`. HTML 4 does not, and XHTML 1.0 Appendix C.16 tells authors to use `&#39;` for HTML 4 user agents.
- Apple Podcasts, "Podcast RSS feed technical requirements" (https://podcasters.apple.com/support/823-podcast-requirements): "Use &apos; (for single quote) or &quot; (for double quote)." and "Avoid using HTML naming conventions such as &rsquo;, &lsquo;, &ldquo;, or &rdquo;." The table of the page gives `&apos;` for the curly apostrophe `’`, but `&apos;` is the straight apostrophe U+0027.
- The published feeds of the corpus use all three forms. MSP 2.0 writes `&apos;` (`../MSP-2.0/src/utils/xmlGenerator.ts` line 15). Fountain and gfr write `'`. Sovereign Feeds and The Split Kit write `&#39;`. The research found no report of a reader that fails with one of the forms.
- The writer already keeps the form of the source for CDATA: a new text uses a CDATA section when the source text used one (`Element::set_text`).
- Studio replaces MSP 2.0, so many feeds of its users use `&apos;`. A rule that always writes `'` changes those feeds in the opposite direction.

## Decision

1. The writer escapes `&` and `<` in each value. In an attribute value, it also escapes `"`, because the writer puts each attribute value in double quotes.
2. For `'`, for `"` in text, and for `>`, a changed value keeps the form of the value that it replaces. When the source value has the entity (`&apos;`, `&quot;` or `&gt;`), the writer uses the entity. When the source value has the literal character, the writer writes the literal character. The writer examines each character separately.
3. The source value of a text is the source text of the element. The source value of an attribute is the source text of that attribute, also when a different attribute of the element changes.
4. A value with no source form for a character uses the entity. This is the output of the writer before this decision, and it is the form that Apple Podcasts recommends. A value with no source form is a new element, a new attribute, or a source value that does not have the character.
5. A character reference such as `&#39;` in the source counts as the entity form.

## Alternatives Considered

- Always the literal character, with only the escapes that XML requires. Not chosen: each changed value of a feed from MSP 2.0 would then change its apostrophes, the same fault in the other direction.
- Always the entity, as now. Not chosen: each changed value of a feed with literal apostrophes changes characters that the user did not change, and the review marks them.
- The form that most of the file uses. Not chosen: the form of the value itself is a nearer source, and it needs no count of the file.

## Consequences

- A change of a value changes only the characters that the user changed. The review of ADR 0114 marks no apostrophe that the user did not change.
- `Attribute` keeps the form of its source text. `Text::new` gets the form of the text that it replaces.
- The corpus tests do not change, because a read and a write with no edit keep the bytes.
- Task 078 gives the work.

## Acceptance Criteria

Mechanical:

- A core test: a changed title with the source `HeyCitizen's` writes `'`, and a source `HeyCitizen&apos;s` writes `&apos;`.
- A core test: a change of the `split` of a recipient keeps `name="Joe's"` as it is.
- A core test: a new element writes `&apos;`, and a `"` in an attribute value writes `&quot;`.
- A core test: a source `&#39;` gives the entity form.
- The corpus tests pass with no change.
- The four commit checks pass.

Visual: the review of a changed title with an apostrophe marks only the changed words.

Human: the operator changes a title with an apostrophe in a feed from MSP 2.0, and in a feed with literal apostrophes. In each review, only the changed words have marks.

# ADR 0114: The Review Shows The Changes In Words And Marks The Changed Part Of A Line

- Status: Accepted
- Date: 2026-11-02
- Owner: operator
- Accepted: 2026-11-02 by the operator, from the mockups in `.studio-dev/mockup-diff/site/`.
- Uses: [ADR 0007](0007-draft-diff-write-and-announce.md) for the diff that the user approves. [ADR 0012](0012-lossless-tree-with-a-typed-model.md) for the typed model. [ADR 0010](0010-studio-uses-the-afterglow-direction.md) for a sign that is not only a color. [ADR 0065](0065-the-core-applies-each-feed-rule.md) for the core as the owner of each rule.

## Context

- The review of a draft and the review of a change set show a unified diff of the XML. Each changed line shows as a removed line and an added line.
- In the test of 2026-11-02, a correction added only one attribute, `npub="…"`, to six credits. The diff showed twelve long lines, and on a phone each line wrapped to four or five rows. The operator could not see the change. The key was the key of a different account, and the operator did not see that either.
- Code review tools solve this with an intraline diff. After the line diff, they compare each removed line with its added line a second time, and mark only the changed part. The Rust crate `similar` 3.2, which the core uses for the diff, has this as `TextDiff::iter_inline_changes` with `InlineChangeOptions::semantic_cleanup`, behind its feature `inline`.
- The operator proposed a simplified view in YAML. XML has no single YAML form. Each converter invents marks for attributes and text, and repeated elements lose their position. A change of escaping, of attribute position or of whitespace shows no difference. The operator accepted a field view in the words of the editor in its place.
- The operator accepted the four screens of `.studio-dev/mockup-diff/site/` on 2026-11-02.

## Decision

### The Field View

1. The core compares the typed model of the live file with the model of the draft. It gives a list of field changes, in groups. A group names its element, for example "Title", "Credits · HeyCitizen" or "Track 3". A change gives the field, the old value and the new value, for example "musician · Nostr key: none → City Radio (npub12um…f5ck)".
2. A group with more than two changes of the same kind gives one summary line, for example "6 credits get the Nostr key City Radio". It shows two changes, and the others in one collapsed line.
3. The UI shows the profile name of a key and the warning of [ADR 0103](0103-one-person-collects-the-items-of-many-suggestions.md) item 18 in a change of a Nostr key.
4. The core marks each changed line of the XML that a field change explains. When changed lines are left over, the field view ends with this row:
    - "1 more change in the XML" or "N more changes in the XML"
    - "This list does not show it. Select the detailed view to see each changed line." (for one change), or "This list does not show them. Select the detailed view to see each changed line." (for more than one)
5. The field view and that row together cover each changed line. A test enforces this.

### The Lines Of The Diff

6. The core turns on the feature `inline` of `similar`. The diff route gives each line of the file with its kind: the same, removed or added. A removed or added line gives its parts, and each changed part is marked. The core uses semantic cleanup, so that a part is a readable piece, not single characters.
7. The diff route keeps `unified`.

### The Two Views

8. Each feed in a review has the select menu "View", with "Simplified" and "Detailed". "Simplified" is the default. The device remembers the last choice of the user. This is a convenience of one device, not a setting of the account.
9. "Simplified" shows "What changes", which is the field view, then "Changed lines". A removed line and its added line can show as one line with the sign `±`. This occurs when the line has only added parts, or only removed parts. An added part has the add color. A removed part has the remove color and a line through it. Each other change shows as in "Detailed".
10. "Detailed" shows each removed line with the sign `−` and each added line with the sign `+`. The changed parts have the same marks as in item 9.
11. Each view shows three lines of context around each change. A run of hidden lines shows as "Show N more lines", which opens that run. The button "Show the Whole File" shows each line of the file. Then the button reads "Show Only the Changes".
12. At 760 px or less, the diff removes the indent that all the shown lines share. Each line keeps its indent relative to the other lines. The file does not change.
13. The two reviews use the same view: the review of a draft and the review of a change set.

## Alternatives Considered

- A simplified view in YAML. Not chosen by the operator: there is no single YAML form of XML, the converter marks are a syntax to learn, and some byte changes show no difference. ADR 0007 and ADR 0012 need the user to approve the real bytes.
- Two columns, the old file and the new file. Not chosen: a phone has no width for two columns of XML.
- A structural diff of the XML tree, as in difftastic. Not chosen: the field view gives the meaning, and the intraline marks give the exact bytes, with much less work.
- A field view with no diff below it. Not chosen: ADR 0007 needs a diff that the user can examine.

## Consequences

- The core gets the field changes, the cover of the changed lines, and the lines with their parts. The diff route gets their fields.
- The UI gets the view menu, the field view, the guard row, the marked lines, "Show N more lines" and "Show the Whole File".
- The field view is a description of the core. The UI has no copy of a feed rule (ADR 0065).
- A task file gives the work.

## Acceptance Criteria

Mechanical:

- A core test: a draft that adds `npub` to six credits gives one group with six changes, and no guard row. Each changed line is covered.
- A core test: a change that the model does not hold, for example a new namespace declaration, gives the guard row with its count.
- A core test: the line parts mark only the added ` npub="…"` of each credit, and the parts of a changed title mark only the changed words.
- A Node test: the one-line form of item 9 for a line with only added parts. The two-line form for a line with a removed and an added part.
- A Node test: the runs of hidden lines, the text "Show N more lines", and the common indent of item 12.
- The four commit checks pass.

Visual: the four screens of the mockups, at 1100 px and at 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human: the operator makes a correction of the Nostr key of a person, and sees the key and its warning in "What changes" before the write.

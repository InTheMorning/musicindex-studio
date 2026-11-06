# ADR 0133: The Share Bar Shows The Divisions Of A Payment

- Status: Accepted
- Date: 2026-11-06
- Owner: operator
- Source: [findings of local use, finding 10](../reviews/2026-11-06-local-use-findings.md)

## Context

The share bar above a list of recipients named at most three parts. All other parts folded into "Other". With six equal recipients, "Other" held 50 %, and the bar hid three divisions that it could show.

No other app of the suite has this bar, so no shared rule owns it.

## Decision

1. The part "Other" of the share bar holds at most 10 %.
2. A part of 5 % or more does not fold into "Other".
3. "Other" with one part does not occur. That part keeps its name.
4. The bar shows at most 20 parts and "Other", so that each part keeps a visible width. With more parts, "Other" can hold more than 10 %.
5. The legend names the three largest parts, in the sequence of the bar. It then gives the number of the other recipients and the sum of their shares. The list of recipients names each one.
6. This rule is a display rule. It changes no split and no feed.

## Alternatives Considered

- A cap with no limit of parts. Not chosen: 100 parts of 1 % have no visible width in the bar.
- A legend that names each part. Not chosen: the full list of recipients is directly below the bar on the page, and the legend also shows in a summary line.

## How This Decision Is Checked

Mechanical: `crates/api/ui-tests/recipients.test.mjs` examines `shareParts`.

Visual: the planning model looks at the share bar on the Value routes page at 1100 px and at 390 px.

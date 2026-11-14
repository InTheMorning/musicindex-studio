# ADR 0199: A Linked Page Row Names Its Host, And Add A Person Keeps The Source Of Its Values

- Status: Accepted
- Date: 2026-11-14
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-14 by the operator, through the questionnaire about the findings of the human test of 0.1.0-rc.22. The operator selected "Host on its own line" for finding 7.
- Technical choices: the planning model selected the source of the host in item 2, the text of item 3, and items 4 to 7. Finding 8 is a defect against [ADR 0196](0196-a-person-keeps-a-list-of-pictures-and-websites-with-a-default.md) item 9, so it needs no new choice of the operator.
- Uses: [ADR 0196](0196-a-person-keeps-a-list-of-pictures-and-websites-with-a-default.md) items 1 and 9, [ADR 0192](0192-add-a-person-has-the-layout-of-the-other-pickers.md) item 3, and the rule of one labeled line for each kind of value in a row.
- Source: finding 7 and finding 8 of the [human test findings of 0.1.0-rc.22](../reviews/2026-11-13-rc22-human-test-findings.md).

## Context

The page of a person shows a row for each linked page (`crates/api/ui/peoplepages.js`, about line 1086). The row gives the page title and "Read only". When two pages have the same title, the user cannot see which page is on which site. The saved link `PersonPage` (`crates/core/src/people.rs`) holds only `feed_guid` and `title`.

The page of a person reads `GET /people/{id}/page-suggestions` each time it opens (`peoplepages.js`, about line 862). Each answer has the `host` of the feed address of the page, from `page_suggestions` in the core.

**Add a Person** makes a person from a live page on Music Index (ADR 0192). The picture and the website come from that page. `addPersonBody` (`crates/api/ui/people.js`) sends only `name`, `img` and `href`. `insert_person` (`crates/core/src/people.rs`) then calls `add_person_value` with no source. Thus each value shows `Added in Studio`, not `From "{title}" on {host}` (ADR 0196 item 9).

## Decision

### The Host Of A Linked Page

1. Each linked page row has a second line below the title: `On {host}`. "Read only" stays on the first line.
2. The host comes from the page suggestions that the page already reads. The UI matches a row and a suggestion by `feed_guid`. Only the core gets a host from a feed address.
3. When no suggestion gives a host, the second line is `Host not known`. Examples are a page that Music Index does not hold, and a page with no network.
4. A row of a feed in the collection keeps its current lines. Item 1 applies only to the linked pages that are not in the collection.

### The Source Of Add A Person

5. When the user selects a page in **Add a Person**, the body of `POST /people` gets a `source` with the `feed_guid`, the `title` and the `feed_url` of the page.
6. The core makes a `ValueSource` from that source, and gets the host from `feed_url` with the same function as `page_suggestions`. It saves the picture and the website with that source. A value that the user changed in the sheet gets no source, so it shows `Added in Studio`. The UI sends a source for a value only when the value is the same as the value of the page.
7. A body with no `source` keeps the current result: values with no source.

## Alternatives Considered

- Save the host in `person_pages`. Rejected: it needs a schema change for a value that the page already reads. A move of the page changes its host, so a saved host can become incorrect.
- The host as a tag after the title, or only the host. Rejected by the operator.
- Get the host in the UI from `feed_url`. Rejected: the core already has this rule.

## Consequences

- Two pages with the same title show different second lines when Music Index holds both.
- A person from **Add a Person** shows the page as the source of the picture and the website. **Update from Linked Pages** then finds no new value for them.
- Without network, each linked page row says `Host not known`.

## How This Decision Is Checked

Mechanical:

- A Node test proves the second line of item 1 with a matching suggestion, and the text of item 3 with no suggestion and with a suggestion that has no host.
- A Node test proves that the body of item 5 has the source for a page value, and has no source for a value that the user changed.
- A Rust test proves that `add_person` with a source saves the picture and the website with `source_guid`, `source_title` and `source_host`. A second case proves item 7.
- The four commit checks pass.

Visual, by the planning model:

- The page of a person with two linked pages that have the same title and different hosts, at 1280 px light and 390 px dark. Each row has its host on the second line.
- A person made with **Add a Person** from a page. The picture row and the website row say `From "{title}" on {host}`.

Human, by the operator. This gate passed on 2026-11-14:

- In the desktop app, open the page of Prosperous Soul. Each linked page row names its host.
- Add a person from a live page on Music Index. The picture and the website name that page.

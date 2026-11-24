# ADR 0246: A Link With No Medium, Equal Changes In Review And The People Button

- Status: Accepted
- Date: 2026-11-22
- Owner: operator
- Accepted: 2026-11-22 by the operator, from drawings in the questionnaire. For finding 9, Studio uses the linked feed and also adds the medium. For finding 12, the review shows the count line and a closed fold.
- Technical choices: the main session selected the condition and the shared fold of item 4. It also selected the names of item 5 and the functions of item 2.
- Changes: [ADR 0205](../archive/adr/0205-publisher-feeds-announcements-and-show-grants.md) item 6. A channel link with no `medium` that names a release of the collection is a release link.
- Changes: [ADR 0114](0114-the-review-shows-the-changes-in-words-and-marks-the-changed-part-of-a-line.md) item 2. A set of equal changes shows no example rows.
- Changes: [ADR 0245](0245-review-repairs-short-addresses-and-offers-the-people-values.md) item 11. The label of the People button.
- Source: [the findings of the tutorial for a hosting user](../reviews/2026-11-22-tj-tutorial-findings.md), finding 9, finding 12 and an open point of task 235.
- Bar: crosses modules. Item 1 changes the feed files that the core writes. Items 4 to 6 change the review. Without this record, an agent can keep the medium test of ADR 0205 item 6. An agent can also put the example rows of ADR 0114 item 2 back.

## Context

The artist feed `elijahliedpubfeed.xml` has six channel `podcast:remoteItem` links with no `medium`. Studio reads a link as a release link only when its `medium` is `music` or `musicL` (`is_release` in `crates/core/src/publisher.rs`). Thus "Move Artists Here…" gave the page of Elijah Lied the role artist in the collection, but its file got no `rel`.

The review of The Doerfels shows "18 entries get the relation artist". Below it, the review shows two example rows "Relation: none → artist" and a collapsed line of 16 more. Each row has the same text, so the rows give no information.

The People button of a review is **Use {name}'s Values in {n} Feeds**. For the name "The Doerfels", the label is "The Doerfels's".

## Decision

1. A channel `podcast:remoteItem` of a publisher feed can have no `medium`. It is a release link when it names a collection feed with the medium `music` or `musicL`. The match is the match of ADR 0205 item 12. A link with no `medium` that names no release of the collection is not a release link, and Studio does not change it.
2. `linked_releases`, `publishers_linking`, `set_roles` and `roles` in `crates/core/src/publisher.rs` use the rule of item 1. Each other place that reads a release link of a publisher feed uses it too. Discovery and the store of a read file have no list of the releases of the collection. They keep the medium test of ADR 0205 item 6.
3. When Studio sets the `rel` of a link of item 1, it also writes `medium` with the medium of the named release. Thus the link agrees with ADR 0205 item 6 after the write.
4. In a review group, a set of more than two changes of one path can have one new value and one old value. A group can have more than one such set, for example the medium and the relation of the same links. Each set gives the summary line of ADR 0114 item 2, and the sets share one closed fold. The fold title is "Show the {n} entries", or "Show the {n} credits" for a credit group. The fold lists the name of each entry once. The core gives the position of each entry in its list, and the fold counts the entries by that position. Thus two entries with the same name are two entries. It shows no example row above the fold. A change in a set that has a warning, such as a key that does not agree with the profile name, also shows as a row.
5. The name of a link entry is its `title` in the draft. A link with no `title` in the draft has the title of the collection feed that it names (ADR 0205 item 12). For a feed with a draft, the match and the title come from the draft. Else the name is its `feedUrl`, and else "Entry {k}", where k is the position of the entry in the fold.
6. A set with more than one old value keeps ADR 0114 item 2: two example rows and one collapsed line. A group can have equal sets and such a set. Then each set gives its summary line, the equal sets share the fold of item 4, and the other set keeps its example rows and its collapsed line.
7. The People button of ADR 0245 item 11 is **Use the People Values in {n} Feeds**, or **Use the People Values in 1 Feed**. The label has no name, because the group of the button names the person.

## Alternatives Considered

- Write only the `rel`, and keep the link with no `medium`. The operator selected item 3, so that the file agrees with ADR 0205 item 6.
- Treat each link with no `medium` as a release link. Rejected, because a link can name a label or another artist.
- Show the count line only, with no fold. The operator selected the fold, so that the user can see which entries change.
- Add only `'` after a name that ends in "s". Rejected, because a built possessive can be incorrect for a name in another language.

## Checks

Mechanical:

- `crates/core/src/publisher.rs` tests: a publisher draft with a link with no `medium` that names a `music` feed of the collection. `linked_releases` gives that feed. `set_roles` gives the link `rel="artist"` and `medium="music"`. A link with no `medium` to a feed that is not in the collection does not change.
- `publishers_linking` gives the publisher feed for a release that it links with no `medium`. `roles` gives the role of a link with no `medium` that has a `rel` and names a release.
- A Node test of `fieldRows` in `crates/api/ui/review.js`: 18 equal changes of `rel` give the summary, no example row and a fold with 18 names. 6 equal changes of `medium` and 6 of `rel` on the same entries give two summary lines and one fold with 6 names. Three changes with two old values give two example rows and the collapsed line. A group with an equal set and a set with two old values gives two summaries, the example rows, the collapsed line and the fold. Two entries with the same name give a fold of 2. An equal credit key change with a name that does not agree keeps its warning row.
- A core test: a review of a link change gives the link `title` as the entry, and the pipeline diff gives the title of the named collection feed for a link with no `title`. The match uses the GUID of the draft. A link that loses its `title` gets the collection title.
- A Node test of the People panel model gives **Use the People Values in 8 Feeds** for "The Doerfels".

Visual, by the main session with screenshots of the tutorial data (`.studio-dev/v-230/data-tut`):

- After **Move 3 Artists**, the review contains Elijah Lied, and its rows give the relation artist for its releases.
- The review of The Doerfels shows the summary line and the closed fold "Show the 18 entries". The open fold lists the release titles. No "Medium" row repeats.

## Consequences

- An artist feed that an older tool made with no `medium` gets correct links at the first role change.
- The review of a large role change is short.
- The tutorial for a hosting user can state that the move gives each of the three pages the role artist.

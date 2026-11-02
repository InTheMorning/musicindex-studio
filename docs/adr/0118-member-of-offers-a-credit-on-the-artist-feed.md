# ADR 0118: "Member Of" Offers A Credit On The Artist Feed

- Status: Accepted
- Date: 2026-11-02
- Owner: operator
- Accepted: 2026-11-02 by the operator, from screens 7 and 8 of the mockups in `.studio-dev/mockup-lookup/site/`.
- Changes: [ADR 0111](0111-the-page-of-a-person-checks-the-feeds.md) item 3, the hint of "Member of".
- Uses: [ADR 0098](0098-the-people-module.md) items 2, 6 and 8 for the membership and the change sets. [ADR 0091](0091-a-change-set-edits-one-recipient-in-many-feeds.md) for the change set. [ADR 0111](0111-the-page-of-a-person-checks-the-feeds.md) item 10 for the credits of a person. [ADR 0112](0112-a-credit-role-comes-from-a-list.md) for the role. [ADR 0116](0116-the-page-of-a-person-has-save-and-discard.md) for Save. [ADR 0067](0067-a-read-only-feed-opens-in-the-editor.md) for a read-only feed. [ADR 0071](0071-settings-is-a-destination-and-each-page-orders-its-sections-by-tier.md) for the tiers.

## Context

- "Member of" links a person to one publisher feed of the collection (ADR 0098 item 6). The table `person_members` keeps the link, and the core accepts only a publisher feed.
- On 2026-11-02 the link had these effects, all in Studio only:
    - The People list shows "Member of {feed}" on the row of the person.
    - The page of the person shows the feed, with its dot, its first role and a link.
    - The image and the link of the feed are suggestions for the person (ADR 0113, the source `member`).
    - The export gives the feed title and its `podcast_guid`.
- The link writes no feed. The page of the artist feed does not show its members.
- The operator asked on 2026-11-02 if a membership must add the person to a `podcast:person` of the artist feed. The operator wants "Member of" to link bandmates to the correct feed of the collection.
- The Podcasting 2.0 namespace gives the parent of `podcast:person` as "`<channel>` (for a podcast) or `<item>`", with the count "Multiple" (`docs/tags/person.md`). The medium `publisher` describes a feed that "links to other feeds a publisher owns using the `<podcast:remoteItem>` element", with no other limit (`docs/tags/medium.md`).
- The research note `contributor-database/publisher-entities.md` in `../musicindex-research` counts 16 of 1,779 publisher feeds with a `podcast:person` tag. musicindex reads each `podcast:person` into `source_contributors`, with its `npub`, `img` and `href`. Thus a credit in an artist feed lets musicindex and other apps find the member.
- It is not known which apps show a channel credit of a publisher feed.

## Decision

1. "Member of" keeps the membership of ADR 0098 item 6 in `person_members`. The menu keeps one feed. A change of the menu is a pending change of ADR 0116.
2. The hint of "Member of" is "The artist or label that this person is in. After Save, Studio can add this person to the credits of that feed."
3. After a Save that adds a membership, the page opens the sheet "Add {name} to the Credits of {feed}?". It does not open when the channel of the feed has a credit of the person (ADR 0111 item 10), or when the feed is read-only (ADR 0067).
4. The sheet has:
    - The note "{name} is a member of {feed} in Studio. The credits of the artist feed can also show it, so that apps and musicindex.org can find {name}."
    - The menu "Role" from the role list (ADR 0112). The usual role of the person is the start value.
    - The section "The credit", with one row: the name, the role with its group, and the picture, the information address and the Nostr key of the person that the credit gets.
    - The note "The feed is public. The name, the picture and the information address go in it." The note also names the Nostr key when the person has one.
    - The buttons "Not Now" (normal) and "Review in 1 Feed" (primary).
5. "Review in 1 Feed" makes a change set of ADR 0091 with one feed. The core adds one `podcast:person` to the channel of the publisher feed, with the values of the sheet. The core writes the group of a role of the list (ADR 0112). The write goes through `pipeline::write_draft` with the ID of the diff of that feed.
6. After a Save that removes a membership, the page opens the sheet "Remove {name} from the Credits of {feed}?" when the channel of the feed has a credit of the person. The buttons are "Not Now" (normal) and "Review in 1 Feed" (primary). The change set removes that credit.
7. "Not Now" closes the sheet. The membership stays as Save kept it, and no feed changes.
8. The page of a publisher feed gets the section "Members", with the tier `details`, before the section `details`. `crates/api/ui/pages.js` gives its place.
9. Each person with a membership of a publisher feed is a member of it. Each person with a credit in the channel of the feed is also a member. The core gives the list. Each row shows the picture, the name and the usual role. It also shows "in the credits of this feed" or "only in Studio". A row opens the page of the person.
10. The note of the section is "The people that are members of this {role word}. A member that is not in the credits is only in Studio." The role word is the label of the feed, in lowercase. With no member, the section does not show.

## Alternatives Considered

- A credit that Studio adds at each Save, with no sheet. Not chosen: each write needs a diff that the user has seen (ADR 0007), and the credit puts personal data in a public feed.
- The credit as the only record of a membership, with no row in `person_members`. Not chosen: a read-only feed cannot get the credit, and the user can keep a membership in Studio only.
- A separate operation in "Change your feeds", with no sheet after Save. Not chosen by the operator: the operator selected the sheet after Save.

## Consequences

- The core gets a change set that adds or removes one channel credit of one publisher feed.
- The core gives the members of a publisher feed, from `person_members` and from the index of the credits.
- `pages.js` gets the section `members` on the publisher page.
- The membership and the credit can be different. Item 9 shows each condition, and "Check your feeds" does not change.
- A task file gives the work.

## Non-goals

- More than one membership in the menu. The core keeps a list, but the menu keeps one feed (ADR 0111 item 3).
- A credit on the releases of the feed. "Add to All Releases" of ADR 0098 does that.
- A check of which apps show a channel credit of a publisher feed.

## Acceptance Criteria

Mechanical:

- A core test: the change set adds one `podcast:person` to the channel of a publisher feed with the role and its group, and removes it. A read and a write with no other edit keep the other bytes.
- A core test: the members of a publisher feed hold each person with a membership and each person with a channel credit, with the condition of each.
- An API test: the members route and the change set route.
- A Node test: the sheet opens after a Save that adds a membership, and not when the feed has the credit or is read-only. The model gives the texts of items 2, 4 and 10.
- A Node test: `pages.js` puts `members` in the tier sequence of the publisher page.
- The four commit checks pass.

Visual: screens 7 and 8 of the mockups, at 1100 px and at 390 px with touch, in the two themes. `$edges()`, `$overflow()` and `$fieldText()` are empty.

Human: the operator sets "Member of" of a bandmate to HeyCitizen, selects Save, then "Review in 1 Feed", and writes the feed. The HeyCitizen page then shows the bandmate with "in the credits of this feed".

# ADR 0245: Review Repairs Short Addresses And Offers The People Values

- Status: Accepted
- Date: 2026-11-22
- Owner: operator
- Accepted: 2026-11-22 by the operator, with two options from drawings in the questionnaire. Review shows a repair panel before the write, and the write stays off until the repair. Review also offers the values of People, and a difference does not stop the write. The operator wrote: "we want our users to have correct info in their People list which corrects all the feeds before writing."
- Technical choices: the main session selected the name of the finding kind, the texts and the condition of item 2. It also put the People rule in ADR 0023 (item 9).
- Revised: 2026-11-22 by the main session, before the build. Item 12: the correction changes the drafts directly. ADR 0091 does not let a second change set change a feed of an open change set (`Cause::InOpenChangeSet` in `crates/core/src/change_set.rs`).
- Bar: crosses modules. It changes the feed check of the core, the save of a person, the review of a draft and the review of a change set. Without this record, an agent can add `https://` at the write with no review, or put the People values into the feeds with no button.
- Changes: [ADR 0023](0023-web-addresses-and-audio-file-facts.md) items 1, 3 and 4. A person value is an address field, and an address with no scheme gets a repair.
- Changes: [ADR 0219](0219-a-release-checklist-and-a-check-of-the-address-type.md) item 1. The feed check gets the finding kind `short_address`, with the level Stop.
- Uses: [ADR 0174](0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md) for findings and repairs. [ADR 0224](0224-a-finding-that-stops-the-write-and-a-new-release-opens-add-tracks.md) item 1 for the level Stop. [ADR 0218](0218-a-page-link-in-review-and-a-suggestion-starts-with-no-person.md) items 1 and 5 for a panel that stops the write. [ADR 0203](0203-people.md) items 64 to 73 for the check of a person and the operation **Correct**. [ADR 0244](0244-studio-reads-the-fee-value-of-musicindex-adr-0088.md) item 4 for the shown words.
- Source: [the findings of the tutorial for a hosting user](../reviews/2026-11-22-tj-tutorial-findings.md), findings 10 and 11.

## Context

The release "Bloodshot Lies - The Album" of the tutorial has credits of "The Doerfels" with the information address `www.doerfelverse.com`. ADR 0023 item 3 refuses the write of a feed with an address that is not correct. In the tutorial walk, the move review of 37 feeds wrote the other feeds, and then refused this feed with `bad_address`. The refusal named the model path `items/0/persons/0/href`.

The Check tab showed no finding for the address. ADR 0023 item 9 has a repair only for a space in an address, and only in the list of the releases of a feed.

The check of a person (ADR 0203 item 64) already finds a credit with an information address that differs from the person. It runs only on the page of a person, and only when the user opens it. The tutorial data has no person, so it gave nothing.

People saves a picture and an information address with no check of the address.

## Decision

### The Finding

1. The feed check gives one finding `short_address` for each address field of ADR 0023 item 1 that item 2 can make correct. The level is Stop.

   | Kind | Title | Reason | Button |
   |---|---|---|---|
   | `short_address` | "{place} has an address that is not full" | "{address} has no https://. Studio cannot write it." | **Add https://** |

2. Item 1 applies when the text has no scheme, and `https://` followed by the trimmed text is a correct address under ADR 0023 item 2. Examples: `www.doerfelverse.com` and `doerfelverse.com/artists`. A text with a space, `none` or an empty host gives no `short_address` finding. ADR 0023 item 3 still refuses it.
3. The place of item 1 uses the words of the glossary and the name or title of the element. Examples: "Credit · The Doerfels · Information address", "Track 3 · Neon Diner · Audio file". The core gives the model path with the finding, and the UI shows only the place.
4. **Add https://** is a repair of ADR 0174 item 6. It puts `https://` before the trimmed text of the one field that the finding names. It changes no other field. The user sees the change in Review.

### Review

5. Each review examines each feed that it writes before the write. This includes the review of one draft and the review of a change set.
6. When a feed has a `short_address` finding, the review shows a panel at its top with the title "{n} feeds have addresses that are not full", or "1 feed has addresses that are not full". The panel lists each feed with each place, the address and the new address. The panel button reads **Add https:// in {n} Feeds** or **Add https:// in 1 Feed**. The button makes the repair of item 4 for each listed field. Then the review reads the diffs again.
7. While the panel of item 6 shows, the write button is disabled, and the line "Repair these addresses first." shows above the footer, as in ADR 0218 item 5. Studio writes no feed of the review until the repair.
8. An address that is not correct and has no `short_address` finding stops the write as ADR 0023 item 3 gives. The review names it with the place of item 3 and the button **Open the {release word}**. The refusal of the write names each field with its place, not with its model path.

### People Before The Write

9. ADR 0023 item 1 also covers the picture and the information address of a person in People. The save of a person refuses an address that is not correct. The field shows the error of ADR 0023 item 4. For a text of item 2, the field also has the button **Add https://**, which changes the text in the field.
10. Each review runs the check of ADR 0203 item 64 for each person with a credit or a payee in the feeds that it writes. It keeps the findings for those feeds only. The neutral finding "Not your default address" does not count.
11. When item 10 gives findings, the review shows a second panel under the panel of item 6. Its title is "{n} feeds differ from People", or "1 feed differs from People". It lists each person with each finding, as ADR 0203 item 68 gives. Each person has the button **Use {name}'s Values in {n} Feeds** or **Use {name}'s Values in 1 Feed**.
12. The button of item 11 opens the sheet "Correct the Feeds: {name}" of ADR 0203 item 71 for those feeds only, with the start selections of item 71. In this sheet, the primary button reads **Apply to the Drafts**. The core applies the selected corrections of the operation **Correct** to the drafts of those feeds directly, as a repair of item 6 does. It makes no change set of its own. The user goes back to the same review, which reads the diffs again. **Discard** of the review also removes these corrections (ADR 0207 item 27).
13. The panel of item 11 does not stop the write. A difference can be intentional, for example another picture on one release.
14. Studio applies no People value and adds no `https://` without a button of the user.

## Alternatives Considered

- Write the other feeds and keep the feed with a short address under "Changes to write". The operator selected the panel of item 6, so that no change set is written only in part.
- Apply the People values at each write with no button. The operator selected the button of item 11, because a difference can be intentional.
- Add `https://` at the write with no review. Rejected, because ADR 0023 item 9 makes no change without a button of the user.

## Checks

Mechanical:

- `crates/core/src/feed_check.rs`: `www.doerfelverse.com` in a credit `href` gives one `short_address` finding with the level Stop, the repair **Add https://** and the place "Credit · {name} · Information address". `none`, `https://` and a text with a space give no `short_address` finding.
- The repair changes only the named field to `https://www.doerfelverse.com`, and `bad_addresses` of the draft is then empty.
- The API of a change-set review gives the short addresses of each feed and refuses the write while a short address stays.
- The save of a person with the information address `www.doerfelverse.com` gives an error, and the person keeps the old value.
- The People findings of a review contain only the feeds of the review, and contain no neutral finding.
- After **Apply to the Drafts**, the drafts have the values of the person, and the review gives no People finding for them. This also applies to a feed of an open change set, and **Discard** of that change set removes the corrections.
- A Node test gives the texts of items 1, 6, 7 and 11 from the view model.

Visual, by the main session with screenshots of the tutorial data:

- The move review of the tutorial shows the panel of item 6 with "Bloodshot Lies - The Album", and the write button is disabled. After **Add https:// in 1 Feed**, the panel goes and the write button works.
- With a person "The Doerfels" with the information address `https://www.doerfelverse.com/`, the move review shows the panel of item 11.

## Consequences

- The tutorial for a hosting user can drop its note for "Bloodshot Lies - The Album" when this is built.
- A person with correct values corrects the feeds at each write, with one button for each person.
- A review now reads the index of the credits and the payees before it opens. This read makes no request.
- Open: the page of a person does not offer **Correct the Feeds** at once after a save of a new value. ADR 0203 has three "Changed by" lines, so that change needs a consolidated ADR for People first.

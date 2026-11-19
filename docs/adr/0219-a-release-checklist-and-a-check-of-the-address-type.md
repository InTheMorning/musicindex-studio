# ADR 0219: A Release Checklist, And A Check Of The Address Type

- Status: Accepted
- Date: 2026-11-19
- Owner: operator
- Accepted: 2026-11-19 by the operator, through the questionnaire, from drawings. For T7, the operator selected "Checklist on page". For O2, the operator selected "Finding with a fix". For the payment form, the operator kept the menu "Type" and asked for a different field for each type, and then selected "Menu selects fields".
- Technical choices: the main session selected items 2, 4, 5, 7, 8, 11, 12 and 15.
- Bar: crosses modules. Items 1 to 12 add findings in the core and show them in the editor and on the Check tab. An agent can also make the opposite choice by mistake: one shared address field, or an automatic change of a type.
- Changes: [ADR 0174](0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md) item 7. Five new finding kinds.
- Changes: [ADR 0217](0217-a-new-artist-sees-the-result-of-a-write-the-payment-types-and-an-undo-of-a-merge.md) items 7 and 11. The menu "Type" selects the fields that show, and each type has its own address field.
- Uses: [ADR 0138](0138-the-names-of-a-wallet-address-and-the-correction-of-a-wrong-key.md) for the words of the two types. [ADR 0007](0007-draft-diff-write-and-announce.md) for Review. [ADR 0070](0070-a-cover-opens-the-artwork-sheet.md) for the artwork sheet.
- Source: [analysis of two journeys for a new independent artist](../research/2026-11-18-user-journeys/analysis.md), theme T7 and observation O2, and its section "Operator choices".

## Context

- **T7.** The Check tab has no finding for a release with no tracks, no cover, a track with no audio file or no payment recipient. The finding kinds are in `crates/core/src/feed_check.rs:20`. `check_item` stops when a track has no enclosure (`feed_check.rs:334`). A new artist can write a release that no listener can play or pay.
- **O2.** The form "Add a Recipient" refuses an address that does not match its type (ADR 0217 item 11, `addressProblem` in `recipients.js:459`). An address that is already in a feed gets no check. An import can put `name@example.com` with `type="node"`. The core has no check of the type (`review.rs:274`, `feed_check.rs:199`), so Review and the Check tab show nothing, and each payment to that recipient fails.
- **The form.** The form has one field "Address" for the two types. The feed keeps one `address` attribute for each `podcast:valueRecipient`, but the form does not have to copy that structure. The operator found a shared field unclear.

## Decision

### The Findings

1. The core gives five new finding kinds. Each one has the level Notice. A finding does not stop a write (ADR 0174 item 5).

   | Kind | Condition | Title | Reason | Button |
   |---|---|---|---|---|
   | `no_tracks` | A release has no track and no live item. | "No tracks" | "Listeners have nothing to play." | **Add Tracks** |
   | `no_cover` | A release has no feed image. | "No cover image" | "Apps show a blank square in place of the cover." | **Add a Cover** |
   | `no_audio_file` | A track has no enclosure. One finding for each track. | "{track} has no audio file" | "Listeners cannot play this track." | **Open the Track** |
   | `no_recipient` | A release has no payment recipient in the feed value block and in no track value block. | "No payment recipient" | "Listeners cannot pay you." | **Add a Recipient** |
   | `wrong_address_type` | A recipient address matches the format of the other type (item 9). One finding for each address. | "{name} has the wrong address type" | "{address} is an LNURL address. The type is Node address." Or the same text with the two types in the other order. | **Use LNURL Address** or **Use Node Address** |

2. "A release" is a feed with `podcast:medium` `music`. Items 1 `no_tracks`, `no_cover` and `no_recipient` apply only to a release. `no_audio_file` and `wrong_address_type` apply to each feed that the editor opens.
3. **Add Tracks**, **Add a Cover**, **Open the Track** and **Add a Recipient** open a field or a sheet. They send no request and change no draft. **Add Tracks** opens the step that adds tracks to the release. **Add a Cover** opens the artwork sheet of the feed. **Add a Recipient** opens the payment page of the feed with the form "Add a Recipient" open.
4. **Use LNURL Address** and **Use Node Address** are repairs (ADR 0174 item 6). The repair changes the type of each recipient in the draft that has that address and the other type, in the feed value block and in each track value block. It changes no address, no split and no name. The user then sees the change in Review.
5. A recipient address that matches no format gives no finding. Only the form refuses it (item 11).

### The Checklist

6. The page of a release shows a checklist under its header when it has at least one finding of `no_tracks`, `no_cover`, `no_audio_file` or `no_recipient`. With none of them, the checklist does not show.
7. The checklist title is "This release needs {n} things", or "This release needs 1 thing". The checklist has four rows in this order: "Tracks", "Cover image", "Audio file of each track" and "Payment recipient". A row with no finding shows a check mark. A row with a finding shows a cross and the button of item 1. When more than one track has no audio file, the row reads "{n} tracks have no audio file", and **Open the Track** opens the first one.
8. The checklist gets its rows from the findings of the check of the draft. The UI calculates no condition again. The rows change with each change of the draft, as the local findings do (ADR 0174 item 16).
9. The Check tab shows the five findings as rows, as it shows each other finding. Each one counts in the tab label "Check {n}".

### The Address Type

10. A recipient row on the payment page has a warning mark when its address has a `wrong_address_type` finding.
11. Each form with the menu "Type" of an address shows the fields of the selected type only:

    | Type | Fields |
    |---|---|
    | "LNURL Address" | "LNURL address" |
    | "Node Address" | "Node address", "Custom key" (optional), "Account" (optional) |

    Each field has its own hint and its own format check. A node address is 66 hexadecimal characters. An LNURL address has the form that `keysend::lookup_url` accepts. The labels of the two optional fields tell that they are optional.
12. Each field keeps its text when the user changes the menu. The save uses the fields of the selected type only. When the text of a field matches the other type, the field tells the reason and has the button **Use LNURL Address** or **Use Node Address**. The button moves the text to the field of the other type and changes the menu.
13. The menu "Type" and the panel of ADR 0217 items 7 and 8 stay. The words of ADR 0138 stay.

## Alternatives Considered

- T7: four findings on the Check tab only. Not selected. The user must open the tab to see them, and a new artist can miss them.
- T7: Review refuses the write of a release with no tracks or a track with no audio file. Not selected. A draft release could not go to the host for a test.
- O2: Review refuses the write until the type is correct. Not selected. A user who cannot correct the address now cannot write other changes.
- O2: Studio changes the type in the draft when it opens the feed. Not selected. The draft then changes with no user action.
- The form: one field "Address", where Studio finds the type from the text. Not selected on 2026-11-18 (ADR 0217) and again on 2026-11-19.
- The form: the two field groups always show, with no menu. Not selected. The user can fill the two groups, and the panel of ADR 0217 needs a new place.

## Consequences

- A new artist sees what the release still needs on the release page, before Review.
- A wrong type from an import shows on the Check tab and on the payment page. The repair goes through Review.
- A user who changes the type in the form does not lose the text of the other field.
- A release with only live items gets no `no_tracks` finding.

## How This Decision Is Checked

Mechanical:

- Core tests examine each new kind of item 1: a feed with the condition gives the finding with its title, reason and button, and a feed without it gives none. A test examines item 2 for a feed that is not a release.
- A core test examines the repair of item 4: the type changes in the feed value block and in a track value block, and the address, the split and the name do not change.
- A UI test examines the checklist view of items 6 and 7: the title, the four rows, the marks, the buttons, and no checklist with no finding.
- A UI test examines the fields of item 11 for each type, and item 12: the text of each field stays after a change of the menu, and the save uses only the fields of the selected type.

Visual, by screenshots of the main session before each commit:

- The release page with the checklist at 1280 px and 390 px.
- The Check tab with the new findings, and the payment page with the mark of item 10.
- The form "Add a Recipient" with each type selected.

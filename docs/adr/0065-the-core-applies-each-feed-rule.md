# ADR 0065: The Core Applies Each Feed Rule

- Status: Proposed
- Date: 2026-10-27
- Owner: operator

## Context

[ADR 0002](0002-rust-core-json-api-browser-ui.md) says: "Each feed rule (identity, value blocks, round trip) is in the core, not in the UI." `AGENTS.md` gives this as a durable rule. No test enforces it.

On 2026-10-27 the [UI feed rule audit](../reviews/2026-10-27-ui-feed-rule-audit.md) found four feed rules in the UI:

- The cover form of ADR 0060 items 3 to 5, in `cover.js` `withCover`. Task 038 said "The core has no cover rule".
- The listing rule of ADR 0063 items 3 to 5, in `listing.js` `withHidden` and `withPlatformBlock`. The verification of ADR 0063 names `listing.js`.
- The hidden test of ADR 0063 item 2, in `listing.js` `isHidden` and also in the core `Feed::hidden`.
- The alternative enclosures of a lossless track of ADR 0033 item 6, in `editor.js` `alternateEnclosures`.

A client that writes the model with no UI does not apply these rules. Such clients are an API call, a connected app and a second UI.

The first version of packet 041a also said that the core does not change a value. The operator corrected it on 2026-10-27. ADR 0064 item 4 now gives the explicit form to the core, at the draft save.

On 2026-10-27 the operator made three selections. The core applies a rule in the two ways of items 4 and 5. The guard is items 11 and 12. The work of this ADR comes before task 041.

The core already changes a sent model at the draft save. `pipeline::replace_model` calls `keep_identity` for the GUIDs and `number_tracks` for the track numbers. The live status changes through a core route, `/feeds/{id}/live-items/{guid}/status`.

## Decision

### Feed Rules And Edit Aids

1. A feed rule decides what the written feed holds after a user action. Each feed rule is in the core. The UI does not have a copy of it.
2. An edit aid gives a start value or a display value. The user sees the result in the editor before the save, and can change it. An edit aid can be in a UI model module ([ADR 0042](0042-ui-models-with-node-tests.md)).
3. The core can change a sent value to apply a feed rule. A read and a write with no edit continue to give the same bytes ([ADR 0012](0012-lossless-tree-with-a-typed-model.md)). The writer changes only the elements that the rule or the user changed.

### Two Ways To Apply A Rule

4. A rule that gives one sent field the form of the feed is a step of `pipeline::replace_model`, after `number_tracks`. The UI sends the plain value. The step compares it with the draft. The explicit form of ADR 0064 item 4 and the cover form of item 6 use this way.
5. A rule where one user action changes more than one element is a core route for that action. The route changes the draft and gives the new model, as the live status route does. The listing rule of items 7 and 8 uses this way.

### The Cover

6. The UI sends the new cover address in `image`. A step `keep_cover_form` applies ADR 0060 items 3 to 5:
   - When the draft has only the RSS `<image>`, the address goes to `rss_image.url`, and `image` stays empty.
   - When the draft has both tags and the sent `image` is different from the draft `image`, `rss_image.url` gets the same address.
   - When the draft has both tags with different addresses and the sent `image` is the same as the draft `image`, nothing changes (ADR 0060 item 4).
   - A sent empty `image` removes `image` and the full `rss_image`.
   - The display functions `coverOf`, `coverForm` and `coverConflict` stay in `cover.js` as edit aids. `withCover` goes.

### The Listing

7. `POST /feeds/{id}/listing` with `{"hidden": true}` or `{"hidden": false}` applies ADR 0063 items 3 and 4 in the core, and gives the new model. `POST /feeds/{id}/listing/{platform}` with `{"hidden": …}` applies ADR 0063 item 5 to the block for that platform.
8. The UI sends each pending edit before it calls a listing route, as the live status does. `withHidden` and `withPlatformBlock` go from `listing.js`.
9. The model response of a draft gives `hidden` from `Feed::hidden`. `isHidden` goes from `listing.js`. The UI uses the value of the API.

### The Lossless Enclosures

10. The upload facts of a lossless file give `alternate_enclosures`, which the core makes from ADR 0033 item 6. The import and the change of the audio of a track put these entries into the item. `alternateEnclosures` goes from `editor.js`.

### The Guard

11. A test cannot tell a feed rule from an edit aid in JavaScript. The guard is a manual check. Each task packet that changes the UI has the constraint "No feed rule in the UI (ADR 0065)". The review of each packet examines each new UI function that changes the model, against items 1 and 2.
12. A Node test enforces item 9 and the removals of items 6, 8 and 10. It fails when `listing.js` exports `isHidden`, `withHidden` or `withPlatformBlock`, when `cover.js` exports `withCover`, or when `editor.js` defines `alternateEnclosures`. Its failure message names this ADR and says to call the core.

### Not In This Decision

13. The display rules of the editor, such as the summary lines and the near ratio of the artwork list.
14. A feed rule that the core has and the UI does not have. No change is necessary.

## Alternatives Considered

- Keep the rules in the UI, and change ADR 0002 to permit feed rules in a UI model module. Rejected: each client must then copy each rule. A connected app and a second UI do not run the browser modules.
- Apply the listing rule at the draft save, from the sent tags. Rejected: one switch changes two kinds of tags. The core cannot see from the sent tags that the user set the switch.
- Put each rule in a core route. Rejected for the cover and the explicit value: the user types the cover address, and a route for each change of a field adds requests for no gain.
- A static test for each feed rule in the UI. Rejected: a test cannot tell a feed rule from an edit aid. Item 12 enforces only the removals of this ADR.

## Consequences

- Each client of the API gets the same feed after the same user action.
- ADR 0063 gets an amendment: its verification names the core and the routes of item 7, not `listing.js`.
- ADR 0060 gets an amendment: items 3 to 5 are a step of the draft save.
- Task 038 and the doc comment on `rss_image` in `crates/core/src/feed.rs` say "The core has no cover rule". The implementation task removes the comment. The task file stays as the record of its work.
- The packet template of the project gets the constraint of item 11.
- A task with three packets can do this work: the cover step, the listing routes, and the lossless enclosures with the guard. It comes before [task 041](../tasks/studio-v1-task-041-msp-fields.md). Packets 041b, 041d and 041e get the constraint of item 11.

## Verification

Mechanical:

- The corpus tests: each feed gives the same bytes after a read and a write with no edit.
- A core test of `keep_cover_form` for each case of item 6, with a draft in each of the four forms of ADR 0060 item 3.
- An API test of `POST /feeds/{id}/listing`: a hide of a feed with no block tag adds both tags. A show writes `No` and `no`, and adds no tag. A block for one platform does not change.
- An API test of `POST /feeds/{id}/listing/{platform}`: only the block of that platform changes.
- An API test: the model response of a hidden draft gives `"hidden": true`.
- An API test: the upload facts of a WAV file give two alternative enclosures with the titles and the types of ADR 0033 item 6.
- The Node test of item 12.

Visual:

- The cover field of a DeMu feed, of a feed with only `itunes:image` and of a feed with both tags, after an edit. The written feed keeps its form.
- The switch "Hide from apps" and "Advanced", after a hide and after a show.
- An import of a WAV file gives the two alternative enclosures in the track.

## Invariants

- Each feed rule is in the core. The UI has no copy of a feed rule.
- A read and a write with no edit give the same bytes.

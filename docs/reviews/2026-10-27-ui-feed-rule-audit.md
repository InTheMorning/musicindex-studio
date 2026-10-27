# UI Feed Rule Audit, 2026-10-27

This record gives the result of an audit of the UI modules in `crates/api/ui/`. It states no rule. [ADR 0002](../adr/0002-rust-core-json-api-browser-ui.md) owns the rule "Each feed rule is in the core". [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md) (proposed) owns the corrections.

## Why

On 2026-10-27 the first version of packet 041a said that the core does not change a value. No decision gave that rule. The operator asked where else the project uses this idea, and asked for an audit of the UI modules.

## Method

Claude read the header and the exported functions of each module in `crates/api/ui/`. Claude also read each function in `editor.js` and `values.js` that makes a new model, a new item or a new entry. Claude read the model functions of `cover.js`, `listing.js`, `live.js` and `playlist.js`. Claude searched the docs and the code for statements that the core makes no change.

The audit uses two terms:

- A **feed rule** decides what the written feed holds after a user action. It must apply for each client of the API.
- An **edit aid** gives a start value or a display value. The user sees the result in the editor before the save, and can change it. A different client can do a different thing, and the feed is still correct.

## Feed Rules In The UI

| Location | Rule | Owner | Since |
|---|---|---|---|
| `cover.js` `withCover` | An edit of the cover keeps the form of the feed: only RSS `<image>`, only `itunes:image`, or both. A clear removes each cover tag. | ADR 0060 items 3 to 5 | Task 038 |
| `listing.js` `withHidden`, `withPlatformBlock` | A hide gives `Yes` and `yes`, and adds a missing tag. A show gives `No` and `no`, and adds no tag. A block for one platform does not change. | ADR 0063 items 3 to 5 | Task 040 |
| `listing.js` `isHidden` | A feed is hidden when a block for each app says `yes`. The core has the same rule in `Feed::hidden`. | ADR 0063 item 2 | Task 040 |
| `editor.js` `alternateEnclosures` | A lossless track gets two alternative enclosures with fixed titles, types and a `default`. | ADR 0033 item 6 | Task 019 |

Task 038 says "The core has no cover rule". The doc comment on `rss_image` in `crates/core/src/feed.rs` says the same. ADR 0060 does not say this. It gives the rule to "the writer".

The verification of ADR 0063 names `listing.js`. Thus ADR 0063 put a feed rule in the UI, against ADR 0002.

## Edit Aids In The UI

These stay in the UI:

- `editor.js`: the blank item of "Add a track" and of the import. The core gives the GUID and the track number at the save (ADR 0008, ADR 0024).
- `editor.js`: the sequence of the imported tracks, and the selection of the first image as the album art (ADR 0026 item 3).
- `editor.js` `withNewAudio`: the change of the audio of a track (ADR 0040 items 4 and 5). It uses the alternative enclosures of `alternateEnclosures`, which ADR 0065 moves. `keep_identity` in the core keeps the GUID.
- `values.js`: the copy of the album recipients to a track, a new recipient with no split, and the percentages for the display.
- `live.js`: the conversion of the local time to ISO 8601, and the display of the status. The status changes through a core route.
- `playlist.js`: the add, the move and the removal of a remote item.
- `cover.js` `coverOf`, `coverForm`, `coverConflict`: the display of the cover field (ADR 0060 items 2 and 4). The core has no copy of these functions.
- `picker.js`, `audio.js`, `upload.js`, `draft.js`, `shell.js`, `announce.js`, `signing.js`, `account.js`: the picker, the player, the uploads, the draft keys, the navigation and the sign-in. None of them decides what a feed holds.
- The planned `credits.js` and `artwork.js` of task 041 give start values that the user sees.

## Other Statements Found

- ADR 0024 item 3, "The core does not change a `pubDate` when the sequence changes", is a rule for one field. It is correct.
- "As the source wrote it" in `feed.rs` and `xml.rs` describes the lossless read of ADR 0012. It is correct.
- The first version of packet 041a. The operator corrected it on 2026-10-27.

## Limits

Claude did not run the UI for this audit. Claude read `app.js` only at the calls that change a model or send a body to the API. A rule that is in the CSS or in the HTML is not in this audit.

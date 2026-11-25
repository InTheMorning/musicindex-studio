# Findings: The Wild RSS Check Of A New Release, And Two Proposals

- Date: 2026-11-25
- Source: the operator sent the result of the validator Wild RSS for a new release of a first user.
- Feed: `https://www.whitetriangles.com/Music/Heartseed/nor-you-i.xml` (the release "Nor You I" of Heartseed).
- Method: the main session opened the validator page in headless Chromium and read the rendered report. The main session also downloaded the feed (2,347 bytes, HTTP 200) and examined it.
- Status: recorded. No code change was made for these findings. Findings 1 to 4 and proposals P1 and P2 need work. Findings 5 to 7 need no change in Studio.

## Who Wrote The Feed

The feed has no `generator`, so the feed does not name the tool. The `lastBuildDate` has the form `Tue, 24 Nov 2026 07:16:10 +0000`, which is the form of ADR 0127 item 14. This is an inference: Studio probably wrote the feed. Task 257 (ADR 0265) adds the tag to each feed that Studio changes.

## Findings

| # | Validator result | Opinion | Owner |
| --- | --- | --- | --- |
| 1 | `<podcast:transcript url="" type="text/plain"/>` on the track: "Not a valid web URL". | A defect of Studio. The editor adds a blank lyrics entry with `url: ''` and `media_type: 'text/plain'` (`crates/api/ui/editor.js`, `blankLyrics`). The core link check skips an empty value (`crates/core/src/feed/links.rs`, the filter near line 243). Thus an entry with no address goes to the host. | Studio, a correction |
| 2 | No channel `description` and no item `description`. | Correct. Many apps show an empty page without it. The feed check has no finding for it (ADR 0174 item 7, ADR 0219 item 1). | Studio, a decision |
| 3 | No `itunes:explicit`. | Correct. Apple Podcasts requires it. The feed check has no finding for it. | Studio, a decision |
| 4 | No `itunes:category`. | Correct for Apple Podcasts. Music apps do not use it, so it is less important. Studio has no code for the element. | Studio, a decision |
| 5 | "`<podcast:remoteItem>` has no rel attribute" in `podcast:publisher`. | Not correct. The element has `rel="artist"`, and the example of the validator shows it. This is an inference: the validator expects a different value or has a defect. | No change |
| 6 | Three recipients of type `node`: "consider changing to lnaddress". | Advice only. A keysend payment to a node is correct, and each person selected the node. | No change |
| 7 | The response has no `Access-Control-Allow-Origin` header. | Correct, but the web server of the file host sends the headers, not Studio. Only a browser app with no server is affected. | The operator of the file host |

Recommendation for findings 2 to 4: one decision adds finding kinds to the feed check (ADR 0174 item 7 requires a change of that record for a new finding). A finding stays a Notice and does not stop a write (ADR 0174 item 5). The rows of the release checklist of ADR 0219 item 7 can show finding 2 and finding 3.

## Proposals Of The Operator

### P1: A Check Of All Feeds And One Place For Their Issues

The operator proposed a check of all feeds of the collection, and a repair of each feed from one place. The place is one "Issues" destination at the bottom of the left panel.

- ADR 0259 item 23 leaves this open: "A subsequent ADR decides the button 'Check Everything' of the Releases page, the check of the media files and the report of the results. That ADR needs a mockup of the report first."
- The page of each feed has the tab "Issues ({n})" (ADR 0174 item 17, ADR 0228 item 13). The destination would show the findings of all feeds together.
- A repair of many feeds can use a change set: one operation on many feeds, with one diff for each feed (ADR 0207 items 24 to 33). A repair changes the draft only when the user selects its button (ADR 0174 item 6).
- Next step: mockups of the destination and of a repair of many feeds, before the ADR.

### P2: Read A Change From Outside Studio With No Question

The operator proposed that Studio reads a change of the hosted file by itself when the feed has no draft edits. Studio asks the user only when a draft with edits conflicts with the hosted file.

- Now, the check shows "Changed outside Studio" and the button "Read Again". The check changes no draft (ADR 0259 items 2 and 8).
- ADR 0007 item 6 already moves a draft with no edits to the new file at a read. P2 makes the check do that read by itself.
- A change that needs the user stays a question: a GUID change (ADR 0249), a move to a different address (ADR 0259 items 3 and 4) and a draft with edits (ADR 0129).
- Open point for the decision: the user does not see the change. A short line in the feed history, or a notice that closes by itself, can tell the user.
- P2 changes ADR 0259 item 8, so it needs an ADR.

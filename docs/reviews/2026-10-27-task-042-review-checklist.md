# Task 042 Review Checklist, 2026-10-27

This record holds the orchestration of [task 042](../tasks/studio-v1-task-042-core-feed-rules.md). It records each packet, each review result, and each decision that the planning model made in the limits of the ADRs. It states no rule. [ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md) owns the rules.

## Sequence

The packets run one at a time, because each packet changes `crates/api/ui/editor.js`:

1. Task 042a.
2. Task 042b.
3. Task 042c.

A coding model implements each packet. The planning model examines the diff and runs the four commit checks before the next packet starts. No packet is committed until the operator says so. The visual checks of all packets come at the end. Task 041 starts after this task.

## Packets

| Packet | Status | Review |
|---|---|---|
| 042a | Complete | Pass. See "Packet 042a". |
| 042b | Complete | Pass. See "Packet 042b". |
| 042c | Complete | Pass. See "Packet 042c". |

## The Review Of Each Packet

- Each new UI function that changes the model gives only a start value or a display value (ADR 0065 items 1 and 2).
- Each removed function has no caller in `crates/api/ui/`.
- The corpus round-trip tests pass with no change.

## Decisions

### Before The Packets

- The operator accepted ADR 0065 on 2026-10-27.
- At the acceptance, the planning model found that the UI keeps its own model after a save. `draft.js` copies only the GUIDs and the track numbers from the saved model. A value that a save step changes would not show, and the cover field would show a false note. ADR 0065 item 4 now says that the UI shows the value of the step. Packet 042a adds `withSavedCover` in `cover.js` for the cover.
- At the acceptance, ADR 0065 item 6 got two details. A clear sends an empty text in `image`, because `null` is also the value of a DeMu feed with no edit. The step does nothing when the sent `image` is the same as the draft `image`.
- At the acceptance, ADR 0065 item 7 got the route for one platform by the `id` of the block. An unknown `id` gives `404`.
- The project has no file for a packet template. `AGENTS.md` gives the constraint of ADR 0065 item 11.
- The explicit value of task 041 needs no copy after a save. The UI reads `true` and `yes` as the same value, and the step of packet 041a gives the same result on each save.

### Packet 042a

- Result: pass. The four commit checks pass. The Node tests give 174 passes. No caller of `withCover` stays in `crates/api/ui/`.
- `keep_cover_form` applies the four cases of the packet in the given sequence. The corpus tests and the cover tests of task 038 pass with no change.
- Deviation: the coding model changed the API test `the_feed_list_gives_the_draft_state_and_the_cover_image` in `crates/api/src/tests.rs`. The packet does not name this file. Before the change, the test sent `image: null` with an `rss_image` to a draft with an `image`. For this value, the new step removes the two tags, as ADR 0065 item 6 says.
- The planning model accepts the change. The test also examines the clear with an empty text.
- Result for API clients: an API client that sends `image: null` to a feed with an `image` removes the cover. ADR 0065 item 6 gives this result.

### Packet 042b

- Result: pass. The four commit checks pass. The Rust tests give 939 passes. The Node tests give 169 passes. No caller of `isHidden`, `withHidden` or `withPlatformBlock` stays in `crates/api/ui/`.
- `set_hidden` and `set_platform_hidden` apply ADR 0063 items 3 to 5. The block tests of task 040 pass with no change.
- The two routes follow `set_live_status`. They change the draft only. An unknown platform gives `404` with the code `not_found`.
- The server shell uses `workspace_router`, so it gets the two routes with no change.
- The planning model wrapped one long doc comment line on `draft_json`.

### Packet 042c

- Result: pass. The four commit checks pass. The Rust tests give 940 passes. The Node tests give 170 passes.
- `lossless_pair` gives the two entries of ADR 0033 item 6. The upload facts give `alternate_enclosures`, also as an empty list.
- The guard `feed-rules.test.mjs` replaces the removal tests of 042a and 042b. The coding model added `withCover` back for a short time. The guard then gave an error with the message of ADR 0065 item 6.
- Deviation: when the MP3 file or the FLAC file has no address, the facts give an empty list. Before the change, the UI gave an entry with no `uri`. The planning model accepts this result.
- The test `testing::sshd::tests::fingerprint_matches_the_client` gave an error one time in a run of the coding model. It gave no error in the next two runs and in five runs of the planning model. Packet 042c does not change this code. The planning model records the result and makes no change.

### The Screenshot Check

On 2026-10-27, an agent took `tools/drive-ui.mjs` screenshots in the dark theme, in the light theme and at 390 px. The 390 px runs used the dark theme. The setup was `studio-desktop` with a local folder host on a loopback HTTP server, and copies of the DeMu `demu-feed-template.xml`. The planning model examined the screenshots.

- The cover field: passed in the three feeds. The field shows the new address, and no note shows.
  - The DeMu feed changes only `<image><url>`. The feed with only `itunes:image` changes only that tag. The feed with the two tags changes the two tags.
  - A clear removes the full `<image>`, as ADR 0065 item 6 gives.
- The WAV import: passed. The track has the MP3 entry first, with `default="true"` and the title "Standard (MP3)". The FLAC entry is second, with the title "Maximum fidelity (lossless)".
- The switch: the values passed, but the check found one defect. After a hide, the diff moved the block `id="spotify"` to the end of the channel. The new block with no `id` took its position.
  - Cause: `write_list` fills the existing elements of one type in sequence. `set_hidden` put the new block first in `blocks`. The packet copied this position from the removed `withHidden`. No ADR gives it.
  - Fix: `set_hidden` adds the new block last in `blocks`. Then each block with an `id` keeps its position (ADR 0012). The test `set_hidden_true_keeps_a_platform_block_in_its_place` examines the bytes of the document. Packet 042b gives the new position.
  - After the fix, the four commit checks pass. The Rust tests give 940 passes, and the Node tests give 170 passes. The screenshots of the switch are from before the fix. The core test examines the new diff.
- Each run gave 4 to 6 console messages "Failed to load resource: 502". The cover addresses of the test feeds are on `example.com`, and that host gives `404` for them. The check found no other console error.

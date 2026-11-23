# ADR 0244: Studio Reads The Fee Value Of musicindex ADR 0093

- Status: Accepted
- Date: 2026-11-22
- Owner: operator
- Accepted: 2026-11-22. The operator decided that Studio keeps its own feed reader (musicindex ADR 0093). The orchestrator session relayed the decision and the task. The task let the Studio session select where the warning shows.
- Technical choices: the main session selected a finding of the feed check for the warning (item 2), its level and its texts.
- Bar: crosses repositories. musicindex ADR 0093 R-FEED-6 gives the rule, and this record gives the location of each part in Studio. Without this record, an agent can put the warning only in the log, where a desktop user does not see it.
- Changes: [ADR 0219](0219-a-release-checklist-and-a-check-of-the-address-type.md) item 1. The feed check gets the finding kind `unknown_fee`, with the level Warning.
- Uses: musicindex ADR 0093 R-FEED-6 item 3, musicindex ADR 0093 R-FEED-3 and R-FEED-4 item 5, musicindex ADR 0087 R-WORD-12. [ADR 0174](0174-a-feed-check-gives-findings-and-repairs-through-the-draft.md) for the levels and the Check tab.

## Context

musicindex ADR 0093 R-FEED-6 lets Studio keep its own feed reader, because Studio edits the document and saves it. The reader must accept the two namespace URIs of musicindex ADR 0093 R-FEED-3 with each prefix. It must read `fee` as musicindex ADR 0093 R-FEED-4 item 5 gives.

At commit `84b9970`, `is_fee` in `crates/core/src/change_set.rs` and `isFee` in the UI accepted only `true`. It gave no warning for a value that is not known.

Studio writes its log to the standard error stream with `eprintln!`. The desktop app does not show that stream, so a user does not see a warning in the log. The Check tab of a feed shows each finding of the feed check, also a finding with no repair.

## Decision

1. `FeeReading::of` in `crates/core/src/feed/value.rs` reads the `fee` text. It removes the spaces at the two ends and ignores capital letters. `true`, `1` and `yes` mean a fee. `false`, `0`, `no`, an empty value and a missing attribute mean not a fee. Each other value means not a fee, and it is not known. `is_fee` uses this function, and the UI function `isFee` in `crates/api/ui/recipients.js` has the same rule.
2. The feed check gives one finding `unknown_fee` for each recipient with a `fee` value that is not known. It examines each value block of the channel, of each item and of each live item, and each value time split in them. The level is Warning, and the finding has no repair.

   | Kind | Title | Reason |
   |---|---|---|
   | `unknown_fee` | "{name} has an unknown fee value" | "The fee value is “{value}”. Studio reads it as not a fee. Apps can read it in a different way." |

3. No `fee` value removes a recipient. Studio keeps the `fee` text of the feed when it saves, until the user changes the fee of that recipient.
4. The words that Studio shows for feed data come from `docs/glossary.md` in `../musicindex` (musicindex ADR 0087 R-WORD-12). Studio keeps no word list of its own. A new shown word or a new name in a file gets a glossary row before it ships (musicindex ADR 0087 R-WORD-13).

## Checks

Mechanical, in `crates/core/tests/feed_reader_rules.rs`. Each test name starts with `r_feed_6_`:

- One test for each `fee` value of item 1: `true`, `1`, `yes`, `false`, `0`, `no`, an empty value, a missing attribute and a value that is not known.
- The value `maybe` gives one `unknown_fee` finding with the level Warning, and the recipient stays in the feed. The known values give no finding.
- A save keeps the `fee` texts ` Yes `, `1`, `maybe` and the empty value.
- A Node test in `crates/api/ui-tests/recipients.test.mjs` gives the same values to `isFee`.
- The reader reads a feed with `https://podcastindex.org/namespace/1.0` and the prefix `pi`, and a feed with `https://github.com/Podcastindex-org/podcast-namespace/blob/main/docs/1.0.md` and the prefix `ns`.

Manual, by the session that reviews a change:

- A diff that adds a shown word for feed data has a row in the glossary (item 4). A test cannot find a new word in the UI text.

## Consequences

- A feed with `fee="1"` or `fee="yes"` now gives a fee in the review, the wallet sheets and the sort of the recipients.
- The Check tab shows a warning for a `fee` value that is not known. The user can correct the fee on the payment page.
- [The list of shown words with no glossary row](../reviews/2026-11-22-shown-words-without-a-glossary-row.md) goes to the musicindex session for rows.

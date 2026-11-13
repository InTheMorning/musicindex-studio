# Local Use Findings Of 2026-11-13

Date: 2026-11-13.

Status: advisory. This review records findings from the use of Studio on 2026-11-13. It states no rule. The ADRs that it names own the decisions.

## Findings

### Finding 1: The Notice Of A Different Address Does Not Tell The Cause Or The Effect

Decision: none at this time.

Owner: [ADR 0150](../adr/0150-studio-shows-a-change-from-outside-when-a-page-opens.md) item 5, and [ADR 0184](../adr/0184-after-a-move-studio-shows-the-podping-of-the-move.md) item 9.

The release page of "Some Songs" showed this notice in the plain style:

> Music Index reads this feed from a different address. https://phafe.com/wp-content/uploads/rssfeed/music/mezzaforte/That_Old_Thing/That_Old_Thing.xml. Your feed is at https://phafe.com/wp-content/uploads/rssfeed/music/mezzaforte/some_songs/some_songs.xml. A different collection or a different tool can hold this feed.

The notice is not an error. Studio asks Music Index for the address of the `podcast:guid` of the feed (`index_difference` in `crates/core/src/outside.rs`). Music Index gave an address that is not the public address in Studio.

On 2026-11-13, the old address `That_Old_Thing/That_Old_Thing.xml` answered `301` with the location of `some_songs/some_songs.xml`. Both addresses gave the same feed, with the `podcast:guid` `07adf84f-c1fa-47a7-a121-0fb544e859bd`. Thus the file host moved the feed. Music Index still has the old address.

The notice has these defects:

- It does not tell if the user must do an action.
- The last sentence gives a cause that is not correct for this feed. No other collection and no other tool holds the feed.
- It does not tell the effect. Music Index follows the redirect, but a change of the redirect stops the reads of Music Index.
- It gives no action, for example a podping of the public address.
- The two long addresses are in the text of the sentences, and the user cannot easily compare them.

A possible change: Studio reads the address at Music Index. That address can give a redirect to the public address. Then the notice tells that the file host sends readers to the new address. It also gives a button that sends a podping of the public address. This change needs a decision.

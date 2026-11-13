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

The Stophammer session examined the record on 2026-11-13, through the orchestrator. It found no Stophammer defect:

- A `301` or `308` at the stored address moves the record (Stophammer ADR 0052 §1 and §2, ADR 0083 §6). This feed has no address that it left, thus no return wait applies (Stophammer ADRs 0096 and 0097).
- The last read was at 05:34:59 UTC on 2026-11-08, event seq `473985`, of `That_Old_Thing.xml`. That read stored the `declared_self_url` `some_songs.xml` and no `itunes:new-feed-url`. No subsequent event names this guid.
- Stophammer inference: on 2026-11-08 the old address answered `200`, not `301`. A self link alone does not move a record, because the crawler does not follow a self link (Stophammer ADR 0052 §2).
- A podping of one of the two addresses corrects the record. A podping of the old address is sufficient because the read follows the `301`.
- The operator searched the archive of the gossip listener on the VPS, which keeps 30 days. It has no podping of `some_songs.xml`. Podpings of `That_Old_Thing.xml` came at 04:58:29 and 05:32:20 UTC on 2026-11-08, before the read of event seq `473985`. No podping of the two addresses came after that read.
- Stophammer inference: the publisher added the `301` after the last podping. Stophammer makes no change.

The notice has these defects:

- It does not tell if the user must do an action.
- The last sentence gives a cause that is not correct for this feed. No other collection and no other tool holds the feed.
- It does not tell the effect. Music Index follows the redirect, but a change of the redirect stops the reads of Music Index.
- It gives no action, for example a podping of the public address.
- The two long addresses are in the text of the sentences, and the user cannot easily compare them.

A possible change: Studio reads the address at Music Index. That address can give a redirect to the public address. Then the notice tells that the file host sends readers to the new address. It also gives a button that sends a podping of the public address. The answer of Stophammer shows that this podping corrects the record. This change needs a decision.

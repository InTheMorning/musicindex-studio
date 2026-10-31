# Keysend And Lightning Address Recipients

Research date: 2026-10-31.
Scope: one recipient with a keysend address and a Lightning address. The report gives what the specification permits, what the providers publish, and what the feeds do now.
This report states no rule. The proposal at the end needs an ADR before work starts.
The report was composed in STE. This report does not claim full STE compliance.

## Method And Sources

| Source | Where | Date seen |
| --- | --- | --- |
| Podcast namespace specification at `d4036dc` (2026-10-10) | `../musicindex-research/sources/podcast-namespace/docs/` | 2026-10-31 |
| The same pages on GitHub, `main` | `docs/tags/value.md`, `docs/tags/value-recipient.md` | 2026-10-31 |
| Discussion "Add a new address type for lightning address" | [podcast-namespace discussion 576](https://github.com/Podcastindex-org/podcast-namespace/discussions/576) | 2026-10-31 |
| Castamatic note on Lightning addresses | [castamatic.com, 2025-09-04](https://castamatic.com/blog/2025/09/04/lnaddress/) | 2026-10-31 |
| The corpus of Studio | `crates/core/tests/corpus/`, 28 feeds | 2026-10-31 |
| MSP 2.0 | `../MSP-2.0/src/utils/xmlGenerator.ts` | 2026-10-31 |
| The keysend and LNURL-pay endpoints of each corpus Lightning address | HTTPS requests with `curl` | 2026-10-31 |
| The keysend path of `zbd.gg`, `blink.sv`, `coinos.io`, `stacker.news`, `primal.net`, `speed.app` and `minibits.cash`, for the name `hello` | HTTPS requests with `curl` | 2026-10-31 |
| The alias and the channel count of a node in the Lightning network graph | `https://mempool.space/api/v1/lightning/nodes/{pubkey}` | 2026-10-31 |

## Findings

### The Specification

1. `podcast:valueRecipient` has the required attribute `type`. Its known values are `node` and `lnaddress` (`value-recipient.md` line 25).
2. For `lnaddress`, the specification says: "Apps should resolve the pubkey and custom record data by calling `https://<domain>/.well-known/keysend/<username>` before sending payment. Results should be cached to minimize network requests."
3. The page `examples/value/lnaddress.md` gives two lookups for one Lightning address: `/.well-known/lnurlp/<name>` (LUD-06, an invoice) and `/.well-known/keysend/<name>` (the node key and the custom data).
4. The specification says that `lnaddress` recipients "can be used in place of, or mixed with, `node` recipients in the same value block". Its example has `method="lnaddress"` with two `lnaddress` recipients and two `node` recipients.
5. The slug list `valueslugs.txt` has `keysend`, `lnaddress` and `amp` as methods, and `node` and `wallet` as recipient types.
6. `podcast:value` has the count "Multiple" in a channel or an item. The specification does not say how an app selects one of more than one block at the same level.
7. No attribute gives one recipient two addresses. Discussion 576 examined a node address with a Lightning address as its fallback (dhk2, 2023-11-21), and a Lightning address first (bumi, 2023-12-07). The visible thread has no decision. The specification at `d4036dc` has the separate type `lnaddress` and no fallback attribute.

Thus, in the specification, a Lightning address is not a second address of a keysend recipient. It is a recipient that holds its keysend route: the app gets the node key and the custom data from the provider at payment time.

### The Providers

The corpus has 30 different Lightning addresses on five domains. Studio requested the keysend lookup of each address:

| Domain | Addresses | Keysend data | LNURL-pay |
| --- | --- | --- | --- |
| `fountain.fm` | 18 | 17 | Yes |
| `getalby.com` | 8 | 6 | Yes |
| `strike.me` | 2 | 0 (404) | Yes |
| `walletofsatoshi.com` | 1 | 0 (404) | Yes |
| `phafe.com` | 1 | 0 (301 to the home page) | Yes, through Alby |

- 23 of 30 addresses gave keysend data. The three failures on Fountain and Alby are `malachi@fountain.fm`, `guest@getalby.com` and `msp@getalby.com`. The cause was not examined.
- Strike and Wallet of Satoshi give only LNURL-pay. An app can pay those addresses with an invoice, but not with keysend.
- A custom domain that sends LNURL-pay to Alby (`phafe.com`) does not also send the keysend lookup.

#### The Nodes Behind The Addresses

A second check on 2026-10-31 examined the nodes that the lookups give, for the names of the nodes on the Recipients page:

- Each `fountain.fm` address with keysend data gives the same node key, `03b6f613…54b79`, with the custom key `906608` and one custom value for each address. The corpus has 16 `node` recipients on that key, in 2 feeds. The alias of the node in the Lightning network graph is `klnd0`, so the alias does not name the provider.
- Each `getalby.com` address with keysend data gives a different node key, with the custom key `696969` and a small custom value. `chadf` gives `032870…`, `ericpp` gives `02c7bb…`, `steven` gives `035ad2…`, and `sirtjthewrathful` gives `031ce2…` with the value `1`. These are personal Alby Hub nodes. The corpus feed `kolomona-podcast-v4vmusic-live.xml` names `031ce2…` with the custom key `696969` and the value `1` as the `node` recipient "TJ Doerfel". Thus the lookup and the feed agree.
- The corpus has 7 `node` recipients on the key `030a58b8…677a3` with the custom key `696969`, in 1 feed. Its alias in the graph is `getalby.com`, and it had 0 open channels (`active_channel_count`) on 2026-10-31. No address of the corpus resolves to it at this time. It is the shared Alby node from before Alby Hub.
- `zbd.gg`, `coinos.io` and `stacker.news` have the name `hello` for LNURL-pay, but give 404 on the keysend path. `primal.net` and `blink.sv` give an HTML page on the keysend path. `speed.app` and `minibits.cash` give no keysend data for the name `hello`. None of these is a shared node with evidence.

Thus two shared nodes have evidence: the Fountain node and the Alby node from before Alby Hub. A `getalby.com` address names a personal node. A domain names a shared node only when two or more addresses of the domain resolve to the same key.

### The Feeds

1. The corpus has 53 value blocks: 29 keysend and 24 lnaddress. It has 106 `node` recipients and 119 `lnaddress` recipients.
2. No corpus feed has both a keysend block and an lnaddress block. Five feeds mix the two recipient types in one block. In three feeds, a block with `method="keysend"` has `lnaddress` recipients. In `behindthesch3m3s-music-pixel-palace.xml`, each of the 9 recipients of one such block is an `lnaddress`. In two feeds, a block with `method="lnaddress"` has a `node` recipient. Thus an app cannot select the recipients by the method alone.
3. MSP 2.0 writes `method="lnaddress"` when any recipient of the block is an `lnaddress` (`xmlGenerator.ts:225`). This agrees with finding 4 of the specification.
4. Four Lightning addresses of the corpus are also `node` recipients in other feeds, with the same node key and custom value as their keysend lookup:
   - `boostbot@fountain.fm`, `frankiepaint@fountain.fm` and `lavish@fountain.fm`. Each node row has the Lightning address as its `name`, for example in `podhome-podcast-satellite-skirmish.xml`.
   - `sirtjthewrathful@getalby.com`, which is the node row "TJ Doerfel" in `kolomona-podcast-v4vmusic-live.xml`.

   Studio shows each of these persons as two rows on the Recipients page. The cause is `RecipientKey`: the address with the custom key and the custom value.

### The Apps

- Castamatic: when a feed has a Lightning address, Castamatic uses keysend if it is available, and else LNURL-pay. The note does not say how it finds the keysend data.
- No source that this research found says how Fountain, Podverse, CurioCaster or podStation select one of more than one `podcast:value` block at the same level.
- A forum comment on Stacker News says that keysend can go away in favor of BOLT 12 offers or AMP. This is not an authoritative source. It is a risk, not a finding.

## What This Means For Studio

1. Studio does not need to write two addresses for one recipient. The specification has no form for it, and an `lnaddress` recipient already gives the app the keysend route when the provider publishes it.
2. A Lightning address is the better address to keep in a feed, when its provider publishes the keysend lookup. The feed then continues to work when the provider changes its node key, which was the cause of discussion 576.
3. A node recipient is still necessary for a provider with no Lightning address, and for a self-hosted node. A Lightning address with no keysend lookup (Strike, Wallet of Satoshi) works only in apps that pay with LNURL-pay.
4. The real defect is in Studio: one wallet shows as two recipients when one feed has the node key and another feed has the Lightning address. A change set then must run two times.
5. Two value blocks in each feed (one keysend, one lnaddress) have no support in the evidence. No corpus feed does it, and no source says how an app selects the block.

## Options For An ADR

1. **Studio links a node recipient to a Lightning address from the keysend lookup.** The core requests `/.well-known/keysend/<name>` for each Lightning address of the collection, and caches the result. A node recipient with the same node key, custom key and custom value is the same wallet. The Recipients page shows one row with the two addresses. The private network guard of ADR 0057 applies to the request.
2. **The user links them by hand,** for a provider with no keysend lookup. The link is in the saved recipients. Studio cannot prove it.
3. **A change set acts on the linked recipient.** Rename and Remove act on each address of the wallet. Change address replaces each address of the wallet with the new address.
4. **A new operation replaces the node address with the Lightning address** in the selected feeds. The core writes `type="lnaddress"`, and removes the custom key and the custom value. As MSP 2.0 does, it sets `method="lnaddress"` on each block that then has an `lnaddress` recipient. This needs a decision on the method of a mixed block.
5. **The editor tells the user** when a Lightning address has no keysend lookup, because some apps then cannot pay it.

The recommended start is options 1, 3 and 4. Option 2 can wait for a user who needs it. Option 5 is small, and can go with option 1.

## Open Questions

- Do Fountain, Podverse and the other apps pay an `lnaddress` recipient in a block with `method="keysend"`? The specification permits the mix. This research found no test of it.
- Must Studio change `method` when a block gets its first `lnaddress` recipient? MSP 2.0 does. The specification example does, but it does not state a rule.
- How long can Studio keep a cached keysend result? The specification says to cache, and gives no time.

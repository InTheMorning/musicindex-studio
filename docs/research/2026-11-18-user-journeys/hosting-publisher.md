# Journeys of a publisher who hosts other artists

Date: 2026-11-22. Status: research record. This record is advisory. It states no rule and gives no design.

The main record (`README.md` to `analysis.md`) studies a new independent artist at commit `73828c0`.
This page adds a second type of user. The evidence is the tutorial for a hosting user of 2026-11-22
and the findings of its walks (`docs/reviews/2026-11-22-tj-tutorial-findings.md`).
The step counts use the code at commit `66a1a64` (`0.1.0-rc.32`).

## The user

- The user hosts the feeds of more than one artist on one web host. Each artist has a publisher feed
  (an "artist feed") that lists its releases.
- All the feeds are public already. The user knows RSS feeds and the web host. The user does not
  know the terms of the Podcasting 2.0 tags.
- The goals are work on many feeds at a time: change the payees to Lightning addresses, and put the
  artists below one hosting page.
- The example data has 3 artist feeds, about 39 releases, 2 sites and 5 folders.

## Method

The step method and the evidence marks are the same as in `README.md`.

- The tutorial walks ran Studio on a copy of the public feeds of the two sites. The copies are in
  `.studio-dev/tj-tutorial` and `.studio-dev/v-230/data-tut` (ignored folders). No feed went to a server.
- The tutorial screenshots of 2026-11-22 have the prefix `h-` in `shots/`. They come from the binary
  of that date.
- On 2026-11-22, the main session made these shots again with the binary of `66a1a64`:
  `h-g24-review`, `h-g26-repaired`, `h-g27-elijah` and `h-g28-allinaday` (the move),
  `h-b07-fingerprint` and `h-b08-vault` (the SFTP form, with a local SSH server).

## Step counts

| ID | Journey | Start | Steps | Source |
|---|---|---|---|---|
| H1 | First run to the feeds of a publisher, addresses known | Empty data | 6 | Shot, `h-a02-addresses`, `h-a04-found-end`, and J01 for the library 2 |
| H2 | Connect one SFTP site with a password | Shell | 11 | Shot, `h-b04-sftp-bottom`, `h-b07-fingerprint`. Code, `app.js:945` |
| H2a | The same, the first time: Studio makes the vault | Shell | 16 | Shot, `h-b08-vault`. Code, `app.js:744` |
| H3 | Change one payee in all its feeds, with the address that Studio finds | Shell | 7 | Shot, `h-c06-wallet`, `h-c08-use-sheet-6`, `h-c14-review` |
| H3a | The same, with an address that the user types | Shell | 8 | Shot, `h-f06-dv-typed`. **Inference:** "LNURL Address" is the default type (ADR 0217) |
| H4 | Change each payee that pays a node | Shell | 7 × k | **Inference.** No journey does more than one payee |
| H5 | Make sure that no payee pays a node | Shell | 2, then read each row | **Inference** from the tutorial, step 7. No shot |
| H6 | Make a hosting page | Release list | 5 | Shot, `h-g20-add-menu`, `h-g21-hosting-form` |
| H7 | Move the artists to the hosting page, and write | Hosting page | 4, or 5 with the address repair | Shot, `h-g23-gather`, `h-g24-review`, `h-g26-repaired` |

Notes on the counts:

- **H2:** File hosts, Add a File Host, the type, Server, User name, Folder on the server, Public address,
  Login, Password, Connect and Accept and Add the Server. The name is optional. When the folder has feeds
  that the collection does not have, **Add N to Studio** adds 1 step.
- **H2a:** the vault adds the passphrase, the passphrase again, Make the Vault, the check box of the
  recovery key and Continue.
- **H7:** before ADR 0243, the user opened one review for each of about 38 feeds (finding 1).
  ADR 0243 makes the move one change set with one write.
- **The full task** of the example user is about 45 + 7 × k steps: H1 6, H2a and H2 for two sites 27,
  H5 2, H6 5 and H7 5. k is the number of payees that pay a node. The tutorial names a minimum of three
  payee groups. The data does not give k.

## Observations

**P1. Studio changes one payee at a time.** Inference, tutorial step 6.
The user does H3 again for each payee. No journey changes all the payees that pay a node.

**P2. No check tells that the task is complete.** Inference, tutorial step 7.
The user reads the rows of "Payees with no person" for the word "node". The list has a search, and no
filter for a payment type.

**P3. The order of the work comes only from the tutorial.** Unsure.
The tutorial tells the user to change the payees before the move, "so that each feed has one change at a
time". Studio does not tell this. The main session did not do the move first.

**P4. Change Address does not check the wallet.** Shot `h-f06-dv-typed`.
"Use the LNURL Address" accepts an address only when it pays the same wallet. "Change Address" accepts
the typed address with no such check. The tutorial gives a warning for it.

**P5. The SFTP form has two hidden stages.** Shot `h-b07-fingerprint`, `h-b08-vault`. Code.
The default Login is "Private key" (`app.js:948`). Many users of a web host have a password.

After Connect, Studio shows the fingerprint of the server key. The first secret opens "Protect your keys".
Version 4 of the tutorial did not name these stages. Version 6 names them.

**P6. Real feeds have faults that the test data did not have.** Shot, findings 3, 9, 10 and 13.
The faults are WordPress folders below the scan depth, links with no `medium` and addresses with no
`https://`. Some links also have a feed GUID that the linked feed does not use now.

Walk `h-g26-repaired` found a second such link,
"Them" in the feed of The Doerfels (`75b2ec36…` to `b62cd990…`). The move corrects each such link
(ADR 0247 item 9). Walk `h-g28-allinaday` shows the correction in a `podcast:podroll`.

**P7. Review uses the terms of the feed standard.** Shot `h-g26-repaired`, `h-g28-allinaday`.
Examples: "22 entries get the relation artist", "4 entries get the medium music", "Feed GUID",
"Podroll", and "Artist or label" with "Remote items: Removed" and two rows "Remote items: Added".
The record theme T8 applies to this user too.

## Comparison with the themes of `analysis.md`

This section is an opinion of the main session. It is advisory.

| Theme | Effect on this user |
|---|---|
| T1 A written release is not public | Small. The feeds are public already. |
| T2 Payment setup has no guide and no check | Some. P4 is a missing check. The payee batch is the primary path. |
| T3 and T6 Merges in People | Small on this path. People is the place of the payee batch. |
| T4 Review stops for a missing artist page | Replaced by the move of H7. |
| T5 Errors do not tell how to recover | Large. Findings 2, 3 and 11 were of this type. ADR 0243 and 0245 changed them. |
| T8 Terms with no explanation | Large. P7. |

The themes do not have P1, P2 and P3. Studio has no journey for the full goal of this user. It has
journeys only for the parts of the goal.

## Limits

- No real user did these journeys. The tutorial is a prediction of the steps until the user runs it.
- The SFTP walk used a local SSH server. The add after the vault and the folder scan of a real host are
  code reading and earlier tutorial shots.
- The counts of H4 and H5 are inferences. No walk changed all the payees of the example data.

# Studio v1 Task 128: The Wallets Of Other Recipients Are In A Fold

Decision: [ADR 0117](../adr/0117-the-page-of-a-person-finds-values-on-musicindex.md) item 15.

Status: Complete on 2026-11-12. The mechanical checks and the screenshot visual check passed. The human check passed on 2026-11-12. For the real artist Prosperous Soul, the Fill section showed only the matching wallet, and the closed fold "2 other recipients on Prosperous Soul" held the other two wallets.

## Goal

The sheet "From Music Index: {title}" shows one full row for each wallet of the value block. The app fees, for example `podcastindex@getalby.com` and the wallet of MSP 2.0, have the same weight as the wallet of the person. The page of a person puts these wallets in a closed fold (ADR 0159 item 15).

After this task, the sheet uses the same match. A wallet that matches the person or the title of the feed stays a row of "Fill". Each other wallet is a row of a closed fold "N other recipients on {title}", below the wallets that match.

## Work

- `crates/core/src/people.rs`, `page_wallet_matches`: make the name test usable by `crates/core/src/lookup.rs`. Keep one implementation of the test (ADR 0159 item 13). Do not copy it.
- `crates/core/src/lookup.rs`, `compose_values` and `WalletRow`: add a field `matches: bool` to each wallet row. It is true when the recipient name matches the name of the person, a credit name of the person or the title of the feed.
- `crates/api/ui/people.js`, `fillRows`: give each wallet row a value `other: true` when `wallet.matches` is not true. Add a pure function that splits the rows into the main rows and the fold rows, and gives the fold title "1 other recipient on {title}" or "N other recipients on {title}". Use the words of `pageSuggestions` for the title.
- `crates/api/ui/peoplepages.js`, `ValuesForm`: show the main rows in the group of "Fill". Show the fold rows in a closed fold below the wallets that match, inside the section "Fill". Use the existing fold component of this file (`PickFold` or the fold of the page suggestions), so that the fold looks the same as the fold of the page of a person. A row in the fold keeps its check box and `FillRow`. The Nostr key row stays after the fold.
- The count of the primary button (`linkFillButton`) counts a selected row in the fold.

## Checks

Mechanical:

- A core test in `crates/core/src/lookup.rs`: a value block with the routes "Ann Bell" (matches the person), "Ann Bell Music" (matches the title) and "Podcastindex.org" gives `matches` true, true and false.
- A Node test in `crates/api/ui-tests/people.test.mjs`: the split function puts the wallet with `matches: false` in the fold, keeps the wallet with `matches: true` in the main rows, and gives the title "1 other recipient on Ann Bell Music".
- A Node test: a selected row in the fold counts in the label of the primary button.
- The four checks of `AGENTS.md` pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `tools/node --test crates/api/ui-tests/*.test.mjs`.

Visual, by the main session: the sheet with one matching wallet and two other wallets, at 1280 px and 390 px, in the two themes. The fold is closed at the start.

Human, by the operator: in the desktop app, "Find on Music Index" for a real artist with app fees in the value block shows the app fees only in the closed fold.

## Stop And Ask When

- The match needs a pending credit name that is not saved.
- The fold needs a new component.

# Studio V1 Task 101: Wallets Of A Linked Page

Status: Complete, with its screenshot visual check on 2026-11-11. The screenshots used a page from a fixed answer, not from Music Index. The human test with the page of Matt Finlay is open. A recipient with no name goes to the fold.

Owner of the rules: [ADR 0159](../adr/0159-a-person-can-link-to-a-page-on-music-index.md) items 13 to 15. Evidence: finding 5 of the [rc.21 findings](../reviews/2026-11-10-rc21-human-test-findings.md).

## Goal

The page of a person suggests only the wallets of a linked page whose recipient name matches the person. The other recipients are in a closed fold.

## Files To Inspect

- `crates/core/src/people.rs`: `PageWallet` (already has `name`), `PageSuggestion` and `page_suggestions` (around lines 3140 to 3220). The test `page_suggestions_give_only_the_values_that_differ` is near line 4915.
- `crates/core/src/people.rs`: `Person` with `name`, `credit_names` and `pages`, and `PersonPage` with `title`.
- `crates/api/src/people.rs`: the route `page_suggestions` (around line 150).
- `crates/api/src/tests.rs`: `the_page_suggestions_give_the_differences_of_each_read_only_page` (around line 6756).
- `crates/api/ui/people.js`: `pageSuggestions` (around line 1054).
- `crates/api/ui/peoplepages.js`: the page of a person, `pageRow` and `fromPages` (around line 846), and the use of `PickFold`.
- `crates/api/ui-tests/people.test.mjs`: the tests of `pageSuggestions`.

## Constraints

- No feed rule in the UI (ADR 0065).
- UI files are in `crates/api/ui/*.js` (Preact + htm). `node --check` misses template errors, so check a `.mjs` copy of each changed UI file.
- The core decides if a wallet matches. The UI shows the split that the core gives and has no copy of the test of ADR 0159 item 13.
- A wallet that the person has already is not a suggestion, as before.
- The fold uses the existing fold component of the People pages. No new fold style.

## Acceptance Criteria

Mechanical, with Rust tests in `studio-core`:

- A wallet with the name of the person matches.
- A wallet with a credit name of the person matches.
- A wallet with the title of the page matches.
- The test ignores case, spaces and punctuation.
- A wallet with another name does not match, and the core gives it as another recipient of the page.
- A wallet that the person has already is in neither list.
- A recipient with no name does not match.

Mechanical, with a Rust test in `crates/api`: the route gives the matching wallets in one list. It gives the other recipients in a second list. Each entry has the name of its recipient.

Mechanical, with Node tests: the model of `pageSuggestions` gives the matching wallets with the name of the recipient and the title of the page. It gives the other recipients as a fold with the text "N other recipients on TITLE", and each row has **Use**.

Visual, by the agent, with screenshots by `tools/drive-ui.mjs` at 1100 px and at 390 px with touch, in the dark and the light theme. Run `$edges()` and `$overflow()` with each screenshot:

- The page of a person with a matching wallet and a fold of other recipients, closed and open.
- A page with no matching wallet and only the fold.

Human, by the operator: the page of Matt Finlay with the real link. It must not suggest `podcastindex@getalby.com` or `chadf@getalby.com` as matching wallets.

## Test Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node --test crates/api/ui-tests/*.test.mjs
```

## Escalation Triggers

- Music Index gives no recipient name for a route. The match then has no input. ADR 0159 does not say if such a wallet goes to the fold.
- The fold needs a closed or open state that the pending change of the person must keep.

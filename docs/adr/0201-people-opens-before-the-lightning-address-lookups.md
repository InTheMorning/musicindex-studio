# ADR 0201: People Opens Before The Lightning Address Lookups

- Status: Accepted
- Date: 2026-11-14
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-14 by the operator. The operator selected "Check after": People opens with the last known results, and the lookups run after that.
- Technical choices: the planning model selected items 3 to 9.
- Changes: [ADR 0095](0095-a-lightning-address-and-its-node-key-are-one-recipient.md) item 3.
- Uses: [ADR 0015](0015-connector-interface-and-write-pipeline.md) for the requests with no lock of the workspace. [ADR 0113](0113-the-page-of-a-person-suggests-a-picture-and-an-information-address.md) item 5 and [ADR 0200](0200-studio-deletes-a-feed-file-only-after-music-index-removes-the-entry.md) item 6 use the same pattern.

## Context

The People page opens slowly. On 2026-11-14 the planning model measured the requests of the page in the walk data (58 feeds):

- `GET /recipients` took 471 ms on the first call and 24 ms on the second call. The other four requests of the page took 1 ms to 6 ms each.
- `GET /recipients` sends a lookup for each Lightning address that is due (ADR 0095 item 3). An address is due when it has no result, a result older than one day, or a failed result older than one hour (`crates/core/src/keysend.rs`, `is_due`). Each lookup has a limit of 5 seconds. The first call sent 14 lookups. The second call sent none.
- The route sends the lookups while it holds the workspace lock. The other requests of the page wait for the lock, and the page shows nothing until all five answers arrive.

Thus the cost is the network lookups, not the parse of the feeds. The parse took about 20 ms. The walk data has no release folders, so the parse in a real collection can take longer. This is not measured (see Open Points).

## Decision

1. `GET /recipients` sends no lookup. It gives the list with the lookups that the collection already holds. This changes ADR 0095 item 3.
2. The People page shows the list of people before the answer of `GET /recipients`. The parts of the page that need the recipients show when that answer arrives.
3. A new route `POST /recipients/lookups` sends the lookups that are due. It finds the addresses with the lock, sends the lookups with no lock, and then keeps the results with the lock. The rules of ADR 0095 item 3 for each lookup do not change: the same addresses are due, the requests go at the same time, each has a limit of 5 seconds, and a failed request keeps the previous result.
4. The route finds its addresses from the same list that `GET /recipients` gives. Thus the list and the lookups cannot use different addresses.
5. The answer of the route is `{ "checked": n }`, where `n` is the count of addresses that it requested. With no due address, `n` is 0, and the route sends no request.
6. The People page and the page of a person send `POST /recipients/lookups` after the answer of `GET /recipients`. When `checked` is more than 0, the page reads `GET /recipients` again, and the wallet rows change. A failed request of the route changes nothing on the page and shows no error.
7. The lookup of one address in a payment operation (`check_lnaddress`, ADR 0095 items 11 to 13) does not change. The operation needs a fresh result before the core runs.
8. The lookups in the saves of a person and of the linked page changes do not change. They request only the addresses that the save adds.
9. With no keysend scheme (the API tests, ADR 0045), the route sends no request and gives `{ "checked": 0 }`.

## Alternatives Considered

- The page waits for the lookups, but the lookups run with no lock. Not chosen by the operator: the first open each day can still wait up to 5 seconds.
- Keep the lookups in `GET /recipients`, and change only the order of the page. Not chosen by the operator: other requests still wait for the network.
- Read the recipients from the table `feed_recipients`, not from each feed. Not selected: the measured parse is about 20 ms. The table also has no fee flag and no draft recipients, so the change needs a schema change.
- A timer thread that sends the lookups. Rejected: the lookups then run when no page needs them, and a test cannot control the time.

## Consequences

- For a short time after People opens, a wallet row can show a result that is up to one day old, or one hour old for a failed result.
- One more request goes from the page to the API.
- The People page has a short time where the list of people shows and the payees with no person are not yet on the page.

## Open Points

- The time of the parse of each feed in a real collection with release folders. Measure it in the desktop app of the operator before a decision about `feed_recipients`.

## How This Decision Is Checked

Mechanical:

- An API test with a keysend scheme and a test lookup server proves that `GET /recipients` sends no lookup, and that `POST /recipients/lookups` sends one request for each due address and gives `checked`. A second call of the route gives `{ "checked": 0 }`.
- An API test proves that the route does not hold the workspace lock while it waits for a lookup: a `GET /people` during a slow lookup answers before the lookup ends.
- A Node test proves that the People page reads the people with no wait for `/recipients`, and reads `/recipients` again only when `checked` is more than 0.
- The four commit checks pass.

Visual, by the planning model:

- The People page and the page of a person at 1280 px light and 390 px dark, after the lookups end.

Human, by the operator. This gate passed on 2026-11-14:

- In the desktop app, open People on the first open of a day. The list shows at once, with no wait for the network.

The operator opened People after an earlier open on the same day, so few lookups were due. The walk measured the case with due lookups: the list showed after about 10 ms, and the lookups took 0.3 s to 1.5 s after that.

# ADR 0202: An http Image Shows From Its https Address

- Status: Accepted
- Date: 2026-11-14
- Owner: operator, with the technical choices of the planning model
- Accepted: 2026-11-14 by the operator. The operator selected "Show via https": the UI shows the `https` form of each `http` image address, and the saved address does not change.
- Technical choices: the planning model selected items 2 to 6.
- Changes: [ADR 0109](0109-images-and-audio-load-from-their-hosts.md) item 4 for images. Audio does not change.

## Context

At the human check of ADR 0199 on 2026-11-14, the person "Strange Love" showed an empty circle for the picture `http://phafe.com/wp-content/uploads/strangelove/Album/strangelovepp.png`. The same picture shows in a usual browser.

- The page rule of Studio permits images only from `https:` addresses (`crates/api/src/ui.rs`, ADR 0109 item 1). In the walk, the browser refused the `http` address with the message `violates the following Content Security Policy directive: "img-src 'self' data: https:"`.
- `mediaSource` in `crates/api/ui/media.js` gives `{ kind: 'none', reason: 'http' }` for an `http` address, so `useImage` shows the empty art (ADR 0109 item 4).
- The address `https://phafe.com/wp-content/uploads/strangelove/Album/strangelovepp.png` gave `200` and `image/png` on 2026-11-14.
- Current browsers try `https` first for an `http` address. This is an inference: the planning model did not test the browser of the operator. The WebKit view of the desktop app does not do this.

## Decision

1. For an image, the UI shows the `https` form of an `http` address. The `https` form is the same address with `https://` in place of `http://`.
2. When the image does not load from the `https` form, the UI shows the empty art, as before.
3. The saved address, the text of the address in a row, and the feeds do not change. Studio does not write the `https` form.
4. A file that the server holds for the `https` form loads through Studio, as each held file does (ADR 0109 item 5).
5. Audio does not change. The player shows "This file uses http. Studio cannot play it here." (ADR 0109 item 4).
6. The page rule does not change. It permits no `http` image.

## Alternatives Considered

- Save the `https` form when Studio adds an `http` picture and the `https` form answers. Not chosen by the operator. It sends a network request at each save, and the value is then different from the source page.
- Add `upgrade-insecure-requests` to the page rule. Not chosen by the operator. The desktop page is on a loopback address with `http` (ADR 0109 item 8). The rule can also change the requests of the page to Studio, and the planning model did not test it in WebKit.
- Relay `http` images through Studio. Rejected by ADR 0109: Studio relays no file from a host.

## Consequences

- An `http` picture on a host that also has `https` shows in Studio.
- An `http` picture on a host with no `https` still shows the empty art.
- An app that reads the feed still gets the `http` address.

## How This Decision Is Checked

Mechanical:

- A Node test proves that the image source of an `http` address is the `direct` source of its `https` form, and that a held file of the `https` form gives the `held` source.
- A Node test proves that `mediaSource` still gives `reason: 'http'` for an `http` address, so the player keeps its notice.
- The four commit checks pass.

Visual, by the planning model:

- A person with an `http` picture shows the picture at 1280 px light, when the walk can reach the host. The walk browser has no network, so this check can be open.

Human, by the operator. This gate passed on 2026-11-14:

- In the desktop app, open the person "Strange Love". The picture shows, and its row still gives the `http` address.

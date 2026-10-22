# ADR 0025: Studio Serves Cover Art To The UI

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

On 2026-10-22 the operator asked that the album editor show the art of the album and of each track. The art is on the host of the artist: `itunes:image` gives its address.

The UI loads nothing from other servers, and a test enforces this ([ADR 0016](0016-ui-without-a-build-step.md)). The page policy permits images only from the Studio address and from `data:` addresses.

## Decision

1. The editor shows the `itunes:image` of the album and of each track. A track with no image shows the album image.
2. The UI loads each image from the local API, not from the host of the image. The core reads the image from its host and gives it to the API. The rule of ADR 0016 does not change.
3. The core reads only an `http` or `https` address, follows redirects, and stops at a size limit. It accepts only a response with an image media type.
4. When the core cannot read an image, the editor shows a placeholder and the address. The editor does not stop.

## Alternatives Considered

- Permit `https:` images in the page policy. Rejected by the operator: the rule of ADR 0016 then changes, and each host of art sees the browser of the user.

## Consequences

- The server shell must refuse image addresses in private networks, as it must for discovery (the Server Shell phase).
- An image that changes on its host shows the change only after the core reads it again.
- The task of this decision sets the size limit and the time that the core keeps an image.

## Invariants

- The UI loads no image from a server other than Studio.

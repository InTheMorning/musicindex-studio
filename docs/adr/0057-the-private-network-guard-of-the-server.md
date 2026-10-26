# ADR 0057: The Private-Network Guard Of The Server

- Status: Accepted
- Date: 2026-10-26
- Owner: operator

## Context

Studio reads many addresses that a user gives:

- Feed addresses in discovery, image addresses ([ADR 0025](0025-studio-serves-cover-art.md)) and audio addresses ([ADR 0023](0023-web-addresses-and-audio-file-facts.md)).
- Downloads ([ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md)).
- Relays, Blossom servers, S3 endpoints and SFTP hosts.

On a server, a user could thus make the server read an address in its own network. Examples are a cloud metadata address and an internal service. Earlier ADRs leave the refusal of these addresses to the Server Shell phase.

The code examination of 2026-10-26 found these facts:

- No outbound request checks the address after DNS. The clients resolve the names themselves.
- `studio_core::web::Web` has two `ureq` agents with 5 redirects each. Nothing checks the target of a redirect.
- The S3 client and the Blossom client are separate `ureq` agents with no redirects.
- The relay client and the SFTP client resolve with `to_socket_addrs` and open a `TcpStream`.
- `POST /connectors/fingerprint` connects to each host and port with no vault, so it can test which ports are open.

The PodSovereign server does its check in the DNS step, so that DNS rebinding cannot go around it. The source is `research_notes/studio/podsovereign-comparison.md` in `musicindex-research`.

On 2026-10-26 the operator selected the decisions below.

## Decision

1. The core has a network policy with two values: `Open` and `Public`. The desktop uses `Open`. The server uses `Public`.
2. With `Public`, Studio refuses a connection to an address that is not a global unicast address. This includes:
   - IPv4: `0.0.0.0/8`, `10.0.0.0/8`, `100.64.0.0/10`, `127.0.0.0/8`, `169.254.0.0/16`, `172.16.0.0/12`, `192.0.0.0/24`, `192.0.2.0/24`, `192.168.0.0/16`, `198.18.0.0/15`, `198.51.100.0/24`, `203.0.113.0/24`, `224.0.0.0/4`, `240.0.0.0/4` and `255.255.255.255`.
   - IPv6: `::/128`, `::1/128`, `fc00::/7`, `fe80::/10`, `ff00::/8`, `2001:db8::/32`, and an IPv4-mapped address or a `64:ff9b::/96` address with an IPv4 address of the list above.
3. The check occurs on the resolved address, at the time of the connection. Studio connects to the address that it checked, not to the name again. Thus DNS rebinding cannot go around it.
4. Each client of the core uses the policy:
   - The `ureq` agents of `Web`, S3 and Blossom get a resolver that removes each refused address. A name with no permitted address gives the error of item 6. A redirect goes through the same resolver.
   - The relay client and the SFTP client use one core function that resolves, checks and connects.
   - With `Public`, no client uses a proxy from the environment.
5. The policy applies to each port. An SFTP host on port 2222 of a public address is permitted.
6. A refused address gives the error `address_not_public` with status 422. The message names the host, not the resolved address.
7. A server test that uses a local test server sets an exception list of exact socket addresses. Only a test build can set it.

## Alternatives Considered

- The guard in both shells. Not chosen by the operator. A desktop user can have a NAS, a MinIO server or a Blossom server in the home network, and the user owns that network.
- A check of the name before the request. Rejected: DNS can give a different address at the time of the connection.
- An allow list of hosts. Rejected: a user can have a host at any public address.

## Consequences

- On a server, a user cannot connect a host in a private network. The runbook says this.
- The `ureq` resolver interface is in its `unversioned` module. A `ureq` update can change it. The guard tests find such a change.
- `POST /connectors/fingerprint` on a server can reach only public addresses.

## Verification

Mechanical:

- Unit tests refuse each range of item 2, and permit a global address.
- With `Public`, tests refuse each of these on a loopback address: a feed address, an image address, a redirect target, a relay, an S3 endpoint and an SFTP host.
- With `Open`, the same tests reach the local test servers.
- A test with a resolver that gives a public address and then a loopback address confirms that the connection uses the checked address.

## Invariants

- With `Public`, no outbound connection of Studio goes to an address that item 2 refuses.

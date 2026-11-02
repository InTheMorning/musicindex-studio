# Research: Host Secrets Under A Show Grant

Date: 2026-11-01. This note is advisory. It makes no rule. The owning decisions are [ADR 0054](../adr/0054-live-items-connected-apps-and-show-grants.md) and [ADR 0058](../adr/0058-grant-vaults-and-connected-apps-on-the-server.md).

## The Question

A show grant lets a connected app write the feed XML to a file host while the user is away. The security audit of 2026-11-01 asked if a grant must name one app (SEC-09). The operator then asked a wider question. A key that can write to a host is a large risk. How much power does a grant give, and can Studio decrease it?

## The Current Design

These facts come from the code on 2026-11-01.

- `apps::add_app` makes a token of 32 random bytes and shows it one time. The collection keeps the SHA-256 of the token and a public key. `apps::app_keys` derives the matching private key from the token with HKDF-SHA-256. Studio keeps no copy of the token.
- `/app/v1/` accepts only an app token, and `/api/` refuses it. An app can list its grants, change a live item from pending to live and from live to ended, and set or remove `podcast:liveValue` (ADR 0054 items 10 and 13).
- `grant_vault::make` copies the secrets of the host of the feed XML from the user vault into a new grant vault (`Workspace::grant_secret_ids`). The grant vault has one NIP-44 wrapper for each connected app with a key.
- `grant_vault::open` opens a grant vault with the token of one app. The granted write then uses the copied secret, and writes through `pipeline::write_draft`.
- A grant ends 24 hours after the end of its live item, or when the show ends (`grant::GRANT_MARGIN`). The removal of an app removes its wrappers.

The secret in a grant vault is the full secret that the user gave Studio for that host. Its power is the power of that credential on the host, not the power of the four operations of `/app/v1/`.

## What An Attacker Gets

| The attacker has | Result |
| --- | --- |
| An app token only | Through `/app/v1/`, until each grant ends: start or end a show, and set its live value. A new live value can send the boosts of the show to a different wallet. |
| An app token and a copy of the server disk | Each grant vault opens. The attacker gets the host secret of the feed XML, with its full power on that host, until the user changes it on the host. |
| A copy of the disk only | Nothing from the grant vaults. The token is not on the disk. |
| The server operator only | Nothing from the grant vaults, for the same reason. |

The second row is the large risk. Its size depends on the host credential.

## How Much Can Each Host Limit A Credential

| Connector | Can a credential write only one file? | Can it expire? | Source |
| --- | --- | --- | --- |
| Cloudflare R2 | Yes. A temporary credential has `objectPaths` and a permission such as `object-read-write`. | Yes. The API permits a maximum of 7 days. The local method has a default of 1 hour, and the page states no maximum. | [R2 temporary credentials guide](https://developers.cloudflare.com/r2/examples/authenticate-r2-temp-credentials/), [API reference](https://developers.cloudflare.com/api/resources/r2/subresources/temporary_credentials/methods/create) |
| AWS S3 | Yes, with STS `AssumeRole` and a session policy for one object. It needs an IAM role that the user makes. | Yes, from 15 minutes to the maximum of the role, at most 12 hours. | [Backblaze AssumeRole reference](https://www.backblaze.com/apidocs/assumerole), [IAM session policies notes](https://notes.kodekloud.com/docs/AWS-IAM/Introduction-to-AWS-Identity-and-Access-Management/IAM-Session-Policies). The official AWS page was not checked. |
| Other S3-compatible hosts | It depends on the host. Backblaze B2 has `AssumeRole`. | It depends on the host. | [Backblaze AssumeRole reference](https://www.backblaze.com/apidocs/assumerole) |
| Bunny Storage | No method was found. The storage zone password gives full access to the zone. A read-only password exists. | No. | [Bunny FTP docs](https://docs.bunny.net/storage/ftp), [read-only password API](https://docs.bunny.net/api-reference/core/storage-zone/reset-read-only-password.md) |
| SFTP | Only through the configuration of the server, for example an account in a chroot folder. Studio cannot set it. | No. | General knowledge of OpenSSH. Not checked for this note. |
| Blossom | An auth event (kind 24242) names the blob hashes and an expiry. The hash of a new feed XML is not known before the change, because the live value is free text. | Yes, but it cannot be signed in advance. | Implementation code only, for example [hashtree-cli](https://docs.rs/hashtree-cli/0.2.57/src/hashtree_cli/server/blossom.rs.html). The Blossom specification was not checked. |
| nsite | No. The site key signs each manifest. | No. | ADR 0048. NIP-26 delegation is marked unrecommended ([NIP-26 mirror](https://nips.4rs.nl/nips/26)). |

### The R2 Detail

The R2 guide gives a local method. Studio signs a JWT with the parent secret access key (HS256). The claims give the bucket, the permission, the object paths and the expiry. The derived credential has three parts. The access key ID is the parent access key ID. The secret is the SHA-256 of the JWT, and the session token holds the JWT. The method needs no Cloudflare API token. The R2 connector of Studio already has the parent access key ID and secret in the vault.

The approved write that records a grant occurs while the user vault is open. At that time, Studio could put a scoped credential into the grant vault. That credential can only read and write the one feed XML object, until the end of the grant. A stolen token and a copy of the disk would then give one file for a short time, not the full bucket.

This was not tested. The maximum lifetime of the local method is not stated. A live show that lasts longer than the lifetime needs a new credential, and the user vault can be locked at that time.

## Isolation Of The Feed XML

Each feed has its own host of the feed XML, and it can be different from the host of its audio files (ADR 0035 item 12, ADR 0038 item 16). A grant vault holds only the secret of the host of the feed XML (`Workspace::grant_secret_ids`). Thus the user can decrease the power of a grant on each host type, with no change of Studio:

- Bunny Storage: a separate storage zone that holds only the feed XML of the show. The zone password is the storage key of one zone ([Bunny HTTP storage docs](https://docs.bunny.net/storage/http.md)).
- S3 and R2: a separate bucket, with an access key for that bucket only.
- SFTP: a separate account whose folder holds only the feed XML.
- nsite: a separate site key for the site of the show feed.

A grant vault then opens only a host that holds one small file. The audio and the other feeds are on hosts that no grant vault holds. This was not tested with a real host.

## Options For Studio

1. **No change.** The grant vault keeps the full host secret. The runbook and the user guide tell the user to protect the token and the backups.
2. **Informed consent in the review sheet.** The grant check box tells the user the power of the credential. For example: "Live Publisher gets full write access to Bunny Storage until 8 November." The user then decides with the facts.
3. **Scoped credentials where the host can make them.** For R2, the grant vault holds a temporary credential for the one feed XML object, with the end of the grant as its expiry. AWS S3 can follow with STS, but it needs a role from the user. Each other host keeps the full secret, with option 2.
4. **A grant names its apps (SEC-09).** A token of one instance cannot act on a grant of a different instance. With one program, the live publisher, this matters only when the user connects many instances. It also stops a backup instance during a show, unless the grant names it too.
5. **A NIP-46 signer for nsite and Blossom.** The grant uses a remote signer with rules, and the server keeps no site key. Some NIP-46 libraries and signers permit a rule for each event kind, for example `sign_event:1` ([innis/nostr-nip46](https://packagist.org/packages/innis/nostr-nip46), [Bark](https://addons.mozilla.org/de/firefox/addon/bark-nostr/)). Amber and nsecbunkerd were not confirmed. A rule for one kind does not limit the signer to one site, so the site needs its own key. The signer must run during each show.

## Assessment

- Options 2 and 3 decrease the largest risk, the second row of the attacker table. Option 4 does not change that row.
- The isolation of the feed XML decreases the largest risk on each host type today. It needs only a separate host for the feed XML of each show feed.
- Option 3 is strong for R2 and possible for AWS S3. With the methods that this note found, it is not possible for Bunny Storage, SFTP and nsite.
- Option 2 costs little and applies to each host.

## Open Questions

- Which file hosts do the users of `studio.musicindex.org` use for their feed XML?
- What is the maximum lifetime of an R2 credential from the local JWT method? A test against R2 can answer this.
- Does the official AWS documentation confirm the session policy limits? SDK mirrors give 900 seconds to the maximum of the role, and a policy of at most 2048 bytes ([aws-sdk-core](https://gem.sh/gems/aws-sdk-core/v3.233.0/classes/Aws::STS::Types::AssumeRoleWithSAMLRequest)).
- What does the Blossom specification (BUD-11) require for the `x` tag? Implementations bind an upload token to the hash of the file ([hzrd149/blossom](https://github.com/hzrd149/blossom)). Then a token cannot be signed before the new feed XML exists.
- Do Amber and nsecbunkerd permit a rule for one event kind?

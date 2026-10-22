# Studio Baseline Research

Date: 2026-10-22.
Status: advisory. This note states no binding rule.
A rule becomes binding only through an ADR in `docs/adr/`.

This note collects the research for the first design of musicindex-studio.
It gives the source of each observation.
Observations come from files. Inferences are marked "(inference)".

## Purpose Of The App

musicindex-studio replaces MSP 2.0.
An artist uses it to edit hosted Podcasting 2.0 music feeds directly.
The app keeps the full collection of feeds for each user.
It also keeps the credentials or the method that writes each feed.

## Sources

| Source | Path | State examined |
| --- | --- | --- |
| Music feed research | `../musicindex-research/reports/2026-10-20-music-feed-generator-research.md` and `research_notes/music-feeds/` | 2026-10-20 |
| MSP 2.0 fork | `../MSP-2.0` | Branch `publisher-parties`, commit `09f3cf9` |
| MSP 2.0 upstream pin | `../musicindex-research/sources/music-feeds/manifest.json` | Commit `6e16435` |
| Prior studio project | `../mi-studio` | Last commit `cc0b262`, 2026-05-25 |
| Public site aesthetic | `../musicindex/docs/adr/0001`, `0003`, `0012`, and `search.html` | Baseline commit `71c62f0` |
| Stophammer API | `../musicindex/api.json` | Version 0.13.2 |

The curiohoster architecture report in the research repository is private.
Do not copy its text into a public document.

## MSP 2.0

### Load Paths

- Import modes: file, paste, URL, Nostr, Nostr music, Nostr event and hosted (`src/components/modals/ImportModal.tsx:42`).
- A URL fetch tries a direct request first. Then it uses `/api/proxy-feed` (`src/utils/xmlParser.ts:854`). The proxy refuses a body larger than 4 MB.
- An import and a template import make new GUIDs (`src/utils/regenerateGuids.ts`).

### Save Paths

`src/components/modals/SaveModal.tsx:43` lists nine destinations:

- local storage, download and clipboard
- Nostr kind 30054 and Nostr music kinds 36787 and 34139
- Blossom upload with a kind 1063 pointer
- nsite (NIP-5A kind 35128)
- MSP hosting on Vercel Blob, with the last 10 backups on the server
- Podcast Index notification and submission

Podping goes through a self-hosted hivepinger with a bearer token (`api/podping.ts`).
A hosted create or update also sends a Podcast Index notification and a podping.

### Authentication

A hosted write accepts one of three credentials (`api/hosted/[feedId].ts:49`):

1. An edit token in `X-Edit-Token`. The server keeps a hash of it.
2. A NIP-98 `Authorization: Nostr` header.
3. An email magic-link session in `X-Email-Session`.

The browser keeps edit tokens in localStorage under `msp2-hosted-{guid}` (`src/utils/storage.ts:153`).

### Known Defects

- NIP-98 is not bound to a URL or a method, and it has no nonce (`CLAUDE.md:123`).
- `POST /api/hosted` has no authentication and no limit. Because the feed ID is the podcast GUID, a third party can claim a GUID first.
- A new feed gets a UUIDv4 `podcastGuid` (`src/types/feed.ts:429`). The namespace derives UUIDv5 from the feed URL.
- One lnaddress recipient changes `method` for the full value block (`src/utils/xmlGenerator.ts:224`).
- An item with more than one transcript loses data. Album feeds have no `atom:link rel="self"`.
- MSP 2.0 does not model `podcast:valueTimeSplit`. The parser keeps it only as an unknown element.
- (inference) MSP 2.0 keeps one draft for each mode in the browser. It has no collection model. It can write only to its own host, Blossom and nsite.

## Podtunes

The research lists these defects that an editor must not have:

- A change of the feed URL changes the feed GUID. No field keeps an existing GUID.
- Track GUIDs are random and live only in browser storage. A rebuild makes new items.
- `customKey` and `customValue` are on `<podcast:value>`, not on each recipient.
- The value block is only at channel level, and `method` is always `keysend`.
- The tool promotes WAV files, but Apple accepts only MP3 and AAC.
- The tool cannot import a feed or export a project.
- Podtunes writes no file to a host. It gives a `feed.xml` download and submits the URL to Podcast Index.

## Hosting Platforms

| Platform | Write mechanism | Credential | Evidence |
| --- | --- | --- | --- |
| MSP hosting | HTTP POST and PUT of the full XML | Edit token, NIP-98 or email session | Observed in `api/hosted/` |
| Blossom (BUD-01) | Upload by content hash. Each upload has a new hash. A Nostr event gives a stable pointer. | Nostr kind 24242 | Observed in `src/utils/blossom.ts` |
| nsite (NIP-5A) | Blossom upload and a kind 35128 manifest | Nostr | Observed in `src/utils/nsite.ts` |
| Sovereign Feeds and curiohoster | Multipart POST of `feed.xml` to `/api/sf/upload` | Session cookie from an Alby login | Observed in the Sovereign Feeds source |
| Castopod | No raw feed upload. The database makes the feed. The REST API adds episodes but cannot update a podcast. | Optional HTTP Basic for the instance | Observed in the Castopod source |
| Wavlake | Not found | Unknown | Research gap. Wavlake hosts approximately 88% of the index. |
| Fountain for Artists | Hosted service with no public source | Unknown | Research gap |
| RSS Blue, Podhome, Buzzsprout, Transistor | Not examined | Unknown | Research gap |
| GitHub Pages, S3, Cloudflare R2, FTP, SFTP, IPFS | Not examined | Unknown | Research gap |

(inference) A connector for a platform in a gap row needs research before a design treats the platform as writable.
(inference) A host that makes the feed from its own database, such as Castopod or Wavlake, cannot accept a full feed file. A connector for it must change records through the host API, or it must stay read-only.

## Feed Elements

The gaps in MSP 2.0 and Podtunes give this list (inference):

- Identity: an editable and preserved `podcast:guid`, with UUIDv5 for a new feed. Stable item GUIDs that the collection keeps. `podcast:locked`, `atom:link rel="self"`, `podcast:previousUrl` and `itunes:new-feed-url`.
- Payments: `podcast:value` at channel and item level, with the correct `method` for each block. Each `valueRecipient` has its own `customKey`, `customValue` and `fee`. A modeled `podcast:valueTimeSplit` with `remoteItem` and `remotePercentage`.
- Links: `podcast:medium`, `podcast:publisher` with more than one `remoteItem` and the `rel` role set from Podcast Namespace PR #793, publisher feeds and podroll.
- Content: `enclosure` with the correct byte length, `itunes:duration`, `podcast:person`, `podcast:funding`, `podcast:image`, more than one `podcast:transcript`, `podcast:license`, `podcast:txt` and `itunes:*`.
- Round trip: keep each unknown element. Do not both model an element and pass it through.

## Prior Studio Project (`../mi-studio`)

mi-studio is a Rust identity and value resolver with a studio added later.
It has 73 commits. The last commit is from 2026-05-25.

- Stack: axum, Askama templates, plain CSS, minimal JavaScript and one SQLite file (ADR 0006, ADR 0009).
- Login: Sign in with Nostr through NIP-98 and NIP-07, with a server session, CSRF and a single-use challenge (ADR 0012).
- Studio: a curated music draft schema, a deterministic RSS writer, draft validation, MSP import and immutable export snapshots (ADR 0015 to ADR 0017).
- UI governance: five owners (tokens, atom partials, composed partials, screens and view models) and a regression guard for each UI change.
- Prohibitions that conflict with the new purpose: ADR 0014 forbids stored provider credentials and hosting automation. ADR 0016 forbids stored MSP edit tokens.
- A review on 2026-05-26 found no publish target abstraction and no provider adapter.
- Two route and view-model files are larger than 3,900 lines.

(inference) These parts can apply again:

- the stack, the domain and UI split, and the login design
- the draft schema, the RSS writer, and the import with warnings for unknown fields
- the snapshot history and the UI governance
(inference) The resolver parts do not apply: claims, replay, conflicts, trust labels and the relay. Stophammer also removed its resolver (Stophammer ADR 0032).

## Stophammer API

- Lookup routes exist for a feed GUID, a track, a search, feeds by name, artist names, publishers, route history and copies.
- No route finds feeds by owner, npub or feed URL. A first-run collection import cannot get the feeds of one artist from Stophammer.
- Stophammer has no podping route for an editor. The crawler reads podpings. An editor must send its own podping.
- Stophammer follows a feed move only through a 301 or 308 redirect, `itunes:new-feed-url` or `atom:link rel="self"` (Stophammer ADR 0052).

## Aesthetic Direction

The public site uses the "Afterglow" direction (musicindex ADR 0003).

- Dark theme by default, with a light theme. Apple-like neutral surfaces. Blue accent `#2d7bff` in dark and `#0a5bd6` in light.
- Figtree from `assets/fonts/`, with the system font stack first. Large heavy hero titles and small uppercase eyebrows.
- A sidebar with search and results. A detail pane with a blurred bar and a hero with a glow from the art and film grain.
- Entity colors (`--c-*`) for feed, track, playlist, live, creator roles and business roles. An entity color is never a general accent. Red means only "live now".
- The logo mark has one color.
- Controls are at least 44 px high. All functions work from the keyboard. Empty, load and error states have a design.

The one-color mark, the brand assets, Figtree and the blue accent apply to the full brand.
The layout, the hero and the entity palette are scoped to `search.html`. The studio can adopt them by its own ADR.
The single-file rule without a framework is also scoped to `search.html`.

## Open Questions

The operator answered the first questions on 2026-10-22.
ADR 0001 to ADR 0011 record the answers.
The [Studio v1 phase plan](../plans/studio-v1-phase-plan.md) lists the questions that stay open.

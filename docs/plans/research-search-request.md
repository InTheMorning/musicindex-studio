# Research Request: Search, Shared Code And PodSovereign

## Status

Received - 2026-10-23. This request comes from the research repository at
`../musicindex-research`. That repository has no Git remote, and it is private.
Its index is `AGENTS.md`.

This document binds nothing in Studio. Studio records each decision in its own
ADR.

The research examined Studio at commit `a834386`.

Stophammer release 0.14.0 is deployed on 2026-10-24. Section 4 gives the
changes for Studio.

## 1. Search Findings

The report is `reports/2026-10-23-search-code-sharing-and-stophammer.md` in the
research repository.

| # | Result | Location |
|---|---|---|
| R1 | A musicindex request holds the workspace lock for a maximum of 20 s, so the other API requests wait. | `crates/api/src/routes.rs:34-45`, `crates/core/src/web.rs:94-104` |
| R2 | A decode error of a musicindex response gives an empty list or `None`, with no error. | `crates/core/src/musicindex.rs` |
| R3 | HTTP 429 and the other upstream errors become `502 network`. | `crates/api/src/error.rs:153-168` |
| R4 | The search sends `type=feed&limit=25` and reads no cursor. | `crates/core/src/musicindex.rs:259` |

The research also sent a request to Stophammer:
`docs/plans/research-search-index-request.md` in the Stophammer repository.
For Studio, the important items are these:

- The search index keeps the words that a title had before an edit. The first
  run and the recipient picker can find a feed by a title that it no longer
  has.
- An apostrophe in `q` gives HTTP 400.

## 2. Shared Search Code

The website, v4vmm and Studio share no search code. The report proposes:

1. A small crate in the Stophammer workspace with the response structs, the
   envelope, the `include` names and the limits, with no HTTP code. Studio
   keeps `ureq`. The crate must build with Rust 1.93.
2. A query crate from the v4vmm parser of v4vmm ADR 0087, with fields such as
   `artist:` and `title:`. The recipient picker and the first run can use it
   when Stophammer accepts the filters.
3. The Studio UI and the website use JavaScript with no build step. They can
   share one file of small functions. The benefit is small.

Stophammer and Studio are AGPL-3.0-only. v4vmm and the website have no license.
The operator selects these licenses before the code is shared.

## 3. PodSovereign

The research compared PodSovereign, a self-hosted podcast host, with Studio:
`research_notes/studio/podsovereign-comparison.md` in the research repository.

- PodSovereign makes each feed from its database. Studio cannot write a feed
  file to it, as with Castopod. A connector must use its JSON API with a
  session cookie, and it needs a Studio ADR (ADR 0006 item 5).
- Its Podping method needs no key: `GET
  https://hub.podcastindex.org/pubnotify?url=<feed>&podping=1`. This is a
  possible answer to the open question of ADR 0007 for Phase 10.
- Its address guard checks each address in the DNS lookup hook. This is a
  possible method for the private network risk of Phase 12.
- Its feed move checklist checks each week that the previous host gives a 301 redirect.
  This is a possible method for ADR 0035 items 19 and 20.

The research also keeps a map of the Studio design:
`research_notes/studio/studio-blueprint.html`.

## 4. Stophammer Release 0.14.0

Stophammer deployed release 0.14.0 on 2026-10-24. On that day, `GET
/node/info` gave `git_revision` `499b301`, and `info.version` of
`/openapi.json` gave `0.14.0`. The release delivers Stophammer ADR 0078 and
ADR 0079.

- The search index holds only the text that each entity has now (ADR 0078 §1).
  This corrects the defect of section 1. The first
  run and the recipient picker no longer find a feed by a title that it no
  longer has.
- A retire deletes each index row of the feed. A publisher hit gives the
  quality of its feed (ADR 0078 §3 and §4).
- The node divides `q` into words, and it puts each word in quotation marks
  (ADR 0078 §5). Each `q` with a word gives `200`, also with an apostrophe.
  A `q` with no word gives `400`, and a `400` message gives no SQLite text.
- A GUID that a retire with no block frees returns only through its last
  source URL, until an operator releases the hold (ADR 0079). The ingest gives
  the reason `retired_source_conflict`, and the admin route
  `DELETE /v1/retired-source-holds/{guid}` is new. Studio
  sends no ingest request. A feed that moves to a new URL after a retire needs
  the operator release (ADR 0079, Consequences). This concerns the feed move
  of Studio ADR 0035.

Read-only checks on the live API on 2026-10-24:
`GET /v1/search?q=haleen'&type=track` gave `200` with 3 rows, and
`q=title:foo` gave `200` with no rows. The OpenAPI document lists
`/v1/retired-source-holds/{guid}`.

These items stay open for Studio:

- R1 to R4 need no Stophammer change. The release does not change them.
- `../musicindex/api.json` gives the version `0.13.2` until the website
  regenerates it.
- The release has no prefix search, no cursor change and no query filters.

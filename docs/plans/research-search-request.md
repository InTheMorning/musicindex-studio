# Research Request: Search, Shared Code And PodSovereign

## Status

Received - 2026-10-23. This request comes from the research repository at
`../musicindex-research`. That repository has no Git remote, and it is private.
Its index is `AGENTS.md`.

This document binds nothing in Studio. Studio records each decision in its own
ADR.

The research examined Studio at commit `a834386`.

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

# Research Request: The Shell Of Studio

## Status

Received - 2026-10-24. This request comes from the research repository at
`~/build/musicindex-research`. That repository has no Git remote. Its index is
`AGENTS.md`.

This document binds nothing in Studio. Studio records each decision in its own
ADR.

**Wait before an ADR.** On 2026-10-24 the operator decided that Studio writes
no ADR from this request yet. First the operator fixes the research mockups.
Then a musicindex ADR codifies the shared rules. Read the request now. Do not draft
an ADR yet.

The research examined Studio at commit `563cce6`. The mockups are research
commits `6bacd1c`, `4fedc27` and `8d02b00`.

## The Data

| Data | Path from this repository |
| --- | --- |
| The shell analysis and the decision of the operator | `../musicindex-research/research_notes/ui-design/shell-layout-analysis.md` |
| The Studio mockup | `../musicindex-research/research_notes/ui-design/mockups/studio.html`, with `mockups/studio/` |
| The shared kit of the mockups | `../musicindex-research/research_notes/ui-design/mockups/kit/` |
| The decisions that the mockups show | `../musicindex-research/research_notes/ui-design/mockups/README.md` |
| The journeys of Studio, with their open questions | `../musicindex-research/research_notes/ui-design/journeys/studio/` |
| The cross-app rules | `../musicindex/docs/adr/0020-cross-app-interface-rules.md` to `0026-cross-app-controls-and-feedback.md` |

## 1. The Shell Is A Stack With The Outline Exception

The operator selected this shell for the three apps on 2026-10-24. It changes
Studio ADR 0029 items 5, 6 and 15 (the tree in the sidebar and the drawer).

- With no feed open, the left panel holds the destinations: Releases, Add a
  feed, Hosts and Lock Studio.
- **Releases, the collection, is a list page at full width.** Each row shows
  the cover, the type label, the host and the draft state, for example "Saved,
  not written". It is the start page. An empty collection starts the first-run
  wizard.
- **"Select a feed" goes.** No page is a placeholder.
- **Exception 1: while a feed is open, the left panel shows the outline of
  that feed in place of the destinations**: a row "‹ Releases", the cover and
  the title, then Album, Credits and links, Value routes, Files, the tracks,
  "Add a track" and "Import files". The left panel is one panel at any time.
- Back leaves the editor and returns to Releases at the same scroll position,
  with the row of the feed marked.
- Studio has no player, so no right panel shows.
- At 390 px, the outline opens as a drawer from the editor bar. The open
  question O-8 of the Studio journeys covers the drawer against a tab bar.

## 2. Value Routes

Value routes keep their width of 560 px at most. A share segment and a
percentage show a tooltip at once, with the name, the percentage and the raw
share.

## 3. Open Questions For The Operator

The operator has not decided these:

1. The open questions O-1 to O-16 of the Studio journeys. Examples are the
   noun for an item of the collection (O-1), and the choice after a write
   conflict (O-6).
2. A search of the collection (O-9). The website and v4vmm put one wide search
   field in the toolbar. No decision covers Studio.

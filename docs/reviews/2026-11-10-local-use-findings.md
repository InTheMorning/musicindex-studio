# Findings Of Local Use On 2026-11-10

Date: 2026-11-10.

Status: advisory, and open. This review records the problems that the operator found during the human check of ADR 0162. It states no rule. A rule needs an ADR.

The operator ran `cargo run -p studio-desktop` from local `main` at `55b4c77`, on a copy of the real data, in a desktop browser.

## Findings

### 1. A Live Show Cannot Be Removed

Open. The **Live Shows** tab lists each show, but no page has an action that removes a show. ADR 0054 item 3 gives only the edits of a show, and no ADR decides a removal. A test show stays in the draft.

Next: mockups for the operator, then an ADR.

### 2. The Date Fields Have No OK Button

Open. **Start time** and **End time** open the date picker of the browser. The picker has no **OK** button, so the user does not see how to confirm a value. **Enter** or a click outside the picker closes it, and Studio keeps the value.

The page cannot put a button into the picker of the browser. The two possible changes are a date picker of Studio with **OK** and **Cancel**, or an **OK** button adjacent to the field. A date picker of Studio changes the shared look of the three apps, so it needs a musicindex ADR first (musicindex ADR 0045 R-SYSTEM-1).

Next: mockups for the operator.

### 3. An Artist With Releases Shows "Add your first album"

Open. The page of the artist "HeyCitizen" shows the row **Add your first album** of ADR 0119 item 8. The left panel lists four releases of this artist.

`nextStep` in `crates/api/ui/welcome.js` shows the row when the feed has no `child_ids`. The left panel possibly uses `release_ids`, which the same feed row of `crates/api/src/routes.rs` gives. This cause is not proved. The row must show only for an artist with no release.

Next: find the list that the left panel uses, add a test with an artist that has releases, and correct `nextStep`.

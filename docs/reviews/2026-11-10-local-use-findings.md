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

Corrected on local `main` after 0.1.0-rc.21. The correction goes into the next candidate. The page of the artist "HeyCitizen" showed the row **Add your first album** of ADR 0119 item 8. The left panel listed four releases of this artist.

`nextStep` in `crates/api/ui/welcome.js` showed the row when the feed had no `child_ids`. But `child_ids` holds only the publisher feeds below the feed (ADR 0088). The releases are in `release_ids`. `nextStep` now examines both lists, and a test in `crates/api/ui-tests/welcome.test.mjs` gives an artist with a release.

### 4. The Field List Changes The Height Of A Sheet

Corrected on local `main` after 0.1.0-rc.21. The correction goes into the next candidate. In the sheet **Find on Music Index: {name}**, a click in the field opened the field list of ADR 0154. The list was in the flow below the field. Thus the sheet became taller, and the content below the field moved down.

After the correction, the list is a floating layer at the field, as on musicindex.org. It has fixed position, so the sheet does not cut it, and the sheet keeps its height. When the window has less space below the field, the list opens above it. The agent screenshots at 1280 px in the dark and the light theme, and at 390 px, show the list at the field. The sheet height does not change.

### 5. The Field List Of A Person Search Has No Album Field

Open. The field list of the person search gives only the artist field and the track fields. It gives no `release:` field.

ADR 0154 item 5 decides this. The search of the page of a person finds only artist and label feeds. The list leaves out each field that Music Index cannot use to find these feeds. A search for an artist by the title of an album needs a different search, and thus a new ADR.

Next: research of the Music Index API, then options for the operator.

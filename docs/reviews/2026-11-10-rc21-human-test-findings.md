# Human Test Findings Of 0.1.0-rc.21

Date: 2026-11-10.

Status: advisory. This review records the results of the human test of `0.1.0-rc.21`. It states no rule. The ADRs that it names own the decisions.

The operator ran the test on a Linux x86-64 computer with an NVIDIA GeForce GTX 750 Ti, with the AppImage of the draft release. The server checks ran in Firefox against `studio.musicindex.org` after a deploy of rc.21. The phone checks ran on an iPhone and in a narrow browser window.

## Results

The step numbers are the steps of "What to test" in the [notes of 0.1.0-rc.21](../releases/0.1.0-rc.21.md).

| Step | Check | Result |
|---|---|---|
| — | The draft release has its 15 assets | Passed |
| 1–4 | The upgrade of the data into `collections/main/` | Passed |
| 5 | The splits in **Where this person is** | Passed |
| 6–7 | A live HLS stream on the desktop | Passed |
| 8 | The type warning and **Use MP3** with a real Icecast mount | Passed, with finding 2 |
| 9 | A space before the stream address | Open. An agent check is planned. |
| 10–11 | A read-only link and its suggestions from a real page | Passed, with finding 5 |
| 12 | **Hide This Suggestion** and **Show Again** | Passed |
| 13 | The deploy, and the account stays | Passed |
| 14 | A stream address in a private network on the server | Refused, as expected |
| 15 | The stream check on the server | Passed, with finding 4 |
| 16–17 | The sort menus on the iPhone | Passed |
| 18–19 | The field list above the keyboard | Passed, with finding 7 |
| 20–21 | The time from Play to sound | Passed. About 1 second. |
| 22 | The tray | Passed. On 2026-11-11, the question showed at the first close with no `window-close` file. After **Quit on Close**, the next close quit with no question. |
| 23 | The first page with an NVIDIA card | Passed. The page showed immediately on the GTX 750 Ti. |
| 24–26 | A release move between two real file hosts | Open. The operator stopped before this step. |
| 27 | The mark "Changed outside Studio" | Open. Skipped. |
| 28 | A release with a real Nostr key | Picture and name passed. The page with the network off is open. |

Other checks:

- Runbook check B (WebKit on the AppImage): passed. The sort menu, the page tabs, the field list, the glow and the layout were correct.
- Runbook check F (the first-visit search): passed in an agent check on local `main` at `a18c831`, against the real index. Finding 1 comes from it.

## Findings

### 1. The Field List Stays Open After Enter

After Enter runs a search, the field list stays open and covers the first result until the field loses focus. The cause is in `crates/api/ui/pickers.js`: the list closes only on a blur or Escape. ADR 0154 item 8 does not say that the list closes. The correction is open.

### 2. The Type Line Uses The Server Type

The result line and the warning said "audio/mpeg", and the menu said "MP3". ADR 0162 item 10 now uses the menu label. The code is open.

### 3. A Paste In The Stream Address Field Scrolls The Page

A paste or a delete in the stream address field moves the page. A notice at the top of the form possibly shows and goes away. The cause is not verified.

### 4. A Check Does Not Fill An Empty Stream Type

On the server, the check answered `live: true`, `server_type: audio/mpeg` and `fill_type: audio/mpeg` in 366 ms. The line showed "Live · audio/mpeg", but **Stream type** stayed "Choose a type". The page fills the type only when the user did not select a type on the page (`crates/api/ui/editor.js`, `fillType`). An earlier selection counted, also when the type was empty. An empty type must get the fill. On the first try, no line showed at all. The cause of that first try is not known.

### 5. The Wallets Of A Linked Page Include Other Recipients

The page of Matt Finlay gave `podcastindex@getalby.com` and `chadf@getalby.com` as wallet suggestions. The core suggests each channel recipient of the page (`crates/core/src/people.rs`, `page_suggestion`). ADR 0159 items 13 to 15 now select the wallets with a matching name. The code is open.

### 6. A Closed Fold Does Not Draw Attention

On the People page, a closed fold such as "Payees with no person · 10" looks like a section heading. A comparison with v4vmm and musicindex.org and mockups come first.

### 7. The Find Sheet Moves Its Field

The sheet **Find on Music Index** sits at the bottom of the window, and its height follows its content. The field moved up about 280 px when the field list opened, and down again after each letter. The sheet also showed "No feed on Music Index has this name." while the field list was open, before the results came. The correction is open.

### 8. Find On Music Index For An Existing Credit

The operator asked for **Find on Music Index** in the sheet of an existing credit, not only of a new credit. This needs a check of its owning ADR first.

### 9. A Narrow Window Cuts Off The Bar Of A Credit Page

A screenshot of a narrow window showed **Review** and a red button cut off at the right edge, and a cut-off Nostr row. An agent check at that width is open.

## Operator Requests

- **Settings:** the operator asked for a Settings page with no long scroll, and with Save and Discard. ADR 0166 and musicindex ADR 0059 decide it. The code is open.
- **The release folder:** the operator asked for an editable local library folder for each release, and a default layout by artist and label. Mockups and an ADR come first.

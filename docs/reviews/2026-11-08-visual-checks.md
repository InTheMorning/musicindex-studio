# Visual Checks of 2026-11-08

- Date: 2026-11-08
- Build: `8a6d585` with the correction of the Releases sort menu in this commit
- Method: `tools/drive-ui.mjs` in headless Chromium 156, with Studio and Chromium in one command. Studio read the Music Index API on the internet. The screenshots are in `.studio-dev/v-1108/` on the computer of the check. Git does not keep them.
- Data: a copy of the check data of ADR 0159. It has one album with 19 tracks on a local folder host, the people "Ann Bell" and "MaryKateUltra", and one hidden suggestion.

Three subagents took the screenshots. The main agent looked at the screenshots that this record names, and gave each result.

## Results

| Gate | Result | What the screenshots show |
| --- | --- | --- |
| ADR 0152 | Pass | Each button, heading and note names "Music Index". The only lower-case form is the domain `musicindex.org`. 1280 px and 390 px with touch. |
| ADR 0153 | Pass | The People list in "A to Z", "Newest" and "Oldest" at 1280 px, and at 390 px with touch. The Files tab of the album lists 1.mp3, 2.mp3 ... 9.mp3, 10.mp3 ... 15.mp3. |
| ADR 0154 | Pass | The five fields with the list open, at 1280 px and at 390 px with touch. A field with a start name shows no list. `kind:zzz` gives the error row. A click on a row puts in the token. The first-visit search uses the same component as "By address or name", because the data has feeds and the Welcome page does not show. |
| ADR 0155 | Pass after a correction | Each menu names both directions, and no list has a reverse icon. At 390 px, the menu of Releases started 32 px to the left of the screen. The correction is below. |
| ADR 0156 | Pass | The page of MaryKateUltra with the entry "Test Album" open: "Album 16.7 % split · Host", "1 Track 1 30.0 % split", "3 Track 3 30.0 % split · Vocalist", "5 Track 5 Vocalist". Dark and light at 1280 px, and 390 px with touch. |
| ADR 0157 | Pass | The tabs are one segmented control at 1280 px and at 800 px in both themes, and a select menu at 760 px. Right arrow moves the focus ring and keeps the selection. The image of the tabs above the scope control of `search.html` shows the same selected look in both themes. The text of the site is larger, because Studio uses the macOS text sizes with a mouse (musicindex ADR 0024 R-TYPE-1). |
| ADR 0158 | Pass | The fold "Hidden suggestions" with "Ann Bell" and "Show Again" at 1280 px and at 390 px with touch. After "Show Again", the fold goes and "Ann Bell" is a suggestion again. |
| ADR 0159 | Pass | The sheet with the results for "Ainsley Costello" and the fold of the pages of the collection. The row "Read only" after Save. The suggestions with "Use" for a picture and a website. 1280 px and 390 px with touch. |
| ADR 0160 | Pass | The row `artist:` shows `an artist name · artist:"artist name"`. The example has the muted color of the search field placeholder, as on `search.html`. A click puts in only `artist:`. Both themes, 1280 px and 390 px with touch. |
| musicindex ADR 0056 R-SIDE-1 | Pass | The left panel has the gradient from `sidebar-glow-top` to no color at 320 px in both themes. It stays at the top when the panel scrolls. At 390 px a sticky layer gives the glow. Under `prefers-contrast: more` the panel is flat in both themes and at both widths. The site sidebar has the same gradient. |

`$edges()` and `$overflow()` gave no result on each page of the checks. The runs gave no console error from Studio.

## Correction

At 390 px with touch, the tools of Releases wrap below the heading, and the sort button is at the left edge. The sort menu aligned its right edge with the button, so it started at x = -32 px. `.releases-tools .sort-menu` now aligns the left edge of the menu with the button. At 390 px the menu is at x = 32 px to 212 px. At 1280 px it is at x = 949 px to 1129 px, below the button and in the window.

## Limits

- Chromium could not load pictures from outside hosts in the sandbox. The result rows and the picture suggestions show initials or an empty circle.
- The data has no playlist. The search of "Add tracks" ran in a new playlist that a subagent made in its own data copy.
- No check ran in WebKit, on a phone or in the AppImage. The human tests of the release notes of `0.1.0-rc.20` keep those.

## Notes For The Operator

These items are not failures of a gate.

- With `kind:zzz`, the error text shows two times: in the field list, and in the error notice of the search. The comment in `styles.css` gives this as the design of ADR 0154.
- The row of the Value Routes tab says "Add a recipient", but its sheet title says "Add a Recipient".

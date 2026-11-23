# UI And UX Findings Of 2026-11-23

This file records the UI defects and the UI improvements that the operator finds in use of Studio. Each finding gives the related code and a recommendation. The operator has not decided a finding, and no finding is built. A finding that changes a rule of an ADR needs a new ADR before a task.

## Finding 1: a release that was never written offers a file host delete

The operator added a new release and did not write it. Then the operator tried to remove it from its page.

What the code does:

- The page shows **Delete from File Host…** when `feed.connector_id` is set (`crates/api/ui/editor.js` near line 2659). A new feed gets its file host when the user adds it. Thus the button shows before a file exists on the file host. The sheet of the button then gives steps for a file that is not there.
- The confirmation of **Remove from Collection** always says "The feed file on its file host does not change" (`crates/api/ui/confirms.js`, `removeFeed`). For a feed with no written version, this text is not applicable.
- The server gives `draft_state` "new" when a draft exists and no version was written (`crates/api/src/routes.rs`, `draft_state`). The page can use the same test, or `feed.last_version`.
- Review of the same release shows **Remove from Collection…** in the position of **Discard the Draft** ([ADR 0234](../adr/0234-a-new-release-is-written-with-its-pages-and-a-one-sided-link-is-reported.md) item 1). That behavior is correct.

Recommendation:

1. Defect: do not show **Delete from File Host…** for a feed with no written version.
2. Improvement: for a feed with no written version, change the label **Remove from Collection** to **Discard the Release…** on the page and in Review. Use the type of the feed in the label (release, artist, playlist). The action does not change, and **Undo** stays.
3. Improvement: for a feed with no written version, the confirmation says "Discard “Title”? Studio has not written this release. Nothing on the file host changes."
4. A feed with a written version keeps **Remove from Collection** and **Delete from File Host…**.

Items 2 and 3 change ADR 0234 item 1. The operator asked for a note only. A mockup and an ADR are open.

## Findings 2 to 10: the White Triangles tutorial walk

These findings come from the walk of rc.32 for the White Triangles tutorial. The walk used a test SFTP server and test copies of the two feeds. The operator did not see these findings in use.

## Finding 2: the list of found feeds shows only one time

After **Connect**, File hosts shows "Studio also found these feeds in this folder" with **Add N to Studio**. The list did not show again after a restart of Studio or after **Check the Connection**.

What the code does:

- `FolderFeeds` loads the feeds of the folder that are not in the collection (`crates/api/ui/app.js` near line 2645).
- The page mounts `FolderFeeds` only while the local state `linked` exists (`app.js` near line 3148). Only the folder match after **Connect** sets `linked` (`app.js` near line 3100). Thus the list goes away when the user leaves the page.

Recommendation: defect. Show `FolderFeeds` on each SFTP host card and each folder host card. Show it when the folder has feeds that are not in the collection. `FolderFeeds` loads its list itself, so `linked` is not necessary.

Operator choice (2026-11-23): each host card has the normal button **Find Other Feeds**. The temporary area does not have it. A click shows the list in the card. Studio reads the host only after the click, so File hosts does not read each host when it opens. The list also gives "Studio reads the folder…" and "Each feed in this folder is in Studio."

Status: corrected. A walk with a debug build connected, opened Releases, and then opened File hosts again. **Find Other Feeds** showed the two feeds, added them, and then gave "Each feed in this folder is in Studio."

## Finding 3: a misspelled medium tag makes a music feed read-only, with no reason

The test feed had `podcast:meduim` as its only medium tag. Studio put the album in **Other feeds** with "This feed has no type. Studio edits albums, playlists, and artist and label feeds, so it shows only the facts of this feed."

What the code does:

- `otherFeedText` gives that text (`crates/api/ui/shell.js` near line 112).
- The **Release** menu of an artist page lists only feeds with the medium `music` or `musicL` (`crates/api/ui/editor.js` line 4281). Thus the user cannot connect the album to an artist page.
- **Issues** has no finding for this condition. `FindingKind` has no kind for a missing or misspelled medium tag (`crates/core/src/feed_check.rs`).

After the tag was correct on the server, **Read Again** made the feed a release with its tracks.

Recommendation: improvement. When a feed has no `podcast:medium` tag and has audio enclosures, tell the user that the tag is missing. When the feed has a `podcast:` tag with a name near `medium`, give that name and the line. Do not change the read-only rule without an ADR.

Operator choice (2026-11-23): [ADR 0250](../adr/0250-studio-corrects-the-type-of-a-feed-with-a-bad-or-missing-medium-tag.md) owns findings 3 and 10. The page of the feed has the select menu "Type of this feed" for a bad tag and for a missing tag. The list of found feeds names the cause. Status: built in task 241. A walk with a debug build showed the cause "misspelled tag" in the found list, the notice with line 18 and the menu on "Album (music)". **Review the Change** opened the feed in the editor, and Review showed "Medium: none → music". The write was not done, because the test host has no public address that Studio accepts.

## Finding 4: a 32-bit WAV file gets a general refusal

**Add tracks** refused a WAV file with 32-bit integer samples: "Studio does not read this kind of file." (`crates/api/ui/editor.js` line 1206, code `unsupported_media`). WAV files with 16-bit and 24-bit samples worked.

Recommendation: improvement. Tell the user that the WAV file has 32-bit samples, and give the formats that work. The cause in the decoder is not examined.

Status: corrected. Each refusal of a sample format or a bit depth in `crates/core/src/media.rs` now ends with "Export the file with 16-bit or 24-bit integer samples." A refusal of the channel count ends with "Export the file with one or two channels." The sheet shows the message of the core. A walk with a debug build showed the new text for a 32-bit WAV file.

## Finding 5: the SFTP fingerprint breaks in a word

The fit check (musicindex ADR 0065 R-FIT-3) reported a break in a word in the fingerprint of the SFTP step at 390 px and at 761 px wide. The fingerprint is one long token (`crates/api/ui/app.js` line 1036, also near line 3066).

Recommendation: defect of the markup. A fingerprint must break, as a URL or a GUID does. Add `data-fit-allow-break` to the two fingerprint elements, as `addressrow.js` does for an address.

Status: corrected. The attribute is on the three host key elements: the Connect step, the **Host key** fact of the card, and the new key notice. The fit check of the same walk gave no finding at 390 px and at 761 px.

## Finding 6: "Value routes" and "Value Routes"

The tab is **Value Routes** (`crates/api/ui/shell.js` line 243). The breadcrumb and the heading of the page are "Value routes" (`app.js` line 2327, `values.js` lines 602 and 635).

Recommendation: question for the operator. If tabs use title case and headings use sentence case by rule, this is correct. If not, use one form.

Status: not a defect. musicindex ADR 0085 R-CASE-1 gives title case to a tab and sentence case to a heading and a breadcrumb. Thus "Value Routes" on the tab and "Value routes" on the page are both correct.

## Finding 7: Kind shows "(no tracks yet)" on a release that has tracks

After **＋ Add 3 Tracks**, the release page showed **Kind** with "From the tracks (no tracks yet)". The label comes from `crates/api/ui/addfeeds.js` line 140. The cause is not examined. A stale value of the track count is possible.

Recommendation: defect, if a test confirms it. The label must give the kind from the current tracks.

Status: corrected. The cause was a stale feed list: `ReleaseKindField` in `crates/api/ui/editor.js` used the track count of the list, and the list did not load again after the tracks were added. The field now asks for the list again when the saved draft has a different track count. A walk with a debug build showed "From the tracks (Single)" after three tracks. The core rule gives Single for one to three tracks.

## Finding 8: two labels for one action

- "Add a Release" is on the artist page row and on Welcome (`crates/api/ui/welcome.js` lines 170 and 187). "Add release" is in the left panel (`shell.js` line 484).
- "Add tracks" is in the left panel and in the sheet title (`shell.js` line 227, `editor.js` line 1483). "Add Tracks" is the button of the release checklist (`feedcheck.js` line 59).

Recommendation: question for the operator. A button and a menu row can have different case by rule. If the words differ ("Add release" and "Add a Release"), use one form.

Status: decided by [ADR 0232](../adr/0232-each-group-below-an-open-feed-has-an-add-row.md) item 6. The add rows of the left panel are "Add release", "Add artist" and "Add label", and the add row "Add a Release" in the body of an artist page stays. The operator selected "Add a release" in a question that did not name ADR 0232. A change needs a new decision for the three add rows. The case of "Add tracks" and "Add Tracks" is correct by musicindex ADR 0085 R-CASE-1: a button is title case, and a sidebar row and a sheet title are sentence case.

## Finding 9: the link of an existing release needs two reviews

An artist page adds an existing release in the section **Releases**. Review of the artist page shows only the artist page. The change to the album ("Artist or label: Added") is in the draft of the album, and the user must open the album and write it in a second review. For a new release, one review writes the release with its pages ([ADR 0234](../adr/0234-a-new-release-is-written-with-its-pages-and-a-one-sided-link-is-reported.md)).

Recommendation: question for the operator. Show the album change as a row of the artist page review, as for a new release. This changes the scope of ADR 0234 and needs an ADR.

Operator choice (2026-11-23): one review writes the artist page and the linked album, as for a new release. [ADR 0251](../adr/0251-review-writes-the-two-sides-of-a-changed-link.md) owns finding 9. The check box of the linked feed is on and locked, and the rule applies from the page and from the release. Status: built in task 242.

A walk with a debug build linked a written release to a second written artist page. Review of the page and Review of the release each showed the locked row, at 1100 px and at 390 px. One write from the page wrote the two feeds, and no feed had a pencil mark after it.

## Finding 10: Studio cannot correct a misspelled tag

Finding 3 is about the message. This finding is about the repair. With `podcast:meduim`, Studio cannot correct the tag. The user must edit the feed file on the server with a different tool, then select **Read Again**.

What the code does:

- A feed with no known medium goes to **Other feeds**, and Studio shows only its facts (`otherFeedText`, `crates/api/ui/shell.js` line 111). The page has no control that sets the medium.
- `FindingKind` has no value for a missing medium or for an unknown `podcast:` tag (`crates/core/src/feed_check.rs`). Thus **Issues** has no correction to offer.
- The editor of a release has no field for an unknown tag. A tag that Studio does not know stays in the feed as it is.

Recommendation: improvement. Studio offers a correction for a feed with no medium and with audio enclosures. The correction reads "This feed has `podcast:meduim`. Change it to `podcast:medium`?" The change goes through Review and a write, as each other correction does. It also makes the feed a release that Studio can edit. This changes which feeds Studio writes, so it needs an ADR.

Operator choice (2026-11-23): [ADR 0250](../adr/0250-studio-corrects-the-type-of-a-feed-with-a-bad-or-missing-medium-tag.md) owns findings 3 and 10. The page of the feed has the select menu "Type of this feed" for a bad tag and for a missing tag. The list of found feeds names the cause. Status: built in task 241. A walk with a debug build showed the cause "misspelled tag" in the found list, the notice with line 18 and the menu on "Album (music)". **Review the Change** opened the feed in the editor, and Review showed "Medium: none → music". The write was not done, because the test host has no public address that Studio accepts.

## Finding 11: the editor head of an artist page is too wide at 761 px

The main session found this defect in the walk of task 242. At 761 px, the editor of the artist page "HeyCitizen" has two faults:

- The title breaks inside the word, as "HeyCitiz" and "en". The fit check of `tools/drive-ui.mjs` gives musicindex ADR 0065 R-FIT-3 for the `h1`, which has a width of 90 px.
- The top bar is wider than the main pane. The **Review** button is cut at the right edge, and the page scrolls to the side.

The same page without Review and without a write gives the same finding. Task 242 does not change the head. Thus the defect is older than task 242 (inference, not examined on an older commit).

Recommendation: defect. Let the buttons **Move To…** and **Read Again** go below the title when the pane is narrow, and let the top bar wrap. Status: corrected.

The cause is in `crates/api/ui/styles.css`. The narrow rules use the width of the window, but at 761 px the main pane is only 466 px wide. The head keeps **Move To…** and **Read Again** together in one group. The group goes below the title when the title needs the width. The top bar wraps, and its buttons go to a second line at the right.

A walk with a debug build showed the artist page at 761 px, 1100 px and 390 px. At 761 px the title is on one line, and the two buttons are below it. At 1100 px the buttons are adjacent to the title, as before. The fit check gave no finding at 390 px and at 761 px, and no page scrolled to the side.

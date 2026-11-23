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

## Findings 2 to 9: the White Triangles tutorial walk

These findings come from the walk of rc.32 for the White Triangles tutorial. The walk used a test SFTP server and test copies of the two feeds. The operator did not see these findings in use.

## Finding 2: the list of found feeds shows only one time

After **Connect**, File hosts shows "Studio also found these feeds in this folder" with **Add N to Studio**. The list did not show again after a restart of Studio or after **Check the Connection**.

What the code does:

- `FolderFeeds` loads the feeds of the folder that are not in the collection (`crates/api/ui/app.js` near line 2645).
- The page mounts `FolderFeeds` only while the local state `linked` exists (`app.js` near line 3148). Only the folder match after **Connect** sets `linked` (`app.js` near line 3100). Thus the list goes away when the user leaves the page.

Recommendation: defect. Show `FolderFeeds` on each SFTP host card and each folder host card. Show it when the folder has feeds that are not in the collection. `FolderFeeds` loads its list itself, so `linked` is not necessary.

## Finding 3: a misspelled medium tag makes a music feed read-only, with no reason

The test feed had `podcast:meduim` as its only medium tag. Studio put the album in **Other feeds** with "This feed has no type. Studio edits albums, playlists, and artist and label feeds, so it shows only the facts of this feed."

What the code does:

- `otherFeedText` gives that text (`crates/api/ui/shell.js` near line 112).
- The **Release** menu of an artist page lists only feeds with the medium `music` or `musicL` (`crates/api/ui/editor.js` line 4281). Thus the user cannot connect the album to an artist page.
- **Issues** has no finding for this condition. `FindingKind` has no kind for a missing or misspelled medium tag (`crates/core/src/feed_check.rs`).

After the tag was correct on the server, **Read Again** made the feed a release with its tracks.

Recommendation: improvement. When a feed has no `podcast:medium` tag and has audio enclosures, tell the user that the tag is missing. When the feed has a `podcast:` tag with a name near `medium`, give that name and the line. Do not change the read-only rule without an ADR.

## Finding 4: a 32-bit WAV file gets a general refusal

**Add tracks** refused a WAV file with 32-bit integer samples: "Studio does not read this kind of file." (`crates/api/ui/editor.js` line 1206, code `unsupported_media`). WAV files with 16-bit and 24-bit samples worked.

Recommendation: improvement. Tell the user that the WAV file has 32-bit samples, and give the formats that work. The cause in the decoder is not examined.

## Finding 5: the SFTP fingerprint breaks in a word

The fit check (musicindex ADR 0065 R-FIT-3) reported a break in a word in the fingerprint of the SFTP step at 390 px and at 761 px wide. The fingerprint is one long token (`crates/api/ui/app.js` line 1036, also near line 3066).

Recommendation: defect of the markup. A fingerprint must break, as a URL or a GUID does. Add `data-fit-allow-break` to the two fingerprint elements, as `addressrow.js` does for an address.

## Finding 6: "Value routes" and "Value Routes"

The tab is **Value Routes** (`crates/api/ui/shell.js` line 243). The breadcrumb and the heading of the page are "Value routes" (`app.js` line 2327, `values.js` lines 602 and 635).

Recommendation: question for the operator. If tabs use title case and headings use sentence case by rule, this is correct. If not, use one form.

## Finding 7: Kind shows "(no tracks yet)" on a release that has tracks

After **＋ Add 3 Tracks**, the release page showed **Kind** with "From the tracks (no tracks yet)". The label comes from `crates/api/ui/addfeeds.js` line 140. The cause is not examined. A stale value of the track count is possible.

Recommendation: defect, if a test confirms it. The label must give the kind from the current tracks.

## Finding 8: two labels for one action

- "Add a Release" is on the artist page row and on Welcome (`crates/api/ui/welcome.js` lines 170 and 187). "Add release" is in the left panel (`shell.js` line 484).
- "Add tracks" is in the left panel and in the sheet title (`shell.js` line 227, `editor.js` line 1483). "Add Tracks" is the button of the release checklist (`feedcheck.js` line 59).

Recommendation: question for the operator. A button and a menu row can have different case by rule. If the words differ ("Add release" and "Add a Release"), use one form.

## Finding 9: the link of an existing release needs two reviews

An artist page adds an existing release in the section **Releases**. Review of the artist page shows only the artist page. The change to the album ("Artist or label: Added") is in the draft of the album, and the user must open the album and write it in a second review. For a new release, one review writes the release with its pages ([ADR 0234](../adr/0234-a-new-release-is-written-with-its-pages-and-a-one-sided-link-is-reported.md)).

Recommendation: question for the operator. Show the album change as a row of the artist page review, as for a new release. This changes the scope of ADR 0234 and needs an ADR.

# Human Test Findings Of 0.1.0-rc.22

Date: 2026-11-13.

Status: advisory. This review records the results of the first part of the human test of `0.1.0-rc.22`. It states no rule. The ADRs that it names own the decisions.

The operator ran the test on a Linux x86-64 computer. Steps 1 to 10 used the AppImage `musicindex-studio-v0.1.0-rc.22-linux-x86_64.AppImage`. Steps 11 to 19 used the local build `target/release/studio-desktop-app` from 02:06 on 2026-11-13. Tasks 110 and 111 came after the tag `v0.1.0-rc.22` (`e032486`). The agent read the hosts and the Music Index API at each step that names a read.

## Results

The steps of "What to test" in the [notes of 0.1.0-rc.22](../releases/0.1.0-rc.22.md) have their numbers in the second column.

| Step | Notes step | Check | Result |
|---|---|---|---|
| 1 | 1 | A copy of the data folder | Passed. 263 MB in each folder. |
| 2–4 | 2 | The AppImage opens the data, and the vault unlocks | Passed. The feeds stayed. |
| 5 | 3 | The picture and website lists after the upgrade | Passed, with finding 7 |
| 6 | 5 | **Update from Linked Pages** with a cleared check box | Passed. The sheet had one value, so a cleared box turned **Update** off, and a selected box added the value with its source. |
| 7 | 6 | The same values do not show again | Passed. The sheet said "The linked pages have no new values for your people." |
| 8–10 | 7 | **Add a Person** from a live page, a nickname, and a new credit | Passed, with finding 8. A new credit got the name "Haleen", not the nickname. |
| 11 | — | ADR 0175 in the AppImage | Open. The AppImage does not have the rows of 8a23157. The local build was not used for this step. |
| 12–18 | — | ADR 0131: retire and delete a test feed on a real file host | The delete passed, with findings 3 to 5 |
| 19 | — | ADR 0131 item 14: the feed after the delete | Failed. Finding 6. Passed after task 134. |

Step 11 details: the album "The Heycitizen Experience" (GUID `a2d2e313-9cbd-5169-b89c-ab07b33ecc33`) showed no Music Index finding. The agent read `GET /v1/feeds/{guid}`: the index holds the same address, and `/copies` is empty. So no finding was the correct result. **Copy the Report** shows only for a copy that pays other people. No feed of the operator has such a copy, so that part cannot run with a real feed.

Steps 12 to 18 details: the test feed was "HeyCitizen's Test Feed" at `https://music.cancelbandcamp.com/feed.xml`, GUID `4662f5ff-8cff-5c61-9a07-ce9960d9a735`, on the host `heycitizen`. Before the delete, the feed had `itunes:block` and `podcast:block`, and the index gave `404 feed not found`. The Retire block showed steps 1 and 2 as done, and step 3 offered **Delete the File**. The final sheet named the host and the path, kept **Delete** off until the exact title, and counted 4 files with the check box selected. The review listed five deletes. After the write, `feed.xml` and the four files gave HTTP 404, and the index still gave 404.

Steps 8 to 10 of the notes (the server) and steps 10 and 11 (the iPhone) did not run.

## Findings

### 1. Middle Window Widths Break The Editor

At a window width of about 760 px with the left panel, the tab labels of the editor break inside words ("Valu e Rou tes", "Che ck 1"). The field values are cut off, and the rows of the tab **Check** wrap each title on many lines. The cause is in `crates/api/ui/styles.css`: the layout has one breakpoint at 760 px. From 761 px, the left panel takes about 300 px, and the editor keeps its wide form in about 450 px. The screenshot walks use 1280 px and 390 px only, so no walk saw these widths.

### 2. The Tab Menu At A Narrow Width

At a narrow width, the tab menu fills the width of the page. Its open list is the native menu of GTK. The list is not aligned with the closed menu, and its items have no inset. The correction is open. A comparison with the other two apps comes first.

### 3. The Delete Sheet Jumps

Each click on **Also delete the audio and image files** hides the title, the file host row and the line "“HeyCitizen” links to this feed" for a moment, and the sheet changes its height. The cause is in `crates/api/ui/editor.js`, `loadPreview`: it calls `setPreview(null)` before the new preview arrives. The correction is to keep the old preview until the new one arrives.

### 4. Texts Of The Delete Review

The row of `feed.xml` says "Studio deletes it after the feed is written". That text is not correct for the feed file itself. The row **Announce** says "No announcement: this feed has no public address", while the feed had a public address before the write. The cause of the second text is not verified.

### 5. The Write Button Of A Delete

The review of a delete shows **Write to the File Host** in the primary color. The write deletes five files. ADR 0193 gives the order of the buttons and does not say if this button is destructive. This item is a question for the operator.

### 6. A Deleted Feed Does Not Open

Directly after the write, the top bar showed **Not saved** in red, and the Retire block still offered **Delete the File**. When the operator opened the feed again, the head showed **Read-only**, but the page showed "Studio could not save the draft" and "the release folder has a different feed XML; read the feed again". ADR 0131 item 14 says that the feed stays as read-only, with its last copy.

The probable cause comes from the code. A test has not proved it yet:

- The delete write calls `record_feed_host_delete` (`crates/core/src/collection.rs`), which sets `connector_id` to NULL. It does not change `last_version`, and it writes no copy to the release folder (`crates/core/src/pipeline.rs`, the `deleting_feed` branches).
- A feed with no connector starts its draft with `start_public_draft` (`crates/core/src/pipeline.rs`). That function compares `content_version` of the release folder copy, a `sha256:` value, with `last_version`.
- The `last_version` of a feed on an S3 or R2 host is the ETag of the host (`crates/core/src/connector/s3.rs`). The facts of the test feed showed the version `d4de5b7a46da010a1e6…`, with no `sha256:` prefix. The SHA-256 of the feed before the delete was `c4a7042c…`.
- So the two values cannot agree after a delete on an S3 or R2 host. `start_restore_draft` (`crates/core/src/feed_delete.rs`) makes the same comparison, so a restore possibly fails too.

Task 134 (`51dc619`) proved this cause with a test host that gives ETag versions. A deleted feed now opens from the snapshot of its delete. The human check of task 134 passed on 2026-11-13: the test feed opened as read-only with its tracks, and the Retire block showed the three steps as done.

Two observations from that check, not verified in the code:

- Step 1 of the Retire block said "Hide from apps is on in the draft." After the delete, the draft is the last copy, so "the last write" is possibly the clearer text.
- **Stop Retiring** and **Link to This Page** show on the read-only feed.

### 7. Two Linked Page Rows Look The Same

The page of Prosperous Soul has two linked page rows. Each row shows only "Prosperous Soul" and "Read only". The user cannot see which row is the page on wavlake.com and which is the page on feeds.fountain.fm.

### 8. A Person From A Page Shows "Added In Studio"

**Add a Person** made Haleen from a live page on Music Index. The picture and the website came from that page, but each shows the line `Added in Studio`, not `From "Haleen" on …`. The known gap of rc.22 is only about the values from before the upgrade. Possibly this is a defect. The code was not read yet.

### 9. Music Index Cannot Find An Artist With No Publisher Feed

**Find on Music Index** found only a release feed for "Delta OG", and said "A release has no values for a person. Select an artist or label page." Music Index has no publisher feed for that artist. The known gaps of rc.22 already give this: the search finds a feed, not a person. This is not a defect.

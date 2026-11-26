# The Human Check Walk Of 2026-11-26

This file records the findings of the operator checks of tasks 254, 256, 257 and 258. The check uses the release binary of commit 74efa28 with the real data of the operator. The main session corrects a finding after the check ends.

## Step 1: A Real Drag Of "HeyCitizen" (Task 254)

The real data has no page "Quiet Pine Records". That page is only in the harness data of the main session. The drag of the handle of "HeyCitizen" started.

### Finding 1: "not published" reads as "not released"

The notice after the drag said "not published" for HeyCitizen. The operator said that HeyCitizen is self-published.

Correction of the first cause of the main session: on 2026-11-26 the field "Role" of HeyCitizen is "Artist" only. No publisher page is above it. The code follows [ADR 0254](../adr/0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) items 11, 25, 31 and 32 and [ADR 0257](../adr/0257-a-release-has-one-artist-one-label-and-one-publisher-in-its-fields.md) item 5, which use "Not published" for "no publisher page". The fault is in the words. An artist puts the files on a personal file host, and thus publishes the music.

Recommendation: improvement that changes the words of ADR 0254 and ADR 0257. The words are for the operator to select.

### Finding 2: the drag offers a drop place with no effect

During the drag, the app showed a place to drop that is for a publisher. For "HeyCitizen", a drop there changes nothing. The main session thinks that this is the top area "Drop here: no label and no publisher" (`crates/api/ui/app.js` line 3699). The operator confirms this.

Recommendation: defect. A drop place shows only when a drop there changes the place of the dragged page.

## Step 2: Undo And The Sheet "Sign To A Label…" (Task 254)

The operator opened "Sign to a label…" on the artist page "HeyCitizen" in Dark. The collection has no label page, so the list says "No label matches." The head of the page and the line "Now" say "Independent, not published". That is finding 1 again.

### Finding 3: the sheet "Sign to a label…" cannot make a new label

With no label in the collection, the sheet gives no next step. The operator wants the sheet to offer a new label.

Cause: the code follows [ADR 0254](../adr/0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) item 25. That item gives a sheet of a page action no row for a new page. The Add sheet has the group "New" (item 19), and the release sheet has "New Label Page…" (item 30).

Recommendation: improvement. A sheet that selects a label or a publisher gets a row "＋ Make a new {label or publisher} page", as the group "New" of the Add sheet. This changes ADR 0254 item 25. Main session opinion: also give the sheet "Sign to a label…" the button **Sign**, not **Change Label**, because the artist has no label now.

## Step 3: A New Label And The Add Sheet (Task 254)

The operator made the pages "TestPublisher" and "TestLabel". "TestLabel" is below "TestPublisher". "HeyCitizen" is below the heading "Artists".

### Finding 4: a drag handle is not next to its row

The handle of "TestLabel" is at about 33 px from the left edge of the panel. The handle of "HeyCitizen" is at about 16 px. Each handle has an empty column between it and its row. The handle of "TestLabel" is outside the selected area of its row. Thus the user cannot easily see which row a handle moves.

Cause: in `CatalogRow` (`crates/api/ui/app.js` near line 3742), the handle is the first item of `.tree-row`, before the toggle column. The indent of `.tree-row` (`padding-inline-start`, `styles.css` line 757) moves the handle with the depth. The toggle column then stays between the handle and the row.

Recommendation: defect. Main session opinion: put the handle directly before the art, after the toggle column and in the area of the row. Then the handle and its row move together.

### Finding 5: a menu "⋯" does not close on a click outside it

The menu stays open after a click on an empty area of the window.

Cause: `MenuControl` (`crates/api/ui/menu.js`) closes the menu only on a click of its button, a click of an item and Escape. This applies to each menu of the app that uses `MenuControl`.

Recommendation: defect. A click or a tap outside an open menu closes it, and the focus stays where the user put it.

### Finding 6: the row "Find on Music Index" has no icon

The sheet "Make a new artist page" starts with the row "Find on Music Index", with no icon. The row "My music is online already" of the welcome page has a magnifying glass icon (`search` in `crates/api/ui/welcome.js` line 16).

Recommendation: improvement. Each row "Find on Music Index" shows the icon `search`. The row is made in `values.js`, `editor.js` and `peoplepages.js`.

## The Written Feeds Of The Sign Change

The operator signed "HeyCitizen" to "TestLabel" and wrote the change. The main session read the hosted files on 2026-11-26:

- Each of the four releases names TestPublisher with `rel="hosting"`, TestLabel with `rel="label"` and HeyCitizen with `rel="artist"` in `podcast:publisher`.
- `testlabel.xml` lists the four releases with `rel="label"`. `testpublisher.xml` lists the four releases with `rel="hosting"`.
- No feed file has a link between TestLabel and TestPublisher. This agrees with ADR 0254 items 40 to 42: the shared releases give that link.
- The artist feed of HeyCitizen did not change, so it keeps the tag `generator` of MSP 2.0 (ADR 0265 item 4).

Result: the feed data agrees with ADR 0254 item 26.

### Task 257: the human check passes

Each feed that Studio wrote has `<generator>Music Index Studio 0.1.0-rc.32</generator>`. This is the text of ADR 0265 item 2.

### Finding 7: the Music Index page of TestLabel (website, not Studio)

The page of TestLabel on the Music Index website shows these faults:

- It shows two releases of four. The main session thinks that the index does not have "check 212" and "HeyCitizen's Test Album" yet. This is an inference, not a checked fact.
- The heading "Artists on these releases" lists TestPublisher, which is a publisher.

Recommendation: these items are for the musicindex repository. The main session sends them to the orchestrator.

Answer of the website session through the orchestrator, 2026-11-26:

- Item 1 is not a fault of the page. The index holds no feed for the two missing releases: `GET /v1/feeds/{guid}` gives 404 for each. The cause is with the Stophammer session.
- Item 2 is corrected in musicindex commit 6e1f2c3, which is not deployed. When the releases of a feed give only `rel="hosting"`, the feed goes to the Overview row "Publisher".

## Step 4: A Publisher Change, The Count, Review And Undo (Task 254)

The operator selected "Change publisher…" on HeyCitizen and the row "Not published". The notice "HeyCitizen is now independent, not published. Not written yet." showed at the top of the window, with **Undo**, **Review** and a close button. The top of the Catalog panel showed a second **Review** button. The top bar showed "No changes" and its own **Review** button.

### Finding 12: one change set has three Review buttons, and the top bar says "No changes"

The notice, the top of the Catalog panel and the top bar each have a **Review** button. The operator finds the new button at the top of the panel a bad position. The top bar of HeyCitizen says "No changes", but the change set changes the three releases of HeyCitizen.

Cause: the code follows [ADR 0254](../adr/0254-the-catalog-is-a-tree-and-each-change-uses-music-words.md) items 35 and 36. The top bar gives the draft state of the open page only.

Recommendation: improvement that changes ADR 0254 items 35 and 36. Main session opinion: one **Review** button in the top bar opens the open change set, and the top bar shows the count, for example "4 changes not written". The notice keeps **Undo** and the close button. Mockups first.

## The Retire Of TestLabel

The operator turned on "Hide from apps" for TestLabel and wrote it. The hosted file has `<itunes:block>Yes</itunes:block>` and `<podcast:block>yes</podcast:block>`, with the build date 10:59:37 UTC. At 11:06 UTC, the Music Index API (`/v1/feeds/{guid}`) gave the build date 05:26:53 UTC. Thus Music Index did not read the write yet.

### Finding 8: step 1 of Retire says "in the draft" after the write

Step 1 says "Hide from apps is on in the draft." The head of the page says "No changes", and the hosted file has the block tags.

Cause, not proved: `retire_facts` (`crates/core/src/feed_retire.rs`) reads the hidden state from `hosted_copy` in the library. The answer `draft_only` shows that this copy did not have the block tags. The main session thinks that the write did not update that copy. This needs a test before a correction.

Recommendation: defect. After a write, step 1 says "Hide from apps is on in the last write."

### Finding 9: Check Again gives no visible result

When Music Index still holds the feed, a push of **Check Again** changes nothing on the screen. The user cannot see that the check occurred.

Recommendation: defect. The step shows the time of the last check, for example "Checked at 11:06".

### Finding 10: Music Index does not read the blocked TestLabel feed after two podpings (not Studio)

Studio sent a podping for `testlabel.xml` at 11:00 UTC and again at 11:40 UTC. Each line shows "Sent". At 11:40:53 UTC, the Music Index API gave `last_build_date` 05:26:53 UTC and `updated_at` 05:27:53 UTC for GUID `84e85b32-f0dc-5d1e-952e-13a438685954`.

After the first write at 05:26:53 UTC, Music Index read the feed in about one minute. The file of 10:59:37 UTC is the first one with `podcast:block` and `itunes:block`. The main session thinks that the index does not read a blocked feed through the podping path. This is an inference. The main session did not read the cause in the Stophammer code.

Answer of the Stophammer session, 2026-11-26, from code evidence:

- Cause 1, the node: the verifier `medium_music` refuses a publisher feed with no `remoteItem` of medium music. This occurs before the block rule of Stophammer ADR 0057 §3, so the record is not retired.
- Cause 2, the crawler: after that refusal, the gossip crawler skips each later podping of the URL, with no time limit. Its skip check has no exception for a publisher feed (Stophammer ADR 0101 §4).
- The two missing releases are not a defect. "HeyCitizen's Test Album" has the block tags, so the index retired it (inference). The Stophammer session found `check-212.xml` with HTTP 404 at about 11:40 UTC. The main session read the same file with HTTP 200 at about 11:05 UTC, and again with HTTP 404 at 11:44 UTC.

Operator choices of 2026-11-26: the two Stophammer corrections get an operator choice line, not an ADR. TestLabel stays until the corrections are deployed, so that the Studio retire flow gets a full test. The main session sent these choices to the Stophammer session. The corrections are stophammer ff9bb60 and stophammer-crawler 79bcc74, not pushed.

The Stophammer session withdrew its result for `check-212.xml`. The index possibly refused that body, or a skip check stopped the fetch. Only the server logs can tell which.

The log of the Docker service `primary` gave the cause at 05:27:55 UTC: `reason="[v4v_payment] no feed-level podcast:value block and no tracks — feed does not participate in V4V"`. The release "check 212" has no track and no channel `podcast:value`, so the index refused it by its rule.

The operator deleted `check-212.xml` from the file host at about 11:06 UTC. The crawler got the first HTTP 404 at 11:07:57 UTC. This is not a defect.

### Finding 11: Studio writes a release that Music Index refuses, with no warning

Studio wrote "check 212" and sent a podping, and the release showed as linked to TestLabel. The user did not learn that Music Index refused it.

Recommendation: question for the operator. Studio can tell before the write that Music Index refuses a release with no track and no payment split. Main session opinion: a line in Review is the correct place.

Recommendation: the retire flow of ADR 0131 cannot complete step 2 until Music Index reads a blocked feed and removes its entry. This is for the Stophammer repository. The main session sent it to the orchestrator.

## Operator Choices Of 2026-11-26

- Finding 1: the words for "no publisher page" are "self-published". Examples: "Independent, self-published", and the row "Self-published" in a publisher sheet. This changes the words of ADR 0254 and ADR 0257. The main session sends the words to the orchestrator, because the other apps use the same words.
- Finding 11: Review shows a warning before the write when Music Index refuses a release with no track and no channel `podcast:value`. The write stays available.
- The walk stops after step 4, item 1. The main session corrects the findings first. Open steps: step 4 items 2 to 5, and steps 5 to 8 of task 256.

## Plan For The Findings

- Defects with no decision: findings 2, 4, 5, 6, 8 and 9, in [task 259](../tasks/studio-v1-task-259-defects-of-the-walk-of-2026-11-26.md).
- Changes of an ADR: findings 1, 3, 11 and 12, in [ADR 0267](../adr/0267-a-page-sheet-can-make-a-new-page-and-review-has-one-button.md). The operator selected the mockups of findings 3 and 12 on 2026-11-26: the group "New" at the top of a page sheet, and one **Review** button in the top bar.

# Live shows

A live show is an item in a music feed.
Apps use it to show that you play live and to link to the stream.
You can start and end a show in Studio, or let a connected app do it. See [Connected apps and show grants](connected-apps.md).

## Make a show feed

You can add a live show to any album feed.
To keep your shows in their own feed, make a show feed.

1. Select **Releases** in the left panel. Then select **＋ Add a feed**.
2. On the page **Add feeds**, select **Show** in the group **Make a new feed**. A sheet opens.
3. Select the file host in **File host** if you have more than one.
4. Type the name in **Show feed title**.
5. Type the name of the artist in **Artist**.
6. Select **Make the show feed**.

Studio gives the feed file from the title. To change it, open the row **Feed file**.

A show feed is a music feed, so Studio opens it as an album.

## Add a live show

1. Open the feed.
2. Select **Add a live show** in the outline.
3. Type a **Title**.
4. Select a **Start time**.
5. Select **＋ Add the show**.

Studio opens the page of the show. The outline lists the show under **Live shows** with its status.

## The page of a show

| Section | What it holds |
| --- | --- |
| Status | The status: "Planned", "Live now" or "Ended". It also lists problems that stop a write. |
| Grant | The grant of the show. See [Connected apps and show grants](connected-apps.md). |
| Show | **Title**, **Description**, **Start time**, **End time** and **Image address**. Select **Choose…** to pick an image file. |
| Stream | **Stream address** and **Stream type**, for example `application/x-mpegURL`. It shows until the show ends. |
| Recording | After the show ends. See below. |
| Content links | **Add a content link**. Each link has **Link address** and **Link text**. |
| Live value | **Live value address** and **Live value protocol**. Select **Remove the live value** to remove it. |
| Value routes | Recipients for this show. |
| More | **Identity** shows the show GUID. |

A show needs a start time and one content link.
Studio does not write the show without them.
The status section tells you what is missing: "This show has no start time." or "This show has no content link."

## Go live and end the show

1. Open the page of the show.
2. Select **Go live**.
3. Studio changes the draft and opens the review sheet.
4. Select **Write to the file host**.

When the show is over, select **End the show** and write in the same way.
Neither button changes the file host until you write.
See [Review and write your changes](review-and-write.md).

## Add the recording

After a show ends, the page has the section **Recording**.

1. Select **Choose…** adjacent to **Recording**.
2. Select a source in the sheet: **This release**, **File hosts** or **This computer**.
3. Select the audio file.

You can also drop one audio file on the field.
Studio puts the file in the enclosure of the show.
The stream address goes out of the draft.
If the show has a recording, the section shows the address with **Choose…**, a player and **Properties**.
Use **Choose…** to replace the recording.

The review lists a deletion of the file from before. Select **Undo** to keep it.

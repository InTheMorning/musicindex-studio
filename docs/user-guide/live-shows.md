# Live shows

A live show is an item in a music feed.
Apps use it to show that you play live and to link to the stream.
You can start and end a show in Studio, or let a connected app do it. See [Connected apps and show grants](connected-apps.md).

## Make a show feed

You can add a live show to any album feed.
To keep your shows in their own feed, make a show feed.

1. Select **Releases** in the left panel. Then select **＋ Add a Feed**.
2. On the page **Add feeds**, select **Show** in the group **Make a new feed**. A sheet opens.
3. Select the file host in **File host** if you have more than one.
4. Type the name in **Show feed title**.
5. Type the name of the artist in **Artist**.
6. Select **Make the Show Feed**.

Studio gives the feed file from the title. To change it, open the row **Feed file**.

A show feed is a music feed, so Studio opens it as an album.

## Add a live show

1. Open the feed.
2. Select the tab **Live Shows**, then select **＋ Add a Live Show**.
3. Type a **Title**.
4. Select a **Start time**.
5. Select **＋ Add the Show**.

Studio opens the page of the show. The tab **Live Shows** lists the show with its status.

## The page of a show

| Section | What it holds |
| --- | --- |
| Status | The status: "Planned", "Live now" or "Ended". It also lists the problems of the show. A problem in red stops a write. A warning in orange does not stop it. |
| Grant | The grant of the show. See [Connected apps and show grants](connected-apps.md). |
| Show | **Title**, **Description**, **Start time**, **End time** and **Image address**. Select **Choose…** to pick an image file. |
| Stream | **Stream address**, **Stream type** and **Check Stream**. It shows until the show ends. |
| Recording | After the show ends. See below. |
| Content links | **Add a content link**. Each link has **Link address** and **Link text**. |
| Live value | **Live value address** and **Live value protocol**. Select **Remove the Live Value** to remove it. |
| Value routes | Recipients for this show. |
| More | **Identity** shows the show GUID. |

A show needs a start time and a stream.
The stream needs a **Stream address** and a **Stream type**.
Apps play the stream, so Studio does not write the show without it.
When the stream address ends in a known extension, the stream type must agree with it.
For example, an address that ends in `.m3u8` needs `application/x-mpegURL`, and an address that ends in `.mp3` needs `audio/mpeg`.

**Stream type** is a menu: HLS, MP3, AAC, M4A, Ogg, DASH and **Other…**.
Select **Other…** to type a different type.
A type that is not in the menu shows as **Other…** with its text.
Studio fills the type from the extension of the address, until you select a type yourself.
A type that is in the draft when you open the page counts as selected.

## Check the stream

1. Enter the **Stream address**.
2. Select **Check** adjacent to **Check Stream**.
3. Read the line below the button.

Studio reads the stream for at most 8 seconds. It writes nothing.
When the stream is live, the line shows "Live", the type that the stream server sends, and the time of the check.
The check also fills the type, until you select one yourself.
When you selected a type, and the stream server sends a different type, the line also shows a warning in orange. Select **Use** and the name of the type to use the type of the server.

When the stream is not live, the line shows the cause in orange.
A recording is not live. For example, an HLS playlist with an end marker, a DASH file that is not dynamic, or an audio file with a known length.
A server on the internet cannot read a stream in a private network.

The status section tells you what is missing, for example "This show has no start time." or "This show has no stream address."

A content link is recommended. It gives a page for a person, for example a web player or a chat room.
When the show has no content link, the status section shows the warning "This show has no content link.", and Studio writes the show.

## Go live and end the show

1. Open the page of the show.
2. Select **Go Live**.
3. Studio checks the stream. Then it changes the draft and opens the review sheet.
4. Select **Write to the File Host**.

When the stream is not live, Studio refuses **Go Live** and the draft does not change.
The status section shows "Studio cannot go live, because the stream is not live", and the cause.
Start the stream and select **Go Live** again.
A connected app that starts the show through a grant gets the same check and the same refusal.
**End the Show** has no check. A write of the feed does not check the stream.

When the show is over, select **End the Show** and write in the same way.
Neither button changes the file host until you write.
See [Review and write your changes](review-and-write.md).

## Before a show with a grant

When you give a grant to a connected app, the app starts and ends the show for you.
See [Give a grant for a show](connected-apps.md#give-a-grant-for-a-show).
Studio refuses a granted change while the feed has a draft with changes.

1. Before the show, write or discard each draft of the feed.
2. Select **Review** and **Write to the File Host** to write. Select **Discard the Draft** to discard.

While a grant is active and the feed has a draft with changes, each page of the feed shows a notice.
It names the app and the show and gives the start time. For example: "Live Publisher cannot start Friday Night Signals while this draft has changes".
The notice goes away when the draft has no change.

When Studio refuses a granted change, the section **Grant** lists it with the cause.
See [See what a grant did](connected-apps.md#see-what-a-grant-did).

## Add the recording

After a show ends, the page has the section **Recording**.

1. Select **Choose…** adjacent to **Recording**.
2. Select a source in the sheet: **This Release**, **File Hosts** or **This Computer**.
3. Select the audio file.

You can also drop one audio file on the field.
Studio puts the file in the enclosure of the show.
The stream address goes out of the draft.
If the show has a recording, the section shows the address with **Choose…**, a player and **Properties**.
Use **Choose…** to replace the recording.

The review lists a deletion of the file from before. Select **Undo** to keep it.

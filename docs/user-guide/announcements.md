# Announcements

After you write a feed, Studio can tell the Podcast Index hub about the change.
The hub is the Podcast Index service that lists feeds.

## Set the announcements

1. Select **Hosts** in the left panel.
2. Find the panel **Announcements**.
3. Use the check boxes:
   - **Tell Podcast Index after each write**
   - **Send a podping with it**. This box works only when the first box is on.
4. Select **Save**.

Both boxes are on until you change them.
Studio sends the announcement 30 seconds after a write.

## See the announcement in the review

Before a write, the sheet "Review changes" lists the announcements.
Each line starts with **Announce**. It shows the address that Studio sends.

The sheet can also tell you why Studio sends no announcement:

- "No announcement: the announcements are off in Hosts."
- "No announcement: this feed has no public address."

## See the state of the last announcement

1. Open the album.
2. Open **Feed facts** on the album page.
3. Read the line **Announcement**.

The line shows one of these states:

- Waiting. Studio tells you when it sends the announcement.
- Sent. Studio shows the time.
- Failed. Studio shows the time and the error.
- "No announcement yet."

If an announcement fails, Studio tries again at the time that it shows. It also shows the last error.

# Make a new release

A new release is a new album feed.
Studio makes it as a draft. The feed file does not exist on the file host until you write.

On a server, use an S3 or R2 bucket, an SFTP server or an nsite site for the feed file.
The file host type "Folder on this computer" is not available on a server.

## Before you start

Connect a file host. Studio can make a draft without a public address. It uses a placeholder GUID until the file host has one. See [Connect a file host](hosts.md).
If you have no file host, the sheet shows "Connect a file host to add this feed."
Select **Connect a File Host**.

A Blossom file host keeps media only. Use a different file host for the feed file.

## Make the release

1. Select **Releases**, then **＋ Add**, then **Release**. A sheet opens.
2. If you have more than one file host, select the file host in **File host**.
3. Type the release title in **Title** and the artist name in **Artist name**.
4. Select the artist page of the release in **Artist page**. If Studio shows "No artist page is available. Make an artist page first.", make the artist page first. See [Artist and label feeds](artist-and-label-feeds.md).
5. Select **Make the Release**.

Studio gives the feed file from the title. For example, "Pixel Palace" gives `pixel-palace.xml`.
The row **Feed file** shows this path. To change it, open the row and type a new path in the folder of the file host.
After you type a path, the title does not change it again.

Studio opens the new release. A notice tells you that the file host has no file for this release yet.

If a file with this path exists, Studio shows "A file exists at this path already."

## Add the content

Fill in the album details and add tracks. See these pages:

- [Edit the album details](edit-album.md)
- [Add, import and change tracks](tracks.md)
- [Value routes](value-routes.md)

## Write the feed file

1. Select **Review** in the toolbar.
2. Check the changes.
3. Examine each **Also write** entry for linked pages.
4. Select **Write to the File Host**.

Review writes the new release and its linked pages together. Studio now makes the feed file on the file host. See [Review and write your changes](review-and-write.md).

## Other new feeds

The **＋ Add** menu also has **Artist**, **Label**, **Show**, and **Playlist**. Each entry opens its creation sheet. See [Artist and label feeds](artist-and-label-feeds.md), [Live shows](live-shows.md), and [Playlists](playlists.md).

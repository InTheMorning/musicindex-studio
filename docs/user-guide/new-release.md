# Make a new release

A new release is a new album feed.
Studio makes it as a draft. The feed file does not exist on the host until you write.

On a server, use an S3 or R2 bucket, an SFTP server or an nsite site for the feed file.
The host type "Folder on this computer" is not available on a server.

## Before you start

Connect a host that publishes its files on the web.
The host needs a public address. See [Connect a host](hosts.md).
If you have no host, the group **Make a new feed** shows "Studio puts the files of a release on a host. Connect a host first."
Select **Connect a Host**.

A Blossom host keeps media only. Use a different host for the feed file.

## Make the release

1. Select **Releases** in the left panel. Then select **＋ Add a feed**.
2. On the page **Add feeds**, select **Album** in the group **Make a new feed**. A sheet opens.
3. If you have more than one host, select the host in **Host**.
4. Type the name of the album in **Album title**.
5. Type the name of the artist in **Artist**.
6. Select **Make the release**.

Studio gives the feed file from the title. For example, "Pixel Palace" gives `pixel-palace.xml`.
The row **Feed file** shows this path. To change it, open the row and type a new path in the folder of the host.
After you type a path, the title does not change it again.

Studio opens the new release.
A notice tells you that the host has no file for this release yet.

If the host has no public address, Studio shows "Give this host a public address first, in Hosts."
If a file with this path exists, Studio shows "A file exists at this path already."

## Add the content

Fill in the album details and add tracks. See these pages:

- [Edit the album details](edit-album.md)
- [Add, import and change tracks](tracks.md)
- [Value routes](value-routes.md)

## Write the feed file

1. Select **Review** in the toolbar.
2. Check the changes.
3. Select **Write to the host**.

Studio now makes the feed file on the host. See [Review and write your changes](review-and-write.md).

## Other new feeds

The group **Make a new feed** also has these rows. They open the same sheet:

- **Artist or label**. See [Artist and label feeds](artist-and-label-feeds.md).
- **Show**. See [Live shows](live-shows.md).
- **Playlist**. See [Playlists](playlists.md).

In the sheet, **Type** selects the feed type.

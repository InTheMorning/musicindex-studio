# Make a new release

A new release is a new album feed.
Studio makes it as a draft. The feed file does not exist on the host until you write.

<!-- PHASE-12-GAP: say which hosts a new release can use on the server and that the library folder is set by the server; source: ADR 0055 items 3 and 24 -->

## Before you start

Connect a host that publishes its files on the web.
The host needs a public address. See [Connect a host](hosts.md).
If you have no host, the panel shows "Studio puts the files of a release on a host. Connect a host first."
Select **Connect a Host**.

A Blossom host keeps media only. Use a different host for the feed file.

## Make the release

1. Select **Add a feed** in the left panel.
2. In the panel **Make a new release**, select the host in **Host**. The field shows when you have more than one host.
3. Type the path of the new feed file in **Feed file**, for example `my-album.xml`. The path is in the folder of the host.
4. Type the name of the album in **Album title**.
5. Type the name of the artist in **Artist**.
6. Select **Make the release**.

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

The page **Add feeds** also has these panels:

- **Make a publisher feed**. See [Publisher feeds](publisher-feeds.md).
- **Make a show feed**. See [Live shows](live-shows.md).
- **Make a playlist**. See [Playlists](playlists.md).

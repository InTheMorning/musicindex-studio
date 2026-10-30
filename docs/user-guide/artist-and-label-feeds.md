# Artist and label feeds

An artist feed lists the releases of an artist. A label feed lists the releases of a label.
Each release can link back to its artist feed or its label feed.
Studio keeps both links correct when you add or remove a link.

In RSS, the two types of feed have the medium `publisher`. Studio shows each role of the feed, for example **Artist** or **Artist · Label**. The dot has the color of the first role.

A feed can have more than one role. The sequence is artist, label, hosting, producer, host, author, network and sponsor.
When Studio finds the roles in the feed, the dot is hollow until you select the roles in the field **Role**. A feed with no role shows **Feed** with a hollow dot.

## Make an artist feed or a label feed

You need a host with a public address. See [Connect a host](hosts.md).

1. Select **Releases** in the left panel. Then select **＋ Add a feed**.
2. On the page **Add feeds**, select **Artist or label** in the group **Make a new feed**. A sheet opens.
3. Select the host in **Host** if you have more than one.
4. Type the name in **Name**.
5. Under **Role**, select **Artist**, **Label** or **Hosting**.
6. Select **Make the feed**.

Studio gives the feed file from the name. To change it, open the row **Feed file**.

Studio opens the new feed as a draft.
Select **Review** and **Write to the host** to make the file. See [Review and write your changes](review-and-write.md).

## The pages of the feed

An artist feed has these pages: **Artist**, **Value routes**, **Credits and links** and **Files**. A label feed has **Label**, not **Artist**.

The page **Artist** or **Label** has these sections:

- **Artist** or **Label**. Fields: **Title**, **Name**, **Description**, **Cover image address** and **Website**. Select **Choose…** to pick an image file. See [Edit the album details](edit-album.md).
- **Releases**. The releases of this artist or this label.
- **Details**. Two rows: **Value routes** and **Credits and links**. They open the same pages as the details of an album.
- **Role**. Select **Artist**, **Label**, **Hosting** or **Custom**. **Custom** shows a check box for each role. Select one or more roles.
  - When Studio finds the roles in the feed, it names them. Select the roles to keep them.
  - On a read-only feed, connect the folder that holds the feed in Hosts before you change the roles.

## Add a value route or a funding link

1. Open the artist feed or the label feed.
2. In **Details**, select **Value routes** to add the recipients of the payments.
3. In **Details**, select **Credits and links** to add persons, funding links and the license.
4. Examine the diff, and write the feed.

Studio does not put these recipients into the releases of the feed.

## Add a release to an artist feed or a label feed

1. Open the artist feed or the label feed.
2. In **Releases**, select a release in the field **Release**.
3. Select **Add the Release**.

The list shows only releases from your collection that have a host. A release is an album or a playlist.
Each row says "In the collection" or "Not in the collection".
Select a number in the position menu of a row to move the release. Use **Remove** to remove it.

A link changes two drafts: the draft of the artist feed or label feed, and the draft of the release.
Studio then shows a notice such as "Studio also changed the draft of ...".
Select the button in the notice to open that feed. Then examine the changes and write them.
Write both feeds to publish both sides of the link.

If the feed has no role, Studio asks you to select the role first.

## Add an artist or a label to a release

1. Open the album or the playlist.
2. In the section **Artists and labels**, select a feed in the field **Artist or label**.
3. Select **Add the Feed**.

The first feed in the list is the primary one.
The row shows the role and "Primary".
Use **Move up**, **Move down** and **Remove** to change the list.

## Remove a link

Select **Remove** on the row.
If the other feed is in your collection, Studio changes both drafts.
If it is not, Studio only changes the open feed.

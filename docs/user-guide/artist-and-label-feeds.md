# Artist and label feeds

An artist feed lists the releases of an artist. A label feed lists the releases of a label.
Each release can link back to its artist feed or its label feed.
Studio keeps both links correct when you add or remove a link.

In RSS, the two types of feed have the medium `publisher`. Studio shows each role of the feed, for example **Artist** or **Artist · Label**. The dot has the color of the first role.

A feed can have more than one role. The sequence is artist, label, hosting, curator, producer, host, author, network and sponsor. A curator makes playlists.
When Studio finds the roles in the feed, the dot is hollow until you select the roles in the field **Role**. A feed with no role shows **Feed** with a hollow dot.

## Make an artist feed or a label feed

You need a file host. Without a public address, Studio gives the feed a placeholder GUID. See [Connect a file host](hosts.md).

1. Select **Releases**, then **＋ Add**, then **Artist** or **Label**.
2. Select the file host in **File host** if you have more than one.
3. Type the name in **Name**.
4. Under **Role**, select **Artist**, **Label** or **Hosting**.
5. Select **Make the Page**.

Studio gives the feed file from the name. To change it, open the row **Feed file**.

Studio opens the new feed as a draft.
Select **Review** and **Write to the File Host** to make the file. See [Review and write your changes](review-and-write.md).

## The pages of the feed

The tabs are **Artist** or **Label**, **Value Routes**, **Credits**, **Files**, **Issues**, and **Settings**.

The page **Artist** or **Label** has these sections:

- **Artist** or **Label**. Fields include **Title**, **Name**, **Description**, and **Website**. Open **Artwork** to change the cover image or select an image file. See [Edit the album details](edit-album.md).
- **Releases**. The releases of this artist or this label.
- **Role**. Select **Artist**, **Label**, **Hosting** or **Custom**. **Custom** shows a check box for each role. Select one or more roles.
  - When Studio finds the roles in the feed, it names them. Select the roles to keep them.
  - On a read-only feed, connect the folder that holds the feed in File hosts before you change the roles.

## Add a value route or a funding link

1. Open the artist feed or the label feed.
2. Select the tab **Value Routes** to add the recipients of the payments.
3. Select the tab **Credits** to add persons, funding links and the license.
4. Examine the diff, and write the feed.

Studio does not put these recipients into the releases of the feed.

## Make a new release for an artist feed or a label feed

1. Open an artist page.
2. At the end of **Releases**, select **＋ Add a Release**.
3. Type the title. **Artist** starts with the name of the artist.
4. Select **Make the Release**.

A new label page with no release shows the row **Add a release**. Select it to make a release that the label puts out.

You can also select **Releases**, then **＋ Add**, then **Release**. In the form, select the artist page in **Artist page**.

Studio adds the new release to **Releases** of the feed, and opens the page of the release.
This change also changes two drafts. Write both feeds through Review.

## Add a release to an artist feed or a label feed

1. Open the artist feed or the label feed.
2. In **Releases**, select a release in the field **Release**.
3. Select **Add the Release**.

The list shows only releases from your collection that have a file host. A release is an album or a playlist.
Each row says "In the collection" or "Not in the collection".
Select a number in the position menu of a row to move the release. Use **Remove** to remove it.

A link changes two drafts: the draft of the artist feed or label feed, and the draft of the release.
Studio then shows a notice such as "Studio also changed the draft of ...".
Select the button in the notice to open that feed. Then examine the changes and write them.
Write both feeds to publish both sides of the link.

If the feed has no role, Studio asks you to select the role first.

## Add an artist or a label to a release

1. Open the album or the playlist.
2. In the section **Linked artist and label pages**, select a feed in the field **Link to an artist or label page**.
3. Select **Link to This Page**.

The first feed in the list is the primary one.
The row shows the role and "Primary".
Use **Move Up**, **Move Down** and **Remove** to change the list.

## Remove a link

Select **Remove** on the row.
If the other feed is in your collection, Studio changes both drafts.
If it is not, Studio only changes the open feed.

## An address with a space

A web address must not have a space, and some apps fail on it. A feed from a different tool can have such an address.

The list **Releases** names each address with a space. Select **Correct All** to change each space to `%20` in the draft. Examine the change in Review, then write it. Studio makes this change only when you select the button.


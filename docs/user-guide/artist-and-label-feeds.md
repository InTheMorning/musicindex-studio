# Artist and label feeds

An artist feed lists the releases of an artist. A label feed lists the releases of a label.
Each release can link back to its artist feed or its label feed.
Studio keeps both links correct when you add or remove a link.

In RSS, the two types of feed have the medium `publisher`. Studio shows the role of the feed: **Artist** or **Label**.
A feed with no role shows **Feed** with a hollow dot.

## Make an artist feed or a label feed

You need a host with a public address. See [Connect a host](hosts.md).

1. Select **Releases** in the left panel. Then select **＋ Add a feed**.
2. In the panel **Make an artist feed or a label feed**, select the host in **Host** if you have more than one.
3. Type the path of the new feed file in **Feed file**, for example `my-label.xml`.
4. Type the name in **Name**.
5. Under **Role**, select **Artist** or **Label**.
6. Select **Make the Artist Feed** or **Make the Label Feed**.

Studio opens the new feed as a draft.
Select **Review** and **Write to the host** to make the file. See [Review and write your changes](review-and-write.md).

## The pages of the feed

An artist feed has two pages: **Artist** and **Files**. A label feed has **Label** and **Files**.

The page **Artist** or **Label** has these sections:

- **Artist** or **Label**. Fields: **Title**, **Name**, **Description**, **Cover image address** and **Website**. Select **Choose…** to pick an image file. See [Edit the album details](edit-album.md).
- **Role**. Select **Artist** or **Label**. Studio can find the role in the feed. Then it shows "Studio found this role in the feed."
- **Releases**. The releases of this artist or this label.

## Add a release to an artist feed or a label feed

1. Open the artist feed or the label feed.
2. In **Releases**, select a release in the field **Release**.
3. Select **Add the Release**.

The list shows only releases from your collection that have a host.
Each row says "In the collection" or "Not in the collection".
Use **Move up**, **Move down** and **Remove** to change the list.

A link changes two drafts: the draft of the artist feed or label feed, and the draft of the release.
Studio then shows a notice such as "Studio also changed the draft of ...".
Select the button in the notice to open that feed. Then examine the changes and write them.
Write both feeds to publish both sides of the link.

If the feed has no role, Studio asks you to select the role first.

## Add an artist or a label to a release

1. Open the album.
2. In the section **Artists and labels**, select a feed in the field **Artist or label**.
3. Select **Add the Feed**.

The first feed in the list is the primary one.
The row shows the role and "Primary".
Use **Move up**, **Move down** and **Remove** to change the list.

## Remove a link

Select **Remove** on the row.
If the other feed is in your collection, Studio changes both drafts.
If it is not, Studio only changes the open feed.

# Publisher feeds

A publisher feed lists the releases of an artist or a label.
Each release can link back to its publisher feed.
Studio keeps both links correct when you add or remove a link.

## Make a publisher feed

You need a host with a public address. See [Connect a host](hosts.md).

1. Select **Add a feed** in the left panel.
2. In the panel **Make a publisher feed**, select the host in **Host** if you have more than one.
3. Type the path of the new feed file in **Feed file**, for example `my-label.xml`.
4. Type the name in **Name**.
5. Under **Role**, select **Artist** or **Label**.
6. Select **Make the Publisher Feed**.

Studio opens the new feed as a draft.
Select **Review** and **Write to the host** to make the file. See [Review and write your changes](review-and-write.md).

## The publisher page

A publisher feed has two pages: **Publisher** and **Files**.

The page **Publisher** has these sections:

- **Publisher**. Fields: **Title**, **Name**, **Description**, **Cover image address** and **Website**. Select **Import Image** to use an image file from your computer.
- **Role**. Select **Artist** or **Label**. Studio can find the role in the feed. Then it shows "Studio found this role in the feed."
- **Releases**. The releases of this publisher.

## Add a release to a publisher feed

1. Open the publisher feed.
2. In **Releases**, select a release in the field **Release**.
3. Select **Add the Release**.

The list shows only releases from your collection that have a host.
Each row says "In the collection" or "Not in the collection".
Use **Move up**, **Move down** and **Remove** to change the list.

A link changes two drafts: the draft of the publisher feed and the draft of the release.
Studio then shows a notice such as "Studio also changed the draft of ...".
Select the button in the notice to open that feed. Then examine the changes and write them.
Write both feeds to publish both sides of the link.

If the publisher feed has no role, Studio asks you to select the role first.

## Add a publisher feed to a release

1. Open the album.
2. In the section **Publisher feeds**, select a feed in the field **Publisher feed**.
3. Select **Add the Publisher Feed**.

The first publisher feed in the list is the primary one.
The row shows the role and "Primary".
Use **Move up**, **Move down** and **Remove** to change the list.

## Remove a link

Select **Remove** on the row.
If the other feed is in your collection, Studio changes both drafts.
If it is not, Studio only changes the open feed.

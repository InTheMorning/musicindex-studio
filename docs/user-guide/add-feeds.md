# Add feeds that you already have

Select **Releases** in the left panel. Then select **＋ Add a Feed**.
When Releases has no feed, select **Add a Feed** on the empty page.

The page "Add feeds" has two groups of rows. Select a row to open its sheet.
This page covers the group **Add a feed you have**.
To make a new feed, see [Make a new release](new-release.md).

## Find feeds by address or name

Select the row **By address or name**. A sheet opens.
Feeds that you add this way are read-only until you connect the file host that holds them.

1. Type one feed address on each line in **Feed addresses**. An artist feed or a label feed brings all of its releases.
2. Or type an artist name or a feed name in **Or search musicindex by name** and select **Search**.
3. In the results, select the feeds that are yours.
4. Select **Find My Feeds**. The button shows the number of addresses and feeds.
5. Studio lists what it found, grouped by host. Select the feeds to add.
6. Select **Add N Feeds**.

If Studio cannot read a feed, it lists the feed below "Studio could not read". Each line gives the name of the feed, its site, the cause and what you can do.
Studio stops at 100 feeds. Add the rest in a second search.

A feed that is in your collection shows "in your collection".

To edit a feed that you found, connect its folder. See [Connect a file host](hosts.md).
Some file hosts are services where Studio can only read the feeds.
The page tells you which.

## From a file host folder

If you connected a file host, select the row **From a file host folder**. A sheet opens.

1. If you have more than one file host, select it in **File host**.
2. Under **Feeds in this folder**, select a feed. Studio lists the feed files that it finds, with the type.
3. Select **Add the Feed**.

If the feed is not in the list, select **The Feed Is Not in the List**.
Type the path of the feed file in **Feed file**, for example `album/feed.xml`.
To go back to the list, select **Select from the List**.

If the file is not a feed that Studio can read, Studio shows "This file is not a feed that Studio can read."

## From MSP 2.0 backups

Select the row **From MSP 2.0 backups** to move your hosted feeds from MSP 2.0.

1. Select **Choose Backup Files**.
2. Select one or more .json files that MSP 2.0 saved.
3. Select **Import Backups**.

Studio adds the feeds and keeps the edit tokens in your vault.
If you have no vault, Studio asks you to make one. See [Passphrase, unlock and lock](vault.md).

## Remove a feed from your collection

Open the feed. Open **Feed facts** on the album page and select **Remove from Collection**.
For a read-only feed, select **Remove from Collection** on its page.
Studio asks you to confirm. The feed file on the file host does not change.

## Read a feed again

Select **Read Again** on the page of the feed.
Studio reads the feed file from its address or its file host.

# Add feeds that you already have

Select **Releases** in the left panel. Then select **＋ Add a feed**.
When Releases has no feed, select **Add a Feed** on the empty page.
The page "Add feeds" has one panel for each way to add a feed.
This page covers the panels for feeds that exist.
To make a new feed, see [Make a new release](new-release.md).

## Find feeds by address or name

Use the panel **Find feeds by address or name**.
Feeds that you add this way are read-only until you connect the host that holds them.

1. Type one feed address on each line in **Feed addresses**. An artist feed or a label feed brings all of its releases.
2. Or type an artist name or a feed name in **Or search musicindex by name** and select **Search**.
3. In the results, select the feeds that are yours.
4. Select **Find my feeds**. The button shows the number of addresses and feeds.
5. Studio lists what it found, grouped by host. Select the feeds to add.
6. Select **Add N feeds**.

If Studio cannot read an address, it lists the address under "Studio could not read".
Studio stops at 100 feeds. Add the rest in a second search.

A feed that is in your collection shows "in your collection".

To edit a feed that you found, connect its folder. See [Connect a host](hosts.md).
Some hosts are services where Studio can only read the feeds.
The page tells you which.

## From a connected folder

If you connected a host, use the panel **From a connected folder**.

1. If you have more than one host, select it in **Host**.
2. Under **Feeds in this folder**, select a feed. Studio lists the feed files that it finds, with the type.
3. Select **Add the feed**.

If the feed is not in the list, select **The feed is not in the list**.
Type the path of the feed file in **Feed file**, for example `album/feed.xml`.
To go back to the list, select **Select from the list**.

If the file is not a feed that Studio can read, Studio shows "This file is not a feed that Studio can read."

## From MSP 2.0 backup files

Use the panel **From MSP 2.0 backup files** to move your hosted feeds from MSP 2.0.

1. Select **Choose Backup Files**.
2. Select one or more .json files that MSP 2.0 saved.
3. Select **Import backups**.

Studio adds the feeds and keeps the edit tokens in your vault.
If you have no vault, Studio asks you to make one. See [Passphrase, unlock and lock](vault.md).

## Remove a feed from your collection

Open the feed. Open **Feed facts** on the album page and select **Remove from collection**.
For a read-only feed, select **Remove from collection** on its page.
Studio asks you to confirm. The feed file on the host does not change.

## Read a feed again

Select **Read again** on the page of the feed.
Studio reads the feed file from its address or its host.

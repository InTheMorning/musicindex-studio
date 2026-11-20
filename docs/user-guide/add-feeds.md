# Add feeds that you already have

Select **Releases** in the left panel. Then select **＋ Add** and one of the values: **Release**, **Artist**, **Label**, **Show** or **Playlist**.
The form of that value opens. The first row of the form is **Find on Music Index**.

If you have no file host, the form tells you to connect one. See [Connect a file host](hosts.md).
To make a new feed with no start values, see [Make a new release](new-release.md).

## Find on Music Index

Select **Find on Music Index**. A sheet opens.

1. Type a name in **Name or feed address**. The sheet shows the results that agree with the form. For example, the form of a label shows labels, and not artists.
2. Select a result:
   - A result that is in your collection already opens its page. The row shows "In Studio".
   - A feed on a file host that you connected is yours. Studio adds that feed with its identity and its file.
   - In the form of a release, an artist row shows the releases of that artist. Select **‹ Results** to go back.
   - Each other feed fills the form. The form shows "Copies N tracks and M payment recipients from" the name of the feed.
3. Select the button of the form, for example **Make the Release**.

A copy gets the values, the tracks and the payment recipients of the found feed. It gets a new identity.
Its tracks play from the audio addresses of the found feed until you move the release.

The artist of the found feed can be different from the artist of the form. Then the form tells you that the payments go to the recipients of the found feed. Examine them on **Value Routes** before the write.
If Studio cannot read the feed file, the form gets only the values of Music Index. Studio then tells you that it copies no tracks and no payment recipients.

### Add a feed by its address

Type the feed address in **Name or feed address**, for example `https://example.com/feed.xml`.
Select **Add This Address**. Studio adds that feed.
Feeds that you add this way are read-only until you connect the file host that holds them.

## From a file host folder

If you connected a file host, open **Find on Music Index** and select **Add from a File Host Folder…**.

1. If you have more than one file host, select it in **File host**.
2. Under **Feeds in this folder**, select a feed. Studio lists the feed files that it finds, with the type.
3. Select **Add the Feed**.

If the feed is not in the list, select **The Feed Is Not in the List**.
Type the path of the feed file in **Feed file**, for example `album/feed.xml`.
To go back to the list, select **Select from the List**.

If the file is not a feed that Studio can read, Studio shows "This file is not a feed that Studio can read."

## From MSP 2.0 backups

Open **Find on Music Index** and select **Add from MSP 2.0 Backups…** to move your hosted feeds from MSP 2.0.

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

# Set up Studio for the first time

The first time that you open Studio, it asks you two things.
First, it asks for the library folder.
Then it asks if you have a feed.

On a server, you first make an account. See [Sign in to a Studio server](sign-in.md).
The server sets the library folder of your account, so Studio does not ask for it.
Studio then asks "Do you already have a feed?".

## Choose the library folder

Studio shows the page "Where Studio keeps your releases".
The folder holds the feed file, the images and the audio files of each release.

1. Make sure that the path in the field **Folder** is correct. Studio fills it with a default folder.
2. Select **Use this folder**.

Studio then shows "Studio set up your library".
The page lists the number of release folders, the number of feed files and the number of files that Studio moved.
Select **Continue**.

Studio asks for this folder one time only.
The Settings page shows the folder. You cannot move the library to a different folder in this version.
This step is for the desktop only.

## Answer the first question

Studio asks "Do you already have a feed?" when your collection is empty and you have no file host.

### I have feeds

1. Select **Yes, I have feeds**.
2. Add the address of each feed, or search musicindex by name. See [Add feeds that you already have](add-feeds.md).
3. Select **Find my feeds**.
4. Select the feeds to add. Then select **Add** with the number of feeds.
5. Select **Open Studio**.

A feed that you add by address is read-only.
To edit it, connect the folder that holds the feed files. See [Connect a file host](hosts.md).

### I am new

1. Select **No, I am new**.
2. Read the page "Welcome". Then select **Open Studio**.

To make a new release, first connect a file host. Then select **Album** on the page **Add a feed**.
See [Make a new release](new-release.md).

### I used MSP 2.0

Select **Import from MSP 2.0**.
Then select **Choose Backup Files** and select the .json files that MSP 2.0 saved.
Select **Import backups** to add the feeds.
Studio keeps the edit tokens from the backup files in your vault.
If you have no vault, Studio asks you to make one. See [Passphrase, unlock and lock](vault.md).
On a server, the vault exists from the sign-up. If it is locked, Studio asks you to unlock it.

## Open Studio later

On the next start, Studio opens the Releases page.
If your collection is empty and you have no file host, Studio asks "Do you already have a feed?" again.
On the desktop, if you made a vault, Studio asks for your passphrase first.
On a server, you sign in first.

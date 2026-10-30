# How Studio works

Read this page first. The other pages use these words.

## Feed

A feed is an XML file on the internet. It lists your albums and tracks.
Music apps read the feed to find your music.
Studio edits the feed file and keeps the feed GUID and each track GUID the same.

Studio opens these feed types for editing:

- Albums
- Artist feeds and label feeds
- Playlists

Studio can read other feeds, but it cannot edit them.

## File host

A file host is a location where your files are.
Studio does not host files. It connects to your file hosts.

Studio can connect to these file hosts:

- A folder on this computer (desktop only)
- An S3 or R2 bucket
- An SFTP server
- A Blossom server
- An nsite site

A feed that Studio can write has a file host. A feed with no file host is read-only.
See [Connect a file host](hosts.md).

## Collection

Your collection is the list of feeds that you added to Studio.
The Releases page shows your collection.

## Library folder

Studio keeps one folder for each release in the library folder.
The folder holds the feed file, the images and the audio files of the release.
On the desktop, the library folder is on your computer. Studio asks for it the first time that you start it.
On a server, the library folder is in your account on the server.

## Draft, review and write

When you change a feed, you do not change the hosted file immediately.
Studio keeps your changes in a draft. Studio saves the draft for you.

The hosted file changes only when you write.
Before you write, you open **Review** and examine each change.
Then you select **Write to the File Host**.
See [Review and write your changes](review-and-write.md).

Before each write, Studio keeps the file that the write replaces. This file is an earlier version.
You can go back to an earlier version.

## Vault

The vault holds your secrets, for example the access key of a bucket.
Only your passphrase opens the vault.
On the desktop, Studio asks you to make the vault when you add the first secret.
On a server, Studio makes the vault when it makes your account.
See [Passphrase, unlock and lock](vault.md).

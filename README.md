# Music Index Studio

Edit your music feeds in one location, and write each change to your file host.

![The release page of Music Index Studio in the dark theme, with the track list on the left](docs/user-guide/images/release-page.png)

## Who it is for

Studio is for an artist or a label that publishes music as Podcasting 2.0 feeds.
You keep your albums, your artist page and your label page in Studio. You see each change before Studio writes it.

## Try it

1. Download the installer for your computer from the newest release on the [GitHub Releases](https://github.com/InTheMorning/musicindex-studio/releases) page, and open it. On Linux, use the AppImage: make it executable (`chmod +x` and the file name), then open it. On macOS and Windows, the installers have no code signature: [Start Music Index Studio](docs/runbooks/install-desktop.md) tells you how to open the app the first time.
2. On the page "Where Studio keeps your releases", select **Use This Folder**, then **Continue**.
3. Select **I make music**. Type your artist name, then select **Continue**.
4. Select **Folder on this computer**, then **Make the Page**. Studio makes your artist page.
5. Select **Review**, then **Write to the File Host**. Studio writes your artist page as a feed file in the folder `~/musicindex/pages` on your computer.

This folder is not on the web. To publish your pages, select **Put It Online…** in the notice after the write, or see [Connect a file host](docs/user-guide/hosts.md).

Next, add your first album: see [Start your artist page or label page](docs/user-guide/first-steps.md).
To start Studio from the source code, see [Start Studio on your computer](docs/user-guide/start-on-desktop.md).

## See it

The artist page holds your releases:

![The artist page of a test band, with its three releases](docs/user-guide/images/artist-page.png)

Review shows each change before Studio writes it:

![The review of a change to the description of a release](docs/user-guide/images/review.png)

Value routes tell podcast apps where to send the payments of listeners:

![The value routes of a release, with two recipients](docs/user-guide/images/value-routes.png)

## What it does today

- Opens your albums, playlists, artist pages and label pages, and edits each field that MSP 2.0 edits.
- Makes new releases from your audio files, with an MP3 file and a lossless alternative.
- Writes to a folder on your computer, SFTP, S3 and R2, Blossom and nsite.
- Shows each change before it writes, and keeps each earlier version of a feed. Undo and redo work on each edit before the write.
- Gives a checklist of what a release needs before you publish it.
- Edits the value routes (the Lightning payments) and the credits of your feeds, and keeps a list of the persons that you pay or credit.
- Moves a release to a different file host, and keeps the files on the first file host until you remove them.
- Checks a feed for problems, and gives a repair.
- Runs live shows, and lets a connected app change a live show while you stay in control.
- Sends a podping after a write, so that podcast apps show the change in minutes.

Studio also runs on a server, for use from a phone. See [Use Studio from a phone](docs/user-guide/use-on-a-phone.md).

## Documentation

- [The Studio user guide](docs/user-guide/README.md): all pages, grouped by task.
- Learn: [How Studio works](docs/user-guide/how-studio-works.md)
- Do a task: [Make a new release](docs/user-guide/new-release.md), [Add feeds that you already have](docs/user-guide/add-feeds.md), [Review and write your changes](docs/user-guide/review-and-write.md)
- Look up: [Connect a file host](docs/user-guide/hosts.md), [Value routes](docs/user-guide/value-routes.md)
- Understand: [Passphrase, unlock and lock](docs/user-guide/vault.md)

For a developer or a host operator: [the project documents](docs/README.md).

## The suite

| Project | How it connects to Studio |
| --- | --- |
| Music Index (musicindex.org) | Studio searches it to find artists, labels and persons. |
| Stophammer | The index behind Music Index. It reads the feeds that Studio writes. |
| HitIt | A player for listeners and show hosts. It plays the feeds that Studio writes. |
| musicindex-live-publisher | A connected app. It changes a live show in Studio with a show grant. |

## Status

Studio 0.1.0 is in testing. Each release on the GitHub Releases page is a release candidate. Keep a copy of your data folder before each update. [Start Music Index Studio](docs/runbooks/install-desktop.md) gives the folder of each system.

## License

AGPL-3.0. See [LICENSE](LICENSE).

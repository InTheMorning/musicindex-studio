# Connected apps and show grants

A connected app is an application that asks Studio to change a live show.
Studio keeps the vault, the collection and the write.
The other app does not get a secret from your vault.

A show grant lets a connected app change one show without your approval of each change.
Without a grant, Studio writes only changes that you reviewed.


## Connect an app

1. Select **Settings** in the left panel.
2. In the panel **Connected apps**, select **Connect an App**.
3. Type a name in **Name of the app**.
4. Select **Make the Token**.
5. Studio shows the token. Give it to the app now. Studio does not show it again.
6. Select **Copy** to copy the token. Select **Done** when you finish.

The app uses the token to send its requests to Studio.
The token works for connected apps only. It does not open the Studio pages.

On the desktop, the app must be on the same computer as Studio.
On a server, the app sends its requests to the address of the server, for example `https://studio.musicindex.org`.
The app can then be on a different computer. The server permits 60 requests for each app in each minute.

An app from an earlier version of Studio has no key.
Its line shows "This app has no key, so it cannot use a grant."
Select **Connect Again** to give the app a new token. Studio asks you to confirm. The token from before stops working.

## Remove an app

1. Select **Remove** on the line of the app in the panel **Connected apps**.
2. Confirm. The token of the app stops working.

## Give a grant for a show

You give a grant when you write a planned show.

1. Open the page of a planned show. See [Live shows](live-shows.md).
2. Select **Review** in the toolbar.
3. Select the check box "Let connected apps start and end this show, and set its live value".
4. Select **Write to the File Host**.

If you have no connected app, the sheet shows "No app is connected. Connect an app in Settings."

A file host that uses **My signer** cannot do a granted write. During a granted write, no person can use the signer.
If the feed file is on such a file host, the check box is disabled.

## What a grant allows

With a grant, a connected app can:

- Start the show.
- End the show.
- Set the live value.
- Remove the live value.

The grant ends 24 hours after the end time of the show.
If the show has no end time, it ends 24 hours after the start time.
The grant can end before this time.
It ends when the show has the status **Ended**, leaves the feed, or you stop the grant.

A granted change is refused in these cases:

- The feed has a draft with changes. Write or discard the draft first.
- The grant ended.

A granted write works while the vault is locked.
Each grant has a small grant vault. It holds a copy of the secrets of the file host of the feed file only.

## See what a grant did

Open the page of the show. The section **Grant** tells you the state of the grant.
It lists each granted write with the app, the change and the time.

## Stop a grant

1. Open the page of the show.
2. In the section **Grant**, select **Stop the Grant**.
3. Confirm.

Connected apps cannot change the show.

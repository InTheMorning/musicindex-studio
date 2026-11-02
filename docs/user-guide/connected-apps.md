# Connected apps and show grants

A connected app is an application that asks Studio to change a live show.
Studio keeps the vault, the collection and the write.
The other app does not get a secret from your vault.

Each app has its own key pair. The app makes the key pair and keeps the private key.
Studio keeps only the public key of the app. Each request of the app has a signature from its private key.

A show grant lets a connected app change one show without your approval of each change.
Without a grant, Studio writes only changes that you reviewed.

## Connect an app

1. Select **Settings** in the left panel.
2. In the panel **Connected apps**, select **Connect an App**.
3. Type a name in **Name of the app**. Use a name for this computer or this copy of the app. For example: "Live Publisher (studio PC)".
4. Select **Make Pairing Text**. Studio asks you to unlock the vault if it is locked.
5. Select **Copy**. Studio shows the pairing text as dots. Studio shows "Copied" when the text is on the clipboard.
6. Paste the pairing text into the app in 15 minutes. For example, run `musicindex-live-publisher studio connect` on your own computer.
7. Wait for the app to say that it is connected. The notice in Studio then shows "is connected" and the short key of the app.
8. Compare the key in the line of the app with the key that the app shows.
9. Select **Done**.

The pairing text holds the address of Studio and a code, for example `https://studio.musicindex.org#pair=K7M2-Q9XD`.
The code works one time, for 15 minutes. After that, Studio removes the waiting app and its code.
The line of a waiting app shows the time when the pairing text ends. It does not show the code.

On the desktop, the app must be on the same computer as Studio.
On a server, the app sends its requests to the address of the server, for example `https://studio.musicindex.org`.
The app can then be on a different computer. The server permits 60 requests for each app in each minute.

## Keep the pairing text secret

- The pairing text is a secret for 15 minutes. Do not send it in a chat or an email.
- Paste the pairing text only into the app on your own computer.
- Use **Copy**. Do not select the eye button while a person can see your screen.
  This includes a screen share and a recording.
- Studio writes to the clipboard only when you select **Copy**.
- Select **Done** only after the app says that it is connected. **Done** before that ends the pairing text.
- After the app pairs, the pairing text is not in the page. **Done** then only closes the notice.

When the app says that the pairing text was already used, a different program possibly used it.

1. Select **Remove** on the line of that app in Studio.
2. Make a new pairing text.

## Use one pairing for each computer

- Make one pairing for each computer or copy of the app. Give each one its own name.
- A grant gives one app one show. To use a different copy, approve the show again and select that copy.

## What the line of an app shows

Each app has a line in the panel **Connected apps**.

- The name of the app.
- **Key**. The short form of the public key of the app. A waiting app shows "No key yet".
- The number of shows that you gave to the app, and **Last used**. **Last used** shows the time and the address of the last request.
- **Live value address**. The address where the listeners of the app get the live value.

To change the live value address, remove the app and connect it again.

## Connect an app again

An app from an earlier version of Studio has a token, not a key pair.
Its line shows "Token from before key pairs" and "Studio no longer accepts tokens."

1. Select **Connect Again** on the line of the app.
2. Studio removes the line from before and makes a pairing text for the same name.
3. Paste the pairing text into the app, as in [Connect an app](#connect-an-app).

## Remove an app

1. Select **Remove** on the line of the app in the panel **Connected apps**.
2. Confirm. The grants of the app end at once.

**Remove** also stops a waiting app. Use it when a computer is lost or a key is stolen.
The app can then not change any show.

## Give a grant for a show

You give a grant when you write a planned show.

1. Open the page of a planned show. See [Live shows](live-shows.md).
2. Select **Review** in the toolbar.
3. Select the app for the show.
4. Select **Write to the File Host**.

The control in the review sheet depends on the number of connected apps:

- One app. Select the check box "Let (name of the app) start and end this show, and set its live value". The sheet shows the live value address of the app.
- Two or more apps. Select the app in **Connected app for this show**. The default is **No app**.
- No app. The menu is disabled. The sheet shows "No app is connected. Connect an app in Settings."

A grant names one app. No other app can change the show.
A file host that uses **My signer** cannot do a granted write. During a granted write, no person can use the signer.
If the feed file is on such a file host, the control is disabled.

## What a grant allows

With a grant, the app can change only the show that the grant names:

- Start the show.
- End the show.
- Set the live value.
- Remove the live value.

The app cannot change anything else. It cannot read your vault, change other shows or change other feeds.
It can set a live value only on the live value address that you saw in the review sheet.

The grant ends 24 hours after the end time of the show.
If the show has no end time, it ends 24 hours after the start time.
The grant can end before this time.
It ends when the show has the status **Ended**, leaves the feed, you stop the grant or you remove the app.

A granted change is refused in these cases:

- The feed has a draft with changes. Write or discard the draft first.
- The grant ended.
- The live value does not pass the check for the live value address.
- The grant vault does not open.

A granted write works while the vault is locked.
Each grant has a small grant vault. It holds a copy of the secrets of the file host of the feed file only.

## See what a grant did

Open the page of the show. The section **Grant** tells you the state of the grant.
It lists each granted write with the app, the change and the time.
A refused change shows the app, what did not happen, the cause and the time.
For example: "Live Publisher · Not started: the feed had a draft with changes".

**Last used** on the line of the app tells you when the app last sent a request.
If you do not know that time, remove the app and connect it again.

## Stop a grant

1. Open the page of the show.
2. In the section **Grant**, select **Stop the Grant**.
3. Confirm.

Connected apps cannot change the show.

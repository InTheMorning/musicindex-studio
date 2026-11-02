# Your account on a server

This page is for Studio on a server.
The desktop app has no account.

Select **Settings** at the bottom of the left panel. The panel **Account** is on the Settings page.

## What the panel shows

- **Email address** or **Nostr key**. This is the name of your account.
- **Move this account**. This part has **Export Account**.
- **Passkeys**. This list shows only when your browser can use passkeys.
- **Sign Out** and **Delete the Account**.
- A notice, when the account came from an import. See [Import an account](#import-an-account).

## Add a passkey

A passkey lets you sign in and open the vault in one step.
Use iCloud Keychain or Google Password Manager. Some other passkey managers cannot open the vault.

1. Type a name in **Name of the new passkey**, or keep the name that Studio gives.
2. Select **Add a Passkey**.
3. Confirm the passkey on your device.

If the vault is locked, Studio asks you to unlock it first.
Each passkey shows in the list with its name and the date when you added it.

## Remove a passkey

Select **Remove** on the line of the passkey.
If the vault is locked, Studio asks you to unlock it first.

You cannot remove the last sign-in method of your account.
Studio then shows "This is the last way to sign in to the account. Add a different method first."

## Make a recovery key

A recovery key opens the vault if you forget your passphrase.
It also opens the vault on a different server, when you move the account.
Studio shows a recovery key one time only. Write it down and keep it safe.

You can make a new recovery key when you export the account. The export needs it when only passkeys open your vault.

## Export the account

An export makes one file with your feeds, drafts, versions, people, vault and connected apps.
Use the file to move to a different server, or to Studio on your computer.
The export changes nothing. Your account stays on this server until you remove it.

1. Select **Export Account** in **Move this account**.
2. If the vault is locked, or if you did not unlock it in the last 5 minutes, Studio asks you to unlock it.
3. Read the facts in the sheet. The sizes are estimates.
4. Select **Export Account**.
5. Keep the file that your browser downloads.

The file holds your feeds, drafts and people with no encryption. Keep the file safe.
Your vault stays encrypted in the file.

If your vault opens only with passkeys, the sheet says that the vault cannot open on a different server.
A passkey works only on the server where you made it.
Select **Make a Recovery Key**. Studio shows the key one time. Write it down, select the check box, and select **Continue**.
Then the sheet shows the facts of the export.

A server can keep no audio and no images. Then the file has no audio and no images.
On a server that keeps them, the sheet has the check box **Include the audio and images**.

## Import an account

You can import a file of **Export Account** when you make an account on a Studio server.
The file can come from a different server or from Studio on a computer.

1. On the page **Make an account**, select **Import an Account**.
2. If the server needs an invite code, type it.
3. Select **Select the File**, and select the file. Studio sends the file and checks it.
4. Read the facts of the file: its account, its date and its content.
5. Open the vault of the file with your passphrase, or select **Use the Recovery Key**. A Nostr vault opens with a signer.
6. If the file has no email address, type the email address of the new account.
7. Select **Open and Make the Account**.

The import changes no feed and writes nothing to your file hosts.
If this server has an account with the same email address or Nostr key, Studio refuses the import.
If the file is not a correct file of **Export Account**, Studio shows the cause.

After an import, the account has no passkey and no grant.
The Account panel shows the notice "This account came from a file."
Add a passkey for this server, and connect each app again with the address of this server.
The notice stays until you add a passkey or select **Close**.

## Sign out

Select **Sign Out**.
Studio locks the vault and shows the page **Sign in**.
Your drafts stay on the server.

## Delete the account

1. Select **Delete the Account**.
2. Type your email address or your npub in the field.
3. Select **Yes, Delete the Account**.
4. If the vault is locked, unlock it.

Studio removes your feeds, your vault and your library files from the server. You cannot get them back.
Studio does not change the feed files on your file hosts.
To stop, select **Cancel**.

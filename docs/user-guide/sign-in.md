# Sign in to a Studio server

This page is for Studio on a server, for example `https://studio.musicindex.org`.
On a server, you do not start Studio. You open the address of the server in your web browser.
Studio then shows the page **Sign in**.

To use Studio on your computer, see [Start Studio on your computer](start-on-desktop.md).

## Make an account

1. Open the address of the server.
2. Select **Make an account**.
3. Type your email address in **Email address**.
4. If the server asks for it, type the code that you got from the operator in **Invite code**.
5. Type a passphrase in **Passphrase**. The rules are the same as on the desktop. See [Passphrase, unlock and lock](vault.md).
6. Decide if you want a recovery key. The check box "Make a recovery key, in case I forget the passphrase" is on by default.
7. Select **Make the account**.

The server sends no email. Your email address is only the name of your account.

You can also make an account with a Nostr signer:

- Select **Sign up with a signer extension**. This button shows only when your browser has a NIP-07 signer extension.
- Select **Sign up with Amber or a bunker**. See [Connect a remote signer](#connect-a-remote-signer).

If the server asks for an invite code, type it first. The two buttons are disabled until you type it.

If you chose a recovery key, Studio shows it one time only.
Write it down and keep it in a safe location. Then select the check box "I wrote down the recovery key in a safe place" and select **Continue**.

Studio makes your vault when it makes the account.

## Sign in

Select one of these methods on the page **Sign in**:

| Method | What to do |
| --- | --- |
| Email and passphrase | Type them in **Email address** and **Passphrase**. Then select **Sign in**. |
| Signer extension | Select **Sign in with a signer extension**. Your signer asks you to sign the request. |
| Passkey | Select **Sign in with a passkey**. First add a passkey in the Account panel. See [Your account on a server](account.md). |
| Remote signer | Select **Sign in with Amber or a bunker**. See [Connect a remote signer](#connect-a-remote-signer). |

A sign-in with the passphrase or a passkey also opens the vault.
A sign-in with a Nostr signer does not open the vault. Studio asks for an unlock when a step needs a secret.
See [Passphrase, unlock and lock](vault.md).

A session ends after 7 days with no use, or after 30 days in total.
It also ends when you select **Sign out**. The operator can change the two limits.
After the end of a session, Studio shows "Your session ended. Sign in again."

## Connect a remote signer

A remote signer is a signer app that is not in the browser, for example Amber on a phone.
Studio connects to it through a relay with NIP-46.

1. Studio shows a QR code.
2. Scan the QR code with your signer app. On a phone, select **Open in the signer app**.
3. Or paste the address from your signer in **Bunker address** and select **Connect**. A bunker address starts with `bunker://`.
4. Allow the request in your signer app.

While Studio waits, it shows "Waiting for the signer…".
Select **Stop** to go back.

## Sign in with the recovery key

Use the recovery key when you cannot use a sign-in method of your account.

1. On the page **Sign in**, select **Sign in with the recovery key**.
2. Type your email address or your npub in **Email address or npub**.
3. Type the key in **Recovery key**.
4. Select **Sign in**.

Studio then shows the page **Add a sign-in method**.
You must add a method before you can do other work:

- For an account with an email address, type a passphrase in **New passphrase** and select **Save the passphrase**.
- For an account with a Nostr key, select **Use the signer extension**.
- Select **Use a passkey** to add a passkey.

## Error messages

| Message | What to do |
| --- | --- |
| "The email address or the passphrase is not correct." | Type them again. |
| "The account or the recovery key is not correct." | Examine the account name and the key. |
| "Another account has this email address." | Sign in with that account, or use a different address. |
| "Sign-up needs an invite code that nobody used." | Get a new code from the operator. |
| "Too many sign-in attempts. Try again later." | Wait. The server permits 10 sign-ins that are not correct for each account in each hour. |
| "The server is busy. Try again in a short time." | Try again after some seconds. |
| "This passkey manager cannot open the vault. Use iCloud Keychain or Google Password Manager, or a different method." | Use a different passkey manager or a different method. |
| "The passkey request was cancelled or timed out. Try again." | Try again, and approve the request on your device. |
| "The signer gave no answer in time. Open the signer app and try again." | Open the signer app, then try again. |

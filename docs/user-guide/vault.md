# Passphrase, unlock and lock

The vault holds your secrets. Examples are the access key of a bucket and the private key of an SFTP login.
Only your passphrase opens the vault.
Studio cannot open it for you.

## When Studio makes the vault

On a server, Studio makes the vault when it makes your account. See [Sign in to a Studio server](sign-in.md).

On the desktop, Studio makes the vault when you add the first secret.
A folder on this computer needs no secret, so it needs no vault.
These actions need a secret:

- Add an S3 or R2 bucket
- Add an SFTP server with a private key or a password
- Add an nsite site or a Blossom server with "Key in the vault"
- Import MSP 2.0 backup files

## Make the vault

Studio shows the page "Protect your keys".

1. Type a passphrase in the field **Passphrase**.
2. Type the same passphrase in **Type the passphrase again**.
3. Decide if you want a recovery key. The check box is on by default.
4. Select **Make the vault**.

The passphrase needs a minimum of 12 characters.
Studio also refuses a passphrase that is too easy to guess.
A meter shows the strength. The button works when Studio accepts the passphrase and the two fields are the same.

### Write down the recovery key

If you chose a recovery key, Studio shows it one time only.

1. Write the key on paper and keep it in a safe location.
2. Select the check box "I wrote down the recovery key in a safe place".
3. Select **Continue**.

On the desktop, the Unlock page asks for the passphrase only.
Keep your passphrase safe.
On a server, you can sign in with the recovery key. See [Sign in to a Studio server](sign-in.md).

## Unlock Studio on the desktop

When a vault exists, Studio asks for the passphrase each time that it starts.

1. Type the passphrase in the field **Passphrase**.
2. Select **Unlock**.

If the passphrase is incorrect, Studio shows "The passphrase is not correct."

## Lock Studio

Select **Lock Studio** at the bottom of the left panel.
On a narrow window, it is at the bottom of the first page.
The button shows only when the vault is open.

On the desktop, the Unlock page then opens.
The desktop app does not lock the vault by itself.

## The vault on a server

On a server, a session and an open vault are two different states.
With a locked vault, you can read your collection and edit a draft.
A step that needs a secret, such as a write to a host, needs an open vault.

The server locks the vault in these cases:

- 10 minutes after the last use of a secret. The operator can change this time.
- When you select **Lock Studio**.
- When you sign out, or when your last session ends.
- When the server stops.

When the vault is locked, the left panel shows **Unlock Studio**, not **Lock Studio**.

### Unlock on a server

When a step needs the vault, Studio shows the sheet **Unlock Studio**.
You can also select **Unlock Studio** in the left panel.
The sheet shows the methods of your account that this browser can use:

- **Passphrase**, then **Unlock**. For an account with an email address.
- **Unlock with a signer extension**. For an account with a Nostr key, in a browser with a NIP-07 signer extension.
- **Use the remote signer**. When you signed in with a remote signer in this session. Allow the request in your signer app.
- **Use a passkey**. When your account has a passkey and the browser can use it.

After the unlock, Studio does the step again.
Select **Cancel** to close the sheet. The step then does not occur.

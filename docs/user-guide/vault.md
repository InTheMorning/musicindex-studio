# Passphrase, unlock and lock

The vault holds your secrets. Examples are the access key of a bucket and the private key of an SFTP login.
On a computer, your passphrase opens the vault. You can also set Studio to unlock it from the keyring of this computer. On a server, your account methods can unlock the vault.

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
4. Select **Make the Vault**.

The passphrase needs a minimum of 12 characters.
Studio also refuses a passphrase that is too easy to guess.
A meter shows the strength. The button works when Studio accepts the passphrase and the two fields are the same.

### Write down the recovery key

If you chose a recovery key, Studio shows it one time only.

1. Write the key on paper and keep it in a safe location.
2. Select the check box "I wrote down the recovery key in a safe place".
3. Select **Continue**.

On the desktop, the Unlock page asks for your passphrase. If you set **Unlock on this computer**, Studio can use the computer keyring at startup. On a server, sign-in with a recovery key opens the vault. See [Sign in to a Studio server](sign-in.md).

## Unlock Studio on the desktop

Studio asks for your passphrase unless **Unlock on this computer** opens the vault from the computer keyring.

1. Open **Settings**, then **General**, then **Lock**.
2. Set **At start** to **Unlock on this computer**.
3. Select **Save**.

To unlock with your passphrase, type it in **Passphrase**, then select **Unlock**.

If the passphrase is incorrect, Studio shows "The passphrase is not correct."

## Lock Studio

Select **Lock Studio** at the bottom of the left panel.
On a narrow window, it is at the bottom of the first page.
The button shows only when the vault is open.

On the desktop, the Unlock page then opens.
To lock the vault by itself, open **Settings**, then **General**, then **Lock**. Change **Lock Studio after**, then select **Save**. The start value on the desktop is **Never**. When Studio then needs a key, it asks for your passphrase.

## The vault on a server

On a server, a session and an open vault are two different states.
With a locked vault, you can read your collection and edit a draft.
A step that needs a secret, such as a write to a file host, needs an open vault.

The server locks the vault in these cases:

- After the last use of a secret, when the time at **Settings**, **Lock Studio after** is over. The start value is 10 minutes. The operator of the server sets the longest time.
- When you select **Lock Studio**.
- When the last session of your account ends, also after **Sign Out**.
- When the server stops.

When the vault is locked, the left panel shows **Unlock Studio**, not **Lock Studio**.

### Unlock on a server

When a step needs the vault, Studio shows the sheet **Unlock Studio**.
You can also select **Unlock Studio** in the left panel.
The sheet shows the methods of your account that this browser can use:

- **Passphrase**, then **Unlock**. For an account with an email address.
- **Unlock with a Signer Extension**. For an account with a Nostr key, in a browser with a NIP-07 signer extension.
- **Use the Remote Signer**. When you signed in with a remote signer in this session. Allow the request in your signer app.
- **Use a Passkey**. When your account has a passkey and the browser can use it.

After the unlock, Studio does the step again.
Select **Cancel** to close the sheet. The step then does not occur.

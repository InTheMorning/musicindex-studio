# Passphrase, unlock and lock

The vault holds your secrets. Examples are the access key of a bucket and the private key of an SFTP login.
Only your passphrase opens the vault.
Studio cannot open it for you.

<!-- PHASE-12-GAP: explain the vault window on the server, the lock after 10 minutes, and the unlock methods other than the passphrase; source: ADR 0055 items 14 to 17, ADR 0056 -->

## When Studio makes the vault

Studio makes the vault when you add the first secret.
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

The Unlock page in this version asks for the passphrase only.
Keep your passphrase safe.

## Unlock Studio

When a vault exists, Studio asks for the passphrase each time that it starts.

1. Type the passphrase in the field **Passphrase**.
2. Select **Unlock**.

If the passphrase is incorrect, Studio shows "The passphrase is not correct."

## Lock Studio

Select **Lock Studio** at the bottom of the left panel.
On a narrow window, it is at the bottom of the first page.
The button shows only when the vault is open.

After you lock Studio, the Unlock page opens.
Studio does not lock the vault by itself.

# Connect a file host

A file host is where your feed files and audio files live.
Studio needs a file host to write a feed. See [How Studio works](how-studio-works.md).

Select **File hosts** at the bottom of the left panel.
The page lists the file hosts that you connected.
If you have no file host, the page shows the form "Connect your first file host".
Otherwise select **＋ Add a File Host**.

On a server, the type **Folder on this computer** is not available.
The server also does not connect to an address in a private network, such as `192.168.1.10` or `localhost`.
It then shows "Studio on a server cannot connect to HOST, because it is not a public address."

## Choose the type of file host

Under **Type of file host**, select one of these:

| Type | Use it when |
| --- | --- |
| Folder on this computer | A web server publishes a folder of this computer. Desktop only. |
| S3 or R2 bucket | Your files are in an S3 bucket or a Cloudflare R2 bucket. |
| SFTP server | You upload files to a web server with SFTP. |
| nsite site | You publish a static site on Nostr. |
| Blossom server | You keep media files on a Blossom server. |

The public address of a file host is the web address that publishes its files.
Studio needs it to know the address of each feed.

## Folder on this computer

1. Type a name in **Name (optional)**. If you leave it empty, Studio uses the folder name.
2. Type the full path of the folder in **Folder**. For example `/home/me/feeds`.
3. If a web server publishes the folder, type its address in **Public address of the folder (optional)**.
4. Select **Add the folder**.

If the folder does not exist, Studio shows "Studio did not find this folder on this computer."

## S3 or R2 bucket

1. Select **S3 or R2 bucket**.
2. Fill in the fields:
   - **Name (optional)**. Studio uses the bucket name if you leave it empty.
   - **Endpoint address**. This is the S3 API address from your provider.
   - **Region**. R2 uses `auto`. AWS uses the region of the bucket, for example `us-east-1`.
   - **Bucket**
   - **Folder in the bucket (optional)**
   - **Address style**. Select **Path style** or **Virtual host style**.
   - **Public address of the bucket (optional)**. R2 needs a custom domain for a public feed.
   - **Access key ID**
   - **Secret access key**. Studio keeps this key in your vault.
3. Select **Add the bucket**.

Studio tests the bucket before it adds it.
If you have no vault, Studio asks you to make one first. See [Passphrase, unlock and lock](vault.md).

Studio can show these errors:

- "The provider refused the access key."
- "Studio cannot connect to the endpoint."
- "Studio cannot write to this provider safely." Studio needs a provider that supports safe writes.

## SFTP server

This form has two steps.

1. Select **SFTP server**.
2. Fill in the fields:
   - **Name (optional)**
   - **Server**, for example `sftp.example.com`
   - **Port**. The default is 22.
   - **User name**
   - **Folder on the server**. This is the folder that holds your feed files. It is often not the first folder after the login.
   - **Public address of the folder (optional)**
3. Under **Login**, select **Private key**, **Password** or **ssh-agent**.
   - For **Private key**, paste the text of the key file in **Private key**. If the key has a passphrase, type it in **Passphrase of the key (optional)**. Studio keeps both in your vault.
   - For **Password**, type it in **Password**. Studio keeps it in your vault.
   - For **ssh-agent**, Studio uses the keys of the ssh-agent on this computer. Studio keeps no secret.
4. Select **Connect**.
5. Studio shows the host key fingerprint of the server. Make sure that it is the fingerprint of your server.
6. Select **Accept and add the server**. To change a field, select **Back**.

Studio checks the host key each time that it connects.
If the server gives a different key, the host card shows "The server gives a new host key."
Check the fingerprint. Then select **Accept the new key**.

## nsite site

An nsite site needs a Nostr key.

1. Select **nsite site**.
2. Under **Signing**, select how Studio gets the signature:
   - **My signer**. Select **Use my signer**. Studio asks the signer for your public key. Studio uses the NIP-07 signer extension of the browser. If the browser has none, Studio shows a QR code to connect a remote signer, such as Amber. See [Sign in to a Studio server](sign-in.md#connect-a-remote-signer).
   - **Key in the vault**. Paste the private key (the nsec) in **Private key of the site**. Studio keeps it in your vault.
3. Studio shows your public key (the npub). It also fills **Relays** and **Blossom servers** from the lists of your key. If the key has no lists, type the addresses.
4. Fill in the other fields:
   - **Name (optional)**
   - **Release (optional)**. Select the release that this site publishes.
   - **Site name**. Use 1 to 13 characters: a to z, 0 to 9 and -. A site that exists keeps its name.
   - **Relays**. Type one address on each line.
   - **Blossom servers**. Type one address on each line.
   - **Gateway**. The default is `nsite.lol`.
   - **Title of the site (optional)**
5. Select **Add the site**.

When you use **My signer**, Studio asks your signer to sign each write. See [Review and write your changes](review-and-write.md).

## Blossom server

A Blossom file host keeps media only. The feed XML needs a different file host.

1. Select **Blossom server**.
2. Select **My signer** or **Key in the vault**, as for an nsite site.
3. Fill in **Name (optional)** and **Blossom servers**. Type one address on each line.
4. Select **Add the Blossom File Host**.

## After you add a file host

If you added feeds by address before, Studio matches them with the new file host.
Studio then shows "Studio can now write N feeds through" the file host.
Those feeds are no longer read-only.

Studio can also list feed files in the folder that are not in your collection.
Select the feeds and select **Add N feeds**.

## Change or remove a file host

Each file host has a card on the File hosts page. The card lists the settings of the file host and the feeds that use it.
The line **Different addresses** shows when the address of a feed does not agree with the public address of the folder. Select the feed to open it.

- Select **Edit** to change the name and the public address. For an nsite site, you can also change the relays, the Blossom servers and the gateway. Then select **Save**.
- Select **Check the connection** on an SFTP card to test the login.
- Select **Check the relays** on an nsite card. Studio tells you if each relay has the newest manifest.
- Select **Remove** to remove the file host. Studio asks you to confirm. The files in the file host do not change.

You cannot remove a file host while feeds use it.
Studio shows "Feeds use this file host." Remove those feeds from your collection first.

## Studio settings

The settings of Studio are not on the File hosts page. Select **Settings** at the bottom of the left panel. The Settings page has these panels:

- **Account**, on a server only. See [Your account on a server](account.md).
- **Connected apps**. See [Connected apps and show grants](connected-apps.md).
- **Announcements**. See [Announcements](announcements.md).
- **Library**. On the desktop, it shows the library folder. You can set the largest download and the free space to keep, in MB. Select **Save the limits**.

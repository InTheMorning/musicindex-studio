# Connect a host

A host is where your feed files and audio files live.
Studio needs a host to write a feed. See [How Studio works](how-studio-works.md).

Select **Hosts** at the bottom of the left panel.
The page lists the hosts that you connected.
If you have no host, the page shows the form "Connect your first host".
Otherwise select **＋ Add a host**.

<!-- PHASE-12-GAP: say that the host type "Folder on this computer" is not available on the server, and that the server refuses addresses in a private network; source: ADR 0055 item 24, ADR 0057 -->

## Choose the type of host

Under **Type of host**, select one of these:

| Type | Use it when |
| --- | --- |
| Folder on this computer | A web server publishes a folder of this computer. |
| S3 or R2 bucket | Your files are in an S3 bucket or a Cloudflare R2 bucket. |
| SFTP server | You upload files to a web server with SFTP. |
| nsite site | You publish a static site on Nostr. |
| Blossom server | You keep media files on a Blossom server. |

The public address of a host is the web address that publishes its files.
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
   - **My signer**. Select **Use my signer**. Your browser needs a NIP-07 signer extension. Studio asks the signer for your public key.
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

A Blossom host keeps media only. The feed XML needs a different host.

1. Select **Blossom server**.
2. Select **My signer** or **Key in the vault**, as for an nsite site.
3. Fill in **Name (optional)** and **Blossom servers**. Type one address on each line.
4. Select **Add the Blossom host**.

## After you add a host

If you added feeds by address before, Studio matches them with the new host.
Studio then shows "Studio can now write N feeds through" the host.
Those feeds are no longer read-only.

Studio can also list feed files in the folder that are not in your collection.
Select the feeds and select **Add N feeds**.

## Change or remove a host

Each host has a card on the Hosts page. The card lists the settings of the host and the feeds that use it.
The line **Different addresses** shows when the address of a feed does not agree with the public address of the folder. Select the feed to open it.

- Select **Edit** to change the name and the public address. For an nsite site, you can also change the relays, the Blossom servers and the gateway. Then select **Save**.
- Select **Check the connection** on an SFTP card to test the login.
- Select **Check the relays** on an nsite card. Studio tells you if each relay has the newest manifest.
- Select **Remove** to remove the host. Studio asks you to confirm. The files in the host do not change.

You cannot remove a host while feeds use it.
Studio shows "Feeds use this host." Remove those feeds from your collection first.

## Other settings on the Hosts page

The Hosts page has three more panels:

- **Library**. It shows the library folder. You can set the largest download and the free space to keep, in MB. Select **Save the limits**.
- **Announcements**. See [Announcements](announcements.md).
- **Connected apps**. See [Connected apps and show grants](connected-apps.md).

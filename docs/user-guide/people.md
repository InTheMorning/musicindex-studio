# People

People keeps the persons that are in your feeds. A person can be an engineer, a singer or a writer.
Studio keeps a person in the data of your account. A person is not a feed.
Studio changes no feed when you make or change a person.

Select **People** at the bottom of the left panel to open the page.

## What a person holds

- The name, the usual role, the picture address and the information address.
- The wallets. A wallet is a payment address, with its node key.
- The Nostr keys, the credit names and the support links.
- The artist or label that the person is in.

The usual role is the start value of a new credit. The default address is the address that the pickers use.

## A suggestion

Studio reads your feeds and finds names that can be one person. Each name shows as a suggestion on the page People.

1. Select a suggestion. The sheet **Check: {name}** opens.
2. Select the person in the menu **Person**. Select **A new person** to make a person.
3. Remove the mark from the check box of each item that is not this person.
4. Select **Make the Person**, or **Add 1 Item** or **Add N Items**.

If you have a person with the same name, the menu starts on that person.
Studio then adds the selected items to it. Select **Not This Person** to hide the suggestion.

## The page of a person

The page shows the details first. The page keeps each change until you select **Save**. The toolbar shows how many changes are not saved.
A control with a change has a mark at its start edge.

- Select **Save** to keep all the changes together. Save changes no feed.
- Select **Discard** to put back the saved values.
- Studio asks before you open a different page with a change that is not saved. Select **Keep Editing** to stay on the page. Select **Discard Changes** to open the other page.

- **Suggestions for the picture** and **Suggestions for the information address** show below those two fields. Studio takes the values from the Nostr profile of a key of the person, from the credits of the feeds and from the artist or label feed. Each line names its source. Select **Use** to fill the field. Then select **Save**. When the field has a value, the other values are in a line that you open.
- **More from your feeds** lists items that can belong to this person. Select **Add** for an item. A row can warn that the name of a Nostr profile is not the name of this person. Select **Correct the Feeds** to open **Check your feeds** and replace that key. Select **Add Anyway** when the key is correct, for example for a band or a label.
- **Payment** shows the wallets. The default address has the tag **Default**. To change it, select **Make Default** on a different address, then select **Save**. Select **Add a Wallet** to add one. Select **Remove Wallet** below a wallet to remove it from the person.
- An LNURL address from the Nostr profile of a key of the person shows below the wallets. Select **Add to Wallet** to keep it, or **Dismiss** to hide it for this person. When the key is not the key of this person, remove the key in **Credits**.
- **Credits** has the fold **Credited as**. Open it to see each credit name with its feeds, its roles, its website and its Nostr key. Brackets mark the usual role. A Nostr key with no credit name has its own row. **Remove** on a row with a credit name and a key removes both from the person. It changes no feed.
- **Add a Nostr Key** opens the sheet **Find a Nostr profile**. Type a name, an `npub` key or a NIP-05 address in the field. Then select **Search**, and select a result to add the key.
- The row **Nostr key** of **Add a Credit Name** opens the same sheet.
- **Artists and labels** lists each artist or label that this person is part of. Each link stays in Studio. Use the menu **Add an artist or label** to add a link, and **Remove** to remove one. **Save** keeps the changes.
  - After **Save**, one sheet opens for each new link. It asks to add this person to the credits of that feed, with a role for that feed. Select **Review in 1 Feed** to make a change set. Studio writes the feed only after you review it.
  - Select **Not Now** to keep the link in Studio only. No feed changes.
  - When you remove a link, a sheet asks to remove the credit.
  - After the last sheet, one change set opens its review. Two or more show in **Changes to write** on the People page.
- **Merge with a Person** moves the items of a different person to this person.
- **Delete Person** deletes the person. No feed changes.

## Find on musicindex

Select **Find on musicindex** on the page of a person. Studio searches the artist and label feeds on musicindex.org for the name that you type. The search changes no feed. It adds nothing to your collection.

The feeds are in three levels:

- **Verified role**: the feed and its albums link to each other. They give the role.
- **Verified two-way links**: the feed and its albums link to each other, but they give no role.
- **Name only**: only the name matches. No album links back to this feed.

Select a feed to see its values. Select the check box of each value that you want. Then select **Fill** with the number of fields. Studio puts each value in its field, and the field shows the feed that gave the value. Select **Save** to keep the values. Select **Discard** to remove them.

- A picture and an information address start selected when the field is empty.
- A wallet and a Nostr key start not selected. Make sure that the artist of a wallet is this person.
- A value that the person has has no check box. A wallet of a different person has no check box.

## Check your feeds

Select **Check your feeds** to compare each feed with the person.
Studio uses the data that it has. It does not connect to a relay or to a wallet.

The page puts each feed in a group:

- **Needs a correction**. A credit can have no key, a different key, a different picture or no role.
- **Not your default address**. The feed pays the same wallet, but not with the default address. This is not an error.
- **Correct**. The feed agrees with the person.

Select a feed in **Needs a correction** or **Not your default address** to see its differences. The entry opens in the list.
Each difference shows the position, the type of value, the value of the feed and the value of the person. A missing value is **None**.

Select **Use {name}'s Values** to correct only that feed. The sheet **Correct the Feeds** opens with that feed.
Select **Open the Album** to open the feed in the editor. The button names the type of the feed: **Open the Playlist**, **Open the Artist Page** or **Open the Label Page**.

Select a feed in **Correct** to open it in the editor.

## Correct N Feeds

1. Select **Correct N Feeds**. The sheet **Correct the Feeds** opens.
2. Keep the check boxes that you want: **Nostr key**, **Payment address**, **Picture and information address** and **Role**.
3. Select **Replace other addresses named "{name}"** only when those addresses are yours too. Studio then pays your default address in their place. This box starts without a mark.
4. Remove the mark from the check box of each feed that you do not want to change.
5. Select **Review in N Feeds**. Then do the review and the write as usual.

A credit that has a role keeps its role. A feed with no change is not in the list.
The page [Review and write your changes](review-and-write.md) explains the review.

## Change your feeds

The section **Change your feeds** has four operations. Each operation shows the feeds and a review before Studio writes.

- **Use This Address** changes the payment address of the person in the feeds that you select.
- **Rename** changes the name in the payments, in the credits, or in the two.
- **Add to All Releases** adds the person to each release of an artist or label.
- **Remove from Feeds** removes the person from the payments or the credits.

## A payee with no person

A payee that no person holds shows in the section **Payees with no person**. Select it to open its page.

1. Select **Add to a Person**.
2. Select **A new person**, or select a person that you have. For a new person, check the name.
3. Select **Make the Person** or **Add the Wallet**.

You can also do this from the page of a person. Select **Add a Wallet**. The sheet lists the payees with no person first.

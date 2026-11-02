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

The page shows the details first. A field saves when you select a different field. The field then shows **Saved**.
The page has no **Save** button.

- **More from your feeds** lists items that can belong to this person. Select **Add** for an item.
- **Payment** shows the wallets and the default address. Select **Add a Wallet** to add one.
- **Credits** shows the credit names and the Nostr keys.
- **Artist or label** shows the menu **Member of**.
- **Merge with a Person** moves the items of a different person to this person.
- **Delete Person** deletes the person. No feed changes.

## Check your feeds

Select **Check your feeds** to compare each feed with the person.
Studio uses the data that it has. It does not connect to a relay or to a wallet.

The page puts each feed in a group:

- **Needs a correction**. A line in the error color shows each difference. A credit can have no key, a different key, a different picture or no role.
- **Not your default address**. The feed pays the same wallet, but not with the default address. This is not an error.
- **Correct**. The feed agrees with the person.

Select a feed to open it in the editor.

## Correct N Feeds

1. Select **Correct N Feeds**. The sheet **Correct the Feeds** opens.
2. Keep the check boxes that you want: **Nostr key**, **Payment address**, **Picture and information address** and **Role**.
3. Select **Payees with this name that pay a different wallet** only if you want to move those payments. This box starts without a mark.
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

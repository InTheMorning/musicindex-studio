# Nostr keys

A Nostr key links an artist, a label or a contributor to a Nostr profile.
Studio writes the key in the feed. Apps can then show the profile of the person.

Studio writes a key only when you select a result or type the key.

## Where a key goes

- A person in the credits has the row **Nostr**. The key belongs to that person.
- The first section of an album, an artist feed, a label feed or a playlist has the row **Nostr profile**. The key belongs to the feed.

## Find a key

1. Select the row **Nostr** or **Nostr profile**. The sheet **Find a Nostr profile** opens.
2. Type an `npub` key, a NIP-05 address or a name in the field. Then select **Search**.
3. Select a result. Studio sets the key and closes the sheet.

On a server where you signed in with a Nostr key, the button **Use my key** sets that key.

For a person with no name or no picture, the sheet shows two check boxes: **Use the profile name** and **Use the profile picture**.
Select a box to write that value from the profile into the person.

## The evidence on each result

Each result shows these facts:

- The picture, the name and the NIP-05 address. A check mark after the address means that the domain of the address confirms the key.
- The first and the last characters of the key.
- The sources. **In your feeds** means that a feed of yours has the key. **In 3 musicindex feeds** means that three feeds in musicindex have the key. **Nostr search** means that the search relay gave the result.
- **Confirmed at** and a domain. The domain of the NIP-05 address is also the domain of the website of the feed.

If a source does not answer, the sheet shows a hint. The results of the other sources stay.

## Change or remove a key

Select a row that has a key. The sheet shows the profile card of the key.
Select **Change** to search again. Select **Remove** to remove the key.
If Studio finds no profile, the sheet shows "Studio found no profile for this key".

If a key is not correct, the row shows "Not a Nostr key". Select the row and enter the key again.

## The search relay

Studio sends the text of a name search to a Nostr search relay.
To change the relay, go to **Settings** and open **Nostr**. Change the field **Nostr search relay**.
If the field is empty, Studio searches only your feeds and musicindex.

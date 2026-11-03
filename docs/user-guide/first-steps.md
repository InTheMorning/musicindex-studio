# Start your artist page or label page

Studio shows the page **Welcome to Studio** when your collection has no release.
Use this page to start your first artist page or label page.
Studio shows the same page on a computer and on a server.

## Choose how you start

Select a row on the Welcome page.

- **I make music** starts your artist page. Your albums go on it.
- **I run a label** starts your label page. Your artists and releases go on it.
- **My music is online already** brings in music that is online. See [Add feeds that you already have](add-feeds.md).

## Start an artist page

1. Type your name in **Artist name**. Select **Continue**.
2. Select the file host that Studio uses for your files. Step 2 always shows.
   - On a computer, the first row is **Folder on this computer**. It is selected at the start. Studio starts with the folder `~/musicindex/pages`. Select **Continue**. The public address of the folder is optional.
   - On a server, the first row is **Try it here (temporary)**. It is selected at the start. Studio keeps your files for 7 days. Select **Continue**.
   - For a different type of file host, select its row. Fill in its fields. Studio adds the file host.
3. Studio makes the draft of your artist page and opens it.

If you connected one or more file hosts, the menu **File host** shows above the rows.
Select a file host in the menu. Then select **Make the Page**.

## Start a label page

The steps are the same. Type the name of your label in **Label name**.
The page of your label page shows three rows: **Add an artist**, **Add a release** and **Find an artist that is online**.

## Add your first album

The page of your artist page shows **Add your first album**.
Select **Add an Album**. The sheet asks for the same fields as a new release. See [Make a new release](new-release.md).

## Review and write your first pages

Studio does not write to the file host when it makes a draft.
To write your new pages to the file host, follow these steps.

1. Select **Review** in the toolbar.
2. Read each change.
3. Select **Write to the File Host**.

The first write of an artist page and of its album goes through **Review**.
For more about this step, see [Review and write your changes](review-and-write.md).

## Placeholder GUID

A feed gets its GUID from its public address.
A folder with no public address has no public address. Studio cannot make the GUID at this time, so the feed gets a placeholder GUID.
The feed facts show the placeholder and its cause.

Studio also writes the feed to the file host.
To set the GUID from a public address, give the folder a public address in **File hosts**. Or move the feed to a file host with a public address.
Studio then sets the GUID, and it changes the links between your pages.

## Placeholder address

A file in a folder with no public address has no public address.
Studio gives the file a placeholder address. It starts with `pending:`.
The track page then shows the text "Studio cannot make the address of this file yet."
The player plays the file from Studio.

When the feed gets a public address, Studio sets the address of each file in the feed.
It does this in the same step as the GUID. You then write the changed draft in **Review**.

## The temporary area on a server

The temporary area is a folder in your user folder on the server.
It has no public address, so each feed in it gets a placeholder GUID.

Studio deletes the files of the temporary area 7 days after the last write.
Your feeds and their drafts stay in your collection.
After the deletion, each feed in the area shows this text: "The temporary area has expired. Move this feed to a real file host."

To keep your pages, follow these steps.

1. Add a file host with a public address in **File hosts**.
2. Open each feed of the expired area.
3. Move the feed to the new file host. Then write it through **Review**.

An account has one temporary area. To make a new area, remove the expired area in **File hosts** first.

## Move from a different server

Select **Import an Account** below the rows of the Welcome page.
Use this import only when you moved your account from a different server.

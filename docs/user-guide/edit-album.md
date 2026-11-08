# Edit the album details

Open a release from **Releases**. Select its title in the outline. The tab **Album** opens.
Studio saves each change in the draft as you type.
The hosted file does not change until you write. See [Review and write your changes](review-and-write.md).

The header of the page shows the public address of the feed. Select the address to open the feed in a new tab. Select the copy button, and Studio puts the address on the clipboard. When the address is too long for the header, put the pointer on it to see the full address.

## The album page

The page has four sections.

### Album

| Field | What to enter |
| --- | --- |
| Title | The name of the album. |
| Artist | The artist of the album. |
| Description | A text about the album. |
| Cover image address | The web address of the cover image. Studio shows a preview. Select **Choose…** to pick a file. |
| Website | The web address of your website. |
| Language | For example `en` or `de-AT`. |

An address must start with `https://` or `http://`.
If it does not, the field shows red text.
Studio does not write while an address is not correct.

### Choose a cover file

You can type or paste an address, or you can choose a file.

1. Select **Choose…** adjacent to **Cover image address**.
2. Select a source in the sheet:
   - **This Release**. The image files of this release.
   - **File Hosts**. The image files on your connected file hosts. You can only select a file that has a public address.
   - **This Computer**. You send one image file from your computer.
3. Select the image file.

You can also drop one image file on the field.
A file of the wrong kind, or more than one file, shows an error and changes nothing.
When you replace the cover, the review lists a deletion of the file from before.
Studio does not delete a file that another feed uses.
Select **Undo** in the review to keep the file.
See [Review and write your changes](review-and-write.md).

### Which cover image Studio shows

The field shows the `itunes:image` of the feed.
If the feed has no `itunes:image`, the field shows the RSS image of the feed.
When you change the cover, Studio keeps the form of the feed.
A feed that has only the RSS image keeps only the RSS image.
A feed that has only `itunes:image` keeps only `itunes:image`.
A feed that has both gets the new address in both.

Sometimes the two tags have different addresses.
Then the field shows the second address below it.
Select **Use This Address** to put that address in both tags.

If you do not select it, the next write puts the address of the field in both tags.
The review shows this change before the write.
Studio does not write two different cover addresses.
If you clear the field, Studio removes the cover from the feed.

### Hide from apps

Set **Hide from apps** to hide the album from apps and directories.
The feed stays at its address.
Studio writes `<itunes:block>Yes</itunes:block>` and `<podcast:block>yes</podcast:block>`.
Clear **Hide from apps** to show the album again.
Studio then writes `No` and `no` in these tags.
The release list shows **Hidden from apps** for a hidden feed.

Some feeds also have a block for one platform, such as `spotify`.
Then **Advanced** shows below the switch, with one check box for each platform.
The switch does not change these blocks.

### OP3 statistics

OP3 counts the downloads of each track.

The check box **Lock against import** is on the tab **Settings**. It asks other podcast platforms not to import this feed. If you move the feed to a platform that imports it, turn this off first. A platform can ignore this request, and an app or an index can still read the feed.

The **Lock owner email** is optional. A platform can use this address to confirm that you own the feed. The address is public in the feed.

The switch **OP3 Statistics** is on the tab **Settings**.
Set it to add the OP3 address to the audio address of each track.
The new start of the address is `https://op3.dev/e,pg=GUID/`, with the feed GUID.
The diff shows each address that changes.
Studio does not change an alternative file, an image or a live item.

If you remove the check mark of the switch, Studio removes the OP3 address from each track.
If you add a track while the switch is on, the track gets the OP3 address.
When the switch is on, the link **OP3 Statistics** opens the OP3 page of the album.

The switch is off when only some tracks have the OP3 address.
Then a hint shows the count, for example "2 of 9 tracks use OP3".
Studio then changes no address until you set the switch.

### Album details

The tabs **Credits** and **Value Routes** open the credits and the payment recipients. See [Value routes](value-routes.md).

This section has one row.

- **Feed facts**. Shows the address source, the feed GUID, the file, the version and the last announcement. It also has the button **Remove from Collection**.

### Linked artist and label pages

This section shows the artist feeds and the label feeds that link to this album.
See [Artist and label feeds](artist-and-label-feeds.md).

### Tracks

This section lists the tracks and has the row **Add tracks**.
See [Add, import and change tracks](tracks.md).

## Credits and links

Select the tab **Credits** to open the page.
It has three sections.

### People

1. Select **Add a credit**. The sheet **Add a Credit** opens.
2. Type the name in **Name**, or fill the form: select a row of **Your people**, or select **Find on Music Index** and then a feed. The sheet then lists the feed and each credit of the feed and its tracks. Select one.
3. Fill in the other fields if you want: **Role**, **Picture address** and **Information address**. Select the role from the menu. Studio writes the group of the role. If no role in the menu fits, select **Other role…**, write the role and select a group. If you leave **Role** empty, apps show the credit as host.
4. Select **Add the Credit**.

When Studio can read the image at **Picture address**, the open entry of the person shows the picture.

### Support links

1. Select **Add a support link**. The sheet **Add a Support Link** opens. You can select a row of **Your people**, or select **Find on Music Index** to use the website of a feed.
2. Type the link text in **Text**.
3. Type the web address in **Address**.
4. Select **Add the Support Link**.

### License

Type the license in **License**, for example `cc-by-4.0`.
Type the address of the license text in **License text address**.

### Change a list

Each person and each support link is one line. Select the line to open it.
Only one line is open at a time.
In an open entry, use **Move Up**, **Move Down** or **Remove**.

## The Feed facts row

Open **Feed facts** to see these facts:

- **Address source**. Where Studio found the public address of the feed.
- **Feed GUID**. The identity of the feed. Studio keeps it the same.
- **File**. The path of the feed file on the file host.
- **Version**. The version of the file that Studio read.
- **Announcement**. The state of the last announcement. See [Announcements](announcements.md).

If other sources give a different address, Studio shows a notice.
Studio uses the public address and does not change the feed file.

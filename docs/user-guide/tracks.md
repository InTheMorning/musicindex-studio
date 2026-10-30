# Add, import and change tracks

Open an album. The outline lists the tracks.
Studio sets the track numbers from the order of the tracks.
It does not change the publication dates.

## Add tracks

Use this method to add audio files from your computer.
Studio puts the files in the release folder. It sends them to the host when you write.

1. Select **Add tracks** in the outline or on the album page.
2. Select **＋ Choose music and image files**. You can select many files.
3. Studio reads each file in turn. Each row shows the file name and a state: "Waiting to send…", "Sending…", "Track ready" or "Image ready".
4. Check the facts of each track: **Title**, **Track number** and **Duration**.
5. For an image, use **Use for** to select **Album art**, a track, or **Not used**.
6. Select **＋ Add N tracks**. If you only chose album art, select **＋ Add the album art**.

Studio sends each file in parts of 8 MiB. The row shows the percent that Studio sent.
If a part fails, Studio tries it again 3 times. The row then shows "A part failed. Trying again…".
If the page closes during a send, open **Add tracks** again and select the same file.
Studio continues from the first part that it does not have.

Studio adds the tracks in the order of their track numbers.
Files with no track number come after them, in the order that you selected them.
The first image becomes the album art if the album has no image.

Studio accepts MP3, M4A, FLAC, OGG, Opus and WAV audio files.

- A WAV file: Studio makes an MP3 file and a FLAC file. Each app plays the MP3 file. An app that offers a choice can also play the lossless FLAC file.
- A FLAC file: Studio makes an MP3 file. The track keeps the FLAC file as an alternative.
- A GIF image: Apple Podcasts asks for JPEG or PNG art. Other apps show the GIF image.

If you reuse a file that is on the host, the row says "On the host already".
To remove a row, select **Remove**. To close the sheet, select **Cancel**. Studio then removes the files that it sent.

Studio can show these errors:

- "Studio does not read this kind of file."
- "A file with this name exists on the host already."
- "This host has no public address. Give it one in Hosts."

## The audio is already online

Use this method when the audio file is already on a web server.
Studio does not keep the file.

1. Select **Add tracks** in the outline or on the album page.
2. Select **The audio is already online**. The sheet **Add a track from an address** opens.
3. Type a **Title** if you want.
4. Type the web address of the audio file in **Audio file address**. Studio asks the host for the size and the type.
5. Examine the row **Properties**. Its summary shows the size and the type. If Studio could not find them, the row opens. Type **Audio file size in bytes** and **Audio file type**.
6. Select **＋ Add the track**.

To go back to the files on your computer, select **Choose files from this computer**.

## Edit a track

Select a track in the outline or on the album page.
The track page has these sections:

- **Track**. Fields: **Title**, **Description**, **Publication date** and **Duration**. Some apps sort the tracks by the publication date. Write the date like this: `Thu, 22 Oct 2026 12:00:00 GMT`.
- **Audio file**. The field **Audio file address** with **Choose…**, a player and the disclosure **Properties**. Alternative files show as lines with **Remove**.
- **Value routes**. See [Value routes](value-routes.md).
- **More**. **Track image** sets an image for this track. Use **Choose…** to pick the image file. **Identity** shows the track number and the track GUID. Studio gives a GUID when it saves a new track.
  - **Disc** is in **Identity**, with the mark **Experimental**. Studio writes the disc as `podcast:season`. The Podcasting 2.0 namespace defines that element for the seasons of a podcast, so apps can show it as a season.

The two arrow buttons in the header open the track before and the track after this one.

## Replace the audio of a track

1. Open the track page.
2. Select **Choose…** adjacent to **Audio file address**.
3. Select a source in the sheet: **This release**, **Hosts** or **This computer**.
4. Select the new audio file.

You can also drop one audio file on the field.
The track keeps its GUID and its other values.
A file from **This computer** goes in the release folder.
A file from **This release** or **Hosts** is not copied. Studio uses its public address.

Studio reads the duration from a file that you select from **This release** or **Hosts**.
If Studio cannot read it, the field **Duration** asks you to type it.

When you write, Studio deletes the file from before on the host.
The review lists that deletion.
Studio does not delete a file that another feed uses.
Select **Undo** in the review to keep the file.
See [Review and write your changes](review-and-write.md).

### The audio section

The audio section shows the address with **Choose…**, then a player, then the disclosure **Properties**.
The player plays the file when you press play.
**Properties** is closed. Its summary shows the size and the type.
It holds **Audio file size in bytes** and **Audio file type**.
**Properties** opens by itself when a value is missing or not correct.

## Change the order or remove a track

Each track has a position menu. Select a number, and the track moves to that position. The tracks between the two positions move one position. Studio then sets the track numbers again.

- On the album page, the menu is at the start of each row in **Tracks**.
- On the track page, the menu **Position** is at the bottom, adjacent to **Remove the track**.

**Remove the track** does not ask you to confirm.
It removes the track from the draft.
To get the track back, go back to an earlier version, or use **Discard the draft** to remove all changes that you did not write.

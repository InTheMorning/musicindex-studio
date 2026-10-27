# ADR 0062: A File Picker For Each File Field

- Status: Accepted
- Date: 2026-10-27
- Owner: operator

## Context

On 2026-10-27 the operator found that the editor gives no easy way to put an image on an album or a track. The import of [ADR 0026](0026-import-music-files-through-the-draft.md) makes one track for each audio file. An image of that import can become the album art, or the art of a new track of the same import. After the import, each image field is a text field for an address.

The editor has these file controls on 2026-10-27 (`crates/api/ui/editor.js`):

- The import sheet of the album: audio files and image files from the computer.
- The track page: the button "Replace the Audio File" of [ADR 0040](0040-replace-the-audio-of-a-track.md).
- The publisher page: the button "Import Image". It sends one image to the release folder and puts its address in the cover field.
- The album cover, the track image and the image of a live item: a text field only.

No field lets the user select a file that the release folder or a host already holds. The Files page lists the files of a release, but the user cannot put one of them into a field.

Each connector can list the files of its host, to a folder depth of 4 and a maximum of 1000 files ([ADR 0019](0019-select-a-feed-from-the-host.md) items 2 and 3). A file of a feed can be on any connected host ([ADR 0035](0035-a-release-library-is-the-master-of-each-feed.md)). Studio records each file of each host ([ADR 0036](0036-library-records-host-inventory-and-host-operations.md)).

## Decision

### The Picker

1. Each field that holds the address of an image or of an audio file has the button "Choose…" adjacent to the address ([ADR 0037](0037-button-roles.md)). The fields are:
   - the cover of an album, a publisher feed and a playlist
   - the image of a track and of a live item
   - the audio file of a track
2. The button opens one sheet with three sources, in this sequence:
   - **This release.** The files of the release folder and of its hosts that the field accepts.
   - **Hosts.** The file tree of each connected host, from the list function of ADR 0019.
   - **This computer.** An upload of one file.
3. The sheet shows only the files that the field accepts. An image field accepts the image formats of [ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md) item 1. An audio field accepts the audio formats of that item. Studio selects a host file by its extension.
4. Each image in the sheet shows a small copy of the image, through the API as [ADR 0025](0025-studio-serves-cover-art.md) requires. Each audio file in "This release" shows its duration when the library records it.
5. The address field stays. The user can still type or paste an address.

### This Release

6. "This release" lists the files of the release folder and their copies on hosts. It does not list the files of other releases.
7. A selection puts the public address of the file into the field. When the file has no copy on a host, the write puts it on the host first ([ADR 0026](0026-import-music-files-through-the-draft.md) item 7).

### Hosts

8. "Hosts" shows one tree for each connected host. A host with no folders shows one flat list. The tree shows the limit of ADR 0019 item 3 when the limit stops the list.
9. The user can select only a file that has a public address on its host. A file with no public address shows the reason and cannot be selected.
10. A selection puts the public address into the field. Studio records the file for the release with no bytes in the library ([ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md) item 1). Studio does not copy the file and does not upload it again.

### This Computer

11. An upload sends the file in parts to the release folder ([ADR 0059](0059-uploads-in-parts-and-the-home-screen-app.md)). The field gets the future address of the file on the host. The write puts the file on the host after the diff ([ADR 0026](0026-import-music-files-through-the-draft.md) items 5 to 7).
12. An audio upload to a track is the replacement of ADR 0040. The track keeps its GUID and its values, and a lossless file gives an MP3 file and a FLAC file ([ADR 0033](0033-a-lossless-import-gives-an-mp3-enclosure.md)). The button "Replace the Audio File" goes away in the same change, because "Choose…" replaces it.
13. A file that is not of the kind of the field goes away again, and the sheet shows an error. The button "Import Image" of the publisher page goes away in the same change.

### An Audio File From A Host

14. When the user selects an audio file from "This release" or "Hosts", the track changes as ADR 0040 item 4 says. The track keeps its GUID, its title and its track number. The selected file is the one enclosure, so the track gets no alternative enclosure with an audio media type.
15. When the library holds the bytes of the file, Studio reads its size, media type and duration, as for an import.
16. When only a host holds the file, Studio reads the size and the media type with the request of [ADR 0023](0023-web-addresses-and-audio-file-facts.md) item 5. Studio does not download the file. The player of item 26 then reads the metadata of the file through the API, with no press of the play button. Studio puts the duration of that metadata into the duration of the track. When the player cannot read the metadata, the duration field becomes empty and shows the notice "Type the duration of the new file".

### The File From Before

17. A selection or an upload replaces the address of the field. Then the file from before can have no address in the draft. Its copies on hosts then get a deletion in the draft, as ADR 0040 item 6 says. This applies to images and to audio files.
18. Studio adds no deletion for a file that another feed of the collection refers to ([ADR 0036](0036-library-records-host-inventory-and-host-operations.md) item 7).
19. "Undo" in the review sheet keeps the copy. The file from before stays in the release folder (ADR 0040 item 7).

### Drop A File On A Field

20. Each field of item 1 also accepts one file that the user drops on it. A drop is an upload of item 11. A drop of more than one file, or of a file of a different kind, shows an error and changes nothing.
21. The drop area of a field shows when the pointer brings a file over it. A screen with no pointer uses the button.

### The Cover Of A Channel

22. A selection or an upload in a cover field follows [ADR 0060](0060-the-album-cover-keeps-the-form-of-its-feed.md). A feed in the DeMu form keeps only `<image>`.

### The Audio Field Of A Track

23. The audio section of a track shows, in this sequence: the address field with "Choose…", a player, and the disclosure "Properties".
24. "Properties" holds the size in bytes and the media type. It is closed by default. Its summary gives the two values in short form, for example "3.8 MB · MP3".
25. "Properties" opens by itself when the size or the media type is empty, or when a value has an error ([ADR 0023](0023-web-addresses-and-audio-file-facts.md) item 4).
26. The player plays the file of the address field. It has a play and pause button, a position bar that the user can move, the position and the duration. It also plays a file of the release folder before the write.
27. The UI loads the audio through the local API, not from the host, as ADR 0025 item 2 requires for an image. The API reads the file from the release folder, or from its host. It sends each byte range that the player asks for, so that the user can move the position. The server shell applies the guard of [ADR 0057](0057-the-private-network-guard-of-the-server.md) to the host address.
28. The player reads no audio before the user presses the play button. The only exception is the metadata read of item 16.
29. The player gets the audio with a media ticket, in both shells. The UI asks the API for a ticket for one address, with its usual access ([ADR 0017](0017-local-api-access.md)). The audio route accepts the ticket in its query and needs no other access. A ticket is 16 random bytes, holds one address and one user, and expires after one hour. Studio keeps tickets in memory only.

## Alternatives Considered

- List the files of each release of the library in the picker. Rejected by the operator on 2026-10-27: one file then serves two releases from the release folder of one, and the deletion of item 17 can break the other release.
- A separate button for each source. Rejected: one sheet with three sources keeps one place for each field.
- Copy a host file into the release folder at each selection. Rejected: the file then has two copies with no need, and the selection takes the time of a download.
- For an audio file that only a host holds, download the file one time to read its tags. Rejected: item 10 says that a selection from a host copies nothing. The player reads only the part of the file that holds the metadata.
- Keep "Replace the Audio File" and "Import Image" adjacent to "Choose…". Rejected: two controls then do the same upload.
- Keep the file from before on its host with no deletion. Rejected by the operator on 2026-10-27: the host then fills with files that no feed uses. The review shows each deletion, and "Undo" keeps a file.

## Consequences

- Each image field and each audio field of the editor gets the same control.
- The size and the media type of an audio file go out of sight. The user can still change them in "Properties".
- A list of a host can take time, and a large host gives a partial tree.
- A host file can serve more than one feed. Item 18 protects it from a deletion that one feed starts.
- The phone shell has the button and no drop area.
- A media ticket is in the address of the audio. A person who gets the address can play that one file for one hour.

## Verification

Mechanical:

- An API test lists the files of "This release" for an image field and for an audio field. Each list holds only the files of the release, and only the kinds that the field accepts.
- An API test lists the tree of a test host. It holds only the accepted kinds, marks a file with no public address, and reports the limit when the limit stops the list.
- An API test selects a host file for a track image. The draft refers to its public address. The library records the file for the release with no bytes, and the diff uploads no file.
- An API test selects a host audio file for a track. The track keeps its GUID, its title and its track number, and gets the size, the media type and the duration of the file.
- An API test replaces a track image with an upload. The draft gets a deletion of the image from before. A second test gives the image from before to a second feed. The draft then gets no deletion.
- The Node tests refuse a file of a different kind for a field, and give the removal of that file (item 13).
- A UI guard confirms that `editor.js` has no button "Replace the Audio File" and no button "Import Image".
- An API test reads a byte range of an audio file of the release folder and of a test host. The response has the status 206 and the bytes of that range.
- A server shell test refuses the audio of an address in a private network.
- An API test reads audio with a ticket. A test refuses an expired ticket, a ticket of a different user, and a ticket for a different address.
- The Node tests examine the summary of "Properties", and when it opens by itself (items 24 and 25).
- The Node tests examine the duration of item 16: a duration in seconds from the player metadata gives the duration value of the track. No metadata gives an empty value and the notice.
- The Node tests examine the UI model of the sheet: the three sources, the kinds of each field, the rule of item 9 and the result of a drop of item 20.

Visual:

- The sheet of the album cover with the three sources, in the dark theme, in the light theme and at 390 px.
- A track with no image gets an image from "This computer", by the button and by a drop. The review lists the upload.
- A track gets an image from "Hosts". The review lists no upload.
- A track image is replaced. The review lists the deletion of the image from before, and "Undo" keeps it.
- The audio file of a track is replaced from "This release".
- The audio file of a track is replaced from "Hosts" with a file that only the host holds. The duration of the track changes to the duration of the new file with no press of the play button.
- The audio section of a track: the player plays, the position moves, and "Properties" is closed with its summary. With an empty media type, "Properties" is open. In the dark theme, in the light theme and at 390 px.

## Invariants

- No file goes to a host without a diff that the user saw ([ADR 0032](0032-new-releases-formats-paths-and-new-feeds.md)).
- A selection from a host does not copy or upload the file.
- Studio does not delete a file that another feed of the collection refers to.
- The UI loads no audio and no image from a server other than Studio.
- A media ticket gives access to one address only.

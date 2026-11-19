<!-- Recorded output of ChatGPT inventory run B at commit 73828c0. Kept unchanged as evidence. It is not edited to STE. See ../README.md. -->

# Current feed-editor journey inventory

Source: commit `73828c0`, as specified.

All file references below are relative to `crates/api/ui/`. No files were changed. No repository-changing Git commands ran.

This inventory describes frontend behavior. It includes boundary references where `app.js` opens the editor.

**Evidence limits**

- **INFERENCE — step counts:** Counts come from the event handlers. They are not browser measurements.
- One click, typed field, key, or list selection counts as one step.
- Automatic saves, requests, uploads, and waits count as zero steps.
- Native file or folder selection counts as one selection. **UNSURE:** Its internal steps depend on the operating system.
- Counts start on the open album page unless another start state is stated.
- Each optional field, checkbox, list selection, or closed disclosure adds one step.
- `${...}` identifies text supplied by a template or runtime value.
- **UNSURE:** API-owned messages cannot be enumerated from these frontend files. The UI commonly renders `${error.message}`.
- Shared errors and confirmations below apply wherever their referenced components occur.

## 1. Shared editor behavior

### Pages and navigation

`FeedDetail` selects the editable editor through `isEditable(feed)` (`app.js:2311`).

The editable types include:

- Album: medium `music`.
- Artist, label, and other publisher roles: medium `publisher`.
- Playlist: medium `musicL`, compared without case.

The type checks are in `shell.js:88–100`. A feed without `connector_id` can still open this editor. It needs a file host before writing (`shell.js:118`, `editor.js:2185`).

The tabs are defined in `shell.js:221` and rendered in `app.js:2192`.

| Feed type | Visible page choices |
|---|---|
| Album | `${feedKind(feed).label}`, `"Live Shows"`, `"Value Routes"`, `"Credits"`, `"Files"`, `"Check"` or `"Check ${count}"`, `"Settings"` |
| Publisher | `${feedKind(feed).label}`, `"Value Routes"`, `"Credits"`, `"Files"`, `"Check"` or `"Check ${count}"`, `"Settings"` |
| Playlist | `${feedKind(feed).label}`, `"Tracks"`, `"Value Routes"`, `"Files"`, `"Check"` or `"Check ${count}"`, `"Settings"` |

At narrow width, or when tabs do not fit, a select replaces the tab buttons. Its accessible label is `"Page of this feed"` (`app.js:2208`). Otherwise, each page is a segmented button (`app.js:2214`).

The narrow condition is `(max-width: 760px)` (`app.js:1844`).

Track pages do not show these tabs. A live-item page uses the Live Shows selection.

### Saving a draft

A field change updates the model and schedules a save after 700 ms (`draft.js:100`, `draft.js:268`).

The toolbar shows these states (`editor.js:2274`):

- `"Loading…"`
- `"Saving…"`
- `"Not saved"`
- `"Saved, not written"`
- `"No changes"`

The longer status appears in Review (`draft.js:104`):

- `"Loading the draft…"`
- `"Changes not saved yet…"`
- `"Saving the draft…"`
- `"The draft is saved. The hosted file does not change until you write it."`
- `"Studio could not save the draft."`

There is no separate Save button for ordinary editor fields.

### Shared failure display

`ErrorNotice` renders an alert containing a title and `${error.message}` (`app.js:441`).

The title map includes (`app.js:468`):

| Code | Exact title |
|---|---|
| `wrong_key` | `"The passphrase is not correct."` |
| `invalid_path` | `"That path is not in the folder."` |
| `not_found` | `"Studio did not find the file."` |
| `invalid_feed` | `"That file is not a feed that Studio can read."` |
| `offline` | `"No connection to Studio."` |
| `query_field` | `"This search cannot run."` |
| `index_answer` | `"Music Index gave a bad answer."` |
| `stream_not_live` | `"Go Live refused."` |
| Other codes | `"Something went wrong."` |

A network failure produces:

> `"Studio does not answer. Make sure that it is still running."`

Source: `app.js:169`.

These alerts do not add a universal retry button. Individual journeys sometimes provide their own action.

### Vault and session branches

The shared API wrapper can interrupt an editor request:

- A first `423` with `vault_locked` opens the unlock flow, then retries once (`app.js:138`).
- A recent-unlock `401` can also open that flow (`app.js:138`).
- Server session-required or session-ended responses return to sign-in (`app.js:138`).

These are shell boundaries. **UNSURE:** Complete unlock and sign-in journeys require files outside this editor scope.

## 2. Open a feed and reach an editor page

**Entry points**

| Entry | Visible label | Source |
|---|---|---|
| Release row or tile | Feed title, supplied by the row model | `app.js:3440`, `app.js:3475`, `app.js:4010` |
| Publisher navigation level | Publisher title | `app.js:3231` |
| Publisher outline release | Release title | `app.js:3312` |
| Related-draft notice | `"Open ${c.title || 'Untitled feed'}"` | `editor.js:3816` |
| File-host feed listing | Feed title | `app.js:2716`, `app.js:2721` |
| Restored navigation record | No new control | `app.js:3647`, `app.js:3677` |

**Main path**

1. Click the feed title. The wide layout opens the feed page.
2. On narrow width only, click `${feed.title}` in the outline head. The feed page opens (`app.js:3287`).

Opening loads the draft automatically (`draft.js:185`).

**Branches**

- `openFeedView(id, narrow, from)` opens `outline` when `narrow` is true. Otherwise, it opens `feed` (`shell.js:839`).
- An unsupported feed type uses `ReadOnlyFeed`, outside the detailed editor inventory (`app.js:2311`).
- A missing track key falls back to the album page (`editor.js:5029`, `editor.js:5047`).
- A missing live item redirects to Live Shows (`editor.js:4995`).
- While the draft has no model, the editor shows its status or error (`editor.js:5005`).

**End state:** The first feed page appears. The toolbar retains draft status.

**Count:** Wide: **1**. Narrow: **2**. Restored navigation: **0** user steps.

## 3. Edit album details and feed settings

### Album details

The album page contains Album, Tracks, linked pages, and Feed sections (`pages.js:17`, `editor.js:2839`).

For an ordinary field:

1. Type in the field, or change its switch.
2. Studio saves the draft automatically.

The second item is automatic and counts as zero steps.

| Exact label | Behavior | Source |
|---|---|---|
| `"Title"` | Changes channel title | `editor.js:2668` |
| `"Kind"` | Selects release kind | `editor.js:2707` |
| `"Artist"` | Changes `itunes:author` | `editor.js:2670`, `editor.js:2839` |
| `"Nostr profile"` | Opens profile selection | `editor.js:2660` |
| `"Artwork"` | Opens artwork sheet | `editor.js:2647` |
| `"Description"` | Changes description | `editor.js:2672` |
| `"Website"` | Changes the web address | `editor.js:2674` |
| `"Hide from apps"` | Sends a listing change through the core | `editor.js:2801` |
| `"Explicit"` | Changes the explicit tag value | `editor.js:2693` |

**Kind branch**

`feed.kind_shown` controls the Kind field (`editor.js:5049`). Its choices are:

- `"From the tracks"`
- `"Album"`
- `"EP"`
- `"Single"`

Source: `addfeeds.js:35`.

Kind uses `PUT /feeds/${feed.id}/kind`. It saves immediately through that route (`editor.js:2710`).

**Listing branch**

The text states:

> `"Apps and directories do not list a hidden feed. The feed stays at its address."`

When `platformBlocks.length > 0`, `"Advanced"` exposes `"Hide from ${row.id}"` controls (`editor.js:2801`, `editor.js:2829`).

**Explicit branch**

The hint is:

> `"Apple asks for this tag. Off means clean. Studio adds the tag only when you set the switch."`

Source: `editor.js:2693`.

### Feed settings

1. Select `"Settings"` (`shell.js:221`, `app.js:2192`). Feed settings appear.
2. Type or select a value below. The draft saves automatically.

| Section | Exact controls | Source |
|---|---|---|
| Language | `"Language"` | `editor.js:2743`, `editor.js:2745` |
| Keywords | `"Keywords"` | `editor.js:2754`, `editor.js:2756` |
| Owner | `"Owner name"`, `"Owner email"` | `editor.js:2748`, `editor.js:2750` |
| Lock against import | `"Lock against import"` | `editor.js:2760`, `editor.js:2763` |
| Lock owner | `"Lock owner email"` | `editor.js:2770` |
| OP3 statistics | `"OP3 statistics"` | `editor.js:2773`, `editor.js:2776` |
| Editors | `"Managing editor"`, `"Webmaster"` | `editor.js:2787`, `editor.js:2789` |

**Branches**

- `"Lock owner email"` appears when `model.locked` exists (`editor.js:2770`).
- OP3 appears only for albums (`pages.js:42`).
- With `model.op3 && model.podcast_guid`, `"OP3 Statistics"` opens an external page (`editor.js:2782`).
- That URL is `https://op3.dev/show/${encodeURIComponent(guid)}` (`op3.js:18`).
- Mixed OP3 use can show `"${prefixed} of ${tracks} tracks use OP3"` (`op3.js:13`).

**Errors:** Shared draft errors apply. Invalid address fields show:

> `"This is not a full web address. Start it with https://."`

Source: `editor.js:68`, `editor.js:73`.

**Confirmations:** None for these ordinary fields.

**End state:** The new values remain in the draft. No separate completion message appears.

**Counts**

- One album field: **1**.
- One settings field from album: **2**.
- Open listing Advanced, then change one platform switch: **2**.
- Open OP3 statistics from album: **2**, when the link already exists.

## 4. Link an album or playlist to artist and label pages

**Entry:** `"Linked artist and label pages"` on the main feed page (`editor.js:2556`).

**Main path**

1. Pick an entry under `"Link to an artist or label page"` (`editor.js:2575`).
2. Click `"Link to This Page"` (`editor.js:2584`).

Studio updates the link through `/feeds/${chosen}/links`. Related drafts reload (`editor.js:2551`).

**Other controls**

- `"Remove"` removes a link (`editor.js:2565`).
- `"Move Up"` changes its order (`editor.js:2568`).
- `"Move Down"` changes its order (`editor.js:2571`).

**Branches**

- Candidates must be connected publisher feeds that are not already linked (`editor.js:2531`).
- The selector is absent for `feed.host_deleted` (`editor.js:2574`).
- An unmatched imported link changes only this draft when removed.
- A matched collection link uses a DELETE route and can change both drafts (`editor.js:2548`).
- Empty text: `"Not linked to an artist or label page yet."` (`editor.js:2559`).
- No candidates: `"Studio has no other artist or label page to link to."` (`editor.js:2585`).

**Error**

A `role_required` error becomes:

> `"Open that feed and select its role first."`

Source: `editor.js:2586`.

**Result notice**

> `"Studio also changed the draft of ${c.title || 'Untitled feed'}. Open it to examine and write it."`

The notice offers `"Open ${c.title || 'Untitled feed'}"` (`editor.js:3811`).

**Confirmations:** None.

**End state:** The linked page appears. Related-draft notices can remain visible.

**Counts:** Add link: **2**. Remove or move one visible link: **1**.

## 5. Import tracks and images from the computer

**Entries**

- `"Add tracks"` below album tracks (`editor.js:2863`).
- `"Add tracks"` in the feed outline (`app.js:3310`).
- `"Choose Files from This Computer"` in the address-track sheet (`editor.js:668`).
- File drop into the import sheet (`editor.js:1359`).

**Main path**

1. Click `"Add tracks"`. The `"Add tracks"` sheet appears (`editor.js:1473`, `editor.js:1475`).
2. Click `"＋ Choose Music and Image Files"` (`editor.js:1488`).
3. Select files in the native picker.
4. Wait while Studio sends each file.
5. Click `${addLabel}` (`editor.js:1502`).

Step 4 counts as zero.

`${addLabel}` is (`editor.js:1427`):

- `"＋ Add ${readyAudio.length} Track${readyAudio.length === 1 ? '' : 's'}"`
- Or `"＋ Add the Album Art"`.

The sheet closes. Imported tracks appear on the album page (`editor.js:5070`).

**Drop path**

1. Open `"Add tracks"`.
2. Drop files on `"Drop music and image files here."` (`editor.js:1480`).
3. Click `${addLabel}` after processing.

**Branches**

- Files send sequentially (`editor.js:1322`).
- Audio sorts by track number, then selection order for unnumbered files (`editor.js:1434`).
- The first image defaults to album art only when the album lacks a cover (`editor.js:1346`).
- Each image has `"Use for"` with `"Album art"`, `${a.file.name}`, and `"Not used"` (`editor.js:1269`).
- `"Remove"` removes an import row. It is disabled while sending (`editor.js:1254`).
- The add action waits until processing ends and usable audio or album art exists (`editor.js:1426`).
- `"The Audio Is Already Online"` opens the address-track sheet (`editor.js:1492`).
- Cancel, Close, or Escape removes ready pending imports that were not added (`editor.js:1410`).

**Progress states** (`editor.js:1210`)

- `"Waiting to send…"`
- `"A part failed. Trying again…"`
- `"Sending… ${progressPercent(...)}%"`
- `"Track ready"`
- `"Image ready"`

**Conversion notices**

`editor.js:1156`:

> `"Studio converts this WAV file to MP3 and FLAC. Each app plays the MP3 file. An app that offers a choice can also play the lossless FLAC file."`

`editor.js:1157`:

> `"Studio makes an MP3 file from this FLAC file. Each app plays the MP3 file. An app that offers a choice can also play the lossless FLAC file."`

`editor.js:1179`:

> `"Apple Podcasts asks for JPEG or PNG art. Other apps show this GIF image."`

**Errors** (`editor.js:1198`)

| Condition | Exact text |
|---|---|
| `unsupported_media` | `"Studio does not read this kind of file."` |
| `file_pending` | `"A track or an image of this draft uses a file with this name. Remove it from the draft first."` |
| `file_exists` | `"A file with this name exists on the file host already."` |
| `no_public_address` | `"This file host has no public address. Give it one in File hosts."` |
| `invalid_name` | `"This is not a file name that Studio can use."` |
| Other failure | `${error.message}` or `"Studio could not import this file."` |

An unsupported drop can show:

> `"Studio cannot use ${names.join(', ')}. Drop music and image files."`

Source: `picker.js:153`.

**Confirmations:** None for adding the imported tracks.

**End state:** Album tracks and selected artwork are in the draft. Import progress disappears when the sheet closes.

**Counts:** Native picker: **4**. Drop: **3**. Image assignments add **1 each**.

## 6. Add a track whose audio is already online

**Entry:** `"The Audio Is Already Online"` in the import sheet (`editor.js:1492`).

**Main path**

1. Click `"Add tracks"` (`editor.js:2863`).
2. Click `"The Audio Is Already Online"`. `"Add a track from an address"` appears (`editor.js:650`).
3. Type `"Title"` (`editor.js:655`).
4. Type `"Audio file address"` (`editor.js:656`).
5. Press Tab to leave the address field.
6. Wait for automatic file-fact lookup.
7. Click `"＋ Add the Track"` (`editor.js:672`).

Step 6 counts as zero. The new track page opens (`editor.js:5051`).

Title is optional in the submitted model (`editor.js:621`).

**Lookup states** (`editor.js:642`)

- `"Studio asks the file host for the size and the type of the file…"`
- `"Studio found the size and the type on the file host."`
- `"The file host did not give all the facts. To type them, open Properties and select Advanced."`
- `"Studio could not read the facts from the file host. To type them, open Properties and select Advanced."`

**Branches**

- Lookup runs on address blur (`editor.js:659`).
- Missing facts can open `"Properties"` automatically (`editor.js:587`).
- Add requires a URL, numeric size, nonempty media type, and a non-invalid lookup state (`editor.js:616`).
- Manual facts require `"Advanced"`, confirmation `"Edit Values"`, then `"Audio file size in bytes"` and `"Audio file type"` (`editor.js:661`, `editor.js:662`).
- `"Choose Files from This Computer"` switches back to importing (`editor.js:668`).

**Errors:** Invalid addresses show the shared bad-address message (`editor.js:658`).

**Confirmations:** Manual fact editing uses confirmation C5 below.

**End state:** The new track page appears. The track belongs to the draft.

**Counts:** With title: **6**. Without title: **5**. Two missing facts add **4** when Properties opened automatically.

## 7. Edit, reorder, and remove tracks

**Entries**

- Album row `${item.title || 'Untitled track'}` (`editor.js:2853`).
- Outline track `${item.label}` (`app.js:3312`).
- Previous or next track controls (`editor.js:3403`).

### Edit track fields

1. Click the track title. The track page appears.
2. Type or select a field.

| Group | Exact controls | Source |
|---|---|---|
| Track | `"Title"`, `"Audio file address"`, `"Artwork"`, `"Description"` | `editor.js:3330`, `editor.js:419`, `editor.js:3334` |
| Credits | `"Use the album credits"`, `"Credits for this track"` | `editor.js:3343` |
| Lyrics | `"Add a lyrics file"` | `editor.js:3359` |
| More | `"Publication date"`, `"Explicit"` | `editor.js:3368` |
| Identity | `"Identity"` | `editor.js:3373` |
| Disc | `"Disc"` with `"Experimental"` badge | `editor.js:3381` |
| Disc fields | `"Disc number"`, `"Disc name"` | `editor.js:3386` |

Identity displays `"Track number"` and `"Track GUID"`. A missing GUID shows `"Studio gives one when it saves"` (`editor.js:3377`).

### Reorder

- On the album page, pick a numeric position from `"Position of ${name}"` (`editor.js:103`, `editor.js:2854`).
- On the track page, use `"Position"` (`editor.js:3413`).
- Position selection is disabled with fewer than two entries (`editor.js:103`).

### Remove

1. Open the track.
2. Click `"Remove the Track"` (`editor.js:3416`).

The model removes the track and opens the album page (`editor.js:3320`).

**Confirmations:** Track removal has no confirmation.

**End state:** Edited order or fields remain in the draft. Removal returns to the album. No separate result message appears.

**Counts**

- Edit one track field from album: **2**.
- Reorder on album: **1**.
- Reorder through track page: **2**.
- Remove from album: **2**.
- Open Identity and Disc, then edit one disc field from album: **4**.

## 8. Replace audio, inspect properties, and play audio

### Replace audio through a picker

**Entry:** `"Choose…"` beside `"Audio file address"` (`editor.js:1122`, `editor.js:3331`).

**Main path, release file**

1. Open a track.
2. Click `"Choose…"`. `"Choose an audio file"` appears (`editor.js:860`).
3. Select `${file.name}` in `"This Release"` (`editor.js:986`).

The picker closes. Studio replaces audio facts (`editor.js:886`).

The replacement retains track title, GUID, and order. It replaces audio alternatives and retains nonaudio alternatives (`editor.js:836`).

### Source branches

| Source | Atomic continuation after opening the picker | Source |
|---|---|---|
| `"This Release"` | Select `${file.name}` | `picker.js:61`, `editor.js:986` |
| `"File Hosts"` | Select source, pick `"File host"` if needed, open folders if needed, select `${file.name}` | `editor.js:1094`, `editor.js:1018` |
| `"This Computer"` | Select source, click `"Select a File"`, select native file | `editor.js:1105` |
| Drop | Drop one file on the supported field or picker region | `editor.js:886` |

Selecting an existing release or host file starts a metadata read (`audio.js:82`).

Typing a new address directly changes the enclosure address. It does not use the complete replacement behavior (`editor.js:401`).

### Properties

1. Click `"Properties"` (`editor.js:388`).
2. Read Size, Type, and Duration.
3. To edit them, click `"Advanced"` (`editor.js:395`).
4. Confirm `"Edit Values"`.
5. Type `"Audio file size in bytes"`, `"Audio file type"`, or `"Duration"` (`editor.js:423`, `app.js:531`).

`"Lock Values"` returns the values to the locked presentation (`editor.js:392`).

Properties can open automatically for missing values or metadata failures (`editor.js:416`).

Failure text can be:

> `"Studio could not read the ${names}. To type a value, select Advanced."`

Source: `audio.js:53`.

### Player

`"Player"` exposes controls with accessible labels:

- `"Play"`
- `"Pause"`
- `"Starting…"`
- `"Position"`

Sources: `editor.js:440`, `editor.js:554`.

The player can obtain a media ticket for held files (`editor.js:462`).

**Playback failures**

- `"This file uses http. Studio cannot play it here."` (`media.js:7`)
- `"Studio cannot play this file."` (`media.js:87`)
- `"Studio cannot play this file. Your browser can block audio from other sites."` (`media.js:115`)

### Other audio controls

Alternate enclosures have `"Remove"` (`editor.js:560`). No alternate-enclosure edit form is rendered there.

**Picker errors**

- `"Studio reads the list…"` (`editor.js:1042`)
- `"No file of this kind is here."` (`editor.js:1045`)
- `"The list stops at the limit of Studio. The file host has more files."` (`editor.js:1054`)
- `"No file host is connected."` (`editor.js:1092`)
- `"Select an audio file"` (`picker.js:49`)
- `"Drop one file only"` (`picker.js:49`)
- `"This file has no public address"` (`picker.js:49`)

**Confirmations:** Manual property editing uses C5. Audio replacement itself has no confirmation.

**End state:** Replacement audio remains in the draft. Playback does not change the draft.

**Counts**

- Replace from This Release, starting on album: **3**.
- Replace from This Computer: **5**.
- File Hosts: **4**, plus any host selection and folder openings.
- Direct address edit from album: **2**.
- Open Properties, enable manual editing, confirm, edit one field, starting on track: **4**.
- Play from an open Player: **1**.

## 9. Set cover artwork and additional images

**Entries**

- `"Artwork"` row (`editor.js:2647`).
- Feed header cover, accessible label `"Artwork of ${feed.title || 'Untitled feed'}"` (`app.js:2278`).
- Track header cover, accessible label `"Artwork of ${item.title || 'Untitled track'}"` (`editor.js:3399`).

**Main path**

1. Click `"Artwork"`. `"Artwork · ${title}"` appears (`editor.js:3179`).
2. Type `"Cover image address"` for a feed, or `"Track image address"` for a track (`editor.js:2632`, `editor.js:3188`).
3. Click `"Done"` (`editor.js:3205`).

Edits use the normal draft save. Done closes the sheet.

**Picker alternative**

The image field offers `"Choose…"`. It opens `"Choose an image"` with the same three source tabs (`editor.js:860`).

**Branches**

- A track without its own image shows `"Uses the album cover"` (`editor.js:3184`).
- A track with an image offers `"Use the Album Cover"` (`editor.js:3193`).
- A conflicting RSS image shows:

> `"The RSS image tag has a different address: ${conflict}. A write gives it the address of this field."`

It offers `"Use This Address"` (`editor.js:2635`).

### Additional artwork

Available for tracks and for feed types that are neither publisher nor playlist (`editor.js:3175`).

1. Open Artwork.
2. Click `"Add an image"` (`editor.js:3158`).
3. Type `"Image address"` (`editor.js:3128`).
4. Type or select optional fields.
5. Click `"Done"`.

Optional controls:

- `"Description"` (`editor.js:3132`)
- `"Purpose"` (`editor.js:3134`)
- `"Width"`, `"Height"`, `"Ratio"`, `"Type"` (`editor.js:3142`)
- `"Remove"`, `"Move Up"`, `"Move Down"` (`editor.js:113`)

Purpose choices come from `artwork.js:8`:

`artwork`, `social`, `canvas`, `banner`, `publisher`, `circular`, `poster`.

An unknown existing purpose remains available (`editor.js:3138`).

Image facts read automatically after 400 ms (`editor.js:3117`).

**Error**

> `"Studio cannot read this image. Type the dimensions."`

Source: `editor.js:3130`.

**Confirmations:** None.

**End state:** The artwork preview and draft update. Done closes the sheet without a completion message.

**Counts:** Set address: **3**. Add one additional image with address only: **4**. Each extra field adds **1**.

## 10. Add and edit credits, roles, support links, and license

**Entry:** `"Credits"` tab (`shell.js:221`).

The page has `"People"`, `"Support links"`, and `"License"` sections (`editor.js:3213`).

### Add a credit

1. Select `"Credits"`.
2. Click `"Add a credit"` (`editor.js:330`). `"Add a Credit"` appears (`editor.js:207`).
3. Type `"Name"` (`editor.js:212`).
4. Click `"Add the Credit"` (`editor.js:222`).

The sheet closes. A credit row appears.

Optional fields:

- `"Role"` (`editor.js:213`)
- `"Picture address"` (`editor.js:215`)
- `"Information address"` (`editor.js:216`)
- `"Nostr"` (`editor.js:217`)

`"Your people"` rows fill the form. They do not add the credit automatically (`editor.js:186`).

`"Find on Music Index"` opens the shared search sheet (`editor.js:220`).

The Add action requires a trimmed name (`editor.js:202`).

### Role selection

The role control offers (`app.js:1349`, `roles.js:6`):

- `"Select a role"`
- Role and group entries supplied by the API
- `"Other role…"`
- `"Other role"`
- `"Group"`
- `"No group"`

The hint is:

> `"Apps show a credit with no role as Host."`

Source: `app.js:1349`.

### Edit an existing credit

1. Open the credit summary row `${name}` (`editor.js:310`).
2. Change a field (`editor.js:315`).

`"Remove"`, `"Move Up"`, and `"Move Down"` apply to the list (`editor.js:326`).

An empty credit name produces:

> `"A credit needs a name."`

Source: `credits.js:26`.

### Fill from Music Index

`"Find on Music Index"` can fill empty fields. Conflicting values appear under `"Values from Music Index"` with `"Replace"` (`editor.js:231`, `editor.js:239`).

The shared search path is documented in section 11.

### Track credits

On a track, select `"Credits for this track"` (`editor.js:3345`).

Studio copies album credits. If none exist, it starts an empty credit entry (`editor.js:3305`).

Selecting `"Use the album credits"` invokes C7. Cancel restores the selector to track credits (`editor.js:3309`).

### Support links

1. Select `"Credits"`.
2. Click `"Add a support link"` (`editor.js:354`).
3. Type `"Address"` in `"Add a Support Link"` (`editor.js:277`, `editor.js:283`).
4. Click `"Add the Support Link"` (`editor.js:288`).

Optional `"Text"` is at `editor.js:282`.

`"Find on Music Index"` can fill the website (`editor.js:286`). An empty result shows `"This feed has no website."` (`editor.js:289`).

Existing entries expose `"Text"`, `"Address"`, and common list actions (`editor.js:348`).

### License

The fields are `"License"` and `"License text address"` (`editor.js:369`).

**Confirmations:** Returning track credits to album credits uses C7. Ordinary credit and support removal has no confirmation.

**End state:** The list or field remains in the draft. Add sheets close. No success message is added.

**Counts**

- Minimal credit from album: **4**.
- Add a role: **+1**.
- Minimal support link from album: **4**.
- Support text: **+1**.
- Edit one existing credit: **3** from album.
- Give an open track its own credits: **1**.
- Return an open track to album credits: **2**.
- Edit one license field from album: **2**.

## 11. Select a Nostr profile or a Music Index value

### Nostr profile

**Entries:** `"Nostr profile"` on a feed, or `"Nostr"` in a credit (`editor.js:2660`, `editor.js:217`).

**Main path, empty value**

1. Open the Nostr row. `"Find a Nostr profile"` appears (`editor.js:778`).
2. Type `"Name, npub or NIP-05 address"` (`editor.js:794`).
3. Click `"Search"` (`editor.js:796`).
4. Select a profile result (`editor.js:817`).

The value updates and the sheet closes (`editor.js:764`).

**Branches and controls**

- Search changes to `"Searching…"` while active (`editor.js:796`).
- Existing value: `"Remove"` and `"Change"` (`editor.js:788`).
- `"Change"` exposes the search form.
- A session key permits `"Use My Key"` (`editor.js:798`).
- Person-value selection can offer `"Use the profile name"` and `"Use the profile picture"` (`editor.js:802`).
- `"Find on Music Index"` is another source (`editor.js:808`).
- `"Your people"` provides known keys when `offerPeople` permits it (`editor.js:811`).
- `"Cancel"`, `"Close"`, or Escape closes the sheet (`editor.js:779`, `editor.js:823`).
- Existing keys have a copy control (`editor.js:700`).

**Errors and empty results**

- `"Not a Nostr key"` (`editor.js:697`)
- `"Studio could not read your feeds."` (`editor.js:703`)
- `"Music Index did not answer."` (`editor.js:703`)
- `"The search relay did not answer."` (`editor.js:703`)
- `"A source did not answer."` (`editor.js:814`)
- `"Studio found no profile."` (`editor.js:815`)
- `"Studio found no profile for this key"` (`editor.js:786`)

**Count:** Empty-value search: **4**. Existing-value search: **5**. Use My Key: **2**.

### Shared Music Index search

**Entries**

- Credit `"Find on Music Index"` (`editor.js:220`)
- Credit value fill (`editor.js:231`)
- Support-link `"Find on Music Index"` (`editor.js:286`)
- Nostr `"Find on Music Index"` (`editor.js:808`)
- Recipient address `"Find on Music Index"` (`values.js:276`)

**Main path**

1. Click `"Find on Music Index"`. The same-named sheet appears (`pickers.js:313`).
2. Type `"Name"` (`pickers.js:315`).
3. Wait for lookup.
4. Select a result.
5. If multiple values exist, select `${option.title}` / `${option.line}` (`pickers.js:321`).

Lookup is automatic. A single usable value skips step 5 and fills the parent form (`pickers.js:305`).

`"Back"` returns from value choices to results. `"Cancel"` closes (`pickers.js:324`).

**Search-field assistance**

Clicking or typing can open the field list. It starts without a selected row (`pickers.js:46`).

- Arrow keys select a field.
- Enter or Tab inserts the selected field.
- Enter without selection retains the field action.
- Escape closes the field list before closing the sheet.

Sources: `pickers.js:46`, `fieldlist.js:62`.

**End state:** The parent form receives values. Adding or saving that form remains a separate action.

**Counts:** One available value: **3** after reaching the parent form. Multiple values: **4**. Closed source folds add steps.

## 12. Add recipients, select saved recipients, and set splits or fees

**Entry:** `"Value Routes"` (`shell.js:221`).

### Add a recipient manually

1. Select `"Value Routes"`.
2. If no block exists, click `"Add recipients"` (`values.js:530`).
3. Click `"Add a recipient"` (`values.js:209`). `"Add a Recipient"` appears (`values.js:258`).
4. Type `"Address"` (`values.js:264`).
5. Click `"Add the Recipient"` (`values.js:279`).
6. Type `"Split (shares)"` in the new entry (`values.js:191`).

Step 2 is conditional.

Optional form controls:

- `"Type"` (`values.js:263`)
- `"Name"` (`values.js:268`)
- `"Custom key"` (`values.js:270`)
- `"Account"` (`values.js:271`)

Type choices are exactly `"Node Address"` and `"LNURL Address"` (`recipients.js:453`).

Custom key and Account appear when `fields.recipient_type !== 'lnaddress'` (`values.js:269`).

The Add button checks nonempty address and duplicate identity (`values.js:247`, `values.js:279`).

### Select a saved recipient

After opening Add a Recipient:

1. Open `"Your people"` or `"Payees with no person"` if closed (`values.js:278`).
2. Select `${r.name}` (`values.js:251`).
3. Click `"Add the Recipient"`.
4. Type `"Split (shares)"`.

Selection fills the form. It does not add the recipient (`values.js:255`).

### Change an address

1. Open the address action menu, accessible label `"Actions for ${recipient.address || recipient.name || 'recipient'}"` (`values.js:104`).
2. Select `"Change…"` from the menu.
3. Change form fields in `"Change the Address"` (`values.js:258`).
4. Click `"Use This Address"` (`values.js:279`).

The menu labels come from `people.js:21`, used by the editor helper.

### Split and fee controls

An open recipient exposes:

- `"Split (shares)"` (`values.js:191`)
- `"Fee"` (`values.js:194`)
- `"Remove"` (`values.js:197`)

Recipients and Fees appear in separate groups (`values.js:206`, `values.js:210`).

The UI sorts by split. Reordering waits until focus leaves the entry, with a 200 ms delay (`values.js:158`).

There is no recipient position control.

### Save an address to People

The address menu can offer `"Add to People"` when an address exists and is not already in People (`values.js:104`).

The picker also exposes a save control:

- `"Add to People"`
- `"Adding…"`
- `"Added"`

`"Added"` lasts two seconds (`values.js:288`, `values.js:312`).

This calls `/recipients/saved`. It does not open a People page (`values.js:299`).

The address menu also offers `"Copy"`.

### Branches and empty states

- No block: `"This feed has no recipients."` (`values.js:520`)
- Empty recipients: `"No recipients."` (`values.js:208`)
- Empty fees: `"No fees."` (`values.js:212`)
- Duplicate: `"This address is already in this list."` (`values.js:272`)
- Loading saved choices: `"Loading your people…"` (`values.js:274`)
- Existing values can appear disabled with `"Already in this list"` (`pickers.js:243`).
- Only the first payment block is edited. Other blocks and time splits remain preserved (`values.js:506`).

**Errors:** Saved-choice load errors appear inside the form through ErrorNotice (`values.js:273`). Draft-save errors apply after adding.

**Confirmations:** None for adding, changing, fee toggling, or recipient removal.

**End state:** Recipient rows and the share display update in the draft. Add closes the sheet.

**Counts**

- Manual address plus split, existing block: **5**.
- No block: **6**.
- Saved recipient plus split, fold already open: **5**.
- Closed saved-recipient fold: **+1**.
- Type, name, custom key, or account: **+1 each**.
- Change one address from a visible open recipient: **4**.
- Toggle Fee or remove a visible open recipient: **1**.

## 13. Copy recipients from Music Index and set track-specific routes

### Copy from a feed

**Entry:** `"Copy from a Feed"` in Add a Recipient (`values.js:277`).

**Main path**

1. Select `"Value Routes"`.
2. Click `"Add a recipient"`.
3. Click `"Copy from a Feed"`.
4. Type `"Name"` in `"Copy from a Feed on Music Index"` (`values.js:444`, `values.js:449`).
5. Select a feed result (`values.js:453`).
6. Select each recipient checkbox (`values.js:338`).
7. Click `"Add ${n} ${n === 1 ? 'Recipient' : 'Recipients'}"`.

The add-label helper is in `people.js:1955`, used at `values.js:489`.

Copied recipients start with an empty split. Their fee value is retained (`values.js:438`).

The sheet explains:

> `"A percent is the share in this source. A new recipient gets no split, and you give it in the editor."`

Source: `values.js:358`.

### Source and selection branches

- `"Tracks"` opens a fold of source tracks (`values.js:458`).
- Selecting `${t.title}` loads that track’s recipients.
- `${feedName}` returns to the feed’s recipients (`values.js:468`).
- `"Search results"` returns to results (`values.js:456`).
- `"${total} Selected"` opens or closes the selected panel (`values.js:485`).
- Duplicate recipients have a taken tag and no checkbox.
- Empty feed: `"This feed has no recipients."` (`values.js:463`).
- Empty track: `"This track has no recipients."` (`values.js:471`).
- `"Cancel"` closes without adding (`values.js:487`).

### Give a track or show its own recipients

**Entry:** Track or live-item Value routes section (`editor.js:3364`, `editor.js:3750`).

Inherited state:

> `"This ${noun} uses the album recipients."`

Action:

> `"Give This ${noun.charAt(0).toUpperCase() + noun.slice(1)} Its Own Recipients"`

Source: `values.js:553`.

Clicking copies the first album block without its time splits. With no album block, it creates a blank block.

When own recipients exist, `"Use the Album Recipients"` invokes C11 (`values.js:555`).

The editor does not provide time-split editing controls (`values.js:1`, `values.js:506`).

**End state:** Added recipients remain in the draft. Copy selection closes. Each copied recipient still needs a split.

**Counts**

- Copy `n` recipients, starting on album with an existing block: **6 + n**.
- Open every copied row and set its split: **6 + 3n**.
- Select recipients from a source track: **+2** for Tracks fold and track.
- No destination payment block: **+1**.
- Give an open track or show its own routes: **1**.
- Return to album routes: **2**, including confirmation.

## 14. Add, reorder, and remove playlist items

**Entry:** `"Tracks"` on a playlist (`editor.js:2894`).

### Add from Music Index

1. Select `"Tracks"`.
2. Click `"Add a Track"` (`editor.js:2917`). `"Add tracks"` appears (`editor.js:2988`).
3. Type `"Search Music Index"` (`editor.js:2999`).
4. Click `"Search"` (`editor.js:3004`).
5. Select `${h.title || h.feed_guid}` (`editor.js:3018`).
6. Click `"Add"` for each wanted track (`editor.js:2932`).
7. Click `"Done"` (`editor.js:3041`).

Each Add stays in the sheet. Its accessible label is `"Add ${pick.title || 'this track'}"`.

### Add from the collection

1. Select `"Tracks"`.
2. Click `"Add a Track"`.
3. Select `"My Releases"` (`editor.js:2994`).
4. Click `"Add"` for each wanted track.
5. Click `"Done"`.

The default source is `"Search Music Index"` (`editor.js:2993`).

### Other paths

- Position selection reorders an existing playlist row (`editor.js:2906`).
- `"Remove"` deletes a playlist row without confirmation (`editor.js:2913`).
- Already-added tracks show `"In the playlist"` (`editor.js:2930`).
- Close or Escape exits the picker.

**Errors and empty states**

- `"Music Index found no feed with that name."` (`editor.js:3013`)
- `"This feed has no tracks."` (`editor.js:3022`)
- `"Your collection has no release with tracks."` (`editor.js:3032`)
- Search and load errors appear inside the sheet (`editor.js:3010`, `editor.js:3030`).

**End state:** Remote items appear in the playlist draft. Done closes the picker.

**Counts**

- Music Index, `n` tracks: **6 + n**.
- My Releases, `n` tracks: **4 + n**.
- Reorder or remove from album start: **2**, including Tracks.
- No album-style audio track page is opened by this picker (`editor.js:2894`, `playlist.js`).

## 15. Check a feed and apply a repair

**Entries**

- `"Check"` or `"Check ${count}"` tab (`feedcheck.js:84`).
- Repair navigation from Review can open linked pages instead (`editor.js:2171`).

**Main path**

1. Select Check. Studio reads addresses automatically (`editor.js:4864`).
2. Select a repair button beside a finding.

The page states:

> `"Studio does not change the feed until you select a button. Each change goes to the draft, and you review it before the write."`

Source: `feedcheck.js:8`.

Possible exact repair labels (`feedcheck.js:50`):

- `"Add the Link"`
- `"Use the Public Address"`
- `"Use the https Address"`
- `"Read the Size"`
- `"Open the Track"`
- `"Select a Language"`
- `"Link a Page"`

The first four call repair routes. The others open the relevant editor location (`feedcheck.js:62`, `editor.js:4894`).

**Branches**

- `check.read_only`: findings remain visible, but repair controls are unavailable.
- Text: `"This feed has no file host, so Studio shows the findings and does not repair them."` (`feedcheck.js:15`).
- Active show grants add the warning from `feedcheck.js:18`.
- Findings sort Warning, Notice, then Question (`feedcheck.js:108`).
- Finding titles and reasons are `${row.title}` and `${row.reason}`, supplied by the core.
- No findings: `"Studio found no problem in this feed."` (`feedcheck.js:11`).
- Initial state: `"Studio is reading the addresses of this feed."` (`feedcheck.js:24`).
- Unread addresses appear under `"Not read"` with `"Studio could not read this address."` (`editor.js:4946`, `feedcheck.js:21`).

`"Check Again"` changes to `"Checking…"` during a new read (`editor.js:4905`).

A stale repair returns:

> `"The feed changed. Studio checked it again. Select the button again if the finding is still here."`

Source: `app.js:2145`.

### Copy-payment report

A `copy_other_recipients` finding offers `"Show What to Do"` (`feedcheck.js:117`).

It opens `"A Copy Pays Other People"` (`feedcheck.js:30`, `editor.js:4929`).

The sheet says:

> `"Music Index has a copy of this feed at a different address. The copy has the same GUID, and its payments go to other people."`

> `"Only the operator of Music Index can remove the copy. Studio does not send this report. Copy it and send it to the operator."`

`"Copy the Report"` changes to `"Copied"` (`editor.js:4934`). Close or Escape returns to Check.

### Different indexed address

An `index_address_differs` finding offers `"Show the Ways"` (`feedcheck.js:122`).

It opens `"Music Index Has Another Address"` (`editor.js:4938`).

It presents:

- `"A redirect at the old host"`
- `"A request to the operator of Music Index"`

Source: `feedcheck.js:38`.

It does not send either request.

**Confirmations:** None.

**End state:** A repair updates the draft and findings. Navigation repairs open their target page. Reports stay open until closed.

**Counts**

- View check: **1**.
- Apply one direct repair: **2**.
- Open either explanatory sheet: **2**.
- Copy the report: **3**.
- Repeat check from its page: **1**.

## 16. Review changes and write to the file host

**Entries**

- `"Review"` in the draft toolbar (`editor.js:2301`).
- Live status changes open Review automatically (`editor.js:3630`).
- Feed deletion, restoration, and old-file deletion can open Review automatically (`editor.js:2453`, `editor.js:2466`, `editor.js:4701`).
- `"Show the Difference"` in an outside-change notice requests merge Review (`app.js:4911`).

**Main path**

1. Click `"Review"`. `"Review changes"` appears (`editor.js:2135`).
2. Wait while Studio flushes the draft and loads the diff and link check (`editor.js:1971`).
3. Click `"Write to the File Host"` (`editor.js:2228`).

Step 2 counts as zero.

The write signs required events and posts `diff_id` with `signed_events` (`editor.js:2034`, `editor.js:2040`).

On success, Studio reloads the draft and feed list. Review closes (`editor.js:2059`).

### Diff controls

- `"Simplified"` and `"Detailed"` (`review.js:13`, `editor.js:1612`)
- `"Show the Whole File"` and `"Show Only the Changes"` in Detailed (`editor.js:1617`)
- `${moreLabel(s.count)}` reveals omitted context (`editor.js:1625`)
- `"Undo"` removes a pending host operation (`editor.js:1696`)
- `"Undo"` cancels a pending feed move (`editor.js:1720`)
- `"Examine the Changes"` appears when no diff exists or the diff is stale, outside merge mode (`review.js:389`, `editor.js:2219`)

If no work exists:

> `"The draft is the same as the hosted file."`

Source: `editor.js:2209`.

### No writable file host

The Review host-choice branch appears when the host model requires it (`shell.js:135`, `editor.js:2185`).

It says:

> `"Studio cannot write to the file host of this feed. Connect that file host, or move the feed to a file host that you connected."`

Controls:

1. `"Connect the File Host of This File"` opens File hosts (`editor.js:2188`).
2. Or pick `"Move to a file host"` (`editor.js:2190`).
3. Click `"Move to This File Host"` (`editor.js:2198`).
4. Continue with the refreshed Review.

Without targets:

> `"No file host is connected yet. Connect a file host in File hosts first."`

Source: `editor.js:2192`.

### Write blockers and notices

| Condition | Presentation | Source |
|---|---|---|
| `draft.bad.length > 0` | Address refusal, no write button | `editor.js:2118`, `editor.js:2165` |
| Pending downloads | `"Wait for each download, then write."` | `editor.js:2210` |
| Missing signer | `"Connect a Signer"` | `editor.js:2215` |
| Signer key differs | `"Connect a Different Signer"` | `editor.js:2215` |
| Stream refusal | `${problem.title}` and `${problem.text}` | `editor.js:2174` |
| Repairable stream type | `"Use ${problem.server_type_label}"` | `review.js:28` |
| Missing artist page | `"This release has no artist page."` | `feedcheck.js:73` |
| Missing publisher page | `"This feed has no artist, label or hosting page."` | `feedcheck.js:74` |
| Missing linked page | `"Link a Page"` opens linked pages | `editor.js:2171` |
| Hidden draft | `"This feed will be hidden from apps."` | `editor.js:2203` |
| Busy write | `"Writing…"` or `${signProgressLabel(progress)}` | `editor.js:2228` |

Write is disabled while busy, waiting for downloads, needing a signer, facing a stream refusal, or facing a link refusal (`editor.js:2227`).

Signer connection is embedded through `SignerConnect` (`editor.js:2217`). **UNSURE:** Its full connection journey is outside the listed files.

### Announcement and show-grant choices

`"Send a podping"` appears as a Review choice when applicable (`editor.js:1855`, `announce.js:20`).

Its preference saves through `/settings/announcements` (`editor.js:1833`).

Pending shows can expose (`editor.js:1874`, `apps.js:91`):

- `"Connected app for this show${suffix}"`
- `"Let ${app.name} start and end this show, and set its live value${suffix}"`

For multiple shows, suffix is `": ${showTitle || 'Untitled show'}"`.

Without apps:

> `"No app is connected. Connect an app in Settings."`

### Failures

Shared errors remain inside Review. Specific branches include:

- Conflict: section 19.
- Bad address: section 20.
- Bad move link: section 23.
- `plan_expired`: Studio refreshes the diff automatically (`editor.js:2046`).
- Stale diff:

> `"The draft changed after you examined it."`  
> `"Examine the changes again."`

Source: `editor.js:2159`.

- Stale revision:

> `"The draft changed in a different window."`  
> `"Load the draft again to continue. The changes in this window since the last save go out."`

Action: `"Load the Draft Again"` (`editor.js:2142`).

**Confirmations:** The normal write button has no additional confirmation sheet.

**End state:** Review closes after a successful write. The feed page and draft status update. Index and podping notices can remain.

**Counts**

- Review only: **1**.
- Review and normal write: **2**.
- Choose another connected host before write: **4**, including the host pick.
- Refresh a stale diff: **+1**.
- Each optional announcement or grant choice: **+1**.
- Signer-provider actions: **UNSURE**, outside this scope.

## 17. Examine the write result and index state

The write does not leave a general success dialog. Outside notices present later status (`app.js:4917`).

Possible texts (`outsidecheck.js:136`):

- `"You wrote this feed at ${time}. Music Index reads it after it gets the podping."`
- `"Music Index has your write of ${time}."`
- `"Music Index has not read your write of ${time} yet."`

The confirmed message depends on a previously pending write.

`"Check Again"` appears for the no-reply state (`outsidecheck.js:154`).

Podping rows can show `"Waiting"`, `"Sent"`, `"Failed"`, or `"Not sent"`, with runtime time, address, and error details (`outsidecheck.js:94`).

Actions can be:

- `"Send the Podping"`
- `"Send the Podping Again"`

A move can show:

> `"You moved this feed at ${localTime(check.last_move.moved_at)}. Music Index reads the new address after it gets the podping."`

Source: `outsidecheck.js:122`.

A differing index address can show:

> `"Music Index reads this feed from a different address."`

> `"A different collection or a different tool can hold this feed."`

Source: `outsidecheck.js:80`.

**Index-link finding:** The outside-result renderer displays these addresses as text. It does not render a Music Index result hyperlink (`app.js:4917`).

The editor separately offers:

- Public feed URL `${url}`, opening externally (`app.js:1950`).
- A copy control beside that URL (`app.js:1951`).
- `"OP3 Statistics"` when applicable (`editor.js:2782`).

**End state:** Notices remain while their corresponding state remains. There is no separate dismiss control in the result renderer.

**Counts:** Observe result: **0** after write. Check Again or send podping: **1**.

## 18. Discard a draft or restore an earlier version

### Discard

**Entry:** `"Discard the Draft"` in Review (`editor.js:2226`).

1. Click `"Review"`.
2. Click `"Discard the Draft"`.
3. Confirm `"Discard the Draft"` in C4.

Studio deletes the draft through the API, reloads it, and closes Review (`editor.js:2067`).

The button is disabled when no draft change exists, except the deletion-specific branch (`editor.js:2226`).

**End state:** The editor shows the reloaded draft. No result message stays.

**Count:** **3**.

### Restore a version

**Entry:** `"Versions"` in the toolbar (`editor.js:2300`).

1. Click `"Versions"`. `"Earlier versions"` appears (`editor.js:2255`).
2. Click `"Put in the Draft"` beside a version (`editor.js:2265`).
3. If the current draft has changes, confirm `"Replace the Draft"` in C6.

Studio flushes, restores through `/snapshots/${snapshot.id}/draft`, and closes the sheet (`editor.js:2245`).

The sheet explains:

> `"Before each write, Studio keeps the file that the write replaces. To go back, put an earlier version in the draft, examine the changes and write them."`

Source: `editor.js:2259`.

**Branches**

- Empty list: `"No earlier versions yet."` (`editor.js:2261`).
- Snapshot-load failure becomes an empty list without its own load error (`editor.js:2243`).
- Restore failure appears through ErrorNotice (`editor.js:2260`).

**End state:** The selected version becomes the draft. A separate Review/write is needed to change the hosted file.

**Counts**

- Restore to unchanged draft: **2**.
- Restore over changed draft: **3**.
- Include Review/write: **4** or **5**.

## 19. Resolve a conflict with a changed hosted file

**Entry:** A write returns `conflict` (`editor.js:2044`).

The Review alert says:

> `"The hosted file changed after your draft started. Studio did not write."`

Source: `conflict.js:15`.

It can show:

- `"Draft started ${localTime(...)}"`
- `"File changed ${localTime(...)}"`

The explanation is:

> `"The change can come from a different collection, a different Studio version, a different tool or a different person."`

Source: `conflict.js:17`.

**Main path**

1. Click `"Apply My Edits to the New File"` (`editor.js:2157`).
2. Wait for the merge plan.
3. For each conflicting field, pick `"Your edit"` or `"Hosted file"` (`editor.js:1650`).
4. Click `"Apply to the Draft"` (`editor.js:1655`).
5. Inspect the refreshed diff.
6. Click `"Write to the File Host"`.

Steps 2 and 5 add zero unless the user operates a diff control.

The merge headings are (`review.js:336`):

- `"The hosted file changed outside Studio."`
- `"Your edits stay"`
- `"New from the hosted file"`
- `"Both changed. Select one"`

The explanation is:

> `"Studio can apply your edits to the new file. Nothing changes on a file host until you write."`

**Branches**

- Each conflict select starts at `"Select one"`.
- Apply is disabled until all required choices exist (`editor.js:1655`).
- A second conflict clears the current merge plan (`editor.js:2092`).
- The outside-change `"Show the Difference"` action can enter this merge path without a failed write (`app.js:4911`).

**Errors:** Shared errors appear in Review. A new conflict requires another merge attempt.

**Confirmations:** None beyond the explicit merge choices and Apply action.

**End state:** The merged draft appears in Review. A successful write closes Review.

**Counts**

- From displayed conflict, `k` choices through write: **k + 3**.
- Including the initial Review and failed write: **k + 5**.
- No conflicting fields: **3** from the conflict notice.

## 20. Correct a bad address that blocks writing

**Entry:** Review with `draft.bad.length > 0` (`editor.js:2118`).

The alert says (`editor.js:2165`):

> `"Studio cannot write while an address is not correct."`

> `"Correct the fields with red text. Each address starts with https:// or http://."`

It lists `${b.field}: ${b.value}`.

The outline can mark a track `"Address not correct"` (`app.js:3322`).

**Main path**

1. Click `"Close"` in Review (`editor.js:2136`).
2. Reach the marked field, if another page is needed.
3. Correct its address.
4. Click `"Review"`.
5. Click `"Write to the File Host"` after the bad list clears.

**Branches**

`draft.bad` combines bad addresses and bad keys (`draft.js:115`). Thus the address-worded Review block can include a key-related field.

The individual address field uses the shared message:

> `"This is not a full web address. Start it with https://."`

Source: `editor.js:68`.

**Confirmations:** None.

**End state:** The refusal disappears when the saved draft has no bad fields. A successful write closes Review.

**Count:** **4** for one field already on the underlying page. Add navigation steps for another page or track.

## 21. Read the feed again and handle an outside move

**Entries**

- Header `"Read Again"` when `feed.last_version !== null` (`app.js:2287`).
- `"Read Again"` in an outside-change notice (`app.js:4910`).

**Main path**

1. Click `"Read Again"`.

Studio flushes the draft, posts `/refresh`, and reloads the draft (`app.js:2246`).

The notice states:

> `"The hosted file changed outside Studio."`

> `"Read Again takes the new file, and it keeps a draft with edits."`

Source: `outsidecheck.js:47`.

**Outside-move branch**

The notice can say:

- `"A different Studio moved this feed."`
- `"New address"`
- `"The old address has a pointer file."`

Source: `outsidecheck.js:31`.

Possible actions (`app.js:4908`):

| Exact action | Result |
|---|---|
| `"Follow the Move"` | Posts outside/follow and reloads |
| `"Move It Back…"` | Follows the move, then opens the move sheet with the old host |
| `"Show the Difference"` | Opens merge Review |
| `"Keep This File Host"` | Posts outside/keep |
| `"Use This File Host"` | Uses the matching connected host |
| `"Connect a File Host…"` | Opens the host connection boundary |

Show Difference requires a connected target. Move It Back also requires `new_host` (`outsidecheck.js:36`).

The read-only host notice conditions are in `outsidecheck.js:59`.

Outside and index checks run automatically on load and according to their schedule (`app.js:4841`, `outsidecheck.js:163`).

**Errors:** Read-again failures use the header action’s error display. API-owned wording remains runtime text.

**Confirmations:** None for Read Again or Follow the Move.

**End state:** Feed and draft reload. Outside notices change according to the new check state.

**Counts:** Read Again, Follow, or Keep: **1**. Move It Back adds the move-sheet steps from section 23.

## 22. Download release files, stop downloads, and manage copies

**Entry:** `"Files"` (`editor.js:4543`).

The page loads files, move records, and applicable download state automatically (`editor.js:4552`, `editor.js:4573`).

### Download one file

1. Select `"Files"`.
2. Open `${file.path}` (`editor.js:4138`).
3. Click `"Download"` (`editor.js:4062`).

Progress updates automatically. Download polling runs every second (`editor.js:4610`).

A failed download offers `"Try Again"` (`editor.js:4069`).

An active download offers `"Stop"` (`editor.js:4079`).

### Download all

1. Select Files.
2. Click `"Download All"` (`editor.js:4826`).

While downloads are active, `"Stop All"` replaces that action (`editor.js:4825`).

The banner shows:

- `"Downloading ${activeCount} ${activeCount === 1 ? 'file' : 'files'}…"`
- Or `"${data.remote_count} ${data.remote_count === 1 ? 'file is' : 'files are'} not in the library (${fileSize(data.remote_bytes)})."`

Source: `editor.js:4819`.

### Server-device download

A file can offer `"Download to This Device"` (`editor.js:4172`).

Its condition requires server mode, a nonremoved file, and a local copy or hosted copies (`assets.js:45`).

The action gets a ticket and starts a browser download (`editor.js:4669`, `app.js:186`).

### Copy and select a hosted copy

For an open file:

- `"Copy to ${c.name}"` creates a copy operation (`editor.js:4186`).
- `"Use This Copy"` changes the selected copy (`editor.js:4117`).
- `"Delete"` schedules deletion of an unused copy (`editor.js:4116`).
- `"Undo"` cancels a pending operation (`editor.js:4099`).

An in-use copy has no deletion action (`editor.js:4090`).

A remote file that must download first invokes C10 before creating the operation.

Pending operations require Review/write (`editor.js:4729`).

### Missing files

A removed file offers `"Add the File Again"` (`editor.js:4209`).

This opens the native picker and uploads the replacement to the same path. Its accessible picker label is `"Select ${file.path} again"` (`editor.js:4206`).

Missing-file notices include:

- `"1 file of this draft is missing."`
- `"${count} files of this draft are missing."`

Source: `assets.js:61`.

### Other hosted files

`"Other files on the file hosts"` appears when `data.other_host_files.length > 0` (`editor.js:4840`).

- Unused files offer `"Delete"`.
- Used files show disabled Delete with `"Used by the feed · Remove it from the feed first"`.
- Pointer files and feed XML have no delete action in these rows.

Source: `editor.js:4218`.

### Shell and retention branches

When `server && !holdAssets`, the page says (`assets.js:68`, `editor.js:4761`):

> `"This server keeps no audio and no images."`

> `"After each write, Studio removes its copies. Your file hosts and your own computer keep your files."`

Library-download controls depend on `holdAssets` (`editor.js:4819`).

Empty library text: `"No files yet."` (`editor.js:4829`).

**Errors**

- Download row: `${download.message || 'The download failed.'}` (`editor.js:4067`).
- Page-load error replaces the Files content with ErrorNotice (`editor.js:4744`).
- Scan and operation failures appear in the File hosts section (`editor.js:4791`).
- Add-again failure appears under the file (`editor.js:4211`).

**End state:** Downloads update file state without a success toast. Copy and deletion operations remain pending until written.

**Counts**

- Download one: **3**.
- Download all: **2**.
- Stop a visible active download: **1**.
- Stop All from album: **2**.
- Copy a local file: **3**, or **5** including Review/write.
- Copy requiring confirmation: **4**, or **6** including Review/write.
- Download to device: **3** from album.
- Add a missing file again: **4**, including native selection.

## 23. Move a release and handle move-link refusals

**Entries**

- `"Move This Release…"` in Files, when `connectors.length > 1` (`editor.js:4787`).
- `"Move the Files Too"` when a pending feed move leaves media files behind (`editor.js:4765`).
- `"Put the feed on a different file host"` from hosted-feed deletion choices (`feeddelete.js:9`).
- `"Move It Back…"` from an outside-move notice (`app.js:4909`).

### Main path

1. Select `"Files"`.
2. Click `"Move This Release…"`.
3. Pick `"To this file host"` (`editor.js:4341`).
4. Wait for the plan.
5. Click `"Add to the Draft"` (`editor.js:4378`).
6. Click `"Review"`.
7. Click `"Write to the File Host"`.

Step 4 counts as zero.

The sheet title is `"Move ${feed.title || 'This Release'}"` (`editor.js:4331`).

### Move choices

`"What moves"` offers (`editor.js:4336`, `releasemove.js:16`):

- `"Feed and media files"`
- `"Feed file only"`
- `"Media files only"`

Additional controls:

- `"Delete the files on ${place} when the move is complete"` (`editor.js:4367`)
- `"Keep the old files for"` with `"${days} days"` (`editor.js:4369`)

The delete-later choice defaults to true (`releasemove.js:99`).

The plan loads from `/move/plan` when target or scope changes (`editor.js:4315`).

### Branches

- Default target is the first other host. Explicitly selecting it is unnecessary when accepted.
- No work: `"Each file of this release is already on this file host."` (`editor.js:4362`).
- Add is disabled while busy, without a plan, with nothing to move, or with a refusal (`editor.js:4378`).
- Existing different feed file:

> `"A different file is at the feed path."`

> `"${file.host} has a file at ${file.path}. It is not a copy of this feed."`

The UI shows File, Title, GUID, and Size facts, plus `"Replace the File"` (`releasemove.js:164`, `releasemove.js:167`).

### Old-file deletion after a move

In Files, `"Old files to delete"` lists move waits (`editor.js:4808`).

- Ready move: `"Review the Deletion"` or `"Keep the Files"` (`editor.js:4442`).
- Waiting move: `"Delete Now…"` or `"Keep the Files"` (`editor.js:4454`).
- Delete Now invokes C2.
- Review the Deletion creates operations and opens Review (`editor.js:4701`).
- Keep the Files posts `/moves/keep` (`editor.js:4711`).

### Move-link refusal

Review displays `"Studio refuses this write."` (`movelink.js:70`, `editor.js:1727`).

The core condition selects one exact explanation (`movelink.js:11`):

| Condition | Text |
|---|---|
| `loop` | `"The new addresses come back to an address that they visited, or go through more than 10 addresses."` |
| `read_fails` | `"That address does not give a correct feed, or Studio cannot read it."` |
| `guid_mismatch` | `"The feed at that address has a different podcast:guid, so it is a different feed."` |
| `conflicting_names` | `"The feed names two different new addresses."` |
| `self_link_differs` | `"That address is not the address of this feed."` |
| `not_public` | `"That address is not a public address, so apps cannot read it."` |
| Unknown | `"A link of the draft is not correct."` |

The source adds explanatory text (`movelink.js:22`):

- Tag: `"This feed says that it moved to a new address, in the tag itunes:new-feed-url."`
- Self link: `"The draft changes the address that this feed gives for itself, in its self link."`
- Move: `"You selected a move of this feed to a new address."`

Possible repair actions are `"Use the Public Address"` and `"Remove the Tag"` (`movelink.js:65`).

The refusal also lists each hop and its result (`movelink.js:40`):

- `"Gives a feed that names the next address."`
- `"Already in this list. This is a loop."`
- `"The list has 10 addresses already. Studio did not read this address."`
- `"Does not give an RSS feed."`
- `"Gives the status 404 or 410. The file is not there."`
- `"Gives a status that is not a success."`
- `"Studio could not connect."`
- `"The file is too large."`
- `"This is not an http or https address."`
- `"This is not a public address."`
- `"Has a different podcast:guid, or none."`
- `"Names other addresses than the draft."`
- `"Names two different new addresses."`
- `"The self link differs from the address of the feed. Studio did not read it."`

Unknown hop text is `"Studio cannot use this address."` (`movelink.js:105`).

Stage text is (`movelink.js:58`):

- `"Nothing changed on a file host."`
- Or `"The feed stays at its address. The new file stays on the new file host, and the next write replaces it."`

### Nonblocking self-link and return notices

A self-link notice says:

> `"The self link names a different address."`

It offers `"Use the Public Address"` while still permitting a write (`movelink.js:71`, `movelink.js:73`).

A return to an earlier address shows:

> `"This is a return to an earlier address."`

Depending on the API state, it shows immediate-follow, unknown-wait, or:

> `"Music Index waits until ${localTime(until)} before it follows this return. Studio sends the podping after that time."`

Source: `movelink.js:80`.

**Errors:** Move-plan and move-submit failures appear inside the sheet (`editor.js:4354`). Link failures remain in Review.

**Confirmations:** Early deletion uses C2. The normal move uses its plan and Add action, without another confirmation.

**End state:** Add closes the sheet and leaves a pending move. Write completes the move. Old files remain until a later reviewed deletion.

**Counts**

- Move through write, explicit target selection: **6**.
- Accept default target: **5**.
- Change scope, deletion choice, wait, or replacement choice: **+1 each**.
- From Files, ready old-file deletion through Review write: **2**.
- From Files, early deletion through confirmation and Review write: **3**.
- Keep old files from their visible row: **1**.
- Repair a visible move-link refusal, then write: **2**, if that repair resolves all blockers.

## 24. Check library changes and change the release folder

**Entries**

- Files: `"Scan the Library Folder"` (`editor.js:4805`).
- Global notice: `"Show"` (`app.js:4025`).

The global notice says:

> `"Studio found ${differences.length} ${...} in the library folder."`

It opens the first affected feed’s Files page. Close dismisses that notice (`app.js:4022`).

Automatic library scans depend on `holdAssets` (`app.js:3913`).

### Main scan path

1. Select `"Files"`.
2. Click `"Scan the Library Folder"`.
3. Select an action for a detected change.

Possible rows and actions (`editor.js:4259`):

| Change | Visible action | Result |
|---|---|---|
| New file, `"New file: ${path} (${size})"` | `"Add to the Library"` | Registers the file |
| Missing file | `"Forget the File"` | Removes its library record |
| Missing file with source or copy | `"Download Again"` | Starts a replacement download |
| Changed file with hosted copy | `"Restore the Hosted Copy…"` | Invokes C1 |
| Changed file | `"Use the New File"` | Accepts the local change |
| Changed file without copy | `"No copy on a file host"` | No hosted restoration available |

Actions post `/library-changes`, then rescan and reload (`editor.js:4678`).

Row failures appear under the affected row (`editor.js:4293`).

### Change the folder

**Entry:** `"Change…"` beside `"Release folder"` under `"On this computer"` (`editor.js:4795`, `editor.js:4800`).

1. Select Files.
2. Click `"Change…"`. `"Change Library Folder"` appears (`editor.js:4499`).
3. Select the destination.
4. Click `"Move the Folder"` (`editor.js:4531`).

**Desktop-app branch**

`desktopApp` is based on `!serverMode && hasCollections` (`app.js:109`).

It offers `"Choose Folder…"` and a native folder dialog (`editor.js:4507`, `editor.js:4480`).

**Other-shell branch**

The sheet offers a folder browser:

- `"Up One Level"` (`editor.js:4520`)
- `${folder}` (`editor.js:4524`)
- `"New Folder"` (`editor.js:4509`)
- `"New folder"` or `"New folder in ${parent}"` (`editor.js:4512`)
- `"Create Folder"` and `"Cancel"` (`editor.js:4514`)

All branches offer `"Default"` (`editor.js:4508`).

Moving uses `PUT /library-folder` directly. It does not require Review (`editor.js:4754`).

The section appears only when `library_folder` is non-null (`feedfacts.js:43`).

**Confirmations:** Hosted restoration uses C1. Folder movement has no separate confirmation in this sheet.

**End state:** Change rows update after actions. Folder movement closes the sheet and updates the displayed folder.

**Counts**

- Scan: **2** from album.
- Scan plus one direct action: **3**.
- Scan plus hosted restoration confirmation: **4**.
- Change folder using Default: **4** from album.
- Desktop native folder selection: **5**.
- Folder browsing adds one step per folder or Up action.
- Create a folder adds New Folder, name, and Create Folder: **3**.

## 25. Examine feed facts or remove the feed from the collection

### Feed facts

**Entry:** `"Feed facts"` disclosure in the main feed page (`editor.js:2348`).

1. Open `"Feed facts"`.

Its summary is `"GUID, file, version and announcement"`.

It displays (`editor.js:2313`):

- Address source
- New feed address, when present
- Feed GUID
- File
- Version, or `"Not read yet"`
- Announcement state

Announcement state polls every five seconds while pending (`editor.js:2327`).

**Count:** **1**. The disclosure remains open until closed or the page changes.

### Remove from the collection

**Entry:** `"Remove from Collection"` (`editor.js:2362`).

1. Click `"Remove from Collection"`.
2. Confirm the same action in C3.

The API deletes the feed record (`editor.js:2339`). The application reloads and replaces the view with Releases (`app.js:3998`).

**Branches:** No file-host connection is required for this collection action.

**Errors:** Failure remains in the editor action’s ErrorNotice.

**End state:** Releases appears. No collection-removal success message is rendered by this path.

**Count:** **2**, once the button is visible.

## 26. Retire a feed or delete its hosted feed file

**Entry:** `"Delete from File Host…"` when `connector_id != null` (`editor.js:2364`).

### Choice sheet

1. Click `"Delete from File Host…"`.
2. `"What do you want to do?"` appears (`editor.js:2490`).

Choices (`feeddelete.js:6`):

- `"Put the feed on a different file host"` → move sheet.
- `"Take the feed out of the apps"` → retirement draft.
- `"Remove the file"` → final deletion sheet.

`"Remove the file"` appears only when `state?.removal_done === true` (`feeddelete.js:29`).

Cancel, Close, or Escape closes the choice sheet.

### Retire

1. Click `"Take the feed out of the apps"`.
2. Click `"Review"`.
3. Click `"Write to the File Host"`.
4. Wait for removal from Music Index.
5. Use `"Check Again"` as needed (`editor.js:2486`).

The retirement block can show `"Retire"` and the literal summary `"Studio cannot check Podcast Index."` (`editor.js:2477`).

Its steps are (`feeddelete.js:80`):

1. `"The feed asks to be removed"`
2. `"Music Index removes the entry"`
3. `"Delete the file"`

State messages include (`feeddelete.js:62`):

- `"Hide from apps is on in the draft."`
- `"Hide from apps is on in the last write."`
- `"Hide from apps is off in the draft. The last write still hides the feed."`
- `"Hide from apps is not on in the draft or last write."`
- `"Music Index still holds this feed."`
- `"Music Index does not hold this feed."`
- `"Music Index holds this feed with a different GUID."`
- `"Studio could not read ${removal.address}."`
- `"This feed has no GUID. Studio did not send a request."`
- `"Studio has not checked Music Index."`
- `"Wait until Music Index removes the entry."`

`"Stop Retiring"` removes the retirement request through the API (`editor.js:2487`).

### Final deletion

After Music Index removal:

1. Click `"Remove the file"` in the choice sheet, or `"Delete the File"` in retirement.
2. `"Delete the Feed File"` appears (`editor.js:2501`).
3. Type `"Type the title of the feed: ${preview ? preview.names.confirm_text : ''}"` (`editor.js:2509`).
4. Click `"Delete"` (`editor.js:2513`).
5. Review opens automatically.
6. Click the destructive write action.

Steps 2 and 5 are appearances, not user actions.

The destructive write label is (`feeddelete.js:92`):

- `"Delete the Feed File"`
- Or `"Delete ${deletes.length} Files"`.

The final sheet shows File host and Path (`editor.js:2503`).

Its optional checkbox is `"Also delete the audio and image files"` (`editor.js:2507`). It defaults to false.

The title must match exactly (`feeddelete.js:88`).

The sheet also states (`feeddelete.js:49`):

- `"Music Index removed the entry"`
- `"Studio keeps a copy of the feed"`
- `"You can put the file back later with"` **Put the File Back** `"on this feed"`

Close, Cancel, or Escape discards the pending deletion draft unless the user continued into Review (`editor.js:2455`).

### Deletion limits and restoration

Pointer files and files used by other feeds can remain.

Review can show `"Not deleted"` with (`feeddelete.js:111`):

- `"Studio does not delete this file. It tells the apps the new address of the feed."`
- Or `"Studio does not delete this file. These feeds use it: ${blocked.feed_titles.join(', ')}"`

An Undo action is rendered for blocked operations (`editor.js:1809`).

After deletion, retirement actions can offer `"Put the File Back"` (`editor.js:2484`).

That action creates a restore draft and opens Review. A usual write restores the hosted file (`editor.js:2450`).

### Domain notice

When `address_host` exists, the UI says:

> `"Keep the domain ${host}."`

Before deletion:

> `"If it expires, its new owner can take over this feed and its payments."`

After deletion:

> `"If it expires, its new owner can publish a feed with the same GUID and collect the payments that apps still send to it."`

Source: `feeddelete.js:37`.

**Errors:** Preview, retirement, and deletion failures appear in the current sheet or retirement block (`editor.js:2488`, `editor.js:2511`).

**Confirmations:** Final deletion uses the title-entry sheet and destructive Review write. It does not invoke a separate `askConfirm`.

**End state:** The feed remains in the collection with deleted-host state. Restoration remains available through the retirement block.

**Counts**

- Already removed from Music Index, from album through host deletion: **5**.
- Include audio/image checkbox: **6**.
- Start retirement from album through its first write: **4**.
- Each manual retirement check: **+1**.
- From an available retirement Delete the File button through deletion write: **4**.
- Restore deleted file through its automatically opened Review: **2**.
- Stop Retiring from its visible block: **1**.

## 27. Edit artist and label feed specifics

Both use `PublisherPage` (`editor.js:3824`). The role determines the page identity.

### Change roles

**Entry:** `"Role"` section (`editor.js:3925`).

1. Select the role menu (`editor.js:3930`).
2. Pick Artist, Label, Hosting, or `"Custom"`.

Custom exposes role checkboxes (`editor.js:3938`).

The code’s role set is:

`artist`, `label`, `hosting`, `curator`, `producer`, `host`, `author`, `network`, `sponsor`.

Source: `shell.js:12`.

**Branches**

- Role selection is disabled for the read-only condition, `!connector_id` (`editor.js:3930`).
- The last role cannot be removed (`editor.js:3940`).
- Role changes use `PUT /role` and can change related drafts (`editor.js:3874`).
- Role-required text is `"Select the role of this feed first."` (`editor.js:3819`).

### Publisher fields

The main fields match the album field group, with `"Name"` instead of Artist (`editor.js:3907`).

That field’s hint is `"The itunes:author of the feed."` (`editor.js:3908`).

### Link an existing release

1. Pick `"Release"` (`editor.js:3973`).
2. Click `"Add the Release"` (`editor.js:3982`).

The placeholder is `"Select a release"` (`editor.js:3976`).

Existing release rows have position selection and `"Remove"` (`editor.js:3958`, `editor.js:3967`).

Spaces in imported link addresses can expose:

- `"Correct the Address"`
- `"Correct All ${count}"`

Sources: `editor.js:3952`, `feedfacts.js:19`.

These use the core address-encoding route (`editor.js:3853`).

### Start a new release or artist page

The current `releaseAdd()` returns `"Add a Release"` for a connected publisher feed containing the artist role (`welcome.js:129`).

The current code does not give every label feed that same row.

First-step conditions use no release IDs and no child IDs (`welcome.js:117`).

Possible first-step controls (`welcome.js:144`, `editor.js:4019`):

- `"Add your first album"`
- `"Add a Release"`
- `"Add an artist"`
- `"Add a release"`
- `"Find an artist that is online"`

They open `AddFeedRowSheet`, an editor boundary (`editor.js:4032`).

A release-artist selection sheet can show (`editor.js:2602`):

- `"Add a release"`
- `"Artist"`
- `"Select an artist page"`
- `"Make an Artist Page"` when no eligible artist page exists.

**UNSURE:** Completing the new-feed forms requires the add-feed modules outside this run’s detailed scope.

### People link boundary

When members exist, the publisher page has a `"People"` section. Clicking `${m.name}` opens `{ name: 'person', id }` (`editor.js:3918`, `editor.js:3920`).

The person page itself is excluded.

**Errors:** Role and link action errors appear in their sections (`editor.js:3946`, `editor.js:3984`). Related-draft notices use section 4.

**Confirmations:** No confirmation for role changes, release linking, position changes, or release unlinking.

**End state:** Roles and linked releases update. Related-draft notices can remain.

**Counts**

- Choose a standard role: **1**.
- Select Custom, then toggle one role: **2**.
- Link an existing release: **2**.
- Reorder, remove, or correct a visible release link: **1**.
- Open a linked person: **1**.

## 28. Create and edit a live show inside the editor

Live Shows is available for album feeds (`editor.js:5033`, `editor.js:5035`).

### Create a show

1. Select `"Live Shows"`.
2. Click `"＋ Add a Live Show"` (`editor.js:3502`).
3. Type `"Title"` in `"Add a live show"` (`editor.js:3448`, `editor.js:3453`).
4. Set `"Start time"` (`editor.js:3454`).
5. Click `"＋ Add the Show"` (`editor.js:3460`).

Title and start are required (`editor.js:3441`).

Studio posts `/live-items` and opens the new live-item page (`editor.js:5059`).

An empty list says `"No live show yet."` (`editor.js:3491`).

Existing rows use `${live.item.title || 'Untitled show'}` (`editor.js:3498`).

### Edit show fields

The Show section contains (`editor.js:3676`):

- `"Title"`
- `"Description"`
- `"Start time"`
- `"End time"`
- `"Image address"`

The image uses the shared picker (`editor.js:3683`).

### Configure and check a stream

When `!canRecord(live)`, the Stream section appears (`editor.js:3689`).

Controls:

- `"Stream address"` (`editor.js:3691`)
- `"Stream type"` (`editor.js:3694`)
- `"Choose a type"` (`editor.js:3697`)
- `"Other…"` (`editor.js:3699`)
- `"Other stream type"` (`editor.js:3701`)
- Field label `"Check Stream"` and button `"Check"` / `"Checking…"` (`editor.js:3707`)

Main check path:

1. Type Stream address.
2. Pick Stream type, if necessary.
3. Click `"Check"`.

The UI can infer a type through `/stream/type` when none is selected (`editor.js:3553`).

The result uses `${checkResult.text}`. A warning uses `${check.typeWarning.text}` and can offer `"Use ${check.typeWarning.fillLabel}"` (`editor.js:3713`).

Errors render `${checkError.message}` (`editor.js:3714`).

### Go live or end the show

Status labels include `"Planned"`, `"Live now"`, `"Ended"`, and `"No status"` (`live.js:57`).

The next action is:

- `"Go Live"` for pending shows.
- `"End the Show"` for live shows.
- No next-status button for ended shows.

Source: `live.js:68`.

Path:

1. Click the status action (`editor.js:3658`).
2. Review opens automatically.
3. Click `"Write to the File Host"`.

The status route updates the draft before opening Review (`editor.js:3630`).

Core validation problems render `${p.text}` by group (`editor.js:3600`). `stream_not_live` errors use `"Go Live refused."` (`app.js:481`).

### Recording after the show

`canRecord(live)` is true for ended shows (`live.js:76`).

The Recording section then replaces Stream (`editor.js:3721`).

It shows the shared audio enclosure editor when recording audio exists. Otherwise, it offers audio selection. Selecting recording audio replaces the stream enclosure (`editor.js:3731`).

### Content links and live value

`"Add a content link"` creates an open entry (`editor.js:3482`).

Fields:

- `"Link address"`
- `"Link text"`

Source: `editor.js:3476`.

Common Remove and Move controls apply.

Live value fields are:

- `"Live value address"`
- `"Live value protocol"`
- `"Remove the Live Value"`

Source: `editor.js:3743`.

The show also has track-style recipient routes (`editor.js:3750`).

Identity displays `"Show GUID"` (`editor.js:3755`).

### Show grants

A Grant section appears when grants and GUID are available (`editor.js:3664`).

`"Stop the Grant"` invokes C8, then ends the grant immediately through the API (`editor.js:3626`, `editor.js:3671`).

An active grant blocked by draft changes can show (`editor.js:5080`, `apps.js:144`):

> `"${app} cannot start ${show} while this draft has changes"`

> `"Write or discard the draft before ${startText || 'the show starts'}. Select Review to write it."`

### Remove a show

1. Click `"Remove the Show"` (`editor.js:3767`).
2. Confirm C9.

The show leaves the draft. Live Shows opens (`editor.js:3641`).

**End state:** New and edited shows remain in the draft. Status writes close Review. Grant termination acts immediately.

**Counts**

- Create show from album: **5**.
- Open an existing show from album: **2**.
- Change one field from its open page: **1**.
- Check stream with address and explicit type: **3**.
- Accept automatically supplied type: **2**.
- Go Live or End through write, from open show: **2**.
- Stop grant: **2**.
- Remove show: **2**.
- Add a content link with address and text: **3**.
- Set both live-value fields: **2**.

## 29. Confirmation sheets

All confirmations use the shared renderer (`app.js:5410`).

- Cancel receives first focus.
- Destructive action appears before Cancel.
- Close or Escape cancels.
- The download confirmation places Cancel before its primary action.

Source: `confirms.js:83`.

| ID | Exact title | Exact body | Action and source |
|---|---|---|---|
| C1 | `"Restore the hosted copy of “${name}”?"` | `"Studio downloads the copy on the file host and writes it over the changed file in your folder. The changes in the local file are lost."` | `"Restore the Hosted Copy"` + `"Cancel"` — `confirms.js:7` |
| C2 | `"Delete ${count === 1 ? 'the old file' : \`the ${count} old files\`} before the move is ready?"` | `"An app that still uses an old address gets an error. You review each deletion, and ${host} keeps the files until you write."` | `"Review the Deletion"` + `"Cancel"` — `confirms.js:11` |
| C3 | `"Remove “${name}” from your collection?"` | `"The feed file on its file host does not change. You can add the feed again later."` | `"Remove from Collection"` + `"Cancel"` — `confirms.js:15` |
| C4 | `"Discard the draft?"` | `"Studio discards any change that you did not write. The feed file on its file host does not change."` | `"Discard the Draft"` + `"Cancel"` — `confirms.js:31` |
| C5 | `"Edit these values by hand?"` | `"Studio reads these values from the audio file. A wrong value stays in your feed, and apps can show the wrong length or fail to play the file. To change them safely, select a new audio file."` | `"Edit Values"` + `"Cancel"` — `confirms.js:35` |
| C6 | `"Replace the draft with this version?"` | `"Studio deletes each change in the draft. Then it starts a new draft from this earlier version."` | `"Replace the Draft"` + `"Cancel"` — `confirms.js:39` |
| C7 | `"Remove the credits of this track?"` | `"The track then uses the credits of the album."` | `"Remove the Credits"` + `"Cancel"` — `confirms.js:43` |
| C8 | `"Stop the grant?"` | `"Connected apps can no longer change this show."` | `"Stop the Grant"` + `"Cancel"` — `confirms.js:47` |
| C9 | `"Remove “${title || 'Untitled show'}”?"` | `"Studio removes the show from the draft. The feed changes at the next write.${live && live.status === 'live' ? ' Apps lose the stream of this show at that write.' : ''}"` | `"Remove the Show"` + `"Cancel"` — `confirms.js:51` |
| C10 | `"Download this file first?"` | `"Studio downloads the file${size ? \` (${size})\` : ''} first. The copy goes into the draft now, and the write waits for the download."` | `"Cancel"` + `"Download the File"` — `confirms.js:56` |
| C11 | `"Remove the recipients of this ${noun}?"` | `"The ${noun} then uses the recipients of the album."` | `"Remove the Recipients"` + `"Cancel"` — `confirms.js:75` |

Other `confirms.js` entries serve app, host, People, or change-set journeys outside this editor scope.

## 30. Page, sheet, dialog, and navigation registry

### Editor pages

The routing component is `AlbumEditor` (`editor.js:4963`). Its dispatch is at `editor.js:5017`.

| Surface | Render code | Show condition | Entry and exit |
|---|---|---|---|
| Feed outline | `app.js:3287`, `app.js:3359` | Navigation record `name: 'outline'` | Narrow feed open. Head or track opens editor. Back returns through navigation history. |
| Album main page | `editor.js:2839`, `editor.js:5047` | Album fallback/main page | Feed open or first tab. Tabs, outline, Back leave it. |
| Publisher main page | `editor.js:3824`, `editor.js:5022` | Publisher, except common dedicated pages | Feed open or first tab. |
| Playlist main page | `editor.js:2873`, `editor.js:5027` | Playlist, except its dedicated pages | Feed open or first tab. |
| Playlist Tracks | `editor.js:2894`, `editor.js:5025` | Playlist and `page === 'tracks'` | Tracks tab. Other page choices leave it. |
| Track | `editor.js:3277`, `editor.js:5029` | Valid track key | Track row, outline, pager, new address track, Check. Back or another navigation record leaves it. |
| Live Shows | `editor.js:3491`, `editor.js:5035` | Album and `page === 'lives'` | Live Shows tab or removed-show return. |
| Live item | `editor.js:3506`, `editor.js:5033` | Album and valid live key | Show row or newly created show. Back or Live Shows leaves it. |
| Credits | `editor.js:3213`, `editor.js:5037` | `page === 'credits'` after type dispatch | Credits tab. |
| Value Routes | `editor.js:5039`, `values.js:520` | `page === 'recipients'` | Value Routes tab. |
| Files | `editor.js:4543`, `editor.js:5041` | `page === 'files'` | Files tab or library notice Show. |
| Check | `editor.js:4854`, `editor.js:5017` | `page === 'check'` | Check tab. |
| Feed settings | `editor.js:2726`, `editor.js:5019` | `page === 'settings'` | Settings tab or Select a Language repair. |

These are JavaScript navigation records. The inspected routing does not assign a separate pathname to each editor page.

Navigation state is stored as `"studio-nav"` in session storage (`app.js:3647`). Browser history mirrors navigation indices (`app.js:3715`).

### Sheets and dialogs

`useSheet` calls `showModal()` and closes the native dialog when its open state becomes false (`editor.js:1921`).

Native Escape invokes each dialog’s close behavior. The helper does not implement click-outside dismissal.

| Surface/title | Renderer | Condition and entry | Exit |
|---|---|---|---|
| `"Add tracks"` import | `editor.js:1291`, `editor.js:1473` | `importing`; Add tracks or computer-file switch | Add, Cancel, Close, Escape |
| `"Add a track from an address"` | `editor.js:575` | `adding === true`; online-audio action | Add, switch to files, Cancel, Close, Escape |
| `"Add a live show"` | `editor.js:3431` | `adding === 'live'` | Add, Cancel, Close, Escape |
| `"Artwork · ${title}"` | `editor.js:3168` | Artwork target exists | Done, Close, Escape |
| `"Choose an image"` | `editor.js:1059` | Image picker active | Select file, successful upload, Cancel, Close, Escape |
| `"Choose an audio file"` | `editor.js:1059` | Audio picker active | Same |
| `"Choose a lyrics file"` | `editor.js:1059` | Lyrics picker active | Same |
| `"Add a Credit"` | `editor.js:186` | Add-credit state | Add, Close, Escape |
| `"Add a Support Link"` | `editor.js:261` | Add-support state | Add, Close, Escape |
| `"Find a Nostr profile"` | `editor.js:682` | Nostr picker active | Select, Remove, Cancel, Close, Escape |
| `"Find on Music Index"` | `pickers.js:286` | Parent `finding` state | Pick, Cancel, Escape. Back returns from choices. |
| `"Add a Recipient"` | `values.js:219`, `values.js:258` | Recipient add state | Add, Close, Escape |
| `"Change the Address"` | `values.js:219`, `values.js:258` | Existing recipient selected | Use This Address, Close, Escape |
| `"Copy from a Feed on Music Index"` | `values.js:362` | Copy-recipient picker active | Add, Cancel, Close/Escape through dialog |
| `"Add tracks"` playlist picker | `editor.js:2938` | Playlist adding state | Done, Close, Escape |
| `"Review changes"` | `editor.js:1958` | Toolbar or Review event | Write success, discard success, Close, Escape, host/page navigation |
| Merge panel inside Review | `editor.js:1634` | `merge` exists | Apply returns to diff. Close/Escape closes Review. |
| `"Earlier versions"` | `editor.js:2236` | Versions open state | Restore success, Close, Escape |
| `"What do you want to do?"` | `editor.js:2489` | Hosted-delete choice active | Choice, Cancel, Close, Escape |
| `"Delete the Feed File"` | `editor.js:2500` | Final-delete state | Delete opens Review. Cancel/Close/Escape discards deletion draft. |
| `"Move ${feed.title || 'This Release'}"` | `editor.js:4300` | Move event for current feed | Add to Draft, Cancel, Close, Escape |
| `"Change Library Folder"` | `editor.js:4463` | Files folder-change state | Move the Folder, Cancel, Close, Escape |
| `"A Copy Pays Other People"` | `editor.js:4929` | `sheet.action === 'show-copy'` | Close, Escape |
| `"Music Index Has Another Address"` | `editor.js:4938` | `sheet.action === 'show-ways'` | Close, Escape |
| Shared confirmation | `app.js:5410` | `askConfirm` pending | Action, Cancel, Close, Escape |
| Native file picker | `editor.js:1488`, `editor.js:1105`, `editor.js:4206` | File input activation | Operating-system selection or cancel |
| Native folder picker | `editor.js:4480` | Desktop Choose Folder | Operating-system selection or cancel |

`MoveAllSheet` also exists in this source file (`editor.js:4385`). Its title is `"Move All Releases from ${connector.name}"`.

Its entry belongs to file-host management. It is not a feed-editor entry in the paths above.

## 31. Routes used by these journeys

These are API routes, not browser-page URLs. `api()` prefixes them with `/api/v1` (`app.js:196`).

### Feed, draft, and publishing routes

| Route | Use and source |
|---|---|
| `GET /feeds/${feedId}/draft` | Load draft — `draft.js:185` |
| Draft save at `/feeds/${feedId}/draft` | Save model and revision — `draft.js:220` |
| `DELETE /feeds/${feedId}/draft` | Discard — `editor.js:2070` |
| `GET /feeds/${feedId}/diff` | Review diff — `editor.js:1975` |
| `GET /feeds/${feedId}/link-check` | Check move/self links — `editor.js:1971` |
| `POST /feeds/${feedId}/write` | Write reviewed diff — `editor.js:2040` |
| `GET /feeds/${feedId}/draft/merge` | Merge plan — `editor.js:2083` |
| `POST /feeds/${feedId}/draft/merge` | Apply choices — `editor.js:2089` |
| `POST /feeds/${feedId}/draft/use-public-address` | Repair self/public address — `editor.js:2002`, `movelink.js:66` |
| `POST /feeds/${feedId}/draft/remove-tag` | Remove move tag — `editor.js:2002`, `movelink.js:67` |
| `GET /feeds/${feedId}/snapshots` | Version list — `editor.js:2243` |
| `POST /feeds/${feedId}/snapshots/${snapshot.id}/draft` | Restore version — `editor.js:2249` |
| `POST /feeds/${feed.id}/refresh` | Read Again — `app.js:2246` |
| `DELETE /feeds/${feed.id}` | Remove from collection — `editor.js:2339` |
| `PUT /feeds/${feed.id}/kind` | Release kind — `editor.js:2710` |
| `POST /feeds/${feedId}/listing${path}` | Global/platform listing choice — `editor.js:2809` |
| `PUT /feeds/${feed.id}/role` | Publisher roles — `editor.js:3874` |
| `POST /feeds/${chosen}/links` | Link release and publisher — `editor.js:2551` |
| `POST /feeds/${feed.id}/links` | Publisher-side link — `editor.js:3881` |
| `DELETE /feeds/${row.match.id}/links/${feed.id}` | Release-side unlink — `editor.js:2548` |
| `DELETE /feeds/${feed.id}/links/${row.match.id}` | Publisher-side unlink — `editor.js:3894` |
| `POST /feeds/${feed.id}/draft/encode-link-spaces` | Correct link spaces — `editor.js:3853` |
| `GET /feeds/${feed.id}/members` | Publisher People links — `editor.js:3838` |
| `/feeds/${feed.id}/check/reads` | Read feed-check addresses — `app.js:2112` |
| `/feeds/${feed.id}/check/repair` | Apply a repair — `app.js:2136` |
| `/feeds/${feed.id}/outside/follow` | Follow outside move — `app.js:4908` |
| `/feeds/${feed.id}/outside/keep` | Keep current host — `app.js:4912` |
| `GET /feeds/${feed.id}/announcements` | Feed facts announcement state — `editor.js:2319` |
| `GET/PUT /settings/announcements` | Podping preference — `editor.js:1831` |

### File and move routes

| Route | Use and source |
|---|---|
| `GET /feeds/${feed.id}/files` | Files page — `editor.js:4573` |
| `DELETE /feeds/${feedId}/files?file=${facts.library_file_id}` | Remove unused pending import — `editor.js:930`, `editor.js:1388` |
| `POST /feeds/${feedId}/operations` | Pending file action or unused-file cleanup — `editor.js:910`, `editor.js:4729` |
| `DELETE /feeds/${feedId}/operations/${operationId}` | Undo action — `editor.js:1984`, `editor.js:4739` |
| `PUT /feeds/${feedId}/feed-host` | Select write host — `editor.js:1993` |
| `GET /feeds/${feed.id}/downloads` | Download states — `editor.js:4595` |
| `POST /feeds/${feed.id}/downloads` | Start downloads — `editor.js:4617` |
| `DELETE /feeds/${feed.id}/downloads/${fileId}` | Stop download — `editor.js:4621` |
| `POST /hosts/${id}/scan` | Scan release file hosts — `editor.js:4650` |
| `POST /feeds/${feed.id}/library-changes` | Resolve local changes — `editor.js:4678` |
| `PUT /feeds/${feed.id}/library-folder` | Move release folder — `editor.js:4754` |
| `GET /library/folders${...}` | Folder browser — `editor.js:4475` |
| `POST /library/folders` | Create folder — `editor.js:4486` |
| `POST /choose-folder` | Native folder dialog — `editor.js:4480` |
| `GET /feeds/${feed.id}/move/plan?connector_id=${chosen}&what=${what}` | Move plan — `editor.js:4315` |
| `POST /feeds/${feed.id}/move` | Add move to draft — `editor.js:4322` |
| `GET /feeds/${feed.id}/moves` | Old move records — `editor.js:4552` |
| `POST /feeds/${feed.id}/moves/delete` | Review old-file deletion — `editor.js:4701` |
| `POST /feeds/${feed.id}/moves/keep` | Keep old copies — `editor.js:4711` |
| `GET /feeds/${feedId}/return-wait` | Earlier-address wait — `editor.js:1762` |
| `POST /feeds/${feed.id}/retire/state` | Retirement status — `editor.js:2396` |
| `POST/DELETE /feeds/${feed.id}/retire` | Start/stop retirement — `editor.js:2436`, `editor.js:2442` |
| `POST /feeds/${feed.id}/delete/draft` | Deletion preview and draft — `editor.js:2424` |
| `POST /feeds/${feed.id}/delete/restore` | Restore hosted file draft — `editor.js:2451` |

### Media, search, recipient, and live routes

| Route | Use and source |
|---|---|
| `POST /media/facts` | Audio or image facts — `editor.js:608`, `editor.js:3070` |
| `POST /media/tickets` | Held playback or device download — `editor.js:462`, `editor.js:4669` |
| `GET /images?file=${source.file}` | Held image read — `editor.js:3060` |
| `GET /nostr/profiles?q=${...}${suffix}` | Nostr profile search — `editor.js:758`, `editor.js:772` |
| `GET /people` | Saved picker choices — `editor.js:200`, `values.js:238` |
| `GET /recipients` | Known recipients and lookup state — `values.js:74` |
| `POST /recipients/saved` | Add to People — `values.js:118`, `values.js:299` |
| `GET /musicindex/fields?q=${...}&cursor=${at}${scopePart}` | Search field list — `pickers.js:61` |
| `GET /musicindex/artist-names?from=${...}` | Artist-name completion — `pickers.js:73` |
| `GET /musicindex/values/${...}${credits ? '?credits=true' : ''}` | Values from selected result — `pickers.js:305` |
| `GET /musicindex/feeds/${...}/recipients` | Copy feed recipients — `values.js:375` |
| Track-recipient route built at `values.js:381` | Copy track recipients |
| `GET /search?q=${...}` | Playlist feed search — `editor.js:2969` |
| `GET /musicindex/feeds/${...}/tracks` | Remote playlist choices — `editor.js:2976` |
| `GET /playlist/tracks` | Collection playlist choices — `editor.js:2960` |
| `POST /stream/type` | Infer stream type — `editor.js:3554` |
| `POST /stream/check` | Check stream — `editor.js:3577` |
| `POST /feeds/${feed.id}/live-items` | Create live item — `editor.js:5061` |
| `POST /feeds/${feed.id}/live-items/${...}/status` | Go Live/End — `editor.js:3634` |
| `GET /feeds/${feed.id}/grants` | Show grants — `editor.js:3609` |
| `POST /feeds/${feed.id}/grants/${...}/end` | Stop grant — `editor.js:3626` |
| Grant route constructed at `editor.js:1879` | Review grant selection |
| `GET /apps` | Available connected apps — `editor.js:3619` |
| `GET /signer` | Signer state — `editor.js:1952` |
| `POST /signer/sign` | Remote signing — `editor.js:1940` |

**UNSURE — route completeness beyond callers:** Upload transport, ticket-consumption URLs, and some list routes are delegated to shared helpers. Their visible user paths are inventoried above.

## 32. Remaining visible controls and keyboard operations

These controls supplement the main paths above.

| Control | Exact text or accessible label | Behavior and source |
|---|---|---|
| Narrow root navigation | `"Destinations"` | Opens root navigation — `app.js:1963` |
| History | `"Back"` | Goes to earlier navigation record — `app.js:1972` |
| Mouse history | `"Forward"` | Goes to later navigation record — `app.js:1976` |
| Narrow title control | Runtime page title | Opens its navigation target — `app.js:1963` |
| Previous track | `"The track before this one"` | Opens previous track. Disabled at first track — `editor.js:3403` |
| Next track | `"The track after this one"` | Opens next track. Disabled at last track — `editor.js:3405` |
| Date shortcut | `"Now"` | Sets current time — `app.js:520` |
| Date reset | `"Clear"` | Clears optional time — `app.js:521` |
| Duration input | `"Duration"` | Accepts minutes and seconds — `app.js:531` |
| Lyrics add | `"Add a lyrics file"` | Creates an open lyrics entry — `editor.js:3360` |
| Lyrics address | `"Lyrics file address"` | Typed address or shared picker — `editor.js:3244` |
| Lyrics type | `"Type"` | `text/plain`, `text/html`, `text/vtt`, `application/json`, `application/srt`, plus preserved unknown type — `editor.js:3244` |
| Lyrics language | `"Language"` | Changes transcript language — `editor.js:3244` |
| Lyrics captions | `"Captions"` / `"These are captions"` | Changes transcript relation — `editor.js:3244` |
| Common entry actions | `"Remove"`, `"Move Up"`, `"Move Down"` | Acts on credits, support links, images, lyrics, and content links — `editor.js:113` |
| File-host scan | `"Scan the File Hosts"` / `"Scanning…"` | Scans the release’s hosts — `editor.js:4789` |
| Pending-copy action | `"Use This Copy"` | Selects a pending hosted copy — `editor.js:4181` |
| Pending-copy removal | `"Delete"` | Removes or schedules the corresponding copy operation — `editor.js:4180` |
| File row disclosure | `${file.path}` | Opens file detail — `editor.js:4138` |
| Picker folder | `${folder.name}` | Opens or closes folder — `editor.js:1018` |
| Picker source | `"This Release"`, `"File Hosts"`, `"This Computer"` | Changes source — `picker.js:61` |
| Picker source host | `"File host"` | Changes listed host — `editor.js:1094` |
| Picker fold | `${title}` with `${rows.length}` | Opens or closes saved-source list — `pickers.js:206` |
| Recipient selection panel | `"${total} Selected"` | Shows selected recipients — `values.js:485` |
| Share segment | `${partDetail(p)}` | Keyboard-focusable share description when permitted — `values.js:43` |
| Public URL | `${url}` | Opens hosted feed externally — `app.js:1950` |
| Public URL copy | Copy control | Copies address — `app.js:1951` |
| Existing Nostr copy | Copy control | Copies key — `editor.js:700` |

### Additional field failures

Duration validation can show:

> `"Type the duration as minutes and seconds, for example 4:21."`

Source: `duration.js:45`.

An unreadable existing date can show:

> `"The feed has the date “${text}”. Studio cannot read it. Select a time to replace it."`

Source: `datetime.js:65`.

Unreadable source duration also has a preservation hint (`duration.js:49`).

### List keyboard behavior

The list helpers implement (`listkeys.js:73`):

- Arrow Up and Arrow Down move focus.
- Home and End move to boundaries.
- Arrow Right expands an expandable row.
- Arrow Left collapses it or reaches its parent.
- Enter activates a row or toggles its disclosure.

Relevant handlers are at `listkeys.js:102`, `listkeys.js:128`, and `listkeys.js:172`.

These are alternatives to the corresponding click steps. They do not add separate job outcomes.

### Helpers without separate destinations

`breaks.js`, `casing.js`, `order.js`, `explicit.js`, `cover.js`, `roles.js`, and related model helpers do not create separate page destinations. Their effects appear through the fields, summaries, choices, and list operations inventoried above.
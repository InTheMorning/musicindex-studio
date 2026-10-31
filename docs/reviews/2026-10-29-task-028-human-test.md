# Task 028 Human Test: An R2 Bucket On studio.musicindex.org

- Date: 2026-10-29
- Task: [task 028](../tasks/studio-v1-task-028-s3-connector.md)
- Decision: [ADR 0046](../adr/0046-the-s3-and-r2-connector.md)
- Studio: `studio.musicindex.org`, `0.1.0-rc.4`, in Chromium 156.0.8078.47 on Linux
- Bucket: `heycitizen`, with the custom domain `music.cancelbandcamp.com`

The operator does each step. Claude records the result. Claude saw the Studio screens through screenshots from the DevTools port of the Chromium window. The operator did the Cloudflare steps in a private Firefox window. Claude saw only the screenshots that the operator sent. A step that did not occur is an open gate.

## The R2 API Token

The operator did these steps on 2026-10-29:

1. In the Cloudflare dashboard, the operator opened the API tokens page of R2. The page has two sections: **Account API Tokens** and **User API Tokens**. Cloudflare marks the account token as "recommended" for production systems.
2. The operator clicked **Create Account API token**.
3. The operator typed the name `Musicindex-Studio`, selected **Object Read & Write**, and selected **Apply to specific buckets only** with the bucket `heycitizen`.
4. The operator kept **TTL** at **Forever**, and left **Client IP Address Filtering** empty.
5. The operator clicked **Create Account API Token**. The result page gave **Token value**, **Access Key ID**, **Secret Access Key** and a **Default** endpoint in the section "Use jurisdiction-specific endpoints for S3 clients". Studio uses only the three S3 values.

The description of **Object Read & Write** says "read, write, and list objects". It does not say "delete". The provider test of ADR 0046 item 4 deletes its probe key, so the host test examines the delete permission.

## Observations

### The Options Of A Select Menu In Chromium

On 2026-10-29, the operator opened the **Address style** select menu of the S3 or R2 form. Chromium 156 on Linux showed the open list with a light gray background. The selected option, **Path style**, had a light blue background with dark text. The other option, **Virtual host style**, had near-white text on the light gray background, and it was almost not readable.

`styles.css` sets `color-scheme: dark` on `:root`, and `.field select` sets `color: var(--text)`. The text of each option thus is light. Chromium on Linux drew the background of the list in a light color. This explanation is not confirmed.

Claude compared the other two apps. `search.html` of musicindex.org has no `select` element, and v4vmm is a native app. Thus no shared rule owns the control.

The cause: `.field select` has the fill `--fill-soft`, which is transparent. Chromium on Linux drew the open list on that fill.

The correction gives each `option` the level color `--level-3` and the text color `--text`. These colors have no transparency. The computed colors were these:

- Dark theme: text `rgb(245, 245, 247)` on `rgb(38, 38, 42)`.
- Light theme: text `rgb(29, 29, 31)` on `rgb(236, 236, 239)`.

A screenshot does not show an open select list, so the operator did the visual check. On 2026-10-29, the operator opened **Type of host** in a local desktop Studio with the correction, in Chromium 156 on Linux. Each option was readable in the dark theme and in the light theme. The visual check passed.

### The Access Key ID Shows In Clear Text

The **Access key ID** field of the S3 or R2 form is a text field. Studio keeps this value in the vault (ADR 0046 item 2), but the form shows it.

On 2026-10-29 the operator made a decision about the fields of secret values. A secret value is in a vault, or Studio shows it one time only. The field hides the value until the user clicks an eye control. The SFTP private key gets a masked text area. [ADR 0078](../adr/0078-a-secret-value-stays-hidden-until-the-user-shows-it.md) records the decision. On 2026-10-29 Claude checked the correction in a local desktop Studio with screenshots. Each field and each display of a secret value showed dots, and its eye button showed the value. `$edges()` found no content at the edge of a group.

### Two Track Numbers For Each Track After A Removal

The feed before the test had three tracks with `itunes:episode` 1, 2 and 3, and no `podcast:episode`. Its SHA-256 was `fdc3c605aa1085c73f30cda3d3ad2b0d33f657173013f6266bfae591ee918d94`. The operator removed the first track, "Emily-song". The diff of the Review sheet then showed these values:

| Track | `itunes:episode` | `podcast:episode` |
| --- | --- | --- |
| minor r1 session | 2, no change | 1, added |
| sine-30 | 3, no change | 2, added |

[ADR 0024](../adr/0024-track-numbers-follow-the-track-sequence.md) item 2 sets the position of each track as its `podcast:episode`. ADR 0024 says nothing about `itunes:episode`. Thus the core follows the decision, but the written feed gives two different numbers for one track. An app that reads `itunes:episode` shows a different number from an app that reads `podcast:episode`.

On 2026-10-29 the operator decided that the core writes the two elements. [ADR 0079](../adr/0079-the-core-writes-both-track-numbers.md) records the decision. The core sets the two numbers, because the rule is a feed rule ([ADR 0065](../adr/0065-the-core-applies-each-feed-rule.md)). The test `a_change_of_sequence_sets_the_two_track_numbers` in `crates/core/tests/model.rs` does the removal of this test again.

### The Announcement Before The Write

The first Review sheet showed "Tell Podcast Index and send a podping" for `https://music.cancelbandcamp.com/feed.xml`. The test feed was not in Podcast Index. The operator turned off **Tell Podcast Index after each write** in Settings before the write. The Review sheet then showed "No announcement: the announcements are off in Settings."

### A False Conflict From A Weak ETag

This is a defect. The operator selected **Write to the host**. Studio did not write. It showed "The hosted file changed after the draft started." The file in the bucket did not change: its ETag, its `Last-Modified` and its SHA-256 were the same before and after.

Claude recorded the API response through the DevTools port: `409 /api/v1/feeds/5/write`, `{"error":{"code":"conflict","message":"feed.xml changed on the host"}}`. The S3 connector gives this message when `PutObject` gives 412. Thus the pipeline check of the live version passed, and R2 refused the conditional write.

Claude then did two checks with a second token, `ifmatch-test`, on a new key `.studio-probe-ifmatch-<time>`:

1. A Python script with only the standard library. R2 accepted a correct `If-Match` with and without quotes, signed in the header and signed in the URL. An incorrect `If-Match` gave 412 `PreconditionFailed`.
2. A small Rust tool in `.studio-dev/` that uses the `S3Bucket` of `studio-core`. The write, the read, a correct `If-Match` write and the delete of the probe key all passed. The read of `feed.xml` gave the version `W/"3e26cc06134977a1dcadea8a68773660"`.

The public address gives the same result:

| Request to `feed.xml` | `ETag` |
| --- | --- |
| No `Accept-Encoding` | `"3e26cc06134977a1dcadea8a68773660"` |
| `Accept-Encoding: gzip` | `W/"3e26cc06134977a1dcadea8a68773660"`, with `content-encoding: gzip` |

The cause:

- `studio-core` uses `ureq` 3 with its default features. `ureq` asks for gzip.
- Cloudflare compresses the feed, 3,867 bytes of XML, and gives a weak ETag.
- Studio keeps the weak ETag as the version, and sends it in `If-Match`. `If-Match` uses the strong comparison, so a weak ETag does not agree. R2 gives 412.
- The probe key has 21 bytes. Cloudflare did not compress it, so its ETag stayed strong. The provider test of ADR 0046 item 4 sends only an incorrect `If-Match`, so it did not find the defect.

The fake S3 server of the tests gave no weak ETag, so no test found the defect.

The correction:

- The read and the stat of the S3 connector send `Accept-Encoding: identity`. R2 then gives the strong ETag.
- The fake S3 server gives a weak ETag to a read that accepts gzip. Before the correction, ten S3 tests did not pass with this server, and the trait suite was one of them. Thus the tests find the defect if it occurs again.
- The test `a_read_gives_the_strong_etag_so_a_write_with_it_passes` writes with the version of a read.

The check against R2 with the correction is test 3 on `0.1.0-rc.5`.

## Results

| Test | Date | Result |
| --- | --- | --- |
| 1. Make an R2 API token with access to one bucket | 2026-10-29 | Done |
| 2. Add the bucket as an S3 or R2 host. The provider test passes. | 2026-10-29 | Passed. The host card `heycitizen` shows. The probe deleted its test key, so **Object Read & Write** permits a delete. Studio found `feed.xml` in the bucket as "HeyCitizen's Test Feed". |
| 3. Write a feed to the bucket through Studio | 2026-10-29 | Failed. A false conflict from a weak ETag. See the section "A False Conflict From A Weak ETag". |
| 3. Write a feed to the bucket through Studio, again with the correction | 2026-10-31 | Passed on `0.1.0-rc.5`. |

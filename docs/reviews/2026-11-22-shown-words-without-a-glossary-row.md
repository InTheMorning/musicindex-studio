# Shown Words Without A Glossary Row

- Date: 2026-11-22
- Owner: [ADR 0244](../adr/0244-studio-reads-the-fee-value-of-musicindex-adr-0088.md) item 4
- Source: a ChatGPT search of `crates/api/ui/` against `../musicindex/docs/glossary.md`. The main session examined six rows of the list, and each was correct.

## Result

Studio has no word list of its own, so this work removed no file.

Each row below is a word that Studio shows for feed data, and the glossary has no row for it. The musicindex session adds the rows (musicindex ADR 0087 R-WORD-13). Studio does not add rows in `../musicindex`.

| Shown word | Feed data | First location |
|---|---|---|
| Cover image / Cover image address / Album art | itunes:image/@href; RSS image/url | `crates/api/ui/editor.js:2993` |
| Artwork / More artwork / Images | podcast:image; itunes:image | `crates/api/ui/editor.js:3008` |
| Image address / Track image address | podcast:image/@href; itunes:image/@href | `crates/api/ui/editor.js:3557` |
| RSS image | RSS channel/image | `crates/api/ui/review.js:176` |
| Purpose / artwork / social / canvas / banner / publisher / circular / poster | podcast:image/@purpose | `crates/api/ui/editor.js:3505` |
| Width / Height / Ratio | podcast:image/@width, @height, @aspect-ratio | `crates/api/ui/editor.js:3511` |
| Credit / Credits | podcast:person | `crates/api/ui/editor.js:3708` |
| Name | podcast:person text; podcast:valueRecipient/@name; itunes:author | `crates/api/ui/editor.js:217` |
| Picture / Picture address | podcast:person/@img | `crates/api/ui/editor.js:220` |
| Information address | podcast:person/@href | `crates/api/ui/editor.js:221` |
| Group | podcast:person/@group | `crates/api/ui/app.js:1413` |
| Nostr / Nostr key | podcast:person/@npub | `crates/api/ui/editor.js:326` |
| Nostr profile | podcast:txt with purpose="npub" | `crates/api/ui/editor.js:3021` |
| Author / Artist name | itunes:author | `crates/api/ui/addfeeds.js:148` |
| Website / Link | RSS channel/link | `crates/api/ui/editor.js:3035` |
| Support links | podcast:funding | `crates/api/ui/editor.js:3587` |
| Text | podcast:funding text | `crates/api/ui/editor.js:287` |
| Address | podcast:funding/@url; podcast:valueRecipient/@address | `crates/api/ui/editor.js:288` |
| License | podcast:license | `crates/api/ui/editor.js:374` |
| License text address | podcast:license/@url | `crates/api/ui/editor.js:375` |
| Audio file / Audio file address / Recording | RSS enclosure/@url | `crates/api/ui/editor.js:424` |
| Size / Audio file size in bytes | RSS enclosure/@length; podcast:alternateEnclosure/@length | `crates/api/ui/editor.js:428` |
| Type / File type / Audio file type | RSS enclosure/@type; podcast:alternateEnclosure/@type; podcast:image/@type; podcast:transcript/@type; podcast:valueRecipient/@type | `crates/api/ui/review.js:171` |
| Duration | itunes:duration | `crates/api/ui/audio.js:64` |
| Alternate files | podcast:alternateEnclosure | `crates/api/ui/review.js:164` |
| Bitrate | podcast:alternateEnclosure/@bitrate | `crates/api/ui/review.js:164` |
| Default | podcast:alternateEnclosure/@default | `crates/api/ui/review.js:165` |
| Sources | podcast:source | `crates/api/ui/review.js:176` |
| Relation | podcast:alternateEnclosure/@rel; podcast:remoteItem/@rel; podcast:transcript/@rel | `crates/api/ui/review.js:174` |
| Publication date | RSS pubDate | `crates/api/ui/editor.js:3737` |
| Explicit | itunes:explicit | `crates/api/ui/editor.js:3057` |
| Language | RSS language; podcast:transcript/@language | `crates/api/ui/editor.js:3113` |
| Owner | itunes:owner | `crates/api/ui/editor.js:3116` |
| Owner name | itunes:owner/itunes:name | `crates/api/ui/editor.js:3118` |
| Owner email / Email | itunes:owner/itunes:email | `crates/api/ui/editor.js:3119` |
| Keywords | itunes:keywords | `crates/api/ui/editor.js:3124` |
| Lock against import / Locked | podcast:locked | `crates/api/ui/editor.js:3128` |
| Lock owner email | podcast:locked/@owner | `crates/api/ui/editor.js:3138` |
| Managing editor | RSS managingEditor | `crates/api/ui/editor.js:3157` |
| Webmaster | RSS webMaster | `crates/api/ui/editor.js:3158` |
| Hide from apps | itunes:block; podcast:block | `crates/api/ui/editor.js:3180` |
| podcast:block tags | podcast:block | `crates/api/ui/review.js:165` |
| itunes:block tag | itunes:block | `crates/api/ui/review.js:168` |
| OP3 statistics / OP3 | RSS enclosure/@url, through its OP3 prefix | `crates/api/ui/editor.js:3141` |
| Medium | podcast:medium; podcast:remoteItem/@medium | `crates/api/ui/review.js:171` |
| New feed address | itunes:new-feed-url | `crates/api/ui/editor.js:2598` |
| Build date | RSS lastBuildDate | `crates/api/ui/review.js:179` |
| Podroll | podcast:podroll | `crates/api/ui/review.js:173` |
| Remote items | podcast:remoteItem | `crates/api/ui/review.js:175` |
| Text channels | podcast:txt | `crates/api/ui/review.js:178` |
| Disc / Disc number | podcast:season text | `crates/api/ui/editor.js:3755` |
| Disc name | podcast:season/@name | `crates/api/ui/editor.js:3756` |
| Captions | podcast:transcript/@rel="captions" | `crates/api/ui/editor.js:3633` |
| Custom key (optional) | podcast:valueRecipient/@customKey | `crates/api/ui/values.js:341` |
| Account (optional) | podcast:valueRecipient/@customValue | `crates/api/ui/values.js:342` |
| Method | podcast:value/@method | `crates/api/ui/review.js:171` |
| Value type | podcast:value/@type | `crates/api/ui/review.js:178` |
| Suggested | podcast:value/@suggested | `crates/api/ui/review.js:177` |
| Remote start time | podcast:valueTimeSplit/@remoteStartTime | `crates/api/ui/review.js:175` |
| Status | podcast:liveItem/@status | `crates/api/ui/editor.js:4022` |
| Start / Start time | podcast:liveItem/@start; podcast:valueTimeSplit/@startTime | `crates/api/ui/review.js:176` |
| End / End time | podcast:liveItem/@end | `crates/api/ui/editor.js:4050` |
| Stream / Stream address | RSS enclosure/@url within podcast:liveItem | `crates/api/ui/editor.js:4060` |
| Stream type | RSS enclosure/@type within podcast:liveItem | `crates/api/ui/editor.js:4063` |
| Content links | podcast:contentLink | `crates/api/ui/editor.js:4103` |
| Link address | podcast:contentLink/@href | `crates/api/ui/editor.js:3845` |
| Link text | podcast:contentLink text | `crates/api/ui/editor.js:3846` |
| Live value address | podcast:liveValue/@uri | `crates/api/ui/editor.js:4112` |
| Live value protocol | podcast:liveValue/@protocol | `crates/api/ui/editor.js:4113` |
| Show GUID | RSS guid within podcast:liveItem | `crates/api/ui/editor.js:4128` |
| Feed file / Feed files | RSS document; no dedicated element or attribute | `crates/api/ui/app.js:1826` |
| Public address | RSS document URL; no dedicated RSS element or attribute | `crates/api/ui/editor.js:2306` |
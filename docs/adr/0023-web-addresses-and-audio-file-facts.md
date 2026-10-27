# ADR 0023: Web Addresses And The Facts Of An Audio File

- Status: Accepted
- Date: 2026-10-22
- Owner: operator
- Amended: 2026-10-27 by operator decision. [ADR 0060](0060-the-album-cover-keeps-the-form-of-its-feed.md) item 1 adds the `url` of the channel RSS `<image>` to the address fields of item 1.

## Context

On 2026-10-22 the operator did the first visual check of the album editor ([task 011](../tasks/studio-v1-task-011-album-editor.md)). The draft got `url="none"`, `href="www.example.com"` and `url="www.google.com/example"`. Studio saved them with no warning. A listener app cannot open these addresses.

In the same check, the operator added a track with only its audio address. The new `enclosure` had no `length` and no `type`. RSS 2.0 requires the two attributes, and apps and indexes can refuse an enclosure without them.

Studio does not host files ([ADR 0009](0009-studio-connects-to-hosts-and-moves-files.md)). The host of the audio file knows its size and its media type.

## Decision

### Web Addresses

1. These model fields are address fields:
   - the channel `link`, the `href` of `itunes:image`, the `href` of each `podcast:image`, and the `url` of `podcast:license`.
   - the `img` and the `href` of each `podcast:person`, and the `url` of each `podcast:funding`, in the channel and in each item.
   - for each item: the `url` of `enclosure`, the `href` of `itunes:image`, the `href` of each `podcast:image`, the `url` of each `podcast:transcript`, and the `url` of `podcast:license`.
2. An address is correct when it starts with `http://` or `https://`, has a host name, and contains no space. An empty field is no address.
3. The core refuses a write when an address field of the draft is not correct. This includes a field that the user did not change. The error names each field.
4. A draft with an incorrect address saves. The API gives each incorrect field with the draft. The UI shows red text below the field, with the `--error` token ([ADR 0018](0018-passphrase-strength-and-error-color.md)), and does not offer the write.

### The Facts Of An Audio File

5. When the user adds a track, Studio asks the host of the audio file for its size and its media type. Studio sends a `HEAD` request and reads `Content-Length` and `Content-Type`. When the response has no length, Studio sends a `GET` with `Range: bytes=0-0` and reads the total size from `Content-Range`. Studio follows redirects and does not read the full file.
6. The add form shows the size and the type that Studio found. The user can change them. When the host does not give them, the user types them.
7. The add form adds a track only with an address, a size in bytes and a media type.

## Evidence

On 2026-10-22 the agent examined the address fields of the 25 feeds in the test corpus. Of 1263 addresses, 2 were not correct. The two are the channel `link` of one MSP 2.0 publisher feed: `strangeloveofmusic.wixsite.com/strangelove`, with no scheme. With item 3, Studio cannot write that feed until the user corrects its link.

## Alternatives Considered

- Warn and do not stop the write. Rejected by the operator: an incorrect address reaches listeners when the user does not see the warning.
- Examine only the fields that the user changed. Rejected: the rule then depends on the edit history, and an incorrect address in the live file stays.
- The user types the size and the type of each new track. Rejected by the operator: the host knows them, and a typed size can be incorrect.
- Add a track with only its address, and show a warning. Rejected: RSS 2.0 requires the two attributes.

## Consequences

- A feed with an incorrect address in the live file needs a correction before its first write through Studio.
- The server shell must refuse audio addresses in private networks, as it must for discovery (the Server Shell phase).
- Some hosts do not answer a `HEAD` request or a range request. The user then types the size and the type.

## Invariants

- No write occurs when an address field of the draft is not a correct `http` or `https` address.
- A track that the editor adds has an enclosure with a `url`, a `length` and a `type`.

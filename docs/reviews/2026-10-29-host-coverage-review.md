# Host Coverage Review, 2026-10-29

This record compares the hosts of the published music feeds with the connectors of Studio. It states no rule. [ADR 0006](../adr/0006-connectors-and-the-v1-set.md) and [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md) own the connector set.

## Why

On 2026-10-29 the operator asked for a new rclone connector. Before a design, the operator asked Claude to examine the host coverage of Studio, and to find the hosts that Studio cannot write.

## Method

1. Claude opened the Stophammer database copy `stophammer-20261025T230927Z.db` read-only. It has 10,583 feeds on 88 host names.
2. Claude counted the feeds by the host name of `feeds.feed_url`.
3. For each host name other than `wavlake.com`, Claude sent one `HEAD` request to one feed address of that host on 2026-10-29. Claude read the status, the `Server` header and the provider headers, such as `x-amz-request-id` and `x-github-request-id`.
4. Claude put each host into one type from these facts.

The method has these limits:

- A header names the web server, not the method that the artist uses for an upload. A host of the type "own web server" can accept SFTP, FTP, WebDAV or only a control panel.
- Claude examined one feed address for each host.
- A host behind Cloudflare does not show its source server.
- `epochradio.com` gave `text/html` for a feed address. Its type is not confirmed.

## Result

| Feeds | Share | Host type | Can Studio write it? |
| ---: | ---: | --- | --- |
| 9,291 | 87.8% | Wavlake | No. No write path is known. |
| 415 | 3.9% | Fountain, `feeds.fountain.fm` | No. The write path is not researched. |
| 161 | 1.5% | RSS Blue | No. The write path is not researched. |
| 129 | 1.2% | `epochradio.com` | Unknown |
| 57 | 0.5% | Other platforms: Podhome 20, an app on Vercel 19, MSP 2.0 hosting 5, JustCast 5, Sunami 3, staticsave 3, Podcast Index 3, Squarespace 1, Feedburner 1, Blubrry 1 | No |
| 378 | 3.6% | Own web server: Apache, nginx, Caddy, LiteSpeed and others | Only through SFTP with `posix-rename@openssh.com` ([ADR 0047](../adr/0047-the-sftp-connector.md) item 6) |
| 66 | 0.6% | Cloudflare in front of an unknown source server | Yes if the source server is R2 or S3. If not, unknown. |
| 33 | 0.3% | Cloudflare in front of a source server that gives `x-amz-request-id` | Yes if the source server passes the provider test |
| 25 | 0.2% | S3 API: AWS 13, DigitalOcean Spaces 12 | AWS: yes. Spaces: only if it passes the provider test. |
| 15 | 0.1% | nsite | Yes |
| 9 | 0.1% | GitHub Pages and `raw.githubusercontent.com` | No. The Git pages connector comes after v1 ([ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md) item 10). |

The largest hosts of the type "own web server" are `music.behindthesch3m3s.com` (105), `www.doerfelverse.com` (50), `pc2.basspistol.com` (29) and `mmmusic.show` (21).

## Gaps

1. **Platforms.** Approximately 95% of the feeds are on a platform. A platform makes the feed from its own database, so a file connector cannot write the feed ([baseline research](../research/2026-10-22-studio-baseline-research.md), section "Hosting Platforms"). The write paths of Wavlake, Fountain and RSS Blue are open research items since 2026-10-22.
2. **Shared web hosting.** The type "own web server" is the largest group that a file connector can write to. Many shared hosts give FTP or FTPS and no SFTP. Studio also refuses an SFTP server with no `posix-rename@openssh.com`. The upload method of these artists is not known.
3. **S3-compatible providers.** [ADR 0046](../adr/0046-the-s3-and-r2-connector.md) item 5 refuses a provider that does not refuse an incorrect conditional write. No test examined DigitalOcean Spaces, Backblaze B2 or Wasabi.
4. **Git pages.** Nine feeds. The connector comes after v1.

## Open Items

- Research which S3-compatible providers pass the provider test of ADR 0046 item 4.
- Send a question to the artists of the largest own-server hosts how they upload their files.
- Research FTP, FTPS and WebDAV as write paths. rclone is one possible method. It has no conditional write, so a write with rclone has a window between the check and the write, as SFTP has.
- Research the write paths of Wavlake, Fountain and RSS Blue again.

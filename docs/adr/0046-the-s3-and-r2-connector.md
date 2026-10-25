# ADR 0046: The S3 And R2 Connector

- Status: Accepted
- Date: 2026-10-25
- Owner: operator, with the technical choices of the planning model

## Context

[ADR 0043](0043-phase-9-connectors-and-the-interface-change.md) makes S3 and R2 the first remote connector. [ADR 0045](0045-phase-9-crates-test-servers-and-host-forms.md) selects `rusty-s3` with `ureq` and a fake S3 server for the tests.

The research found these facts:

- AWS and R2 document `If-None-Match: *` and `If-Match` on PutObject. A condition that is not true gives 412.
- AWS documents `If-Match` on DeleteObject and on CompleteMultipartUpload. R2 does not document them. Other providers are unknown.
- An ETag is a correct opaque version. A multipart upload gives a different ETag for the same bytes.
- ListObjectsV2 gives the key, the size and the ETag of each object, in pages of 1000.
- `r2.dev` has a rate limit and is not for production. A public feed needs a custom domain.

On 2026-10-25 the operator selected a test of the provider when the user adds the host. The planning model added the technical choices below.

## Decision

### The Host

1. The connector kind is `s3`. Its collection row has the endpoint address, the region, the bucket, the address style (`path` or `virtual`), a key prefix and the public base address. None of these is a secret.
2. The vault holds the access key ID and the secret access key as one JSON value. `from_row` gets it with `secret_of` ([ADR 0043](0043-phase-9-connectors-and-the-interface-change.md) item 5).
3. A path of the connector is a key below the prefix. The prefix ends with `/`, or it is empty.

### The Provider Test

4. Before Studio adds the host, it tests the provider with a key below the prefix whose first segment starts with `.studio-probe-`:
   1. A PUT with `If-None-Match: *` must pass.
   2. A second PUT with `If-None-Match: *` must give 412.
   3. A PUT with `If-Match` and an ETag that is not correct must give 412.
   4. Then Studio deletes the key.
5. When step 2 or step 3 does not give 412, Studio refuses the host. The message says that the provider does not refuse a conditional write, so Studio cannot write to it safely.
6. A 403 from the provider gives `Unauthorized`. A failed connection gives `Unavailable`.

### The Operations

7. The version is the ETag, with its quotes, as the provider gives it. The stamp is the ETag and the size, as `<etag>|<size>`.
8. `read` is a GetObject. `stat` is a HeadObject. `list` is ListObjectsV2 with no delimiter, page after page, with the limits of [ADR 0019](0019-select-a-feed-from-the-host.md). The list leaves out each key with a segment that starts with `.`, and each key that is deeper than `LIST_MAX_DEPTH`.
9. `write` and `write_from` send `If-None-Match: *` for `Expected::Absent`, and `If-Match` with the version for `Expected::Version`. A 412 gives `Conflict`.
10. A file of 64 MiB or less is one PutObject. A larger file is a multipart upload with parts of 16 MiB. The condition is on CompleteMultipartUpload. When the upload fails, the connector aborts it.
11. Each write sets `Content-Type` from the extension of the path, with the media types of [ADR 0023](0023-web-addresses-and-audio-file-facts.md). A feed XML gets `application/rss+xml`.
12. `delete` does a HeadObject and compares the stamp. Then it sends DeleteObject with `If-Match`. A provider that ignores `If-Match` on a delete thus has a short window between the check and the delete, as the local folder has ([ADR 0015](0015-connector-interface-and-write-pipeline.md), Consequences).
13. `public_url` is the public base address and the path. A host with no public base address gives no public address.
14. `sha256` gives no value. `can_redirect` gives false.

## Alternatives Considered

- The AWS SDK or `object_store`. Rejected: each one needs an async runtime ([ADR 0015](0015-connector-interface-and-write-pipeline.md) item 1).
- No provider test. Not chosen by the operator: a provider that ignores the conditions can lose a concurrent change with no error.
- A SHA-256 in the object metadata. Rejected for v1: `write_from` must then read the file two times. The pipeline already records the SHA-256 of each file that it uploads.
- A per-object redirect through the AWS website endpoint. Deferred: R2 does not document it, and a feed move can use the pointer file of [ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md) item 18.

## Consequences

- A provider that refuses the test cannot be a host. Some S3 providers are thus not available, and the message tells the user why.
- The same bytes in a multipart upload and in a single PUT give different ETags. The inventory uses the SHA-256 for content identity, so this has no effect on the library ([ADR 0036](0036-library-records-host-inventory-and-host-operations.md) item 6).
- The R2 conditional delete and multipart upload stay open research items. A human test with a real bucket checks them.

## Invariants

- The connector writes only with a condition on the host.
- No access key is in the collection row, a log or an API response.

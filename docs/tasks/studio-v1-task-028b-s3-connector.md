# Studio v1 Task 028b: The S3 Connector

Parent: [task 028](studio-v1-task-028-s3-connector.md). Decisions: [ADR 0046](../adr/0046-the-s3-and-r2-connector.md) items 1 to 3 and 7 to 14, [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) item 1 and [ADR 0043](../adr/0043-phase-9-connectors-and-the-interface-change.md).
Sequence: the second packet of task 028. It depends on 028a.

## Goal

`studio-core` has the connector kind `s3`. It reads, writes, lists and deletes the objects of one bucket with the conditions of ADR 0046. It passes the test module of the trait against `FakeS3`.

## Files

Inspect: `crates/core/src/connector.rs` (the trait, `from_row`, `secret_of`), `crates/core/src/connector/local.rs` (the form of a connector), `crates/core/src/connector/suite.rs`, `crates/core/src/testing/fake_s3.rs`, `crates/core/src/web.rs` (the `ureq` agent and `url_path`). Also inspect `crates/core/src/media.rs` or the code that gives a media type from an extension (ADR 0023), `crates/core/Cargo.toml`, the root `Cargo.toml`.

Change: add `crates/core/src/connector/s3.rs`. Change `connector.rs` (the module, `from_row`), `crates/core/Cargo.toml` and the root `Cargo.toml` (the `rusty-s3` dependency).

Do not change: the pipeline, the API, the UI, `LocalFolder`, the suite.

## Architecture

These choices are fixed. Do not change them.

- Add `rusty-s3` 0.10 to the workspace dependencies and to `studio-core`. Enable only the features that the signing needs. `ureq` 3 sends each request.
- `pub struct S3Bucket` has `pub const KIND: &'static str = "s3"`. Its configuration JSON has `endpoint`, `region`, `bucket`, `url_style` (`"path"` or `"virtual"`), `prefix` and `base_url` (optional). `S3Bucket::new(config, credentials)` and `config()` follow the form of `LocalFolder`.
- The secret is JSON: `{"access_key_id": "…", "secret_access_key": "…"}`. `from_row` gives it from `secret_of`. A row of kind `s3` with no secret gives `InvalidConfig`. The `Debug` form of `S3Bucket` does not show the secret access key.
- `prefix` is empty or ends with `/`. The key of a path is the prefix and the path. The path check of `relative_path` applies.
- Each request is a presigned URL from `rusty-s3`, valid for one hour, with the extra headers in the signed headers. Set the `Content-Type` and the conditions on the action with `headers_mut`.
- The status mapping:

  | Status | Result |
  | --- | --- |
  | 2xx | the operation result |
  | 404 on GET and HEAD | `Ok(None)` |
  | 412 | `Conflict` |
  | 401 and 403 | `Unauthorized` |
  | a failed connection or a timeout | `Unavailable` |
  | any other status | `Io`, with the status and the S3 error code in the message |

- `read`: GetObject. The version is the `ETag` header, unchanged.
- `stat`: HeadObject. The size is `Content-Length`. The stamp is `<etag>|<size>`.
- `list`: ListObjectsV2 with `prefix` and no delimiter. It follows `NextContinuationToken`. It removes the prefix from each key. It leaves out a key with a segment that starts with `.`, and a key with more than `LIST_MAX_DEPTH` segments. It stops at `LIST_MAX_FILES` files and then sets `truncated`.
- Parse the list XML with `quick-xml`.
- `write`: one PutObject with `If-None-Match: *` for `Expected::Absent`, or `If-Match: <version>` for `Expected::Version`. The version is the `ETag` of the response.
- `write_from`: when `length` is at most `single_max`, one PutObject with the body from the reader. When it is larger, a multipart upload: CreateMultipartUpload, then UploadPart for each part of `part_size` bytes, then CompleteMultipartUpload with the condition. When a step fails, the connector sends AbortMultipartUpload and gives the error. A reader with fewer bytes than `length` gives `Io`, and no object is at the key.
- `single_max` is 64 MiB and `part_size` is 16 MiB. A `pub(crate)` function `with_limits(single_max, part_size)` changes them for a test.
- Each write sets `Content-Type` from the extension of the path, with the media types of ADR 0023. A path that ends with `.xml` gets `application/rss+xml`. An extension with no known type gets `application/octet-stream`.
- `delete`: `stat`, then a compare of the stamp, then DeleteObject with `If-Match: <etag>`. `NotFound` and `Conflict` as in the trait.
- `public_url`: `base_url` and the path, with each segment encoded as `web::url_path` does. No `base_url` gives `None`.
- `sha256` and `can_redirect` use the trait defaults.
- `from_row` makes an `S3Bucket` for the kind `s3`.

## Steps

1. Add the dependency and the configuration, with a test of `config()` and `from_row`.
2. Add `read`, `stat`, `write` and `delete`, with tests against `FakeS3`.
3. Add `list`, with tests of the prefix, the hidden keys, the depth and the pages.
4. Add `write_from` with the multipart upload, with tests that use `with_limits`.
5. Add one test for each check of the suite against `FakeS3`, as `connector/local.rs` does.
6. Add the tests of the status mapping: 403 with `refuse_all`, and `Unavailable` for an endpoint with no server.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- One new crate only: `rusty-s3`.
- `write_from` reads the reader in parts. It does not read the full file into memory.
- No test sends a request to a host outside the computer.

## Acceptance Criteria

Mechanical:

- `S3Bucket` passes each check of the suite against `FakeS3`.
- A `write_from` with `with_limits(10, 4)` and 10 bytes is one PutObject. With 11 bytes, it is a multipart upload with 3 parts, and `FakeS3` gives the joined bytes.
- A multipart upload whose CompleteMultipartUpload gives 412 gives `Conflict`, and `FakeS3::open_uploads()` is 0 after it.
- A write with `Expected::Absent` on a key that exists gives `Conflict`, also with multipart.
- The list leaves out `.hidden/a.xml` and `a/.b/c.xml`, removes the prefix, and gives two pages as one list.
- `refuse_all(true)` gives `Unauthorized`. An endpoint with no server gives `Unavailable`.
- A feed XML gets `application/rss+xml`, and an MP3 gets `audio/mpeg`, on `FakeS3`.
- The `Debug` text of an `S3Bucket` does not contain the secret access key.
- The three commit checks pass.

Visual:

- None in this packet.

## Test Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Escalation

Stop and report in these cases:

- `rusty-s3` cannot sign a request with the extra headers, or it needs a newer Rust than the version of `rust-toolchain.toml`.
- `S3Bucket` fails a check of the suite. Do not change the suite.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-028b-s3-connector.md
- docs/adr/0046-the-s3-and-r2-connector.md
- The files that the Files section names.

Goal:
- studio-core gets the connector kind s3 in crates/core/src/connector/s3.rs, with the conditions of ADR 0046. It passes the trait suite against FakeS3.

Constraints:
- Follow the Architecture section of the task file fully.
- One new crate only: rusty-s3.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The pipeline, the API, the UI, LocalFolder, the suite.
- Do not commit.

Acceptance criteria:
- The mechanical criteria of the task file.

Test commands:
- cargo fmt --all --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace

At the end, report:
1. files changed
2. tests run
3. behavior changed
4. deviations from task
5. unresolved concerns

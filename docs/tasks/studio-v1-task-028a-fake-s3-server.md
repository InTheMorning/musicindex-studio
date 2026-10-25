# Studio v1 Task 028a: The Fake S3 Server

Parent: [task 028](studio-v1-task-028-s3-connector.md). Decisions: [ADR 0045](../adr/0045-phase-9-crates-test-servers-and-host-forms.md) item 3 and [ADR 0046](../adr/0046-the-s3-and-r2-connector.md).
Sequence: the first packet of task 028.

## Goal

The `testing` module of `studio-core` has a small S3 server that runs in the test process. It has only the S3 operations that the connector uses, with the conditions of AWS. The connector of 028b and the API tests of 028c test against it.

## Files

Inspect: `crates/core/src/testing.rs` (`TestServer`, `RangeServer`: the method of a test server), `crates/core/src/lib.rs`, `crates/core/Cargo.toml`.

Change: add `crates/core/src/testing/fake_s3.rs`. Add `pub mod fake_s3;` to `crates/core/src/testing.rs`.

Do not change: the connectors, the pipeline, the API, the UI. Add no crate.

## Architecture

These choices are fixed. Do not change them.

- `pub struct FakeS3` starts with `FakeS3::start(bucket: &str)`. It listens on a free loopback port, in a thread, as `TestServer` does. `url()` gives the endpoint address, for example `http://127.0.0.1:41234`. It stops when it is dropped.
- It accepts path-style addresses only: `/<bucket>/<key>`. A different bucket gives 404 `NoSuchBucket`.
- It ignores the signature, but it requires a query parameter `X-Amz-Signature` or an `Authorization` header. A request with neither gives 403 `AccessDenied`. `FakeS3::refuse_all(true)` makes each request give 403, for the test of `Unauthorized`.
- It keeps each object in memory: the bytes, the `Content-Type` and the ETag. The ETag of a single PUT is the MD5 of the bytes in hexadecimal, with quotes. The ETag of a multipart object is `"<md5 of the part md5s>-<part count>"`. MD5 needs no new crate: write the MD5 function in the test file, about 60 lines, with the RFC 1321 test vectors as tests.
- The operations are these:

  | Request | Behavior |
  | --- | --- |
  | `PUT /<bucket>/<key>` | Stores the object. `If-None-Match: *` gives 412 `PreconditionFailed` when the key exists. `If-Match: <etag>` gives 412 when the ETag is different, and 404 `NoSuchKey` when the key does not exist. Gives 200 with the `ETag` header. |
  | `GET /<bucket>/<key>` | Gives 200 with the bytes, `ETag`, `Content-Type` and `Content-Length`, or 404 `NoSuchKey`. |
  | `HEAD /<bucket>/<key>` | Gives the same headers as GET, with no body, or 404 with no body. |
  | `DELETE /<bucket>/<key>` | `If-Match` gives 412 when the ETag is different. Gives 204. A key that does not exist gives 204. |
  | `GET /<bucket>?list-type=2` | ListObjectsV2 XML with `Key`, `Size` and `ETag` of each object in key sequence. It obeys `prefix`, `max-keys` (default 1000) and `continuation-token`. It gives `IsTruncated` and `NextContinuationToken`. |
  | `POST /<bucket>/<key>?uploads` | CreateMultipartUpload. Gives XML with an `UploadId`. |
  | `PUT /<bucket>/<key>?partNumber=N&uploadId=I` | UploadPart. Stores the part and gives its `ETag`. |
  | `POST /<bucket>/<key>?uploadId=I` | CompleteMultipartUpload. It joins the parts in the sequence of the request XML. It obeys `If-None-Match: *` and `If-Match` as PUT does. Gives XML with the `ETag`. |
  | `DELETE /<bucket>/<key>?uploadId=I` | AbortMultipartUpload. Removes the parts. Gives 204. |

- An error body is S3 error XML: `<Error><Code>…</Code><Message>…</Message></Error>`.
- `FakeS3::object(key) -> Option<Vec<u8>>`, `FakeS3::put_object(key, bytes)` and `FakeS3::open_uploads() -> usize` let a test read and change the state with no request.
- `FakeS3::ignore_delete_condition(true)` makes DELETE ignore `If-Match`, as R2 is possibly. `FakeS3::ignore_put_conditions(true)` makes PUT and CompleteMultipartUpload ignore the conditions, for the provider test of 028c.
- The server reads a body with `Content-Length`. It does not need chunked transfer.

## Steps

1. Add the MD5 function with its test vectors.
2. Add `FakeS3` with the object operations and the list, with tests that send requests with `ureq`.
3. Add the multipart operations, with tests.

## Constraints

- Each code comment is in ASD-STE100 Simplified Technical English, as the comments in the files are.
- No new crate. Use `quick-xml` only when it makes the XML code shorter. A plain `format!` is correct for the response XML.
- The tests of the server send real HTTP requests to it with `ureq`.

## Acceptance Criteria

Mechanical:

- The tests examine each row of the operations table, with each condition that gives 412.
- A multipart object has the ETag form `"<hex>-<count>"`, and GET gives the joined bytes.
- The list gives two pages for 3 objects with `max-keys=2`, and the second page has `IsTruncated` false.
- A request with no signature gives 403. `refuse_all(true)` gives 403 for a signed request.
- The MD5 function gives the RFC 1321 values for `""`, `"a"`, `"abc"` and `"message digest"`.
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

Stop and report when the server needs a new crate.

## Prompt for lower-context coding model

You are implementing one bounded task from a larger plan.

Implement only this task. Do not redesign the architecture.

Read:
- AGENTS.md
- docs/tasks/studio-v1-task-028a-fake-s3-server.md
- docs/adr/0046-the-s3-and-r2-connector.md
- The files that the Files section names.

Goal:
- crates/core/src/testing/fake_s3.rs has FakeS3, an in-process S3 server with the operations table of the task file and the AWS conditions.

Constraints:
- Follow the Architecture section of the task file fully.
- No new crate.
- Write each code comment in ASD-STE100 Simplified Technical English, as the comments in the files do.

Do not touch:
- The connectors, the pipeline, the API, the UI.
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

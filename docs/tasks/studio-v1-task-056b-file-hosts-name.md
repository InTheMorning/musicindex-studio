# Studio v1 Task 056b: The Name "File Hosts"

Parent: [task 056](studio-v1-task-056-file-hosts-and-icons.md). Decision: [ADR 0092](../adr/0092-file-hosts-and-the-destination-icons.md) items 1 to 4.

Status: Complete, with its screenshot visual check. The check had no feed with the role `host`. A Node test examines the two labels.

## Goal

The destination, its page and each UI text that names a connector say "file host". The word "host" alone in a UI text is the role `host`.

## Files

Inspect: each file in `crates/api/ui/`, `crates/api/ui-tests/words.test.mjs` and `docs/user-guide/`. Also inspect the error messages of `crates/api/src/routes.rs` and `crates/core/src/pipeline.rs` that reach the UI.

Change: the UI files with a text that names a connector, and the messages of `crates/api/src/routes.rs` and `crates/core/src/pipeline.rs`. Also change the Node tests that give a changed text, and the `docs/user-guide/` pages that name the destination.

Do not change: a code name, a route, an API field, a test of the role `host`, the ADRs.

## Architecture

These choices are fixed. Do not change them.

1. The label of the `hosts` entry of `leftPanel` is "File hosts". The page title and the bar title are "File hosts". The view name stays `hosts`.
2. The picker source "Hosts" of `picker.js` is "File hosts".
3. Each other UI text that names a connector uses "file host" or "file hosts", in sentence case. Examples:
   - "Add a host first. Studio writes each feed through its host." becomes "Add a file host first. Studio writes each feed through its file host."
   - "Type of host" becomes "Type of file host".
   - "in Hosts" becomes "in File hosts".
   A button label uses title case (musicindex ADR 0023), for example "Add a File Host".
4. A text that names a server name, such as "the host name" of SFTP or "Host key fingerprint", does not change. These are network names, not connectors.
5. A server message that names a connector and reaches the UI uses "file host". About 46 messages in `routes.rs` and `pipeline.rs` contain "host". Change only the messages that name a connector.
6. A new test in `crates/api/ui-tests/destinations.test.mjs`: no destination label of `leftPanel` is the same as a role heading of `shell.js`. The failure message names ADR 0092 item 1 and the fix. Packet 056a moved this test here, because it fails until the change of item 1.
7. A new test in `crates/api/ui-tests/words.test.mjs` finds "host" and "hosts" with no "file" before them in the string literals of `crates/api/ui/*.js`. A list gives each permitted text: the role heading, the role words of `shell.js`, and the network names of item 4. The failure message names ADR 0092 item 2 and the fix.

## Steps

1. Change the UI texts, and the tests that give them.
2. Change the server messages, and the tests that give them.
3. Write the tests of items 6 and 7.
4. Change the user guide pages that name the destination.
5. Take the screenshots of each changed page, and run `$edges()`.
6. Run the four commit checks.

## Constraints

- No change of behavior.
- Each UI text and each message is in ASD-STE100 Simplified Technical English.

## Acceptance Criteria

Mechanical:

- The test of item 6 passes.
- The test of item 7 passes, and it fails on the text "Add a host first".
- A Node test: the `hosts` entry of `leftPanel` has the label "File hosts".
- The four commit checks pass.

Visual, with `tools/drive-ui.mjs`, in the dark theme, in the light theme and at 390 px:

- The File hosts page, the form to add a file host, and the Add feeds page.
- A collection with a feed with the role `host` shows the role heading "Hosts" and the destination "File hosts".
- `$edges()` finds no content at the edge of a group.

## Escalation

- Stop and report when a text names both a connector and the role `host`.

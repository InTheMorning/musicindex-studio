# ADR 0047: The SFTP Connector

- Status: Accepted
- Date: 2026-10-25
- Owner: operator, with the technical choices of the planning model

## Context

[ADR 0045](0045-phase-9-crates-test-servers-and-host-forms.md) selects `ssh2` and the system `sshd` for the tests.

The research found these facts:

- SFTP has no conditional write. A plain rename fails when the target exists.
- OpenSSH has the extensions `posix-rename@openssh.com`, `fsync@openssh.com` and `hardlink@openssh.com`. `ssh2` has no call for `posix-rename`.
- The modification time of SFTP version 3 has a resolution of one second.
- SFTP gives no public address and no redirect.

On 2026-10-25 the operator selected three login methods and a confirmation of the host key at the first connection. The planning model added the technical choices below.

## Decision

### The Host

1. The connector kind is `sftp`. Its collection row has the host name, the port, the user name and the root folder. It also has the public base address, the login method and the SHA-256 fingerprint of the host key.
2. The login methods are these:
   - "Private key": the vault holds the private key, and its passphrase when the key is encrypted.
   - "Password": the vault holds the password.
   - "ssh-agent": Studio uses the agent of the user, and the vault holds no secret. Only the desktop shell offers this method.
3. When the user adds the host, Studio connects and shows the SHA-256 fingerprint of the host key. The host is added only when the user accepts the fingerprint.
4. At each connection, a different host key gives `Unauthorized`, and Studio does not log in. The message gives the new fingerprint. The user must accept the new key in the host settings.

### The SFTP Requests

5. Studio uses `ssh2` for the SSH session, the login and the host key. It opens the `sftp` subsystem on a channel, and it sends the SFTP version 3 requests that the connector needs itself. With its own requests, Studio can use the OpenSSH extensions.
6. At the start of a session, Studio reads the extensions of the server. A server with no `posix-rename@openssh.com` cannot be a write host. Studio refuses the host when the user adds it.

### The Operations

7. The version is the SHA-256 of the content, as for the local folder ([ADR 0015](0015-connector-interface-and-write-pipeline.md) item 4). The stamp is the size and the modification time.
8. A write has this sequence:
   1. Studio reads the current file and calculates its SHA-256. It compares the result with `Expected`, and gives `Conflict` when they are different.
   2. It writes the bytes to a hidden temporary file in the same folder.
   3. It sends `fsync@openssh.com` when the server has it.
   4. For `Expected::Absent`, it makes the target with `hardlink@openssh.com` and then removes the temporary file. Without `hardlink`, it uses a plain rename, which fails when the target exists. For `Expected::Version`, it uses `posix-rename@openssh.com`.
9. The time from step 1 to step 4 is a window in which a different client can change the file. The window is longer than the window of the local folder, because the transfer is on the network. The snapshot of [ADR 0015](0015-connector-interface-and-write-pipeline.md) item 9 keeps the file before the write.
10. `list` reads each folder, with the limits of [ADR 0019](0019-select-a-feed-from-the-host.md). It leaves out hidden names and symbolic links. A write or a delete through a symbolic link gives `InvalidPath`.
11. `delete` compares the stamp from `stat`, and then removes the file.
12. `public_url` is the public base address and the path. `sha256` gives no value. `can_redirect` gives false.

## Alternatives Considered

- A patch of `ssh2` that wraps a libssh2 rename. Rejected: libssh2 has no `posix-rename` call, and a patch of a C binding must be kept.
- Delete and then rename. Rejected: the feed has no file for a short time.
- The `known_hosts` file of the user. Not chosen by the operator: the server shell has no such file.
- A redirect through an Apache `.htaccess` file. Deferred: it works only on some hosts, and a feed move can use the pointer file of [ADR 0038](0038-published-feeds-downloads-folder-scan-and-feed-moves.md) item 18.

## Consequences

- An SFTP server without the OpenSSH extensions cannot be a host.
- Two writes with the same size in one second give the same stamp. The inventory can thus miss a change in that second.
- The SFTP version 3 requests are Studio code, with their own tests against `sshd`.

## Invariants

- Studio does not log in to a host whose key the user did not accept.
- No password, private key or passphrase is in the collection row, a log or an API response.

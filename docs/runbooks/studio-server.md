# Runbook: studio-server

- Date: 2026-10-26
- Owner: operator
- Decisions: [ADR 0055](../adr/0055-the-server-shell-user-folders-sessions-and-the-vault-window.md), [ADR 0056](../adr/0056-server-sign-in-methods-and-vault-wrappers.md), [ADR 0057](../adr/0057-the-private-network-guard-of-the-server.md), [ADR 0058](../adr/0058-grant-vaults-and-connected-apps-on-the-server.md) and [ADR 0059](../adr/0059-uploads-in-parts-and-the-home-screen-app.md)
- Task: [task 037](../tasks/studio-v1-task-037-runbook-and-human-tests.md)

This runbook tells an operator how to run `studio-server` behind a reverse proxy. The decisions above own each rule. This runbook only restates them. The planning model wrote it from the code of commit `485dd2e`.

## 1. Purpose

`studio-server` serves the Studio API and the Studio UI to many users on one public origin. Each user has a data folder, a session and a vault. The server keeps the vault key in memory only while the vault window is open.

There are two uses:

- A public service, such as `studio.musicindex.org`. Use `--signup invite` and give an invite code to each new user.
- A personal server for one artist. Use `--signup invite`, make one invite code, and sign up. Then make no more invite codes.

## 2. Install And Run

### With Docker

The image is `ghcr.io/inthemorning/studio-server:VERSION`. The compose file is `deploy/compose.yml` ([ADR 0066](../adr/0066-the-server-ships-as-a-container-image.md)). The container listens on port 8080. The host publishes it only on `127.0.0.1:8080`, so each request goes through nginx.

GHCR makes a new package private. After the first push of the image, the operator sets the package `studio-server` to public. Or the operator logs in on the VPS with `docker login ghcr.io`.

1. On the VPS, make the folder of the deployment, and copy `deploy/env.example` to `.env` in it. Set `STUDIO_PUBLIC_ORIGIN` in `.env`.
2. On this computer, run `tools/deploy-server.sh --release VERSION`. VERSION has a hyphen, for example `0.1.0-rc.1`, and the local tag `vVERSION` is required. The script copies `compose.yml`, pulls the image, pins it in `.env` and starts the container. Then it waits at most 60 seconds for two facts. The revision label of the container is the commit of the tag, and the log has the line `Music Index Studio server VERSION`.
3. On the VPS, in the folder of the deployment, make the first invite code:

```sh
docker compose exec studio studio-server invite --data-dir /data
```

The script makes no account and no invite code. Its options `--host` and `--dir` give a different VPS and a different folder.

To read the log, run `docker compose logs -f studio` on the VPS.

### Without Docker

Build the binary on a computer with the Rust version of `rust-toolchain.toml`:

```sh
cargo build --release -p studio-server
install -m 0755 target/release/studio-server /usr/local/bin/studio-server
```

Make a system user and a data folder. Only that user can read the folder:

```sh
useradd --system --home-dir /var/lib/studio --shell /usr/sbin/nologin studio
install -d -m 0700 -o studio -g studio /var/lib/studio
```

Put this unit in `/etc/systemd/system/studio-server.service`:

```ini
[Unit]
Description=Music Index Studio server
After=network-online.target
Wants=network-online.target

[Service]
User=studio
Group=studio
ExecStart=/usr/local/bin/studio-server serve \
  --data-dir /var/lib/studio \
  --listen 127.0.0.1:8080 \
  --public-origin https://studio.musicindex.org \
  --client-address-header X-Real-IP
Restart=on-failure
LimitCORE=0
NoNewPrivileges=true
ProtectSystem=strict
ReadWritePaths=/var/lib/studio
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

Start the server:

```sh
systemctl daemon-reload
systemctl enable --now studio-server
journalctl -u studio-server -f
```

At the start, the server prints its version, the data folder, the listen address and the public origin.

## 3. The Reverse Proxy

The proxy ends TLS and sends each request to the listen address. These rules apply:

- The public origin must be `https`. Only a loopback origin, such as `http://127.0.0.1:8080`, can use `http`.
- The proxy must send the `Host` header of the public origin. The server refuses a different `Host` with 403 `host_not_allowed`.
- The server refuses a change (a method other than `GET` and `HEAD`) with an `Origin` that is not the public origin, with 403 `origin_not_allowed`. Paths that start with `/app/v1/` do not use this rule, because a connected app is not a browser.
- Set a body limit of a minimum of 9 MiB. A part of an upload is 8 MiB (ADR 0059). The nginx default is 1 MiB, and with it each upload fails with 413.
- A NIP-46 wait holds one request for at most 25 seconds. Keep the read timeout of the proxy at 60 seconds or more.
- Send the client address in a header, and give its name to `--client-address-header`. The server uses the last item of the header. Without this option, all users behind the proxy share one address in the limits of section 5.

The nginx server block of `studio.musicindex.org` in `../musicindex/nginx-site` sends `Host` and `X-Real-IP`. Add this line to that server block before the deployment:

```nginx
client_max_body_size 9m;
```

## 4. The Options

`studio-server serve` has these options:

| Option | Default | Effect |
| --- | --- | --- |
| `--data-dir PATH` | none, required | The data folder. |
| `--listen ADDR` | `127.0.0.1:8080` | The address of the HTTP listener. |
| `--public-origin URL` | none, required | The origin that the users open. |
| `--signup open\|invite` | `invite` | Sign-up with or with no invite code. |
| `--client-address-header NAME` | none | The header that gives the client address. |
| `--session-idle-days N` | `7` | A session ends after this time with no request. |
| `--session-max-days N` | `30` | A session ends after this time in total. |
| `--vault-idle-minutes N` | `10` | The vault locks after this time with no use of a secret. |
| `--storage-limit-gib N` | `10` | The largest size of one user folder. |
| `--argon2-concurrency N` | `2` | The number of Argon2id derivations at the same time. |
| `--musicindex URL` | the musicindex.org API | A different musicindex API. |
| `--podcastindex-hub URL` | the Podcast Index hub | A different hub for the announcements. |
| `--nip46-relay URL` | `wss://relay.nsec.app` | A relay of the `nostrconnect://` address. Give the option more than one time for more relays. |

`studio-server invite --data-dir PATH` prints one new invite code. This command works also when the server is stopped.

## 5. Sign-Up, Invite Codes And Limits

With `--signup invite`, each new account needs an invite code that nobody used. Make one code for each new user:

```sh
sudo -u studio studio-server invite --data-dir /var/lib/studio
```

A code works one time. `server.sqlite` keeps only its SHA-256. A failed sign-up gives the code back. With `--signup open`, each person who can open the origin can make an account.

A user has five sign-in methods: email with a passphrase, NIP-07, NIP-46, a passkey and the recovery key. A sign-in with the recovery key opens the vault and asks for a new method before the session continues.

The server has these limits:

- 10 failed sign-ins for one account, or 30 for one client address, in one hour. After that, a sign-in gets 429 `too_many_attempts`.
- A wait of 10 seconds for an Argon2id permit. After that, a request gets 503 `busy`.
- 4 pending NIP-46 connections for each client address.
- 60 requests for each connected app in each minute, and 20 unknown app tokens for each client address in each hour.
- The storage limit of each user folder. An upload or a download over the limit gets 507 `storage_full`.

The counters are in memory. A restart clears them.

## 6. Backup And Restore

With Docker, the data folder is the volume `studio-data`, which the container mounts at `/data`. The commands below use the path of the folder. Run them on a copy of the volume, or on its path on the host. Compose puts the project name `studio` before the volume name, so the command is `docker volume inspect studio_studio-data`.

The data folder has this content:

- `server.sqlite`: the accounts, the sessions, the invite codes, the challenges, the passkeys and the index of the app tokens. It holds only the SHA-256 of each token and of each code.
- `users/<folder>/`: one folder for each account. It has `collection.sqlite`, `vault.json`, `grants/`, `library/` and `incoming/`.

The vault files are encrypted, and no file holds a plain secret. But each file in `grants/` opens with the token of a connected app. A copy of the disk together with an app token gives the host secret of an active grant (ADR 0058). Keep the backups as safe as the server.

To make a backup while the server runs, copy each SQLite file with the SQLite backup command, and copy the other files after it:

```sh
cd /var/lib/studio
sqlite3 server.sqlite ".backup '/backup/studio/server.sqlite'"
for db in users/*/collection.sqlite; do
  mkdir -p "/backup/studio/$(dirname "$db")"
  sqlite3 "$db" ".backup '/backup/studio/$db'"
done
rsync -a --exclude '*.sqlite' --exclude '*.sqlite-wal' --exclude '*.sqlite-shm' --exclude 'incoming/' ./ /backup/studio/
```

For a backup with no change during the copy, stop the server, copy the full folder, and start the server. Each vault then locks, and each session stays.

To restore, stop the server, put the folder back with the owner `studio` and the mode `0700`, and start the server.

## 7. Memory

Each Argon2id derivation uses 64 MiB. `--argon2-concurrency 2` thus needs 128 MiB for the derivations. Increase it only when the computer has the memory for it.

The vault key of a user is in memory while the vault window is open. `LimitCORE=0` in the unit stops core dumps, so a crash does not write a key to the disk. Do not enable swap to an unencrypted disk on this computer.

## 8. The Private-Network Guard

The server uses the network policy `Public` (ADR 0057). Studio refuses a connection to a private, loopback, link-local or reserved address that a user gives. This applies to a feed, an image, a redirect, a relay, an S3 endpoint, an SFTP host and a Blossom server. The check uses the resolved address, so a DNS answer cannot go around it. With `Public`, no client uses a proxy from the environment.

A user sees the error `address_not_public` with status 422 and this text: "Studio on a server cannot connect to <host>, because it is not a public address."

Some functions are for the desktop only. The server refuses them with 403 `not_on_server`. These are a folder host, and a library folder other than the folder of the user.

The musicindex API, the Podcast Index hub and the lookup relays are given by the operator, so they do not use this guard.

## 9. Update And Restart

With Docker, run `tools/deploy-server.sh --release VERSION` on the computer that has the tag. The script replaces the container. The volume `studio-data` stays.

Without Docker, build the new binary, install it, and restart the unit:

```sh
cargo build --release -p studio-server
install -m 0755 target/release/studio-server /usr/local/bin/studio-server
systemctl restart studio-server
```

A restart has these effects:

- Each vault locks. The next step that needs a secret asks the user for an unlock.
- Each session stays. The users stay signed in.
- Each NIP-46 connection ends. A user who signs with a NIP-46 signer connects it again.
- A connected app with an active grant continues to work, because its grant vault is on the disk.
- An upload continues from the first missing part.
- The limit counters start at zero.

A new migration of `server.sqlite` or of a collection runs at the start. Make a backup before an update.

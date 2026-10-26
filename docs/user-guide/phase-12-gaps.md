# Phase 12 gaps in the user guide

This page is for maintainers. It is not part of the reader guide, and the reader index does not link it.

Phase 12 adds the server shell: sign-in, sessions, a phone UI, uploads in parts, the home-screen app and grant vaults.
The guide describes the desktop app only.
Each place that Phase 12 changes has a marker in this form, on its own line:

```text
<!-- PHASE-12-GAP: <short description>; source: <ADR or task number> -->
```

## How to fill a gap

1. Run `grep -rn PHASE-12-GAP docs/user-guide` to list the open markers. The command also matches this page. Ignore the matches in this page.
2. Find the marker in the table below. Read the owner decision and task.
3. Confirm each label and step in the code of `crates/api/ui/`. If the code does not do it, do not write it.
4. Write the text in the section named in the table. Write for desktop and server together where the steps are the same.
5. Remove the marker when the gap is filled.
6. Remove the row from this table.
7. Check the page with `python3 "$HOME/.agents/skills/asd-ste100/scripts/ste_lint.py" --check --no-heuristics FILE`.

## Open markers

There are 10 markers.

| File | Section | What to write | Owner |
| --- | --- | --- | --- |
| `README.md` | Start here | Add links to the new server pages (see the planned pages below), after "Set up Studio for the first time". | ADR 0055, task 034, task 035 |
| `start-on-desktop.md` | Top of the page | Say that this page is for the desktop only. Link `sign-in.md` for the server. | ADR 0055 |
| `first-run.md` | Top of the page | Say how a new account is made on the server: sign-up, the invite code, the recovery key offer. Say that the server sets the library folder, so the step "Choose the library folder" does not appear. | ADR 0055 items 3 and 7, ADR 0056 item 4, task 034f |
| `vault.md` | Top of the page | Explain the vault window: a session and an open vault are two states. The server locks the vault after 10 minutes with no use of a secret. Give the unlock methods other than the passphrase: NIP-07, NIP-46 and passkey. Give the recovery key sign-in. | ADR 0055 items 14 to 17, ADR 0056 |
| `hosts.md` | Top of the page | Say that "Folder on this computer" is not available on the server. Say that the server refuses an address in a private network. | ADR 0055 item 24, ADR 0057 |
| `new-release.md` | Top of the page | Say which hosts a new release can use on the server. Say that the server sets the library folder. | ADR 0055 items 3 and 24 |
| `review-and-write.md` | Top of "Write to the host" | Say that Studio asks for an unlock when a write needs a secret and the vault is locked. Say that the review sheet can use a NIP-46 signer when the browser has no NIP-07 signer. | ADR 0055 item 17, ADR 0056 items 17 to 19, task 035d |
| `release-files.md` | Top of the page | Say which actions of the Files page work on the server. Say how downloads and the storage limit (507 `storage_full`) appear. | ADR 0055 item 21, ADR 0057 |
| `connected-apps.md` | Top of the page | Say that apps connect to the public address of the server. | ADR 0058 item 16, task 036b |
| `find-your-way.md` | End of the page | Describe the Account panel (email or npub, sign-out, account deletion) and how to install the app on the home screen of a phone. | task 034f, task 035f |

The Hosts page also gets an Account panel in server mode (task 034f). Describe it in `account.md` and link it from `hosts.md` when you write that page.

## Planned pages

These pages do not exist yet. Make each one when its task is complete. Link each one from `README.md` and from `docs/README.md`.

| Page | Content | Owner |
| --- | --- | --- |
| `sign-in.md` | Sign up with an invite code. Sign in with an email and a passphrase, with NIP-07 or NIP-46, and with a passkey. Sign in with the recovery key. Error messages of the sign-in page. | ADR 0055, ADR 0056, task 034e, task 034f, task 035b, task 035c |
| `account.md` | The Account panel: the sign-in methods, add and remove a method, sign out, delete the account. | ADR 0055 items 9 and 15, ADR 0056 item 3, task 034f |
| `use-on-a-phone.md` | Use Studio from a phone: the one-stack layout, the unlock sheet and a signer on the phone. | ADR 0059, ADR 0056, task 035 |
| `home-screen-app.md` | Add Studio to the home screen of a phone. | ADR 0059 items 8 to 11, task 035f |

When you add a page, also add it to the matching list in `README.md`.
Pages that exist today and that Phase 12 changes are listed in the table of open markers.

## Content that is the same on the desktop and the server

These pages need no marker. They describe steps that are the same in both shells:
`how-studio-works.md`, `add-feeds.md`, `edit-album.md`, `value-routes.md`, `playlists.md`, `publisher-feeds.md`, `live-shows.md` and `announcements.md`.
Check them again when Phase 12 is complete. For example, the server refuses the host type "Folder on this computer", and `add-feeds.md` describes the panel "From a connected folder".

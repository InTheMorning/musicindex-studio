# ADR 0078: A Secret Value Stays Hidden Until The User Shows It

- Status: Accepted
- Date: 2026-10-29
- Owner: operator
- Accepted: 2026-10-29 by the operator.

## Context

- The human test of task 028 on 2026-10-29 added an R2 bucket on `studio.musicindex.org`. The **Access key ID** field showed its value in clear text ([human test record](../reviews/2026-10-29-task-028-human-test.md), section "The Access Key ID Shows In Clear Text"). Studio keeps that value in the vault ([ADR 0046](0046-the-s3-and-r2-connector.md) item 2).
- On that date, the UI had these fields and values of secrets:
  - Masked, with no control to show the value: each passphrase of the vault, the S3 secret access key and the SFTP password. The passphrase of an SFTP key and the private key of an nsite or Blossom host were also masked.
  - Shown in clear text: the S3 access key ID, the SFTP private key and the token of a new connected app. The recovery key at sign-in and the recovery key to write down were also in clear text.
- The SFTP private key is a `textarea`, because a key file has more than one line. A password field cannot hold line breaks.
- MDN browser-compat-data gives these first versions of `-webkit-text-security`: Chrome 1, Firefox 114, Safari 3 and Safari on iOS 1. The property is not on the standard track.
- `api.html` of musicindex.org also has a password field. No shared musicindex rule owns this control.

## Decision

1. A secret value is a value that Studio keeps in a vault, or a value that Studio shows one time only.
2. Each field and each display of a secret value hides the value at the start.
3. Adjacent to the value, an eye button shows the value. The same button then hides it again. The button has the name "Show NAME" or "Hide NAME", and `aria-pressed`.
4. A single-line field is a password field while it hides its value.
5. The SFTP private key stays a `textarea`. While it hides its value, `-webkit-text-security: disc` masks the text.
6. A display of a one-time value, such as the recovery key, shows dots, not the value, while it hides it. The copy button copies the value also when the value is hidden.
7. One UI component makes each field of a secret value. A Node test finds each field of a secret value in the UI that does not use it.

## Alternatives Considered

- Add the eye button only to the fields that were masked on 2026-10-29. Not chosen by the operator.
- A file picker for the SFTP key, with no text field. Not chosen by the operator.
- A shared musicindex ADR for the control. Not chosen by the operator. musicindex.org can use the same control after a decision in its own record.

## Consequences

- A user who writes down the recovery key must select the eye button first.
- A password manager sees a password field for each single-line secret field.
- A browser older than Firefox 114 shows the SFTP private key in clear text.

## Invariants

- No secret value shows on the screen before the user selects its eye button.

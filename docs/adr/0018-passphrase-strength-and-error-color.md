# ADR 0018: Passphrase Strength And The Error Color

- Status: Accepted
- Date: 2026-10-22
- Owner: operator

## Context

[ADR 0013](0013-vault-format-and-cryptography.md) item 7 requires a minimum of 12 characters for a passphrase and sets no other rule.
On 2026-10-22, the operator examined the first-run wizard. The operator rejected advice on how to make a passphrase, because a password manager makes the passphrase.
The operator asked for a strength meter and for standard red text that tells what is incorrect.

NIST SP 800-63B recommends a minimum length, no rules for character classes, a check against common passwords, and a strength meter.
[ADR 0010](0010-studio-uses-the-afterglow-direction.md) says that red means only "live now". That rule applies to the entity colors, but its text does not say so.

## Decision

### Strength

1. This decision replaces the last bullet of ADR 0013 item 7.
2. A passphrase must have a minimum of 12 characters and a zxcvbn score of 3 or more. A score of 3 means that an attacker needs more than 10^8 guesses.
3. Studio sets no rule for character classes.
4. The core makes the check with the `zxcvbn` crate (MIT). The vault refuses a weak passphrase, also when a client does not use the meter.
5. The API gives the score, the zxcvbn warning and the zxcvbn suggestions for a passphrase. The UI shows them in a meter below the passphrase field.
6. The UI asks for the score when the user stops typing for a short time, not for each key. The passphrase goes only to the own API of Studio.
7. The UI gives no advice on how to make a passphrase.

### Error Color

8. This decision narrows the ADR 0010 rule "Red means only live now". The rule applies to the entity colors (`--c-*`).
9. An error has its own token, `--error`. Its value is a red that is different from `--c-live`, with a contrast of 4.5:1 or more on the surfaces of each theme.
10. A field error is a line of `--error` text directly below its field. The field gets `aria-invalid` and refers to the text with `aria-describedby`. An error is not a box that looks like a field.

## Alternatives Considered

- The zxcvbn JavaScript library in the browser. Rejected: with its word lists it is approximately 800 KB, and a check in the browser only gives advice.
- A meter that only gives advice. Rejected by the operator.

## Consequences

- The vault and the server use the same rule.
- A long passphrase of common words can fail the check. A passphrase from a password manager passes.

## Invariants

- The vault refuses a passphrase with fewer than 12 characters or a zxcvbn score below 3.
- No entity color shows an error.

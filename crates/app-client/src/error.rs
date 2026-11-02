//! The errors of the client (ADR 0110 item 9).

use std::fmt;

/// An error of the client. No text of an error holds a pairing text, a signed
/// event or a key (ADR 0110 item 29).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The pairing text is not correct or expired, or the user selected Done
    /// before the pairing (`app_code_invalid`).
    CodeInvalid,
    /// A program used the pairing text in the last 15 minutes
    /// (`app_code_used`). A different program possibly used it.
    CodeUsed,
    /// The clock differs from the clock of Studio by more than 5 minutes.
    /// The difference is in seconds. A positive value means that Studio is
    /// ahead of the computer.
    ClockWrong(i64),
    /// Studio does not know the key, also after a clock correction
    /// (`app_signature_invalid`).
    SignatureInvalid,
    /// The app uses a token from before ADR 0110 (`app_key_required`).
    KeyRequired,
    /// A different account uses this key (`app_key_in_use`).
    KeyInUse,
    /// The live value is not on the live value origin, or its protocol is not
    /// permitted (`live_value_invalid`).
    LiveValueInvalid,
    /// The grant does not name this app, or the grant ended (404).
    NotGranted,
    /// The grant vault key is not correct, or the host needs a secret that the
    /// grant vault does not hold (`vault_locked`).
    VaultLocked,
    /// More than 60 requests in one minute (`too_many_attempts`).
    TooManyAttempts,
    /// Studio did not answer, or the answer is not correct.
    Network(String),
    /// The text is not a pairing text.
    PairingTextInvalid(&'static str),
    /// The Studio origin is not `https` and not a loopback address.
    OriginInvalid(&'static str),
    /// The signer failed.
    Signer(String),
    /// Studio gave an error that this crate does not know.
    Other {
        /// The HTTP status.
        status: u16,
        /// The error code of Studio, or an empty text.
        code: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CodeInvalid => f.write_str(
                "the pairing text is not correct or it expired: ask the user for a new pairing text",
            ),
            Self::CodeUsed => f.write_str(
                "a program used this pairing text: a different program possibly used it, so the user must remove the app in Studio and make a new pairing text",
            ),
            Self::ClockWrong(seconds) => write!(
                f,
                "the clock of this computer differs from the clock of Studio by {seconds} seconds: turn on the automatic time of the computer"
            ),
            Self::SignatureInvalid => f.write_str(
                "Studio does not accept the key of this app: ask the user to connect the app again",
            ),
            Self::KeyRequired => f.write_str(
                "Studio needs an app key and not a token: ask the user to connect the app again",
            ),
            Self::KeyInUse => f.write_str(
                "a different account uses this key: make a new key and pair again",
            ),
            Self::LiveValueInvalid => f.write_str(
                "Studio refuses the live value: it is not on the live value origin or its protocol is not permitted",
            ),
            Self::NotGranted => f.write_str(
                "the grant does not name this app or the grant ended: stop the writes for this show",
            ),
            Self::VaultLocked => f.write_str(
                "the grant vault key is not correct or the grant vault lacks a secret that the host needs: stop the writes for this show",
            ),
            Self::TooManyAttempts => {
                f.write_str("too many requests: wait one minute and send the request again")
            }
            Self::Network(why) => write!(f, "Studio did not answer correctly: {why}"),
            Self::PairingTextInvalid(why) => write!(f, "the pairing text is not correct: {why}"),
            Self::OriginInvalid(why) => write!(f, "the Studio origin is not accepted: {why}"),
            Self::Signer(why) => write!(f, "the signer failed: {why}"),
            Self::Other { status, code } => {
                write!(f, "Studio refused the request with status {status} {code}")
            }
        }
    }
}

impl std::error::Error for Error {}

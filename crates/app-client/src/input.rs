//! Read a pairing text from a prompt or from the clipboard (ADR 0110 items
//! 33 and 34).
//!
//! These two ways are the only ways. The crate has no function that takes a
//! pairing text from a command argument or an environment variable, because a
//! shell history and a process list keep them. The functions show no part of
//! the text.

use std::fmt;

use zeroize::Zeroizing;

/// An error of the `input` module. Its text holds no part of the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum InputError {
    /// Standard input is not a terminal, or the terminal cannot hide the
    /// text.
    NoTerminal,
    /// The read failed.
    Read,
    /// The clipboard is not available, or it holds no pairing text.
    Clipboard,
    /// The platform has no support for this function.
    Unsupported,
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NoTerminal => "standard input is not a terminal that can hide the text",
            Self::Read => "the text cannot be read",
            Self::Clipboard => "the clipboard is not available or holds no pairing text",
            Self::Unsupported => "this platform does not support this function",
        })
    }
}

impl std::error::Error for InputError {}

/// Read a pairing text from the terminal. The terminal does not show the
/// typed or pasted text. The caller prints the question first.
///
/// ```no_run
/// use studio_app_client::{PairingText, input};
///
/// eprint!("Pairing text: ");
/// let text = input::prompt()?;
/// let pairing = PairingText::parse_with_origin(&text, "https://studio.example.com")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Gives [`InputError::NoTerminal`] when standard input is not a terminal.
pub fn prompt() -> Result<Zeroizing<String>, InputError> {
    prompt_impl()
}

/// Restores the terminal at each exit, also after a panic.
#[cfg(unix)]
struct Restore(rustix::termios::Termios);

#[cfg(unix)]
impl Drop for Restore {
    fn drop(&mut self) {
        let _ = rustix::termios::tcsetattr(
            std::io::stdin(),
            rustix::termios::OptionalActions::Flush,
            &self.0,
        );
    }
}

#[cfg(unix)]
fn prompt_impl() -> Result<Zeroizing<String>, InputError> {
    use std::io::BufRead as _;

    use rustix::termios::{LocalModes, OptionalActions, tcgetattr, tcsetattr};

    let stdin = std::io::stdin();
    let saved = tcgetattr(&stdin).map_err(|_| InputError::NoTerminal)?;
    let mut hidden = saved.clone();
    hidden.local_modes.remove(LocalModes::ECHO);
    // The ECHONL bit is kept, so that the line end shows after the read.
    hidden.local_modes.insert(LocalModes::ECHONL);
    tcsetattr(&stdin, OptionalActions::Flush, &hidden).map_err(|_| InputError::NoTerminal)?;
    let _restore = Restore(saved);
    let mut line = Zeroizing::new(String::new());
    stdin
        .lock()
        .read_line(&mut line)
        .map_err(|_| InputError::Read)?;
    let trimmed = line.trim_end_matches(['\r', '\n']).len();
    line.truncate(trimmed);
    Ok(line)
}

#[cfg(not(unix))]
fn prompt_impl() -> Result<Zeroizing<String>, InputError> {
    Err(InputError::Unsupported)
}

/// Read a pairing text from the clipboard. After the read, the function
/// clears the clipboard if it still holds the pairing text. A clipboard with
/// other content is not read back and not cleared.
///
/// # Errors
///
/// Gives [`InputError::Clipboard`] when the clipboard is not available or does
/// not hold a pairing text.
#[cfg(feature = "clipboard")]
pub fn from_clipboard() -> Result<Zeroizing<String>, InputError> {
    use crate::PairingText;

    let mut clipboard = arboard::Clipboard::new().map_err(|_| InputError::Clipboard)?;
    let text = Zeroizing::new(clipboard.get_text().map_err(|_| InputError::Clipboard)?);
    // Only a pairing text leaves the clipboard. This origin is a dummy for the
    // code-alone form.
    if PairingText::parse_with_origin(&text, "https://studio.invalid").is_err() {
        return Err(InputError::Clipboard);
    }
    let still = clipboard.get_text().is_ok_and(|now| now == *text);
    if still {
        let _ = clipboard.clear();
    }
    Ok(text)
}

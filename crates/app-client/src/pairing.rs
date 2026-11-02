//! The pairing text and the rules for origins (ADR 0110 items 6, 9, 34 and
//! 35).

use std::fmt;
use std::net::Ipv4Addr;

use zeroize::Zeroizing;

use crate::error::Error;

/// The characters of the code, as Crockford base32 gives them.
const ALPHABET: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
/// The number of characters in a code.
const CODE_LENGTH: usize = 8;
/// The text between the origin and the code.
const MARKER: &str = "#pair=";

/// A pairing text that Studio made: the origin of Studio and a code.
///
/// The type shows no part of the code. `Debug` hides the code, and the type
/// has no function that gives it back as text (ADR 0110 item 34).
///
/// ```
/// use studio_app_client::PairingText;
///
/// let text = PairingText::parse("https://studio.example.com#pair=K7M2-Q9XD")?;
/// assert_eq!(text.origin(), "https://studio.example.com");
/// // The code is not case-sensitive and ignores spaces and hyphens.
/// let same = PairingText::parse("https://studio.example.com#pair=k7m2 q9xd")?;
/// assert_eq!(text, same);
/// # Ok::<(), studio_app_client::Error>(())
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct PairingText {
    origin: String,
    code: Zeroizing<String>,
}

impl fmt::Debug for PairingText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PairingText")
            .field("origin", &self.origin)
            .field("code", &"<hidden>")
            .finish()
    }
}

impl PairingText {
    /// Read the full pairing text, for example
    /// `https://studio.musicindex.org#pair=K7M2-Q9XD`.
    ///
    /// # Errors
    ///
    /// Gives [`Error::PairingTextInvalid`] when the text has no origin or no
    /// code, and [`Error::OriginInvalid`] when the origin is not `https` and
    /// not a loopback address (ADR 0110 item 35).
    pub fn parse(text: &str) -> Result<Self, Error> {
        let text = text.trim();
        let Some(at) = text.find(MARKER) else {
            return Err(Error::PairingTextInvalid(
                "the text has no origin or no code",
            ));
        };
        let (origin, code) = (&text[..at], &text[at + MARKER.len()..]);
        if origin.is_empty() {
            return Err(Error::PairingTextInvalid("the text has no origin"));
        }
        Ok(Self {
            origin: studio_origin(origin)?,
            code: normalize_code(code)?,
        })
    }

    /// Read the full pairing text, or the code alone. For the code alone,
    /// the app gives the origin that it saved at the first pairing (ADR 0110
    /// item 34). A full text has its own origin, and `origin` is not used.
    ///
    /// ```
    /// use studio_app_client::PairingText;
    ///
    /// let text = PairingText::parse_with_origin("K7M2-Q9XD", "https://studio.example.com")?;
    /// assert_eq!(text.origin(), "https://studio.example.com");
    /// # Ok::<(), studio_app_client::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The same as [`PairingText::parse`]. A code alone with an origin that
    /// is not allowed gives [`Error::OriginInvalid`].
    pub fn parse_with_origin(text: &str, origin: &str) -> Result<Self, Error> {
        if text.contains(MARKER) {
            return Self::parse(text);
        }
        Ok(Self {
            origin: studio_origin(origin)?,
            code: normalize_code(text)?,
        })
    }

    /// The origin of Studio, for example `https://studio.musicindex.org`.
    /// The app can save it, and later use the code alone.
    #[must_use]
    pub fn origin(&self) -> &str {
        &self.origin
    }

    /// The normalized code. Only this crate uses it.
    pub(crate) fn code(&self) -> &str {
        &self.code
    }
}

/// Apply the rules of ADR 0110 item 9 to the text of a code.
fn normalize_code(text: &str) -> Result<Zeroizing<String>, Error> {
    let mut code = Zeroizing::new(String::with_capacity(CODE_LENGTH));
    for c in text.chars() {
        if c == '-' || c.is_whitespace() {
            continue;
        }
        let c = match c.to_ascii_uppercase() {
            'O' => '0',
            'I' | 'L' => '1',
            other => other,
        };
        if !ALPHABET.contains(c) || code.len() == CODE_LENGTH {
            return Err(Error::PairingTextInvalid("the code is not a code"));
        }
        code.push(c);
    }
    if code.len() == CODE_LENGTH {
        Ok(code)
    } else {
        Err(Error::PairingTextInvalid("the text has no code"))
    }
}

/// The origin of Studio: `https`, or `http` with a loopback host.
pub(crate) fn studio_origin(text: &str) -> Result<String, Error> {
    let (scheme, host) = split_origin(text)?;
    match scheme {
        "https" => {}
        "http" if is_loopback(&host) => {}
        _ => {
            return Err(Error::OriginInvalid(
                "ADR 0110 item 35: Studio origin must use https. Only a loopback address can use http. Use the https address of Studio",
            ));
        }
    }
    Ok(format!("{scheme}://{host}"))
}

/// The live value origin of the app: `https` or `wss` (ADR 0110 item 24).
pub(crate) fn live_value_origin(text: &str) -> Result<String, Error> {
    let (scheme, host) = split_origin(text)?;
    if scheme != "https" && scheme != "wss" {
        return Err(Error::OriginInvalid(
            "ADR 0110 item 24: the live value origin must use https or wss",
        ));
    }
    Ok(format!("{scheme}://{host}"))
}

/// Split an origin into the scheme and the host with its port. The host is
/// in lowercase. A path, a query, a fragment or user data is not allowed.
fn split_origin(text: &str) -> Result<(&str, String), Error> {
    let text = text.trim();
    let Some((scheme, rest)) = text.split_once("://") else {
        return Err(Error::OriginInvalid("the origin has no scheme"));
    };
    let rest = rest.strip_suffix('/').unwrap_or(rest);
    let scheme = match scheme.to_ascii_lowercase().as_str() {
        "https" => "https",
        "http" => "http",
        "wss" => "wss",
        "ws" => "ws",
        _ => return Err(Error::OriginInvalid("the scheme is not known")),
    };
    if rest.is_empty() || rest.contains(['/', '?', '#', '@', ' ', '\\']) {
        return Err(Error::OriginInvalid(
            "the origin must be a scheme, a host and a port only",
        ));
    }
    let (name, port) = if let Some(inner) = rest.strip_prefix('[') {
        let Some((name, after)) = inner.split_once(']') else {
            return Err(Error::OriginInvalid("the host is not correct"));
        };
        (format!("[{name}]"), after.strip_prefix(':'))
    } else if let Some((name, port)) = rest.split_once(':') {
        (name.to_owned(), Some(port))
    } else {
        (rest.to_owned(), None)
    };
    if name.is_empty() || name == "[]" {
        return Err(Error::OriginInvalid("the origin has no host"));
    }
    let host = match port {
        Some(port) if port.parse::<u16>().is_ok() => format!("{}:{port}", name.to_lowercase()),
        Some(_) => return Err(Error::OriginInvalid("the port is not correct")),
        None => name.to_lowercase(),
    };
    Ok((scheme, host))
}

/// Whether a host (with an optional port) is a loopback address.
fn is_loopback(host: &str) -> bool {
    let name = if host.starts_with('[') {
        host.split_once(']').map_or(host, |(name, _)| &name[1..])
    } else {
        host.split(':').next().unwrap_or(host)
    };
    name == "localhost" || name == "::1" || name.parse::<Ipv4Addr>().is_ok_and(|a| a.is_loopback())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_form_of_one_code_is_the_same_code() {
        let base = "https://studio.example.com#pair=";
        let forms = [
            "k7m2 q9xd",
            "K7M2-Q9XD",
            "K7M2Q9XD",
            "k7m2-q9xd",
            " K7M2 Q9XD ",
        ];
        let first = PairingText::parse(&format!("{base}{}", forms[0])).unwrap();
        for form in forms {
            let parsed = PairingText::parse(&format!("{base}{form}")).unwrap();
            assert_eq!(
                parsed, first,
                "ADR 0110 item 9: the code ignores case, spaces and hyphens, and reads O as 0. Fix normalize_code ({form})"
            );
            assert_eq!(parsed.code(), "K7M2Q9XD");
        }
    }

    #[test]
    fn crockford_look_alikes_are_the_same_character() {
        let base = "https://studio.example.com#pair=";
        let zero = PairingText::parse(&format!("{base}K7M2-Q9X0")).unwrap();
        for form in ["K7M2Q9XO", "k7m2 q9xo"] {
            assert_eq!(
                PairingText::parse(&format!("{base}{form}")).unwrap(),
                zero,
                "ADR 0110 item 9: read O as 0. Fix normalize_code"
            );
        }
        let one = PairingText::parse(&format!("{base}1111-1111")).unwrap();
        assert_eq!(
            PairingText::parse(&format!("{base}iIlL 1111")).unwrap(),
            one
        );
    }

    #[test]
    fn a_text_with_no_origin_or_no_code_is_refused() {
        for text in [
            "",
            "K7M2-Q9XD",
            "#pair=K7M2-Q9XD",
            "https://studio.example.com",
            "https://studio.example.com#pair=",
            "https://studio.example.com#pair=K7M2",
            "https://studio.example.com#pair=K7M2-Q9XD1",
            "https://studio.example.com#pair=K7M2-Q9XU",
        ] {
            assert!(
                PairingText::parse(text).is_err(),
                "ADR 0110 items 6 and 34: refuse a pairing text with no origin or no code ({text:?})"
            );
        }
    }

    #[test]
    fn the_code_alone_needs_the_saved_origin() {
        let text =
            PairingText::parse_with_origin("k7m2 q9xd", "https://Studio.Example.com/").unwrap();
        assert_eq!(text.origin(), "https://studio.example.com");
        assert!(PairingText::parse_with_origin("k7m2 q9xd", "").is_err());
    }

    #[test]
    fn http_is_refused_except_on_a_loopback_address() {
        for origin in [
            "http://studio.example.com",
            "http://192.168.1.5:8080",
            "http://localhost.example.com",
            "ftp://studio.example.com",
        ] {
            let text = format!("{origin}#pair=K7M2-Q9XD");
            assert!(
                matches!(PairingText::parse(&text), Err(Error::OriginInvalid(_))),
                "ADR 0110 item 35: refuse a Studio origin that is not https ({origin}). Use https"
            );
        }
        for origin in [
            "http://127.0.0.1:8080",
            "http://localhost:3000",
            "http://[::1]:9",
            "https://studio.example.com:8443",
        ] {
            assert!(
                PairingText::parse(&format!("{origin}#pair=K7M2-Q9XD")).is_ok(),
                "{origin}"
            );
        }
    }

    #[test]
    fn debug_hides_the_code() {
        let text = PairingText::parse("https://s.example.com#pair=K7M2-Q9XD").unwrap();
        let shown = format!("{text:?}");
        assert!(!shown.contains("K7M2") && !shown.contains("Q9XD"));
    }

    #[test]
    fn a_live_value_origin_has_no_path_and_uses_https_or_wss() {
        assert_eq!(
            live_value_origin("wss://live.example.com").unwrap(),
            "wss://live.example.com"
        );
        assert!(live_value_origin("ws://live.example.com").is_err());
        assert!(live_value_origin("https://live.example.com/path").is_err());
    }
}

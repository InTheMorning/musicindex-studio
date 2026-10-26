//! The command line of the server (ADR 0055 items 7, 18 and 19).

use std::net::SocketAddr;
use std::path::PathBuf;

/// How a new user can sign up (ADR 0055 item 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signup {
    /// Each person can sign up.
    Open,
    /// A person needs an invite code.
    Invite,
}

impl Signup {
    /// The name in the options and in the session route.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Invite => "invite",
        }
    }
}

/// The public origin of the server, such as `https://studio.example.org`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin {
    /// The origin as a browser sends it in the `Origin` header.
    pub text: String,
    /// The host and the port, as a browser sends them in the `Host` header.
    pub authority: String,
}

impl Origin {
    /// Read a public origin. The scheme must be `https`. A loopback host
    /// can use `http` (ADR 0055 item 18).
    ///
    /// # Errors
    ///
    /// Gives a message when the text is not an origin that the server
    /// accepts.
    pub fn parse(text: &str) -> Result<Self, String> {
        let text = text.trim_end_matches('/');
        let (scheme, authority) = text
            .split_once("://")
            .ok_or("the public origin needs a scheme")?;
        if authority.is_empty() || authority.contains(['/', '?', '#', '@']) {
            return Err("the public origin must have a host and a port only".to_owned());
        }
        let host = if let Some(rest) = authority.strip_prefix('[') {
            rest.split_once(']').map_or("", |(host, _)| host)
        } else {
            authority.split(':').next().unwrap_or("")
        };
        let loopback = matches!(host, "127.0.0.1" | "localhost" | "::1");
        match scheme {
            "https" => {}
            "http" if loopback => {}
            _ => {
                return Err(
                    "the public origin must use https. Only a loopback origin can use http"
                        .to_owned(),
                );
            }
        }
        Ok(Self {
            text: text.to_ascii_lowercase(),
            authority: authority.to_ascii_lowercase(),
        })
    }
}

/// The options of `studio-server serve`.
#[derive(Debug, Clone)]
pub struct ServeOptions {
    /// The folder of `server.sqlite` and the user folders.
    pub data_dir: PathBuf,
    /// The address to listen on.
    pub listen: SocketAddr,
    /// The public origin.
    pub origin: Origin,
    /// The sign-up mode.
    pub signup: Signup,
    /// The header that gives the client address behind a reverse proxy.
    /// The limit of failed sign-ins uses it in a later packet.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the rate limit of a later packet reads it")
    )]
    pub client_address_header: Option<String>,
    /// A different musicindex API.
    pub musicindex: Option<String>,
    /// A different hub for the announcements.
    pub hub: Option<String>,
}

/// A command of the server.
#[derive(Debug)]
pub enum Command {
    /// Run the server.
    Serve(Box<ServeOptions>),
    /// Print a new invite code.
    Invite {
        /// The folder of `server.sqlite`.
        data_dir: PathBuf,
    },
}

/// Read the command line, without the program name.
///
/// # Errors
///
/// Gives a message for an unknown command or option, or a missing value.
pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut args = args.into_iter();
    let command = args.next().ok_or("give a command: serve or invite")?;
    let mut data_dir = None;
    let mut listen = None;
    let mut origin = None;
    let mut signup = Signup::Invite;
    let mut client_address_header = None;
    let mut musicindex = None;
    let mut hub = None;
    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().ok_or(format!("{name} needs a value"));
        match (command.as_str(), arg.as_str()) {
            (_, "--data-dir") => data_dir = Some(PathBuf::from(value("--data-dir")?)),
            ("serve", "--listen") => {
                listen = Some(
                    value("--listen")?
                        .parse::<SocketAddr>()
                        .map_err(|_| "--listen needs an address, such as 127.0.0.1:8080")?,
                );
            }
            ("serve", "--public-origin") => {
                origin = Some(Origin::parse(&value("--public-origin")?)?);
            }
            ("serve", "--signup") => {
                signup = match value("--signup")?.as_str() {
                    "open" => Signup::Open,
                    "invite" => Signup::Invite,
                    _ => return Err("--signup needs open or invite".to_owned()),
                };
            }
            ("serve", "--client-address-header") => {
                client_address_header = Some(value("--client-address-header")?);
            }
            ("serve", "--musicindex") => musicindex = Some(value("--musicindex")?),
            ("serve", "--podcastindex-hub") => hub = Some(value("--podcastindex-hub")?),
            _ => return Err(format!("unknown option {arg}")),
        }
    }
    let data_dir = data_dir.ok_or("--data-dir is required")?;
    match command.as_str() {
        "serve" => Ok(Command::Serve(Box::new(ServeOptions {
            data_dir,
            listen: listen.unwrap_or(SocketAddr::from(([127, 0, 0, 1], 8080))),
            origin: origin.ok_or("--public-origin is required")?,
            signup,
            client_address_header,
            musicindex,
            hub,
        }))),
        "invite" => Ok(Command::Invite { data_dir }),
        other => Err(format!("unknown command {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(text: &str) -> Vec<String> {
        text.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn the_public_origin_must_use_https_unless_it_is_loopback() {
        assert!(Origin::parse("https://studio.example.org").is_ok());
        assert!(Origin::parse("http://127.0.0.1:8080").is_ok());
        assert!(Origin::parse("http://localhost:3000/").is_ok());
        assert!(Origin::parse("http://studio.example.org").is_err());
        assert!(Origin::parse("https://studio.example.org/path").is_err());
        assert!(Origin::parse("studio.example.org").is_err());
        let origin = Origin::parse("https://Studio.Example.org:8443/").unwrap();
        assert_eq!(origin.text, "https://studio.example.org:8443");
        assert_eq!(origin.authority, "studio.example.org:8443");
    }

    #[test]
    fn serve_has_the_defaults_of_the_task() {
        let command = parse(args(
            "serve --data-dir d --public-origin https://studio.example.org",
        ))
        .unwrap();
        let Command::Serve(options) = command else {
            panic!("expected serve");
        };
        assert_eq!(options.listen.to_string(), "127.0.0.1:8080");
        assert_eq!(options.signup, Signup::Invite);
        assert!(options.client_address_header.is_none());
    }

    #[test]
    fn serve_needs_the_data_folder_and_the_origin() {
        assert!(parse(args("serve --public-origin https://a.example")).is_err());
        assert!(parse(args("serve --data-dir d")).is_err());
        assert!(parse(args("invite")).is_err());
        assert!(parse(args("invite --data-dir d --listen 127.0.0.1:1")).is_err());
        assert!(matches!(
            parse(args("invite --data-dir d")),
            Ok(Command::Invite { .. })
        ));
    }
}

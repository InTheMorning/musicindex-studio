//! The command line of the server (ADR 0055 items 7, 18 and 19).

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

/// The seconds in a day.
const DAY: i64 = 86_400;
/// The bytes in a GiB.
const GIB: u64 = 1024 * 1024 * 1024;

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
    /// The limit of failed sign-ins uses it (ADR 0055 item 23).
    pub client_address_header: Option<String>,
    /// A session ends after this many seconds with no request (ADR 0055
    /// item 13).
    pub session_idle: i64,
    /// A session ends after this many seconds in total.
    pub session_max: i64,
    /// The vault of a user locks after this time with no use of a secret
    /// (ADR 0055 item 16).
    pub vault_idle: Duration,
    /// The most bytes in the folder of a user (ADR 0055 item 21).
    pub storage_limit: u64,
    /// The most Argon2id derivations at the same time (ADR 0055 item 22).
    pub argon2_concurrency: usize,
    /// How long a request waits for a derivation before it gets 503.
    pub argon2_wait: Duration,
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

/// Read a whole number above zero that an option gives.
fn number(name: &str, text: &str) -> Result<u64, String> {
    text.parse::<u64>()
        .ok()
        .filter(|n| (1..=1_000_000).contains(n))
        .ok_or_else(|| format!("{name} needs a whole number from 1 to 1000000"))
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
    let mut session_idle_days = 7;
    let mut session_max_days = 30;
    let mut vault_idle_minutes = 10;
    let mut storage_limit_gib = 10;
    let mut argon2_concurrency = 2;
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
            ("serve", "--session-idle-days") => {
                session_idle_days = number("--session-idle-days", &value("--session-idle-days")?)?;
            }
            ("serve", "--session-max-days") => {
                session_max_days = number("--session-max-days", &value("--session-max-days")?)?;
            }
            ("serve", "--vault-idle-minutes") => {
                vault_idle_minutes =
                    number("--vault-idle-minutes", &value("--vault-idle-minutes")?)?;
            }
            ("serve", "--storage-limit-gib") => {
                storage_limit_gib = number("--storage-limit-gib", &value("--storage-limit-gib")?)?;
            }
            ("serve", "--argon2-concurrency") => {
                argon2_concurrency =
                    number("--argon2-concurrency", &value("--argon2-concurrency")?)?;
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
            session_idle: i64::try_from(session_idle_days).unwrap_or(i64::MAX) * DAY,
            session_max: i64::try_from(session_max_days).unwrap_or(i64::MAX) * DAY,
            vault_idle: Duration::from_secs(vault_idle_minutes * 60),
            storage_limit: storage_limit_gib.saturating_mul(GIB),
            argon2_concurrency: usize::try_from(argon2_concurrency).unwrap_or(usize::MAX),
            argon2_wait: Duration::from_secs(10),
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
        assert_eq!(options.session_idle, 7 * 86_400);
        assert_eq!(options.session_max, 30 * 86_400);
        assert_eq!(options.vault_idle, Duration::from_secs(600));
        assert_eq!(options.storage_limit, 10 * 1024 * 1024 * 1024);
        assert_eq!(options.argon2_concurrency, 2);
        assert_eq!(options.argon2_wait, Duration::from_secs(10));
    }

    #[test]
    fn the_limit_options_take_whole_numbers_above_zero() {
        let line = "serve --data-dir d --public-origin https://a.example";
        let given = format!(
            "{line} --session-idle-days 2 --session-max-days 3 --vault-idle-minutes 1 \
             --storage-limit-gib 4 --argon2-concurrency 1"
        );
        let Ok(Command::Serve(options)) = parse(args(&given)) else {
            panic!("expected serve");
        };
        assert_eq!(options.session_idle, 2 * 86_400);
        assert_eq!(options.vault_idle, Duration::from_secs(60));
        assert_eq!(options.storage_limit, 4 * 1024 * 1024 * 1024);
        assert_eq!(options.argon2_concurrency, 1);
        assert!(parse(args(&format!("{line} --argon2-concurrency 0"))).is_err());
        assert!(parse(args(&format!("{line} --storage-limit-gib x"))).is_err());
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

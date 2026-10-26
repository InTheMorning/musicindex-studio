//! The network policy of the outbound connections (ADR 0057).
//!
//! The desktop shell uses [`NetPolicy::Open`]. The server shell uses
//! [`NetPolicy::Public`]. With `Public`, a connection to an address that
//! a user gives goes only to a global unicast address. The check is on the
//! resolved address. The connection then uses the address that passed the
//! check. Thus a second DNS answer cannot go around it.

use std::fmt;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::{Duration, Instant};

use ureq::unversioned::resolver::{DefaultResolver, ResolvedSocketAddrs, Resolver};
use ureq::unversioned::transport::{DefaultConnector, NextTimeout};

/// The addresses that [`NetPolicy::Public`] permits in addition to each
/// global address.
///
/// The list is empty. Only a test build can add an address.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PublicRules {
    permitted: Vec<SocketAddr>,
}

impl PublicRules {
    /// Rules that permit also the exact socket addresses of a local test
    /// server (ADR 0057 item 7).
    #[cfg(any(test, feature = "testing"))]
    #[must_use]
    pub fn with_test_exceptions(permitted: Vec<SocketAddr>) -> Self {
        Self { permitted }
    }
}

/// The rule for the addresses that Studio connects to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum NetPolicy {
    /// Each address is permitted.
    #[default]
    Open,
    /// Only a global unicast address is permitted.
    Public(PublicRules),
}

impl NetPolicy {
    /// The policy `Public` with no exception.
    #[must_use]
    pub fn public() -> Self {
        Self::Public(PublicRules::default())
    }

    /// True for the policy `Public`.
    #[must_use]
    pub fn is_public(&self) -> bool {
        matches!(self, Self::Public(_))
    }

    /// True when the policy permits a connection to `address`.
    #[must_use]
    pub fn permits(&self, address: SocketAddr) -> bool {
        match self {
            Self::Open => true,
            Self::Public(rules) => is_global(address.ip()) || rules.permitted.contains(&address),
        }
    }
}

/// An error of a guarded connection.
#[derive(Debug)]
pub enum NetError {
    /// The name has no permitted address. `host` is the name that the user
    /// gave, not an address.
    NotPublic {
        /// The host that the user gave.
        host: String,
    },
    /// The name cannot be resolved, or no connection succeeded.
    Io(io::Error),
}

impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotPublic { host } => f.write_str(&not_public_message(host)),
            Self::Io(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for NetError {}

/// The message for a host that is not a public address.
pub(crate) fn not_public_message(host: &str) -> String {
    format!("Studio on a server cannot connect to {host}, because it is not a public address.")
}

/// True when `ip` is a global unicast address (ADR 0057 item 2).
///
/// An IPv4-mapped address and an address in `64:ff9b::/96` use the check
/// of the IPv4 address in them.
#[must_use]
pub fn is_global(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_global_v4(v4),
        IpAddr::V6(v6) => is_global_v6(v6),
    }
}

fn is_global_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    let refused = a == 0
        || a == 10
        || (a == 100 && (64..=127).contains(&b))
        || a == 127
        || (a == 169 && b == 254)
        || (a == 172 && (16..=31).contains(&b))
        || (a == 192 && b == 0 && c == 0)
        || (a == 192 && b == 0 && c == 2)
        || (a == 192 && b == 168)
        || (a == 198 && (b == 18 || b == 19))
        || (a == 198 && b == 51 && c == 100)
        || (a == 203 && b == 0 && c == 113)
        // Multicast, reserved and the broadcast address.
        || a >= 224;
    !refused
}

fn is_global_v6(ip: Ipv6Addr) -> bool {
    let segments = ip.segments();
    // `::ffff:a.b.c.d` and `64:ff9b::a.b.c.d` hold an IPv4 address.
    let embedded = |tail: [u16; 2]| {
        let [high, low] = tail;
        let [a, b] = high.to_be_bytes();
        let [c, d] = low.to_be_bytes();
        Ipv4Addr::new(a, b, c, d)
    };
    if segments[..5] == [0; 5] && segments[5] == 0xffff {
        return is_global_v4(embedded([segments[6], segments[7]]));
    }
    if segments[0] == 0x0064 && segments[1] == 0xff9b && segments[2..6] == [0; 4] {
        return is_global_v4(embedded([segments[6], segments[7]]));
    }
    let refused = ip.is_unspecified()
        || ip.is_loopback()
        // fc00::/7
        || (segments[0] & 0xfe00) == 0xfc00
        // fe80::/10
        || (segments[0] & 0xffc0) == 0xfe80
        // ff00::/8
        || (segments[0] & 0xff00) == 0xff00
        // 2001:db8::/32
        || (segments[0] == 0x2001 && segments[1] == 0x0db8);
    !refused
}

/// Connect to `host` and `port`, with the rule of `policy`.
///
/// The function resolves the name, removes each address that the policy
/// refuses, and connects to the first address that remains. `timeout` is
/// the time for all the attempts.
///
/// # Errors
///
/// Gives [`NetError::NotPublic`] when the name has addresses and the
/// policy refuses each of them. Gives [`NetError::Io`] when the name has
/// no address or no connection succeeds.
pub fn connect(
    host: &str,
    port: u16,
    policy: &NetPolicy,
    timeout: Duration,
) -> Result<TcpStream, NetError> {
    connect_with(host, port, policy, timeout, &|host, port| {
        Ok((host, port).to_socket_addrs()?.collect())
    })
}

/// The same as [`connect`], with the resolution as a parameter. A test
/// uses this to give the addresses of a name.
pub(crate) fn connect_with(
    host: &str,
    port: u16,
    policy: &NetPolicy,
    timeout: Duration,
    resolve: &dyn Fn(&str, u16) -> io::Result<Vec<SocketAddr>>,
) -> Result<TcpStream, NetError> {
    let deadline = Instant::now() + timeout;
    let resolved = resolve(host, port).map_err(NetError::Io)?;
    let total = resolved.len();
    let permitted: Vec<SocketAddr> = resolved
        .into_iter()
        .filter(|address| policy.permits(*address))
        .collect();
    if permitted.is_empty() {
        return Err(if total == 0 {
            NetError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "the name has no address",
            ))
        } else {
            NetError::NotPublic {
                host: host.to_owned(),
            }
        });
    }
    let mut last = None;
    for address in permitted {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            break;
        }
        match TcpStream::connect_timeout(&address, left) {
            Ok(stream) => return Ok(stream),
            Err(error) => last = Some(error),
        }
    }
    Err(NetError::Io(last.unwrap_or_else(|| {
        io::Error::new(io::ErrorKind::TimedOut, "the connection timed out")
    })))
}

/// The proxy of an agent. With `Public`, no agent uses a proxy from the
/// environment (ADR 0057 item 4).
pub(crate) fn proxy(policy: &NetPolicy) -> Option<ureq::Proxy> {
    if policy.is_public() {
        None
    } else {
        ureq::Proxy::try_from_env()
    }
}

/// Make an agent. With `Public`, the agent resolves names through
/// [`GuardResolver`].
pub(crate) fn agent(config: ureq::config::Config, policy: &NetPolicy) -> ureq::Agent {
    match policy {
        NetPolicy::Open => config.into(),
        NetPolicy::Public(rules) => ureq::Agent::with_parts(
            config,
            DefaultConnector::default(),
            GuardResolver::new(rules.clone(), Arc::new(DefaultResolver::default())),
        ),
    }
}

/// The error source in a `ureq` error that a refused name gives.
#[derive(Debug)]
struct Refused {
    host: String,
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&not_public_message(&self.host))
    }
}

impl std::error::Error for Refused {}

/// The host of a refused name, when `error` comes from [`GuardResolver`].
pub(crate) fn refused_host(error: &ureq::Error) -> Option<&str> {
    match error {
        ureq::Error::Io(io_error) => io_error
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<Refused>())
            .map(|refused| refused.host.as_str()),
        _ => None,
    }
}

/// A `ureq` resolver that removes each address that the policy refuses.
#[derive(Debug)]
pub(crate) struct GuardResolver {
    rules: PublicRules,
    inner: Arc<dyn Resolver>,
}

impl GuardResolver {
    pub(crate) fn new(rules: PublicRules, inner: Arc<dyn Resolver>) -> Self {
        Self { rules, inner }
    }
}

impl Resolver for GuardResolver {
    fn resolve(
        &self,
        uri: &ureq::http::Uri,
        config: &ureq::config::Config,
        timeout: NextTimeout,
    ) -> Result<ResolvedSocketAddrs, ureq::Error> {
        let all = self.inner.resolve(uri, config, timeout)?;
        let policy = NetPolicy::Public(self.rules.clone());
        let mut kept = self.empty();
        for address in all.iter().copied().filter(|a| policy.permits(*a)) {
            kept.push(address);
        }
        if kept.is_empty() {
            let host = uri.host().unwrap_or_default().to_owned();
            return Err(ureq::Error::Io(io::Error::new(
                io::ErrorKind::PermissionDenied,
                Refused { host },
            )));
        }
        Ok(kept)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    fn ip(text: &str) -> IpAddr {
        text.parse().expect("an address")
    }

    #[test]
    fn is_global_refuses_one_address_of_each_range() {
        let refused = [
            "0.1.2.3",
            "10.1.2.3",
            "100.64.0.1",
            "100.127.255.254",
            "127.0.0.1",
            "169.254.169.254",
            "172.16.0.1",
            "172.31.255.255",
            "192.0.0.8",
            "192.0.2.1",
            "192.168.1.1",
            "198.18.0.1",
            "198.19.255.255",
            "198.51.100.1",
            "203.0.113.1",
            "224.0.0.1",
            "240.0.0.1",
            "255.255.255.255",
            "::",
            "::1",
            "fc00::1",
            "fd12:3456::1",
            "fe80::1",
            "ff02::1",
            "2001:db8::1",
            "::ffff:127.0.0.1",
            "::ffff:10.0.0.1",
            "64:ff9b::10.0.0.1",
            "64:ff9b::a00:1",
        ];
        for text in refused {
            assert!(!is_global(ip(text)), "{text}");
        }
    }

    #[test]
    fn is_global_permits_a_global_address() {
        let permitted = [
            "1.1.1.1",
            "8.8.8.8",
            "100.63.255.255",
            "100.128.0.1",
            "172.15.255.255",
            "172.32.0.1",
            "198.17.255.255",
            "198.20.0.1",
            "223.255.255.255",
            "2606:4700::1111",
            "::ffff:1.1.1.1",
            "64:ff9b::1.1.1.1",
        ];
        for text in permitted {
            assert!(is_global(ip(text)), "{text}");
        }
    }

    #[test]
    fn a_policy_permits_only_a_global_address_or_an_exception() {
        let local = SocketAddr::new(ip("127.0.0.1"), 4000);
        assert!(NetPolicy::Open.permits(local));
        assert!(!NetPolicy::public().permits(local));
        let rules = PublicRules::with_test_exceptions(vec![local]);
        let policy = NetPolicy::Public(rules);
        assert!(policy.permits(local));
        // The exception is for the exact socket address.
        assert!(!policy.permits(SocketAddr::new(ip("127.0.0.1"), 4001)));
    }

    #[test]
    fn connect_refuses_a_loopback_address_with_public() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a listener");
        let port = listener.local_addr().expect("an address").port();
        let error = connect(
            "127.0.0.1",
            port,
            &NetPolicy::public(),
            Duration::from_secs(2),
        )
        .expect_err("a refusal");
        assert!(matches!(error, NetError::NotPublic { ref host } if host == "127.0.0.1"));
        assert!(connect("127.0.0.1", port, &NetPolicy::Open, Duration::from_secs(2)).is_ok());
    }

    #[test]
    fn connect_uses_only_the_permitted_address_of_a_resolution() {
        let refused = TcpListener::bind("127.0.0.1:0").expect("a listener");
        let allowed = TcpListener::bind("127.0.0.1:0").expect("a listener");
        refused.set_nonblocking(true).expect("non-blocking");
        let refused_address = refused.local_addr().expect("an address");
        let allowed_address = allowed.local_addr().expect("an address");
        // The resolver test double gives the refused address first.
        let policy = NetPolicy::Public(PublicRules::with_test_exceptions(vec![allowed_address]));
        let stream = connect_with(
            "double.example",
            allowed_address.port(),
            &policy,
            Duration::from_secs(2),
            &|_, _| Ok(vec![refused_address, allowed_address]),
        )
        .expect("a connection");
        assert_eq!(stream.peer_addr().expect("a peer"), allowed_address);
        allowed.accept().expect("the permitted address got it");
        assert!(refused.accept().is_err(), "the refused address got nothing");
    }

    #[test]
    fn connect_gives_not_public_when_the_resolution_has_no_permitted_address() {
        let local = SocketAddr::new(ip("127.0.0.1"), 9);
        let error = connect_with(
            "double.example",
            9,
            &NetPolicy::public(),
            Duration::from_secs(1),
            &|_, _| Ok(vec![local]),
        )
        .expect_err("a refusal");
        assert!(matches!(error, NetError::NotPublic { ref host } if host == "double.example"));
        assert_eq!(
            error.to_string(),
            "Studio on a server cannot connect to double.example, because it is not a public address."
        );
    }

    #[derive(Debug)]
    struct Double(Vec<SocketAddr>);

    impl Resolver for Double {
        fn resolve(
            &self,
            _: &ureq::http::Uri,
            _: &ureq::config::Config,
            _: NextTimeout,
        ) -> Result<ResolvedSocketAddrs, ureq::Error> {
            let mut all = self.empty();
            for address in &self.0 {
                all.push(*address);
            }
            Ok(all)
        }
    }

    #[test]
    fn the_guard_resolver_removes_each_refused_address() {
        let uri: ureq::http::Uri = "http://double.example/".parse().expect("a URI");
        let config = ureq::config::Config::default();
        let timeout = NextTimeout::default();
        let public = SocketAddr::new(ip("1.1.1.1"), 80);
        let private = SocketAddr::new(ip("10.0.0.1"), 80);

        let guard = GuardResolver::new(
            PublicRules::default(),
            Arc::new(Double(vec![private, public])),
        );
        let kept = guard
            .resolve(&uri, &config, timeout)
            .expect("a permitted address");
        assert_eq!(&kept[..], &[public]);

        let guard = GuardResolver::new(PublicRules::default(), Arc::new(Double(vec![private])));
        let error = guard
            .resolve(&uri, &config, timeout)
            .expect_err("a refusal");
        assert_eq!(refused_host(&error), Some("double.example"));
    }
}

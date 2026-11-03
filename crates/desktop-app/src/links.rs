//! The rule for a link that leaves the page (ADR 0124 item 5).

use url::{Origin, Url};

/// What the window does with an address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkAction {
    /// The window goes to the address.
    Keep,
    /// The system browser opens the address. The window stays.
    OpenOutside,
    /// Nothing opens the address.
    Block,
}

/// Decide what to do with `address`, for a window with the origin `window`.
///
/// A new window is never [`LinkAction::Keep`]: pass `new_window` as `true`
/// and a [`LinkAction::Keep`] result becomes [`LinkAction::OpenOutside`].
#[must_use]
pub fn link_action(window: &Origin, address: &Url, new_window: bool) -> LinkAction {
    let action = if address.as_str() == "about:blank" || address.origin() == *window {
        LinkAction::Keep
    } else if matches!(address.scheme(), "http" | "https" | "mailto") {
        LinkAction::OpenOutside
    } else {
        LinkAction::Block
    };
    if new_window && action == LinkAction::Keep {
        LinkAction::OpenOutside
    } else {
        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(address: &str, new_window: bool) -> LinkAction {
        let window = Url::parse("http://127.0.0.1:42069/#token=x")
            .unwrap()
            .origin();
        link_action(&window, &Url::parse(address).unwrap(), new_window)
    }

    #[test]
    fn an_address_of_the_window_origin_is_kept() {
        assert_eq!(
            action("http://127.0.0.1:42069/api/x", false),
            LinkAction::Keep,
            "ADR 0124 item 5: an address of the origin of the window stays in the window."
        );
        assert_eq!(
            action("blob:http://127.0.0.1:42069/abc", false),
            LinkAction::Keep,
            "ADR 0124 item 5: the origin of a blob address is the origin in it."
        );
        assert_eq!(action("about:blank", false), LinkAction::Keep);
    }

    #[test]
    fn another_origin_opens_outside() {
        for address in [
            "https://musicindex.org/",
            "http://127.0.0.1:42070/",
            "http://localhost:42069/",
            "mailto:a@example.org",
        ] {
            assert_eq!(
                action(address, false),
                LinkAction::OpenOutside,
                "ADR 0124 item 5: {address} must open in the system browser."
            );
        }
    }

    #[test]
    fn other_schemes_are_blocked() {
        for address in [
            "data:text/html,x",
            "javascript:alert(1)",
            "file:///etc/passwd",
            "blob:https://example.org/abc",
        ] {
            assert_eq!(
                action(address, false),
                LinkAction::Block,
                "ADR 0124 item 5: {address} must not open."
            );
        }
    }

    #[test]
    fn a_new_window_is_never_kept() {
        assert_eq!(
            action("http://127.0.0.1:42069/", true),
            LinkAction::OpenOutside,
            "ADR 0124 item 5: a new window opens outside, also for the origin of the window."
        );
        assert_eq!(action("about:blank", true), LinkAction::OpenOutside);
        assert_eq!(action("javascript:x", true), LinkAction::Block);
    }
}

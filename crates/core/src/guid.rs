//! Feed identity (ADR 0008).

use uuid::Uuid;

/// The namespace UUID that the Podcasting 2.0 specification gives for
/// `podcast:guid`.
const PODCAST_GUID_NAMESPACE: Uuid = Uuid::from_u128(0xead4_c236_bf58_58c6_a2c6_a6b2_8d12_8cb6);

/// Make the `podcast:guid` of a new feed from its URL.
///
/// The function removes the protocol scheme and each trailing slash, then
/// makes a `UUIDv5` in the podcast namespace, as the specification says. Use
/// it only for a new feed: a feed keeps its GUID when its URL changes.
///
/// # Examples
///
/// ```
/// use studio_core::guid::podcast_guid_for_url;
///
/// assert_eq!(
///     podcast_guid_for_url("https://podnews.net/rss"),
///     "9b024349-ccf0-5f69-a609-6b82873eab3c",
/// );
/// ```
#[must_use]
pub fn podcast_guid_for_url(url: &str) -> String {
    let without_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    let seed = without_scheme.trim_end_matches('/');
    Uuid::new_v5(&PODCAST_GUID_NAMESPACE, seed.as_bytes()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guid_matches_the_specification_examples() {
        assert_eq!(
            podcast_guid_for_url("mp3s.nashownotes.com/pc20rss.xml"),
            "917393e3-1b1e-5cef-ace4-edaa54e1f810"
        );
        assert_eq!(
            podcast_guid_for_url("https://mp3s.nashownotes.com/pc20rss.xml"),
            "917393e3-1b1e-5cef-ace4-edaa54e1f810"
        );
        assert_eq!(
            podcast_guid_for_url("http://podnews.net/rss/"),
            "9b024349-ccf0-5f69-a609-6b82873eab3c"
        );
    }
}

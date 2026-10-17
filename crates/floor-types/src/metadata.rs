//!
//!
//!
//!
//!
//!
//!
//!
//!
//!

use crate::constants::{MAX_METADATA_BASE_LEN, MAX_URI_LEN, METADATA_URI_SUFFIX};

///
pub const MAX_BASE58_PUBKEY_LEN: usize = 44;

///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
pub const METADATA_BASE_SCHEMES: [&str; 3] = ["https://", "ipfs://", "ar://"];

///
const _: () = assert!(MAX_METADATA_BASE_LEN + 1 + MAX_BASE58_PUBKEY_LEN + METADATA_URI_SUFFIX.len() <= MAX_URI_LEN);

///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
pub fn normalize_metadata_base(raw: &str) -> Option<&str> {
    if raw.is_empty() {
        return Some("");
    }
    let mut end = raw.len();
    while end > 0 && raw.as_bytes()[end - 1] == b'/' {
        end -= 1;
    }
    let base = &raw[..end];
    if base.is_empty() || base.len() > MAX_METADATA_BASE_LEN {
        return None;
    }
    if !base.as_bytes().iter().all(|c| (0x21..=0x7E).contains(c)) {
        return None;
    }
    if base.as_bytes().iter().any(|c| matches!(c, b'#' | b'?' | b'\\')) {
        return None;
    }


    let scheme = METADATA_BASE_SCHEMES.iter().find(|s| base.starts_with(**s))?;
    if base.len() == scheme.len() {
        return None;
    }


    let after = &base[scheme.len()..];
    let authority = &after[..after.find('/').unwrap_or(after.len())];
    if authority.contains('@') {
        return None;
    }
    Some(base)
}

///
///
///
///
pub fn metadata_uri(base: &str, mint_base58: &str) -> String {
    if base.is_empty() {
        return String::new();
    }
    let mut s = String::with_capacity(base.len() + 1 + mint_base58.len() + METADATA_URI_SUFFIX.len());
    s.push_str(base);
    s.push('/');
    s.push_str(mint_base58);
    s.push_str(METADATA_URI_SUFFIX);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINT: &str = "8JUjWjAyXTMB4ZXcV7nk3p6aiaPtGGbJcUFTPKxGC2Wx";

    #[test]
    fn the_template_is_base_slash_mint_dot_json() {
        assert_eq!(
            metadata_uri("https://floor.supply/meta/solana", MINT),
            "https://floor.supply/meta/solana/8JUjWjAyXTMB4ZXcV7nk3p6aiaPtGGbJcUFTPKxGC2Wx.json"
        );
        assert_eq!(metadata_uri("", MINT), "", "no host yet is the empty uri, never a dead link");
    }

    #[test]
    fn a_trailing_slash_is_stripped_so_the_join_never_doubles_it() {
        for raw in ["https://h.example/m/", "https://h.example/m//", "https://h.example/m"] {
            assert_eq!(normalize_metadata_base(raw), Some("https://h.example/m"), "{raw}");
        }
        assert_eq!(metadata_uri(normalize_metadata_base("https://h.example/m/").unwrap(), MINT), format!("https://h.example/m/{MINT}.json"));
    }

    #[test]
    fn empty_is_no_host_yet_but_slashes_alone_are_refused() {
        assert_eq!(normalize_metadata_base(""), Some(""));
        assert_eq!(normalize_metadata_base("/"), None);
        assert_eq!(normalize_metadata_base("///"), None);
    }

    #[test]
    fn a_base_that_is_not_an_absolute_ascii_url_is_refused() {
        assert_eq!(normalize_metadata_base("floor.supply/meta"), None, "no scheme");
        assert_eq!(normalize_metadata_base("://h"), None, "no scheme name");
        assert_eq!(normalize_metadata_base("https://"), None, "scheme only");
        assert_eq!(normalize_metadata_base("https://h example"), None, "a space");
        assert_eq!(normalize_metadata_base("https://h\u{0}x"), None, "an interior NUL");
        assert_eq!(normalize_metadata_base("https://hé.example"), None, "not ASCII");
        assert_eq!(normalize_metadata_base("https://h\nx"), None, "a control byte");


        assert_eq!(normalize_metadata_base("https://h/x#"), None, "a fragment: the JSON is never fetched");
        assert_eq!(normalize_metadata_base("https://h/x#frag"), None, "a fragment with a body");
        assert_eq!(normalize_metadata_base("https://h/x?v=1"), None, "a query");
        assert_eq!(normalize_metadata_base("https://h\\x"), None, "a backslash");
        assert_eq!(normalize_metadata_base("https://evil@h"), None, "userinfo moves the host");
        assert_eq!(normalize_metadata_base("https://evil@h/x"), None, "userinfo with a path");



        assert_eq!(normalize_metadata_base("javascript://evil.example"), None, "an XSS payload as a base");
        assert_eq!(normalize_metadata_base("javascript://x/a"), None, "the same with a path");
        assert_eq!(normalize_metadata_base("file:///etc/passwd"), None, "a local file");
        assert_eq!(normalize_metadata_base("data://x"), None, "any other scheme");
        assert_eq!(normalize_metadata_base("http://h"), None, "plaintext: a permanent pointer is not MITM-able");
        assert_eq!(normalize_metadata_base("HTTPS://h"), None, "the scheme is case-sensitive, never rewritten");
        assert_eq!(normalize_metadata_base("ipfs://"), None, "a scheme with nothing after it");
        assert_eq!(normalize_metadata_base("ar://"), None, "a scheme with nothing after it");


        assert_eq!(normalize_metadata_base("ipfs://bafyfakecid"), Some("ipfs://bafyfakecid"), "ipfs");
        assert_eq!(normalize_metadata_base("ar://txid/"), Some("ar://txid"), "arweave, trailing slash stripped");
        assert_eq!(normalize_metadata_base("https://h/a%20b"), Some("https://h/a%20b"), "percent-encoding");
        assert_eq!(normalize_metadata_base("https://h/a@b"), Some("https://h/a@b"), "`@` in the path");
        assert!(normalize_metadata_base(&format!("https://h.example/{}", "a".repeat(MAX_METADATA_BASE_LEN))).is_none());
        let longest = format!("https://h/{}", "a".repeat(MAX_METADATA_BASE_LEN - 10));
        assert_eq!(longest.len(), MAX_METADATA_BASE_LEN);
        assert_eq!(normalize_metadata_base(&longest), Some(longest.as_str()));
    }

    #[test]
    fn the_longest_template_fits_the_uri_bound() {
        let base = format!("https://h/{}", "a".repeat(MAX_METADATA_BASE_LEN - 10));
        let uri = metadata_uri(&base, &"1".repeat(MAX_BASE58_PUBKEY_LEN));
        assert_eq!(uri.len(), MAX_METADATA_BASE_LEN + 1 + MAX_BASE58_PUBKEY_LEN + 5);
        assert!(uri.len() <= MAX_URI_LEN, "{} <= {MAX_URI_LEN}", uri.len());
    }
}

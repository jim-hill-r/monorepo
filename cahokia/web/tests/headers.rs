//! Cloudflare Pages sends `public/_headers` as real HTTP headers. Its CSP must
//! be the `<meta>` CSP in `src/main.rs` plus `frame-ancestors`, which a
//! `<meta>` CSP can't set.

const PAGES_HEADERS: &str = include_str!("../public/_headers");
const MAIN_RS: &str = include_str!("../src/main.rs");

/// The value of `name` in the `/*` rule of `_headers`.
fn pages_header(name: &str) -> Option<&'static str> {
    let prefix = format!("{name}:");
    PAGES_HEADERS
        .lines()
        .skip_while(|line| line.trim() != "/*")
        .skip(1)
        .take_while(|line| line.starts_with(char::is_whitespace))
        .find_map(|line| line.trim().strip_prefix(prefix.as_str()))
        .map(str::trim)
}

/// The string value of `CONTENT_SECURITY_POLICY` in `src/main.rs`.
fn meta_csp() -> &'static str {
    let start = MAIN_RS
        .find("const CONTENT_SECURITY_POLICY: &str = \"")
        .expect("src/main.rs defines CONTENT_SECURITY_POLICY")
        + "const CONTENT_SECURITY_POLICY: &str = \"".len();
    let len = MAIN_RS[start..].find('"').expect("closing quote");
    &MAIN_RS[start..start + len]
}

#[test]
fn pages_headers_csp_is_the_meta_csp_plus_frame_ancestors() {
    assert_eq!(
        pages_header("Content-Security-Policy"),
        Some(format!("{}; frame-ancestors 'none'", meta_csp()).as_str())
    );
}

#[test]
fn pages_headers_send_the_security_headers() {
    assert_eq!(pages_header("Referrer-Policy"), Some("no-referrer"));
    assert_eq!(pages_header("X-Content-Type-Options"), Some("nosniff"));
    assert_eq!(pages_header("X-Frame-Options"), Some("DENY"));
    assert_eq!(
        pages_header("Strict-Transport-Security"),
        Some("max-age=31536000; includeSubDomains")
    );
}

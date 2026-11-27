//! Tests that keep `web/_headers` in step with the game. Cloudflare Pages
//! sends that file as real HTTP headers on the web build
//! (`standards/scripts/bevy-release.sh` copies it into `dist/web`), and its
//! Content-Security-Policy must still let the game run: Bevy's wasm needs
//! `'wasm-unsafe-eval'`, and the gameplay events in `src/telemetry.rs` go to
//! another origin, which `connect-src` must allow.

use crate::telemetry::ENDPOINT;

/// The Pages `_headers` file.
const PAGES_HEADERS: &str = include_str!("../web/_headers");

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

/// The sources of one CSP directive, or `None` when the CSP doesn't set it.
fn csp_directive(name: &str) -> Option<Vec<&'static str>> {
    pages_header("Content-Security-Policy")?
        .split(';')
        .map(str::split_whitespace)
        .find_map(|mut parts| (parts.next() == Some(name)).then(|| parts.collect::<Vec<_>>()))
}

/// The origin (`scheme://host`) of an absolute URL.
fn origin(url: &str) -> &str {
    let after_scheme = url.find("://").map_or(0, |i| i + 3);
    let end = url[after_scheme..]
        .find('/')
        .map_or(url.len(), |i| after_scheme + i);
    &url[..end]
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

#[test]
fn csp_denies_by_default_and_forbids_framing() {
    assert_eq!(csp_directive("default-src"), Some(vec!["'none'"]));
    assert_eq!(csp_directive("frame-ancestors"), Some(vec!["'none'"]));
}

#[test]
fn csp_lets_the_wasm_start_without_inline_scripts() {
    let script_src = csp_directive("script-src").unwrap_or_default();
    assert!(script_src.contains(&"'self'"), "{script_src:?}");
    assert!(script_src.contains(&"'wasm-unsafe-eval'"), "{script_src:?}");
    assert!(!script_src.contains(&"'unsafe-inline'"), "{script_src:?}");
    assert!(!script_src.contains(&"'unsafe-eval'"), "{script_src:?}");
}

#[test]
fn csp_lets_the_gameplay_events_reach_the_telemetry_worker() {
    let connect_src = csp_directive("connect-src").unwrap_or_default();
    assert!(
        connect_src.contains(&origin(ENDPOINT)),
        "connect-src {connect_src:?} must allow {}",
        origin(ENDPOINT)
    );
}

#[test]
fn csp_allows_the_web_analytics_beacon_pages_injects() {
    let script_src = csp_directive("script-src").unwrap_or_default();
    let connect_src = csp_directive("connect-src").unwrap_or_default();
    assert!(script_src.contains(&"https://static.cloudflareinsights.com"));
    assert!(connect_src.contains(&"https://cloudflareinsights.com"));
}

#[test]
fn origin_drops_the_path() {
    assert_eq!(
        origin("https://example.workers.dev/event"),
        "https://example.workers.dev"
    );
    assert_eq!(origin("https://example.com"), "https://example.com");
}

#[test]
fn csp_directive_is_none_when_unset() {
    assert_eq!(csp_directive("worker-src"), None);
    assert_eq!(pages_header("Not-A-Header"), None);
}

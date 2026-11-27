//! The security headers Cloudflare Pages sends for the exported site.
use std::fs;

fn headers() -> String {
    fs::read_to_string("_headers").expect("Failed to read _headers")
}

#[test]
fn test_headers_file_sends_the_security_headers() {
    let headers = headers();
    for required in [
        "Content-Security-Policy:",
        "frame-ancestors 'none'",
        "X-Content-Type-Options: nosniff",
        "Referrer-Policy:",
        "Strict-Transport-Security:",
    ] {
        assert!(
            headers.contains(required),
            "_headers should send {required}"
        );
    }
}

#[test]
fn test_export_copies_headers_file_to_its_root() {
    let main = fs::read_to_string("src/main.rs").expect("Failed to read src/main.rs");
    assert!(
        main.contains(r#".static_alias("/_headers", "_headers")"#),
        "src/main.rs should alias _headers to the export's root"
    );
}

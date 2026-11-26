//! Pure helpers for the browser redirect flow.
//!
//! They hold no browser calls, so they are tested natively. `web` uses them
//! for the redirect URI, the page to return to after sign-in, and the URL to
//! show once the authorization response is consumed.

/// The OAuth `redirect_uri` for a site: its origin plus `/`.
///
/// Auth0 accepts a redirect URI only when it exactly matches an Allowed
/// Callback URL, and the token exchange must send the same URI as the
/// authorize request. The current page URL breaks both (other paths, query
/// strings, and the `code` and `state` Auth0 adds on the way back), so the
/// redirect URI is always the site root.
pub fn redirect_url_for_origin(origin: &str) -> String {
    format!("{}/", origin.trim_end_matches('/'))
}

/// The page to send the user back to after sign-in, as a same-origin path.
///
/// Returns the path unchanged when it is a local path (starts with one `/`),
/// and `/` for anything else, so a tampered value can't redirect off-site
/// (for example a protocol-relative `//evil.example` or a full URL).
pub fn safe_return_path(return_to: Option<&str>) -> String {
    match return_to {
        Some(path) if path.starts_with('/') && !path.starts_with("//") && !path.contains('\\') => {
            path.to_string()
        }
        _ => "/".to_string(),
    }
}

/// A URL's query string with the authorization response parameters removed.
///
/// `search` is `location.search` (with or without its leading `?`). Returns
/// the remaining query with a leading `?`, or an empty string when nothing is
/// left, so `pathname + result + hash` is the URL to show after the token
/// exchange. Without this, reloading the page would send the single-use code
/// to the token endpoint again.
pub fn strip_authorization_response(search: &str) -> String {
    const RESPONSE_PARAMS: [&str; 4] = ["code", "state", "error", "error_description"];
    let kept: Vec<&str> = search
        .trim_start_matches('?')
        .split('&')
        .filter(|pair| !pair.is_empty())
        .filter(|pair| {
            let name = pair.split('=').next().unwrap_or_default();
            !RESPONSE_PARAMS.contains(&name)
        })
        .collect();
    if kept.is_empty() {
        String::new()
    } else {
        format!("?{}", kept.join("&"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirect_url_is_origin_plus_slash() {
        assert_eq!(
            redirect_url_for_origin("https://jimhillr-cahokia.pages.dev"),
            "https://jimhillr-cahokia.pages.dev/"
        );
        assert_eq!(
            redirect_url_for_origin("http://localhost:8080"),
            "http://localhost:8080/"
        );
    }

    #[test]
    fn redirect_url_does_not_double_the_slash() {
        assert_eq!(
            redirect_url_for_origin("https://jimhillr-cahokia.pages.dev/"),
            "https://jimhillr-cahokia.pages.dev/"
        );
    }

    #[test]
    fn return_path_keeps_local_paths() {
        assert_eq!(safe_return_path(Some("/history")), "/history");
        assert_eq!(
            safe_return_path(Some("/explore?era=late#map")),
            "/explore?era=late#map"
        );
        assert_eq!(safe_return_path(Some("/")), "/");
    }

    #[test]
    fn return_path_rejects_off_site_targets() {
        for bad in [
            "https://evil.example/",
            "//evil.example/",
            "/\\evil.example",
            "javascript:alert(1)",
            "history",
            "",
        ] {
            assert_eq!(safe_return_path(Some(bad)), "/", "{bad:?} must not be kept");
        }
        assert_eq!(safe_return_path(None), "/");
    }

    #[test]
    fn strips_code_and_state() {
        assert_eq!(strip_authorization_response("?code=abc&state=xyz"), "");
        assert_eq!(strip_authorization_response("code=abc&state=xyz"), "");
        assert_eq!(strip_authorization_response(""), "");
    }

    #[test]
    fn strips_error_response() {
        assert_eq!(
            strip_authorization_response("?error=access_denied&error_description=no&state=s"),
            ""
        );
    }

    #[test]
    fn keeps_other_query_parameters() {
        assert_eq!(
            strip_authorization_response("?tab=map&code=abc&state=xyz"),
            "?tab=map"
        );
        assert_eq!(
            strip_authorization_response("?code=abc&a=1&state=xyz&b=2"),
            "?a=1&b=2"
        );
        // Only exact parameter names are removed.
        assert_eq!(strip_authorization_response("?codex=1"), "?codex=1");
    }
}

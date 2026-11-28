#[cfg(test)]
use auth_sdk::auth0::Auth0Urls;
#[cfg(test)]
use auth_sdk::provider::ProviderConfig;
#[cfg(target_arch = "wasm32")]
use auth_sdk::provider::{AuthError, AuthProvider, ProviderConfig};
#[cfg(any(target_arch = "wasm32", test))]
use auth_sdk::redirect::redirect_url_for_origin;
#[cfg(target_arch = "wasm32")]
use auth_sdk::web::{WebAuthProvider, fetch_origin_from_browser};

#[cfg(target_arch = "wasm32")]
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use ui::{Navbar, Sidebar};
#[cfg(target_arch = "wasm32")]
use views::about::About;
#[cfg(target_arch = "wasm32")]
use views::explore::Explore;
#[cfg(target_arch = "wasm32")]
use views::family_tree::FamilyTree;
#[cfg(target_arch = "wasm32")]
use views::history::History;
#[cfg(target_arch = "wasm32")]
use views::home::Home;

mod views;

/// Auth0 tenant. `ProviderConfig::auth0` builds the issuer (with the trailing
/// slash OIDC discovery needs), authorize and token URLs from it.
#[cfg(any(target_arch = "wasm32", test))]
const AUTH0_DOMAIN: &str = "dev-jdadpn4pckxevrv5.us.auth0.com";
#[cfg(target_arch = "wasm32")]
const CLIENT_ID: &str = "6CHDECRfCsyYdCFq1hwqKNwCHxxmum3E";

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(WebNavbar)]
    #[route("/")]
    Home {},
    #[route("/about")]
    About {},
    #[route("/history")]
    History {},
    #[route("/explore")]
    Explore {},
    #[route("/family-tree")]
    FamilyTree {},
}

#[cfg(target_arch = "wasm32")]
const FAVICON: Asset = asset!("/assets/favicon.ico");
#[cfg(target_arch = "wasm32")]
const MAIN_CSS: Asset = asset!("/assets/main.css");
#[cfg(target_arch = "wasm32")]
const HEADER_CSS: Asset = asset!("/assets/styling/header.css");

/// Content Security Policy for the application
/// - default-src 'none': Block all sources by default
/// - script-src 'self' 'wasm-unsafe-eval': Allow scripts from same origin and WASM
/// - connect-src 'self' AUTH0_DOMAIN: Allow API calls to same origin and Auth0
/// - img-src 'self': Allow images from same origin
/// - style-src 'self': Allow stylesheets from same origin only. No inline
///   styles: every style is a class in a linked stylesheet (see
///   `ui/CONTRIBUTING.md`), so an injected `<style>` or `style` attribute is blocked
/// - font-src 'self': Allow fonts from same origin (required for self-hosted web fonts)
/// - base-uri 'self': Restrict base tag to same origin
/// - form-action 'self': Restrict form submissions to same origin
///
/// Note: The Auth0 origin is hardcoded in the CSP string because it must be a
/// const. `test_csp_connect_src_matches_auth0_domain` checks it against
/// `Auth0Urls::origin` for AUTH0_DOMAIN.
#[cfg(any(target_arch = "wasm32", test))]
const CONTENT_SECURITY_POLICY: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; connect-src 'self' https://dev-jdadpn4pckxevrv5.us.auth0.com; img-src 'self'; style-src 'self'; font-src 'self'; base-uri 'self'; form-action 'self'";

#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // This binary is only meant to run in a WebAssembly environment
    eprintln!("This application is designed to run in a web browser as WebAssembly.");
    eprintln!("Please use `dx serve` to run the development server.");
}

#[cfg(target_arch = "wasm32")]
#[component]
fn App() -> Element {
    let auth = use_resource(|| async move {
        // Always the site root, so it matches Auth0's Allowed Callback URLs
        // from any page; the user is sent back to their page after.
        let redirect_url = fetch_origin_from_browser()
            .map(|origin| redirect_url_for_origin(&origin))
            .unwrap_or_default();
        match ProviderConfig::auth0(AUTH0_DOMAIN, CLIENT_ID, redirect_url) {
            Ok(config) => WebAuthProvider::new(config).await,
            Err(error) => Err(error),
        }
    });
    use_context_provider(|| auth);

    rsx! {
        // Include this CSP in server response headers for defense in depth redundancy
        // CSP Analysis: Modern Dioxus (v0.7.x) compiles to WebAssembly and doesn't require
        // unsafe-eval or unsafe-inline for scripts. WASM modules and proper event binding
        // are used instead of eval() or inline script handlers. The 'wasm-unsafe-eval'
        // directive is needed for WebAssembly instantiation in some browsers.
        // Styles are linked stylesheets only, so style-src has no unsafe-inline.
        // Auth0 domain whitelisted for OAuth authentication flows (authorize, token endpoints)
        document::Meta {
            http_equiv: "Content-Security-Policy",
            content: CONTENT_SECURITY_POLICY,
        }
        document::Link { rel: "icon", href: FAVICON }
        // The design system's tokens and base sheet, before the app's own
        // sheets so they can use the tokens.
        design_system_dioxus::Stylesheets { default_theme: ui::DEFAULT_THEME }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: HEADER_CSS }

        Router::<Route> {}
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
fn WebNavbar() -> Element {
    let auth = use_context::<Resource<Result<WebAuthProvider, AuthError>>>();
    let auth_state = auth.read();
    let mut sidebar_open = use_signal(|| false);

    // After a sign-in redirect the app loads at `/`; put the user back on the
    // page they clicked Login from.
    use_effect(move || {
        if let Some(Ok(provider)) = &*auth.read()
            && let Some(route) = provider.return_to().and_then(return_route)
        {
            navigator().replace(route);
        }
    });

    rsx! {
        // Header bar with Cahokia title and navigation links
        div {
            id: "header",
            div {
                class: "header-title",
                h1 { "Cahokia" }
            }
            nav {
                class: "header-nav",
                Link { to: Route::Home {}, "Home" }
                Link { to: Route::About {}, "About" }
                Link { to: Route::History {}, "History" }
                Link { to: Route::Explore {}, "Explore" }
            }
            button {
                class: "sidebar-toggle",
                onclick: move |_| sidebar_open.set(!sidebar_open()),
                "☰"
            }
        }

        Navbar {
            match &*auth_state {
                Some(Ok(provider)) => {
                    let provider = provider.clone();
                    rsx! {
                        button {
                            onclick: move |_| {
                                if let Err(e) = provider.login() {
                                    eprintln!("Login error: {}", e);
                                }
                            },
                            "Login"
                        }
                    }
                },
                Some(Err(err)) => rsx! {
                    div {
                        class: "error",
                        "Authentication Error: {err}"
                    }
                },
                None => rsx! {
                    div { "Loading authentication..." }
                },
            }
        }

        Sidebar {
            is_open: sidebar_open,
            h2 { "Controls" }
            design_system_dioxus::ThemePicker {}
            nav {
                class: "sidebar-links",
                ul {
                    li {
                        Link {
                            to: Route::FamilyTree {},
                            onclick: move |_| sidebar_open.set(false),
                            "Family Tree"
                        }
                    }
                }
            }
            button {
                onclick: move |_| sidebar_open.set(false),
                "Close"
            }
        }

        Outlet::<Route> {}
    }
}

/// The route for a sign-in return path, or `None` when it is unknown or is
/// the home page the redirect already landed on.
#[cfg(target_arch = "wasm32")]
fn return_route(path: &str) -> Option<Route> {
    // No route takes a query or hash, so match on the path alone.
    let path = path.split(['?', '#']).next().unwrap_or("/");
    path.parse::<Route>()
        .ok()
        .filter(|route| *route != Route::Home {})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csp_contains_auth0_domain() {
        // Verify CSP includes the Auth0 domain for OAuth requests
        assert!(
            CONTENT_SECURITY_POLICY.contains("https://dev-jdadpn4pckxevrv5.us.auth0.com"),
            "CSP should include Auth0 domain"
        );
    }

    #[test]
    fn test_csp_blocks_default_sources() {
        // Verify CSP starts with default-src 'none' to block all by default
        assert!(
            CONTENT_SECURITY_POLICY.starts_with("default-src 'none'"),
            "CSP should block all sources by default"
        );
    }

    #[test]
    fn test_csp_allows_wasm() {
        // Verify CSP includes wasm-unsafe-eval for WebAssembly
        assert!(
            CONTENT_SECURITY_POLICY.contains("'wasm-unsafe-eval'"),
            "CSP should allow WASM evaluation"
        );
    }

    #[test]
    fn test_csp_blocks_inline_styles() {
        assert!(
            CONTENT_SECURITY_POLICY.contains("style-src 'self';"),
            "style-src must allow only same-origin stylesheets"
        );
        assert!(
            !CONTENT_SECURITY_POLICY.contains("'unsafe-inline'"),
            "CSP must not allow inline styles or scripts"
        );
    }

    #[test]
    fn test_csp_allows_fonts() {
        // Verify CSP includes font-src for web fonts
        assert!(
            CONTENT_SECURITY_POLICY.contains("font-src 'self'"),
            "CSP should allow fonts from same origin"
        );
    }

    #[test]
    fn test_auth0_urls_built_from_domain() {
        // The issuer needs Auth0's trailing slash for OIDC discovery; the
        // helper adds it, so it can't drift from the endpoint URLs.
        let config = ProviderConfig::auth0(AUTH0_DOMAIN, "client", "https://app.example/");
        let config = config.ok();
        assert_eq!(
            config.as_ref().and_then(|c| c.issuer_url.as_deref()),
            Some("https://dev-jdadpn4pckxevrv5.us.auth0.com/")
        );
        assert_eq!(
            config.as_ref().map(|c| c.auth_url.as_str()),
            Some("https://dev-jdadpn4pckxevrv5.us.auth0.com/authorize")
        );
        assert_eq!(
            config.as_ref().map(|c| c.token_url.as_str()),
            Some("https://dev-jdadpn4pckxevrv5.us.auth0.com/oauth/token")
        );
    }

    #[test]
    fn test_csp_connect_src_matches_auth0_domain() {
        let origin = Auth0Urls::from_domain(AUTH0_DOMAIN)
            .map(|urls| urls.origin().to_string())
            .unwrap_or_default();
        let connect_src = CONTENT_SECURITY_POLICY
            .split(';')
            .map(str::trim)
            .find_map(|directive| directive.strip_prefix("connect-src "))
            .unwrap_or_default();
        assert!(
            connect_src
                .split_whitespace()
                .any(|source| source == origin),
            "CSP connect-src must allow {origin}: {connect_src}"
        );
    }

    #[test]
    fn test_redirect_url_matches_auth0_callback_urls() {
        // These must stay in Auth0's Allowed Callback URLs (issues/human/):
        // the redirect URI is the site root from every page, never the
        // current page URL.
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
    fn test_provider_config_includes_issuer_url() {
        // Verify ProviderConfig is created with issuer_url
        // This is a compile-time check that the config structure is correct
        let config = ProviderConfig {
            client_id: "test".into(),
            auth_url: "https://example.com/auth".into(),
            token_url: "https://example.com/token".into(),
            redirect_url: "https://example.com/callback".into(),
            issuer_url: Some("https://example.com".into()),
        };
        assert_eq!(config.issuer_url, Some("https://example.com".into()));
    }
}

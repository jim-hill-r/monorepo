//! The cookbook web app: the routes, the stylesheets and the CSP.

mod auth;
mod components;
mod schedule;

use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use auth::{AUTH0_DOMAIN, CLIENT_ID};
#[cfg(target_arch = "wasm32")]
use auth_sdk::provider::ProviderConfig;
#[cfg(target_arch = "wasm32")]
use auth_sdk::web::{WebAuthProvider, fetch_current_location_from_browser};

const HEADER_CSS: Asset = asset!("/assets/styling/header.css");
const SIDEBAR_CSS: Asset = asset!("/assets/styling/sidebar.css");
const HOME_CSS: Asset = asset!("/assets/styling/home.css");
const RECIPE_CSS: Asset = asset!("/assets/styling/recipe.css");
const PLAN_CSS: Asset = asset!("/assets/styling/plan.css");

/// The design system theme the cookbook uses until the reader picks another.
const DEFAULT_THEME: &str = "earthy";

/// Content Security Policy for the application
/// - default-src 'none': Block all sources by default
/// - script-src 'self' 'wasm-unsafe-eval' POSTHOG_ASSETS: Allow scripts from same origin,
///   WASM, and PostHog's script host for session replay (public/posthog.js, production only)
/// - connect-src 'self' AUTH0_DOMAIN POSTHOG_HOSTS: Allow API calls to same origin, Auth0,
///   and PostHog's ingest and config hosts
/// - img-src 'self': Allow images from same origin
/// - style-src 'self': Allow stylesheets from same origin only. No inline styles: the
///   design system links its CSS as files and the app sets no `style` attributes
/// - font-src 'self': Allow fonts from same origin (required for self-hosted web fonts)
/// - base-uri 'self': Restrict base tag to same origin
/// - form-action 'self': Restrict form submissions to same origin
///
/// Note: The Auth0 origin is hardcoded in the CSP string because it must be a
/// const. `test_csp_connect_src_matches_auth0_domain` checks it against
/// `Auth0Urls::origin` for `auth::AUTH0_DOMAIN`.
const CONTENT_SECURITY_POLICY: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval' https://us-assets.i.posthog.com; connect-src 'self' https://dev-jdadpn4pckxevrv5.us.auth0.com https://us.i.posthog.com https://us-assets.i.posthog.com; img-src 'self'; style-src 'self'; font-src 'self'; base-uri 'self'; form-action 'self'";

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    #[cfg(target_arch = "wasm32")]
    {
        let auth = use_resource(|| async move {
            let redirect_url = fetch_current_location_from_browser().unwrap_or_default();
            match ProviderConfig::auth0(AUTH0_DOMAIN, CLIENT_ID, redirect_url) {
                Ok(config) => WebAuthProvider::new(config).await,
                Err(error) => Err(error),
            }
        });
        use_context_provider(|| auth);
    }

    // Initialize sidebar visibility state
    // Start with desktop default (visible), will be updated on mount for mobile
    #[allow(unused_mut)]
    let mut sidebar_visible = use_signal(|| true);

    // Detect mobile viewport and update sidebar state accordingly
    #[cfg(target_arch = "wasm32")]
    {
        use_effect(move || {
            // Hide the sidebar on a phone-width viewport (768 px or less)
            if let Some(window) = web_sys::window()
                && let Ok(width) = window.inner_width()
                && let Some(width) = width.as_f64()
                && width <= 768.0
            {
                sidebar_visible.set(false);
            }
        });
    }

    use_context_provider(|| sidebar_visible);

    rsx! {
        // Cloudflare Pages also sends this CSP, plus frame-ancestors, as a real
        // header from public/_headers; tests keep the two in step.
        // CSP Analysis: Modern Dioxus (v0.7.x) compiles to WebAssembly and doesn't require
        // unsafe-eval or unsafe-inline for scripts. WASM modules and proper event binding
        // are used instead of eval() or inline script handlers. The 'wasm-unsafe-eval'
        // directive is needed for WebAssembly instantiation in some browsers.
        // Styles are same-origin files only, with no 'unsafe-inline'.
        // Auth0 domain whitelisted for OAuth authentication flows (authorize, token endpoints)
        if cfg!(target_arch = "wasm32") {
            document::Meta {
                http_equiv: "Content-Security-Policy",
                content: CONTENT_SECURITY_POLICY,
            }
        }
        // The design system's tokens, base sheet and theme script, before
        // the app's own sheets so they can use the tokens.
        design_system_dioxus::Stylesheets { default_theme: DEFAULT_THEME }
        document::Link { rel: "stylesheet", href: HEADER_CSS }
        document::Link { rel: "stylesheet", href: SIDEBAR_CSS }
        document::Link { rel: "stylesheet", href: HOME_CSS }
        document::Link { rel: "stylesheet", href: RECIPE_CSS }
        document::Link { rel: "stylesheet", href: PLAN_CSS }
        Router::<Route> {}
    }
}

// dioxus 0.7.10's Routable derive generates code that rustc 1.100 flags as
// unreachable. The allow has to cover the generated impls, so the enum sits
// in its own module.

// dioxus 0.7.10's Routable derive generates code that rustc 1.100 flags as
// unreachable. The allow has to cover the generated impls, so the enum sits
// in its own module.
mod route {
    #![allow(unreachable_code)]
    use crate::components::{Header, Home, PageNotFound, Plan, Recipe};
    use dioxus::prelude::*;

    #[derive(Clone, Routable, Debug, PartialEq)]
    pub enum Route {
        #[layout(Header)]
        #[route("/")]
        Home {},

        #[route("/recipe/:day")]
        Recipe { day: u32 },

        #[route("/plan/:week")]
        Plan { week: u32 },

        #[route("/:..route")]
        PageNotFound { route: Vec<String> },
    }
}
use route::Route;

#[cfg(test)]
mod tests {
    use super::*;

    /// The Pages `_headers` file, served as real HTTP headers by Cloudflare.
    const PAGES_HEADERS: &str = include_str!("../public/_headers");

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

    #[test]
    fn pages_headers_csp_is_the_meta_csp_plus_frame_ancestors() {
        assert_eq!(
            pages_header("Content-Security-Policy"),
            Some(format!("{CONTENT_SECURITY_POLICY}; frame-ancestors 'none'").as_str())
        );
    }

    #[test]
    fn csp_allows_no_inline_styles() {
        let style_src = CONTENT_SECURITY_POLICY
            .split(';')
            .map(str::trim)
            .find(|directive| directive.starts_with("style-src"));
        assert_eq!(style_src, Some("style-src 'self'"));
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

    /// The PostHog loader, served from 'self' (decision 0001).
    const POSTHOG_JS: &str = include_str!("../public/posthog.js");

    #[test]
    fn csp_allows_only_the_posthog_hosts_the_loader_uses() {
        for host in [
            "https://us.i.posthog.com",
            "https://us-assets.i.posthog.com",
        ] {
            assert!(POSTHOG_JS.contains(host), "posthog.js uses {host}");
            assert!(
                CONTENT_SECURITY_POLICY.contains(host),
                "the CSP allows {host}"
            );
        }
        assert!(!CONTENT_SECURITY_POLICY.contains("*.posthog.com"));
    }

    #[test]
    fn posthog_loads_only_on_production_for_real_visitors_with_inputs_masked() {
        assert!(POSTHOG_JS.contains(r#"PRODUCTION_HOST = "jimhillr-cookbook.pages.dev""#));
        assert!(POSTHOG_JS.contains("window.location.hostname !== PRODUCTION_HOST"));
        assert!(POSTHOG_JS.contains("jimhillr-evals"));
        assert!(POSTHOG_JS.contains("doNotTrack"));
        assert!(POSTHOG_JS.contains("maskAllInputs: true"));
        assert!(POSTHOG_JS.contains("recordBody: false"));
        assert!(POSTHOG_JS.contains(r#"var PROJECT = "phc_"#));
        assert!(!POSTHOG_JS.contains("\"phx_"), "never a personal API key");
    }

    #[test]
    fn index_html_loads_the_posthog_loader_from_self() {
        let index = include_str!("../index.html");
        assert!(index.contains(r#"<script src="/posthog.js" defer></script>"#));
    }

    #[test]
    fn pages_header_ignores_headers_outside_the_catch_all_rule() {
        assert_eq!(pages_header("Not-A-Header"), None);
    }

    #[test]
    fn test_route_home_path() {
        // Test that Home route is at root path
        assert_eq!(Route::Home {}.to_string(), "/");
    }

    #[test]
    fn test_route_recipe_path() {
        // Test that Recipe route generates correct path
        assert_eq!(Route::Recipe { day: 1 }.to_string(), "/recipe/1");
        assert_eq!(Route::Recipe { day: 100 }.to_string(), "/recipe/100");
        assert_eq!(Route::Recipe { day: 365 }.to_string(), "/recipe/365");
    }

    #[test]
    fn test_route_plan_path() {
        // Test that Plan route generates correct path
        assert_eq!(Route::Plan { week: 1 }.to_string(), "/plan/1");
        assert_eq!(Route::Plan { week: 26 }.to_string(), "/plan/26");
        assert_eq!(Route::Plan { week: 52 }.to_string(), "/plan/52");
        assert_eq!(Route::Plan { week: 53 }.to_string(), "/plan/53");
    }

    #[test]
    fn test_auth0_issuer_has_trailing_slash() {
        // Auth0's OIDC discovery returns its issuer with a trailing slash and
        // openidconnect compares it exactly; the auth_sdk helper adds it.
        let urls = auth_sdk::auth0::Auth0Urls::from_domain(auth::AUTH0_DOMAIN).ok();
        assert_eq!(
            urls.as_ref().map(|u| u.issuer.as_str()),
            Some("https://dev-jdadpn4pckxevrv5.us.auth0.com/")
        );
        assert_eq!(
            urls.as_ref().map(|u| u.token.as_str()),
            Some("https://dev-jdadpn4pckxevrv5.us.auth0.com/oauth/token")
        );
    }

    #[test]
    fn test_csp_connect_src_matches_auth0_domain() {
        let origin = auth_sdk::auth0::Auth0Urls::from_domain(auth::AUTH0_DOMAIN)
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
}

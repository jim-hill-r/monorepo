use auth_sdk::provider::{AuthError, AuthProvider, ProviderConfig};
use auth_sdk::web::{WebAuthProvider, fetch_current_location_from_browser};

use dioxus::prelude::*;

use ui::Navbar;
use views::about::About;
use views::explore::Explore;
use views::history::History;
use views::home::Home;

mod views;

const CLIENT_ID: &str = "6CHDECRfCsyYdCFq1hwqKNwCHxxmum3E";
const AUTH_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/authorize";
const TOKEN_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/oauth/token";

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
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_CSS: Asset = asset!("/assets/styling/header.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let auth = use_resource(|| async move {
        WebAuthProvider::new(ProviderConfig {
            client_id: CLIENT_ID.into(),
            auth_url: AUTH_URL.into(),
            token_url: TOKEN_URL.into(),
            redirect_url: fetch_current_location_from_browser().unwrap_or("".into()),
        })
        .await
    });
    use_context_provider(|| auth);

    rsx! {
        // Include this CSP in server response headers for defense in depth redundancy
        // TODO: Audit unsafe-inline and unsafe-eval to understand if this opens potential for XSS
        document::Meta {
            http_equiv: "Content-Security-Policy",
            content: "default-src 'none'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; connect-src 'self'; img-src 'self'; style-src 'self';base-uri 'self';form-action 'self'",
        }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: HEADER_CSS }

        Router::<Route> {}
    }
}

#[component]
fn WebNavbar() -> Element {
    let auth = use_context::<Resource<Result<WebAuthProvider, AuthError>>>();
    let auth_state = auth.read();
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
        }

        Navbar {
            match &*auth_state {
                Some(Ok(provider)) => {
                    let provider = provider.clone();
                    rsx! {
                        button { onclick: move |_| provider.login().unwrap(), "Login" } // TODO (agent-generated): Handle login() errors properly instead of unwrapping
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

        Outlet::<Route> {}
    }
}

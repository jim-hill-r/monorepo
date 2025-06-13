use dioxus::prelude::*;

use ui::Navbar;
use views::{Home, Login, LoginCahokia, LoginCahokiaCode};

mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(WebNavbar)]
    #[route("/")]
    Home {},
    #[route("/login")]
    Login {},
    #[route("/login/cahokia")]
    LoginCahokia {},
    #[route("/login/cahokia/code")]
    LoginCahokiaCode {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        // TODO: Validate that this content-security-policy is actually doing anything
        // Ensure this policy is also set by headers in the server
        document::Meta {
            http_equiv: "Content-Security-Policy",
            content: "default-src 'none'; script-src 'unsafe-inline'; connect-src 'self'; img-src 'self'; style-src 'self';base-uri 'self';form-action 'self'",
        }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Router::<Route> {}
    }
}

/// A web-specific Router around the shared `Navbar` component
/// which allows us to use the web-specific `Route` enum.
#[component]
fn WebNavbar() -> Element {
    rsx! {
        Navbar {
            Link { to: Route::Login {}, "Login" }
        }

        Outlet::<Route> {}
    }
}

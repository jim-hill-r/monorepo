use dioxus::prelude::*;

use ui::Navbar;
use views::home::Home;
use views::login::Login;
use views::login_cahokia::LoginCahokia;
use views::login_cahokia_code::LoginCahokiaCode;

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
        // Include this CSP in server response headers for defense in depth redundancy
        // TODO: Audit unsafe-inline and unsafe-eval to understand if this opens potential for XSS
        document::Meta {
            http_equiv: "Content-Security-Policy",
            content: "default-src 'none'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; connect-src 'self'; img-src 'self'; style-src 'self';base-uri 'self';form-action 'self'",
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

use dioxus::prelude::*;

use ui::Navbar;
use views::Home;

mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(DesktopNavbar)]
    #[route("/")]
    Home {}
}

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Router::<Route> {}
    }
}

#[component]
fn DesktopNavbar() -> Element {
    rsx! {
        Navbar {
            button { 
                onclick: move |_| {
                    // TODO: from AI: Implement desktop authentication when auth_sdk supports desktop platforms
                }, 
                "Login" 
            }
        }

        Outlet::<Route> {}
    }
}

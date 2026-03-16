mod canvas_js;
mod components;
mod pages;
mod state;

use dioxus::prelude::*;
use pages::{congratulations::Congratulations, home::Home, letter::Letter, word::Word};

fn main() {
    dioxus::launch(App);
}

/// Application routes matching blue.eel.education's navigation structure
#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/letter")]
    Letter {},
    #[route("/word")]
    Word {},
    #[route("/congratulations")]
    Congratulations {},
}

/// Root application component with navigation and routing
#[component]
fn App() -> Element {
    rsx! {
        div {
            style: "font-family: system-ui, -apple-system, sans-serif; min-height: 100vh; background-color: #f9f9f9;",

            // Navigation header
            nav {
                style: "background-color: #178CA4; padding: 12px 24px; display: flex; align-items: center; gap: 24px;",
                Link {
                    to: Route::Home {},
                    style: "color: white; text-decoration: none; font-size: 1.4rem; font-weight: 700;",
                    "Blue Eel"
                }
                div { style: "flex: 1;" }
                Link {
                    to: Route::Letter {},
                    style: "color: white; text-decoration: none; font-size: 0.95rem; opacity: 0.9;",
                    "Letters"
                }
                Link {
                    to: Route::Word {},
                    style: "color: white; text-decoration: none; font-size: 0.95rem; opacity: 0.9;",
                    "Words"
                }
            }

            // Page content
            main {
                style: "padding: 16px;",
                Router::<Route> {}
            }
        }
    }
}

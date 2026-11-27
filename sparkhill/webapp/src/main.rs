mod canvas_js;
mod components;
mod pages;
mod patterns;
mod progress;
mod state;
mod technique;
mod tracing_check;

use dioxus::prelude::*;
use pages::{
    capital::Capital, congratulations::Congratulations, home::Home, letter::Letter, word::Word,
};

fn main() {
    dioxus::launch(App);
}

/// Application routes matching blue.eel.education's navigation structure
#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[layout(Shell)]
    #[route("/")]
    Home {},
    #[route("/letter")]
    Letter {},
    #[route("/word")]
    Word {},
    #[route("/capital")]
    Capital {},
    #[route("/congratulations")]
    Congratulations {},
}

/// Root application component: the router
#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

/// Navigation header and page content, around every route. It is a layout
/// inside the router because `Link` needs a parent router.
#[component]
fn Shell() -> Element {
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
                Link {
                    to: Route::Capital {},
                    style: "color: white; text-decoration: none; font-size: 0.95rem; opacity: 0.9;",
                    "Capitals"
                }
            }

            // Page content
            main {
                style: "padding: 16px;",
                Outlet::<Route> {}
            }
        }
    }
}

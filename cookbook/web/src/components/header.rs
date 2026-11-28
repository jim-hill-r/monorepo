//! The header and the page layout around every route.

use dioxus::prelude::*;

use crate::auth::render_auth_section;
use crate::components::Sidebar;
use crate::route::Route;
use crate::schedule::{get_current_day_of_year, get_current_week_of_year};

#[component]
pub fn Header() -> Element {
    let mut sidebar_visible = use_context::<Signal<bool>>();

    let today = get_current_day_of_year();
    let current_week = get_current_week_of_year();

    rsx! {
        header {
            id: "header",
            div {
                class: "header-left",
                button {
                    class: "hamburger-btn",
                    onclick: move |_| {
                        sidebar_visible.set(!sidebar_visible());
                    },
                    "aria-label": "Toggle sidebar",
                    "aria-expanded": "{sidebar_visible}",
                    span { class: "hamburger-icon" }
                    span { class: "hamburger-icon" }
                    span { class: "hamburger-icon" }
                }
                h1 { class: "header-title", "Cookbook" }
            }
            // Below 768 px these links move into the sidebar (see Sidebar),
            // so the header stays on one row.
            nav {
                class: "header-nav",
                "aria-label": "Main",
                Link { to: Route::Home {}, "Home" }
                Link { to: Route::Recipe { day: today }, "Recipes" }
                Link { to: Route::Plan { week: current_week }, "Plans" }
            }
            div {
                class: "header-auth",
                { render_auth_section() }
            }
        }

        Sidebar {}

        div {
            id: "content",
            class: if !sidebar_visible() { "sidebar-hidden" } else { "" },
            Outlet::<Route> {}
        }
    }
}

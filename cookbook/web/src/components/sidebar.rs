//! The sidebar: the main links on phones, this week's recipes, nearby plans and the appearance picker.

use dioxus::prelude::*;

use crate::route::Route;
use crate::schedule::{
    get_current_day_of_year, get_current_week_of_year, get_sidebar_plan_weeks,
    get_sidebar_recipe_days,
};

#[component]
pub fn Sidebar() -> Element {
    let sidebar_visible = use_context::<Signal<bool>>();
    let recipe_days = get_sidebar_recipe_days();
    let plan_weeks = get_sidebar_plan_weeks();
    let today = get_current_day_of_year();
    let current_week = get_current_week_of_year();

    rsx! {
        aside {
            id: "sidebar",
            class: if !sidebar_visible() { "hidden" } else { "" },
            // The header's links, shown here only below 768 px, where the
            // header hides them to stay on one row.
            nav {
                class: "sidebar-nav",
                "aria-label": "Main",
                Link { to: Route::Home {}, "Home" }
                Link { to: Route::Recipe { day: today }, "Recipes" }
                Link { to: Route::Plan { week: current_week }, "Plans" }
            }
            h2 { "Quick Navigation" }

            div {
                class: "sidebar-section",
                h3 { "Daily Recipes" }
                for (day, formatted_date) in recipe_days {
                    Link { to: Route::Recipe { day }, "{formatted_date}" }
                }
            }

            div {
                class: "sidebar-section",
                h3 { "Weekly Plans" }
                for (week, formatted_date) in plan_weeks {
                    Link { to: Route::Plan { week }, "Week {week}: {formatted_date}" }
                }
            }

            div {
                class: "sidebar-appearance",
                design_system_dioxus::ThemePicker {}
            }
        }
    }
}

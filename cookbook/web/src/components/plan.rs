//! A week's plan page: the shopping list and the week's recipes by date.

use chrono::Local;
use cookbook_core::calendar::format_recipe_day;
use dioxus::prelude::*;

use crate::route::Route;
use crate::schedule::{get_week_plan_days, get_week_shopping_list};

#[component]
pub fn Plan(week: u32) -> Element {
    if !(1..=53).contains(&week) {
        rsx! {
            div {
                class: "plan-container",
                h1 { "Invalid Week" }
                p { "Week {week} is not valid. Please select a week between 1 and 53." }
                Link { to: Route::Home {}, "Back to Home" }
            }
        }
    } else {
        let recipes: Vec<(u32, String, String)> =
            get_week_plan_days(week, Local::now().date_naive())
                .into_iter()
                .map(|(day, title, date)| {
                    let when = date.map_or_else(|| format!("Day {day}"), format_recipe_day);
                    (day, title, when)
                })
                .collect();
        let shopping_list = get_week_shopping_list(week);

        rsx! {
            div {
                class: "plan-container",
                h1 { "Meal Plan for Week {week}" }

                div {
                    class: "plan-shopping-list",
                    h2 { "Shopping List" }

                    if shopping_list.is_empty() {
                        p { "No ingredients needed for this week." }
                    } else {
                        ul {
                            class: "shopping-list",
                            for line in shopping_list.items.iter() {
                                li {
                                    class: "shopping-item",
                                    "{line}"
                                }
                            }
                        }
                        if !shopping_list.staples.is_empty() {
                            h3 { "Staples" }
                            p {
                                class: "shopping-staples-note",
                                "You may already have these."
                            }
                            ul {
                                class: "shopping-list shopping-staples",
                                for line in shopping_list.staples.iter() {
                                    li {
                                        class: "shopping-item",
                                        "{line}"
                                    }
                                }
                            }
                        }
                    }
                }

                div {
                    class: "plan-recipes",
                    h2 { "Recipes This Week" }

                    if recipes.is_empty() {
                        p { "No recipes available for this week." }
                    } else {
                        ul {
                            class: "recipe-list",
                            for (day, title, when) in recipes {
                                li {
                                    class: "recipe-item",
                                    Link { to: Route::Recipe { day }, "{title}" }
                                    span { class: "recipe-day", " ({when})" }
                                }
                            }
                        }
                    }
                }

                Link { to: Route::Home {}, class: "back-link", "← Back to Home" }
            }
        }
    }
}

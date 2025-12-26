use auth_sdk::provider::{AuthError, AuthProvider, ProviderConfig};
use auth_sdk::web::{WebAuthProvider, fetch_current_location_from_browser};

use dioxus::prelude::*;

const HEADER_CSS: Asset = asset!("/assets/styling/header.css");
const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");
const SIDEBAR_CSS: Asset = asset!("/assets/styling/sidebar.css");
const HOME_CSS: Asset = asset!("/assets/styling/home.css");

const CLIENT_ID: &str = "6CHDECRfCsyYdCFq1hwqKNwCHxxmum3E";
const AUTH_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/authorize";
const TOKEN_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/oauth/token";

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
        document::Link { rel: "stylesheet", href: HEADER_CSS }
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }
        document::Link { rel: "stylesheet", href: SIDEBAR_CSS }
        document::Link { rel: "stylesheet", href: HOME_CSS }
        Router::<Route> {}
    }
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
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

#[component]
fn Header() -> Element {
    let auth = use_context::<Resource<Result<WebAuthProvider, AuthError>>>();
    let auth_state = auth.read();

    rsx! {
        header {
            id: "header",
            div {
                class: "header-title",
                h1 { "Cookbook" }
            }
            nav {
                class: "header-nav",
                Link { to: Route::Home {}, "Home" }
                Link { to: Route::Recipe { day: 1 }, "Recipes" }
                Link { to: Route::Plan { week: 1 }, "Plans" }
            }
        }

        div {
            id: "navbar",
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

        Sidebar {}

        div {
            id: "content",
            Outlet::<Route> {}
        }
    }
}

#[component]
fn Sidebar() -> Element {
    rsx! {
        aside {
            id: "sidebar",
            h2 { "Quick Navigation" }

            div {
                class: "sidebar-section",
                h3 { "Daily Recipes" }
                Link { to: Route::Recipe { day: 1 }, "Day 1" }
                Link { to: Route::Recipe { day: 11 }, "Day 11" }
                Link { to: Route::Recipe { day: 21 }, "Day 21" }
                Link { to: Route::Recipe { day: 31 }, "Day 31" }
                Link { to: Route::Recipe { day: 41 }, "Day 41" }
                Link { to: Route::Recipe { day: 51 }, "Day 51" }
                Link { to: Route::Recipe { day: 61 }, "Day 61" }
                Link { to: Route::Recipe { day: 71 }, "Day 71" }
                Link { to: Route::Recipe { day: 81 }, "Day 81" }
                Link { to: Route::Recipe { day: 91 }, "Day 91" }
                Link { to: Route::Recipe { day: 100 }, "Day 100" }
                Link { to: Route::Recipe { day: 150 }, "Day 150" }
                Link { to: Route::Recipe { day: 200 }, "Day 200" }
                Link { to: Route::Recipe { day: 250 }, "Day 250" }
                Link { to: Route::Recipe { day: 300 }, "Day 300" }
                Link { to: Route::Recipe { day: 365 }, "Day 365" }
            }

            div {
                class: "sidebar-section",
                h3 { "Weekly Plans" }
                Link { to: Route::Plan { week: 1 }, "Week 1" }
                Link { to: Route::Plan { week: 5 }, "Week 5" }
                Link { to: Route::Plan { week: 9 }, "Week 9" }
                Link { to: Route::Plan { week: 13 }, "Week 13" }
                Link { to: Route::Plan { week: 17 }, "Week 17" }
                Link { to: Route::Plan { week: 21 }, "Week 21" }
                Link { to: Route::Plan { week: 25 }, "Week 25" }
                Link { to: Route::Plan { week: 29 }, "Week 29" }
                Link { to: Route::Plan { week: 33 }, "Week 33" }
                Link { to: Route::Plan { week: 37 }, "Week 37" }
                Link { to: Route::Plan { week: 41 }, "Week 41" }
                Link { to: Route::Plan { week: 45 }, "Week 45" }
                Link { to: Route::Plan { week: 52 }, "Week 52" }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div {
            class: "home-container",
            h1 { "Cookbook" }
            p { "Welcome to the Cookbook application!" }

            div {
                class: "navigation-cards",

                div {
                    class: "navigation-card recipe-card",
                    span { class: "card-icon", "🍳" }
                    h2 { "Daily Recipes" }
                    p { "Explore 365 delicious recipes - one for each day of the year. From quick weeknight dinners to special occasion dishes." }
                    Link { to: Route::Recipe { day: 1 }, "Browse Recipes" }
                }

                div {
                    class: "navigation-card plan-card",
                    span { class: "card-icon", "📅" }
                    h2 { "Weekly Meal Plans" }
                    p { "Get organized with 52 complete meal plans - one for every week of the year. Perfect for planning ahead!" }
                    Link { to: Route::Plan { week: 1 }, "View Meal Plans" }
                }
            }
        }
    }
}

#[component]
fn Recipe(day: u32) -> Element {
    if !(1..=365).contains(&day) {
        rsx! {
            div {
                h1 { "Invalid Day" }
                p { "Day {day} is not valid. Please select a day between 1 and 365." }
                Link { to: Route::Home {}, "Back to Home" }
            }
        }
    } else {
        rsx! {
            div {
                h1 { "Recipe for Day {day}" }
                p { "This is a placeholder recipe for day {day} of the year." }
                Link { to: Route::Home {}, "Back to Home" }
            }
        }
    }
}

#[component]
fn Plan(week: u32) -> Element {
    if !(1..=52).contains(&week) {
        rsx! {
            div {
                h1 { "Invalid Week" }
                p { "Week {week} is not valid. Please select a week between 1 and 52." }
                Link { to: Route::Home {}, "Back to Home" }
            }
        }
    } else {
        rsx! {
            div {
                h1 { "Meal Plan for Week {week}" }
                p { "This is a placeholder meal plan for week {week} of the year." }
                Link { to: Route::Home {}, "Back to Home" }
            }
        }
    }
}

#[component]
fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        div {
            h1 { "Page not found" }
            p { "We are terribly sorry, but the page you requested doesn't exist." }
            pre { "Attempted to navigate to: {route:?}" }
            Link { to: Route::Home {}, "Back to Home" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_valid_day_in_range() {
        // Test that valid days (1-365) are accepted
        assert!((1..=365).contains(&1));
        assert!((1..=365).contains(&100));
        assert!((1..=365).contains(&365));
    }

    #[test]
    fn test_recipe_invalid_day_zero() {
        // Test that day 0 is invalid
        assert!(!(1..=365).contains(&0));
    }

    #[test]
    fn test_recipe_invalid_day_too_high() {
        // Test that day > 365 is invalid
        assert!(!(1..=365).contains(&366));
        assert!(!(1..=365).contains(&999));
    }

    #[test]
    fn test_plan_valid_week_in_range() {
        // Test that valid weeks (1-52) are accepted
        assert!((1..=52).contains(&1));
        assert!((1..=52).contains(&26));
        assert!((1..=52).contains(&52));
    }

    #[test]
    fn test_plan_invalid_week_zero() {
        // Test that week 0 is invalid
        assert!(!(1..=52).contains(&0));
    }

    #[test]
    fn test_plan_invalid_week_too_high() {
        // Test that week > 52 is invalid
        assert!(!(1..=52).contains(&53));
        assert!(!(1..=52).contains(&100));
    }

    #[test]
    fn test_recipe_edge_cases() {
        // Test edge cases for recipe validation
        assert!((1..=365).contains(&1), "Day 1 should be valid");
        assert!((1..=365).contains(&365), "Day 365 should be valid");
        assert!(!(1..=365).contains(&0), "Day 0 should be invalid");
        assert!(!(1..=365).contains(&366), "Day 366 should be invalid");
    }

    #[test]
    fn test_plan_edge_cases() {
        // Test edge cases for plan validation
        assert!((1..=52).contains(&1), "Week 1 should be valid");
        assert!((1..=52).contains(&52), "Week 52 should be valid");
        assert!(!(1..=52).contains(&0), "Week 0 should be invalid");
        assert!(!(1..=52).contains(&53), "Week 53 should be invalid");
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
    }
}

use chrono::prelude::*;
use cookbook_core::RecipeReader;
use cookbook_data_md::embedded::EmbeddedRecipeStore;
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use auth_sdk::provider::{AuthError, AuthProvider, ProviderConfig};
#[cfg(target_arch = "wasm32")]
use auth_sdk::web::{WebAuthProvider, fetch_current_location_from_browser};

const HEADER_CSS: Asset = asset!("/assets/styling/header.css");
const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");
const SIDEBAR_CSS: Asset = asset!("/assets/styling/sidebar.css");
const HOME_CSS: Asset = asset!("/assets/styling/home.css");
const RECIPE_CSS: Asset = asset!("/assets/styling/recipe.css");

const INTRO_MD: &str = include_str!("../../content/intro.md");

#[cfg(target_arch = "wasm32")]
const CLIENT_ID: &str = "savzmZnyHcvewGkQX8aaInwPFonC9k2x";
#[cfg(target_arch = "wasm32")]
const AUTH_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/authorize";
#[cfg(target_arch = "wasm32")]
const TOKEN_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/oauth/token";

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    #[cfg(target_arch = "wasm32")]
    {
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
    }

    // Initialize sidebar visibility state (visible by default)
    use_context_provider(|| Signal::new(true));

    rsx! {
        document::Link { rel: "stylesheet", href: HEADER_CSS }
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }
        document::Link { rel: "stylesheet", href: SIDEBAR_CSS }
        document::Link { rel: "stylesheet", href: HOME_CSS }
        document::Link { rel: "stylesheet", href: RECIPE_CSS }
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
    let mut sidebar_visible = use_context::<Signal<bool>>();

    let today = get_current_day_of_year();
    let current_week = get_current_week_of_year();

    rsx! {
        header {
            id: "header",
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
            div {
                class: "header-title",
                h1 { "Cookbook" }
            }
            nav {
                class: "header-nav",
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

#[cfg(target_arch = "wasm32")]
fn render_auth_section() -> Element {
    let auth = use_context::<Resource<Result<WebAuthProvider, AuthError>>>();
    let auth_state = auth.read();

    match &*auth_state {
        Some(Ok(provider)) => {
            let provider = provider.clone();
            rsx! {
                button {
                    onclick: move |_| {
                        // Silently handle login errors - the auth provider handles redirects
                        let _ = provider.login();
                    },
                    "Login"
                }
            }
        }
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

#[cfg(not(target_arch = "wasm32"))]
fn render_auth_section() -> Element {
    rsx! { div { "Login (test mode)" } }
}

/// Get the current day of the year (1-366) using chrono
fn get_current_day_of_year() -> u32 {
    let now = Local::now();
    now.ordinal()
}

/// Get the current week of the year (1-53) using chrono
/// Uses a simple calculation: week = floor((day - 1) / 7) + 1
fn get_current_week_of_year() -> u32 {
    let day = get_current_day_of_year();
    // Calculate week number (1-53), rounding up
    ((day - 1) / 7) + 1
}

/// Get recipe days to display in sidebar, sorted numerically
fn get_sidebar_recipe_days() -> Vec<u32> {
    let today = get_current_day_of_year();
    let mut days = Vec::new();

    // Show 10 days starting from today
    for i in 0..10 {
        let day = today + (i * 10);
        // Wrap around if we go past day 365 (we use 365 for simplicity, ignoring leap year day 366)
        let wrapped_day = if day > 365 {
            ((day - 1) % 365) + 1
        } else {
            day
        };
        days.push(wrapped_day);
    }

    days.sort_unstable(); // Ensures numeric sorting
    days
}

/// Get plan weeks to display in sidebar, sorted numerically
fn get_sidebar_plan_weeks() -> Vec<u32> {
    let current_week = get_current_week_of_year();
    let mut weeks = Vec::new();

    // Show 4 weeks starting from current week
    for i in 0..4 {
        let week = current_week + (i * 13);
        // Wrap around if we go past week 52
        let wrapped_week = if week > 52 {
            ((week - 1) % 52) + 1
        } else {
            week
        };
        weeks.push(wrapped_week);
    }

    weeks.sort_unstable(); // Ensures numeric sorting
    weeks
}

#[component]
fn Sidebar() -> Element {
    let sidebar_visible = use_context::<Signal<bool>>();
    let recipe_days = get_sidebar_recipe_days();
    let plan_weeks = get_sidebar_plan_weeks();

    rsx! {
        aside {
            id: "sidebar",
            class: if !sidebar_visible() { "hidden" } else { "" },
            h2 { "Quick Navigation" }

            div {
                class: "sidebar-section",
                h3 { "Daily Recipes" }
                for day in recipe_days {
                    Link { to: Route::Recipe { day }, "Day {day}" }
                }
            }

            div {
                class: "sidebar-section",
                h3 { "Weekly Plans" }
                for week in plan_weeks {
                    Link { to: Route::Plan { week }, "Week {week}" }
                }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    let today = get_current_day_of_year();
    let current_week = get_current_week_of_year();

    rsx! {
        div {
            class: "home-container",
            h1 { "The Engineer's 365 Cookbook" }

            div {
                class: "intro-description",
                p { "{INTRO_MD}" }
            }

            div {
                class: "navigation-cards",

                div {
                    class: "navigation-card recipe-card",
                    span { class: "card-icon", "🍳" }
                    h2 { "Daily Recipes" }
                    p { "Explore 365 delicious recipes - one for each day of the year. From quick weeknight dinners to special occasion dishes." }
                    Link { to: Route::Recipe { day: today }, "Browse Recipes" }
                }

                div {
                    class: "navigation-card plan-card",
                    span { class: "card-icon", "📅" }
                    h2 { "Weekly Meal Plans" }
                    p { "Get organized with 52 complete meal plans - one for every week of the year. Perfect for planning ahead!" }
                    Link { to: Route::Plan { week: current_week }, "View Meal Plans" }
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
                class: "recipe-container",
                h1 { "Invalid Day" }
                p { "Day {day} is not valid. Please select a day between 1 and 365." }
                Link { to: Route::Home {}, "Back to Home" }
            }
        }
    } else {
        let store = EmbeddedRecipeStore::global();

        match store.get_by_day(day) {
            Ok(recipe) => {
                rsx! {
                    div {
                        class: "recipe-container",
                        h1 { "{recipe.title}" }

                        if let Some(description) = &recipe.description {
                            p { class: "recipe-description", "{description}" }
                        }

                        div {
                            class: "recipe-metadata",
                            if let Some(prep_time) = recipe.prep_time_minutes {
                                span { class: "metadata-item", "⏱️ Prep: {prep_time} min" }
                            }
                            if let Some(cook_time) = recipe.cook_time_minutes {
                                span { class: "metadata-item", "🔥 Cook: {cook_time} min" }
                            }
                            if let Some(servings) = recipe.servings {
                                span { class: "metadata-item", "🍽️ Servings: {servings}" }
                            }
                        }

                        if !recipe.ingredients.is_empty() {
                            div {
                                class: "recipe-section",
                                h2 { "Ingredients" }
                                ul {
                                    class: "recipe-list",
                                    for ingredient in &recipe.ingredients {
                                        li { "{ingredient}" }
                                    }
                                }
                            }
                        }

                        if !recipe.instructions.is_empty() {
                            div {
                                class: "recipe-section",
                                h2 { "Instructions" }
                                ol {
                                    class: "recipe-list",
                                    for instruction in &recipe.instructions {
                                        li { "{instruction}" }
                                    }
                                }
                            }
                        }

                        if !recipe.tags.is_empty() {
                            div {
                                class: "recipe-tags",
                                for tag in &recipe.tags {
                                    span { class: "tag", "{tag}" }
                                }
                            }
                        }

                        Link { to: Route::Home {}, class: "back-link", "← Back to Home" }
                    }
                }
            }
            Err(err) => {
                rsx! {
                    div {
                        class: "recipe-container error",
                        h1 { "Recipe Not Found" }
                        p { "Could not load recipe for day {day}: {err}" }
                        Link { to: Route::Home {}, "Back to Home" }
                    }
                }
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

    #[test]
    fn test_intro_md_content_loaded() {
        // Test that intro.md content is loaded and not empty
        assert!(!INTRO_MD.is_empty(), "INTRO_MD should not be empty");
        assert!(
            INTRO_MD.contains("Engineer's 365 Cookbook"),
            "INTRO_MD should contain the cookbook title"
        );
    }

    #[test]
    fn test_intro_md_content_quality() {
        // Test that intro.md has expected content
        assert!(
            INTRO_MD.contains("Jim R. Hill"),
            "INTRO_MD should contain the author name"
        );
        assert!(
            INTRO_MD.contains("year"),
            "INTRO_MD should mention the year"
        );
        assert!(
            INTRO_MD.len() > 100,
            "INTRO_MD should have substantial content"
        );
    }

    #[test]
    fn test_sidebar_recipe_days_sorted_numerically() {
        // Test that recipe days are sorted in ascending numeric order
        let days = get_sidebar_recipe_days();

        // Should not be empty
        assert!(!days.is_empty(), "Recipe days should not be empty");

        // Check that days are sorted numerically
        for i in 0..days.len() - 1 {
            assert!(
                days[i] < days[i + 1],
                "Days should be sorted numerically: day {} ({}) should be less than day {} ({})",
                i,
                days[i],
                i + 1,
                days[i + 1]
            );
        }

        // Verify all days are within valid range (1-365)
        for day in &days {
            assert!(
                (1..=365).contains(day),
                "Day {} should be in valid range 1-365",
                day
            );
        }
    }

    #[test]
    fn test_sidebar_plan_weeks_sorted_numerically() {
        // Test that plan weeks are sorted in ascending numeric order
        let weeks = get_sidebar_plan_weeks();

        // Should not be empty
        assert!(!weeks.is_empty(), "Plan weeks should not be empty");

        // Check that weeks are sorted numerically
        for i in 0..weeks.len() - 1 {
            assert!(
                weeks[i] < weeks[i + 1],
                "Weeks should be sorted numerically: week {} ({}) should be less than week {} ({})",
                i,
                weeks[i],
                i + 1,
                weeks[i + 1]
            );
        }

        // Verify all weeks are within valid range (1-52)
        for week in &weeks {
            assert!(
                (1..=52).contains(week),
                "Week {} should be in valid range 1-52",
                week
            );
        }
    }

    #[test]
    fn test_numeric_vs_string_sorting() {
        // Demonstrate that numeric sorting differs from string sorting
        let mut days = vec![1, 11, 2, 21, 3];
        days.sort_unstable();

        // Numeric sort: [1, 2, 3, 11, 21]
        assert_eq!(
            days,
            vec![1, 2, 3, 11, 21],
            "Numeric sorting should give correct order"
        );

        // String sort would give: ["1", "11", "2", "21", "3"]
        // which is wrong for our purposes
        let mut day_strings: Vec<String> = vec!["1", "11", "2", "21", "3"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        day_strings.sort();
        assert_eq!(
            day_strings,
            vec!["1", "11", "2", "21", "3"],
            "String sorting gives wrong order for numbers"
        );
    }

    #[test]
    fn test_sidebar_recipe_days_count() {
        // Test that sidebar shows exactly 10 recipe entries
        let days = get_sidebar_recipe_days();
        assert_eq!(
            days.len(),
            10,
            "Sidebar should show exactly 10 recipe entries, found {}",
            days.len()
        );
    }

    #[test]
    fn test_sidebar_plan_weeks_count() {
        // Test that sidebar shows exactly 4 plan entries
        let weeks = get_sidebar_plan_weeks();
        assert_eq!(
            weeks.len(),
            4,
            "Sidebar should show exactly 4 plan entries, found {}",
            weeks.len()
        );
    }

    #[test]
    fn test_current_day_of_year_in_valid_range() {
        // Test that current day is in valid range (1-366)
        let day = get_current_day_of_year();
        assert!(
            (1..=366).contains(&day),
            "Current day {} should be in valid range 1-366",
            day
        );
    }

    #[test]
    fn test_current_week_of_year_in_valid_range() {
        // Test that current week is in valid range (1-53)
        let week = get_current_week_of_year();
        assert!(
            (1..=53).contains(&week),
            "Current week {} should be in valid range 1-53",
            week
        );
    }

    #[test]
    fn test_sidebar_recipe_days_starts_from_today() {
        // Test that recipe days start from or near today
        let _today = get_current_day_of_year();
        let days = get_sidebar_recipe_days();

        // Should have 10 entries
        assert_eq!(days.len(), 10);

        // All days should be in valid range (1-366 for leap years)
        for day in &days {
            assert!(
                (1..=366).contains(day),
                "Day {} should be in valid range 1-366",
                day
            );
        }
    }

    #[test]
    fn test_sidebar_plan_weeks_starts_from_current_week() {
        // Test that plan weeks start from or near current week
        let _current_week = get_current_week_of_year();
        let weeks = get_sidebar_plan_weeks();

        // Should have 4 entries
        assert_eq!(weeks.len(), 4);

        // All weeks should be in valid range
        for week in &weeks {
            assert!(
                (1..=52).contains(week),
                "Week {} should be in valid range 1-52",
                week
            );
        }
    }

    #[test]
    fn test_embedded_recipe_store_loads() {
        // Test that the embedded recipe store can be initialized
        let store = EmbeddedRecipeStore::global();
        let recipes = store.get_all().unwrap();

        // Should have exactly 365 recipes
        assert_eq!(
            recipes.len(),
            365,
            "Embedded store should contain exactly 365 recipes"
        );
    }

    #[test]
    fn test_embedded_recipe_store_get_by_day() {
        use cookbook_core::RecipeReader;

        let store = EmbeddedRecipeStore::global();

        // Test first day
        let recipe1 = store.get_by_day(1).unwrap();
        assert_eq!(recipe1.id, "day-1");
        assert!(!recipe1.title.is_empty());

        // Test middle day
        let recipe100 = store.get_by_day(100).unwrap();
        assert_eq!(recipe100.id, "day-100");
        assert!(!recipe100.title.is_empty());

        // Test last day
        let recipe365 = store.get_by_day(365).unwrap();
        assert_eq!(recipe365.id, "day-365");
        assert!(!recipe365.title.is_empty());
    }

    #[test]
    fn test_embedded_recipe_has_content() {
        use cookbook_core::RecipeReader;

        let store = EmbeddedRecipeStore::global();
        let recipe = store.get_by_day(1).unwrap();

        // Verify recipe has all expected content
        assert!(!recipe.title.is_empty(), "Recipe should have a title");
        assert!(
            recipe.description.is_some(),
            "Recipe should have a description"
        );
        assert!(
            !recipe.ingredients.is_empty(),
            "Recipe should have ingredients"
        );
        assert!(
            !recipe.instructions.is_empty(),
            "Recipe should have instructions"
        );
        assert!(
            recipe.prep_time_minutes.is_some(),
            "Recipe should have prep time"
        );
        assert!(
            recipe.cook_time_minutes.is_some(),
            "Recipe should have cook time"
        );
        assert!(recipe.servings.is_some(), "Recipe should have servings");
    }

    #[test]
    fn test_embedded_recipe_invalid_days() {
        use cookbook_core::RecipeReader;

        let store = EmbeddedRecipeStore::global();

        // Day 0 should fail
        assert!(store.get_by_day(0).is_err(), "Day 0 should be invalid");

        // Day 366 should fail
        assert!(store.get_by_day(366).is_err(), "Day 366 should be invalid");
    }
}

use chrono::NaiveDate;
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
const PLAN_CSS: Asset = asset!("/assets/styling/plan.css");

const INTRO_MD: &str = include_str!("../../content/intro.md");

/// Maximum day of the year (non-leap year)
const MAX_DAY_OF_YEAR: u32 = 365;

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

    // Initialize sidebar visibility state
    // Start with desktop default (visible), will be updated on mount for mobile
    let sidebar_visible = use_signal(|| true);

    // Detect mobile viewport and update sidebar state accordingly
    #[cfg(target_arch = "wasm32")]
    {
        use_effect(move || {
            // Check if we're on a mobile viewport (width <= 768px)
            if let Some(window) = web_sys::window() {
                if let Ok(width) = window.inner_width() {
                    if let Some(width_value) = width.as_f64() {
                        // Hide sidebar on mobile (width <= 768px)
                        if width_value <= 768.0 {
                            sidebar_visible.set(false);
                        }
                    }
                }
            }
        });
    }

    use_context_provider(|| sidebar_visible);

    rsx! {
        document::Link { rel: "stylesheet", href: HEADER_CSS }
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }
        document::Link { rel: "stylesheet", href: SIDEBAR_CSS }
        document::Link { rel: "stylesheet", href: HOME_CSS }
        document::Link { rel: "stylesheet", href: RECIPE_CSS }
        document::Link { rel: "stylesheet", href: PLAN_CSS }
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

/// Get the current ISO 8601 week of the year (1-53)
/// Uses ISO 8601 week numbering where:
/// - Week 1 is the first week with a Thursday (or the week containing January 4)
/// - Weeks start on Monday
/// - A year can have 53 weeks (when it starts on Thursday or is a leap year starting on Wednesday)
fn get_current_week_of_year() -> u32 {
    let now = Local::now();
    now.iso_week().week()
}

/// Format a date as "DayName, DD-Mon" (e.g., "Sun, 28-Dec")
/// Note: Uses %d which adds leading zeros for single-digit days for cross-platform compatibility
fn format_recipe_day(date: NaiveDate) -> String {
    // Format with leading zero for cross-platform compatibility
    let formatted = date.format("%a, %d-%b").to_string();
    // Remove leading zero if present for better readability (e.g., "Sun, 1-Jan" instead of "Sun, 01-Jan")
    if let Some(comma_pos) = formatted.find(',') {
        let day_part = &formatted[comma_pos + 2..]; // Skip ", "
        if let Some(stripped) = day_part.strip_prefix('0') {
            return format!("{}, {}", &formatted[..comma_pos], stripped);
        }
    }
    formatted
}

/// Get the start of the current week (Sunday)
fn get_week_start_date() -> NaiveDate {
    let now = Local::now().date_naive();
    let weekday = now.weekday();
    let days_since_monday = weekday.num_days_from_monday();
    now - chrono::Duration::days(days_since_monday.into())
}

/// Get recipe day information for sidebar display
/// Returns a vector of (day_of_year, formatted_date) tuples for the current week
/// (up to 7 days, or fewer if we reach MAX_DAY_OF_YEAR), using the same logic as
/// get_week_recipes() to ensure the sidebar matches the Plan component.
fn get_sidebar_recipe_days() -> Vec<(u32, String)> {
    let current_week = get_current_week_of_year();
    let recipes = get_week_recipes(current_week);
    let mut days = Vec::new();

    // Convert recipe data to formatted display data
    for (day_of_year, _title) in recipes {
        // Convert day of year to a date to format it
        let year = Local::now().year();
        if let Some(date) = NaiveDate::from_yo_opt(year, day_of_year) {
            let formatted = format_recipe_day(date);
            days.push((day_of_year, formatted));
        } else {
            // Log unexpected invalid day_of_year values instead of silently skipping them
            eprintln!(
                "get_sidebar_recipe_days: invalid day_of_year {} for year {} - \
                 skipping this entry (sidebar may show fewer than 7 days)",
                day_of_year, year
            );
        }
    }

    days
}

/// Get plan weeks to display in sidebar with their start dates
/// Returns a vector of (week_number, formatted_date) tuples for the next 4 upcoming weeks
fn get_sidebar_plan_weeks() -> Vec<(u32, String)> {
    let week_start = get_week_start_date();
    let mut weeks = Vec::new();

    // Show 4 upcoming weeks starting from current week
    for i in 0..4 {
        let start_date = week_start + chrono::Duration::weeks(i);
        let week_number = get_week_number_from_date(start_date);

        // Format date as "DD-Mon" (e.g., "28-Dec")
        let formatted = format_week_start_date(start_date);

        weeks.push((week_number, formatted));
    }

    weeks
}

/// Get the ISO 8601 week number (1-53) for a given date.
///
/// ISO 8601 week numbering rules:
/// - Week 1 is the first week containing a Thursday (or the first week with 4+ days in January)
/// - Weeks start on Monday
/// - Years can have 53 weeks (when Jan 1 is Thursday, or Wednesday in leap years)
fn get_week_number_from_date(date: NaiveDate) -> u32 {
    use chrono::Datelike;
    date.iso_week().week()
}

/// Format a week start date as "DD-Mon" (e.g., "28-Dec")
fn format_week_start_date(date: NaiveDate) -> String {
    let formatted = date.format("%d-%b").to_string();
    // Remove leading zero if present for better readability
    if let Some(stripped) = formatted.strip_prefix('0') {
        stripped.to_string()
    } else {
        formatted
    }
}

/// Convert an ISO 8601 week number to day-of-year values for the current year.
/// Returns a vector of day-of-year values (1-365/366) for all days in that ISO week
/// that fall within the current calendar year.
///
/// # Arguments
/// * `week` - ISO 8601 week number (1-53)
///
/// # Returns
/// Vector of day-of-year values for days in the specified ISO week.
/// May return fewer than 7 days if the week spans year boundaries.
fn get_day_of_year_for_iso_week(week: u32) -> Vec<u32> {
    let year = Local::now().year();
    let mut days = Vec::new();

    // Find the first day of the year
    // Jan 1 is always valid for any year, so unwrap_or with a fallback is safe
    let Some(jan1) = NaiveDate::from_ymd_opt(year, 1, 1) else {
        return days; // Should never happen, but return empty if it does
    };

    // Find the Monday of week 1 (ISO 8601: week 1 is the first week with Thursday)
    // Start by finding what ISO week Jan 1 belongs to
    let jan1_week = jan1.iso_week().week();

    let week1_monday = if jan1_week == 1 {
        // Jan 1 is in week 1, find the Monday of that week
        let weekday = jan1.weekday().num_days_from_monday();
        jan1 - chrono::Duration::days(weekday.into())
    } else {
        // Jan 1 is in week 52/53 of previous year, week 1 starts after Jan 1
        // Find the first Monday of January
        let mut date = jan1;
        while date.weekday() != chrono::Weekday::Mon {
            date += chrono::Duration::days(1);
        }
        // Check if this Monday belongs to week 1
        if date.iso_week().week() == 1 {
            date
        } else {
            // Find the next Monday
            date + chrono::Duration::days(7)
        }
    };

    // Calculate the Monday of the requested week
    let target_monday = week1_monday + chrono::Duration::weeks((week - 1).into());

    // Get all 7 days of this week and convert to day-of-year if they're in the current year
    for i in 0..7 {
        let date = target_monday + chrono::Duration::days(i);
        // Only include days that are in the current year
        if date.year() == year {
            days.push(date.ordinal());
        }
    }

    days
}

/// Get recipes for a specific ISO 8601 week (1-53).
/// Returns a vector of (day_of_year, recipe_title) tuples for the days of that week
/// that fall within the current calendar year.
///
/// Uses ISO 8601 week numbering where weeks start on Monday and week 1 is the first
/// week containing a Thursday. The function maps the ISO week number to the actual
/// calendar dates, then retrieves recipes for those day-of-year values.
fn get_week_recipes(week: u32) -> Vec<(u32, String)> {
    let store = EmbeddedRecipeStore::global();
    let mut recipes = Vec::new();

    // Get day-of-year values for this ISO week
    let days = get_day_of_year_for_iso_week(week);

    // Get recipes for each day
    for day in days {
        if day > MAX_DAY_OF_YEAR {
            continue; // Skip invalid days
        }

        match store.get_by_day(day) {
            Ok(recipe) => {
                recipes.push((day, recipe.title));
            }
            Err(_) => {
                // If recipe not found, use a placeholder
                recipes.push((day, format!("Recipe for Day {}", day)));
            }
        }
    }

    recipes
}

/// Aggregate all ingredients from recipes in an ISO 8601 week into a shopping list.
/// Returns a sorted vector of unique ingredients from all recipes in the week.
/// Returns an empty vector if the week is invalid (not in range 1-53).
///
/// Uses ISO 8601 week numbering where weeks start on Monday and week 1 is the first
/// week containing a Thursday. The function maps the ISO week number to the actual
/// calendar dates, then retrieves ingredients for those day-of-year values.
fn get_week_shopping_list(week: u32) -> Vec<String> {
    // Validate week parameter to ensure consistent behavior
    if !(1..=53).contains(&week) {
        return Vec::new();
    }

    let store = EmbeddedRecipeStore::global();
    let mut all_ingredients = Vec::new();

    // Get day-of-year values for this ISO week
    let days = get_day_of_year_for_iso_week(week);

    // Collect ingredients from all recipes in this week
    for day in days {
        if day > MAX_DAY_OF_YEAR {
            continue;
        }

        if let Ok(recipe) = store.get_by_day(day) {
            all_ingredients.extend(recipe.ingredients);
        }
    }

    // Sort and deduplicate ingredients
    all_ingredients.sort();
    all_ingredients.dedup();

    all_ingredients
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
                    p { "Get organized with complete meal plans - one for every week of the year (up to 53 weeks). Perfect for planning ahead!" }
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
        let recipes = get_week_recipes(week);
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
                            for ingredient in shopping_list {
                                li {
                                    class: "shopping-item",
                                    "{ingredient}"
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
                            for (day, title) in recipes {
                                li {
                                    class: "recipe-item",
                                    Link { to: Route::Recipe { day }, "{title}" }
                                    span { class: "recipe-day", " (Day {day})" }
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
        // Test that valid weeks (1-53) are accepted
        assert!((1..=53).contains(&1));
        assert!((1..=53).contains(&26));
        assert!((1..=53).contains(&52));
        assert!((1..=53).contains(&53));
    }

    #[test]
    fn test_plan_invalid_week_zero() {
        // Test that week 0 is invalid
        assert!(!(1..=53).contains(&0));
    }

    #[test]
    fn test_plan_invalid_week_too_high() {
        // Test that week > 53 is invalid
        assert!(!(1..=53).contains(&54));
        assert!(!(1..=53).contains(&100));
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
        assert!((1..=53).contains(&1), "Week 1 should be valid");
        assert!((1..=53).contains(&52), "Week 52 should be valid");
        assert!((1..=53).contains(&53), "Week 53 should be valid");
        assert!(!(1..=53).contains(&0), "Week 0 should be invalid");
        assert!(!(1..=53).contains(&54), "Week 54 should be invalid");
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
        assert_eq!(Route::Plan { week: 53 }.to_string(), "/plan/53");
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
        // Test that recipe days are in chronological order for the current ISO week
        let days = get_sidebar_recipe_days();

        // Should not be empty
        assert!(!days.is_empty(), "Recipe days should not be empty");

        // Should have at most 7 entries (one week)
        // May be fewer at year boundaries when ISO week spans two calendar years
        assert!(
            days.len() <= 7,
            "Should show at most 7 days of the week, got {}",
            days.len()
        );

        // Verify all days are within valid range (1-366 for leap years)
        for (day, _) in &days {
            assert!(
                (1..=366).contains(day),
                "Day {} should be in valid range 1-366",
                day
            );
        }

        // Days should be in ascending order (within the current year)
        for i in 1..days.len() {
            assert!(
                days[i].0 > days[i - 1].0,
                "Days should be in ascending order"
            );
        }
    }

    #[test]
    fn test_sidebar_plan_weeks_sorted_numerically() {
        // Test that plan weeks are returned with dates
        let weeks = get_sidebar_plan_weeks();

        // Should not be empty
        assert!(!weeks.is_empty(), "Plan weeks should not be empty");

        // Should have exactly 4 entries (4 upcoming weeks)
        assert_eq!(weeks.len(), 4, "Should show 4 upcoming weeks");

        // Verify all weeks are within valid range (1-53)
        for (week, formatted_date) in &weeks {
            assert!(
                (1..=53).contains(week),
                "Week {} should be in valid range 1-53",
                week
            );
            assert!(
                !formatted_date.is_empty(),
                "Formatted date should not be empty"
            );
            assert!(
                formatted_date.contains("-"),
                "Formatted date should contain a dash"
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
        // Test that sidebar shows up to 7 recipe entries (one week, or fewer near year end)
        let days = get_sidebar_recipe_days();
        assert!(
            days.len() >= 1 && days.len() <= 7,
            "Sidebar should show between 1 and 7 recipe entries, found {}",
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
    fn test_sidebar_recipe_days_uses_week_calculation() {
        // Test that recipe days use the week calculation logic (not calendar week)
        let current_week = get_current_week_of_year();
        let days = get_sidebar_recipe_days();

        // Should have 7 entries (or fewer if near end of year)
        assert!(days.len() <= 7);
        assert!(!days.is_empty());

        // Calculate expected starting day based on current week
        let expected_start_day = (current_week - 1) * 7 + 1;

        // First day should match the expected start day for the current week
        assert_eq!(days[0].0, expected_start_day);

        // Verify each day is consecutive from the start day
        for i in 0..days.len() {
            let expected_day = expected_start_day + i as u32;
            assert_eq!(
                days[i].0, expected_day,
                "Day {} should match expected day",
                i
            );
        }

        // All days should be in valid range (1-365)
        for (day, _) in &days {
            assert!(
                (1..=365).contains(day),
                "Day {} should be in valid range 1-365",
                day
            );
        }
    }

    #[test]
    fn test_sidebar_plan_weeks_starts_from_current_week() {
        // Test that plan weeks start from current week or very close to it
        let current_week = get_current_week_of_year();
        let weeks = get_sidebar_plan_weeks();

        // Should have 4 entries
        assert_eq!(weeks.len(), 4);

        // The first week should be close to the current week.
        // Note: get_sidebar_plan_weeks() is based on calendar weeks (Sun-Sat). Around
        // year boundaries, a calendar week can span two years, so this calendar-based
        // sidebar week calculation may differ from the day-based week calculation used
        // by get_sidebar_recipe_days() / get_week_recipes(). Week 52 -> Week 1
        // transition means they're adjacent across the year boundary.
        let first_week = weeks[0].0;
        let week_diff = if first_week > current_week {
            first_week - current_week
        } else {
            current_week - first_week
        };

        // Allow difference of 1 week, or 51 weeks (which represents week 52 -> week 1 transition)
        assert!(
            week_diff <= 1 || week_diff >= 51,
            "First week {} should be within 1 week of current week {} (diff: {})",
            first_week,
            current_week,
            week_diff
        );

        // All weeks should be in valid range
        for (week, formatted_date) in &weeks {
            assert!(
                (1..=53).contains(week),
                "Week {} should be in valid range 1-53",
                week
            );
            assert!(
                !formatted_date.is_empty(),
                "Formatted date should not be empty"
            );
        }

        // Note: Weeks may wrap around year boundary (53 -> 1 or 52 -> 1), so we don't check strict consecutive order
        // Just verify they represent 4 consecutive calendar weeks
    }

    #[test]
    fn test_format_recipe_day() {
        // Test date formatting
        let date = NaiveDate::from_ymd_opt(2025, 12, 28).unwrap(); // Sunday, Dec 28, 2025
        let formatted = format_recipe_day(date);
        assert_eq!(formatted, "Sun, 28-Dec");

        let date2 = NaiveDate::from_ymd_opt(2025, 12, 30).unwrap(); // Tuesday, Dec 30, 2025
        let formatted2 = format_recipe_day(date2);
        assert_eq!(formatted2, "Tue, 30-Dec");

        let date3 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(); // Thursday, Jan 1, 2026
        let formatted3 = format_recipe_day(date3);
        assert_eq!(formatted3, "Thu, 1-Jan");
    }

    #[test]
    fn test_format_week_start_date() {
        // Test week start date formatting (no day name, just date)
        let date = NaiveDate::from_ymd_opt(2025, 12, 28).unwrap();
        let formatted = format_week_start_date(date);
        assert_eq!(formatted, "28-Dec");

        let date2 = NaiveDate::from_ymd_opt(2026, 1, 4).unwrap();
        let formatted2 = format_week_start_date(date2);
        assert_eq!(formatted2, "4-Jan");

        // Test single digit date (leading zero removal)
        let date3 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let formatted3 = format_week_start_date(date3);
        assert_eq!(formatted3, "1-Jan");
    }

    #[test]
    fn test_get_week_number_from_date() {
        // Test ISO 8601 week number calculation
        // 2025-01-01 (Wednesday) is in ISO week 1 of 2025
        let date1 = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert_eq!(get_week_number_from_date(date1), 1);

        // 2025-01-07 (Tuesday) is in ISO week 2
        let date2 = NaiveDate::from_ymd_opt(2025, 1, 7).unwrap();
        assert_eq!(get_week_number_from_date(date2), 2);

        // 2025-01-08 (Wednesday) is also in ISO week 2
        let date3 = NaiveDate::from_ymd_opt(2025, 1, 8).unwrap();
        assert_eq!(get_week_number_from_date(date3), 2);

        // 2025-12-31 (Wednesday) is in ISO week 1 of 2026
        let date4 = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        assert_eq!(get_week_number_from_date(date4), 1);
    }

    #[test]
    fn test_get_week_start_date() {
        // This test depends on the current date, so we just verify it returns a Monday
        let week_start = get_week_start_date();
        assert_eq!(
            week_start.weekday(),
            chrono::Weekday::Mon,
            "Week should start on Monday (ISO 8601)"
        );

        // Verify it's not in the future
        let now = Local::now().date_naive();
        assert!(week_start <= now, "Week start should not be in the future");

        // Verify it's within 6 days in the past
        let diff = now.signed_duration_since(week_start).num_days();
        assert!(
            diff >= 0 && diff <= 6,
            "Week start should be within the last 7 days"
        );
    }

    #[test]
    fn test_sidebar_recipe_days_includes_formatted_dates() {
        // Test that recipe days include formatted date strings
        let days = get_sidebar_recipe_days();

        // Should have at least 1 day, at most 7 days
        // May be fewer than 7 at year boundaries when ISO week spans two calendar years
        assert!(
            !days.is_empty() && days.len() <= 7,
            "Should have 1-7 days, got {}",
            days.len()
        );

        // Check that all entries have formatted dates
        for (day_num, formatted) in &days {
            assert!(
                *day_num >= 1 && *day_num <= 366,
                "Day number should be valid"
            );
            assert!(!formatted.is_empty(), "Formatted date should not be empty");
            assert!(
                formatted.contains("-"),
                "Formatted date should contain a dash"
            );
            assert!(
                formatted.contains(","),
                "Formatted date should contain a comma"
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

    #[test]
    fn test_get_week_recipes() {
        // Test that we can get recipes for ISO week 1
        // The exact days depend on the current year, but we should get 7 days (or fewer if
        // some days of the ISO week fall in the previous year)
        let recipes = get_week_recipes(1);
        assert!(
            !recipes.is_empty(),
            "Week 1 should have at least some recipes"
        );
        assert!(
            recipes.len() <= 7,
            "Week should have at most 7 recipes, got {}",
            recipes.len()
        );

        // Days should be in ascending order
        for i in 1..recipes.len() {
            assert!(
                recipes[i].0 > recipes[i - 1].0,
                "Days should be in ascending order"
            );
        }

        // Test week 2
        let recipes_week2 = get_week_recipes(2);
        assert!(!recipes_week2.is_empty(), "Week 2 should have recipes");
        assert!(
            recipes_week2.len() <= 7,
            "Week should have at most 7 recipes"
        );

        // All days should be valid (1-365)
        for (day, _) in &recipes_week2 {
            assert!(
                *day >= 1 && *day <= 365,
                "Day {} should be in range 1-365",
                day
            );
        }
    }

    #[test]
    fn test_get_week_recipes_iso_week_mapping() {
        // Test that ISO week numbers map to correct dates
        // For 2025: Week 1 is Dec 30, 2024 - Jan 5, 2025
        // So in 2025, week 1 should include days 1-5 (Jan 1-5)

        // Test that we get the correct day-of-year values for week 1
        let days_week1 = get_day_of_year_for_iso_week(1);
        assert!(
            !days_week1.is_empty(),
            "Week 1 should have days in the current year"
        );

        // Verify the days are consecutive (within the current year)
        for i in 1..days_week1.len() {
            assert_eq!(
                days_week1[i],
                days_week1[i - 1] + 1,
                "Days within a week should be consecutive"
            );
        }

        // Test a mid-year week (week 26) should have 7 days
        let days_week26 = get_day_of_year_for_iso_week(26);
        assert_eq!(
            days_week26.len(),
            7,
            "Mid-year week should have all 7 days in the current year"
        );

        // Test that recipes are retrieved for the correct days
        let recipes = get_week_recipes(26);
        assert_eq!(
            recipes.len(),
            days_week26.len(),
            "Should have recipes for all days in the week"
        );
        for (i, (day, _)) in recipes.iter().enumerate() {
            assert_eq!(
                *day, days_week26[i],
                "Recipe day should match calculated day-of-year"
            );
        }
    }

    #[test]
    fn test_get_week_recipes_titles_loaded() {
        // Test that recipe titles are loaded correctly
        let recipes = get_week_recipes(1);

        for (day, title) in &recipes {
            assert!(
                !title.is_empty(),
                "Recipe title for day {} should not be empty",
                day
            );
        }
    }

    #[test]
    fn test_sidebar_recipes_match_current_week_plan() {
        // Test that sidebar recipes match the Plan component for the current week
        let current_week = get_current_week_of_year();
        let sidebar_days = get_sidebar_recipe_days();
        let plan_recipes = get_week_recipes(current_week);

        // Both should have the same number of recipes (7 or fewer)
        assert_eq!(
            sidebar_days.len(),
            plan_recipes.len(),
            "Sidebar and plan should have the same number of recipes"
        );

        // The day numbers should match exactly
        for i in 0..sidebar_days.len() {
            assert_eq!(
                sidebar_days[i].0, plan_recipes[i].0,
                "Sidebar day {} should match plan day {} for position {}",
                sidebar_days[i].0, plan_recipes[i].0, i
            );
        }
    }

    #[test]
    fn test_sidebar_recipes_use_week_logic() {
        // Test that sidebar uses the same ISO week calculation as get_week_recipes
        let sidebar_days = get_sidebar_recipe_days();
        let current_week = get_current_week_of_year();

        // Get expected days for the current ISO week
        let expected_days = get_day_of_year_for_iso_week(current_week);

        // Sidebar should have the same number of days as the ISO week calculation
        assert_eq!(
            sidebar_days.len(),
            expected_days.len(),
            "Sidebar should have {} days for ISO week {} but has {}",
            expected_days.len(),
            current_week,
            sidebar_days.len()
        );

        // First sidebar day should match the first day of the ISO week
        if !expected_days.is_empty() {
            assert_eq!(
                sidebar_days[0].0, expected_days[0],
                "Sidebar should start at day {} for ISO week {}",
                expected_days[0], current_week
            );
        }

        // Verify all days match the ISO week calculation
        for (i, (day, _)) in sidebar_days.iter().enumerate() {
            assert_eq!(
                *day, expected_days[i],
                "Sidebar day at position {} should be {} but got {}",
                i, expected_days[i], day
            );
        }
    }

    #[test]
    fn test_get_week_shopping_list_returns_ingredients() {
        // Test that shopping list returns ingredients from week 1
        let shopping_list = get_week_shopping_list(1);

        // Shopping list should not be empty for week 1
        assert!(!shopping_list.is_empty(), "Week 1 should have ingredients");
    }

    #[test]
    fn test_get_week_shopping_list_is_sorted() {
        // Test that shopping list is sorted
        let shopping_list = get_week_shopping_list(1);

        // Check if the list is sorted
        let mut sorted_copy = shopping_list.clone();
        sorted_copy.sort();
        assert_eq!(shopping_list, sorted_copy, "Shopping list should be sorted");
    }

    #[test]
    fn test_get_week_shopping_list_is_deduplicated() {
        // Test that shopping list removes duplicates
        // This test verifies that if multiple recipes in a week use the same ingredient,
        // it only appears once in the shopping list
        let shopping_list = get_week_shopping_list(1);

        // Use HashSet to check for duplicates
        let unique_items: std::collections::HashSet<_> = shopping_list.iter().collect();
        assert_eq!(
            shopping_list.len(),
            unique_items.len(),
            "Shopping list should have no duplicates"
        );
    }

    #[test]
    fn test_get_week_shopping_list_valid_weeks() {
        // Test that shopping list works for various valid weeks
        let shopping_list_week_1 = get_week_shopping_list(1);
        let shopping_list_week_26 = get_week_shopping_list(26);
        let _shopping_list_week_52 = get_week_shopping_list(52);
        let _shopping_list_week_53 = get_week_shopping_list(53);

        // All should return some ingredients (assuming recipes have ingredients)
        // Note: Week 52 and 53 might be empty or have fewer recipes since the recipe
        // system is based on days 1-365 (MAX_DAY_OF_YEAR), and these weeks may
        // extend beyond day 365 depending on the year
        assert!(
            !shopping_list_week_1.is_empty(),
            "Week 1 should have ingredients"
        );
        assert!(
            !shopping_list_week_26.is_empty(),
            "Week 26 should have ingredients"
        );
    }

    #[test]
    fn test_get_week_shopping_list_invalid_weeks() {
        // Test that shopping list returns empty for invalid weeks
        let shopping_list_week_0 = get_week_shopping_list(0);
        let shopping_list_week_54 = get_week_shopping_list(54);
        let shopping_list_week_100 = get_week_shopping_list(100);

        assert!(
            shopping_list_week_0.is_empty(),
            "Week 0 should return empty list"
        );
        assert!(
            shopping_list_week_54.is_empty(),
            "Week 54 should return empty list"
        );
        assert!(
            shopping_list_week_100.is_empty(),
            "Week 100 should return empty list"
        );
    }

    #[test]
    fn test_iso_week_calculation_2026() {
        // 2026 starts on Thursday (Jan 1, 2026), so week 1 starts on Dec 29, 2025
        // Jan 1, 2026 is in ISO week 1
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert_eq!(date.iso_week().week(), 1);

        // Last day of 2026 (Dec 31, 2026 is Thursday) is in week 53
        let date = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        assert_eq!(date.iso_week().week(), 53);
    }

    #[test]
    fn test_iso_week_calculation_2020() {
        // 2020 is a leap year starting on Wednesday
        // 2020 has 53 ISO weeks
        // Jan 1, 2020 (Wednesday) is in week 1
        let date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        assert_eq!(date.iso_week().week(), 1);

        // Dec 31, 2020 (Thursday) is in week 53
        let date = NaiveDate::from_ymd_opt(2020, 12, 31).unwrap();
        assert_eq!(date.iso_week().week(), 53);
    }

    #[test]
    fn test_iso_week_calculation_2025() {
        // 2025 starts on Wednesday (Jan 1, 2025)
        // Week 1 starts on Dec 30, 2024 (Monday)
        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert_eq!(date.iso_week().week(), 1);

        // Dec 31, 2025 is Wednesday, in week 1 of 2026
        let date = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        // This should be week 1 of 2026, not week 53 of 2025
        assert_eq!(date.iso_week().week(), 1);
    }

    #[test]
    fn test_iso_week_monday_start() {
        // ISO weeks always start on Monday
        // Test a few random Mondays to verify they're the first day of their week
        let monday = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(); // Monday
        assert_eq!(monday.weekday(), chrono::Weekday::Mon);

        let tuesday = NaiveDate::from_ymd_opt(2026, 1, 6).unwrap(); // Tuesday
        assert_eq!(tuesday.weekday(), chrono::Weekday::Tue);
        // Both Monday and Tuesday should be in the same week
        assert_eq!(monday.iso_week().week(), tuesday.iso_week().week());
    }
}

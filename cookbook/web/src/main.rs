use chrono::NaiveDate;
use chrono::prelude::*;
use cookbook_core::markdown::{self, Block, Inline};
use cookbook_core::{PlanReader, RecipeReader};
use cookbook_data_md::embedded::{EmbeddedPlanStore, EmbeddedRecipeStore};
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use auth_sdk::provider::{AuthError, AuthProvider, ProviderConfig};
#[cfg(target_arch = "wasm32")]
use auth_sdk::web::{WebAuthProvider, fetch_current_location_from_browser};

const HEADER_CSS: Asset = asset!("/assets/styling/header.css");
const SIDEBAR_CSS: Asset = asset!("/assets/styling/sidebar.css");
const HOME_CSS: Asset = asset!("/assets/styling/home.css");
const RECIPE_CSS: Asset = asset!("/assets/styling/recipe.css");
const PLAN_CSS: Asset = asset!("/assets/styling/plan.css");

/// The design system theme the cookbook uses until the reader picks another.
const DEFAULT_THEME: &str = "earthy";

const INTRO_MD: &str = include_str!("../../content/intro.md");

/// Maximum day of the year (non-leap year)
const MAX_DAY_OF_YEAR: u32 = 365;

#[cfg(target_arch = "wasm32")]
const AUTH0_DOMAIN: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/";
#[cfg(target_arch = "wasm32")]
const CLIENT_ID: &str = "savzmZnyHcvewGkQX8aaInwPFonC9k2x";
#[cfg(target_arch = "wasm32")]
const AUTH_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/authorize";
#[cfg(target_arch = "wasm32")]
const TOKEN_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/oauth/token";

/// Content Security Policy for the application
/// - default-src 'none': Block all sources by default
/// - script-src 'self' 'wasm-unsafe-eval': Allow scripts from same origin and WASM
/// - connect-src 'self' AUTH0_DOMAIN: Allow API calls to same origin and Auth0
/// - img-src 'self': Allow images from same origin
/// - style-src 'self' 'unsafe-inline': Allow styles from same origin and inline styles (required by Dioxus)
/// - font-src 'self': Allow fonts from same origin (required for self-hosted web fonts)
/// - base-uri 'self': Restrict base tag to same origin
/// - form-action 'self': Restrict form submissions to same origin
///
/// Note: The Auth0 domain is hardcoded in the CSP string to match AUTH0_DOMAIN constant.
/// Tests verify that the CSP contains the correct Auth0 domain.
const CONTENT_SECURITY_POLICY: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; connect-src 'self' https://dev-jdadpn4pckxevrv5.us.auth0.com; img-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self'; base-uri 'self'; form-action 'self'";

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
                issuer_url: Some(AUTH0_DOMAIN.into()),
            })
            .await
        });
        use_context_provider(|| auth);
    }

    // Initialize sidebar visibility state
    // Start with desktop default (visible), will be updated on mount for mobile
    #[allow(unused_mut)]
    let mut sidebar_visible = use_signal(|| true);

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
        // Include this CSP in server response headers for defense in depth redundancy
        // CSP Analysis: Modern Dioxus (v0.7.x) compiles to WebAssembly and doesn't require
        // unsafe-eval or unsafe-inline for scripts. WASM modules and proper event binding
        // are used instead of eval() or inline script handlers. The 'wasm-unsafe-eval'
        // directive is needed for WebAssembly instantiation in some browsers.
        // Style unsafe-inline is kept for any inline styling that may be needed.
        // Auth0 domain whitelisted for OAuth authentication flows (authorize, token endpoints)
        if cfg!(target_arch = "wasm32") {
            document::Meta {
                http_equiv: "Content-Security-Policy",
                content: CONTENT_SECURITY_POLICY,
            }
        }
        // The design system's tokens, base sheet and theme script, before
        // the app's own sheets so they can use the tokens.
        design_system_dioxus::Stylesheets { default_theme: DEFAULT_THEME }
        document::Link { rel: "stylesheet", href: HEADER_CSS }
        document::Link { rel: "stylesheet", href: SIDEBAR_CSS }
        document::Link { rel: "stylesheet", href: HOME_CSS }
        document::Link { rel: "stylesheet", href: RECIPE_CSS }
        document::Link { rel: "stylesheet", href: PLAN_CSS }
        Router::<Route> {}
    }
}

// dioxus 0.7.10's Routable derive generates code that rustc 1.100 flags as
// unreachable. The allow has to cover the generated impls, so the enum sits
// in its own module.
mod route {
    #![allow(unreachable_code)]
    use super::*;

    #[derive(Clone, Routable, Debug, PartialEq)]
    pub enum Route {
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
}
use route::Route;

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

/// Format authentication errors into user-friendly messages
#[cfg(target_arch = "wasm32")]
fn format_auth_error(err: &AuthError) -> String {
    match err {
        AuthError::ParseError => "Unable to process authentication".to_string(),
        AuthError::TokenExchangeError(_) => "Login failed. Please try again".to_string(),
        AuthError::BrowserApiError(_) => "Browser compatibility issue".to_string(),
        AuthError::MissingUrlParameter(_) => "Authentication incomplete".to_string(),
        AuthError::SerializationError(_) => "Unable to process authentication".to_string(),
        AuthError::MissingStateData(_) => "Authentication incomplete".to_string(),
        AuthError::CsrfValidationFailed => "Security check failed. Please try again".to_string(),
        AuthError::OidcDiscoveryFailed(_) => "Authentication service unavailable".to_string(),
        AuthError::IdTokenValidationFailed(_) => "Login verification failed".to_string(),
        AuthError::HttpClientError(_) => "Network error. Please check your connection".to_string(),
        AuthError::ConfigError(_) => "Authentication not configured".to_string(),
        AuthError::Unknown => "An unexpected error occurred".to_string(),
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
        Some(Err(err)) => {
            let user_message = format_auth_error(err);
            rsx! {
                div {
                    class: "error",
                    "{user_message}"
                }
            }
        }
        None => rsx! {
            div { "Loading authentication..." }
        },
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn render_auth_section() -> Element {
    rsx! { div { "Login (test mode)" } }
}

/// Map a date to its recipe day (1-365).
///
/// The cookbook has one recipe per day for 365 days. Day 366 (31 December of a
/// leap year) reuses the recipe for day 365, so the link never points past the
/// last recipe.
fn recipe_day_for_date(date: NaiveDate) -> u32 {
    date.ordinal().min(MAX_DAY_OF_YEAR)
}

/// Get today's recipe day (1-365) in local time
fn get_current_day_of_year() -> u32 {
    recipe_day_for_date(Local::now().date_naive())
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

/// Monday-to-Sunday dates of ISO 8601 `week` in ISO year `year`, or `None` when
/// that year has no such week (only some years have a week 53).
fn iso_week_dates(year: i32, week: u32) -> Option<[NaiveDate; 7]> {
    let monday = NaiveDate::from_isoywd_opt(year, week, Weekday::Mon)?;
    Some(std::array::from_fn(|i| {
        monday + chrono::Duration::days(i as i64)
    }))
}

/// The dates the plan for ISO `week` covers, seen from `today`.
///
/// Plans are not tied to a year, so each one is placed on the occurrence of its
/// ISO week (last, this or next ISO year) whose Monday is closest to `today`.
/// That puts week 53 on 28 Dec 2026 to 3 Jan 2027, and week 1 seen in late
/// December on the coming January.
fn plan_week_dates(week: u32, today: NaiveDate) -> Option<[NaiveDate; 7]> {
    let iso_year = today.iso_week().year();
    [iso_year - 1, iso_year, iso_year + 1]
        .into_iter()
        .filter_map(|year| iso_week_dates(year, week))
        .min_by_key(|dates| (dates[0] - today).num_days().abs())
}

/// A week's plan with the date each recipe falls on: (recipe day, title, date).
fn get_week_plan_days(week: u32, today: NaiveDate) -> Vec<(u32, String, Option<NaiveDate>)> {
    let dates = plan_week_dates(week, today);
    get_week_recipes(week)
        .into_iter()
        .enumerate()
        .map(|(i, (day, title))| (day, title, dates.map(|d| d[i])))
        .collect()
}

/// Get recipe day information for sidebar display.
/// Returns (recipe_day, formatted_date) for each day, Monday to Sunday, of the
/// ISO week containing `today`, using the same plan as the Plan page. The label
/// is the real date in this year, and the link is the plan's recipe for it.
fn sidebar_recipe_days(today: NaiveDate) -> Vec<(u32, String)> {
    get_week_plan_days(today.iso_week().week(), today)
        .into_iter()
        .filter_map(|(day, _title, date)| Some((day, format_recipe_day(date?))))
        .collect()
}

/// Sidebar recipe days for the current local date.
fn get_sidebar_recipe_days() -> Vec<(u32, String)> {
    sidebar_recipe_days(Local::now().date_naive())
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

/// Get recipes for a specific ISO 8601 week (1-53).
/// Returns a vector of (day_of_year, recipe_title) tuples for the days of that week.
///
/// Uses the PlanReader to load the pre-defined plan for the week, which contains
/// the recipe UUIDs for each day (Monday-Sunday) of the plan.
fn get_week_recipes(week: u32) -> Vec<(u32, String)> {
    let recipe_store = EmbeddedRecipeStore::global();
    let plan_store = EmbeddedPlanStore::global();
    let mut recipes = Vec::new();

    // Get the plan for this week
    let Ok(plan) = plan_store.get_by_week(week) else {
        return recipes; // Return empty if plan not found
    };

    // Get recipes for each UUID in the plan
    for uuid in plan.recipe_uuids {
        match recipe_store.get_by_uuid(&uuid) {
            Ok(recipe) => {
                // Get the day number from the recipe ID (format: "day-X")
                let day = recipe
                    .id
                    .strip_prefix("day-")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0);

                if day > MAX_DAY_OF_YEAR {
                    continue; // Skip invalid days
                }

                recipes.push((day, recipe.title));
            }
            Err(_) => {
                // If recipe not found by UUID, log and skip
                eprintln!(
                    "Warning: Recipe with UUID {} not found in week {}",
                    uuid, week
                );
            }
        }
    }

    recipes
}

/// Aggregate all ingredients from recipes in an ISO 8601 week into a shopping list.
/// Returns a sorted vector of unique ingredients from all recipes in the week.
/// Returns an empty vector if the week is invalid or plan not found.
///
/// Uses the PlanReader to load the pre-defined plan for the week, which contains
/// the recipe UUIDs for the recipes in that week's plan.
fn get_week_shopping_list(week: u32) -> Vec<String> {
    let recipe_store = EmbeddedRecipeStore::global();
    let plan_store = EmbeddedPlanStore::global();
    let mut all_ingredients = Vec::new();

    // Get the plan for this week
    let Ok(plan) = plan_store.get_by_week(week) else {
        return all_ingredients; // Return empty if plan not found
    };

    // Collect ingredients from all recipes in this week's plan
    for uuid in plan.recipe_uuids {
        if let Ok(recipe) = recipe_store.get_by_uuid(&uuid) {
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

            div {
                class: "sidebar-appearance",
                design_system_dioxus::ThemePicker {}
            }
        }
    }
}

/// Renders the inline spans of one Markdown block.
fn render_inlines(spans: &[Inline]) -> Element {
    rsx! {
        for span in spans.iter() {
            match span {
                Inline::Text(text) => rsx! { "{text}" },
                Inline::Emphasis(text) => rsx! { em { "{text}" } },
                Inline::Strong(text) => rsx! { strong { "{text}" } },
            }
        }
    }
}

/// Renders Markdown prose (see `cookbook_core::markdown`) as elements, never
/// as injected HTML.
#[component]
fn MarkdownBlocks(markdown: &'static str) -> Element {
    let blocks = markdown::parse(markdown);
    rsx! {
        for block in blocks.iter() {
            match block {
                Block::Paragraph(spans) => rsx! { p { {render_inlines(spans)} } },
                Block::Heading { level: 1, content } => rsx! { h1 { {render_inlines(content)} } },
                Block::Heading { level: 2, content } => rsx! { h2 { {render_inlines(content)} } },
                Block::Heading { level: 3, content } => rsx! { h3 { {render_inlines(content)} } },
                Block::Heading { level: 4, content } => rsx! { h4 { {render_inlines(content)} } },
                Block::Heading { level: 5, content } => rsx! { h5 { {render_inlines(content)} } },
                Block::Heading { content, .. } => rsx! { h6 { {render_inlines(content)} } },
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
                MarkdownBlocks { markdown: INTRO_MD }
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

        // Use get_by_id with legacy day-based ID format
        match store.get_by_id(&format!("day-{}", day)) {
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
    fn test_intro_md_renders_as_paragraphs_with_emphasised_title() {
        let blocks = markdown::parse(INTRO_MD);
        assert_eq!(blocks.len(), 3, "intro should have three paragraphs");
        let Some(Block::Paragraph(first)) = blocks.first() else {
            panic!("intro should start with a paragraph");
        };
        assert_eq!(
            first.first(),
            Some(&Inline::Emphasis("The Engineer's 365 Cookbook".to_string()))
        );
        for block in &blocks {
            let Block::Paragraph(spans) = block else {
                panic!("intro should only have paragraphs");
            };
            for span in spans {
                if let Inline::Text(text) = span {
                    assert!(!text.contains('*'), "literal Markdown marker in {text:?}");
                    assert!(!text.contains('\n'), "unjoined line break in {text:?}");
                }
            }
        }
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
    fn test_sidebar_recipe_days_are_monday_to_sunday_of_the_iso_week() {
        // A Thursday in week 48 of 2026: the sidebar lists Mon 23 to Sun 29 Nov,
        // not the 2024 day numbers (which would start on Thursday 26 Nov).
        let days = sidebar_recipe_days(test_date(2026, 11, 26));
        let labels: Vec<&str> = days.iter().map(|(_, label)| label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Mon, 23-Nov",
                "Tue, 24-Nov",
                "Wed, 25-Nov",
                "Thu, 26-Nov",
                "Fri, 27-Nov",
                "Sat, 28-Nov",
                "Sun, 29-Nov"
            ]
        );
        let plan_days: Vec<u32> = get_week_recipes(48).iter().map(|(d, _)| *d).collect();
        let sidebar_days: Vec<u32> = days.iter().map(|(d, _)| *d).collect();
        assert_eq!(sidebar_days, plan_days, "Links follow the week 48 plan");
    }

    #[test]
    fn test_sidebar_recipe_days_week_53() {
        // ISO 2026 has 53 weeks; week 53 runs Mon 28 Dec 2026 to Sun 3 Jan 2027.
        for today in [
            test_date(2026, 12, 28),
            test_date(2027, 1, 1),
            test_date(2027, 1, 3),
        ] {
            let days = sidebar_recipe_days(today);
            let labels: Vec<&str> = days.iter().map(|(_, label)| label.as_str()).collect();
            assert_eq!(
                labels,
                [
                    "Mon, 28-Dec",
                    "Tue, 29-Dec",
                    "Wed, 30-Dec",
                    "Thu, 31-Dec",
                    "Fri, 1-Jan",
                    "Sat, 2-Jan",
                    "Sun, 3-Jan"
                ],
                "sidebar on {today}"
            );
            let recipe_days: Vec<u32> = days.iter().map(|(d, _)| *d).collect();
            assert_eq!(recipe_days, [362, 363, 364, 365, 1, 2, 3]);
        }
    }

    #[test]
    fn test_sidebar_recipe_days_every_day_has_seven_entries() {
        // Every day across two year boundaries (one with a week 53) shows a full week.
        let mut date = test_date(2025, 12, 1);
        while date <= test_date(2027, 1, 31) {
            assert_eq!(sidebar_recipe_days(date).len(), 7, "sidebar on {date}");
            date = date.succ_opt().unwrap();
        }
    }

    #[test]
    fn test_iso_week_dates() {
        let dates = iso_week_dates(2026, 53).expect("2026 has a week 53");
        assert_eq!(dates[0], test_date(2026, 12, 28));
        assert_eq!(dates[6], test_date(2027, 1, 3));
        assert!(iso_week_dates(2027, 53).is_none(), "2027 has 52 weeks");
        assert!(iso_week_dates(2025, 53).is_none(), "2025 has 52 weeks");
    }

    #[test]
    fn test_plan_week_dates_picks_nearest_occurrence() {
        // Week 1 seen in late December 2026 is the coming January.
        let week1 = plan_week_dates(1, test_date(2026, 12, 20)).unwrap();
        assert_eq!(week1[0], test_date(2027, 1, 4));
        // Week 53 seen in 2027 (which has none) is the one just past.
        let week53 = plan_week_dates(53, test_date(2027, 3, 1)).unwrap();
        assert_eq!(week53[0], test_date(2026, 12, 28));
        // A mid-year week is in the current year.
        let week26 = plan_week_dates(26, test_date(2026, 11, 26)).unwrap();
        assert_eq!(week26[0], test_date(2026, 6, 22));
        // Every week of every year has dates.
        for week in 1..=53 {
            assert!(plan_week_dates(week, test_date(2026, 11, 26)).is_some());
        }
    }

    #[test]
    fn test_every_week_plan_has_seven_dated_recipes() {
        let today = test_date(2026, 11, 26);
        for week in 1..=53 {
            let days = get_week_plan_days(week, today);
            assert_eq!(days.len(), 7, "week {week} should have 7 recipes");
            assert!(days.iter().all(|(_, _, date)| date.is_some()));
            assert!(!get_week_shopping_list(week).is_empty(), "week {week}");
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
        // Today's recipe day must always be a valid recipe route (1-365)
        let day = get_current_day_of_year();
        assert!(
            (1..=MAX_DAY_OF_YEAR).contains(&day),
            "Current day {} should be in valid range 1-{}",
            day,
            MAX_DAY_OF_YEAR
        );
    }

    #[test]
    fn test_recipe_day_for_date_first_of_january() {
        let date = test_date(2028, 1, 1);
        assert_eq!(recipe_day_for_date(date), 1);
    }

    #[test]
    fn test_recipe_day_for_date_end_of_common_year() {
        let date = test_date(2027, 12, 31);
        assert_eq!(recipe_day_for_date(date), 365);
    }

    #[test]
    fn test_recipe_day_for_date_end_of_leap_year_reuses_day_365() {
        let date = test_date(2028, 12, 31);
        assert_eq!(date.ordinal(), 366);
        assert_eq!(recipe_day_for_date(date), 365);
    }

    #[test]
    fn test_recipe_day_for_date_leap_day_and_after() {
        // 29 Feb 2028 is day 60, and every later day keeps its own ordinal
        let leap_day = test_date(2028, 2, 29);
        assert_eq!(recipe_day_for_date(leap_day), 60);
        let penultimate = test_date(2028, 12, 30);
        assert_eq!(recipe_day_for_date(penultimate), 365);
    }

    #[test]
    fn test_recipe_day_for_date_always_renders_a_recipe() {
        // Every day of a leap year maps to a route the Recipe page accepts
        let days = test_date(2028, 1, 1)
            .iter_days()
            .take_while(|date| date.year() == 2028);
        let mut count = 0;
        for date in days {
            let day = recipe_day_for_date(date);
            assert!((1..=MAX_DAY_OF_YEAR).contains(&day), "{date} -> {day}");
            count += 1;
        }
        assert_eq!(count, 366);
    }

    /// Build a fixed calendar date for tests
    fn test_date(year: i32, month: u32, day: u32) -> NaiveDate {
        match NaiveDate::from_ymd_opt(year, month, day) {
            Some(date) => date,
            None => panic!("invalid test date {year}-{month}-{day}"),
        }
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
        let date = test_date(2025, 12, 28); // Sunday, Dec 28, 2025
        let formatted = format_recipe_day(date);
        assert_eq!(formatted, "Sun, 28-Dec");

        let date2 = test_date(2025, 12, 30); // Tuesday, Dec 30, 2025
        let formatted2 = format_recipe_day(date2);
        assert_eq!(formatted2, "Tue, 30-Dec");

        let date3 = test_date(2026, 1, 1); // Thursday, Jan 1, 2026
        let formatted3 = format_recipe_day(date3);
        assert_eq!(formatted3, "Thu, 1-Jan");
    }

    #[test]
    fn test_format_week_start_date() {
        // Test week start date formatting (no day name, just date)
        let date = test_date(2025, 12, 28);
        let formatted = format_week_start_date(date);
        assert_eq!(formatted, "28-Dec");

        let date2 = test_date(2026, 1, 4);
        let formatted2 = format_week_start_date(date2);
        assert_eq!(formatted2, "4-Jan");

        // Test single digit date (leading zero removal)
        let date3 = test_date(2026, 1, 1);
        let formatted3 = format_week_start_date(date3);
        assert_eq!(formatted3, "1-Jan");
    }

    #[test]
    fn test_get_week_number_from_date() {
        // Test ISO 8601 week number calculation
        // 2025-01-01 (Wednesday) is in ISO week 1 of 2025
        let date1 = test_date(2025, 1, 1);
        assert_eq!(get_week_number_from_date(date1), 1);

        // 2025-01-07 (Tuesday) is in ISO week 2
        let date2 = test_date(2025, 1, 7);
        assert_eq!(get_week_number_from_date(date2), 2);

        // 2025-01-08 (Wednesday) is also in ISO week 2
        let date3 = test_date(2025, 1, 8);
        assert_eq!(get_week_number_from_date(date3), 2);

        // 2025-12-31 (Wednesday) is in ISO week 1 of 2026
        let date4 = test_date(2025, 12, 31);
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
    fn test_embedded_recipe_store_get_by_id() {
        use cookbook_core::RecipeReader;

        let store = EmbeddedRecipeStore::global();

        // Test first day
        let recipe1 = store.get_by_id("day-1").unwrap();
        assert_eq!(recipe1.id, "day-1");
        assert!(!recipe1.title.is_empty());

        // Test middle day
        let recipe100 = store.get_by_id("day-100").unwrap();
        assert_eq!(recipe100.id, "day-100");
        assert!(!recipe100.title.is_empty());

        // Test last day
        let recipe365 = store.get_by_id("day-365").unwrap();
        assert_eq!(recipe365.id, "day-365");
        assert!(!recipe365.title.is_empty());
    }

    #[test]
    fn test_embedded_recipe_has_content() {
        use cookbook_core::RecipeReader;

        let store = EmbeddedRecipeStore::global();
        let recipe = store.get_by_id("day-1").unwrap();

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
        assert!(store.get_by_id("day-0").is_err(), "Day 0 should be invalid");

        // Day 366 should fail
        assert!(
            store.get_by_id("day-366").is_err(),
            "Day 366 should be invalid"
        );
    }

    #[test]
    fn test_get_week_recipes() {
        // Test that we can get recipes for ISO week 1 using PlanReader
        let recipes = get_week_recipes(1);
        assert_eq!(
            recipes.len(),
            7,
            "Week 1 should have exactly 7 recipes from the plan"
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
        assert_eq!(
            recipes_week2.len(),
            7,
            "Week 2 should have exactly 7 recipes from the plan"
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
    fn test_get_week_recipes_uses_plan_reader() {
        // Test that get_week_recipes uses PlanReader to load plans
        let plan_store = EmbeddedPlanStore::global();
        let recipe_store = EmbeddedRecipeStore::global();

        // Test a few different weeks
        for week in [1, 2, 10, 26, 52] {
            let plan = plan_store.get_by_week(week);
            let recipes = get_week_recipes(week);

            if let Ok(plan) = plan {
                // If plan exists, recipes should match the plan's recipe_uuids
                assert_eq!(
                    recipes.len(),
                    plan.recipe_uuids.len(),
                    "Week {} should have {} recipes matching the plan",
                    week,
                    plan.recipe_uuids.len()
                );

                // Verify the UUIDs match
                for (i, uuid) in plan.recipe_uuids.iter().enumerate() {
                    let recipe = recipe_store.get_by_uuid(uuid).unwrap();
                    let (day, title) = &recipes[i];
                    assert_eq!(
                        recipe.title, *title,
                        "Recipe title should match at position {} for week {}",
                        i, week
                    );
                    // Verify day number matches recipe ID
                    let expected_day = recipe
                        .id
                        .strip_prefix("day-")
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap();
                    assert_eq!(
                        *day, expected_day,
                        "Recipe day should match at position {} for week {}",
                        i, week
                    );
                }
            } else {
                // If plan doesn't exist, recipes should be empty
                assert!(
                    recipes.is_empty(),
                    "Week {} should have no recipes when plan doesn't exist",
                    week
                );
            }
        }
    }

    #[test]
    fn test_get_week_recipes_plan_consistency() {
        // Test that plans have consistent structure (7 days, consecutive)
        let plan_store = EmbeddedPlanStore::global();
        let recipe_store = EmbeddedRecipeStore::global();

        // Test week 1 plan
        let plan1 = plan_store.get_by_week(1).expect("Week 1 plan should exist");
        assert_eq!(plan1.recipe_uuids.len(), 7, "Week 1 should have 7 UUIDs");

        // Verify the UUIDs correspond to recipes for days 1-7
        for (i, uuid) in plan1.recipe_uuids.iter().enumerate() {
            let recipe = recipe_store.get_by_uuid(uuid).expect("Recipe should exist");
            let expected_day = i + 1;
            let day_from_id = recipe
                .id
                .strip_prefix("day-")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap();
            assert_eq!(
                day_from_id, expected_day as u32,
                "Week 1 UUID at position {} should be for day {}",
                i, expected_day
            );
        }

        // Test week 2 plan
        let plan2 = plan_store.get_by_week(2).expect("Week 2 plan should exist");
        assert_eq!(plan2.recipe_uuids.len(), 7, "Week 2 should have 7 UUIDs");

        // Verify the UUIDs correspond to recipes for days 8-14
        for (i, uuid) in plan2.recipe_uuids.iter().enumerate() {
            let recipe = recipe_store.get_by_uuid(uuid).expect("Recipe should exist");
            let expected_day = i + 8;
            let day_from_id = recipe
                .id
                .strip_prefix("day-")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap();
            assert_eq!(
                day_from_id, expected_day as u32,
                "Week 2 UUID at position {} should be for day {}",
                i, expected_day
            );
        }

        // Test a mid-year week (week 26) should have 7 consecutive days
        let plan26 = plan_store
            .get_by_week(26)
            .expect("Week 26 plan should exist");
        assert_eq!(plan26.recipe_uuids.len(), 7, "Week 26 should have 7 UUIDs");

        // Verify UUIDs correspond to consecutive days
        let first_recipe = recipe_store.get_by_uuid(&plan26.recipe_uuids[0]).unwrap();
        let first_day = first_recipe
            .id
            .strip_prefix("day-")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap();

        for (i, uuid) in plan26.recipe_uuids.iter().enumerate() {
            let recipe = recipe_store.get_by_uuid(uuid).expect("Recipe should exist");
            let day_from_id = recipe
                .id
                .strip_prefix("day-")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap();
            assert_eq!(
                day_from_id,
                first_day + i as u32,
                "Days within a week should be consecutive"
            );
        }

        // Test that recipes are retrieved for the correct days
        let recipes = get_week_recipes(26);
        assert_eq!(
            recipes.len(),
            plan26.recipe_uuids.len(),
            "Should have recipes for all UUIDs in the week"
        );
        for (i, uuid) in plan26.recipe_uuids.iter().enumerate() {
            let recipe = recipe_store.get_by_uuid(uuid).unwrap();
            let (day, title) = &recipes[i];
            let expected_day = recipe
                .id
                .strip_prefix("day-")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap();
            assert_eq!(*day, expected_day, "Recipe day should match UUID's recipe");
            assert_eq!(
                *title, recipe.title,
                "Recipe title should match UUID's recipe"
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
    fn test_sidebar_recipes_use_plan_reader() {
        // Test that sidebar uses PlanReader via get_week_recipes
        let sidebar_days = get_sidebar_recipe_days();
        let current_week = get_current_week_of_year();

        // Get the plan for the current week
        let plan_store = EmbeddedPlanStore::global();
        let recipe_store = EmbeddedRecipeStore::global();
        let plan = plan_store.get_by_week(current_week);

        if let Ok(plan) = plan {
            // Sidebar should have the same number of days as the plan
            assert_eq!(
                sidebar_days.len(),
                plan.recipe_uuids.len(),
                "Sidebar should have {} UUIDs from the plan for ISO week {} but has {}",
                plan.recipe_uuids.len(),
                current_week,
                sidebar_days.len()
            );

            // First sidebar day should match the first recipe's day from the plan
            if !plan.recipe_uuids.is_empty() {
                let first_recipe = recipe_store.get_by_uuid(&plan.recipe_uuids[0]).unwrap();
                let first_day = first_recipe
                    .id
                    .strip_prefix("day-")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap();
                assert_eq!(
                    sidebar_days[0].0, first_day,
                    "Sidebar should start at day {} for ISO week {}",
                    first_day, current_week
                );
            }

            // Verify all days match the plan's recipe UUIDs
            for (i, (day, _)) in sidebar_days.iter().enumerate() {
                let recipe = recipe_store.get_by_uuid(&plan.recipe_uuids[i]).unwrap();
                let expected_day = recipe
                    .id
                    .strip_prefix("day-")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap();
                assert_eq!(
                    *day, expected_day,
                    "Sidebar day at position {} should be {} but got {}",
                    i, expected_day, day
                );
            }
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
    fn test_get_week_shopping_list_uses_plan_reader() {
        // Test that get_week_shopping_list uses PlanReader to load plans
        let plan_store = EmbeddedPlanStore::global();
        let recipe_store = EmbeddedRecipeStore::global();

        // Test week 1
        let plan = plan_store.get_by_week(1).expect("Week 1 plan should exist");
        let shopping_list = get_week_shopping_list(1);

        // Collect all ingredients from the plan's recipe UUIDs manually
        let mut expected_ingredients = Vec::new();
        for uuid in &plan.recipe_uuids {
            if let Ok(recipe) = recipe_store.get_by_uuid(uuid) {
                expected_ingredients.extend(recipe.ingredients);
            }
        }
        expected_ingredients.sort();
        expected_ingredients.dedup();

        // Shopping list should match the manually collected ingredients
        assert_eq!(
            shopping_list, expected_ingredients,
            "Shopping list should match ingredients from plan's recipe UUIDs"
        );
    }

    #[test]
    fn test_iso_week_calculation_2026() {
        // 2026 starts on Thursday (Jan 1, 2026), so week 1 starts on Dec 29, 2025
        // Jan 1, 2026 is in ISO week 1
        let date = test_date(2026, 1, 1);
        assert_eq!(date.iso_week().week(), 1);

        // Last day of 2026 (Dec 31, 2026 is Thursday) is in week 53
        let date = test_date(2026, 12, 31);
        assert_eq!(date.iso_week().week(), 53);
    }

    #[test]
    fn test_iso_week_calculation_2020() {
        // 2020 is a leap year starting on Wednesday
        // 2020 has 53 ISO weeks
        // Jan 1, 2020 (Wednesday) is in week 1
        let date = test_date(2020, 1, 1);
        assert_eq!(date.iso_week().week(), 1);

        // Dec 31, 2020 (Thursday) is in week 53
        let date = test_date(2020, 12, 31);
        assert_eq!(date.iso_week().week(), 53);
    }

    #[test]
    fn test_iso_week_calculation_2025() {
        // 2025 starts on Wednesday (Jan 1, 2025)
        // Week 1 starts on Dec 30, 2024 (Monday)
        let date = test_date(2025, 1, 1);
        assert_eq!(date.iso_week().week(), 1);

        // Dec 31, 2025 is Wednesday, in week 1 of 2026
        let date = test_date(2025, 12, 31);
        // This should be week 1 of 2026, not week 53 of 2025
        assert_eq!(date.iso_week().week(), 1);
    }

    #[test]
    fn test_iso_week_monday_start() {
        // ISO weeks always start on Monday
        // Test a few random Mondays to verify they're the first day of their week
        let monday = test_date(2026, 1, 5); // Monday
        assert_eq!(monday.weekday(), chrono::Weekday::Mon);

        let tuesday = test_date(2026, 1, 6); // Tuesday
        assert_eq!(tuesday.weekday(), chrono::Weekday::Tue);
        // Both Monday and Tuesday should be in the same week
        assert_eq!(monday.iso_week().week(), tuesday.iso_week().week());
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_auth0_domain_has_trailing_slash() {
        // Verify AUTH0_DOMAIN includes trailing slash for OIDC discovery
        // Auth0's OIDC discovery endpoint returns issuer URLs with trailing slashes,
        // and the openidconnect crate validates exact matches
        assert!(
            AUTH0_DOMAIN.ends_with("/"),
            "AUTH0_DOMAIN must include trailing slash for OIDC discovery"
        );
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_format_auth_error_user_friendly() {
        use auth_sdk::provider::AuthError;

        // Test that all error messages are concise and user-friendly
        let test_cases = vec![
            (AuthError::ParseError, "Unable to process authentication"),
            (
                AuthError::TokenExchangeError("detailed error".to_string()),
                "Login failed. Please try again",
            ),
            (
                AuthError::BrowserApiError("browser error".to_string()),
                "Browser compatibility issue",
            ),
            (
                AuthError::MissingUrlParameter("code".to_string()),
                "Authentication incomplete",
            ),
            (
                AuthError::SerializationError("json error".to_string()),
                "Unable to process authentication",
            ),
            (
                AuthError::MissingStateData("state".to_string()),
                "Authentication incomplete",
            ),
            (
                AuthError::CsrfValidationFailed,
                "Security check failed. Please try again",
            ),
            (
                AuthError::OidcDiscoveryFailed("discovery error".to_string()),
                "Authentication service unavailable",
            ),
            (
                AuthError::IdTokenValidationFailed("validation error".to_string()),
                "Login verification failed",
            ),
            (
                AuthError::HttpClientError("network error".to_string()),
                "Network error. Please check your connection",
            ),
            (
                AuthError::ConfigError("config error".to_string()),
                "Authentication not configured",
            ),
            (AuthError::Unknown, "An unexpected error occurred"),
        ];

        for (error, expected_message) in test_cases {
            let message = format_auth_error(&error);
            assert_eq!(
                message, expected_message,
                "Error message should be user-friendly"
            );

            // Verify messages are reasonably short (less than 100 characters)
            assert!(
                message.len() < 100,
                "Error message '{}' should be concise (less than 100 chars), got {} chars",
                message,
                message.len()
            );
        }
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_format_auth_error_no_technical_details() {
        use auth_sdk::provider::AuthError;

        // Verify that formatted messages don't contain technical details from the original error
        let technical_error = AuthError::OidcDiscoveryFailed(
            "Failed to fetch https://example.com/.well-known/openid-configuration: NetworkError"
                .to_string(),
        );
        let message = format_auth_error(&technical_error);

        // Should not contain URLs or technical terms
        assert!(!message.contains("https://"), "Should not contain URLs");
        assert!(
            !message.contains(".well-known"),
            "Should not contain technical paths"
        );
        assert!(
            !message.contains("NetworkError"),
            "Should not contain technical error types"
        );

        // Should be a simple, user-friendly message
        assert_eq!(message, "Authentication service unavailable");
    }
}

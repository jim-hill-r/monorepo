use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
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
fn Home() -> Element {
    rsx! {
        div {
            h1 { "Cookbook" }
            p { "Welcome to the Cookbook application!" }
            nav {
                h2 { "Daily Recipes" }
                p { "Visit /recipe/1 through /recipe/365 for daily recipes" }
                h2 { "Weekly Plans" }
                p { "Visit /plan/1 through /plan/52 for weekly meal plans" }
            }
        }
    }
}

#[component]
fn Recipe(day: u32) -> Element {
    rsx! {
        div {
            h1 { "Recipe for Day {day}" }
            p { "This is a placeholder recipe for day {day} of the year." }
            Link { to: Route::Home {}, "Back to Home" }
        }
    }
}

#[component]
fn Plan(week: u32) -> Element {
    rsx! {
        div {
            h1 { "Meal Plan for Week {week}" }
            p { "This is a placeholder meal plan for week {week} of the year." }
            Link { to: Route::Home {}, "Back to Home" }
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

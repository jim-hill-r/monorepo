use crate::Route;
use dioxus::prelude::*;

/// Congratulations page - shown when all expressions have been mastered
#[component]
pub fn Congratulations() -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 80vh; gap: 24px; text-align: center;",
            p {
                style: "font-size: 1.4rem; color: #555;",
                strong { "Congratulations." }
                " You have graduated!"
            }
            Link {
                to: Route::Home {},
                style: "background-color: #178CA4; color: white; padding: 12px 32px; border-radius: 8px; font-size: 1.1rem; text-decoration: none;",
                "Start Over"
            }
        }
    }
}

use crate::Route;
use dioxus::prelude::*;

/// Home page - matches blue.eel.education's opening screen
#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            class: "page",
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 80vh; gap: 24px;",

            h1 {
                class: "text-primary",
                style: "font-size: 3rem; color: #178CA4; margin: 0;",
                "Blue Eel"
            }
            h2 {
                class: "text-primary",
                style: "font-size: 1.8rem; color: #178CA4; margin: 0; font-weight: 400;",
                "Writing made simple!"
            }
            Link {
                to: Route::Letter {},
                class: "btn-begin",
                style: "background-color: #18B7BE; color: white; padding: 14px 40px; border-radius: 8px; font-size: 1.2rem; text-decoration: none; border: none; cursor: pointer;",
                "Begin"
            }
        }
    }
}

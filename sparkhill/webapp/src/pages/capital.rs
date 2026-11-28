use crate::components::practice::Practice;
use crate::state::AppState;
use dioxus::prelude::*;

/// Capital letter tracing practice page, after the words
#[component]
pub fn Capital() -> Element {
    let state = use_signal(|| AppState::new("capital"));
    rsx! {
        Practice { state, level: "capital".to_string() }
    }
}

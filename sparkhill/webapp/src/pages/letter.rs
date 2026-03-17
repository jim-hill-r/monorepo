use crate::components::practice::Practice;
use crate::state::AppState;
use dioxus::prelude::*;

/// Letter tracing practice page
#[component]
pub fn Letter() -> Element {
    let state = use_signal(|| AppState::new("letter"));
    rsx! {
        Practice { state, level: "letter".to_string() }
    }
}

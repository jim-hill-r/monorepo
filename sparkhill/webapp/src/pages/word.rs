use crate::components::practice::Practice;
use crate::state::AppState;
use dioxus::prelude::*;

/// Word tracing practice page
#[component]
pub fn Word() -> Element {
    let state = use_signal(|| AppState::new("word"));
    rsx! {
        Practice { state, level: "word".to_string() }
    }
}

use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn Login() -> Element {
    let navigator = navigator(); // Get the navigator

    // Directly reroute to login cahokia as there are no other login options for this project
    navigator.push(Route::LoginCahokia {});

    rsx! {}
}

use auth::authorize;
use dioxus::prelude::*;

#[component]
pub fn LoginCahokia() -> Element {
    let navigator = navigator(); // Get the navigator

    // Programmatically navigate to the auth0 universal login
    authorize();

    rsx! {}
}

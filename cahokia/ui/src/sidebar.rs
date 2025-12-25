use dioxus::prelude::*;

const SIDEBAR_CSS: Asset = asset!("/assets/styling/sidebar.css");

#[component]
pub fn Sidebar(is_open: Signal<bool>, children: Element) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: SIDEBAR_CSS }

        div {
            id: "sidebar",
            class: if is_open() { "open" } else { "" },
            div {
                class: "sidebar-content",
                {children}
            }
        }
    }
}

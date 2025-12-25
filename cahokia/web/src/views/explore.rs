use dioxus::prelude::*;

#[component]
pub fn Explore() -> Element {
    rsx! {
        div {
            class: "page-content",
            h2 { "Explore Cahokia" }
            p {
                "Discover the wonders of this ancient city through our interactive features."
            }
            ul {
                li { "Virtual tours of Monks Mound" }
                li { "Artifact galleries" }
                li { "Archaeological findings" }
                li { "Daily life in ancient Cahokia" }
            }
            p {
                "More features coming soon!"
            }
        }
    }
}

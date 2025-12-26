use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            h1 { "Cookbook" }
            p { "This is a placeholder Dioxus application." }
        }
    }
}

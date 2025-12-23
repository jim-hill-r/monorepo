use dioxus::prelude::*;

const HERO_CSS: Asset = asset!("/assets/styling/hero.css");
#[allow(dead_code)]
const HEADER_SVG: Asset = asset!("/assets/header.svg");

#[component]
pub fn Hero() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: HERO_CSS }

        h1 {
            "Cahokia"
        }
        h2 {
            "Discover your ancestors!"
        }
    }
}

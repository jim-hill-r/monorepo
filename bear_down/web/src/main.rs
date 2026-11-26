//! The bear_down web page. A placeholder until the page that reads
//! `rooting.json` is built (see `ISSUES.md`).

use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        main {
            h1 { "bear_down" }
            p { "Which NFL games to root for this week, for Bears fans. Coming soon." }
        }
    }
}

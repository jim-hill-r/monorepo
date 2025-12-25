use dioxus::prelude::*;

#[component]
pub fn About() -> Element {
    rsx! {
        div {
            class: "page-content",
            h2 { "About Cahokia" }
            p {
                "Cahokia was a Native American city located near present-day Collinsville, Illinois. "
                "It was the largest pre-Columbian settlement north of Mexico."
            }
            p {
                "At its peak around 1100 CE, Cahokia covered six square miles and was home to "
                "an estimated 10,000-20,000 people."
            }
        }
    }
}

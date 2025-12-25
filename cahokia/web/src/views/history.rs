use dioxus::prelude::*;

#[component]
pub fn History() -> Element {
    rsx! {
        div {
            class: "page-content",
            h2 { "History of Cahokia" }
            p {
                "The city of Cahokia began to develop around 600 CE and reached its peak between "
                "1050 and 1200 CE. The site features numerous earthen mounds, including the famous "
                "Monks Mound, which is the largest prehistoric earthwork in the Americas."
            }
            p {
                "The city's decline began around 1300 CE, and by 1400 CE, the site was largely "
                "abandoned. The reasons for its decline are still debated by archaeologists."
            }
        }
    }
}

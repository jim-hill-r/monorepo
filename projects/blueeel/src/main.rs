use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            class: "app-container",
            Header {}
            MainContent {}
            Footer {}
        }
    }
}

#[component]
fn Header() -> Element {
    rsx! {
        header {
            class: "header",
            div {
                class: "header-content",
                h1 { "Blue Eel Education" }
                nav {
                    class: "nav",
                    ul {
                        li { a { href: "#home", "Home" } }
                        li { a { href: "#courses", "Courses" } }
                        li { a { href: "#about", "About" } }
                        li { a { href: "#contact", "Contact" } }
                    }
                }
            }
        }
    }
}

#[component]
fn MainContent() -> Element {
    rsx! {
        main {
            class: "main-content",
            section {
                class: "hero",
                h2 { "Welcome to Blue Eel Education" }
                p { "Your journey to knowledge starts here." }
            }
            section {
                class: "features",
                h3 { "Featured Courses" }
                div {
                    class: "course-grid",
                    CourseCard {
                        title: "Introduction to Programming",
                        description: "Learn the basics of programming with hands-on examples."
                    }
                    CourseCard {
                        title: "Web Development",
                        description: "Build modern web applications from scratch."
                    }
                    CourseCard {
                        title: "Data Science",
                        description: "Analyze data and create insights with Python."
                    }
                }
            }
        }
    }
}

#[component]
fn CourseCard(title: String, description: String) -> Element {
    rsx! {
        div {
            class: "course-card",
            h4 { "{title}" }
            p { "{description}" }
            button { "Learn More" }
        }
    }
}

#[component]
fn Footer() -> Element {
    rsx! {
        footer {
            class: "footer",
            p { "© 2025 Blue Eel Education. All rights reserved." }
        }
    }
}

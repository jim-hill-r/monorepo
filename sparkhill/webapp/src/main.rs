use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            class: "container",
            style: "font-family: system-ui, -apple-system, sans-serif; max-width: 1200px; margin: 0 auto; padding: 20px;",

            // Header
            header {
                style: "background-color: #0c4a6e; color: white; padding: 20px; border-radius: 8px; margin-bottom: 30px;",
                nav {
                    style: "display: flex; justify-content: space-between; align-items: center;",
                    h1 {
                        style: "font-size: 2rem; margin: 0;",
                        "Blue Eel"
                    }
                    div {
                        style: "display: flex; gap: 20px;",
                        a {
                            style: "color: white; text-decoration: none; padding: 8px 16px; border: 2px solid white; border-radius: 20px;",
                            href: "#what-we-do",
                            "What We Do"
                        }
                        a {
                            style: "color: white; text-decoration: none; padding: 8px 16px; border: 2px solid white; border-radius: 20px;",
                            href: "#who-we-are",
                            "Who We Are"
                        }
                        a {
                            style: "color: white; text-decoration: none; padding: 8px 16px; border: 2px solid white; border-radius: 20px;",
                            href: "#how-we-do-it",
                            "How We Do It"
                        }
                    }
                }
            }

            // Hero Section
            section {
                style: "text-align: center; padding: 60px 20px; background: linear-gradient(135deg, #0c4a6e 0%, #0e7490 100%); color: white; border-radius: 8px; margin-bottom: 30px;",
                h2 {
                    style: "font-size: 3rem; margin-bottom: 20px; font-weight: 600;",
                    "Everyone deserves to read"
                }
                p {
                    style: "font-size: 1.5rem; margin: 0;",
                    "...we help get them there."
                }
            }

            // What We Do Section
            section {
                id: "what-we-do",
                style: "padding: 40px; background-color: #f0f9ff; border-radius: 8px; margin-bottom: 30px;",
                h2 {
                    style: "color: #0c4a6e; font-size: 2.5rem; margin-bottom: 20px;",
                    "What We Do"
                }
                p {
                    style: "font-size: 1.1rem; line-height: 1.6; color: #334155;",
                    "We provide a range of services to help individuals improve their reading skills. These services
                    may include one-on-one tutoring or small group instruction, using specialized teaching methods
                    and techniques to help individuals with dyslexia or other reading difficulties. We also offer
                    assessments and evaluations to identify an individual's specific needs and challenges, and then
                    develop a personalized plan to address those needs. Additionally, we provide training and support
                    for parents, teachers, and other professionals who work with individuals with reading difficulties,
                    to help them understand how to effectively support and assist these individuals."
                }
            }

            // Approach Section
            section {
                id: "how-we-do-it",
                style: "padding: 40px; margin-bottom: 30px;",
                h2 {
                    style: "color: #0c4a6e; font-size: 2.5rem; margin-bottom: 30px;",
                    "How We Do It"
                }
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px;",

                    ApproachCard {
                        title: "Phonological Awareness",
                        description: "Build foundational skills in recognizing and manipulating sounds in words."
                    }
                    ApproachCard {
                        title: "Reading Comprehension",
                        description: "Develop strategies for understanding and analyzing written text."
                    }
                    ApproachCard {
                        title: "Fluency Building",
                        description: "Practice reading smoothly and confidently with appropriate expression."
                    }
                    ApproachCard {
                        title: "Vocabulary Development",
                        description: "Expand word knowledge and understanding through engaging activities."
                    }
                }
            }

            // Footer
            footer {
                style: "background-color: #0c4a6e; color: white; padding: 30px; border-radius: 8px; text-align: center; margin-top: 40px;",
                div {
                    style: "margin-bottom: 20px;",
                    a {
                        style: "color: white; text-decoration: none; margin: 0 15px;",
                        href: "https://dyslexiaida.org",
                        target: "_blank",
                        "Dyslexia Resources"
                    }
                    a {
                        style: "color: white; text-decoration: none; margin: 0 15px;",
                        href: "https://CHADD.org",
                        target: "_blank",
                        "CHADD.org"
                    }
                    a {
                        style: "color: white; text-decoration: none; margin: 0 15px;",
                        href: "https://ASHA.org",
                        target: "_blank",
                        "ASHA.org"
                    }
                }
                p {
                    style: "margin: 10px 0 0 0; font-size: 0.9rem;",
                    "© 2024 Blue Eel Education. All Rights Reserved."
                }
            }
        }
    }
}

#[component]
fn ApproachCard(title: String, description: String) -> Element {
    rsx! {
        div {
            style: "background-color: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",
            h3 {
                style: "color: #0c4a6e; font-size: 1.5rem; margin-bottom: 15px;",
                "{title}"
            }
            p {
                style: "color: #64748b; line-height: 1.5;",
                "{description}"
            }
        }
    }
}

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
                            style: "color: white; text-decoration: none; padding: 8px 16px; border: 2px solid white; border-radius: 20px; transition: background-color 0.3s;",
                            href: "#about",
                            "About" 
                        }
                        a { 
                            style: "color: white; text-decoration: none; padding: 8px 16px; border: 2px solid white; border-radius: 20px;",
                            href: "#lessons",
                            "Lessons" 
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
            
            // About Section
            section {
                id: "about",
                style: "padding: 40px; background-color: #f0f9ff; border-radius: 8px; margin-bottom: 30px;",
                h2 { 
                    style: "color: #0c4a6e; font-size: 2.5rem; margin-bottom: 20px;",
                    "What We Do" 
                }
                p { 
                    style: "font-size: 1.1rem; line-height: 1.6; color: #334155;",
                    "We provide comprehensive reading instruction and support for learners of all ages. 
                    Our platform offers interactive lessons, personalized learning paths, and engaging 
                    activities designed to help individuals improve their reading skills. Whether you're 
                    working with dyslexia, other reading challenges, or simply want to enhance your 
                    reading abilities, we're here to support your journey." 
                }
            }
            
            // Lessons Section
            section {
                id: "lessons",
                style: "padding: 40px; margin-bottom: 30px;",
                h2 { 
                    style: "color: #0c4a6e; font-size: 2.5rem; margin-bottom: 30px;",
                    "Our Approach" 
                }
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px;",
                    
                    LessonCard {
                        title: "Phonological Awareness",
                        description: "Build foundational skills in recognizing and manipulating sounds in words."
                    }
                    LessonCard {
                        title: "Reading Comprehension",
                        description: "Develop strategies for understanding and analyzing written text."
                    }
                    LessonCard {
                        title: "Fluency Building",
                        description: "Practice reading smoothly and confidently with appropriate expression."
                    }
                    LessonCard {
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
fn LessonCard(title: String, description: String) -> Element {
    rsx! {
        div {
            style: "background-color: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); transition: transform 0.3s;",
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

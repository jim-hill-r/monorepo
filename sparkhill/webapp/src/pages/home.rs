use crate::Route;
use crate::progress;
use dioxus::document::eval;
use dioxus::prelude::*;

/// Home page - matches blue.eel.education's opening screen, with the
/// learner's name, under which their progress is kept (`progress.rs`).
#[component]
pub fn Home() -> Element {
    let mut name = use_signal(String::new);

    // Fill in the name typed last time on this device.
    use_future(move || async move {
        if let Ok(saved) = eval(&progress::get_learner_js()).join::<String>().await {
            name.set(saved);
        }
    });

    rsx! {
        div {
            class: "page",
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 80vh; gap: 24px;",

            h1 {
                class: "text-primary",
                style: "font-size: 3rem; color: #178CA4; margin: 0;",
                "Blue Eel"
            }
            h2 {
                class: "text-primary",
                style: "font-size: 1.8rem; color: #178CA4; margin: 0; font-weight: 400;",
                "Writing made simple!"
            }
            label {
                style: "display: flex; flex-direction: column; align-items: center; gap: 8px; color: #072A40; font-size: 1.1rem;",
                "Who is practising?"
                input {
                    r#type: "text",
                    name: "learner",
                    autocomplete: "off",
                    maxlength: 40,
                    placeholder: "Your name",
                    value: "{name}",
                    style: "font-size: 1.1rem; padding: 10px 14px; border: 2px solid #178CA4; border-radius: 8px; text-align: center; width: 14em; max-width: 80vw;",
                    oninput: move |e| {
                        let typed = e.value();
                        eval(&progress::set_learner_js(&typed));
                        name.set(typed);
                    },
                }
            }
            Link {
                to: Route::Letter {},
                class: "btn-begin",
                style: "background-color: #18B7BE; color: white; padding: 14px 40px; border-radius: 8px; font-size: 1.2rem; text-decoration: none; border: none; cursor: pointer;",
                "Begin"
            }
        }
    }
}

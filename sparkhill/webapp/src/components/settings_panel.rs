use crate::SettingsSaved;
use crate::settings::{self, Settings};
use crate::technique::Technique;
use dioxus::document::eval;
use dioxus::prelude::*;

const FIELD_STYLE: &str = "font-size: 1.05rem; padding: 10px 12px; border: 2px solid #178CA4; border-radius: 8px; background: white; width: 100%; box-sizing: border-box;";
const LABEL_STYLE: &str =
    "display: flex; flex-direction: column; gap: 6px; color: #072A40; font-size: 1rem;";
const HINT_STYLE: &str = "font-size: 0.85rem; color: #555;";

/// The settings panel behind the top-right button, as on the live app:
/// Student, Sequence and Technique (`settings.rs`). Save keeps them on this
/// device and starts the open level again with them.
#[component]
pub fn SettingsPanel(on_close: EventHandler<()>) -> Element {
    let mut student = use_signal(String::new);
    let mut sequence = use_signal(String::new);
    let mut technique = use_signal(Technique::default);
    let mut saved = use_context::<SettingsSaved>().0;

    // Fill in what was saved last time on this device.
    use_future(move || async move {
        if let Ok(loaded) = eval(&settings::load_js()).join::<String>().await {
            let loaded = Settings::parse(&loaded);
            student.set(loaded.student);
            sequence.set(loaded.sequence);
            technique.set(loaded.technique);
        }
    });

    let save = move |e: FormEvent| {
        e.prevent_default();
        let chosen = Settings {
            student: student(),
            sequence: sequence(),
            technique: technique(),
        };
        eval(&chosen.save_js());
        saved += 1;
        on_close.call(());
    };

    rsx! {
        div {
            class: "drawer-backdrop",
            style: "position: fixed; inset: 0; background: rgba(7, 42, 64, 0.35); z-index: 10;",
            onclick: move |_| on_close.call(()),
        }
        aside {
            role: "dialog",
            aria_label: "Settings",
            tabindex: "-1",
            // Focus the panel, so Escape closes it, without opening a
            // phone's keyboard.
            onmounted: move |e| async move {
                let _ = e.set_focus(true).await;
            },
            style: "position: fixed; top: 0; right: 0; bottom: 0; width: 300px; max-width: 85vw; background: #F9F7F0; box-shadow: -2px 0 12px rgba(0,0,0,0.2); z-index: 20; padding: 20px 16px; box-sizing: border-box; overflow-y: auto;",
            onkeydown: move |e| {
                if e.key() == Key::Escape {
                    on_close.call(());
                }
            },
            h2 { style: "margin: 0 0 16px; color: #178CA4; font-size: 1.3rem;", "Settings" }
            form {
                style: "display: flex; flex-direction: column; gap: 18px;",
                onsubmit: save,
                label {
                    style: LABEL_STYLE,
                    "Student"
                    input {
                        r#type: "text",
                        name: "student",
                        autocomplete: "off",
                        maxlength: 40,
                        placeholder: "Your name",
                        value: "{student}",
                        style: FIELD_STYLE,
                        oninput: move |e| student.set(e.value()),
                    }
                    span { style: HINT_STYLE, "Each student's progress is kept on this device." }
                }
                label {
                    style: LABEL_STYLE,
                    "Sequence"
                    input {
                        r#type: "text",
                        name: "sequence",
                        autocomplete: "off",
                        maxlength: 200,
                        placeholder: "b, d, hi",
                        value: "{sequence}",
                        style: FIELD_STYLE,
                        oninput: move |e| sequence.set(e.value()),
                    }
                    span { style: HINT_STYLE, "Letters or words to practise. Leave it empty for the whole course." }
                }
                label {
                    style: LABEL_STYLE,
                    "Technique"
                    select {
                        name: "technique",
                        style: FIELD_STYLE,
                        onchange: move |e| technique.set(Technique::choose("", &e.value())),
                        for t in Technique::ALL {
                            option {
                                value: t.as_str(),
                                selected: technique() == t,
                                "{t.as_str()}"
                            }
                        }
                    }
                }
                button {
                    r#type: "submit",
                    style: "background-color: #18B7BE; color: white; border: none; padding: 12px; border-radius: 8px; font-size: 1.1rem; cursor: pointer;",
                    "Save"
                }
            }
        }
    }
}

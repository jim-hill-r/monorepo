mod canvas_js;
mod components;
mod pages;
mod patterns;
mod progress;
mod settings;
mod state;
mod technique;
mod tracing_check;

use components::settings_panel::SettingsPanel;
use dioxus::prelude::*;
use pages::{
    capital::Capital, congratulations::Congratulations, home::Home, letter::Letter, word::Word,
};

fn main() {
    dioxus::launch(App);
}

/// Application routes matching blue.eel.education's navigation structure
#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[layout(Shell)]
    #[route("/")]
    Home {},
    #[route("/letter")]
    Letter {},
    #[route("/word")]
    Word {},
    #[route("/capital")]
    Capital {},
    #[route("/congratulations")]
    Congratulations {},
}

/// Root application component: the router
#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

/// Counts the settings panel's saves, so an open practice page starts its
/// level again with the new settings.
#[derive(Clone, Copy)]
pub struct SettingsSaved(pub Signal<u32>);

/// The left menu's entries: where each goes, its name and what it is for.
/// The live app has Letter and Word Practice; capitals are this app's own.
fn practice_menu() -> [(Route, &'static str, &'static str); 3] {
    [
        (
            Route::Letter {},
            "Letter Practice",
            "Practise writing individual letters with assistance or freeform",
        ),
        (
            Route::Word {},
            "Word Practice",
            "Practise combining letters to form complete words",
        ),
        (
            Route::Capital {},
            "Capital Practice",
            "Practise writing capital letters",
        ),
    ]
}

const HEADER_BUTTON: &str = "background: none; border: none; color: white; font-size: 1.5rem; line-height: 1; padding: 6px 10px; cursor: pointer; border-radius: 6px;";

/// Header, the left practice menu, the settings panel and page content,
/// around every route, as on the live app. It is a layout inside the router
/// because `Link` needs a parent router.
#[component]
fn Shell() -> Element {
    let mut menu_open = use_signal(|| false);
    let mut settings_open = use_signal(|| false);
    use_context_provider(|| SettingsSaved(Signal::new(0)));

    rsx! {
        div {
            style: "font-family: system-ui, -apple-system, sans-serif; min-height: 100vh; background-color: #f9f9f9;",

            header {
                style: "background-color: #178CA4; padding: 8px 12px; display: flex; align-items: center; gap: 8px;",
                button {
                    r#type: "button",
                    aria_label: "Menu",
                    aria_expanded: "{menu_open}",
                    style: HEADER_BUTTON,
                    onclick: move |_| menu_open.toggle(),
                    "☰"
                }
                Link {
                    to: Route::Home {},
                    style: "color: white; text-decoration: none; font-size: 1.4rem; font-weight: 700;",
                    "Blue Eel"
                }
                div { style: "flex: 1;" }
                button {
                    r#type: "button",
                    aria_label: "Settings",
                    aria_expanded: "{settings_open}",
                    style: HEADER_BUTTON,
                    onclick: move |_| settings_open.toggle(),
                    "⋮"
                }
            }

            if menu_open() {
                div {
                    class: "drawer-backdrop",
                    style: "position: fixed; inset: 0; background: rgba(7, 42, 64, 0.35); z-index: 10;",
                    onclick: move |_| menu_open.set(false),
                }
                nav {
                    aria_label: "Practice",
                    style: "position: fixed; top: 0; left: 0; bottom: 0; width: 300px; max-width: 85vw; background: #F9F7F0; box-shadow: 2px 0 12px rgba(0,0,0,0.2); z-index: 20; padding: 12px 0; box-sizing: border-box; overflow-y: auto;",
                    for (route, name, caption) in practice_menu() {
                        Link {
                            to: route,
                            onclick: move |_| menu_open.set(false),
                            style: "display: block; padding: 14px 20px; text-decoration: none; color: #072A40;",
                            span { style: "display: block; font-size: 1.1rem; font-weight: 600;", "{name}" }
                            span { style: "display: block; font-size: 0.85rem; color: #555; margin-top: 2px;", "{caption}" }
                        }
                    }
                }
            }

            if settings_open() {
                SettingsPanel { on_close: move |_| settings_open.set(false) }
            }

            // Page content
            main {
                style: "padding: 16px;",
                Outlet::<Route> {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_menu_opens_each_level() {
        let routes: Vec<Route> = practice_menu().into_iter().map(|(r, _, _)| r).collect();
        assert_eq!(
            routes,
            vec![Route::Letter {}, Route::Word {}, Route::Capital {}]
        );
    }
}

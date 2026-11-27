use dioxus::prelude::*;

use ui::Navbar;
use views::{FamilyTree, Home};

mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(DesktopNavbar)]
    #[route("/")]
    Home {},
    #[route("/family-tree")]
    FamilyTree {},
}

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        design_system_dioxus::Stylesheets { default_theme: ui::DEFAULT_THEME }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Router::<Route> {}
    }
}

#[component]
fn DesktopNavbar() -> Element {
    rsx! {
        Navbar {
            Link { to: Route::Home {}, "Home" }
            Link { to: Route::FamilyTree {}, "Family Tree" }
            button {
                onclick: move |_| {
                    // TODO: Implement desktop authentication when auth_sdk supports desktop platforms
                },
                "Login"
            }
        }

        Outlet::<Route> {}

        footer { class: "appearance", design_system_dioxus::ThemePicker {} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn family_tree_has_the_same_path_as_on_the_web() {
        assert_eq!(
            "/family-tree".parse::<Route>().ok(),
            Some(Route::FamilyTree {})
        );
        assert_eq!(Route::FamilyTree {}.to_string(), "/family-tree");
    }

    #[test]
    fn home_is_still_the_root() {
        assert_eq!("/".parse::<Route>().ok(), Some(Route::Home {}));
    }
}

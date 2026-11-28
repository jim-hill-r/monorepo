use dioxus::mobile::tao::event::Event;
use dioxus::prelude::*;

use ui::Navbar;
use views::{FamilyTree, Home};

mod auth;
mod views;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(MobileNavbar)]
    #[route("/")]
    Home {},
    #[route("/family-tree")]
    FamilyTree {},
}

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    // The OS opens the app with the sign-in callback URL (the custom
    // scheme in `auth::REDIRECT_URI`); tao reports it as `Event::Opened`.
    let config = dioxus::mobile::Config::new().with_custom_event_handler(|event, _| {
        if let Event::Opened { urls } = event {
            deliver_opened(&auth::INBOX, urls);
        }
    });
    dioxus::LaunchBuilder::new().with_cfg(config).launch(App);
}

/// Hand each URL the app was opened with to the sign-in waiting for it.
fn deliver_opened(inbox: &auth::CallbackInbox, urls: impl IntoIterator<Item = impl AsRef<str>>) {
    for url in urls {
        if !inbox.deliver(url.as_ref()) {
            tracing::debug!("opened with a URL that isn't a sign-in callback");
        }
    }
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
fn MobileNavbar() -> Element {
    rsx! {
        Navbar {
            Link { to: Route::Home {}, "Home" }
            Link { to: Route::FamilyTree {}, "Family Tree" }
            auth::MobileAuth {}
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

    #[tokio::test]
    async fn the_callback_the_app_is_opened_with_reaches_sign_in() {
        let inbox = auth::CallbackInbox::new();
        let waiting = inbox.expect();
        let callback = format!("{}?code=c&state=s", auth::REDIRECT_URI);
        deliver_opened(&inbox, ["https://example.com/", callback.as_str()]);
        assert_eq!(waiting.await.ok(), Some(callback));
    }

    #[test]
    fn the_navbar_has_the_login_button() {
        let html = dioxus_ssr::render_element(rsx! { MobileNavbar {} });
        assert!(html.contains(">Login</button>"), "{html}");
    }

    #[test]
    fn home_is_still_the_root() {
        assert_eq!("/".parse::<Route>().ok(), Some(Route::Home {}));
    }
}

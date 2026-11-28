use perseus::errors::ClientError;
use perseus::prelude::*;
use sycamore::prelude::*;

use crate::templates::index::CONTACT_URL;

/// What an error view tells the visitor. The site is static files on
/// Cloudflare Pages, with no server of its own and no error reporting, so
/// the messages never mention either: they say what went wrong in plain
/// words and what the visitor can do (reload, go home, or get in touch).
#[derive(Debug, PartialEq, Eq)]
struct ErrorMessage {
    title: &'static str,
    text: &'static str,
}

fn message(err: &ClientError) -> ErrorMessage {
    match err {
        ClientError::ServerError { status: 404, .. } => ErrorMessage {
            title: "Page not found",
            text: "Sorry, that page doesn't exist.",
        },
        ClientError::ServerError { .. } => ErrorMessage {
            title: "Something went wrong",
            text: "Sorry, this page didn't load. Please try reloading it.",
        },
        ClientError::FetchError(_) => ErrorMessage {
            title: "Couldn't load the page",
            text: "Sorry, this page couldn't be loaded. Check your internet connection, \
                   then try reloading it.",
        },
        // A panic, or a failure inside Perseus or the browser: the visitor
        // can't fix any of these, and the details would mean nothing to
        // them, so they are not shown.
        _ => ErrorMessage {
            title: "Something went wrong",
            text: "Sorry, something went wrong showing this page. Please try reloading it.",
        },
    }
}

/// The body of every error view: the message, then links to the home page
/// and to where people reach Spark Hill.
fn body<G: Html>(cx: Scope, msg: &ErrorMessage) -> View<G> {
    let title = msg.title;
    let text = msg.text;
    view! { cx,
        main(class="mr-4 ml-4 pt-10 pb-10 text-center") {
            h1(class="text-sky-700 text-4xl font-medium") { (title) }
            p(class="pt-4") { (text) }
            p(class="pt-4") {
                a(href="/", class="text-sky-700 underline pr-4") { "Go to the home page" }
                a(href=CONTACT_URL, target="_blank", rel="noopener noreferrer", class="text-sky-700 underline") { "Contact Us" }
            }
        }
    }
}

/// The error views for every page. Each returns a `<head>` with only a
/// title and the body above; the stylesheets are in the page shell
/// (`templates::index::shell`), which every page and error page shares.
pub fn get_error_views<G: Html>() -> ErrorViews<G> {
    ErrorViews::new(|cx, err, _err_info, _err_pos| {
        let msg = message(&err);
        let title = msg.title;
        (view! { cx, title { (title) } }, body(cx, &msg))
    })
}

#[cfg(test)]
mod tests {
    use super::{body, message, ClientError, ErrorMessage, CONTACT_URL};

    fn server(status: u16) -> ClientError {
        ClientError::ServerError {
            status,
            message: "details".into(),
        }
    }

    fn every_kind() -> Vec<ClientError> {
        vec![
            server(404),
            server(400),
            server(500),
            ClientError::Panic("boom".into()),
        ]
    }

    fn render(msg: &ErrorMessage) -> String {
        sycamore::render_to_string(|cx| body(cx, msg))
    }

    #[test]
    fn not_found_says_so() {
        assert_eq!(message(&server(404)).title, "Page not found");
    }

    #[test]
    fn other_errors_ask_the_visitor_to_reload() {
        for err in every_kind().iter().skip(1) {
            assert!(message(err).text.contains("reload"), "{err:?}");
        }
    }

    #[test]
    fn never_claims_a_server_or_error_reporting() {
        // The site is static files with no server and reports nothing.
        for err in every_kind() {
            let msg = message(&err);
            let words = format!("{} {}", msg.title, msg.text).to_lowercase();
            for claim in ["server", "reported", "our team"] {
                assert!(!words.contains(claim), "{err:?} says {claim}");
            }
        }
    }

    #[test]
    fn never_shows_the_error_details() {
        let panic = ClientError::Panic("secret panic text".into());
        assert!(!render(&message(&panic)).contains("secret panic text"));
        assert!(!render(&message(&server(500))).contains("details"));
    }

    #[test]
    fn every_view_links_home_and_to_contact() {
        for err in every_kind() {
            let html = render(&message(&err));
            assert!(html.contains("href=\"/\""), "{err:?}");
            assert!(html.contains(&format!("href=\"{CONTACT_URL}\"")), "{err:?}");
            assert_eq!(html.matches("<h1").count(), 1, "{err:?}");
        }
    }
}

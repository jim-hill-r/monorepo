use perseus::prelude::*;
use sycamore::prelude::*;

const TITLE: &str = "Jim Hill"; // TODO: Replace with content found in translations
const TAGLINE: &str = "Be better, build better."; // TODO: Replace with content found in translations

fn index_page<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        // Don't worry, there are much better ways of styling in Perseus!
        div(class="splash") {
            // h1 { (t!(cx, "tagline")) } // TODO: Debug why the t! macro is not working
            h1(class="title") { (TITLE) }
            h2(class="tagline") { (TAGLINE) }
        }
        div(class="content") {
            div(class="section") {
                h3(class="section_header") { "My Passions" }
            }
            div(class="section") {
                h3(class="section_header") { "My Projects" }
            }
            div(class="section") {
                h3(class="section_header") { "My Hobbies" }
            }
        }
        div(class="footer") {

        }
    }
}

#[engine_only_fn]
fn head(cx: Scope) -> View<SsrNode> {
    view! { cx,
        title { (TITLE) }
        link(rel = "stylesheet", href = ".perseus/static/css/reset.css")
        link(rel = "stylesheet", href = ".perseus/static/css/theme.css")
        link(rel = "stylesheet", href = ".perseus/static/css/index.css")
    }
}

pub fn get_template<G: Html>() -> Template<G> {
    Template::build("index").view(index_page).head(head).build()
}

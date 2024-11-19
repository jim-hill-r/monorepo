use perseus::prelude::*;
use sycamore::prelude::*;

fn index_page<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        div(class="content") {
            h1() { "TODO" }
        }
        div(class="footer") {

        }
    }
}

#[engine_only_fn]
fn head(cx: Scope) -> View<SsrNode> {
    view! { cx,
        link(rel = "stylesheet", href = ".perseus/static/css/reset.css")
        link(rel = "stylesheet", href = ".perseus/static/css/theme.css")
        link(rel = "stylesheet", href = ".perseus/static/css/index.css")
    }
}

pub fn get_template<G: Html>() -> Template<G> {
    Template::build("index").view(index_page).head(head).build()
}

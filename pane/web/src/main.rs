//! Pane: one place to review the monorepo. Reads `/data/snapshot.json`
//! (built by `pane_snapshot` in the release) and shows it.

use dioxus::prelude::*;
use pane_model::Snapshot;

mod view;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const SNAPSHOT_URL: &str = "/data/snapshot.json";

fn main() {
    dioxus::launch(App);
}

/// The snapshot as loaded: `None` while loading, then the snapshot or why it
/// couldn't be read.
type Loaded = Option<Result<Snapshot, String>>;

async fn load_snapshot() -> Result<Snapshot, String> {
    let response = gloo_net::http::Request::get(SNAPSHOT_URL)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.ok() {
        return Err(format!("{SNAPSHOT_URL} returned {}", response.status()));
    }
    let text = response.text().await.map_err(|e| e.to_string())?;
    Snapshot::from_json(&text).map_err(|e| e.to_string())
}

/// Seconds since the Unix epoch, from the browser's clock.
fn now() -> i64 {
    // f64 to i64: whole seconds are all we need.
    #[allow(clippy::cast_possible_truncation)]
    let secs = (js_sys::Date::now() / 1000.0) as i64;
    secs
}

#[component]
fn App() -> Element {
    let snapshot = use_resource(load_snapshot);
    let loaded: Memo<Loaded> = use_memo(move || snapshot.read().clone());
    use_context_provider(|| loaded);
    rsx! {
        document::Title { "Pane" }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

// The Routable derive can generate code rustc flags as unreachable; the
// allow has to cover the generated impls, so the enum sits in its own module.
mod route {
    #![allow(unreachable_code)]
    use super::*;

    #[derive(Clone, Routable, Debug, PartialEq)]
    pub enum Route {
        #[layout(Shell)]
        #[route("/")]
        Today {},
        #[route("/health")]
        Health {},
        #[route("/projects")]
        Projects {},
        #[route("/projects/:..path")]
        ProjectPage { path: Vec<String> },
        #[route("/deliverables")]
        Deliverables {},
        #[route("/reviews")]
        Reviews {},
        #[route("/human-issues")]
        HumanIssues {},
        #[route("/work")]
        Work {},
        #[route("/add")]
        Add {},
        #[route("/:..route")]
        NotFound { route: Vec<String> },
    }
}
use route::Route;

/// The nav entries, in order.
fn nav() -> [(Route, &'static str); 8] {
    [
        (Route::Today {}, "Today"),
        (Route::Health {}, "Health"),
        (Route::Projects {}, "Projects"),
        (Route::Deliverables {}, "Deliverables"),
        (Route::Reviews {}, "Reviews"),
        (Route::HumanIssues {}, "Human issues"),
        (Route::Work {}, "Work"),
        (Route::Add {}, "Add"),
    ]
}

/// Left nav, the page, and a footer with the snapshot's commit and age.
#[component]
fn Shell() -> Element {
    let loaded = use_context::<Memo<Loaded>>();
    let footer = match &*loaded.read() {
        Some(Ok(s)) => view::footer_text(s, now()),
        Some(Err(_)) => "No snapshot".to_owned(),
        None => "Loading snapshot…".to_owned(),
    };
    rsx! {
        div { class: "shell",
            nav { class: "nav", "aria-label": "Main",
                Link { class: "brand", to: Route::Today {}, "Pane" }
                ul {
                    for (to, label) in nav() {
                        li {
                            Link { to, active_class: "active", "{label}" }
                        }
                    }
                }
            }
            main { class: "page", Outlet::<Route> {} }
            footer { class: "footer", "data-testid": "snapshot-footer", "{footer}" }
        }
    }
}

/// Renders `body` with the snapshot once it has loaded, or a loading or
/// error message.
#[component]
fn WithSnapshot(body: Callback<Snapshot, Element>) -> Element {
    let loaded = use_context::<Memo<Loaded>>();
    match &*loaded.read() {
        None => rsx! { p { class: "muted", "Loading the snapshot…" } },
        Some(Err(e)) => rsx! {
            div { class: "error", role: "alert",
                p { "Couldn't load the repo snapshot." }
                p { class: "muted", "{e}" }
            }
        },
        Some(Ok(s)) => body.call(s.clone()),
    }
}

/// A page that later phases fill in.
#[component]
fn Planned(title: String, phase: u8, what: String) -> Element {
    rsx! {
        h1 { "{title}" }
        p { "{what}" }
        p { class: "muted", "Coming in Phase {phase} of the review dashboard plan." }
    }
}

#[component]
fn Today() -> Element {
    rsx! {
        h1 { "Today" }
        WithSnapshot {
            body: move |s: Snapshot| {
                let projects = s.projects.len();
                let items = s.work_items.len();
                let human = s.human_issues.len();
                rsx! {
                    ul { class: "stats",
                        li {
                            Link { to: Route::Projects {},
                                strong { "{projects}" }
                                " projects"
                            }
                        }
                        li {
                            Link { to: Route::Work {},
                                strong { "{items}" }
                                " open TODOs and FIXes"
                            }
                        }
                        li {
                            Link { to: Route::HumanIssues {},
                                strong { "{human}" }
                                " human issues"
                            }
                        }
                    }
                    p { class: "muted",
                        "What needs you, new since your last visit, comes in Phase 1."
                    }
                }
            },
        }
    }
}

#[component]
fn Projects() -> Element {
    rsx! {
        h1 { "Projects" }
        WithSnapshot {
            body: move |s: Snapshot| {
                let rows: Vec<_> = view::sorted_projects(&s)
                    .into_iter()
                    .map(|p| {
                        let (todo, fix) = view::open_counts(&s, &p.path);
                        let segments: Vec<String> = p.path.split('/').map(str::to_owned).collect();
                        (p.clone(), segments, todo, fix)
                    })
                    .collect();
                rsx! {
                    ul { class: "projects", "data-testid": "projects",
                        for (p, segments, todo, fix) in rows {
                            li { key: "{p.path}",
                                Link { class: "project-name", to: Route::ProjectPage { path: segments }, "{p.path}" }
                                span { class: "badge status-{view::status_class(p.status)}",
                                    "{view::status_label(p.status)}"
                                }
                                span { class: "muted", " {p.kind}" }
                                if let Some(summary) = &p.summary {
                                    p { class: "summary", "{summary}" }
                                }
                                p { class: "muted", "{todo} TODO, {fix} FIX" }
                            }
                        }
                    }
                }
            },
        }
    }
}

#[component]
fn ProjectPage(path: Vec<String>) -> Element {
    let path = path.join("/");
    rsx! {
        WithSnapshot {
            body: move |s: Snapshot| {
                let Some(p) = view::find_project(&s, &path).cloned() else {
                    return rsx! {
                        h1 { "{path}" }
                        p { "No project at this path in the snapshot." }
                        Link { to: Route::Projects {}, "All projects" }
                    };
                };
                let (todo, fix) = view::open_counts(&s, &p.path);
                rsx! {
                    h1 { "{p.path}" }
                    p {
                        span { class: "badge status-{view::status_class(p.status)}", "{view::status_label(p.status)}" }
                        span { class: "muted", " {p.kind}" }
                    }
                    if let Some(summary) = &p.summary {
                        p { class: "summary", "{summary}" }
                    }
                    if let Some(url) = &p.deploy_url {
                        p {
                            "Live at "
                            a { href: "{url}", "{url}" }
                        }
                    }
                    p { "{todo} open TODOs and {fix} FIXes." }
                    p { class: "muted", "Deliverables, work and health for each project come in Phases 1 and 4." }
                }
            },
        }
    }
}

#[component]
fn Health() -> Element {
    rsx! {
        Planned {
            title: "Health",
            phase: 4,
            what: "A red, amber or green light for every project and shared system, with the evidence behind each.",
        }
    }
}

#[component]
fn Deliverables() -> Element {
    rsx! {
        Planned {
            title: "Deliverables",
            phase: 1,
            what: "Live sites, releases, review decks, plans, decisions and retros.",
        }
    }
}

#[component]
fn Reviews() -> Element {
    rsx! {
        Planned {
            title: "Reviews",
            phase: 2,
            what: "Executive review decks, shown as slides with a PDF download.",
        }
    }
}

#[component]
fn HumanIssues() -> Element {
    rsx! {
        Planned {
            title: "Human issues",
            phase: 1,
            what: "Tasks only you can do, oldest first, with what each one unblocks.",
        }
    }
}

#[component]
fn Work() -> Element {
    rsx! {
        Planned {
            title: "Work",
            phase: 1,
            what: "Every TODO and FIX across the repo, filterable by project, kind and tag.",
        }
    }
}

#[component]
fn Add() -> Element {
    rsx! {
        Planned {
            title: "Add",
            phase: 5,
            what: "Add a TODO, a FIX or a human issue, as a pull request.",
        }
    }
}

#[component]
fn NotFound(route: Vec<String>) -> Element {
    let path = route.join("/");
    rsx! {
        h1 { "Not found" }
        p { "Nothing at /{path}." }
        Link { to: Route::Today {}, "Back to Today" }
    }
}

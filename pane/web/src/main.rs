//! Pane: one place to review the monorepo. Reads `/data/snapshot.json`
//! (built by `pane_snapshot` in the release) and shows it.

use dioxus::prelude::*;
use pane_model::{DeliverableKind, Snapshot, WorkKind};

mod markdown;
mod view;

use view::{Bucket, WorkFilter};

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
        #[route("/docs/:..path")]
        DocPage { path: Vec<String> },
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

/// A route to a document page from its repo path.
fn doc_route(path: &str) -> Route {
    Route::DocPage {
        path: path.split('/').map(str::to_owned).collect(),
    }
}

#[component]
fn Deliverables() -> Element {
    rsx! {
        h1 { "Deliverables" }
        WithSnapshot {
            body: move |s: Snapshot| {
                let groups: Vec<_> = view::DELIVERABLE_GROUPS
                    .iter()
                    .map(|(kind, heading)| {
                        let items: Vec<_> = view::deliverables_of(&s, *kind).into_iter().cloned().collect();
                        (*kind, *heading, items)
                    })
                    .collect();
                rsx! {
                    for (kind , heading , items) in groups {
                        section { class: "group", "data-testid": "deliverables-{view::deliverable_key(kind)}",
                            h2 { "{heading} " span { class: "muted count", "{items.len()}" } }
                            if items.is_empty() {
                                p { class: "muted", "None yet." }
                            }
                            ul { class: "rows",
                                for d in items {
                                    li { key: "{d.title}{d.path:?}{d.url:?}",
                                        if let Some(path) = d.path.clone() {
                                            Link { class: "row-title", to: doc_route(&path), "{d.title}" }
                                        } else if let Some(url) = &d.url {
                                            a { class: "row-title", href: "{url}", "{d.title}" }
                                        } else {
                                            span { class: "row-title", "{d.title}" }
                                        }
                                        if kind == DeliverableKind::LiveSite {
                                            if let Some(url) = &d.url {
                                                span { class: "muted", " {url}" }
                                            }
                                        }
                                        div { class: "meta",
                                            if let Some(p) = &d.project {
                                                if kind != DeliverableKind::LiveSite {
                                                    span { "{p}" }
                                                }
                                            }
                                            if let Some(date) = &d.date {
                                                span { "{date}" }
                                            }
                                            if let Some(status) = &d.status {
                                                span { class: "badge", "{status}" }
                                            }
                                            for link in d.downloads.clone() {
                                                a { class: "download", href: "{link.url}", "Download for {link.label}" }
                                            }
                                            if let Some(path) = &d.path {
                                                a { href: "{view::github_file_url(&s.commit, path)}", "On GitHub" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    p { class: "muted",
                        "Review decks come in Phase 2, and PR previews and UAT reports in Phases 3 and 6."
                    }
                }
            },
        }
    }
}

/// A plan, decision or retro, rendered from its Markdown.
#[component]
fn DocPage(path: Vec<String>) -> Element {
    let path = path.join("/");
    rsx! {
        WithSnapshot {
            body: move |s: Snapshot| {
                let Some(doc) = view::find_doc(&s, &path).cloned() else {
                    return rsx! {
                        h1 { "{path}" }
                        p { "No plan, decision or retro at this path in the snapshot." }
                        Link { to: Route::Deliverables {}, "All deliverables" }
                    };
                };
                let html = markdown::to_html(doc.body.as_deref().unwrap_or_default(), &path, &s.commit);
                let github = view::github_file_url(&s.commit, &path);
                rsx! {
                    p { class: "doc-path",
                        Link { to: Route::Deliverables {}, "Deliverables" }
                        " / {path} · "
                        a { href: "{github}", "Open on GitHub" }
                    }
                    article { class: "doc", "data-testid": "doc", dangerous_inner_html: "{html}" }
                }
            },
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
        h1 { "Human issues" }
        p { class: "muted", "Tasks only you can do, oldest first. Mark done and add a note come in Phase 5." }
        WithSnapshot {
            body: move |s: Snapshot| {
                let now = now();
                let issues: Vec<_> = view::sorted_human_issues(&s)
                    .into_iter()
                    .map(|i| {
                        let age = i
                            .added
                            .as_deref()
                            .and_then(|d| view::days_since(d, now))
                            .map(view::days_label);
                        (i.clone(), age, view::github_line_url(&s.commit, &i.file, i.line))
                    })
                    .collect();
                rsx! {
                    if issues.is_empty() {
                        p { "Nothing is waiting on you." }
                    }
                    ul { class: "rows", "data-testid": "human-issues",
                        for (issue , age , url) in issues {
                            li { key: "{issue.file}:{issue.line}",
                                p { class: "row-text", "{issue.text}" }
                                div { class: "meta",
                                    if let Some(age) = age {
                                        span { class: "badge", "{age}" }
                                    }
                                    if let Some(added) = &issue.added {
                                        span { "Added {added}" }
                                    }
                                    a { href: "{url}", "Open on GitHub" }
                                }
                                if let Some(unblocks) = &issue.unblocks {
                                    p { class: "muted unblocks", "Unblocks {unblocks}" }
                                }
                            }
                        }
                    }
                }
            },
        }
    }
}

#[component]
fn Work() -> Element {
    let mut filter = use_signal(WorkFilter::default);
    rsx! {
        h1 { "Work" }
        WithSnapshot {
            body: move |s: Snapshot| {
                let (projects, tags) = view::work_facets(&s.work_items);
                let current = filter.read().clone();
                let rows: Vec<_> = view::filter_work(&s.work_items, &current)
                    .into_iter()
                    .map(|i| (i.clone(), view::github_line_url(&s.commit, &i.file, i.line)))
                    .collect();
                let total = s.work_items.len();
                let shown = rows.len();
                let some = |v: String| (!v.is_empty()).then_some(v);
                rsx! {
                    form { class: "filters", "aria-label": "Filters",
                        label {
                            "Project "
                            select {
                                name: "project",
                                onchange: move |e| filter.write().project = some(e.value()),
                                option { value: "", "Any" }
                                for p in projects {
                                    option { value: "{p}", selected: current.project.as_deref() == Some(p.as_str()), "{p}" }
                                }
                            }
                        }
                        label {
                            "Kind "
                            select {
                                name: "kind",
                                onchange: move |e| {
                                    filter.write().kind = match e.value().as_str() {
                                        "todo" => Some(WorkKind::Todo),
                                        "fix" => Some(WorkKind::Fix),
                                        _ => None,
                                    };
                                },
                                option { value: "", "Any" }
                                option { value: "todo", selected: current.kind == Some(WorkKind::Todo), "TODO" }
                                option { value: "fix", selected: current.kind == Some(WorkKind::Fix), "FIX" }
                            }
                        }
                        label {
                            "Where "
                            select {
                                name: "bucket",
                                onchange: move |e| filter.write().bucket = Bucket::from_key(&e.value()),
                                option { value: "", "Anywhere" }
                                for b in Bucket::ALL {
                                    option { value: "{b.key()}", selected: current.bucket == Some(b), "{b.label()}" }
                                }
                            }
                        }
                        label {
                            "Tag "
                            select {
                                name: "tag",
                                onchange: move |e| filter.write().tag = some(e.value()),
                                option { value: "", "Any" }
                                for t in tags {
                                    option { value: "{t}", selected: current.tag.as_deref() == Some(t.as_str()), "{t}" }
                                }
                            }
                        }
                    }
                    p { class: "muted", "data-testid": "work-count", "Showing {shown} of {total}" }
                    ul { class: "rows", "data-testid": "work-items",
                        for (item , url) in rows {
                            li { key: "{item.file}:{item.line}",
                                p { class: "row-text",
                                    span { class: "badge kind-{view::kind_label(item.kind).to_lowercase()}", "{view::kind_label(item.kind)}" }
                                    " {item.text}"
                                }
                                div { class: "meta",
                                    span { "{view::project_label(item.project.as_deref())}" }
                                    span { "{Bucket::of(&item).label()}" }
                                    if let Some(epic) = &item.epic {
                                        span { "{epic}" }
                                    }
                                    for t in item.tags.clone() {
                                        span { class: "tag", "{t}" }
                                    }
                                    a { href: "{url}", "{item.file}:{item.line}" }
                                }
                            }
                        }
                    }
                }
            },
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

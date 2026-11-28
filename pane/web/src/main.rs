//! Pane: one place to review the monorepo. Reads `/data/snapshot.json`
//! (built by `pane_snapshot` in the release) and shows it.

use dioxus::prelude::*;
use pane_model::{
    Deliverable, DeliverableKind, HumanIssue, Review, ScorecardRow, Snapshot, WorkItem, WorkKind,
    signoff::SignOff,
};

mod acknowledge;
mod approve;
mod markdown;
mod money;
mod view;
mod visits;

use acknowledge::Acknowledge;
use approve::Approve;
use money::Money;
use view::{Bucket, WorkFilter};
use visits::Visits;

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

/// The browser's `localStorage`, if it has one and lets pane use it.
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Read the stored visits, record this one, and return the result, or
/// `None` when the browser keeps no storage.
fn record_visit(snapshot: &Snapshot) -> Option<Visits> {
    let storage = storage()?;
    let stored = storage.get_item(visits::STORAGE_KEY).ok().flatten();
    let next = visits::record(visits::parse(stored.as_deref()), snapshot, now());
    let json = serde_json::to_string(&next).ok()?;
    // A full or blocked storage only loses "new since last visit".
    storage.set_item(visits::STORAGE_KEY, &json).ok()?;
    Some(next)
}

#[component]
fn App() -> Element {
    let snapshot = use_resource(load_snapshot);
    let loaded: Memo<Loaded> = use_memo(move || snapshot.read().clone());
    use_context_provider(|| loaded);
    rsx! {
        document::Title { "Pane" }
        design_system_dioxus::Stylesheets { default_theme: "professional" }
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
        #[route("/approve")]
        Approve {},
        #[route("/acknowledge")]
        Acknowledge {},
        #[route("/health")]
        Health {},
        #[route("/projects")]
        Projects {},
        #[route("/money")]
        Money {},
        #[route("/projects/:..path")]
        ProjectPage { path: Vec<String> },
        #[route("/deliverables")]
        Deliverables {},
        #[route("/docs/:..path")]
        DocPage { path: Vec<String> },
        #[route("/reviews")]
        Reviews {},
        // Not /reviews/<folder>: the release serves the built decks there.
        #[route("/review/:folder")]
        ReviewPage { folder: String },
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
fn nav() -> [(Route, &'static str); 11] {
    [
        (Route::Today {}, "Today"),
        (Route::Approve {}, "Approve"),
        (Route::Acknowledge {}, "Acknowledge"),
        (Route::Health {}, "Health"),
        (Route::Projects {}, "Projects"),
        (Route::Money {}, "Money"),
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
            footer { class: "footer",
                p { "data-testid": "snapshot-footer", "{footer}" }
                design_system_dioxus::ThemePicker {}
            }
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
        WithSnapshot { body: move |s: Snapshot| rsx! { TodayBody { snapshot: s } } }
    }
}

/// How many new work items Today lists before linking to Work.
const TODAY_WORK_LIMIT: usize = 20;

#[component]
fn TodayBody(snapshot: Snapshot) -> Element {
    // Record the visit once per mount, not on every render.
    let stored = use_hook({
        let s = snapshot.clone();
        move || record_visit(&s)
    });
    let s = snapshot;
    let projects = s.projects.len();
    let items = s.work_items.len();
    let human = s.human_issues.len();
    let now = now();
    let waiting: Vec<HumanIssue> = view::sorted_human_issues(&s).into_iter().cloned().collect();
    let baseline = stored.as_ref().and_then(|v| v.previous.clone());
    let new = baseline.as_ref().map(|b| visits::new_since(&s, b));
    let new_count = new.as_ref().map(visits::NewItems::len);
    let (new_human, new_docs, new_work, more_work) = match &new {
        Some(n) => (
            n.human_issues
                .iter()
                .map(|i| (*i).clone())
                .collect::<Vec<_>>(),
            n.deliverables
                .iter()
                .map(|d| (*d).clone())
                .collect::<Vec<_>>(),
            n.work_items
                .iter()
                .take(TODAY_WORK_LIMIT)
                .map(|i| (*i).clone())
                .collect::<Vec<_>>(),
            n.work_items.len().saturating_sub(TODAY_WORK_LIMIT),
        ),
        None => (vec![], vec![], vec![], 0),
    };
    let since = baseline.as_ref().map(|b| {
        format!(
            "Compared with your visit {} (snapshot {}).",
            view::age(b.at, now),
            view::short_commit(&b.commit)
        )
    });
    let commit = s.commit.clone();
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
            if let Some(n) = new_count {
                li {
                    a { href: "#new",
                        strong { "{n}" }
                        " new since your last visit"
                    }
                }
            }
        }
        section { class: "group", "data-testid": "today-waiting",
            h2 { "Waiting on you " span { class: "muted count", "{waiting.len()}" } }
            if waiting.is_empty() {
                p { "Nothing is waiting on you." }
            }
            ul { class: "rows",
                for issue in waiting {
                    HumanIssueRow { key: "{issue.file}:{issue.line}", issue, commit: commit.clone(), now }
                }
            }
        }
        section { class: "group", id: "new", "data-testid": "today-new",
            h2 {
                "New since your last visit "
                if let Some(n) = new_count {
                    span { class: "muted count", "{n}" }
                }
            }
            match (&stored, &since) {
                (None, _) => rsx! {
                    p { class: "muted", "This browser isn't keeping pane's visits, so nothing is marked new." }
                },
                (Some(_), None) => rsx! {
                    p { class: "muted", "Nothing to compare with yet. From your next visit with a newer snapshot, what was added shows here." }
                },
                (Some(_), Some(since)) => rsx! {
                    p { class: "muted", "{since}" }
                    if new_count == Some(0) {
                        p { "Nothing new." }
                    }
                },
            }
            if !new_human.is_empty() {
                h3 { "Human issues" }
                ul { class: "rows",
                    for issue in new_human {
                        HumanIssueRow { key: "{issue.file}:{issue.line}", issue, commit: commit.clone(), now }
                    }
                }
            }
            if !new_docs.is_empty() {
                h3 { "Deliverables" }
                ul { class: "rows",
                    for d in new_docs {
                        DeliverableRow { key: "{d.title}{d.path:?}{d.url:?}", d, commit: commit.clone() }
                    }
                }
            }
            if !new_work.is_empty() {
                h3 { "TODOs and FIXes" }
                ul { class: "rows",
                    for item in new_work {
                        WorkRow { key: "{item.file}:{item.line}", item, commit: commit.clone() }
                    }
                }
                if more_work > 0 {
                    p { class: "muted",
                        "And {more_work} more on the "
                        Link { to: Route::Work {}, "Work" }
                        " page."
                    }
                }
            }
        }
    }
}

/// One human issue: its text, age, date, GitHub and runbook links, what it
/// unblocks, and why only a person can do it.
#[component]
fn HumanIssueRow(issue: HumanIssue, commit: String, now: i64) -> Element {
    let age = issue
        .added
        .as_deref()
        .and_then(|d| view::days_since(d, now))
        .map(view::days_label);
    let url = view::github_line_url(&commit, &issue.file, issue.line);
    let runbook = issue
        .runbook
        .as_deref()
        .map(|r| view::github_file_url(&commit, r));
    rsx! {
        li {
            p { class: "row-text", "{issue.text}" }
            div { class: "meta",
                if let Some(age) = age {
                    span { class: "badge", "{age}" }
                }
                if let Some(added) = &issue.added {
                    span { "Added {added}" }
                }
                a { href: "{url}", "Open on GitHub" }
                if let Some(runbook) = &runbook {
                    a { href: "{runbook}", "Runbook" }
                }
            }
            if let Some(unblocks) = &issue.unblocks {
                p { class: "muted unblocks", "Unblocks {unblocks}" }
            }
            if let Some(why) = &issue.why {
                p { class: "muted why", "Why a person: {why}" }
            }
        }
    }
}

/// One TODO or FIX with its project, list, epic, tags and line on GitHub.
#[component]
fn WorkRow(item: WorkItem, commit: String) -> Element {
    let url = view::github_line_url(&commit, &item.file, item.line);
    let kind = view::kind_label(item.kind);
    let kind_class = kind.to_lowercase();
    rsx! {
        li {
            p { class: "row-text",
                span { class: "badge kind-{kind_class}", "{kind}" }
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

/// One deliverable: a link to it in pane or outside, and its details.
#[component]
fn DeliverableRow(d: Deliverable, commit: String) -> Element {
    let kind = d.kind;
    rsx! {
        li {
            if kind == DeliverableKind::ReviewDeck {
                if let Some(path) = d.path.as_deref() {
                    Link {
                        class: "row-title",
                        to: Route::ReviewPage { folder: pane_model::review_folder(path).to_owned() },
                        "{d.title}"
                    }
                }
            } else if let Some(path) = d.path.clone() {
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
                    if kind == DeliverableKind::ReviewDeck {
                        a { class: "download", href: "{link.url}", "Download {link.label}" }
                    } else {
                        a { class: "download", href: "{link.url}", "Download for {link.label}" }
                    }
                }
                if let Some(path) = &d.path {
                    a { href: "{view::github_file_url(&commit, path)}", "On GitHub" }
                }
            }
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
                let subs: Vec<String> = view::sub_projects(&s, &p.path).into_iter().map(|q| q.path.clone()).collect();
                let work: Vec<WorkItem> = view::project_work(&s, &p.path).into_iter().cloned().collect();
                let (todo, fix) = view::kind_counts(&work.iter().collect::<Vec<_>>());
                let latest: Vec<(&str, String, String)> = view::latest_docs(&s, &p.path)
                    .into_iter()
                    .filter_map(|(label, d)| Some((label, d.title.clone(), d.path.clone()?)))
                    .collect();
                let groups: Vec<(&str, Vec<Deliverable>)> = view::project_deliverables(&s, &p.path)
                    .into_iter()
                    .map(|(_, heading, items)| (heading, items.into_iter().cloned().collect()))
                    .collect();
                let review = view::latest_review(&s, &p.path).cloned();
                let commit = s.commit.clone();
                let work_scope = if subs.is_empty() { "" } else { ", including its sub-projects" };
                rsx! {
                    p { class: "doc-path",
                        Link { to: Route::Projects {}, "Projects" }
                        " / {p.path}"
                    }
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
                    p {
                        a { href: "{view::github_tree_url(&commit, &p.path)}", "Open on GitHub" }
                    }
                    if !subs.is_empty() {
                        section { class: "group", "data-testid": "project-sub-projects",
                            h2 { "Sub-projects" }
                            ul { class: "inline-list",
                                for sub in subs {
                                    li { key: "{sub}",
                                        Link { to: Route::ProjectPage { path: sub.split('/').map(str::to_owned).collect() }, "{sub}" }
                                    }
                                }
                            }
                        }
                    }
                    section { class: "group", "data-testid": "project-latest",
                        h2 { "Plans, decisions and retros" }
                        if latest.is_empty() {
                            p { class: "muted", "None yet." }
                        }
                        ul { class: "rows",
                            for (label , title , doc) in latest {
                                li { key: "{doc}",
                                    span { class: "muted", "{label}: " }
                                    Link { to: doc_route(&doc), "{title}" }
                                }
                            }
                        }
                    }
                    section { class: "group", "data-testid": "project-review",
                        h2 { "Latest review" }
                        match review {
                            Some(r) => rsx! {
                                ReviewLinks { review: r, show_project: false }
                            },
                            None => rsx! {
                                p { class: "muted",
                                    "Not reviewed yet. The weekly deep review picks the project that has waited longest; see "
                                    Link { to: Route::Reviews {}, "Reviews" }
                                    "."
                                }
                            },
                        }
                    }
                    section { class: "group", "data-testid": "project-deliverables",
                        h2 { "Deliverables" }
                        if groups.is_empty() {
                            p { class: "muted", "None yet." }
                        }
                        for (heading , items) in groups {
                            h3 { "{heading} " span { class: "muted count", "{items.len()}" } }
                            ul { class: "rows",
                                for d in items {
                                    DeliverableRow { key: "{d.title}{d.path:?}{d.url:?}", d, commit: commit.clone() }
                                }
                            }
                        }
                    }
                    section { class: "group", "data-testid": "project-work",
                        h2 { "Open TODOs and FIXes" }
                        p { "{todo} open TODOs and {fix} FIXes{work_scope}." }
                        ul { class: "rows",
                            for item in work {
                                WorkRow { key: "{item.file}:{item.line}", item, commit: commit.clone() }
                            }
                        }
                    }
                    money::ProjectMoneySection { snapshot: s.clone(), path: p.path.clone() }
                    p { class: "muted", "Health and open PRs for each project come in Phases 3 and 4." }
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
                                    DeliverableRow { key: "{d.title}{d.path:?}{d.url:?}", d, commit: s.commit.clone() }
                                }
                            }
                        }
                    }
                    p { class: "muted",
                        "PR previews and UAT reports come in Phases 3 and 6."
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
        h1 { "Reviews" }
        p {
            "Each week the deep project review studies one project and writes a slide deck. "
            "The project that has gone longest without a review goes next. "
            "Open a deck to read it as slides, download its PDF, or comment on its pull request."
        }
        WithSnapshot {
            body: move |s: Snapshot| {
                let reviews: Vec<Review> = view::sorted_reviews(&s).into_iter().cloned().collect();
                rsx! {
                    if reviews.is_empty() {
                        p { "No reviews yet." }
                    } else {
                        table { class: "rotation", "data-testid": "reviews-rotation",
                            thead {
                                tr {
                                    th { scope: "col", "Date" }
                                    th { scope: "col", "Project" }
                                    th { scope: "col", "Deck" }
                                    th { scope: "col", "Scorecard" }
                                    th { scope: "col", "Signed off" }
                                    th { scope: "col", "Comments" }
                                }
                            }
                            tbody {
                                for r in reviews {
                                    ReviewRow { key: "{r.path}", review: r }
                                }
                            }
                        }
                    }
                }
            },
        }
    }
}

/// One row of the rotation table.
#[component]
fn ReviewRow(review: Review) -> Element {
    let folder = review.folder().to_owned();
    let project: Vec<String> = review.project.split('/').map(str::to_owned).collect();
    rsx! {
        tr {
            td { "{review.date}" }
            td {
                Link { to: Route::ProjectPage { path: project }, "{review.project}" }
            }
            td {
                Link { to: Route::ReviewPage { folder: folder.clone() }, "Slides" }
                " · "
                a { href: "{view::deck_pdf_url(&folder)}", "PDF" }
            }
            td {
                ScorecardSummary { rows: review.scorecard.clone() }
            }
            td { "data-testid": "review-signed-off",
                if review.signoffs.is_empty() {
                    span { class: "muted", "Not yet" }
                } else {
                    "{view::signoff_summary(&review)}"
                }
            }
            td {
                if let Some(pr) = &review.pr {
                    a { href: "{pr}", "{view::pr_label(pr)}" }
                } else {
                    span { class: "muted", "No PR yet" }
                }
            }
        }
    }
}

/// A review's slides, PDF and pull request links, for the Project page.
#[component]
fn ReviewLinks(review: Review, show_project: bool) -> Element {
    let folder = review.folder().to_owned();
    let title = if show_project {
        format!("{} deep review, {}", review.project, review.date)
    } else {
        format!("Deep review, {}", review.date)
    };
    rsx! {
        p {
            Link { to: Route::ReviewPage { folder: folder.clone() }, "{title}" }
        }
        div { class: "meta",
            a { class: "download", href: "{view::deck_pdf_url(&folder)}", "Download PDF" }
            if let Some(pr) = &review.pr {
                a { href: "{pr}", "Comment on {view::pr_label(pr)}" }
            }
        }
        if !review.scorecard.is_empty() {
            p { class: "scorecard-line",
                "Scorecard: "
                ScorecardSummary { rows: review.scorecard.clone() }
            }
        }
    }
}

/// A scorecard's ratings counted on one line, such as `2 Good · 4 Fair ·
/// 1 Poor`, each count coloured by its rating.
#[component]
fn ScorecardSummary(rows: Vec<ScorecardRow>) -> Element {
    if rows.is_empty() {
        return rsx! {
            span { class: "muted", "None" }
        };
    }
    let parts: Vec<(String, &str)> = view::scorecard_summary(&rows)
        .split(" · ")
        .map(|part| {
            let label = part.rsplit(' ').next().unwrap_or("");
            (part.to_owned(), view::rating_class(label))
        })
        .collect();
    rsx! {
        span { class: "scorecard-summary", "data-testid": "scorecard-summary",
            for (i , (part , class)) in parts.into_iter().enumerate() {
                span { key: "{part}",
                    if i > 0 {
                        " · "
                    }
                    span { class: "rating rating-{class}", "{part}" }
                }
            }
        }
    }
}

/// A deck's Scorecard slide as a table: each area, its rating and why.
#[component]
fn Scorecard(rows: Vec<ScorecardRow>) -> Element {
    rsx! {
        section { class: "group", "data-testid": "review-scorecard",
            h2 { "Scorecard" }
            if rows.is_empty() {
                p { class: "muted", "This deck has no scorecard pane can read." }
            } else {
                table { class: "rotation scorecard",
                    thead {
                        tr {
                            th { scope: "col", "Area" }
                            th { scope: "col", "Rating" }
                            th { scope: "col", "Why" }
                        }
                    }
                    tbody {
                        for r in rows {
                            tr { key: "{r.area}",
                                th { scope: "row", "{r.area}" }
                                td {
                                    span { class: "badge rating-{view::rating_class(&r.rating)}", "{r.rating}" }
                                }
                                td { "{r.why}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The deck's slides in the page.
const DECK_FRAME: &str = "deck-frame";

/// One review deck: its slides in the page, with full screen, the PDF and
/// its pull request.
#[component]
fn ReviewPage(folder: String) -> Element {
    rsx! {
        WithSnapshot {
            body: move |s: Snapshot| {
                let Some(review) = view::find_review(&s, &folder).cloned() else {
                    return rsx! {
                        h1 { "Review not found" }
                        p { "No review deck {folder} in the snapshot." }
                        Link { to: Route::Reviews {}, "All reviews" }
                    };
                };
                let github = view::github_file_url(&s.commit, &review.path);
                let slides = view::deck_html_url(&folder);
                let pdf = view::deck_pdf_url(&folder);
                let project: Vec<String> = review.project.split('/').map(str::to_owned).collect();
                rsx! {
                    p { class: "doc-path",
                        Link { to: Route::Reviews {}, "Reviews" }
                        " / {folder}"
                    }
                    h1 { "{review.project} deep review" }
                    p { class: "muted",
                        "{review.date} · "
                        Link { to: Route::ProjectPage { path: project }, "Project page" }
                    }
                    div { class: "deck-actions",
                        button {
                            r#type: "button",
                            onclick: move |_| {
                                document::eval(&format!(
                                    "document.querySelector('.{DECK_FRAME}')?.requestFullscreen?.()"
                                ));
                            },
                            "Full screen"
                        }
                        a { class: "download", href: "{pdf}", download: "{folder}.pdf", "Download PDF" }
                        if let Some(pr) = &review.pr {
                            a { href: "{pr}", "Comment on {view::pr_label(pr)}" }
                        }
                        a { href: "{github}", "deck.md on GitHub" }
                    }
                    iframe {
                        class: DECK_FRAME,
                        "data-testid": "deck",
                        title: "{review.project} deep review slides",
                        src: "{slides}",
                        allow: "fullscreen",
                        allowfullscreen: true,
                        // Give the slides the keyboard, so the arrow keys move
                        // between slides straight away.
                        onload: move |_| {
                            document::eval(&format!(
                                "document.querySelector('.{DECK_FRAME}')?.contentWindow?.focus()"
                            ));
                        },
                    }
                    p { class: "muted",
                        "Use the arrow keys to move between slides, and f or Full screen for full screen. "
                        "Leave comments on the pull request, on the lines of the slide."
                    }
                    Scorecard { rows: review.scorecard.clone() }
                    SignOffs { signoffs: review.signoffs.clone(), commit: s.commit.clone() }
                }
            },
        }
    }
}

/// Who has signed the deck off, with each verdict and feedback item, and
/// whether an agent has done the item yet (its issue file deleted).
#[component]
fn SignOffs(signoffs: Vec<SignOff>, commit: String) -> Element {
    rsx! {
        section { class: "group", "data-testid": "review-signoffs",
            h2 { "Sign-offs" }
            if signoffs.is_empty() {
                p { class: "muted", "Nobody has signed this deck off yet." }
            }
            for s in signoffs {
                div { key: "{s.path}", class: "signoff",
                    p {
                        strong { "{s.reviewer}" }
                        " · {s.verdict.label()} · {s.date} · "
                        a { href: "{view::github_file_url(&commit, &s.path)}", "sign-off file" }
                    }
                    if !s.feedback.is_empty() {
                        ul { class: "feedback",
                            for (i , f) in s.feedback.iter().enumerate() {
                                li { key: "{i}",
                                    if let Some(n) = f.slide {
                                        "Slide {n}: "
                                    }
                                    "{f.text} "
                                    match (&f.issue, f.open) {
                                        (Some(issue), true) => rsx! {
                                            a { class: "badge", href: "{view::github_file_url(&commit, issue)}", "Open issue" }
                                        },
                                        (Some(_), false) => rsx! {
                                            span { class: "badge rating-good", "Done" }
                                        },
                                        (None, _) => rsx! {
                                            span { class: "badge muted", "Not filed" }
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "muted",
                "Signing off from pane, with feedback that becomes issue files for agents, comes once pane is behind its sign-in; see the "
                a { href: "{view::github_file_url(&commit, PLAN)}", "deck sign-off plan" }
                "."
            }
        }
    }
}

/// The deck sign-off plan.
const PLAN: &str = "pane/docs/plans/2026-11-deck-sign-off.md";

#[component]
fn HumanIssues() -> Element {
    rsx! {
        h1 { "Human issues" }
        p { class: "muted", "Tasks only you can do, oldest first. Mark done and add a note come in Phase 5." }
        WithSnapshot {
            body: move |s: Snapshot| {
                let now = now();
                let issues: Vec<HumanIssue> = view::sorted_human_issues(&s).into_iter().cloned().collect();
                rsx! {
                    if issues.is_empty() {
                        p { "Nothing is waiting on you." }
                    }
                    ul { class: "rows", "data-testid": "human-issues",
                        for issue in issues {
                            HumanIssueRow { key: "{issue.file}:{issue.line}", issue, commit: s.commit.clone(), now }
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
                let rows: Vec<WorkItem> = view::filter_work(&s.work_items, &current).into_iter().cloned().collect();
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
                        for item in rows {
                            WorkRow { key: "{item.file}:{item.line}", item, commit: s.commit.clone() }
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

//! The Repo health summary on Today and its drill-down on the Health page.
//! Both show the badges in `standards/repo-health.toml`, the file the root
//! README's Repo health tables are written from, so pane and the README
//! always show the same checks, in the same domains, with GitHub's own live
//! badge images (Jim, 2026-11-28).

use dioxus::prelude::*;
use pane_model::Snapshot;
use pane_model::repo_health::{Badge, Domain, Row, anchor};

use crate::{Route, markdown, nav};

/// One badge: GitHub's image, linking to the runs behind it.
#[component]
fn BadgeImage(badge: Badge) -> Element {
    rsx! {
        a { class: "badge", href: "{badge.link}", target: "_blank", rel: "noopener",
            img { src: "{badge.image}", alt: "{badge.alt}", height: "20" }
        }
    }
}

/// The pane page a row drills into, and its name in the nav.
fn pane_page(route: &str) -> Option<(Route, &'static str)> {
    let target: Route = route.parse().ok()?;
    let label = nav()
        .into_iter()
        .find(|(r, _)| *r == target)
        .map_or("Open", |(_, label)| label);
    Some((target, label))
}

/// The top-level summary on Today: one card per domain with its badges.
#[component]
pub fn Summary(snapshot: Snapshot) -> Element {
    let health = snapshot.repo_health;
    if health.domain.is_empty() {
        return rsx! {};
    }
    rsx! {
        section { class: "group", "data-testid": "today-health",
            h2 {
                "Repo health "
                span { class: "muted count", "{health.rows()} checks" }
            }
            p { class: "muted",
                "The same badges as the README. "
                Link { to: Route::Health {}, "Open a domain" }
                " for what each check runs and what red means."
            }
            ul { class: "domains",
                for domain in health.domain {
                    li { key: "{domain.name}", class: "domain-card", "data-testid": "health-domain",
                        Link { class: "domain-name", to: Route::Health {}, "{domain.name}" }
                        div { class: "badges",
                            for row in domain.row.iter() {
                                for badge in row.badge.iter() {
                                    BadgeImage { key: "{badge.image}", badge: badge.clone() }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The Health page: every domain, and every check with its detail.
#[component]
pub fn Detail(snapshot: Snapshot) -> Element {
    let commit = snapshot.commit.clone();
    let health = snapshot.repo_health;
    rsx! {
        p {
            "Every check that guards the whole repo, grouped by domain. These are the "
            a { href: "{pane_model::REPO_URL}#repo-health", "README's badges" }
            ", read from the same file ("
            code { "{pane_model::repo_health::SOURCE}" }
            "), so the two always agree. Each badge links to its runs on GitHub."
        }
        if health.domain.is_empty() {
            p { class: "muted", "This snapshot has no repo health badges." }
        }
        nav { class: "domain-index",
            for domain in health.domain.iter() {
                a { key: "{domain.name}", href: "#{anchor(&domain.name)}", "{domain.name}" }
            }
        }
        for domain in health.domain {
            DomainSection { key: "{domain.name}", domain, commit: commit.clone() }
        }
        p { class: "muted",
            "A red, amber or green light for every project, with the evidence behind it, comes in Phase 4 of the review dashboard plan."
        }
    }
}

#[component]
fn DomainSection(domain: Domain, commit: String) -> Element {
    let id = anchor(&domain.name);
    rsx! {
        section { class: "group", id: "{id}", "data-testid": "health-section",
            h2 { "{domain.name} " span { class: "muted count", "{domain.row.len()}" } }
            p { class: "muted", "{domain.summary}" }
            ul { class: "rows",
                for row in domain.row {
                    CheckRow { key: "{row.area}", row, commit: commit.clone() }
                }
            }
        }
    }
}

#[component]
fn CheckRow(row: Row, commit: String) -> Element {
    let id = anchor(&row.area);
    let checks = markdown::to_html(&row.checks, "README.md", &commit);
    let red = markdown::to_html(&row.red, "README.md", &commit);
    let runs = markdown::to_html(&format!("Runs: {}", row.runs), "README.md", &commit);
    let page = row.pane.as_deref().and_then(pane_page);
    rsx! {
        li { id: "{id}", class: "check", "data-testid": "health-check",
            p { class: "row-text",
                strong { "{row.area}" }
                " "
                for badge in row.badge {
                    BadgeImage { key: "{badge.image}", badge }
                }
            }
            div { class: "check-text muted", dangerous_inner_html: "{runs}" }
            div { class: "check-text", dangerous_inner_html: "{checks}" }
            div { class: "check-text", strong { "Red means: " } span { dangerous_inner_html: "{red}" } }
            if let Some((to, label)) = page {
                p { "In pane: " Link { to, "{label}" } }
            }
        }
    }
}

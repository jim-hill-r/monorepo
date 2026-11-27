//! The Approve page: the review queue from decision 0014. It reads GitHub's
//! public API from the browser, without a token, so it needs no secret and
//! can only read: approving still happens on GitHub, one tap from each row.

use dioxus::prelude::*;
use pane_model::queue::{self, Queue, QueueItem, Stack};

use crate::{now, view};

/// GET a public GitHub API URL. No custom headers, so the browser sends a
/// simple CORS request with no preflight.
pub(crate) async fn get_text(url: &str) -> Result<String, String> {
    let response = gloo_net::http::Request::get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    match response.status() {
        200 => response.text().await.map_err(|e| e.to_string()),
        403 | 429 => Err(
            "GitHub's limit for reading without signing in (60 calls an hour per network) \
             is used up. Try again in a few minutes."
                .to_owned(),
        ),
        status => Err(format!("GitHub returned {status}")),
    }
}

async fn load_queue() -> Result<(Queue, i64), String> {
    let pulls = queue::parse_pulls(&get_text(queue::PULLS_URL).await?)?;
    let ready = queue::parse_ready(&get_text(queue::READY_SEARCH_URL).await?)?;
    Ok((queue::build(&pulls, &ready), now()))
}

#[component]
pub fn Approve() -> Element {
    let mut loaded = use_resource(load_queue);
    let state = loaded.read().clone();
    rsx! {
        h1 { "Approve" }
        p { class: "muted",
            "Pull requests whose checks passed and that wait on your review, clear ones first. "
            "Approve opens the pull request's changes on GitHub: Review changes, Approve, Submit. "
            "Stacks get one approval for the whole stack."
        }
        match state {
            None => rsx! { p { class: "muted", "Loading pull requests from GitHub…" } },
            Some(Err(e)) => rsx! {
                div { class: "error", role: "alert",
                    p { "Couldn't load the pull requests." }
                    p { class: "muted", "{e}" }
                }
                button { r#type: "button", onclick: move |_| loaded.restart(), "Try again" }
            },
            Some(Ok((q, at))) => rsx! {
                QueueBody { queue: q, at }
                button { r#type: "button", onclick: move |_| loaded.restart(), "Refresh" }
            },
        }
    }
}

#[component]
fn QueueBody(queue: Queue, at: i64) -> Element {
    let q = queue;
    let now = now();
    let approve = q.approve.len();
    let decide = q.decide.len();
    rsx! {
        ul { class: "stats", "data-testid": "queue-counts",
            li { span { class: "stat", strong { "{approve}" } " to approve" } }
            li { span { class: "stat", strong { "{decide}" } " need a decision" } }
            li { span { class: "stat", strong { "{q.waiting}" } " waiting on checks" } }
        }
        section { class: "group",
            h2 { "Ready to approve" }
            if q.approve.is_empty() {
                p { "Nothing is waiting for your approval." }
            }
            ul { class: "rows", "data-testid": "queue-approve",
                for item in q.approve {
                    QueueRow { key: "{item.number}", item, now }
                }
            }
        }
        if !q.decide.is_empty() {
            section { class: "group",
                h2 { "Needs a decision first" }
                p { class: "muted", "Labelled needs-human or credential-change: read what the pull request asks you to decide." }
                ul { class: "rows", "data-testid": "queue-decide",
                    for item in q.decide {
                        QueueRow { key: "{item.number}", item, now }
                    }
                }
            }
        }
        if !q.stacks.is_empty() {
            section { class: "group",
                h2 { "Stacks" }
                p { class: "muted",
                    "Stacked pull requests can only reach main together. Their thread opens one pull request "
                    "for the whole stack into main, and that one approval covers it (decision 0014)."
                }
                ul { class: "rows", "data-testid": "queue-stacks",
                    for stack in q.stacks {
                        StackRow { key: "{stack.root}", stack }
                    }
                }
            }
        }
        p { class: "muted", "{q.drafts} drafts not shown. Loaded from GitHub {view::age(at, now)}." }
    }
}

#[component]
fn QueueRow(item: QueueItem, now: i64) -> Element {
    let age = view::days_since(&item.created_at, now).map(view::days_label);
    let word = item.attention.word();
    rsx! {
        li { "data-attention": "{word}",
            a { class: "row-title", href: "{item.url}", "#{item.number} {item.title}" }
            if !item.summary.is_empty() {
                p { class: "row-text", "{item.summary}" }
            }
            div { class: "meta",
                span { class: "badge attention-{word}", "{word}" }
                for label in item.badges.iter() {
                    span { class: "badge attention-{word}", "{label}" }
                }
                if item.covers > 1 {
                    span { class: "badge", "covers {item.covers} pull requests" }
                }
                if let Some(age) = age {
                    span { "{age}" }
                }
                span { "{item.author}" }
                a { class: "approve", href: "{item.review_url}", "Approve on GitHub" }
            }
        }
    }
}

#[component]
fn StackRow(stack: Stack) -> Element {
    let n = stack.numbers.len();
    rsx! {
        li {
            p { class: "row-text", "{n} pull requests on {stack.base}" }
            div { class: "meta",
                for number in stack.numbers {
                    a { href: "{pane_model::REPO_URL}/pull/{number}", "#{number}" }
                }
            }
        }
    }
}

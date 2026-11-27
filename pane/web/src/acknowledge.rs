//! The Acknowledge page: post-merge review (decision 0014). Low-risk PRs
//! merge without Jim's approval, so he acknowledges every merged PR here
//! after the fact, or flags one for a follow-up issue. Acknowledgements are
//! kept in this browser's `localStorage`.

use dioxus::prelude::*;
use pane_model::acks::{self, AckStore, Merged};

use crate::{Loaded, approve::get_text, now, storage, view};

/// Today in UTC, `YYYY-MM-DD`, from the browser's clock.
fn today() -> String {
    let iso: String = js_sys::Date::new_0().to_iso_string().into();
    iso.get(..10).unwrap_or_default().to_owned()
}

fn load_store() -> AckStore {
    let stored = storage().and_then(|s| s.get_item(acks::STORAGE_KEY).ok().flatten());
    AckStore::parse(stored.as_deref(), &today())
}

/// Save the store; false when the browser keeps no storage.
fn save_store(store: &AckStore) -> bool {
    let Some(storage) = storage() else {
        return false;
    };
    let Ok(json) = serde_json::to_string(store) else {
        return false;
    };
    storage.set_item(acks::STORAGE_KEY, &json).is_ok()
}

async fn load_merged(since: String) -> Result<(Vec<Merged>, i64), String> {
    let mut all = Vec::new();
    for page in 1..=acks::MAX_PAGES {
        let items = acks::parse_merged(&get_text(&acks::merged_search_url(&since, page)).await?)?;
        let full = items.len() == acks::PER_PAGE;
        all.extend(items);
        if !full {
            break;
        }
    }
    Ok((all, now()))
}

#[component]
pub fn Acknowledge() -> Element {
    let mut store = use_signal(load_store);
    let since = store.read().since.clone();
    let mut merged = use_resource(move || load_merged(since.clone()));
    let loaded = use_context::<Memo<Loaded>>();
    let projects: Vec<String> = match &*loaded.read() {
        Some(Ok(s)) => s.projects.iter().map(|p| p.path.clone()).collect(),
        _ => Vec::new(),
    };
    let kept = storage().is_some();
    let state = merged.read().clone();
    let since = store.read().since.clone();
    rsx! {
        h1 { "Acknowledge" }
        p { class: "muted",
            "Every pull request merged since {since}, newest first, for you to read after the fact. "
            "Acknowledge the ones that look right; flag one to open a follow-up issue file on GitHub."
        }
        if !kept {
            p { class: "error", role: "alert", "This browser isn't keeping pane's storage, so acknowledgements won't be remembered." }
        }
        match state {
            None => rsx! { p { class: "muted", "Loading merged pull requests from GitHub…" } },
            Some(Err(e)) => rsx! {
                div { class: "error", role: "alert",
                    p { "Couldn't load the merged pull requests." }
                    p { class: "muted", "{e}" }
                }
                button { r#type: "button", onclick: move |_| merged.restart(), "Try again" }
            },
            Some(Ok((all, at))) => {
                let left = acks::unacknowledged(&all, &store.read());
                let count = left.len();
                let done = all.len() - count;
                let numbers: Vec<u64> = left.iter().map(|m| m.number).collect();
                let today = today();
                rsx! {
                    ul { class: "stats", "data-testid": "ack-counts",
                        li { span { class: "stat", strong { "{count}" } " to acknowledge" } }
                        li { span { class: "stat", strong { "{done}" } " acknowledged" } }
                    }
                    if left.is_empty() {
                        p { "Every merged pull request is acknowledged." }
                    } else {
                        button {
                            r#type: "button",
                            onclick: move |_| {
                                let mut next = store.read().clone();
                                next.acked.extend(numbers.iter().copied());
                                save_store(&next);
                                store.set(next);
                            },
                            "Acknowledge all {count}"
                        }
                    }
                    ul { class: "rows", "data-testid": "ack-list",
                        for m in left {
                            AckRow {
                                key: "{m.number}",
                                flag: acks::flag_url(&m, &projects, &today),
                                m,
                                on_ack: move |n: u64| {
                                    let mut next = store.read().clone();
                                    next.acked.insert(n);
                                    save_store(&next);
                                    store.set(next);
                                },
                            }
                        }
                    }
                    p { class: "muted", "Loaded from GitHub {view::age(at, now())}." }
                    button { r#type: "button", onclick: move |_| merged.restart(), "Refresh" }
                }
            }
        }
    }
}

#[component]
fn AckRow(m: Merged, flag: String, on_ack: EventHandler<u64>) -> Element {
    let word = m.attention.word();
    let merged_on = m.merged_at.get(..10).unwrap_or_default().to_owned();
    let number = m.number;
    rsx! {
        li { "data-attention": "{word}",
            a { class: "row-title", href: "{m.url}", "#{m.number} {m.title}" }
            if !m.summary.is_empty() {
                p { class: "row-text", "{m.summary}" }
            }
            div { class: "meta",
                span { class: "badge attention-{word}", "{word}" }
                for label in m.badges.iter() {
                    span { class: "badge attention-{word}", "{label}" }
                }
                span { "Merged {merged_on}" }
                span { "{m.author}" }
                a { href: "{m.url}/files", "Changes" }
                a { href: "{flag}", "Flag for follow-up" }
                button { r#type: "button", class: "ack", onclick: move |_| on_ack.call(number), "Acknowledge" }
            }
        }
    }
}

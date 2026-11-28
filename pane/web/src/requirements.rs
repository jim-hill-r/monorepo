//! Each Project page's Requirements section: every requirement's verdict
//! from the eval engine, each eval's latest result with its evidence link,
//! and its recent runs ("Eval engine: requirements and feedback" in the
//! review dashboard plan). The data is the snapshot's `requirements`
//! (`pane_model::requirements`), read from the `evals-results` branch.

use dioxus::prelude::*;
use pane_model::requirements::{EvalRun, EvalRuns, Requirement};
use pane_model::{Health, Snapshot, signals};

use crate::view;

/// The class suffix and label of a light, such as `health-amber`.
pub fn health_class(health: Health) -> &'static str {
    match health {
        Health::Grey => "grey",
        Health::Green => "green",
        Health::Amber => "amber",
        Health::Red => "red",
    }
}

/// How a light reads in words.
pub fn health_label(health: Health) -> &'static str {
    match health {
        Health::Grey => "Grey",
        Health::Green => "Green",
        Health::Amber => "Amber",
        Health::Red => "Red",
    }
}

/// An RFC 3339 UTC time as `2026-11-27 04:25 UTC`; anything shorter as is.
pub fn when(at: &str) -> String {
    match (at.get(..10), at.get(11..16)) {
        (Some(day), Some(time)) => format!("{day} {time} UTC"),
        _ => at.to_owned(),
    }
}

/// One run in words, for a history dot's label: `pass, 2026-11-27 04:25 UTC`.
pub fn run_label(run: &EvalRun) -> String {
    let mut label = format!("{}, {}", run.status.label(), when(&run.at));
    if let Some(note) = &run.note {
        label.push_str(": ");
        label.push_str(note);
    }
    label
}

#[component]
pub fn ProjectRequirementsSection(snapshot: Snapshot, path: String) -> Element {
    let signal = signals::requirements(&snapshot, &path);
    let top = path.split('/').next().unwrap_or(&path).to_owned();
    let doc = format!("{top}/docs/requirements.md");
    let doc_url = view::github_file_url(&snapshot.commit, &doc);
    let generated = snapshot
        .requirements
        .as_ref()
        .map(|r| when(&r.generated_at));
    let reqs: Vec<Requirement> = snapshot
        .requirements
        .as_ref()
        .and_then(|r| r.project(&top))
        .map(|p| p.requirements.clone())
        .unwrap_or_default();
    let class = health_class(signal.health);
    let label = health_label(signal.health);
    rsx! {
        section { class: "group", "data-testid": "project-requirements",
            h2 { "Requirements" }
            p { "data-testid": "project-requirements-light",
                span { class: "badge health-{class}", "{label}" }
                " {signal.reason}"
            }
            match generated {
                None => rsx! {
                    p { class: "muted", "This snapshot was built without the eval results, so there are no verdicts." }
                },
                Some(_) if reqs.is_empty() => rsx! {
                    p { class: "muted",
                        "No requirements yet. A project lists them in docs/requirements.md (see standards/docs/evals.md)."
                    }
                },
                Some(at) => rsx! {
                    p { class: "muted",
                        "Verdicts from {at}, for "
                        a { href: "{doc_url}", "{doc}" }
                        "."
                    }
                    ul { class: "rows",
                        for r in reqs {
                            RequirementRow { key: "{r.id}", requirement: r }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn RequirementRow(requirement: Requirement) -> Element {
    let r = requirement;
    let class = health_class(r.verdict.health());
    rsx! {
        li { "data-testid": "requirement",
            div { class: "row-title",
                span { class: "badge health-{class}", "{r.verdict.label()}" }
                " {r.id}: {r.title}"
                span { class: "muted", " {r.priority}" }
            }
            if !r.reasons.is_empty() {
                ul { class: "reasons",
                    for reason in r.reasons.clone() {
                        li { key: "{reason}", "{reason}" }
                    }
                }
            }
            if r.evals.is_empty() {
                p { class: "meta", "No eval results yet." }
            }
            for e in r.evals.clone() {
                EvalLine { key: "{e.eval}/{e.stage}", runs: e }
            }
        }
    }
}

#[component]
fn EvalLine(runs: EvalRuns) -> Element {
    let latest = runs.latest().cloned();
    rsx! {
        div { class: "meta eval", "data-testid": "requirement-eval",
            span { class: "eval-name", "{runs.eval} ({runs.stage})" }
            if let Some(run) = latest {
                span {
                    "latest "
                    span { class: "run-status run-{run.status.label()}", "{run.status.label()}" }
                    " {when(&run.at)}"
                    if let Some(note) = &run.note {
                        ", {note}"
                    }
                }
                if let Some(url) = &run.evidence {
                    a { href: "{url}", "Evidence" }
                }
            }
            span { class: "history", "aria-label": "Last {runs.runs.len()} runs, newest first",
                for (i , run) in runs.runs.iter().enumerate() {
                    match &run.evidence {
                        Some(url) => rsx! {
                            a { key: "{i}", class: "run run-{run.status.label()}", href: "{url}", title: "{run_label(run)}", "aria-label": "{run_label(run)}" }
                        },
                        None => rsx! {
                            span { key: "{i}", class: "run run-{run.status.label()}", title: "{run_label(run)}", "aria-label": "{run_label(run)}" }
                        },
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pane_model::requirements::RunStatus;

    #[test]
    fn formats_times_and_runs() {
        assert_eq!(when("2026-11-27T04:25:07.84Z"), "2026-11-27 04:25 UTC");
        assert_eq!(when("2026-11"), "2026-11");
        let run = EvalRun {
            status: RunStatus::Fail,
            at: "2026-11-27T04:25:07Z".into(),
            commit: "abc".into(),
            evidence: None,
            note: Some("2 of 3 tests failed".into()),
        };
        assert_eq!(
            run_label(&run),
            "fail, 2026-11-27 04:25 UTC: 2 of 3 tests failed"
        );
    }

    #[test]
    fn lights_have_classes_and_labels() {
        assert_eq!(health_class(Health::Amber), "amber");
        assert_eq!(health_label(Health::Grey), "Grey");
    }
}

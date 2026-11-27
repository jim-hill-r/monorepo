//! `evals publish`: writes the job summary table from a results file.
//! `evals store` (store.rs) adds them to the `evals-results` branch.

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use evals_model::{EvalResult, ResultStatus, requirement_order};

/// Parses a JSONL results file. Blank lines are ignored.
pub fn parse_results(text: &str) -> Result<Vec<EvalResult>> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            serde_json::from_str(line).with_context(|| format!("results line {}", index + 1))
        })
        .collect()
}

/// One JSONL line per result.
pub fn to_jsonl(results: &[EvalResult]) -> Result<String> {
    let mut out = String::new();
    for result in results {
        out.push_str(&serde_json::to_string(result)?);
        out.push('\n');
    }
    Ok(out)
}

/// A Markdown summary: one table per project and stage, requirements in
/// file order, with counts in the heading.
pub fn summary(results: &[EvalResult]) -> String {
    if results.is_empty() {
        return "## Evals\n\nNo eval results.\n".to_string();
    }
    let mut groups: BTreeMap<(&str, &str), Vec<&EvalResult>> = BTreeMap::new();
    for result in results {
        groups
            .entry((result.project.as_str(), result.stage.as_str()))
            .or_default()
            .push(result);
    }
    let mut out = String::new();
    for ((project, stage), mut rows) in groups {
        rows.sort_by(|a, b| {
            requirement_order(&a.requirement)
                .cmp(&requirement_order(&b.requirement))
                .then(a.eval.cmp(&b.eval))
        });
        let count = |status| rows.iter().filter(|r| r.status == status).count();
        out.push_str(&format!(
            "## Evals: {project} ({stage})\n\n{} pass, {} fail, {} error, {} skip\n\n",
            count(ResultStatus::Pass),
            count(ResultStatus::Fail),
            count(ResultStatus::Error),
            count(ResultStatus::Skip),
        ));
        out.push_str(
            "| Requirement | Eval | Status | Note | Evidence |\n| --- | --- | --- | --- | --- |\n",
        );
        for row in rows {
            let evidence = row
                .evidence
                .as_deref()
                .map(|url| format!("[run]({url})"))
                .unwrap_or_default();
            out.push_str(&format!(
                "| {} | `{}` | {} | {} | {} |\n",
                row.requirement,
                row.eval,
                status_label(row.status),
                cell(row.note.as_deref().unwrap_or_default()),
                evidence
            ));
        }
        out.push('\n');
    }
    out
}

fn status_label(status: ResultStatus) -> &'static str {
    match status {
        ResultStatus::Pass => "✅ pass",
        ResultStatus::Fail => "❌ fail",
        ResultStatus::Error => "⚠️ error",
        ResultStatus::Skip => "⏭️ skip",
    }
}

/// Keeps a value inside one Markdown table cell.
fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const RESULTS: &str = r#"
{"project":"cookbook","requirement":"COOK-010","eval":"b","stage":"ci","kind":"cargo","commit":"c","status":"fail","note":"1 of 2 tests failed: a|b","at":"2026-12-01T00:00:00Z","by":"ci"}
{"project":"cookbook","requirement":"COOK-002","eval":"a","stage":"ci","kind":"playwright","commit":"c","status":"pass","evidence":"https://x/runs/1","at":"2026-12-01T00:00:00Z","by":"ci"}

{"project":"cookbook","requirement":"COOK-002","eval":"c","stage":"ci","kind":"cargo","commit":"c","status":"skip","at":"2026-12-01T00:00:00Z","by":"ci"}
{"project":"pane","requirement":"PANE-001","eval":"d","stage":"ci","kind":"cargo","commit":"c","status":"error","at":"2026-12-01T00:00:00Z","by":"ci"}
"#;

    #[test]
    fn writes_a_table_per_project() -> Result<()> {
        let results = parse_results(RESULTS)?;
        assert_eq!(results.len(), 4);
        let text = summary(&results);
        let expected = "\
## Evals: cookbook (ci)

1 pass, 1 fail, 0 error, 1 skip

| Requirement | Eval | Status | Note | Evidence |
| --- | --- | --- | --- | --- |
| COOK-002 | `a` | ✅ pass |  | [run](https://x/runs/1) |
| COOK-002 | `c` | ⏭️ skip |  |  |
| COOK-010 | `b` | ❌ fail | 1 of 2 tests failed: a\\|b |  |

## Evals: pane (ci)

0 pass, 0 fail, 1 error, 0 skip

| Requirement | Eval | Status | Note | Evidence |
| --- | --- | --- | --- | --- |
| PANE-001 | `d` | ⚠️ error |  |  |

";
        assert_eq!(text, expected);
        Ok(())
    }

    #[test]
    fn an_empty_results_file_says_so() -> Result<()> {
        assert_eq!(
            summary(&parse_results("\n")?),
            "## Evals\n\nNo eval results.\n"
        );
        Ok(())
    }

    #[test]
    fn reports_the_bad_line() {
        let error = parse_results("{}\n").err().map(|e| e.to_string());
        assert_eq!(error.as_deref(), Some("results line 1"));
    }

    #[test]
    fn round_trips_jsonl() -> Result<()> {
        let results = parse_results(RESULTS)?;
        assert_eq!(parse_results(&to_jsonl(&results)?)?, results);
        Ok(())
    }
}

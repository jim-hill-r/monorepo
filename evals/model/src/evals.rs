//! Parser for `<project>/evals.toml`.
//!
//! ```toml
//! prefix = "COOK"
//!
//! [[eval]]
//! id = "recipe-mobile-layout"
//! requirement = "COOK-001"
//! stages = ["ci", "cd"]
//! kind = "playwright"
//! grep = "@req:COOK-001"
//! ```

use serde::{Deserialize, Serialize};

use crate::ParseError;
use crate::ids::{is_kebab_case, parse_requirement_id, snake_case_id};

/// When an eval runs. See the Stages table in `standards/docs/evals.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Ci,
    Cd,
    Chaos,
    Observe,
    Judge,
    /// Jim's own judgment: his rating in pane (implicit for every
    /// requirement, never listed) and his answers to `human-check` evals,
    /// the only kind that lists this stage.
    Human,
}

impl Stage {
    /// `cd`, `chaos` and `observe` run against production.
    pub fn is_production(self) -> bool {
        matches!(self, Stage::Cd | Stage::Chaos | Stage::Observe)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Stage::Ci => "ci",
            Stage::Cd => "cd",
            Stage::Chaos => "chaos",
            Stage::Observe => "observe",
            Stage::Judge => "judge",
            Stage::Human => "human",
        }
    }
}

/// What an eval runs, with the fields each kind adds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Kind {
    /// `cargo test` results; test names start with `filter`.
    Cargo {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<String>,
    },
    /// Playwright JSON reporter results; test titles contain `grep`.
    Playwright {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        grep: Option<String>,
    },
    /// CLI transcript tests; case files match `glob`.
    Trycmd {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        glob: Option<String>,
    },
    /// A named check in a Bevy scripted run.
    Script { script: String, check: String },
    /// One HTTP request with an expected response.
    Http {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        method: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        status: Option<u16>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        header: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        body_contains: Option<String>,
        /// The slowest acceptable response, in milliseconds.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_ms: Option<u64>,
    },
    /// A threshold on observability data over a window.
    Metric {
        source: String,
        metric: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        window: String,
    },
    /// A Playwright test run with a fault injected.
    Fault {
        fault: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        grep: Option<String>,
    },
    /// An agent grades the live product against a rubric.
    Judge { rubric: String },
    /// Jim checks a setting made by hand; `setting` is an `id` in
    /// `standards/settings.toml`.
    HumanCheck { setting: String },
    /// The project's cost analysis holds: `cost check` finds nothing that
    /// can fail in its `docs/cost.toml` (stage `ci`), and its actual usage
    /// stays within what that file expects (stage `observe`).
    Cost {},
}

impl Kind {
    pub fn name(&self) -> &'static str {
        match self {
            Kind::Cargo { .. } => "cargo",
            Kind::Playwright { .. } => "playwright",
            Kind::Trycmd { .. } => "trycmd",
            Kind::Script { .. } => "script",
            Kind::Http { .. } => "http",
            Kind::Metric { .. } => "metric",
            Kind::Fault { .. } => "fault",
            Kind::Judge { .. } => "judge",
            Kind::HumanCheck { .. } => "human-check",
            Kind::Cost {} => "cost",
        }
    }

    /// The fields this kind accepts besides the common ones.
    fn fields(name: &str) -> Option<&'static [&'static str]> {
        Some(match name {
            "cargo" => &["filter"],
            "playwright" => &["grep"],
            "trycmd" => &["glob"],
            "script" => &["script", "check"],
            "http" => &[
                "url",
                "path",
                "method",
                "status",
                "header",
                "body_contains",
                "max_ms",
            ],
            "metric" => &["source", "metric", "min", "max", "window"],
            "fault" => &["fault", "grep"],
            "judge" => &["rubric"],
            "human-check" => &["setting"],
            "cost" => &[],
            _ => return None,
        })
    }
}

/// Observability sources a `metric` eval can read.
pub const METRIC_SOURCES: [&str; 4] = ["web-analytics", "analytics-engine", "posthog", "github"];

const COMMON_FIELDS: [&str; 5] = ["id", "requirement", "stages", "kind", "description"];

/// One `[[eval]]` table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Eval {
    pub id: String,
    pub requirement: String,
    pub stages: Vec<Stage>,
    #[serde(flatten)]
    pub kind: Kind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Eval {
    /// True if any of the eval's stages runs against production.
    pub fn runs_in_production(&self) -> bool {
        self.stages.iter().any(|s| s.is_production())
    }

    /// How tests are matched to this eval for the kinds that read test
    /// reports, with the documented defaults filled in.
    pub fn test_selector(&self) -> Option<String> {
        match &self.kind {
            Kind::Cargo { filter } => Some(
                filter
                    .clone()
                    .unwrap_or_else(|| snake_case_id(&self.requirement)),
            ),
            Kind::Playwright { grep } | Kind::Fault { grep, .. } => Some(
                grep.clone()
                    .unwrap_or_else(|| format!("@req:{}", self.requirement)),
            ),
            Kind::Trycmd { glob } => Some(
                glob.clone()
                    .unwrap_or_else(|| format!("tests/cmd/{}-*.toml", self.requirement)),
            ),
            _ => None,
        }
    }
}

/// A parsed `evals.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalsFile {
    pub prefix: String,
    #[serde(default, rename = "eval")]
    pub evals: Vec<Eval>,
}

/// Parses and validates an `evals.toml`.
///
/// Returns `Err` with every problem found. Cross-file checks (does the
/// requirement exist, are IDs unique) are left to the caller.
pub fn parse_evals(text: &str) -> Result<EvalsFile, Vec<ParseError>> {
    let table: toml::Table = toml::from_str(text).map_err(|e| vec![toml_error(text, &e)])?;

    let mut errors = check_keys(&table);
    let file: EvalsFile = match EvalsFile::deserialize(toml::Value::Table(table)) {
        Ok(file) => file,
        Err(e) => {
            errors.push(ParseError::whole_file(e.message().trim().to_string()));
            return Err(errors);
        }
    };

    if !valid_prefix(&file.prefix) {
        errors.push(ParseError::whole_file(format!(
            "prefix `{}` must be 2 to 5 uppercase letters",
            file.prefix
        )));
    }
    for eval in &file.evals {
        errors.extend(
            check_eval(eval, &file.prefix)
                .into_iter()
                .map(|message| ParseError::whole_file(format!("eval `{}`: {message}", eval.id))),
        );
    }

    if errors.is_empty() {
        Ok(file)
    } else {
        Err(errors)
    }
}

fn valid_prefix(prefix: &str) -> bool {
    (2..=5).contains(&prefix.len()) && prefix.chars().all(|c| c.is_ascii_uppercase())
}

/// Reports keys no kind accepts, which serde would otherwise ignore.
fn check_keys(table: &toml::Table) -> Vec<ParseError> {
    let mut errors = Vec::new();
    for key in table.keys() {
        if key != "prefix" && key != "eval" {
            errors.push(ParseError::whole_file(format!(
                "unknown top-level key `{key}`"
            )));
        }
    }
    let Some(evals) = table.get("eval").and_then(|v| v.as_array()) else {
        return errors;
    };
    for (index, eval) in evals.iter().enumerate() {
        let Some(eval) = eval.as_table() else {
            continue;
        };
        let name = eval
            .get("id")
            .and_then(|v| v.as_str())
            .map_or_else(|| format!("#{}", index + 1), str::to_string);
        let kind = eval
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let Some(kind_fields) = Kind::fields(kind) else {
            errors.push(ParseError::whole_file(format!(
                "eval `{name}`: unknown kind `{kind}`; expected cargo, playwright, trycmd, script, http, metric, fault, judge, human-check or cost"
            )));
            continue;
        };
        for key in eval.keys() {
            if !COMMON_FIELDS.contains(&key.as_str()) && !kind_fields.contains(&key.as_str()) {
                errors.push(ParseError::whole_file(format!(
                    "eval `{name}`: `{key}` is not a field of a {kind} eval"
                )));
            }
        }
    }
    errors
}

fn check_eval(eval: &Eval, prefix: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if !is_kebab_case(&eval.id) {
        problems.push("id must be kebab-case".to_string());
    }
    match parse_requirement_id(&eval.requirement) {
        None => problems.push(format!(
            "requirement `{}` is not an ID like {prefix}-001",
            eval.requirement
        )),
        Some((p, _)) if p != prefix => problems.push(format!(
            "requirement `{}` is not a {prefix} requirement",
            eval.requirement
        )),
        Some(_) => {}
    }
    if eval.stages.is_empty() {
        problems.push("stages is empty".to_string());
    }
    let mut seen = Vec::new();
    for stage in &eval.stages {
        if *stage == Stage::Human && !matches!(eval.kind, Kind::HumanCheck { .. }) {
            problems.push(
                "stage `human` is listed only by human-check evals (Jim's rating is implicit)"
                    .to_string(),
            );
        }
        if seen.contains(stage) {
            problems.push(format!("stage `{}` is listed twice", stage.as_str()));
        }
        seen.push(*stage);
    }
    if matches!(eval.kind, Kind::HumanCheck { .. }) && eval.stages != [Stage::Human] {
        problems.push("a human-check eval runs in the human stage only".into());
    }
    if eval.stages.contains(&Stage::Judge) != matches!(eval.kind, Kind::Judge { .. }) {
        problems.push("a judge eval runs in the judge stage only, and only judge evals do".into());
    }
    if matches!(eval.kind, Kind::Cost {})
        && eval
            .stages
            .iter()
            .any(|s| !matches!(s, Stage::Ci | Stage::Observe))
    {
        problems.push("a cost eval runs in the ci and observe stages only".into());
    }

    match &eval.kind {
        Kind::Script { script, check } => {
            require_text(&mut problems, "script", script);
            require_text(&mut problems, "check", check);
        }
        Kind::Http {
            url,
            path,
            method,
            status,
            header,
            max_ms,
            ..
        } => {
            match (url, path) {
                (Some(_), Some(_)) => problems.push("set url or path, not both".into()),
                (None, None) => problems.push("an http eval needs url or path".into()),
                (Some(url), None)
                    if !url.starts_with("https://") && !url.starts_with("http://") =>
                {
                    problems.push("url must start with https:// or http://".into());
                }
                (None, Some(path)) if !path.starts_with('/') => {
                    problems.push("path must start with /".into());
                }
                _ => {}
            }
            if let Some(method) = method
                && (method.is_empty() || !method.chars().all(|c| c.is_ascii_uppercase()))
            {
                problems.push(format!("method `{method}` must be uppercase, like GET"));
            }
            if let Some(status) = status
                && !(100..=599).contains(status)
            {
                problems.push(format!("status {status} is not an HTTP status"));
            }
            if let Some(header) = header
                && !header
                    .split_once(':')
                    .is_some_and(|(name, _)| !name.trim().is_empty())
            {
                problems.push(format!("header `{header}` must be \"name: value\""));
            }
            if *max_ms == Some(0) {
                problems.push("max_ms must be more than 0".into());
            }
        }
        Kind::Metric {
            source,
            metric,
            min,
            max,
            window,
        } => {
            if !METRIC_SOURCES.contains(&source.as_str()) {
                problems.push(format!(
                    "source `{source}` is not one of {}",
                    METRIC_SOURCES.join(", ")
                ));
            }
            require_text(&mut problems, "metric", metric);
            match (min, max) {
                (None, None) => problems.push("a metric eval needs min, max or both".into()),
                (Some(min), Some(max)) if min > max => {
                    problems.push("min is greater than max".into());
                }
                _ => {}
            }
            if !valid_window(window) {
                problems.push(format!(
                    "window `{window}` must be a number of hours or days, like 1h or 7d"
                ));
            }
        }
        Kind::Fault { fault, .. } => require_text(&mut problems, "fault", fault),
        Kind::Judge { rubric } => require_text(&mut problems, "rubric", rubric),
        Kind::HumanCheck { setting } => {
            if !is_kebab_case(setting) {
                problems.push(format!(
                    "setting `{setting}` is not a kebab-case id from standards/settings.toml"
                ));
            }
        }
        Kind::Cargo { .. } | Kind::Playwright { .. } | Kind::Trycmd { .. } | Kind::Cost {} => {}
    }
    problems
}

fn require_text(problems: &mut Vec<String>, field: &str, value: &str) {
    if value.trim().is_empty() {
        problems.push(format!("{field} is empty"));
    }
}

fn valid_window(window: &str) -> bool {
    let Some(number) = window
        .strip_suffix('h')
        .or_else(|| window.strip_suffix('d'))
    else {
        return false;
    };
    !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()) && number != "0"
}

fn toml_error(text: &str, error: &toml::de::Error) -> ParseError {
    let message = error.message().trim().to_string();
    match error.span() {
        Some(span) => {
            let line = text
                .get(..span.start)
                .map_or(0, |before| before.matches('\n').count())
                + 1;
            ParseError::at(line, message)
        }
        None => ParseError::whole_file(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COOKBOOK: &str = r#"
prefix = "COOK"

[[eval]]
id = "recipe-mobile-layout"
requirement = "COOK-001"
stages = ["ci", "cd"]
kind = "playwright"
grep = "@req:COOK-001"

[[eval]]
id = "recipe-lcp"
requirement = "COOK-001"
stages = ["observe"]
kind = "metric"
source = "web-analytics"
metric = "lcp_p75_ms"
max = 2500
window = "7d"

[[eval]]
id = "recipe-offline-after-load"
requirement = "COOK-004"
stages = ["chaos"]
kind = "fault"
fault = "offline"
grep = "@req:COOK-004"

[[eval]]
id = "recipe-voice"
requirement = "COOK-007"
stages = ["judge"]
kind = "judge"
rubric = "Reads like a friendly home cook, not a food magazine."

[[eval]]
id = "units-convert"
requirement = "COOK-002"
stages = ["ci"]
kind = "cargo"

[[eval]]
id = "home-page-up"
requirement = "COOK-001"
stages = ["cd"]
kind = "http"
path = "/"
body_contains = "Cookbook"
"#;

    fn errors(text: &str) -> Vec<String> {
        match parse_evals(text) {
            Ok(file) => panic!("expected errors, parsed {file:?}"),
            Err(errors) => errors.into_iter().map(|e| e.to_string()).collect(),
        }
    }

    fn one_eval(fields: &str) -> String {
        format!("prefix = \"COOK\"\n[[eval]]\nid = \"e\"\nrequirement = \"COOK-001\"\n{fields}\n")
    }

    #[test]
    fn parses_the_documented_example() {
        let file = match parse_evals(COOKBOOK) {
            Ok(file) => file,
            Err(errors) => panic!("{errors:?}"),
        };
        assert_eq!(file.prefix, "COOK");
        assert_eq!(file.evals.len(), 6);

        let layout = &file.evals[0];
        assert_eq!(layout.stages, vec![Stage::Ci, Stage::Cd]);
        assert!(layout.runs_in_production());
        assert_eq!(layout.test_selector().as_deref(), Some("@req:COOK-001"));

        assert_eq!(
            file.evals[1].kind,
            Kind::Metric {
                source: "web-analytics".into(),
                metric: "lcp_p75_ms".into(),
                min: None,
                max: Some(2500.0),
                window: "7d".into(),
            }
        );
        assert!(!file.evals[3].runs_in_production());
        assert_eq!(file.evals[4].test_selector().as_deref(), Some("cook_002"));
        assert_eq!(file.evals[5].kind.name(), "http");
    }

    #[test]
    fn a_file_with_no_evals_is_valid() {
        let file = parse_evals("prefix = \"PANE\"\n").map_err(|e| format!("{e:?}"));
        assert_eq!(file.map(|f| f.evals.len()), Ok(0));
    }

    #[test]
    fn default_test_selectors() {
        let file = parse_evals(&one_eval("stages = [\"ci\"]\nkind = \"trycmd\""))
            .map_err(|e| format!("{e:?}"));
        let selector = file.map(|f| f.evals[0].test_selector());
        assert_eq!(selector, Ok(Some("tests/cmd/COOK-001-*.toml".into())));

        let file = parse_evals(&one_eval("stages = [\"ci\"]\nkind = \"playwright\""))
            .map_err(|e| format!("{e:?}"));
        let selector = file.map(|f| f.evals[0].test_selector());
        assert_eq!(selector, Ok(Some("@req:COOK-001".into())));
    }

    #[test]
    fn reports_toml_syntax_errors_with_a_line() {
        let result = parse_evals("prefix = \"COOK\"\n[[eval]\n");
        let line = result.err().and_then(|e| e.first().and_then(|e| e.line));
        assert_eq!(line, Some(2));
    }

    #[test]
    fn rejects_a_bad_prefix() {
        assert_eq!(
            errors("prefix = \"cook\"\n"),
            vec!["prefix `cook` must be 2 to 5 uppercase letters"]
        );
    }

    #[test]
    fn rejects_unknown_keys_and_kinds() {
        assert_eq!(
            errors("prefix = \"COOK\"\nowner = \"jim\"\n"),
            vec!["unknown top-level key `owner`"]
        );
        let found = errors(&one_eval(
            "stages = [\"ci\"]\nkind = \"cargo\"\ngrep = \"x\"",
        ));
        assert_eq!(
            found,
            vec!["eval `e`: `grep` is not a field of a cargo eval"]
        );

        let found = errors(&one_eval("stages = [\"ci\"]\nkind = \"selenium\""));
        assert!(found[0].contains("unknown kind `selenium`"), "{found:?}");
    }

    #[test]
    fn rejects_missing_kind_fields() {
        let found = errors(&one_eval("stages = [\"judge\"]\nkind = \"judge\""));
        assert!(found[0].contains("rubric"), "{found:?}");
    }

    #[test]
    fn checks_ids_and_requirements() {
        let text = "prefix = \"COOK\"\n[[eval]]\nid = \"Bad_Id\"\nrequirement = \"PANE-001\"\nstages = [\"ci\"]\nkind = \"cargo\"\n";
        assert_eq!(
            errors(text),
            vec![
                "eval `Bad_Id`: id must be kebab-case",
                "eval `Bad_Id`: requirement `PANE-001` is not a COOK requirement",
            ]
        );
        let text = "prefix = \"COOK\"\n[[eval]]\nid = \"e\"\nrequirement = \"COOK-1\"\nstages = [\"ci\"]\nkind = \"cargo\"\n";
        assert!(errors(text)[0].contains("is not an ID like COOK-001"));
    }

    #[test]
    fn checks_stages() {
        assert_eq!(
            errors(&one_eval("stages = []\nkind = \"cargo\"")),
            vec!["eval `e`: stages is empty"]
        );
        assert_eq!(
            errors(&one_eval("stages = [\"ci\", \"ci\"]\nkind = \"cargo\"")),
            vec!["eval `e`: stage `ci` is listed twice"]
        );
        assert!(
            errors(&one_eval("stages = [\"human\"]\nkind = \"cargo\""))[0].contains("implicit")
        );
        assert!(errors(&one_eval("stages = [\"judge\"]\nkind = \"cargo\""))[0].contains("judge"));
        assert!(
            errors(&one_eval(
                "stages = [\"cd\"]\nkind = \"judge\"\nrubric = \"r\""
            ))[0]
                .contains("judge")
        );
        assert!(
            errors(&one_eval("stages = [\"nightly\"]\nkind = \"cargo\""))[0].contains("nightly")
        );
    }

    #[test]
    fn checks_human_check_evals() {
        let text = "prefix = \"COOK\"\n[[eval]]\nid = \"e\"\nrequirement = \"COOK-001\"\nstages = [\"human\"]\nkind = \"human-check\"\nsetting = \"auth0-cookbook-app\"\n";
        let file = match parse_evals(text) {
            Ok(file) => file,
            Err(e) => panic!("{e:?}"),
        };
        assert_eq!(
            file.evals[0].kind,
            Kind::HumanCheck {
                setting: "auth0-cookbook-app".into()
            }
        );
        assert_eq!(file.evals[0].kind.name(), "human-check");

        assert_eq!(
            errors(&one_eval(
                "stages = [\"human\", \"cd\"]\nkind = \"human-check\"\nsetting = \"a\""
            )),
            vec!["eval `e`: a human-check eval runs in the human stage only"]
        );
        assert!(
            errors(&one_eval(
                "stages = [\"human\"]\nkind = \"human-check\"\nsetting = \"Not An Id\""
            ))[0]
                .contains("kebab-case")
        );
        assert!(
            errors(&one_eval("stages = [\"human\"]\nkind = \"human-check\""))[0]
                .contains("setting"),
        );
    }

    #[test]
    fn checks_cost_evals() {
        let file = match parse_evals(&one_eval("stages = [\"ci\", \"observe\"]\nkind = \"cost\"")) {
            Ok(file) => file,
            Err(e) => panic!("{e:?}"),
        };
        assert_eq!(file.evals[0].kind, Kind::Cost {});
        assert_eq!(file.evals[0].kind.name(), "cost");
        assert!(file.evals[0].runs_in_production());
        assert_eq!(file.evals[0].test_selector(), None);

        for stages in ["\"cd\"", "\"ci\", \"chaos\""] {
            assert_eq!(
                errors(&one_eval(&format!("stages = [{stages}]\nkind = \"cost\""))),
                vec!["eval `e`: a cost eval runs in the ci and observe stages only"],
                "{stages}"
            );
        }
        assert_eq!(
            errors(&one_eval("stages = [\"ci\"]\nkind = \"cost\"\nmax = 0")),
            vec!["eval `e`: `max` is not a field of a cost eval"]
        );
    }

    #[test]
    fn checks_http_evals() {
        let cases = [
            ("", "needs url or path"),
            ("url = \"https://a\"\npath = \"/\"", "not both"),
            ("url = \"ftp://a\"", "must start with https://"),
            ("path = \"about\"", "path must start with /"),
            ("path = \"/\"\nmethod = \"get\"", "uppercase"),
            ("path = \"/\"\nstatus = 999", "not an HTTP status"),
            ("path = \"/\"\nheader = \"text/html\"", "\"name: value\""),
            ("path = \"/\"\nmax_ms = 0", "max_ms must be more than 0"),
        ];
        for (fields, expected) in cases {
            let found = errors(&one_eval(&format!(
                "stages = [\"cd\"]\nkind = \"http\"\n{fields}"
            )));
            assert!(found[0].contains(expected), "{fields}: {found:?}");
        }
    }

    #[test]
    fn checks_metric_evals() {
        let base = "stages = [\"observe\"]\nkind = \"metric\"\nmetric = \"m\"";
        let cases = [
            (
                "source = \"datadog\"\nmax = 1\nwindow = \"7d\"",
                "source `datadog`",
            ),
            (
                "source = \"github\"\nwindow = \"7d\"",
                "needs min, max or both",
            ),
            (
                "source = \"github\"\nmin = 5\nmax = 1\nwindow = \"7d\"",
                "min is greater",
            ),
            (
                "source = \"github\"\nmax = 1\nwindow = \"1w\"",
                "window `1w`",
            ),
            (
                "source = \"github\"\nmax = 1\nwindow = \"0d\"",
                "window `0d`",
            ),
        ];
        for (fields, expected) in cases {
            let found = errors(&one_eval(&format!("{base}\n{fields}")));
            assert!(found[0].contains(expected), "{fields}: {found:?}");
        }
    }

    #[test]
    fn round_trips_through_json() {
        let file = parse_evals(COOKBOOK).map_err(|e| format!("{e:?}"));
        let json = file
            .as_ref()
            .map(|f| serde_json::to_value(&f.evals[1]).unwrap_or_default());
        let json = json.unwrap_or_default();
        assert_eq!(json["kind"], "metric");
        assert_eq!(json["max"], 2500.0);
        assert_eq!(json["stages"][0], "observe");
    }
}

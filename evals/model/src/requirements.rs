//! Parser for `<project>/docs/requirements.md`.
//!
//! Format (see `standards/docs/evals.md`):
//!
//! ```markdown
//! # Cookbook requirements
//!
//! ## COOK-001: A recipe is easy to cook from on a phone
//!
//! - Priority: must
//! - Why: Jim cooks from his phone, one hand free.
//! - Accept: no sideways scrolling at 375 px wide;
//!   body text at least 16 px.
//! - Status: retired (2026-12-01, nobody prints recipes)
//! ```

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::ParseError;
use crate::ids::parse_requirement_id;

/// How much a requirement matters. Only `must` can turn a project red.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Must,
    Should,
    Could,
}

/// Whether a requirement still applies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "status")]
pub enum Status {
    Active,
    Retired { date: NaiveDate, why: String },
}

/// One `## <PREFIX>-<nnn>: <title>` section of a requirements file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub title: String,
    pub priority: Priority,
    pub why: String,
    pub accept: String,
    pub status: Status,
    /// 1-based line of the heading.
    pub line: usize,
}

impl Requirement {
    pub fn is_active(&self) -> bool {
        self.status == Status::Active
    }

    /// The prefix of the ID, such as `COOK` for `COOK-001`.
    pub fn prefix(&self) -> &str {
        self.id.split('-').next().unwrap_or_default()
    }
}

const FIELDS: [&str; 4] = ["Priority", "Why", "Accept", "Status"];

struct Section {
    id: String,
    title: String,
    line: usize,
    /// (field name, value, line), in file order.
    fields: Vec<(String, String, usize)>,
}

/// Parses a requirements file.
///
/// Returns every requirement that parsed and every problem found, so one bad
/// section doesn't hide the others. Text above the first `##` heading is
/// free-form.
pub fn parse_requirements(text: &str) -> (Vec<Requirement>, Vec<ParseError>) {
    let mut errors = Vec::new();
    let mut sections: Vec<Section> = Vec::new();
    let mut in_code_block = false;

    for (index, raw) in text.lines().enumerate() {
        let line_no = index + 1;
        let line = raw.trim_end();

        if line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            if in_code_block && !sections.is_empty() {
                errors.push(ParseError::at(
                    line_no,
                    "code blocks are not allowed inside a requirement",
                ));
            }
            continue;
        }
        if in_code_block {
            continue;
        }

        if let Some(heading) = line.strip_prefix("## ") {
            match parse_heading(heading) {
                Ok((id, title)) => sections.push(Section {
                    id,
                    title,
                    line: line_no,
                    fields: Vec::new(),
                }),
                Err(message) => errors.push(ParseError::at(line_no, message)),
            }
            continue;
        }

        let Some(section) = sections.last_mut() else {
            // Free-form text before the first requirement.
            continue;
        };

        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            errors.push(ParseError::at(
                line_no,
                format!(
                    "unexpected heading in {}: only `## <PREFIX>-<nnn>: <title>` headings are allowed after the first requirement",
                    section.id
                ),
            ));
            continue;
        }
        if let Some(item) = line.strip_prefix("- ") {
            match item.split_once(':') {
                Some((name, value)) if FIELDS.contains(&name) => {
                    section
                        .fields
                        .push((name.to_string(), value.trim().to_string(), line_no));
                }
                _ => errors.push(ParseError::at(
                    line_no,
                    format!(
                        "unknown field in {}: expected one of Priority, Why, Accept, Status",
                        section.id
                    ),
                )),
            }
            continue;
        }
        if raw.starts_with(' ') || raw.starts_with('\t') {
            match section.fields.last_mut() {
                Some((_, value, _)) => {
                    value.push(' ');
                    value.push_str(line.trim());
                }
                None => errors.push(ParseError::at(
                    line_no,
                    format!("indented text in {} before any field", section.id),
                )),
            }
            continue;
        }
        errors.push(ParseError::at(
            line_no,
            format!(
                "unexpected text in {}: a requirement holds only its field list",
                section.id
            ),
        ));
    }

    let mut requirements = Vec::new();
    for section in sections {
        match build_requirement(section) {
            Ok(requirement) => requirements.push(requirement),
            Err(mut section_errors) => errors.append(&mut section_errors),
        }
    }
    errors.sort_by_key(|e| e.line);
    (requirements, errors)
}

fn parse_heading(heading: &str) -> Result<(String, String), String> {
    let malformed = || {
        format!(
            "malformed requirement heading `## {heading}`: expected `## <PREFIX>-<nnn>: <title>`"
        )
    };
    let (id, title) = heading.split_once(':').ok_or_else(malformed)?;
    if parse_requirement_id(id).is_none() {
        return Err(malformed());
    }
    let title = title.trim();
    if title.is_empty() {
        return Err(format!("requirement {id} has no title"));
    }
    Ok((id.to_string(), title.to_string()))
}

fn build_requirement(section: Section) -> Result<Requirement, Vec<ParseError>> {
    let mut errors = Vec::new();
    let id = &section.id;

    // Fields must appear in FIELDS order, each at most once.
    let mut last_index = None;
    for (name, _, line) in &section.fields {
        let index = FIELDS.iter().position(|f| f == name);
        if let (Some(index), Some(last)) = (index, last_index) {
            if index == last {
                errors.push(ParseError::at(*line, format!("{id} has {name} twice")));
            } else if index < last {
                errors.push(ParseError::at(
                    *line,
                    format!("{id}: fields must be in the order Priority, Why, Accept, Status"),
                ));
            }
        }
        last_index = index.max(last_index);
    }

    let field = |name: &str| {
        section
            .fields
            .iter()
            .find(|(n, _, _)| n == name)
            .map(|(_, value, line)| (value.as_str(), *line))
    };

    let mut required = |name: &str| match field(name) {
        Some((value, _)) if !value.is_empty() => Some(value.to_string()),
        Some((_, line)) => {
            errors.push(ParseError::at(line, format!("{id}: {name} is empty")));
            None
        }
        None => {
            errors.push(ParseError::at(
                section.line,
                format!("{id} has no {name} field"),
            ));
            None
        }
    };
    let priority_text = required("Priority");
    let why = required("Why");
    let accept = required("Accept");

    let priority = priority_text.and_then(|p| match p.as_str() {
        "must" => Some(Priority::Must),
        "should" => Some(Priority::Should),
        "could" => Some(Priority::Could),
        other => {
            let line = field("Priority").map_or(section.line, |(_, l)| l);
            errors.push(ParseError::at(
                line,
                format!("{id}: priority `{other}` is not must, should or could"),
            ));
            None
        }
    });

    let status = match field("Status") {
        None => Some(Status::Active),
        Some((value, line)) => match parse_retired(value) {
            Some(status) => Some(status),
            None => {
                errors.push(ParseError::at(
                    line,
                    format!("{id}: status must be `retired (<yyyy-mm-dd>, <why>)`"),
                ));
                None
            }
        },
    };

    match (priority, why, accept, status) {
        (Some(priority), Some(why), Some(accept), Some(status)) if errors.is_empty() => {
            Ok(Requirement {
                id: section.id,
                title: section.title,
                priority,
                why,
                accept,
                status,
                line: section.line,
            })
        }
        _ => Err(errors),
    }
}

fn parse_retired(value: &str) -> Option<Status> {
    let inner = value.strip_prefix("retired (")?.strip_suffix(')')?;
    let (date, why) = inner.split_once(',')?;
    let date = NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d").ok()?;
    let why = why.trim();
    if why.is_empty() {
        return None;
    }
    Some(Status::Retired {
        date,
        why: why.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const COOKBOOK: &str = "\
# Cookbook requirements

Notes above the first requirement are free-form.

## COOK-001: A recipe is easy to cook from on a phone

- Priority: must
- Why: Jim cooks from his phone, one hand free.
- Accept: no sideways scrolling at 375 px wide; body text at least 16 px;
  the recipe is readable within 2.5 s on a 4G phone (LCP p75).

## COOK-002: Recipes can be found by ingredient

- Priority: should
- Why: Jim plans meals around what is already in the fridge.
- Accept: searching \"leek\" lists every recipe with leeks within 1 s.

## COOK-003: Recipes can be printed

- Priority: could
- Why: For cooking away from a phone.
- Accept: printing a recipe fits it on one page with no navigation.
- Status: retired (2026-12-01, nobody prints recipes)
";

    fn parse_ok(text: &str) -> Vec<Requirement> {
        let (requirements, errors) = parse_requirements(text);
        assert_eq!(errors, vec![], "unexpected errors");
        requirements
    }

    fn only_error(text: &str) -> ParseError {
        let (_, errors) = parse_requirements(text);
        assert_eq!(errors.len(), 1, "{errors:?}");
        errors[0].clone()
    }

    #[test]
    fn parses_the_documented_example() {
        let requirements = parse_ok(COOKBOOK);
        assert_eq!(requirements.len(), 3);

        let first = &requirements[0];
        assert_eq!(first.id, "COOK-001");
        assert_eq!(first.title, "A recipe is easy to cook from on a phone");
        assert_eq!(first.priority, Priority::Must);
        assert_eq!(first.why, "Jim cooks from his phone, one hand free.");
        assert_eq!(
            first.accept,
            "no sideways scrolling at 375 px wide; body text at least 16 px; \
             the recipe is readable within 2.5 s on a 4G phone (LCP p75)."
        );
        assert!(first.is_active());
        assert_eq!(first.line, 5);
        assert_eq!(first.prefix(), "COOK");

        assert_eq!(requirements[1].priority, Priority::Should);

        let retired = &requirements[2];
        assert_eq!(retired.priority, Priority::Could);
        assert!(!retired.is_active());
        assert_eq!(
            retired.status,
            Status::Retired {
                date: NaiveDate::from_ymd_opt(2026, 12, 1).unwrap_or_default(),
                why: "nobody prints recipes".into(),
            }
        );
    }

    #[test]
    fn an_empty_file_has_no_requirements() {
        assert_eq!(parse_ok(""), vec![]);
        assert_eq!(parse_ok("# Pane requirements\n\nNone yet.\n"), vec![]);
    }

    #[test]
    fn rejects_a_malformed_heading() {
        for heading in [
            "## COOK-1: Short number",
            "## cook-001: Lowercase",
            "## COOK-001 Missing colon",
            "## Notes",
        ] {
            let error = only_error(heading);
            assert_eq!(error.line, Some(1));
            assert!(error.message.contains("malformed"), "{error}");
        }
    }

    #[test]
    fn rejects_a_heading_without_a_title() {
        let error = only_error("## COOK-001: \n- Priority: must\n- Why: x\n- Accept: y\n");
        assert!(error.message.contains("no title"), "{error}");
    }

    #[test]
    fn reports_missing_fields() {
        let (requirements, errors) = parse_requirements("## COOK-001: Title\n\n- Priority: must\n");
        assert!(requirements.is_empty());
        let messages: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
        assert_eq!(
            messages,
            vec!["COOK-001 has no Why field", "COOK-001 has no Accept field"]
        );
    }

    #[test]
    fn reports_an_empty_field() {
        let error = only_error("## COOK-001: T\n- Priority: must\n- Why:\n- Accept: y\n");
        assert_eq!(error.line, Some(3));
        assert!(error.message.contains("Why is empty"), "{error}");
    }

    #[test]
    fn rejects_an_unknown_priority() {
        let error = only_error("## COOK-001: T\n- Priority: high\n- Why: x\n- Accept: y\n");
        assert_eq!(error.line, Some(2));
        assert!(error.message.contains("`high`"), "{error}");
    }

    #[test]
    fn rejects_fields_out_of_order_or_repeated() {
        let error = only_error("## COOK-001: T\n- Why: x\n- Priority: must\n- Accept: y\n");
        assert!(error.message.contains("order"), "{error}");

        let error =
            only_error("## COOK-001: T\n- Priority: must\n- Why: x\n- Why: z\n- Accept: y\n");
        assert!(error.message.contains("twice"), "{error}");
    }

    #[test]
    fn rejects_an_unknown_field() {
        let error =
            only_error("## COOK-001: T\n- Priority: must\n- Owner: Jim\n- Why: x\n- Accept: y\n");
        assert_eq!(error.line, Some(3));
        assert!(error.message.contains("unknown field"), "{error}");
    }

    #[test]
    fn rejects_a_bad_status() {
        for status in [
            "- Status: active",
            "- Status: retired",
            "- Status: retired (yesterday, gone)",
            "- Status: retired (2026-12-01, )",
        ] {
            let text =
                format!("## COOK-001: T\n- Priority: must\n- Why: x\n- Accept: y\n{status}\n");
            let error = only_error(&text);
            assert!(error.message.contains("retired ("), "{error}");
        }
    }

    #[test]
    fn rejects_other_headings_and_prose_inside_requirements() {
        let base = "## COOK-001: T\n- Priority: must\n- Why: x\n- Accept: y\n";
        let error = only_error(&format!("{base}### Details\n"));
        assert!(error.message.contains("unexpected heading"), "{error}");

        let error = only_error(&format!("{base}\nA paragraph.\n"));
        assert!(error.message.contains("unexpected text"), "{error}");

        let error = only_error(&format!("{base}```\ncode\n```\n"));
        assert!(error.message.contains("code blocks"), "{error}");
    }

    #[test]
    fn ignores_headings_inside_code_blocks_before_the_first_requirement() {
        let text = "# Notes\n\n```markdown\n## NOT-A-HEADING\n```\n\n## COOK-001: T\n- Priority: must\n- Why: x\n- Accept: y\n";
        let requirements = parse_ok(text);
        assert_eq!(requirements.len(), 1);
    }

    #[test]
    fn keeps_parsing_after_a_bad_section() {
        let text = "## COOK-001: Bad\n- Priority: huge\n- Why: x\n- Accept: y\n\n## COOK-002: Good\n- Priority: must\n- Why: x\n- Accept: y\n";
        let (requirements, errors) = parse_requirements(text);
        assert_eq!(errors.len(), 1);
        assert_eq!(requirements.len(), 1);
        assert_eq!(requirements[0].id, "COOK-002");
    }

    #[test]
    fn serializes_to_json() {
        let requirements = parse_ok(COOKBOOK);
        let json = serde_json::to_value(&requirements[2]).unwrap_or_default();
        assert_eq!(json["priority"], "could");
        assert_eq!(json["status"]["status"], "retired");
        assert_eq!(json["status"]["date"], "2026-12-01");
    }
}

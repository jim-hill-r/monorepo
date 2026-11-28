//! `chores pr-body`: write a pull request description in the shape
//! `standards/docs/pull-requests.md` asks for, and check one (PR-001, the
//! same rules as `standards/scripts/pr-summary-check.sh`).
//!
//! The shape (Jim, 2026-11-27), after the attribution lines: an Attention
//! section of tags from [`crate::badges::Tag`], then What changed and Why
//! (at most 50 words each), Cost changes and Risk changes (under 25 words
//! each), then Details (anything else, no limit).
//!
//! A planning PR, one that adds or edits a plan or decision record
//! ([`badges::is_plan_file`]), can't say "None." for cost (Jim,
//! 2026-11-28): its Cost changes section estimates the plan's cost as
//! implemented across all current projects, at 5x and at 100x as many
//! (`Now`, `5x` and `100x`), as "Cost at scale" in
//! `standards/docs/cost.md` says.

use crate::badges::{self, Group, Tag};

/// A summary section and its word limit. `max` is inclusive, so "under 25
/// words" is 24.
#[derive(Debug, Clone, Copy)]
pub struct Section {
    pub name: &'static str,
    pub max: usize,
}

/// The four summary sections, in order.
pub const SUMMARIES: [Section; 4] = [
    Section {
        name: "What changed",
        max: 50,
    },
    Section {
        name: "Why",
        max: 50,
    },
    Section {
        name: "Cost changes",
        max: 24,
    },
    Section {
        name: "Risk changes",
        max: 24,
    },
];

/// The free-form last section.
pub const DETAILS: &str = "Details";

impl Section {
    fn limit(self) -> String {
        if self.max == 24 {
            "under 25 words".to_string()
        } else {
            format!("at most {} words", self.max)
        }
    }

    fn defaults_to_none(self) -> bool {
        self.name.starts_with("Cost") || self.name.starts_with("Risk")
    }
}

/// What goes into a description.
#[derive(Debug, Default)]
pub struct Parts {
    /// GitHub login for the `claude-projects-attribution` comment.
    pub login: Option<String>,
    /// Who asked for the work, for the "Requested by" line.
    pub requested_by: Option<String>,
    /// The project thread link for the "Requested by" line.
    pub thread_url: Option<String>,
    /// The Attention section (`chores badges`), right under the attribution.
    pub attention: Option<String>,
    pub what: String,
    pub why: String,
    pub cost: String,
    pub risk: String,
    pub details: String,
    /// Lines the agent platform wants at the end, such as a session link.
    pub footer: Option<String>,
}

pub fn render(parts: &Parts) -> String {
    let mut s = String::new();
    if let Some(login) = &parts.login {
        s.push_str(&format!(
            "<!-- claude-projects-attribution: {{\"github_login\":\"{login}\"}} -->\n"
        ));
    }
    if let Some(who) = &parts.requested_by {
        match &parts.thread_url {
            Some(url) => s.push_str(&format!(
                "_Requested by **{who}** · [project thread]({url})_\n"
            )),
            None => s.push_str(&format!("_Requested by **{who}**_\n")),
        }
    }
    if !s.is_empty() {
        s.push('\n');
    }
    if let Some(section) = &parts.attention {
        s.push_str(section.trim());
        s.push_str("\n\n");
    }
    let bodies = [&parts.what, &parts.why, &parts.cost, &parts.risk];
    for (section, text) in SUMMARIES.iter().zip(bodies) {
        s.push_str(&format!("## {}\n\n{}\n\n", section.name, text.trim()));
    }
    s.push_str(&format!("## {DETAILS}\n\n{}\n", parts.details.trim()));
    if let Some(footer) = parts.footer.as_deref().filter(|f| !f.trim().is_empty()) {
        s.push('\n');
        s.push_str(footer.trim());
        s.push('\n');
    }
    s
}

/// Removes `<!-- ... -->` comments, including multi-line ones.
fn strip_comments(body: &str) -> String {
    let mut out = String::new();
    let mut rest = body;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        match rest[start..].find("-->") {
            Some(end) => rest = &rest[start + end + 3..],
            None => {
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

fn is_rule(line: &str) -> bool {
    matches!(line.trim_end(), "---" | "***" | "___")
}

/// The text under a heading named `name` (any level, any case), up to the
/// next heading or rule. `None` when the description has no such heading.
pub fn section(body: &str, name: &str) -> Option<String> {
    let body = strip_comments(&body.replace('\r', ""));
    let mut found = false;
    let mut current = false;
    let mut out = Vec::new();
    for line in body.lines() {
        let heading = line.trim_start_matches('#');
        if heading.len() < line.len() && heading.starts_with(char::is_whitespace) {
            current = heading.trim().eq_ignore_ascii_case(name);
            found |= current;
            continue;
        }
        if is_rule(line) {
            current = false;
            continue;
        }
        if current {
            out.push(line);
        }
    }
    found.then(|| out.join("\n"))
}

/// Words in a section. Markdown links count as their text; link targets
/// and bare URLs don't count.
pub fn word_count(text: &str) -> usize {
    let mut plain = String::new();
    let mut rest = text;
    while let Some(i) = rest.find("](") {
        plain.push_str(&rest[..=i]);
        match rest[i..].find(')') {
            Some(end) => rest = &rest[i + end + 1..],
            None => {
                rest = "";
            }
        }
    }
    plain.push_str(rest);
    plain
        .split_whitespace()
        .filter(|w| !(w.starts_with("http://") || w.starts_with("https://")))
        .count()
}

fn is_placeholder(text: &str) -> bool {
    let t = text.trim();
    t.starts_with('<') && t.ends_with('>')
}

/// Warnings for the Attention section: every bullet is a known tag, with
/// exactly one requirements tag, one cost tag, one threat/risk tag and one
/// docs tag.
fn check_attention(body: &str, warnings: &mut Vec<String>) {
    let Some(text) = section(body, "Attention") else {
        warnings.push(
            "The description has no \"## Attention\" section; run `chores badges` and put its \
             output first."
                .to_string(),
        );
        return;
    };
    let mut groups = Vec::new();
    for line in text.lines().filter(|l| l.trim().starts_with("- ")) {
        let name = badges::bullet_tag(line).unwrap_or("");
        match Tag::parse(name) {
            Some(tag) => groups.push(tag.group()),
            None => {
                let known: Vec<&str> = Tag::ALL.iter().map(|t| t.as_str()).collect();
                warnings.push(format!(
                    "\"{name}\" is not an attention tag; use one of {}.",
                    known.join(", ")
                ));
            }
        }
    }
    for (group, label) in [
        (Group::Requirements, "requirements"),
        (Group::Cost, "cost"),
        (Group::ThreatRisk, "threat/risk"),
        (Group::Docs, "docs"),
    ] {
        let n = groups.iter().filter(|g| **g == group).count();
        if n != 1 {
            warnings.push(format!(
                "The \"## Attention\" section has {n} {label} tags; it needs exactly one."
            ));
        }
    }
}

/// The scale markers a planning PR's Cost changes section names, each as
/// a whole word (letters and digits on neither side), in any case.
pub const SCALES: [&str; 3] = ["now", "5x", "100x"];

/// Which of [`SCALES`] the text leaves out. `×` counts as `x`.
pub fn missing_scales(text: &str) -> Vec<&'static str> {
    let text = text.to_ascii_lowercase().replace('×', "x");
    let words: Vec<&str> = text
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    SCALES.into_iter().filter(|s| !words.contains(s)).collect()
}

/// The warning for a planning PR whose Cost changes section lacks a scaled
/// estimate; `None` when it has all three or the PR changes no plan.
fn check_plan_costs(body: &str, plans: &[String]) -> Option<String> {
    let first = plans.first()?;
    let text = section(body, "Cost changes")?;
    let missing = missing_scales(&text);
    if missing.is_empty() {
        return None;
    }
    let more = if plans.len() > 1 {
        format!(" and {} more", plans.len() - 1)
    } else {
        String::new()
    };
    Some(format!(
        "This PR changes a plan or decision (`{first}`{more}), so \"## Cost changes\" must \
         estimate its cost as implemented across all current projects, at 5x and at 100x as \
         many: name Now, 5x and 100x (missing {}). See \"Cost at scale\" in \
         standards/docs/cost.md.",
        missing.join(", ")
    ))
}

/// PR-001: the word count of each summary section and the warnings for a
/// description; no warnings when it passes. `plans` are the plans and
/// decision records the PR adds or edits ([`badges::Attention::plans`]).
pub fn check(body: &str, plans: &[String]) -> (Vec<(&'static str, usize)>, Vec<String>) {
    let mut counts = Vec::new();
    let mut warnings = Vec::new();
    check_attention(body, &mut warnings);
    for s in SUMMARIES {
        let Some(text) = section(body, s.name) else {
            warnings.push(format!("The description has no \"## {}\" section.", s.name));
            continue;
        };
        let words = word_count(&text);
        counts.push((s.name, words));
        if is_placeholder(&text) {
            warnings.push(format!(
                "The \"## {}\" section is still the template's placeholder.",
                s.name
            ));
        } else if words == 0 {
            if s.defaults_to_none() {
                warnings.push(format!(
                    "The \"## {}\" section is empty; write \"None.\" when nothing changes.",
                    s.name
                ));
            } else {
                warnings.push(format!(
                    "The \"## {}\" section is empty; keep it to {}.",
                    s.name,
                    s.limit()
                ));
            }
        } else if words > s.max {
            warnings.push(format!(
                "The \"## {}\" section is {words} words; keep it to {} and move the rest to \
                 Details or a linked doc.",
                s.name,
                s.limit()
            ));
        }
    }
    warnings.extend(check_plan_costs(body, plans));
    if section(body, DETAILS).is_none() {
        warnings.push(format!("The description has no \"## {DETAILS}\" section."));
    }
    (counts, warnings)
}

/// The ok line, matching the shell check's.
pub fn ok_line(counts: &[(&str, usize)]) -> String {
    let parts: Vec<String> = counts.iter().map(|(n, w)| format!("{n} {w}")).collect();
    format!("PR-001 ok: {} words.", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::badges::Attention;

    /// PR-001 for a PR that changes no plan.
    fn check(body: &str) -> (Vec<(&'static str, usize)>, Vec<String>) {
        super::check(body, &[])
    }

    fn parts() -> Parts {
        Parts {
            login: Some("jimhillr-agt".into()),
            requested_by: Some("Jim".into()),
            thread_url: Some("https://claude.ai/code/project/x?thread=y".into()),
            attention: Some(badges::render(&Attention::default())),
            what: "Adds chores. See [the plan](standards/docs/plans/x.md).".into(),
            why: "Routine work costs tokens.".into(),
            cost: "None.".into(),
            risk: "None.".into(),
            details: "Hooks passed.".into(),
            footer: Some("Generated with Claude Code".into()),
        }
    }

    fn words(n: usize) -> String {
        "word ".repeat(n)
    }

    #[test]
    fn renders_the_template_shape() {
        let body = render(&parts());
        assert_eq!(
            body,
            "<!-- claude-projects-attribution: {\"github_login\":\"jimhillr-agt\"} -->\n\
             _Requested by **Jim** · [project thread](https://claude.ai/code/project/x?thread=y)_\n\
             \n\
             ## Attention\n\
             \n\
             - 🟢 requirements-unchanged\n\
             - 🟢 cost-unchanged\n\
             - 🟢 threat-risk-unchanged\n\
             - 🟢 docs-unchanged\n\
             \n\
             ## What changed\n\nAdds chores. See [the plan](standards/docs/plans/x.md).\n\n\
             ## Why\n\nRoutine work costs tokens.\n\n\
             ## Cost changes\n\nNone.\n\n\
             ## Risk changes\n\nNone.\n\n\
             ## Details\n\nHooks passed.\n\
             \n\
             Generated with Claude Code\n"
        );
        let (counts, warnings) = check(&body);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(
            ok_line(&counts),
            "PR-001 ok: What changed 5, Why 4, Cost changes 1, Risk changes 1 words."
        );
    }

    #[test]
    fn word_limits() {
        let mut p = parts();
        p.what = words(50);
        p.why = words(50);
        p.cost = words(24);
        p.risk = words(24);
        assert!(check(&render(&p)).1.is_empty());
        p.what = words(51);
        p.cost = words(25);
        let w = check(&render(&p)).1;
        assert!(w[0].contains("\"## What changed\" section is 51 words; keep it to at most 50"));
        assert!(w[1].contains("\"## Cost changes\" section is 25 words; keep it to under 25"));
    }

    #[test]
    fn empty_and_missing_sections() {
        let mut p = parts();
        p.risk = String::new();
        assert!(check(&render(&p)).1[0].contains("write \"None.\""));
        let (_, w) = check("Old summary, see [doc](a.md).\n\n## Testing\n\nRan it.\n");
        assert!(w[0].contains("no \"## Attention\" section"));
        assert!(
            w.iter()
                .any(|w| w.contains("no \"## What changed\" section"))
        );
        assert!(w.iter().any(|w| w.contains("no \"## Details\" section")));
    }

    #[test]
    fn attention_tags_are_a_closed_set() {
        let body = render(&parts()).replace("threat-risk-unchanged", "looks-fine");
        let w = check(&body).1;
        assert!(w[0].starts_with("\"looks-fine\" is not an attention tag"));
        assert!(w[1].contains("0 threat/risk tags"));
        let doubled = render(&parts()).replace(
            "- 🟢 threat-risk-unchanged\n",
            "- 🟢 threat-risk-unchanged\n- 🟠 threat-risk-changed: `x`\n",
        );
        assert!(check(&doubled).1[0].contains("2 threat/risk tags"));
    }

    #[test]
    fn link_targets_and_urls_do_not_count() {
        assert_eq!(
            word_count("See [the long plan](a/b/c.md) https://example.com/z now"),
            5
        );
    }

    #[test]
    fn sections_any_case_end_at_a_rule_and_skip_comments() {
        let body =
            "### what changed\n<!-- guidance words -->\nTwo words\n\n---\n\nMore text here.\n";
        assert_eq!(
            section(body, "What changed").as_deref(),
            Some("\nTwo words\n")
        );
        assert_eq!(section(body, "Why"), None);
    }

    #[test]
    fn planning_prs_need_scaled_cost_estimates() {
        let plans = vec!["standards/docs/plans/x.md".to_string()];
        let w = super::check(&render(&parts()), &plans).1;
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w[0].contains("`standards/docs/plans/x.md`"), "{w:?}");
        assert!(w[0].contains("missing now, 5x, 100x"), "{w:?}");
        let mut p = parts();
        p.cost = "Now (27 projects): $0. 5x: $0, D1 at 40%. 100X: $5/mo, Workers Paid.".into();
        assert!(super::check(&render(&p), &plans).1.is_empty());
        p.cost = "Now $0; 5× $0; 1.5x nothing else.".into();
        let w = super::check(&render(&p), &plans).1;
        assert!(w[0].contains("missing 100x"), "{w:?}");
        // Not a planning PR: None. is fine.
        assert!(super::check(&render(&parts()), &[]).1.is_empty());
    }

    #[test]
    fn scale_markers_are_whole_words() {
        assert_eq!(missing_scales("nowhere 15x 1100x"), ["now", "5x", "100x"]);
        assert!(missing_scales("NOW: $0 | 5x: $0 | 100x: $3").is_empty());
    }

    #[test]
    fn template_is_flagged_as_placeholder() {
        let template = include_str!("../../.github/pull_request_template.md");
        let w = check(template).1;
        assert!(w.iter().any(|w| w.contains("0 requirements tags")), "{w:?}");
        assert!(
            w.iter()
                .any(|w| w
                    .contains("\"## What changed\" section is still the template's placeholder")),
            "{w:?}"
        );
    }
}

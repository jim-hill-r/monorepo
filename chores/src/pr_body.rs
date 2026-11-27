//! `chores pr-body`: write a pull request description in the shape
//! `standards/docs/pull-requests.md` asks for, and check one (PR-001, the
//! same rules as `standards/scripts/pr-summary-check.sh`).

/// The word limit for the executive summary (Jim, 2026-11-27).
pub const MAX_WORDS: usize = 75;

/// What goes into a description.
#[derive(Debug, Default)]
pub struct Parts {
    /// GitHub login for the `claude-projects-attribution` comment.
    pub login: Option<String>,
    /// Who asked for the work, for the "Requested by" line.
    pub requested_by: Option<String>,
    /// The project thread link for the "Requested by" line.
    pub thread_url: Option<String>,
    pub summary: String,
    pub testing: String,
    pub review_notes: Option<String>,
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
    s.push_str(parts.summary.trim());
    s.push_str("\n\n## Testing\n\n");
    s.push_str(parts.testing.trim());
    s.push('\n');
    if let Some(notes) = parts
        .review_notes
        .as_deref()
        .filter(|n| !n.trim().is_empty())
    {
        s.push_str("\n## Review notes\n\n");
        s.push_str(notes.trim());
        s.push('\n');
    }
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

/// The executive summary: the text after the attribution lines up to the
/// first heading or rule. A leading `## Summary` heading is skipped.
pub fn summary(body: &str) -> String {
    let body = strip_comments(&body.replace('\r', ""));
    let mut started = false;
    let mut out = Vec::new();
    for line in body.lines() {
        if !started {
            if line.starts_with("_Requested by ") || line.trim().is_empty() {
                continue;
            }
            let heading = line.trim_start_matches('#');
            if heading.len() < line.len()
                && heading.starts_with(char::is_whitespace)
                && heading.trim().eq_ignore_ascii_case("summary")
            {
                started = true;
                continue;
            }
        }
        if line.starts_with('#') || is_rule(line) {
            break;
        }
        started = true;
        out.push(line);
    }
    out.join("\n")
}

/// Words in the summary. Markdown links count as their text; link targets
/// and bare URLs don't count.
pub fn word_count(summary: &str) -> usize {
    let mut text = String::new();
    let mut rest = summary;
    while let Some(i) = rest.find("](") {
        text.push_str(&rest[..=i]);
        match rest[i..].find(')') {
            Some(end) => rest = &rest[i + end + 1..],
            None => {
                rest = "";
            }
        }
    }
    text.push_str(rest);
    text.split_whitespace()
        .filter(|w| !(w.starts_with("http://") || w.starts_with("https://")))
        .count()
}

/// PR-001 warnings for a description; empty when it passes.
pub fn check(body: &str) -> (usize, Vec<String>) {
    let summary = summary(body);
    let words = word_count(&summary);
    let mut warnings = Vec::new();
    if words == 0 {
        warnings.push(format!(
            "The description has no executive summary: open it, after the attribution lines, \
             with at most {MAX_WORDS} words on what, why and tradeoffs."
        ));
    } else if words > MAX_WORDS {
        warnings.push(format!(
            "The executive summary is {words} words; keep it to {MAX_WORDS} and link to \
             committed docs, plans or issues for the details."
        ));
    }
    if summary.contains("<Summary:") {
        warnings.push(
            "The executive summary is still the template's placeholder; replace it with what, \
             why and tradeoffs."
                .to_string(),
        );
    }
    if words > 0 && !(summary.contains("](") || summary.contains("http")) {
        warnings.push(
            "The executive summary has no link; point it at the committed docs, plan or issue \
             file that holds the details."
                .to_string(),
        );
    }
    (words, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(summary: &str) -> Parts {
        Parts {
            login: Some("jimhillr-agt".into()),
            requested_by: Some("Jim".into()),
            thread_url: Some("https://claude.ai/code/project/x?thread=y".into()),
            summary: summary.into(),
            testing: "Hooks passed.".into(),
            review_notes: None,
            footer: Some("Generated with Claude Code".into()),
        }
    }

    #[test]
    fn renders_the_template_shape() {
        let body = render(&parts(
            "Adds chores. See [the plan](standards/docs/plans/x.md).",
        ));
        assert_eq!(
            body,
            "<!-- claude-projects-attribution: {\"github_login\":\"jimhillr-agt\"} -->\n\
             _Requested by **Jim** · [project thread](https://claude.ai/code/project/x?thread=y)_\n\
             \n\
             Adds chores. See [the plan](standards/docs/plans/x.md).\n\
             \n\
             ## Testing\n\
             \n\
             Hooks passed.\n\
             \n\
             Generated with Claude Code\n"
        );
        let (words, warnings) = check(&body);
        assert_eq!(words, 5);
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn link_targets_and_urls_do_not_count() {
        assert_eq!(
            word_count("See [the long plan](a/b/c.md) https://example.com/z now"),
            5
        );
    }

    #[test]
    fn warns_like_the_shell_check() {
        let long = format!("{} [plan](p.md)", "word ".repeat(80));
        assert!(check(&render(&parts(&long))).1[0].contains("81 words"));
        assert!(check(&render(&parts("No link here."))).1[0].contains("no link"));
        let (_, w) = check("<!-- c -->\n\n<Summary: what, why. [plan](x)>\n\n## Testing\n");
        assert!(w.iter().any(|w| w.contains("placeholder")));
        let (words, w) = check("## Testing\n\nRan it.\n");
        assert_eq!(words, 0);
        assert!(w[0].contains("no executive summary"));
    }

    #[test]
    fn skips_a_summary_heading_and_stops_at_a_rule() {
        let body =
            "_Requested by **Jim**_\n\n## Summary\n\nTwo words [x](y)\n\n---\n\nMore text here.\n";
        assert_eq!(check(body).0, 3);
    }
}

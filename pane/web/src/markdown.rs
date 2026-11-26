//! Renders a plan, decision or retro's Markdown as HTML for the page.

use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};

use pane_model::REPO_URL;

/// HTML for `markdown`, the file at `path` in the repo at `commit`.
///
/// Raw HTML in the source is shown as text, never inserted, so a document
/// can't add scripts or styles to the page. Relative links and images point
/// at the file they name on GitHub, at the same commit.
pub fn to_html(markdown: &str, path: &str, commit: &str) -> String {
    let dir = path.rsplit_once('/').map_or("", |(d, _)| d);
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;
    let events = Parser::new_ext(markdown, options).map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: resolve(dest_url, dir, commit, "blob"),
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Image {
            link_type,
            dest_url: resolve(dest_url, dir, commit, "raw"),
            title,
            id,
        }),
        other => other,
    });
    let mut out = String::new();
    html::push_html(&mut out, events);
    out
}

/// Schemes a document link may not use, since they run code in the page.
const BLOCKED_SCHEMES: &[&str] = &["javascript", "vbscript", "data"];

/// A link target made safe and absolute: `javascript:`, `vbscript:` and
/// `data:` become `#`; other URLs with a scheme and `#` anchors stay as
/// they are; repo paths become GitHub URLs.
fn resolve<'a>(url: CowStr<'a>, dir: &str, commit: &str, view: &str) -> CowStr<'a> {
    let scheme = url.split_once(':').map(|(s, _)| s.trim()).filter(|s| {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
    });
    if scheme.is_some_and(|s| BLOCKED_SCHEMES.contains(&s.to_ascii_lowercase().as_str())) {
        return CowStr::Borrowed("#");
    }
    let has_scheme = scheme.is_some();
    if url.is_empty() || url.starts_with('#') || has_scheme || url.starts_with("//") {
        return url;
    }
    let (target, fragment) = match url.split_once('#') {
        Some((t, f)) => (t, format!("#{f}")),
        None => (url.as_ref(), String::new()),
    };
    let joined = if let Some(abs) = target.strip_prefix('/') {
        abs.to_owned()
    } else if dir.is_empty() {
        target.to_owned()
    } else {
        format!("{dir}/{target}")
    };
    let mut parts: Vec<&str> = Vec::new();
    for part in joined.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    CowStr::from(format!(
        "{REPO_URL}/{view}/{commit}/{}{fragment}",
        parts.join("/")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATH: &str = "pane/docs/plans/plan.md";

    #[test]
    fn renders_headings_lists_and_tables() {
        let html = to_html(
            "# Title\n\n- one\n- [x] done\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n",
            PATH,
            "c",
        );
        assert!(html.contains("<h1>Title</h1>"), "{html}");
        assert!(html.contains("<li>one</li>"), "{html}");
        assert!(html.contains("checkbox"), "{html}");
        assert!(html.contains("<td>2</td>"), "{html}");
    }

    #[test]
    fn shows_raw_html_as_text() {
        let html = to_html(
            "Hi <script>alert(1)</script>\n\n<div onclick=x>b</div>\n",
            PATH,
            "c",
        );
        assert!(!html.contains("<script"), "{html}");
        assert!(!html.contains("<div"), "{html}");
        assert!(html.contains("&lt;script&gt;"), "{html}");
    }

    #[test]
    fn points_relative_links_at_github() {
        let html = to_html(
            "[a](../../ISSUES.md) [b](./x.md#part) [c](/AGENTS.md) [d](https://e.com) [e](#top) ![i](img.png)",
            PATH,
            "abc",
        );
        assert!(
            html.contains("href=\"https://github.com/jimhillr/monorepo/blob/abc/pane/ISSUES.md\""),
            "{html}"
        );
        assert!(
            html.contains(
                "href=\"https://github.com/jimhillr/monorepo/blob/abc/pane/docs/plans/x.md#part\""
            ),
            "{html}"
        );
        assert!(
            html.contains("href=\"https://github.com/jimhillr/monorepo/blob/abc/AGENTS.md\""),
            "{html}"
        );
        assert!(html.contains("href=\"https://e.com\""), "{html}");
        assert!(html.contains("href=\"#top\""), "{html}");
        assert!(
            html.contains(
                "src=\"https://github.com/jimhillr/monorepo/raw/abc/pane/docs/plans/img.png\""
            ),
            "{html}"
        );
    }

    #[test]
    fn blocks_script_urls() {
        let html = to_html(
            "[x](javascript:alert(1)) [y](JavaScript:x) [z](data:text/html,x) [m](mailto:a@b.c)",
            "top.md",
            "c",
        );
        assert!(!html.to_lowercase().contains("javascript"), "{html}");
        assert!(!html.contains("data:"), "{html}");
        assert!(html.contains("href=\"mailto:a@b.c\""), "{html}");
    }
}

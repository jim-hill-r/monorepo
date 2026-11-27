//! The limit tables in `standards/docs/free-tiers.md`, generated from
//! `limits.toml`.

use crate::{Limit, LimitsFile, Scope};

/// One table per blip, in the order the blips first appear in the file.
pub fn render_limits(limits: &LimitsFile) -> String {
    let mut blips: Vec<&str> = Vec::new();
    for limit in &limits.limit {
        if !blips.contains(&limit.blip.as_str()) {
            blips.push(&limit.blip);
        }
    }
    let mut out = String::new();
    for blip in blips {
        out.push_str(&format!("#### {blip}\n\n"));
        out.push_str("| Item | Free limit | Our budget | Counted | Past the limit | Source |\n");
        out.push_str("| --- | --- | --- | --- | --- | --- |\n");
        for limit in limits.limit.iter().filter(|l| l.blip == blip) {
            out.push_str(&format!(
                "| {} (`{}`) | {} | {} | {} | {} | [{}]({}) |\n",
                cell(&limit.item),
                limit.key,
                amount(limit.free, limit.free_note.as_deref(), limit),
                amount(limit.budget, limit.budget_note.as_deref(), limit),
                match limit.scope {
                    Scope::Account => "per account",
                    Scope::Project => "per project",
                },
                cell(&limit.past),
                host(&limit.source),
                limit.source,
            ));
        }
        out.push('\n');
    }
    out
}

fn cell(s: &str) -> String {
    s.replace('|', "\\|")
}

fn amount(n: Option<u64>, note: Option<&str>, limit: &Limit) -> String {
    let number = n.map(|n| quantity(n, &limit.unit));
    match (number, note) {
        (Some(n), Some(note)) => format!("{n} ({})", cell(note)),
        (Some(n), None) => n,
        (None, Some(note)) => cell(note),
        (None, None) => "-".to_string(),
    }
}

fn quantity(n: u64, unit: &str) -> String {
    if unit == "bytes" {
        const MB: u64 = 1024 * 1024;
        const GB: u64 = 1024 * MB;
        if n >= GB && n.is_multiple_of(GB) {
            return format!("{} GB", n / GB);
        }
        if n >= MB && n.is_multiple_of(MB) {
            return format!("{} MB", n / MB);
        }
    }
    format!("{} {unit}", thousands(n))
}

fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn host(url: &str) -> &str {
    url.split("://")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .unwrap_or(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_limits;

    #[test]
    fn test_renders_a_table_per_blip() {
        let limits = parse_limits(
            r#"
[[limit]]
key = "github_cache_bytes"
blip = "GitHub"
item = "Actions cache"
free = 10737418240
free_note = "per repo"
budget = 8589934592
scope = "account"
unit = "bytes"
past = "Evicted"
source = "https://docs.github.com/billing"
checked = "2026-11-26"

[[limit]]
key = "pages_projects"
blip = "Cloudflare Pages"
item = "Pages projects"
free = 100000
scope = "project"
unit = "projects"
past = "Creation fails | no bill"
source = "https://developers.cloudflare.com/pages"
checked = "2026-11-26"
"#,
        )
        .expect("parses");
        let out = render_limits(&limits);
        assert!(out.starts_with("#### GitHub\n"), "{out}");
        assert!(out.contains("| Actions cache (`github_cache_bytes`) | 10 GB (per repo) | 8 GB | per account | Evicted | [docs.github.com](https://docs.github.com/billing) |"), "{out}");
        assert!(
            out.contains("| 100,000 projects | - | per project | Creation fails \\| no bill |"),
            "{out}"
        );
    }

    #[test]
    fn test_thousands() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1000), "1,000");
        assert_eq!(thousands(5000000), "5,000,000");
    }
}

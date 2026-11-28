//! `threat live`: the weekly live check's rules, run on what
//! `standards/scripts/threat-live.sh` fetched from Cloudflare and the
//! declared URLs.

use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result};
use threat_model::{
    Account, Model, Probe, THREAT_FILE, ThreatFile, check_live, parse_threat_file, render_live,
    targets,
};

use crate::check::projects;

/// Every project's parsed threat model; missing or broken files are the
/// audit's job (THR-001, THR-002), so they are skipped here.
fn models(root: &Path) -> Result<Vec<(String, ThreatFile)>> {
    let mut out = Vec::new();
    for name in projects(root)? {
        let Ok(text) = fs::read_to_string(root.join(&name).join(THREAT_FILE)) else {
            continue;
        };
        if let Ok(file) = parse_threat_file(&text) {
            out.push((name, file));
        }
    }
    Ok(out)
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

pub struct Outcome {
    /// The URLs to request (without probes) or the Markdown report.
    pub output: String,
    pub findings: usize,
}

/// Without `probes`, lists the URLs to request, one per line. With them,
/// renders the report and writes an issue file per orphan into
/// `issues_dir` when given.
pub fn live(
    root: &Path,
    account: &Path,
    probes: Option<&Path>,
    issues_dir: Option<&Path>,
    today: &str,
) -> Result<Outcome> {
    let account: Account = read_json(account)?;
    let parsed = models(root)?;
    let models: Vec<Model> = parsed
        .iter()
        .map(|(project, file)| Model { project, file })
        .collect();
    let targets = targets(&models, &account);
    let Some(probes) = probes else {
        let output = targets.iter().map(|t| format!("{}\n", t.url)).collect();
        return Ok(Outcome {
            output,
            findings: 0,
        });
    };
    let probes: Vec<Probe> = read_json(probes)?;
    let (findings, orphans) = check_live(&models, &account, &probes);
    if let Some(dir) = issues_dir {
        for o in &orphans {
            let path = dir.join(o.issue_path());
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, o.issue_text(today))
                .with_context(|| format!("writing {}", path.display()))?;
        }
    }
    Ok(Outcome {
        output: render_live(&account, targets.len(), &findings, &orphans, today),
        findings: findings.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, text: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        fs::write(path, text).expect("write");
    }

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        write(root, "app/mise.toml", "");
        write(root, "broken/mise.toml", "");
        write(root, "broken/docs/threat.toml", "nope");
        write(
            root,
            "app/docs/threat.toml",
            "reviewed = \"2026-11-27\"\nsummary = \"x\"\n[[surface]]\nname = \"site\"\nkind = \"pages\"\nurl = \"https://jimhillr-app.pages.dev\"\naccess = \"public\"\ndata = \"public\"\npreviews = \"public\"\n",
        );
        write(
            root,
            "account.json",
            r#"{"pages": [{"name": "jimhillr-app", "preview_url": "https://abc.jimhillr-app.pages.dev"}, {"name": "pane-dbc"}], "workers": []}"#,
        );
        write(
            root,
            "probes.json",
            r#"[{"url": "https://jimhillr-app.pages.dev", "status": 200, "headers": {"content-security-policy": "frame-ancestors 'none'", "x-content-type-options": "nosniff", "referrer-policy": "no-referrer", "strict-transport-security": "max-age=1"}},
                {"url": "https://abc.jimhillr-app.pages.dev", "status": 200, "location": null, "headers": {}}]"#,
        );
        dir
    }

    #[test]
    fn test_lists_urls_without_probes() {
        let dir = fixture();
        let root = dir.path();
        let out = live(root, &root.join("account.json"), None, None, "2026-11-30").expect("live");
        assert_eq!(
            out.output,
            "https://jimhillr-app.pages.dev\nhttps://abc.jimhillr-app.pages.dev\n"
        );
        assert_eq!(out.findings, 0);
    }

    #[test]
    fn test_reports_and_writes_orphan_issues() {
        let dir = fixture();
        let root = dir.path();
        let issues = root.join("out");
        let out = live(
            root,
            &root.join("account.json"),
            Some(&root.join("probes.json")),
            Some(&issues),
            "2026-11-30",
        )
        .expect("live");
        assert_eq!(out.findings, 2, "{}", out.output);
        assert!(out.output.contains("pane-dbc"), "{}", out.output);
        assert!(
            out.output
                .contains("preview https://abc.jimhillr-app.pages.dev is public but doesn't send"),
            "{}",
            out.output
        );
        let issue =
            fs::read_to_string(issues.join("threat/docs/issues/claim-or-delete-pages-pane-dbc.md"))
                .expect("issue file");
        assert!(issue.contains("added: 2026-11-30"), "{issue}");
    }

    #[test]
    fn test_bad_json_is_an_error() {
        let dir = fixture();
        let root = dir.path();
        write(root, "account.json", "{\"pages\": 3}");
        assert!(live(root, &root.join("account.json"), None, None, "2026-11-30").is_err());
    }
}
